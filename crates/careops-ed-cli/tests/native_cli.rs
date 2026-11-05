use careops_ed::disposition::api_runner::{compute_sha256_hex, CancellationToken, ExitCodeDef};
use careops_ed_cli::{execute, Completion, Invocation};
use std::io::{self, Write};
use std::path::Path;
use std::sync::atomic::Ordering;
struct Temp(tempfile::TempDir);
impl Temp {
    fn new() -> Self {
        let owner = {
            let mut builder = tempfile::Builder::new();
            builder.prefix("native-cli-");
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                builder.permissions(std::fs::Permissions::from_mode(0o700));
            }
            builder.tempdir().unwrap()
        };
        std::fs::write(
            owner.path().join("scenario.json"),
            include_str!("../../careops-ed/examples/nominal_mvp.json"),
        )
        .unwrap();
        Self(owner)
    }
    fn args(&self, command: &str, name: &str, extra: &[&str]) -> Vec<String> {
        let mut args = vec![
            command.into(),
            self.0.path().join("scenario.json").to_str().unwrap().into(),
            "--output-dir".into(),
            self.0.path().join(name).to_str().unwrap().into(),
        ];
        args.extend(extra.iter().map(|s| s.to_string()));
        args
    }
}

fn json(path: impl AsRef<Path>) -> serde_json::Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}
fn interrupt_after_actual_checkpoint(temp: &Temp, name: &str) {
    let mut source = json(temp.0.path().join("scenario.json"));
    let template = source["patients"][0].clone();
    source["patients"] = (0..2000)
        .map(|index| {
            let mut row = template.clone();
            row["patient_id"] = format!("cancel-patient-{index:05}").into();
            row
        })
        .collect();
    std::fs::write(
        temp.0.path().join("scenario.json"),
        serde_json::to_vec(&source).unwrap(),
    )
    .unwrap();
    let cancel = CancellationToken::new();
    let args = temp.args("batch", name, &["--horizon", "8", "--workers", "1"]);
    let checkpoint = temp.0.path().join(name).join("capture/checkpoint.json");
    let finished = std::sync::atomic::AtomicBool::new(false);
    std::thread::scope(|scope| {
        let worker = scope.spawn(|| {
            let result = execute(&args, &mut Vec::new(), Some(&cancel));
            finished.store(true, Ordering::SeqCst);
            result
        });
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while !checkpoint.exists()
            && !finished.load(Ordering::SeqCst)
            && std::time::Instant::now() < deadline
        {
            std::thread::yield_now();
        }
        cancel.cancel();
        let result = worker.join().unwrap();
        assert!(
            checkpoint.is_file(),
            "actual controller never committed checkpoint: {result:?}"
        );
        assert_eq!(result.unwrap_err().code, ExitCodeDef::Interrupted);
    });
}
#[test]
fn run_publishes_native_parquet_and_small_complete_controls() {
    let temp = Temp::new();
    let output = temp.0.path().join("output");
    let mut stdout = Vec::new();
    execute(
        &temp.args("run", "output", &["--horizon", "8"]),
        &mut stdout,
        None,
    )
    .unwrap();
    let completion: Completion = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(completion.schema, "careops.ed.cli-artifacts.v1");
    assert_eq!(completion.status, "complete");
    assert_eq!(
        completion.invocation_sha256,
        compute_sha256_hex(&std::fs::read(output.join("invocation.json")).unwrap())
    );
    assert_eq!(
        completion.batch_sha256,
        compute_sha256_hex(&std::fs::read(output.join("batch_run.json")).unwrap())
    );
    assert_eq!(
        completion.dataset_manifest_sha256,
        compute_sha256_hex(&std::fs::read(output.join("data/manifest.json")).unwrap())
    );
    let manifest = json(output.join("data/manifest.json"));
    let invocation: Invocation =
        serde_json::from_slice(&std::fs::read(output.join("invocation.json")).unwrap()).unwrap();
    assert_eq!(manifest["format"], "parquet");
    assert_eq!(manifest["tables"][0]["rows"], 5);
    assert_eq!(manifest["horizon_ticks"], 8);
    assert_eq!(
        manifest["parsed_input_sha256"],
        invocation.parsed_input_sha256
    );
    assert_eq!(
        manifest["model_contract_sha256"],
        invocation.model_contract_sha256
    );
    assert_ne!(
        manifest["effective_input_sha256"],
        manifest["parsed_input_sha256"]
    );
    assert!(manifest["raw_input_sha256"].is_null());
    assert_eq!(
        invocation.raw_input_sha256,
        compute_sha256_hex(&std::fs::read(output.join("input.json")).unwrap())
    );
    assert_eq!(
        std::fs::read(output.join("input.json")).unwrap(),
        std::fs::read(temp.0.path().join("scenario.json")).unwrap()
    );
    assert!(!json(output.join("batch_run.json"))
        .to_string()
        .contains("\"patients\""));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&output).unwrap().permissions().mode() & 0o777,
            0o700
        );
    }
}
#[test]
fn explicit_csv_stream_uses_real_quoted_patient_data_not_a_json_bundle() {
    let temp = Temp::new();
    let mut source = json(temp.0.path().join("scenario.json"));
    source["patients"][0]["patient_id"] = "case,\"quote\r\nline ☃".into();
    std::fs::write(
        temp.0.path().join("scenario.json"),
        serde_json::to_vec(&source).unwrap(),
    )
    .unwrap();
    let mut stdout = Vec::new();
    execute(
        &temp.args("run", "csv", &["--format", "csv", "--stdout"]),
        &mut stdout,
        None,
    )
    .unwrap();
    assert!(stdout.starts_with(b"time_unit,schema,"));
    let text = String::from_utf8(stdout).unwrap();
    assert!(text.contains("\"case,\"\"quote\r\nline ☃\""));
    assert!(!text.contains("cli-artifacts.v1"));
    assert_eq!(
        json(temp.0.path().join("csv/data/manifest.json"))["format"],
        "csv"
    );
}
#[test]
fn required_capabilities_and_invalid_requests_fail_before_input_or_output_io() {
    let temp = Temp::new();
    for command in ["run", "batch"] {
        for backend in ["metal", "pdes", "grpc", "mpi", "distributed"] {
            let root = temp.0.path().join(format!("{command}-{backend}"));
            let mut stdout = Vec::new();
            let args = vec![
                command.into(),
                temp.0.path().join("absent.json").to_str().unwrap().into(),
                "--output-dir".into(),
                root.to_str().unwrap().into(),
                "--backend".into(),
                backend.into(),
            ];
            let error = execute(&args, &mut stdout, None).unwrap_err();
            assert_eq!(error.code, ExitCodeDef::SimulationFailed);
            assert!(!root.exists());
            assert!(stdout.is_empty());
        }
    }
    for extra in [
        vec!["--backend", "cuda"],
        vec!["--backend", "cpu_serial", "--backend", "cpu_serial"],
        vec!["--format", "json"],
        vec!["--stdout"],
        vec!["--overwrite"],
        vec!["--backend"],
    ] {
        let mut stdout = Vec::new();
        assert_eq!(
            execute(&temp.args("run", "invalid", &extra), &mut stdout, None)
                .unwrap_err()
                .code,
            ExitCodeDef::UsageError
        );
        assert!(!temp.0.path().join("invalid").exists());
    }
    let enormous = usize::MAX.to_string();
    for flag in ["--replications", "--workers", "--checkpoint"] {
        let mut stdout = Vec::new();
        assert_eq!(
            execute(
                &temp.args("batch", "huge", &[flag, &enormous]),
                &mut stdout,
                None
            )
            .unwrap_err()
            .code,
            ExitCodeDef::UsageError
        );
        assert!(!temp.0.path().join("huge").exists());
    }
}
#[test]
fn malformed_oversized_and_unqualified_identifiers_create_no_root() {
    let temp = Temp::new();
    let path = temp.0.path().join("scenario.json");
    std::fs::write(&path, b"not json").unwrap();
    assert_eq!(
        execute(&temp.args("run", "bad", &[]), &mut Vec::new(), None)
            .unwrap_err()
            .code,
        ExitCodeDef::MalformedConfig
    );
    assert!(!temp.0.path().join("bad").exists());
    std::fs::File::create(&path)
        .unwrap()
        .set_len(careops_ed::disposition::api_runner::MAX_SCENARIO_PAYLOAD_BYTES as u64 + 1)
        .unwrap();
    assert_eq!(
        execute(&temp.args("run", "large", &[]), &mut Vec::new(), None)
            .unwrap_err()
            .code,
        ExitCodeDef::MalformedConfig
    );
    assert!(!temp.0.path().join("large").exists());
    let mut source: serde_json::Value =
        serde_json::from_str(include_str!("../../careops-ed/examples/nominal_mvp.json")).unwrap();
    source["patients"][0]["patient_id"] = "x".repeat(4097).into();
    std::fs::write(&path, serde_json::to_vec(&source).unwrap()).unwrap();
    assert_eq!(
        execute(&temp.args("run", "identifier", &[]), &mut Vec::new(), None)
            .unwrap_err()
            .code,
        ExitCodeDef::MalformedConfig
    );
    assert!(!temp.0.path().join("identifier").exists());
}
#[test]
fn existing_roots_are_never_overwritten() {
    let temp = Temp::new();
    let root = temp.0.path().join("owned");
    std::fs::create_dir(&root).unwrap();
    std::fs::write(root.join("owned"), b"preserve").unwrap();
    assert_eq!(
        execute(&temp.args("run", "owned", &[]), &mut Vec::new(), None)
            .unwrap_err()
            .code,
        ExitCodeDef::FileAlreadyExists
    );
    assert_eq!(std::fs::read(root.join("owned")).unwrap(), b"preserve");
    assert_eq!(std::fs::read_dir(root).unwrap().count(), 1);
}
#[test]
fn original_capture_resumes_with_exact_invocation_and_different_worker_budget() {
    let temp = Temp::new();
    interrupt_after_actual_checkpoint(&temp, "resume");
    let root = temp.0.path().join("resume");
    assert!(root.join("capture/checkpoint.json").is_file());
    assert!(!root.join("data").exists());
    let immutable = std::fs::read(root.join("invocation.json")).unwrap();
    let checkpoint = root.join("capture/checkpoint.json");
    let mut stdout = Vec::new();
    execute(
        &temp.args(
            "batch",
            "resume",
            &["--resume", checkpoint.to_str().unwrap(), "--workers", "2"],
        ),
        &mut stdout,
        None,
    )
    .unwrap();
    assert_eq!(
        std::fs::read(root.join("invocation.json")).unwrap(),
        immutable
    );
    assert_eq!(json(root.join("batch_run.json"))["num_workers"], 2);
    assert_eq!(
        json(root.join("data/manifest.json"))["tables"][0]["rows"],
        8000
    );
    assert_eq!(
        execute(
            &temp.args(
                "batch",
                "resume",
                &["--resume", checkpoint.to_str().unwrap()]
            ),
            &mut Vec::new(),
            None
        )
        .unwrap_err()
        .code,
        ExitCodeDef::FileAlreadyExists
    );
}
#[test]
fn changed_invocation_checkpoint_limits_and_export_retries_refuse_before_recovery() {
    for change in ["invocation", "limits", "export", "seed"] {
        let temp = Temp::new();
        interrupt_after_actual_checkpoint(&temp, "resume");
        let root = temp.0.path().join("resume");
        let path = root.join("capture/checkpoint.json");
        if change == "invocation" {
            let mut value = json(root.join("invocation.json"));
            value["format"] = "csv".into();
            std::fs::write(
                root.join("invocation.json"),
                serde_json::to_vec(&value).unwrap(),
            )
            .unwrap();
        }
        if change == "limits" {
            let mut value = json(&path);
            value["limits"]["max_bytes"] = usize::MAX.into();
            std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
        }
        if change == "export" {
            std::fs::create_dir(root.join("data")).unwrap();
        }
        let before = std::fs::read(&path).unwrap();
        let mut args = temp.args("batch", "resume", &["--resume", path.to_str().unwrap()]);
        if change == "seed" {
            args.extend(["--seed".into(), "99".into()]);
        }
        assert_eq!(
            execute(&args, &mut Vec::new(), None).unwrap_err().code,
            ExitCodeDef::MalformedConfig,
            "{change}"
        );
        assert_eq!(std::fs::read(path).unwrap(), before);
        assert!(!root.join("manifest.json").exists());
    }
}
struct FailedSink {
    flush_only: bool,
}
impl Write for FailedSink {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.flush_only {
            Ok(bytes.len())
        } else {
            Err(io::Error::other("sink write failed"))
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::other("sink flush failed"))
    }
}
#[test]
fn real_sink_write_and_flush_errors_never_return_cli_success() {
    let temp = Temp::new();
    for (name, flush_only, extra) in [
        ("write", false, vec!["--format", "csv", "--stdout"]),
        ("flush", true, vec![]),
    ] {
        let mut sink = FailedSink { flush_only };
        assert_eq!(
            execute(&temp.args("run", name, &extra), &mut sink, None)
                .unwrap_err()
                .code,
            ExitCodeDef::OutputWriteFailed
        );
        assert_eq!(
            json(temp.0.path().join(name).join("manifest.json"))["status"],
            "complete"
        );
    }
}
#[test]
fn csv_stream_cap_is_enforced_by_the_actual_writer() {
    let temp = Temp::new();
    execute(&temp.args("run", "stream", &[]), &mut Vec::new(), None).unwrap();
    let scenario =
        careops_ed::parse_scenario_json(include_str!("../../careops-ed/examples/nominal_mvp.json"))
            .unwrap();
    let document = careops_ed::disposition::batch_runner::load_completed_batch(
        &temp.0.path().join("stream/batch_run.json"),
    )
    .unwrap();
    let error = careops_ed::disposition::native_output::stream_patient_csv(
        &scenario,
        &document,
        &mut Vec::new(),
        16,
    )
    .unwrap_err();
    assert_eq!(error.code, ExitCodeDef::OutputWriteFailed);
}

#[test]
fn pre_dispatch_cancellation_neither_reads_input_nor_creates_artifacts() {
    let temp = Temp::new();
    std::fs::remove_file(temp.0.path().join("scenario.json")).unwrap();
    let cancel = CancellationToken::new();
    cancel.cancel();
    let mut stdout = Vec::new();
    let error = execute(
        &temp.args("run", "cancelled", &[]),
        &mut stdout,
        Some(&cancel),
    )
    .unwrap_err();
    assert_eq!(error.code, ExitCodeDef::Interrupted);
    assert!(!temp.0.path().join("cancelled").exists());
    assert!(stdout.is_empty());
}

#[test]
fn native_export_and_compare_consume_actual_typed_outputs() {
    let temp = Temp::new();
    execute(
        &temp.args("run", "left", &["--horizon", "8"]),
        &mut Vec::new(),
        None,
    )
    .unwrap();
    let source = temp.0.path().join("left/data");
    let converted = temp.0.path().join("converted");
    let mut declaration = Vec::new();
    execute(
        &[
            "export".into(),
            source.display().to_string(),
            "--output-dir".into(),
            converted.display().to_string(),
            "--format".into(),
            "ipc".into(),
        ],
        &mut declaration,
        None,
    )
    .unwrap();
    assert_eq!(json(converted.join("manifest.json"))["format"], "ipc");
    assert!(converted.join("conversion.json").is_file());
    let declaration: serde_json::Value = serde_json::from_slice(&declaration).unwrap();
    assert_eq!(declaration["schema"], "careops.ed.cli-export.v1");
    assert_eq!(
        declaration
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>(),
        [
            "schema",
            "status",
            "format",
            "source_manifest_sha256",
            "dataset_manifest",
            "dataset_manifest_sha256",
            "conversion_receipt",
            "conversion_receipt_sha256"
        ]
        .into_iter()
        .map(String::from)
        .collect()
    );

    let mut report = Vec::new();
    execute(
        &[
            "compare".into(),
            source.display().to_string(),
            converted.display().to_string(),
        ],
        &mut report,
        None,
    )
    .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&report).unwrap();
    assert_eq!(report["schema"], "careops.ed.native-comparison.v1");
    assert_eq!(report["uncertainty_method"], "not_estimated");
    assert_eq!(report["metrics"].as_array().unwrap().len(), 5);
    for metric in report["metrics"].as_array().unwrap() {
        assert!(metric["delta"].is_null() || metric["delta"] == 0.0);
    }
    assert_eq!(
        execute(
            &[
                "export".into(),
                source.display().to_string(),
                "--output-dir".into(),
                converted.display().to_string()
            ],
            &mut Vec::new(),
            None
        )
        .unwrap_err()
        .code,
        ExitCodeDef::FileAlreadyExists
    );
}

#[test]
fn native_consumer_arguments_fail_before_io_and_compare_refuses_different_units() {
    let temp = Temp::new();
    let absent = temp.0.path().join("absent").display().to_string();
    let destination = temp.0.path().join("unused").display().to_string();
    for extra in [
        vec!["--seed", "1"],
        vec!["--horizon", "0"],
        vec!["--backend", "metal"],
        vec!["--stdout"],
    ] {
        let mut args = vec![
            "export".into(),
            absent.clone(),
            "--output-dir".into(),
            destination.clone(),
        ];
        args.extend(extra.into_iter().map(String::from));
        assert_eq!(
            execute(&args, &mut Vec::new(), None).unwrap_err().code,
            ExitCodeDef::UsageError
        );
        assert!(!temp.0.path().join("unused").exists());
    }
    execute(
        &temp.args("run", "ticks", &["--horizon", "8"]),
        &mut Vec::new(),
        None,
    )
    .unwrap();
    execute(
        &temp.args("run", "seconds", &["--horizon", "8", "--format", "csv"]),
        &mut Vec::new(),
        None,
    )
    .unwrap();
    let manifest_path = temp.0.path().join("seconds/data/manifest.json");
    let mut manifest = json(&manifest_path);
    manifest["time_unit"] = "second".into();
    for part in manifest["tables"][0]["parts"].as_array_mut().unwrap() {
        let path = temp
            .0
            .path()
            .join("seconds/data")
            .join(part["file"].as_str().unwrap());
        let text = std::fs::read_to_string(&path)
            .unwrap()
            .replace("tick,", "second,");
        std::fs::write(&path, text.as_bytes()).unwrap();
        part["bytes"] = text.len().into();
        part["sha256"] = compute_sha256_hex(text.as_bytes()).into();
    }
    std::fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();
    let mut stdout = Vec::new();
    assert_eq!(
        execute(
            &[
                "compare".into(),
                temp.0.path().join("ticks/data").display().to_string(),
                temp.0.path().join("seconds/data").display().to_string()
            ],
            &mut stdout,
            None
        )
        .unwrap_err()
        .code,
        ExitCodeDef::IncompatibleScenario
    );
    assert!(stdout.is_empty());
}

#[test]
fn compare_rejects_same_unqualified_metric_in_both_rehashed_native_sources() {
    let temp = Temp::new();
    execute(
        &temp.args("run", "dataset", &["--horizon", "8", "--format", "csv"]),
        &mut Vec::new(),
        None,
    )
    .unwrap();
    let root = temp.0.path().join("dataset/data");
    let path = root.join("manifest.json");
    let mut manifest = json(&path);
    for part in manifest["tables"][2]["parts"].as_array_mut().unwrap() {
        let file = root.join(part["file"].as_str().unwrap());
        let text = std::fs::read_to_string(&file)
            .unwrap()
            .replace("mean_wait_ticks", "unqualified_metric");
        std::fs::write(&file, text.as_bytes()).unwrap();
        part["bytes"] = text.len().into();
        part["sha256"] = compute_sha256_hex(text.as_bytes()).into();
    }
    std::fs::write(&path, serde_json::to_vec(&manifest).unwrap()).unwrap();
    let mut stdout = Vec::new();
    assert_eq!(
        execute(
            &[
                "compare".into(),
                root.display().to_string(),
                root.display().to_string()
            ],
            &mut stdout,
            None
        )
        .unwrap_err()
        .code,
        ExitCodeDef::MalformedConfig
    );
    assert!(stdout.is_empty());
}
