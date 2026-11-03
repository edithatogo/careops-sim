use careops_birthing_cli::{run_batch, validate_replication, BatchInput};
#[test]
fn captured_rows_use_the_declared_second_unit() {
    let mut input: BatchInput =
        serde_json::from_str(include_str!("../fixtures/synthetic.json")).unwrap();
    input.replications = 1;
    input.scenario.days = 1;
    let root = tempfile::tempdir().unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    let reps = run_batch(root.path(), &input, 1, false, None).unwrap();
    let rep = &reps[0];
    let base = root.path().join(&input.batch_id);
    let manifest =
        validate_replication(&base, &rep.directory, &input, rep.index, rep.seed).unwrap();
    assert_eq!(manifest.tick_unit, "second");
    let bytes = std::fs::read(base.join(&rep.directory).join(&manifest.ipc[0].file)).unwrap();
    let frame =
        kairo_ecs_arrow::polars_interop::read_ipc_stream(std::io::Cursor::new(bytes)).unwrap();
    assert_eq!(
        frame.column("time_scale").unwrap().str().unwrap().get(0),
        Some(manifest.tick_unit.as_str())
    );
}

use careops_birthing_cli::{hash, replication_events};
use kairo_ecs_arrow::polars_interop as io;
use polars::prelude::*;
use serde_json::Value;
fn capture() -> (
    tempfile::TempDir,
    BatchInput,
    careops_birthing_cli::Replication,
) {
    let mut input: BatchInput =
        serde_json::from_str(include_str!("../fixtures/synthetic.json")).unwrap();
    input.replications = 1;
    input.scenario.days = 1;
    let root = tempfile::tempdir().unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    let rep = run_batch(root.path(), &input, 1, false, None)
        .unwrap()
        .remove(0);
    (root, input, rep)
}
fn rewrite_first(
    root: &std::path::Path,
    input: &BatchInput,
    rep: &careops_birthing_cli::Replication,
    transform: impl Fn(DataFrame) -> DataFrame,
) {
    let directory = root.join(&input.batch_id).join(&rep.directory);
    let domain_path = directory.join("birthing_manifest.json");
    let native_path = directory.join("run_manifest.json");
    let mut domain: Value = serde_json::from_slice(&std::fs::read(&domain_path).unwrap()).unwrap();
    let mut native: Value = serde_json::from_slice(&std::fs::read(&native_path).unwrap()).unwrap();
    let ipc_path = directory.join(domain["ipc"][0]["file"].as_str().unwrap());
    let parquet_path = directory.join(native["parts"][0]["file_name"].as_str().unwrap());
    let frame =
        io::read_ipc_stream(std::io::Cursor::new(std::fs::read(&ipc_path).unwrap())).unwrap();
    let mut frame = transform(frame);
    let mut ipc = Vec::new();
    io::write_ipc_stream(&mut frame, &mut ipc).unwrap();
    let mut parquet = Vec::new();
    io::write_parquet_zstd(&mut frame, &mut parquet).unwrap();
    std::fs::write(ipc_path, &ipc).unwrap();
    std::fs::write(parquet_path, &parquet).unwrap();
    domain["ipc"][0]["sha256"] = hash(&ipc).into();
    native["parts"][0]["file_hash"] = format!("{:016x}", io::calculate_checksum(&parquet)).into();
    std::fs::write(domain_path, serde_json::to_vec(&domain).unwrap()).unwrap();
    std::fs::write(native_path, serde_json::to_vec(&native).unwrap()).unwrap();
}
#[test]
fn exact_native_values_survive_part_shuffle_and_source_removal() {
    let (root, input, rep) = capture();
    let base = root.path().join(&input.batch_id);
    let (_, original) =
        replication_events(&base, &rep.directory, &input, rep.index, rep.seed).unwrap();
    rewrite_first(root.path(), &input, &rep, |frame| frame.reverse());
    let (manifest, frame) =
        replication_events(&base, &rep.directory, &input, rep.index, rep.seed).unwrap();
    assert_eq!(manifest.replication, rep);
    assert_eq!(frame.height(), rep.rows);
    assert!(frame.equals_missing(&original));
    assert!(frame.column("entity_id_index").unwrap().null_count() > 0);
    assert_eq!(
        frame.column("entity_id_index").unwrap().null_count(),
        frame.column("entity_id_gen").unwrap().null_count()
    );
    root.close().unwrap();
    assert!(frame.equals_missing(&original));
}
#[test]
fn mutually_matching_formats_cannot_hide_an_extra_column() {
    let (root, input, rep) = capture();
    rewrite_first(root.path(), &input, &rep, |mut frame| {
        frame
            .with_column(Series::new("unexpected".into(), vec![1u64; frame.height()]).into())
            .unwrap();
        frame
    });
    let error = replication_events(
        &root.path().join(&input.batch_id),
        &rep.directory,
        &input,
        rep.index,
        rep.seed,
    )
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("unsupported native event schema"));
}
#[test]
fn a_required_zero_field_cannot_be_replaced_with_null() {
    let (root, input, rep) = capture();
    rewrite_first(root.path(), &input, &rep, |mut frame| {
        frame
            .with_column(Series::new("priority".into(), vec![None::<i32>; frame.height()]).into())
            .unwrap();
        frame
    });
    let error = replication_events(
        &root.path().join(&input.batch_id),
        &rep.directory,
        &input,
        rep.index,
        rep.seed,
    )
    .unwrap_err();
    assert!(error.to_string().contains("null required B7 event field"));
}
