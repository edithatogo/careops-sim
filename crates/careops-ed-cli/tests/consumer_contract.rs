//! Native consumer cutover for the old E31/E43 CLI lifecycle assertions.
//! Actual write/flush failures, cancellation and recovery remain in native_cli.rs.
use careops_ed::disposition::api_runner::ExitCodeDef;
use careops_ed::disposition::native_output::read_native_dataset;
use careops_ed_cli::execute;
use serde_json::Value;
use std::path::{Path, PathBuf};
struct Temp(tempfile::TempDir);
impl Temp {
    fn new() -> Self {
        let owner = {
            let mut builder = tempfile::Builder::new();
            builder.prefix("cli-consumer-");
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
        std::fs::write(&path, text).unwrap();
        path
    }
    fn run(&self, source: &Path, name: &str, extra: &[&str]) -> PathBuf {
        let output = self.0.path().join(name);
        let mut args = vec![
            "run".into(),
            source.display().to_string(),
            "--output-dir".into(),
            output.display().to_string(),
        ];
        args.extend(extra.iter().map(|s| s.to_string()));
        invoke(&args).unwrap();
        output.join("data")
    }
}

fn invoke(args: &[String]) -> Result<Value, careops_ed::disposition::api_runner::ValidationError> {
    let mut output = Vec::new();
    execute(args, &mut output, None)?;
    Ok(serde_json::from_slice(&output).unwrap())
}
fn compare(a: &Path, b: &Path) -> Value {
    invoke(&[
        "compare".into(),
        a.display().to_string(),
        b.display().to_string(),
    ])
    .unwrap()
}
const NOMINAL: &str = include_str!("../../careops-ed/examples/nominal_mvp.json");
const CONSTRAINED: &str = include_str!("../../careops-ed/examples/constrained_mvp.json");
#[test]
fn native_run_compare_export_lifecycle_and_signed_deltas() {
    let temp = Temp::new();
    let nominal = temp.source("nominal.json", NOMINAL);
    let constrained = temp.source("constrained.json", CONSTRAINED);
    let a = temp.run(&nominal, "nominal", &[]);
    let b = temp.run(&constrained, "constrained", &[]);
    let data = read_native_dataset(&a).unwrap();
    assert_eq!(data.patients().height(), 5);
    assert_eq!(
        data.replications()
            .column("arrivals")
            .unwrap()
            .u64()
            .unwrap()
            .get(0),
        Some(5)
    );
    let report = compare(&a, &b);
    assert_eq!(report["schema"], "careops.ed.native-comparison.v1");
    assert_eq!(report["uncertainty_method"], "not_estimated");
    assert_eq!(
        report["uncertainty_reason"],
        "paired_difference_method_unqualified"
    );
    let wait = report["metrics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["name"] == "mean_wait_ticks")
        .unwrap();
    assert_eq!(wait["unit"], "tick");
    assert!((wait["left"].as_f64().unwrap() - 0.4).abs() < 1e-4);
    assert!((wait["right"].as_f64().unwrap() - 4.2).abs() < 1e-4);
    assert!((wait["delta"].as_f64().unwrap() - 3.8).abs() < 1e-4);
    let reverse = compare(&b, &a);
    let reverse_wait = reverse["metrics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["name"] == "mean_wait_ticks")
        .unwrap();
    assert!((reverse_wait["delta"].as_f64().unwrap() + 3.8).abs() < 1e-4);
    // Zero-baseline censoring is valid; no invented relative change or paired SE.
    let zero = report["metrics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["name"] == "censored_presentation_rate")
        .unwrap();
    assert_eq!(zero["left"], 0.0);
    assert!(zero["delta"].as_f64().unwrap().is_finite());
    let export = temp.0.path().join("csv");
    invoke(&[
        "export".into(),
        a.display().to_string(),
        "--format".into(),
        "csv".into(),
        "--output-dir".into(),
        export.display().to_string(),
    ])
    .unwrap();
    let csv = read_native_dataset(&export).unwrap();
    assert!(data.patients().equals_missing(csv.patients()));
    assert!(data.replications().equals_missing(csv.replications()));
    assert!(data.metrics().equals_missing(csv.metrics()));
    assert!(export.join("conversion.json").is_file());
    let before = std::fs::read(export.join("manifest.json")).unwrap();
    let error = invoke(&[
        "export".into(),
        a.display().to_string(),
        "--output-dir".into(),
        export.display().to_string(),
    ])
    .unwrap_err();
    assert_eq!(error.code, ExitCodeDef::FileAlreadyExists);
    assert_eq!(before, std::fs::read(export.join("manifest.json")).unwrap());
}
#[test]
fn native_compare_rejects_incompatible_units_and_manifest_schema() {
    let temp = Temp::new();
    let source = temp.source("nominal.json", NOMINAL);
    let a = temp.run(&source, "ticks", &[]);
    let mut seconds: Value = serde_json::from_str(NOMINAL).unwrap();
    seconds["time_unit"] = "second".into();
    let source = temp.source("seconds.json", &serde_json::to_string(&seconds).unwrap());
    let rejected_root = temp.0.path().join("seconds");
    let invalid_args = vec![
        "run".into(),
        source.display().to_string(),
        "--output-dir".into(),
        rejected_root.display().to_string(),
    ];
    assert_eq!(
        invoke(&invalid_args).unwrap_err().code,
        ExitCodeDef::MalformedConfig
    );
    assert!(
        !rejected_root.exists(),
        "unsupported physical units are rejected before output IO"
    );
    let b = temp.run(&temp.0.path().join("nominal.json"), "second-native", &[]);
    let args = vec![
        "compare".into(),
        a.display().to_string(),
        b.display().to_string(),
    ];
    let manifest = b.join("manifest.json");
    let original = std::fs::read(&manifest).unwrap();
    let mut control: Value = serde_json::from_slice(&original).unwrap();
    control["time_unit"] = "second".into();
    std::fs::write(&manifest, serde_json::to_vec(&control).unwrap()).unwrap();
    // Native patient metadata is cryptographically checked and must match units;
    // unlike a mutable RunDocument, changing the control alone cannot create a valid dataset.
    assert_eq!(
        invoke(&args).unwrap_err().code,
        ExitCodeDef::MalformedConfig
    );
    let mut control: Value = serde_json::from_slice(&original).unwrap();
    control["schema"] = "careops.ed.native-output.v99".into();
    std::fs::write(&manifest, serde_json::to_vec(&control).unwrap()).unwrap();
    // Invalid dataset versions are rejected by the typed reader before comparison.
    assert_eq!(
        invoke(&args).unwrap_err().code,
        ExitCodeDef::MalformedConfig
    );
}
#[test]
fn native_help_validation_and_retired_flags_have_no_output_side_effects() {
    let temp = Temp::new();
    let source = temp.source("nominal.json", NOMINAL);
    let mut help = Vec::new();
    execute(&["--help".into()], &mut help, None).unwrap();
    let help = String::from_utf8(help).unwrap();
    for command in ["run", "batch", "compare", "export", "validate"] {
        assert!(help.contains(command));
    }
    assert_eq!(
        execute(&[], &mut Vec::new(), None).unwrap_err().code,
        ExitCodeDef::UsageError
    );
    let validation = invoke(&["validate".into(), source.display().to_string()]).unwrap();
    assert_eq!(validation["patient_count"], 5);
    assert_eq!(
        invoke(&[
            "validate".into(),
            temp.0.path().join("missing").display().to_string()
        ])
        .unwrap_err()
        .code,
        ExitCodeDef::FileNotFound
    );
    for flag in [
        "--overwrite",
        "--export-arrow",
        "--export-spatial",
        "--spatial-rev",
    ] {
        let output = temp.0.path().join("forbidden");
        let args = vec![
            "run".into(),
            source.display().to_string(),
            "--output-dir".into(),
            output.display().to_string(),
            flag.into(),
        ];
        assert_eq!(
            execute(&args, &mut Vec::new(), None).unwrap_err().code,
            ExitCodeDef::UsageError
        );
        assert!(!output.exists());
    }
}
