use std::process::Command;

#[test]
fn e23_cli_runs_nominal_scenario_without_error() {
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_careops-ed"))
        .current_dir(workspace_root)
        .arg("run")
        .arg("configs/scenarios/e23_nominal.json")
        .output()
        .expect("careops-ed binary executes");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let doc: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    assert_eq!(doc["manifest"]["scenario_id"], "e23-nominal-mvp");
    assert_eq!(doc["summary"]["scenario_id"], "e23-nominal-mvp");
    assert_eq!(doc["summary"]["horizon_ticks"], 20);
}

#[test]
fn e23_cli_runs_constrained_scenario_without_error() {
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_careops-ed"))
        .current_dir(workspace_root)
        .arg("run")
        .arg("configs/scenarios/e23_constrained.json")
        .output()
        .expect("careops-ed binary executes");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let doc: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    assert_eq!(doc["manifest"]["scenario_id"], "e23-constrained-mvp");
    assert_eq!(doc["summary"]["scenario_id"], "e23-constrained-mvp");
}

#[test]
fn e23_cli_rejects_malformed_input_with_clear_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_careops-ed"))
        .arg("run")
        .arg("non_existent_file.json")
        .output()
        .expect("careops-ed binary executes");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("careops-ed:"));
}

#[test]
fn e23_cli_runs_repeatable_under_identical_seed() {
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let run1 = Command::new(env!("CARGO_BIN_EXE_careops-ed"))
        .current_dir(workspace_root)
        .arg("run")
        .arg("configs/scenarios/e23_nominal.json")
        .output()
        .expect("careops-ed binary executes");

    let run2 = Command::new(env!("CARGO_BIN_EXE_careops-ed"))
        .current_dir(workspace_root)
        .arg("run")
        .arg("configs/scenarios/e23_nominal.json")
        .output()
        .expect("careops-ed binary executes");

    assert_eq!(run1.stdout, run2.stdout);
}
