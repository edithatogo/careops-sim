#![cfg(feature = "native-data")]
use careops_ed::disposition::batch_runner::{run_batch_with_workers, BatchConfig};
use careops_ed::disposition::native_output::*;
fn source(format: NativeFormat) -> tempfile::TempDir {
    let scenario =
        careops_ed::parse_scenario_json(include_str!("../examples/nominal_mvp.json")).unwrap();
    let config = BatchConfig {
        batch_id: "native-reader".into(),
        scenario_id: scenario.scenario_id.clone(),
        replications: 4,
        num_workers: 1,
        horizon_override: Some(8),
        ..Default::default()
    };
    let document = run_batch_with_workers(&scenario, &config, None, None, None).unwrap();
    let root = tempfile::tempdir().unwrap();
    export_native_batch(
        &scenario,
        &document,
        root.path(),
        format,
        ExportLimits {
            max_rows_per_part: 2,
            ..Default::default()
        },
    )
    .unwrap();
    root
}
#[test]
fn actual_native_files_import_to_owned_typed_tables() {
    let root = source(NativeFormat::Parquet);
    let dataset = read_native_dataset(root.path()).unwrap();
    assert_eq!(dataset.patients().height(), 20);
    assert_eq!(dataset.replications().height(), 4);
    assert_eq!(dataset.metrics().height(), 5);
}

fn same_tables(left: &NativeDataset, right: &NativeDataset) {
    for (left, right) in [
        (left.patients(), right.patients()),
        (left.replications(), right.replications()),
        (left.metrics(), right.metrics()),
    ] {
        assert_eq!(left.schema(), right.schema());
        assert_eq!(left.shape(), right.shape());
        for (left, right) in left.columns().iter().zip(right.columns()) {
            assert!(
                left.equals_missing(right),
                "column differs: {}",
                left.name()
            );
        }
    }
}
#[test]
fn all_formats_convert_from_owned_snapshot_after_original_files_are_deleted() {
    for format in [NativeFormat::Parquet, NativeFormat::Ipc, NativeFormat::Csv] {
        let root = source(format);
        let source_manifest = std::fs::read(root.path().join("manifest.json")).unwrap();
        let dataset = read_native_dataset(root.path()).unwrap();
        let original_model = dataset.manifest().model_contract_sha256.clone();
        assert_eq!(
            dataset.manifest_sha256(),
            careops_ed::disposition::api_runner::compute_sha256_hex(&source_manifest)
        );
        drop(root);
        for target in [NativeFormat::Parquet, NativeFormat::Ipc, NativeFormat::Csv] {
            let dest = tempfile::tempdir().unwrap();
            let converted =
                convert_native_dataset(&dataset, dest.path(), target, ExportLimits::default())
                    .unwrap();
            assert_eq!(converted.format, target);
            assert_eq!(converted.model_contract_sha256, original_model);
            assert_eq!(
                converted.effective_input_sha256,
                dataset.manifest().effective_input_sha256
            );
            let reloaded = read_native_dataset(dest.path()).unwrap();
            same_tables(&dataset, &reloaded);
            let receipt: serde_json::Value = serde_json::from_slice(
                &std::fs::read(dest.path().join("conversion.json")).unwrap(),
            )
            .unwrap();
            assert_eq!(receipt["source_manifest_sha256"], dataset.manifest_sha256());
            assert_eq!(
                receipt["target_manifest_sha256"],
                reloaded.manifest_sha256()
            );
            let saved = std::fs::read(dest.path().join("manifest.json")).unwrap();
            assert!(
                convert_native_dataset(&dataset, dest.path(), target, ExportLimits::default())
                    .is_err()
            );
            assert_eq!(
                std::fs::read(dest.path().join("manifest.json")).unwrap(),
                saved
            );
        }
        let dest = tempfile::tempdir().unwrap();
        assert!(convert_native_dataset(
            &dataset,
            dest.path(),
            NativeFormat::Csv,
            ExportLimits {
                max_total_rows: 1,
                ..Default::default()
            }
        )
        .is_err());
        assert_eq!(std::fs::read_dir(dest.path()).unwrap().count(), 0);
    }
}
#[test]
fn rehashed_csv_content_cannot_change_provenance_or_hide_malformed_rows() {
    for mutation in [
        "batch",
        "input",
        "model",
        "unit",
        "scenario",
        "seed",
        "horizon",
        "malformed",
    ] {
        let root = source(NativeFormat::Csv);
        let path = root.path().join("manifest.json");
        let mut manifest: NativeManifest =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let table_index = if mutation == "horizon" { 1 } else { 0 };
        let part = &mut manifest.tables[table_index].parts[0];
        let part_path = root.path().join(&part.file);
        let raw = std::fs::read_to_string(&part_path).unwrap();
        let mut lines = raw.lines().map(str::to_owned).collect::<Vec<_>>();
        let headers = lines[0].split(',').collect::<Vec<_>>();
        let name = match mutation {
            "batch" => "run_id",
            "input" => "input_sha256",
            "model" => "model_contract_sha256",
            "unit" => "time_unit",
            "scenario" => "scenario_id",
            "seed" => "seed",
            "horizon" => "horizon_ticks",
            _ => "run_id",
        };
        let index = headers.iter().position(|h| *h == name).unwrap();
        let mut cells = lines[1].split(',').map(str::to_owned).collect::<Vec<_>>();
        cells[index] = if ["seed", "horizon"].contains(&mutation) {
            "999".into()
        } else {
            "foreign".into()
        };
        if mutation == "malformed" {
            cells.pop();
        }
        lines[1] = cells.join(",");
        let changed = lines.join("\n") + "\n";
        std::fs::write(&part_path, &changed).unwrap();
        part.bytes = changed.len();
        part.sha256 = careops_ed::disposition::api_runner::compute_sha256_hex(changed.as_bytes());
        std::fs::write(&path, serde_json::to_vec(&manifest).unwrap()).unwrap();
        assert!(
            read_native_dataset(root.path()).is_err(),
            "accepted rehashed {mutation}"
        );
    }
}
#[test]
fn rehashed_ipc_and_parquet_parts_still_require_valid_predecode_profiles() {
    use polars_parquet_format::{
        thrift::protocol::{TCompactInputProtocol, TCompactOutputProtocol},
        FileMetaData, PageHeader,
    };
    for format in [NativeFormat::Ipc, NativeFormat::Parquet] {
        let root = source(format);
        let path = root.path().join("manifest.json");
        let mut manifest: NativeManifest =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let part = &mut manifest.tables[0].parts[0];
        let part_path = root.path().join(&part.file);
        let mut bytes = std::fs::read(&part_path).unwrap();
        if format == NativeFormat::Ipc {
            let offset = bytes.windows(8).position(|s| s == b"patient-").unwrap();
            bytes[offset] = 255;
        } else {
            let length =
                u32::from_le_bytes(bytes[bytes.len() - 8..bytes.len() - 4].try_into().unwrap())
                    as usize;
            let begin = bytes.len() - 8 - length;
            let m = FileMetaData::read_from_in_protocol(&mut TCompactInputProtocol::new(
                std::io::Cursor::new(&bytes[begin..bytes.len() - 8]),
                1024 * 1024,
            ))
            .unwrap();
            let c = m.row_groups[0].columns[0].meta_data.as_ref().unwrap();
            let offset = c.dictionary_page_offset.unwrap_or(c.data_page_offset) as usize;
            let mut page = PageHeader::read_from_in_protocol(&mut TCompactInputProtocol::new(
                std::io::Cursor::new(&bytes[offset..begin]),
                65536,
            ))
            .unwrap();
            page.uncompressed_page_size = i32::MAX;
            let mut header = Vec::new();
            page.write_to_out_protocol(&mut TCompactOutputProtocol::new(&mut header))
                .unwrap();
            bytes[offset..offset + header.len()].copy_from_slice(&header);
        }
        part.sha256 = careops_ed::disposition::api_runner::compute_sha256_hex(&bytes);
        std::fs::write(&part_path, &bytes).unwrap();
        std::fs::write(&path, serde_json::to_vec(&manifest).unwrap()).unwrap();
        assert!(
            read_native_dataset(root.path()).is_err(),
            "accepted rehashed {format:?}"
        );
    }
}
/// Opt-in actual conversion of the retained independently qualified fixture.
/// No source model is rerun. The companion Python reader checks these files.
#[test]
fn explicit_qualified_historical_conversion_fixture() {
    let Some((source, target)) = std::env::var_os("CAREOPS_NATIVE_READER_SOURCE")
        .zip(std::env::var_os("CAREOPS_NATIVE_READER_CONVERTED"))
    else {
        return;
    };
    let source = std::path::PathBuf::from(source);
    let target = std::path::PathBuf::from(target);
    assert!(source.is_absolute() && target.is_absolute());
    std::fs::create_dir(&target).unwrap();
    for suffix in ["", "max-tick"] {
        let source = source.join(suffix).join("parquet");
        let destination = target.join(suffix);
        if !suffix.is_empty() {
            std::fs::create_dir(&destination).unwrap();
        }
        let dataset = read_native_dataset(&source).unwrap();
        for (format, name) in [
            (NativeFormat::Parquet, "parquet"),
            (NativeFormat::Ipc, "ipc"),
            (NativeFormat::Csv, "csv"),
        ] {
            let path = destination.join(name);
            std::fs::create_dir(&path).unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
            }
            convert_native_dataset(&dataset, &path, format, ExportLimits::default()).unwrap();
        }
    }
}
