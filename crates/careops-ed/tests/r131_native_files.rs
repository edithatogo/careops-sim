#![cfg(feature = "native-data")]
use careops_ed::disposition::composite_fixture::{
    native_calibration::calibration_table, CalibrationTraceRecord, CalibrationTransitSource,
};
fn calibration() -> Vec<CalibrationTraceRecord> {
    vec![
        CalibrationTraceRecord {
            case_id: "case,\"a\r\nline".into(),
            day: 1,
            start_tick: Some(u64::MAX - 1),
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
    ]
}
#[test]
fn calibration_csv_requires_real_quoting_for_captured_case_ids() {
    let table = calibration_table(&calibration(), "r131-fixture", "tick").unwrap();
    let text = String::from_utf8(encode(&table, "csv", &BTreeMap::new())).unwrap();
    assert!(
        text.contains("\"case,\"\"a\r\nline\""),
        "native CSV must quote comma/quote/CRLF case identity"
    );
}
#[test]
fn calibration_native_source_retains_nullable_endpoints() {
    let table = calibration_table(&calibration(), "r131-fixture", "tick").unwrap();
    assert_eq!(table.height(), 2);
    assert_eq!(
        table
            .column("start_tick_lo")
            .unwrap()
            .u64()
            .unwrap()
            .null_count(),
        1
    );
}

use careops_ed::disposition::api_runner::compute_sha256_hex;
use careops_ed::disposition::arrow_export::native_tables::patient_table;
use careops_ed::disposition::batch_runner::{run_batch_with_workers, BatchConfig};
use polars::prelude::*;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;
fn physical() -> DataFrame {
    df!("row_index"=>[0u64,1,2,3,4,5,6],
        "entity_id_index"=>[Some(0u64),Some(u64::MAX),None,Some(2),Some(3),Some(4),Some(5)],
        "entity_id_gen"=>[Some(u32::MAX),Some(0),None,Some(1),Some(2),Some(3),Some(4)],
        "tick_hi"=>[Some(u64::MAX),Some(0),Some(1),None,Some(0),Some(0),Some(0)],
        "tick_lo"=>[Some(u64::MAX),Some(9_007_199_254_740_993),Some(0),None,Some(0),Some(1),Some(2)],
        "estimate"=>[Some(1.5f64),None,Some(-0.0),Some(0.0),Some(f64::NAN),Some(f64::INFINITY),Some(f64::NEG_INFINITY)],
        "flag"=>[Some(true),Some(false),None,Some(true),None,Some(false),Some(true)],
        "text"=>[Some("comma,\"quote\r\nline"),Some(""),Some("\\N"),None,Some("雪"),Some("NA"),Some("null")]).unwrap()
}
fn multiple_batches() -> DataFrame {
    df!("row_index"=>(0..524_288u32).collect::<Vec<_>>()).unwrap()
}
#[test]
fn multi_batch_ipc_preserves_native_row_order() {
    let df = multiple_batches();
    let round = decode(&df, "ipc", encode(&df, "ipc", &BTreeMap::new()));
    assert!(round.equals_missing(&df));
}

fn encode(df: &DataFrame, format: &str, metadata: &BTreeMap<String, String>) -> Vec<u8> {
    let mut df = df.clone();
    let mut bytes = Vec::new();
    match format {
        "parquet" => {
            ParquetWriter::new(&mut bytes)
                .with_compression(ParquetCompression::Zstd(None))
                .with_key_value_metadata(Some(KeyValueMetadata::from_static(
                    metadata
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect(),
                )))
                .finish(&mut df)
                .unwrap();
        }
        "ipc" => {
            let mut writer = IpcStreamWriter::new(&mut bytes);
            writer.set_custom_schema_metadata(Arc::new(
                metadata
                    .iter()
                    .map(|(k, v)| (k.as_str().into(), v.as_str().into()))
                    .collect(),
            ));
            writer.finish(&mut df).unwrap();
        }
        "csv" => {
            CsvWriter::new(&mut bytes)
                .include_header(true)
                .with_quote_style(QuoteStyle::Necessary)
                .with_null_value("".into())
                .finish(&mut df)
                .unwrap();
        }
        _ => panic!("test fixture format"),
    }
    bytes
}
fn decode(df: &DataFrame, format: &str, bytes: Vec<u8>) -> DataFrame {
    match format {
        "parquet" => ParquetReader::new(Cursor::new(bytes)).finish().unwrap(),
        "ipc" => IpcStreamReader::new(Cursor::new(bytes)).finish().unwrap(),
        "csv" => CsvReadOptions::default()
            .with_has_header(true)
            .with_schema(Some(df.schema().clone()))
            .into_reader_with_file_handle(Cursor::new(bytes))
            .finish()
            .unwrap(),
        _ => panic!("test fixture format"),
    }
}
#[test]
fn native_files_keep_full_limbs_generation_zero_nulls_and_string_distinctions() {
    let df = physical();
    for format in ["parquet", "ipc", "csv"] {
        let round = decode(&df, format, encode(&df, format, &BTreeMap::new()));
        assert!(round.equals_missing(&df), "{format}: {round:?}");
        assert_eq!(
            round
                .column("estimate")
                .unwrap()
                .f64()
                .unwrap()
                .get(2)
                .unwrap()
                .to_bits(),
            (-0.0f64).to_bits()
        );
        assert_eq!(
            round.column("tick_hi").unwrap().u64().unwrap().get(0),
            Some(u64::MAX)
        );
        assert_eq!(
            round.column("tick_lo").unwrap().u64().unwrap().get(1),
            Some(9_007_199_254_740_993)
        );
    }
}
#[test]
fn typed_empty_files_do_not_invent_observations() {
    let df = physical().clear();
    for format in ["parquet", "ipc", "csv"] {
        let round = decode(&df, format, encode(&df, format, &BTreeMap::new()));
        assert!(round.equals_missing(&df));
        assert_eq!(round.height(), 0);
    }
}
fn emit(
    root: &Path,
    name: &str,
    schema: &str,
    kind: &str,
    df: &DataFrame,
    mut metadata: BTreeMap<String, String>,
) -> Value {
    metadata.insert(
        "careops.data_library_revision".into(),
        "473ff2c0e640a7e457f500593f6ea1f06602ab35".into(),
    );
    metadata.insert(
        "careops.runtime_revision".into(),
        "07410266bf54b6f73c59db2f8cbfbc5697c53734".into(),
    );
    metadata.insert(
        "careops.producer".into(),
        format!("careops-ed/{}", env!("CARGO_PKG_VERSION")),
    );
    metadata.insert("careops.table_schema".into(), schema.into());
    metadata.insert("careops.time_unit".into(), "tick".into());
    metadata.insert("careops.fixture_kind".into(), kind.into());
    metadata.insert(
        "careops.polars_revision".into(),
        "96439685eb2f95108f51087d2b76091917f2593c".into(),
    );
    let mut files = Vec::new();
    for format in ["parquet", "ipc", "csv"] {
        let file = format!("{name}.{format}");
        let bytes = encode(df, format, &metadata);
        assert!(bytes.len() < 4 * 1024 * 1024);
        use std::io::Write;
        let mut output = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(root.join(&file))
            .unwrap();
        output.write_all(&bytes).unwrap();
        output.flush().unwrap();
        files.push(json!({"format":format,"path":file,"bytes":bytes.len(),"sha256":compute_sha256_hex(&bytes)}));
    }
    json!({"name":name,"schema":schema,"kind":kind,"rows":df.height(),"columns":df.columns().iter().map(|col|json!([col.name().as_str(),format!("{:?}",col.dtype())])).collect::<Vec<_>>(),"metadata":metadata,"files":files})
}
#[test]
fn actual_native_domain_fixture_is_published_only_to_explicit_fresh_destination() {
    assert!(include_str!("../Cargo.toml").contains("473ff2c0e640a7e457f500593f6ea1f06602ab35"));
    let contract: Value = serde_json::from_str(include_str!(
        "../../../conductor/evidence/d2.4-kairos-contract.json"
    ))
    .unwrap();
    assert_eq!(
        contract["kairos_commit"],
        "07410266bf54b6f73c59db2f8cbfbc5697c53734"
    );
    let owned = tempfile::tempdir().unwrap();
    let supplied = std::env::var_os("CAREOPS_R131_FILES_DESTINATION").map(std::path::PathBuf::from);
    let root = supplied.as_deref().unwrap_or(owned.path());
    if supplied.is_some() {
        let builder = std::fs::DirBuilder::new();
        #[cfg(unix)]
        let builder = {
            use std::os::unix::fs::DirBuilderExt;
            let mut builder = builder;
            builder.mode(0o700);
            builder
        };
        builder
            .create(root)
            .expect("fresh explicit disposable native-file fixture");
    }
    let scenario =
        careops_ed::parse_scenario_json(include_str!("../examples/nominal_mvp.json")).unwrap();
    let config = BatchConfig {
        batch_id: "r131-domain-run".into(),
        scenario_id: scenario.scenario_id.clone(),
        base_seed: 7,
        replications: 4,
        num_workers: 2,
        horizon_override: Some(8),
        ..Default::default()
    };
    let doc = run_batch_with_workers(&scenario, &config, None, None, None).unwrap();
    let patients = patient_table(&scenario, &doc).unwrap();
    assert_eq!(patients.height(), 20);
    let parsed = serde_json::to_vec(&scenario).unwrap();
    let mut effective = scenario.clone();
    effective.seed = 7;
    effective.horizon_ticks = 8;
    let effective = serde_json::to_vec(&effective).unwrap();
    let effective_hash = compute_sha256_hex(&effective);
    let parsed_hash = compute_sha256_hex(&parsed);
    assert_ne!(effective_hash, parsed_hash);
    let input_column = patients.column("input_sha256").unwrap().str().unwrap();
    assert!((0..patients.height()).all(|i| input_column.get(i) == Some(effective_hash.as_str())));
    std::fs::write(root.join("effective-input.json"), effective).unwrap();
    std::fs::write(root.join("parsed-input.json"), parsed).unwrap();
    let inputs = BTreeMap::from([
        (
            "careops.effective_input_sha256".into(),
            effective_hash.clone(),
        ),
        ("careops.parsed_input_sha256".into(), parsed_hash.clone()),
        (
            "careops.input_hash_basis".into(),
            "normalized-effective-scenario".into(),
        ),
        ("careops.run_id".into(), doc.batch_id.clone()),
        (
            "careops.model_contract_sha256".into(),
            careops_ed::capture::MODEL_CONTRACT_SHA256.into(),
        ),
    ]);
    let table = calibration_table(&calibration(), "r131-fixture", "tick").unwrap();
    let tables = vec![
        emit(
            root,
            "patients",
            "careops.ed.patients.v2",
            "original_run_capture",
            &patients,
            inputs,
        ),
        emit(
            root,
            "calibration",
            "careops.ed.synthetic-fixture-calibration.v2",
            "caller_supplied_synthetic_calibration",
            &table,
            BTreeMap::from([
                ("careops.fixture_id".into(), "r131-fixture".into()),
                ("careops.source_kind".into(), "synthetic_fixture".into()),
            ]),
        ),
        emit(
            root,
            "physical",
            "careops.tests.physical-edges.v2",
            "synthetic_physical_edge_vector",
            &physical(),
            BTreeMap::new(),
        ),
        emit(
            root,
            "multiple_batches",
            "careops.tests.multiple-batches.v2",
            "synthetic_multi_batch_row_vector",
            &multiple_batches(),
            BTreeMap::new(),
        ),
        emit(
            root,
            "empty",
            "careops.tests.physical-edges.v2",
            "typed_empty_projection",
            &physical().clear(),
            BTreeMap::new(),
        ),
    ];
    let manifest = json!({"schema":"careops.tests.native-files.v2","status":"complete","effective_input_sha256":effective_hash,"parsed_input_sha256":parsed_hash,"raw_input_sha256":null,"model_contract_sha256":careops_ed::capture::MODEL_CONTRACT_SHA256,"csv":{"quote_style":"necessary","null":"unquoted empty","quoted_empty":"empty String","encoding":"UTF-8"},"tables":tables});
    use std::io::Write;
    std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(root.join("manifest.json"))
        .unwrap()
        .write_all(&serde_json::to_vec_pretty(&manifest).unwrap())
        .unwrap();
}
