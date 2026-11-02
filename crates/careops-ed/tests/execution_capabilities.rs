//! Executable ED modes must not be inferred from upstream features or fixture labels.
use careops_ed::disposition::api_runner::execute_cli_args;

fn call(args: &[&str]) -> careops_ed::disposition::api_runner::CliOutput {
    execute_cli_args(&args.iter().map(|s| s.to_string()).collect::<Vec<_>>())
}

#[test]
fn capability_report_distinguishes_replications_from_within_run_execution() {
    let out = call(&["capabilities"]);
    assert_eq!(out.exit_code as i32, 0);
    let report: serde_json::Value = serde_json::from_str(&out.stdout).unwrap();
    assert_eq!(report["schema_version"], 1);
    assert_eq!(
        report["scope"],
        "careops-ed executable modes; no hardware probe"
    );
    let modes = report["modes"].as_array().unwrap();
    let available: Vec<_> = modes
        .iter()
        .filter(|m| m["available"] == true)
        .map(|m| m["id"].as_str().unwrap())
        .collect();
    assert_eq!(available, ["cpu_serial", "cpu_replications"]);
    assert_eq!(modes[1]["parallelism"], "independent_replications");
    for id in ["metal", "pdes", "grpc", "mpi", "distributed"] {
        let mode = modes.iter().find(|m| m["id"] == id).unwrap();
        assert_eq!(mode["available"], false);
        assert!(!mode["reason"].as_str().unwrap().is_empty());
        let required = call(&["capabilities", "--require", id]);
        assert_eq!(required.exit_code as i32, 8, "{id}: {}", required.stderr);
    }
}

#[test]
fn capability_requirements_reject_unknown_and_malformed_requests() {
    for mode in ["cpu_serial", "cpu_replications"] {
        assert_eq!(
            call(&["capabilities", "--require", mode]).exit_code as i32,
            0
        );
    }
    for args in [
        vec!["capabilities", "--require"],
        vec!["capabilities", "--require", "cuda"],
        vec!["capabilities", "--unexpected"],
        vec!["capabilities", "--require", "cpu_serial", "extra"],
    ] {
        assert_eq!(call(&args).exit_code as i32, 2);
    }
}

#[test]
fn unsupported_execution_is_rejected_before_input_or_output_side_effects() {
    for command in ["run", "batch"] {
        for mode in ["metal", "pdes", "grpc", "mpi", "distributed"] {
            let out = call(&[command, "absent-input.json", "--backend", mode]);
            assert_eq!(out.exit_code as i32, 8, "{command}/{mode}: {}", out.stderr);
            assert!(out.stderr.contains("not integrated"));
            assert!(out.stdout.is_empty());
        }
    }
}

#[test]
fn backend_selection_is_strict_and_does_not_change_cpu_results() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let scenario = root.join("examples/nominal_mvp.json");
    let path = scenario.to_str().unwrap();
    let default = call(&["run", path]);
    let explicit = call(&["run", path, "--backend", "cpu_serial"]);
    assert_eq!(default.exit_code as i32, 0);
    assert_eq!(explicit.exit_code as i32, 0);
    assert_eq!(default.stdout, explicit.stdout);
    for args in [
        vec!["run", path, "--backend"],
        vec!["run", path, "--backend", "cuda"],
        vec!["run", path, "--backend", "metal", "--backend", "cpu_serial"],
        vec!["batch", path, "--backend", "metal", "--backend"],
        vec!["run", path, "--backend", "cuda", "--backend", "metal"],
        vec![
            "run",
            path,
            "--backend",
            "cpu_serial",
            "--backend",
            "cpu_serial",
        ],
    ] {
        assert_eq!(call(&args).exit_code as i32, 2);
    }
    assert_eq!(
        call(&["run", path, "--backend", "cpu_replications"]).exit_code as i32,
        2
    );
    assert_eq!(
        call(&["batch", path, "--backend", "cpu_serial"]).exit_code as i32,
        2
    );
    assert!(call(&["--help"]).stdout.contains("capabilities"));
}

#[test]
fn explicit_cpu_batch_runs_and_unavailable_modes_create_no_artifacts() {
    let scenario =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/nominal_mvp.json");
    let scenario = scenario.to_str().unwrap();
    let root = std::env::temp_dir().join(format!(
        "careops_capabilities_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    for (command, output_flag) in [("run", "--output"), ("batch", "--output-dir")] {
        let output = root.join(command);
        let out = call(&[
            command,
            scenario,
            output_flag,
            output.to_str().unwrap(),
            "--backend",
            "metal",
        ]);
        assert_eq!(out.exit_code as i32, 8);
        assert!(
            !root.exists(),
            "unsupported backend must not create outputs"
        );
    }
    let output = root.join("batch");
    let out = call(&[
        "batch",
        scenario,
        "--replications",
        "2",
        "--workers",
        "2",
        "--output-dir",
        output.to_str().unwrap(),
        "--backend",
        "cpu_replications",
    ]);
    assert_eq!(out.exit_code as i32, 0, "{}", out.stderr);
    assert!(output.join("batch_run.json").is_file());
    std::fs::remove_dir_all(root).unwrap();
}
