// Copyright (c) 2026 CareOps contributors.
// SPDX-License-Identifier: Apache-2.0
// Selected parent domain projections through owned C Data and streams.
// Actual B7 durable producer/readback has independent model reference values.
#![cfg(feature = "native-ffi-acceptance")]
#![forbid(unsafe_code)]
use kairo_data_arrow::arrow_c_data::{
    domain, Array, ArrowDataType as D, ArrowField as F, PrimitiveArray, StructArray, Utf8Array,
};
use kairo_data_ffi::native_tables::{import_batch, import_stream};
use std::sync::Arc;
type Expected = (String, Option<u64>, Option<u64>);
fn field(name: &str, dtype: D, nullable: bool) -> F {
    F {
        name: name.into(),
        dtype,
        is_nullable: nullable,
        metadata: None,
    }
}
fn contract(schema: &str, names: [&str; 3], nullable: bool, provenance: &str) -> F {
    F {
        name: schema.into(),
        dtype: D::Struct(vec![
            field(names[0], D::LargeUtf8, false),
            field(names[1], D::UInt64, nullable),
            field(names[2], D::UInt64, nullable),
        ]),
        is_nullable: false,
        metadata: Some(Arc::new(
            [
                ("schema".into(), schema.into()),
                ("time_unit".into(), "tick".into()),
                ("source_provenance".into(), provenance.into()),
            ]
            .into_iter()
            .collect(),
        )),
    }
}
fn check(array: &StructArray, expected: &[Expected]) {
    assert_eq!(array.len(), expected.len());
    assert_eq!(array.values().len(), 3);
    let text = array.values()[0]
        .as_any()
        .downcast_ref::<Utf8Array<i64>>()
        .unwrap();
    let hi = array.values()[1]
        .as_any()
        .downcast_ref::<PrimitiveArray<u64>>()
        .unwrap();
    let lo = array.values()[2]
        .as_any()
        .downcast_ref::<PrimitiveArray<u64>>()
        .unwrap();
    for (i, (s, h, l)) in expected.iter().enumerate() {
        assert!(!text.is_null(i));
        assert_eq!(text.value(i), s);
        assert_eq!(hi.is_null(i), h.is_none());
        assert_eq!(lo.is_null(i), l.is_none());
        if let Some(v) = h {
            assert_eq!(hi.value(i), *v);
        }
        if let Some(v) = l {
            assert_eq!(lo.value(i), *v);
        }
    }
}
fn exercise(mut frame: domain::DataFrame, schema: F, expected: Vec<Expected>) {
    assert!(expected.len() >= 2);
    assert_eq!(frame.height(), expected.len());
    let batch = domain::export_batch(&frame, &schema, Some(&schema)).unwrap();
    let stream = domain::export_stream(&frame, &schema, 1, Some(&schema)).unwrap();
    let early = domain::export_stream(&frame, &schema, 1, None).unwrap();
    let empty = frame.slice(0, 0);
    let empty_batch = domain::export_batch(&empty, &schema, None).unwrap();
    let empty_stream = domain::export_stream(&empty, &schema, 1, None).unwrap();
    assert!(domain::export_stream(&frame, &schema, 0, None).is_err());
    let mut wrong = schema.clone();
    wrong.name = "wrong-negotiation".into();
    assert!(domain::export_batch(&frame, &schema, Some(&wrong)).is_err());
    assert!(domain::export_stream(&frame, &schema, 1, Some(&wrong)).is_err());
    wrong = schema.clone();
    wrong.metadata = None;
    assert!(domain::export_batch(&frame, &schema, Some(&wrong)).is_err());
    frame = frame.clear();
    assert_eq!(frame.height(), 0);
    drop(frame);
    drop(empty);
    let (actual, array) = import_batch(batch).unwrap();
    assert_eq!(actual, schema);
    check(&array, &expected);
    let mut reader = import_stream(stream).unwrap();
    assert_eq!(reader.schema(), &schema);
    for row in &expected {
        let a = reader.next().unwrap().unwrap();
        check(&a, std::slice::from_ref(row));
    }
    assert!(reader.next().is_none());
    assert!(reader.next().is_none());
    drop(reader);
    let mut reader = import_stream(early).unwrap();
    let retained = reader.next().unwrap().unwrap();
    drop(reader);
    check(&retained, &expected[..1]);
    let (actual, empty) = import_batch(empty_batch).unwrap();
    assert_eq!(actual, schema);
    check(&empty, &[]);
    let mut empty = import_stream(empty_stream).unwrap();
    assert_eq!(empty.schema(), &schema);
    assert!(empty.next().is_none());
    assert!(empty.next().is_none());
}
#[test]
fn actual_ed_replication_projection_crosses_owned_c_data_and_stream() {
    use careops_ed::disposition::{
        batch_runner::{run_batch_with_workers, BatchConfig},
        native_output::*,
    };
    let scenario = careops_ed::parse_scenario_json(include_str!(
        "../crates/careops-ed/examples/nominal_mvp.json"
    ))
    .unwrap();
    let config = BatchConfig {
        batch_id: "r143-ed".into(),
        scenario_id: scenario.scenario_id.clone(),
        base_seed: 7,
        replications: 4,
        num_workers: 1,
        horizon_override: Some(8),
        ..Default::default()
    };
    let source = run_batch_with_workers(&scenario, &config, None, None, None).unwrap();
    let expected = source
        .replications
        .iter()
        .map(|r| {
            (
                "r143-ed".into(),
                Some(r.replication_index as u64),
                Some(r.seed),
            )
        })
        .collect();
    let root = tempfile::tempdir().unwrap();
    export_native_batch(
        &scenario,
        &source,
        root.path(),
        NativeFormat::Parquet,
        ExportLimits::default(),
    )
    .unwrap();
    let data = read_native_dataset(root.path()).unwrap();
    let frame = data
        .replications()
        .select(["run_id", "replication_index", "seed"])
        .unwrap();
    let mut schema = contract(
        "careops.r143.ed-replication-projection.v1",
        ["run_id", "replication_index", "seed"],
        false,
        data.manifest_sha256(),
    );
    Arc::make_mut(schema.metadata.as_mut().unwrap())
        .insert("time_unit".into(), data.manifest().time_unit.clone().into());
    drop(data);
    drop(source);
    root.close().unwrap();
    exercise(frame, schema, expected);
}
#[test]
fn actual_calibration_projection_preserves_nullable_endpoints_and_max_limb() {
    use careops_ed::disposition::composite_fixture::{
        native_calibration::calibration_table, CalibrationTraceRecord, CalibrationTransitSource,
    };
    let records = vec![
        CalibrationTraceRecord {
            case_id: "case,\"é\r\n".into(),
            day: 1,
            start_tick: Some(u64::MAX),
            end_tick: Some(u64::MAX),
            transit_ticks: 0,
            transit_source: CalibrationTransitSource::CallerSupplied,
            transit_source_seconds: None,
            work_ticks: 1,
        },
        CalibrationTraceRecord {
            case_id: "missing".into(),
            day: 1,
            start_tick: None,
            end_tick: None,
            transit_ticks: 0,
            transit_source: CalibrationTransitSource::CallerSupplied,
            transit_source_seconds: None,
            work_ticks: 1,
        },
    ];
    let frame = calibration_table(&records, "r143-calibration", "tick")
        .unwrap()
        .select(["case_id", "start_tick_hi", "start_tick_lo"])
        .unwrap();
    let schema = contract(
        "careops.r143.calibration-projection.v1",
        ["case_id", "start_tick_hi", "start_tick_lo"],
        true,
        "synthetic_fixture:r143-calibration",
    );
    let mut required = schema.clone();
    if let D::Struct(fields) = &mut required.dtype {
        fields[1].is_nullable = false;
    }
    assert!(domain::export_batch(&frame, &required, None).is_err());
    drop(records);
    exercise(
        frame,
        schema,
        vec![
            ("case,\"é\r\n".into(), Some(0), Some(u64::MAX)),
            ("missing".into(), None, None),
        ],
    );
}
#[test]
fn actual_birthing_file_validated_projection_crosses_owned_boundary() {
    use careops_birthing_cli::theatre_tables::read_theatre_tables;
    use careops_birthing_cli::{hash, model, run_batch, BatchInput};
    let mut input: BatchInput = serde_json::from_str(include_str!(
        "../crates/careops-birthing-cli/fixtures/synthetic.json"
    ))
    .unwrap();
    input.batch_id = "r143-birthing".into();
    input.seed = 41;
    input.replications = 2;
    input.scenario.days = 2;
    let root = tempfile::tempdir().unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    let replications = run_batch(root.path(), &input, 1, false, None).unwrap();
    let batch_root = root.path().join(&input.batch_id);
    let provenance = hash(&std::fs::read(batch_root.join("batch_manifest.json")).unwrap());
    let (manifests, tables) = read_theatre_tables(&batch_root, &input, &replications).unwrap();
    assert_eq!(manifests.len(), input.replications);
    // The expected values come from the original domain model, independently
    // of the typed frame/readback under test. Re-execution is a test oracle;
    // neither native readback nor FFI export invokes the simulation.
    let mut keyed = Vec::new();
    for replication in &replications {
        let suffix = format!("-rep{}", replication.index);
        let run_id = replication.directory.strip_suffix(&suffix).unwrap();
        let rows = model::simulate(
            &input.scenario,
            replication.seed,
            replication.index as u64 + 1,
            0,
        )
        .unwrap();
        for row in rows.iter().filter(|row| row.kind == "theatre_case") {
            let case = row.data["case"].as_u64().unwrap();
            let decision = row.data["decision_tick"].as_u64().unwrap();
            keyed.push((run_id.to_owned(), replication.index, case, decision));
        }
    }
    // The B7 production view's canonical sort key is run/replication/case.
    keyed.sort_by(|a, b| (&a.0, a.1, a.2).cmp(&(&b.0, b.1, b.2)));
    let expected = keyed
        .into_iter()
        .map(|(run, _, _, decision)| (run, Some(0), Some(decision)))
        .collect();
    let frame = tables
        .cases
        .select(["run_id", "decision_tick_hi", "decision_tick_lo"])
        .unwrap();
    let mut schema = contract(
        "careops.r143.birthing-projection.v1",
        ["run_id", "decision_tick_hi", "decision_tick_lo"],
        false,
        &provenance,
    );
    // B7 declares seconds. Do not infer a tick-to-second conversion.
    Arc::make_mut(schema.metadata.as_mut().unwrap()).insert("time_unit".into(), "second".into());
    drop(tables);
    drop(manifests);
    drop(replications);
    drop(input);
    root.close().unwrap();
    exercise(frame, schema, expected);
}
