//! C63 CLI contracts use actual captured native datasets, never alpha row DTOs.
use careops_ed::disposition::api_runner::{compute_sha256_hex, ExitCodeDef, ValidationError};
use careops_ed::disposition::native_output::read_native_dataset;
use careops_ed_cli::execute;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

const NOMINAL: &str = include_str!("../../careops-ed/examples/nominal_mvp.json");
const CONSTRAINED: &str = include_str!("../../careops-ed/examples/constrained_mvp.json");
const MALFORMED: &str = include_str!("../../careops-ed/examples/malformed_mvp.json");
struct Temp(tempfile::TempDir);
impl Temp {
    fn new() -> Self {
        let owner = {
            let mut builder = tempfile::Builder::new();
            builder.prefix("c63-native-");
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                builder.permissions(std::fs::Permissions::from_mode(0o700));
            }
            builder.tempdir().unwrap()
        };
        Self(owner)
    }
    fn source(&self, name: &str, text: &str) -> PathBuf {
        let path = self.0.path().join(name);
        fs::write(&path, text).unwrap();
        path
    }
    fn run(&self, source: &Path, name: &str, extra: &[&str]) -> PathBuf {
        let root = self.0.path().join(name);
        let mut args = vec![
            "run".into(),
            source.display().to_string(),
            "--output-dir".into(),
            root.display().to_string(),
        ];
        args.extend(extra.iter().map(|s| s.to_string()));
        invoke(&args).unwrap();
        root
    }
}

fn invoke(args: &[String]) -> Result<Value, ValidationError> {
    let mut stdout = Vec::new();
    execute(args, &mut stdout, None)?;
    Ok(serde_json::from_slice(&stdout).unwrap())
}
fn json(path: impl AsRef<Path>) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

#[test]
fn native_c63_manifest_lifecycle_and_real_format_contracts() {
    let temp = Temp::new();
    let source = temp.source("nominal.json", NOMINAL);
    let report = invoke(&["validate".into(), source.display().to_string()]).unwrap();
    assert_eq!(report["scenario_id"], "nominal-ed-mvp");
    assert_eq!(report["patient_count"], 5);
    assert_eq!(
        execute(&[], &mut Vec::new(), None).unwrap_err().code,
        ExitCodeDef::UsageError
    );
    let missing = invoke(&[
        "validate".into(),
        temp.0.path().join("missing.json").display().to_string(),
    ])
    .unwrap_err();
    assert_eq!(missing.code, ExitCodeDef::FileNotFound);
    assert!(!missing.message.is_empty());

    let root = temp.run(&source, "nominal", &["--seed", "42", "--horizon", "30"]);
    let invocation = json(root.join("invocation.json"));
    let data = read_native_dataset(&root.join("data")).unwrap();
    assert_eq!(
        invocation["raw_input_sha256"],
        compute_sha256_hex(NOMINAL.as_bytes())
    );
    assert_eq!(data.manifest().scenario_id, "nominal-ed-mvp");
    assert_eq!(data.manifest().base_seed, 42);
    assert_eq!(data.manifest().time_unit, "tick");
    assert_eq!(data.manifest().horizon_ticks, 30);
    assert_eq!(data.manifest().schema, "careops.ed.native-output.v1");
    let before = fs::read(root.join("manifest.json")).unwrap();
    let args = vec![
        "run".into(),
        source.display().to_string(),
        "--output-dir".into(),
        root.display().to_string(),
    ];
    assert_eq!(
        invoke(&args).unwrap_err().code,
        ExitCodeDef::FileAlreadyExists
    );
    let mut retired = args.clone();
    retired.push("--overwrite".into());
    assert_eq!(invoke(&retired).unwrap_err().code, ExitCodeDef::UsageError);
    assert_eq!(fs::read(root.join("manifest.json")).unwrap(), before);
    temp.run(&source, "fresh", &[]);
    let candidate = temp.run(
        &temp.source("constrained.json", CONSTRAINED),
        "candidate",
        &["--seed", "42"],
    );
    assert_eq!(
        read_native_dataset(&candidate.join("data"))
            .unwrap()
            .manifest()
            .scenario_id,
        "constrained-ed-mvp"
    );
    let comparison = invoke(&[
        "compare".into(),
        root.join("data").display().to_string(),
        candidate.join("data").display().to_string(),
    ])
    .unwrap();
    assert_eq!(comparison["schema"], "careops.ed.native-comparison.v1");
    assert_eq!(comparison["metrics"].as_array().unwrap().len(), 5);
    let wait = comparison["metrics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["name"] == "mean_wait_ticks")
        .unwrap();
    assert_eq!(wait["unit"], "tick");
    let metrics = data.metrics();
    let row = (0..metrics.height())
        .find(|&i| metrics.column("name").unwrap().str().unwrap().get(i) == Some("mean_wait_ticks"))
        .unwrap();
    assert_eq!(
        metrics
            .column("denominator_name")
            .unwrap()
            .str()
            .unwrap()
            .get(row),
        Some("included_replications")
    );

    for format in ["csv", "ipc", "parquet"] {
        let output = temp.0.path().join(format);
        invoke(&[
            "export".into(),
            root.join("data").display().to_string(),
            "--format".into(),
            format.into(),
            "--output-dir".into(),
            output.display().to_string(),
        ])
        .unwrap();
        let exported = read_native_dataset(&output).unwrap();
        assert!(data.patients().equals_missing(exported.patients()));
        assert!(data.replications().equals_missing(exported.replications()));
        assert!(data.metrics().equals_missing(exported.metrics()));
        assert_eq!(
            exported
                .manifest()
                .tables
                .iter()
                .map(|t| t.name.as_str())
                .collect::<Vec<_>>(),
            ["patients", "replications", "metrics"]
        );
        assert_eq!(
            exported
                .patients()
                .column("arrival_tick_lo")
                .unwrap()
                .u64()
                .unwrap()
                .null_count(),
            0
        );
        for part in exported.manifest().tables.iter().flat_map(|t| &t.parts) {
            assert_eq!(
                compute_sha256_hex(&fs::read(output.join(&part.file)).unwrap()),
                part.sha256
            );
        }
    }
    // Invalid closed schemas/control-only unit edits are malformed, not valid
    // incompatible datasets. The genuine rehashed-unit exit5 oracle is retained
    // in native_cli::native_consumer_arguments_fail_before_io_and_compare_refuses_different_units.
    let manifest_path = root.join("data/manifest.json");
    let original = fs::read(&manifest_path).unwrap();
    for (field, value) in [
        ("schema", "careops.ed.native-output.v99"),
        ("time_unit", "second"),
    ] {
        let mut manifest: Value = serde_json::from_slice(&original).unwrap();
        manifest[field] = value.into();
        fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();
        assert_eq!(
            invoke(&[
                "compare".into(),
                root.join("data").display().to_string(),
                candidate.join("data").display().to_string()
            ])
            .unwrap_err()
            .code,
            ExitCodeDef::MalformedConfig
        );
    }
}

#[test]
fn native_c63_invalid_inputs_reject_before_output() {
    let temp = Temp::new();
    let source = temp.source("nominal.json", NOMINAL);
    let mut zero: Value = serde_json::from_str(NOMINAL).unwrap();
    zero["horizon_ticks"] = 0.into();
    let zero = temp.source("zero.json", &serde_json::to_string(&zero).unwrap());
    assert_eq!(
        invoke(&["validate".into(), zero.display().to_string()])
            .unwrap_err()
            .code,
        ExitCodeDef::MalformedConfig
    );
    let output = temp.0.path().join("zero-output");
    assert_eq!(
        invoke(&[
            "run".into(),
            source.display().to_string(),
            "--horizon".into(),
            "0".into(),
            "--output-dir".into(),
            output.display().to_string()
        ])
        .unwrap_err()
        .code,
        ExitCodeDef::MalformedConfig
    );
    assert!(!output.exists());
    let malformed = temp.source("malformed.json", MALFORMED);
    assert_eq!(
        invoke(&["validate".into(), malformed.display().to_string()])
            .unwrap_err()
            .code,
        ExitCodeDef::MalformedConfig
    );
    let valid = temp.run(&source, "valid", &[]);
    assert_eq!(
        invoke(&[
            "compare".into(),
            malformed.display().to_string(),
            valid.join("data").display().to_string()
        ])
        .unwrap_err()
        .code,
        ExitCodeDef::MalformedConfig
    );
    let destination = temp.0.path().join("bad-export");
    assert_eq!(
        invoke(&[
            "export".into(),
            malformed.display().to_string(),
            "--output-dir".into(),
            destination.display().to_string()
        ])
        .unwrap_err()
        .code,
        ExitCodeDef::MalformedConfig
    );
    assert!(!destination.exists());
}

#[test]
fn native_c63_actual_thousand_patient_comparison_and_export_latency() {
    let temp = Temp::new();
    let mut scenario: Value = serde_json::from_str(NOMINAL).unwrap();
    scenario["scenario_id"] = "scale-1000".into();
    scenario["horizon_ticks"] = 2050.into();
    let patient = scenario["patients"][0].clone();
    scenario["patients"] = (0..1000)
        .map(|i| {
            let mut row = patient.clone();
            row["patient_id"] = format!("p_{i:04}").into();
            row["arrival_tick"] = (i as u64).into();
            row["work_ticks"] = 2.into();
            row
        })
        .collect();
    let source = temp.source("base.json", &serde_json::to_string(&scenario).unwrap());
    let base = temp.run(&source, "base", &["--seed", "42"]);
    scenario["scenario_id"] = "scale-1000-cand".into();
    for row in scenario["patients"].as_array_mut().unwrap() {
        row["work_ticks"] = 3.into();
    }
    let source = temp.source("candidate.json", &serde_json::to_string(&scenario).unwrap());
    let candidate = temp.run(&source, "candidate", &["--seed", "999"]);
    let start = Instant::now();
    let comparison = invoke(&[
        "compare".into(),
        base.join("data").display().to_string(),
        candidate.join("data").display().to_string(),
    ])
    .unwrap();
    let elapsed = start.elapsed();
    println!("Actual 1000-patient native comparison (including file validation): {elapsed:?}");
    assert!(
        elapsed.as_millis() < 100,
        "1000-patient comparison must remain under100ms: {elapsed:?}"
    );
    assert_eq!(comparison["metrics"].as_array().unwrap().len(), 5);
    let elapsed_metric = comparison["metrics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["name"] == "mean_elapsed_ticks")
        .unwrap();
    assert!(elapsed_metric["delta"].as_f64().unwrap() > 0.0);
    let export = temp.0.path().join("export");
    let start = Instant::now();
    let report = invoke(&[
        "export".into(),
        base.join("data").display().to_string(),
        "--output-dir".into(),
        export.display().to_string(),
    ])
    .unwrap();
    println!("Actual 1000-patient native export: {:?}", start.elapsed());
    assert_eq!(report["schema"], "careops.ed.cli-export.v1");
    assert_eq!(
        read_native_dataset(&export).unwrap().patients().height(),
        1000
    );
}
