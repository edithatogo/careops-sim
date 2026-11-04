#![cfg(feature = "native-data")]
use careops_ed::disposition::batch_runner::{run_batch_with_workers, BatchConfig};
use careops_ed::disposition::native_output::*;
use careops_ed::ScenarioConfig;
fn scenario() -> ScenarioConfig {
    serde_json::from_str(include_str!("../examples/nominal_mvp.json")).unwrap()
}
fn batch_with(
    s: ScenarioConfig,
) -> (
    ScenarioConfig,
    careops_ed::disposition::batch_runner::BatchRunDocument,
) {
    let c = BatchConfig {
        batch_id: "native-output".into(),
        scenario_id: s.scenario_id.clone(),
        base_seed: 7,
        replications: 4,
        num_workers: 1,
        horizon_override: Some(8),
        ..Default::default()
    };
    let d = run_batch_with_workers(&s, &c, None, None, None).unwrap();
    (s, d)
}
fn batch() -> (
    ScenarioConfig,
    careops_ed::disposition::batch_runner::BatchRunDocument,
) {
    batch_with(scenario())
}
#[test]
fn default_output_is_real_zstd_parquet_with_manifest_last() {
    let (s, d) = batch();
    let destination = tempfile::tempdir().unwrap();
    let manifest = export_native_batch(
        &s,
        &d,
        destination.path(),
        NativeFormat::default(),
        ExportLimits::default(),
    )
    .unwrap();
    assert_eq!(manifest.format, NativeFormat::Parquet);
    assert_eq!(manifest.compression.as_deref(), Some("zstd"));
    assert_eq!(manifest.tables[0].name, "patients");
    assert_eq!(manifest.tables[0].rows, 20);
    for table in &manifest.tables {
        for part in &table.parts {
            let bytes = std::fs::read(destination.path().join(&part.file)).unwrap();
            assert_eq!(&bytes[..4], b"PAR1");
        }
    }
    assert_eq!(
        manifest,
        serde_json::from_slice(&std::fs::read(destination.path().join("manifest.json")).unwrap())
            .unwrap()
    );
}

#[test]
fn all_output_budgets_refuse_before_any_physical_publication() {
    let (s, d) = batch();
    for limits in [
        ExportLimits {
            max_total_rows: 28,
            ..Default::default()
        },
        ExportLimits {
            max_parts: 2,
            ..Default::default()
        },
        ExportLimits {
            max_manifest_bytes: 16,
            ..Default::default()
        },
        ExportLimits {
            max_bytes_per_part: 32,
            ..Default::default()
        },
        ExportLimits {
            max_rows_per_part: 0,
            ..Default::default()
        },
    ] {
        for format in [NativeFormat::Parquet, NativeFormat::Ipc, NativeFormat::Csv] {
            let destination = tempfile::tempdir().unwrap();
            assert!(export_native_batch(&s, &d, destination.path(), format, limits).is_err());
            assert_eq!(std::fs::read_dir(destination.path()).unwrap().count(), 0);
        }
    }
}
#[test]
fn foreign_document_scenario_and_overflow_refuse_without_output() {
    let (s, d) = batch();
    let mut bad = d.clone();
    bad.replications[0].arrivals = usize::MAX;
    let mut other = s.clone();
    other.horizon_ticks += 1;
    for (scenario, document) in [(&other, &d), (&s, &bad)] {
        let destination = tempfile::tempdir().unwrap();
        assert!(export_native_batch(
            scenario,
            document,
            destination.path(),
            NativeFormat::Parquet,
            ExportLimits::default()
        )
        .is_err());
        assert_eq!(std::fs::read_dir(destination.path()).unwrap().count(), 0);
    }
}
#[test]
fn destination_collisions_and_symlinks_preserve_original_content() {
    let (s, d) = batch();
    let destination = tempfile::tempdir().unwrap();
    let path = destination.path().join("manifest.json");
    std::fs::write(&path, b"owned").unwrap();
    assert!(export_native_batch(
        &s,
        &d,
        destination.path(),
        NativeFormat::Parquet,
        ExportLimits::default()
    )
    .is_err());
    assert_eq!(std::fs::read(path).unwrap(), b"owned");
    #[cfg(unix)]
    {
        let parent = tempfile::tempdir().unwrap();
        let link = parent.path().join("alias");
        std::os::unix::fs::symlink(destination.path(), &link).unwrap();
        assert!(export_native_batch(
            &s,
            &d,
            &link,
            NativeFormat::Parquet,
            ExportLimits::default()
        )
        .is_err());
    }
}
#[test]
fn real_csv_byte_cap_splits_preserve_all_offsets_without_assuming_compression_order() {
    let (s, d) = batch();
    let baseline = tempfile::tempdir().unwrap();
    let manifest = export_native_batch(
        &s,
        &d,
        baseline.path(),
        NativeFormat::Csv,
        ExportLimits {
            max_rows_per_part: 1,
            ..Default::default()
        },
    )
    .unwrap();
    let bytes = manifest
        .tables
        .iter()
        .flat_map(|t| &t.parts)
        .map(|p| p.bytes)
        .max()
        .unwrap()
        + 32;
    let destination = tempfile::tempdir().unwrap();
    let split = export_native_batch(
        &s,
        &d,
        destination.path(),
        NativeFormat::Csv,
        ExportLimits {
            max_bytes_per_part: bytes,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(split.tables[0].parts.len() > 1);
    for table in split.tables {
        let mut offset = 0;
        for part in table.parts {
            assert_eq!(part.first_row, offset);
            assert!(part.bytes <= bytes);
            offset += part.rows;
        }
        assert_eq!(offset, table.rows);
    }
}
#[test]
fn byte_cap_signal_never_relabels_an_underlying_io_error_as_a_sizing_split() {
    use careops_ed::capture::BoundedPartWriter;
    use std::io::{self, Write};
    struct Failed;
    impl Write for Failed {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("physical failure"))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut writer = BoundedPartWriter::new(Failed, 8);
    assert!(writer.write_all(b"x").is_err());
    assert!(!writer.limit_exceeded());
    let mut capped = BoundedPartWriter::new(io::sink(), 1);
    assert!(capped.write_all(b"xx").is_err());
    assert!(capped.limit_exceeded());
}
#[test]
fn qualified_native_output_fixture_requires_explicit_fresh_destination() {
    let Some(root) = std::env::var_os("CAREOPS_NATIVE_OUTPUT_FIXTURE") else {
        return;
    };
    let root = std::path::PathBuf::from(root);
    assert!(root.is_absolute());
    std::fs::create_dir(&root).unwrap();
    let mut quoted = scenario();
    quoted.patients[0].patient_id = "case,\"a\r\nline ☃".into();
    let (s, d) = batch_with(quoted);
    for (format, name) in [
        (NativeFormat::Parquet, "parquet"),
        (NativeFormat::Ipc, "ipc"),
        (NativeFormat::Csv, "csv"),
    ] {
        let path = root.join(name);
        std::fs::create_dir(&path).unwrap();
        let manifest = export_native_batch(
            &s,
            &d,
            &path,
            format,
            ExportLimits {
                max_rows_per_part: 2,
                ..Default::default()
            },
        )
        .unwrap();
        assert_ne!(
            manifest.effective_input_sha256,
            manifest.parsed_input_sha256
        );
        assert_eq!(
            manifest.model_contract_sha256,
            careops_ed::capture::MODEL_CONTRACT_SHA256
        );
    }
    let maximum = root.join("max-tick");
    std::fs::create_dir(&maximum).unwrap();
    let (s, d) = maximum_tick_batch();
    for (format, name) in [
        (NativeFormat::Parquet, "parquet"),
        (NativeFormat::Ipc, "ipc"),
        (NativeFormat::Csv, "csv"),
    ] {
        let path = maximum.join(name);
        std::fs::create_dir(&path).unwrap();
        export_native_batch(&s, &d, &path, format, ExportLimits::default()).unwrap();
    }
}

fn maximum_tick_batch() -> (
    ScenarioConfig,
    careops_ed::disposition::batch_runner::BatchRunDocument,
) {
    let mut s = scenario();
    s.patients.truncate(1);
    s.horizon_ticks = u64::MAX;
    s.patients[0].arrival_tick = u64::MAX - 1;
    s.patients[0].work_ticks = 1;
    let config = BatchConfig {
        batch_id: "maximum-tick-output".into(),
        scenario_id: s.scenario_id.clone(),
        base_seed: 7,
        replications: 1,
        num_workers: 1,
        ..Default::default()
    };
    let document = run_batch_with_workers(&s, &config, None, None, None).unwrap();
    (s, document)
}
#[test]
fn maximum_producer_tick_survives_native_output_without_float_conversion() {
    use polars::prelude::*;
    use std::io::Cursor;
    let (s, d) = maximum_tick_batch();
    let destination = tempfile::tempdir().unwrap();
    let manifest = export_native_batch(
        &s,
        &d,
        destination.path(),
        NativeFormat::Parquet,
        ExportLimits::default(),
    )
    .unwrap();
    assert_eq!(manifest.horizon_ticks, u64::MAX);
    let frame = ParquetReader::new(Cursor::new(
        std::fs::read(destination.path().join(&manifest.tables[0].parts[0].file)).unwrap(),
    ))
    .finish()
    .unwrap();
    assert_eq!(
        frame
            .column("completion_tick_lo")
            .unwrap()
            .u64()
            .unwrap()
            .get(0),
        Some(u64::MAX)
    );
    assert_eq!(
        frame
            .column("completion_tick_hi")
            .unwrap()
            .u64()
            .unwrap()
            .get(0),
        Some(0)
    );
}
#[test]
fn missing_original_capture_is_never_regenerated_by_export() {
    let (s, d) = batch();
    let source = d.capture.as_ref().unwrap().directory.join("manifest.json");
    std::fs::remove_file(source).unwrap();
    let destination = tempfile::tempdir().unwrap();
    assert!(export_native_batch(
        &s,
        &d,
        destination.path(),
        NativeFormat::Parquet,
        ExportLimits::default()
    )
    .is_err());
    assert_eq!(std::fs::read_dir(destination.path()).unwrap().count(), 0);
}
