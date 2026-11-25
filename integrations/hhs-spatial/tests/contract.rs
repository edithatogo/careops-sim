use arrow_array::{
    ArrayRef, BooleanArray, Float32Array, Int8Array, Int64Array, RecordBatch, StringArray,
};
use arrow_schema::{DataType, Field, Schema};
use careops_hhs_spatial::{Assumptions, Session, load};
use parquet::{arrow::ArrowWriter, basic::Compression, file::properties::WriterProperties};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    sync::atomic::{AtomicUsize, Ordering},
};
fn hash(p: &Path) -> String {
    Sha256::digest(std::fs::read(p).unwrap())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixture-packages")
        .join(name)
}
fn assumptions() -> Assumptions {
    Assumptions {
        accept_provisional: true,
        assumed_metres_per_source_unit: 1.,
        speed_m_per_s: 2.,
        allowed_edge_ids: vec!["e0".into(), "e1".into()],
    }
}
fn path(names: &[&str]) -> Vec<String> {
    names.iter().map(|s| s.to_string()).collect()
}
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        Self::from_fixture("base")
    }
    fn from_fixture(name: &str) -> Self {
        let p = std::env::temp_dir().join(format!(
            "hhs-spatial-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&p).unwrap();
        for entry in std::fs::read_dir(fixture(name)).unwrap() {
            let entry = entry.unwrap();
            std::fs::copy(entry.path(), p.join(entry.file_name())).unwrap();
        }
        Self(p)
    }
    fn campus_v2() -> Self {
        let p = std::env::temp_dir().join(format!(
            "hhs-spatial-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&p).unwrap();
        write_campus_v2(&p);
        Self(p)
    }
    fn manifest(&self, change: impl FnOnce(&mut Value)) -> String {
        let p = self.0.join("manifest.json");
        let mut v: Value = serde_json::from_slice(&std::fs::read(&p).unwrap()).unwrap();
        change(&mut v);
        std::fs::write(&p, serde_json::to_vec(&v).unwrap()).unwrap();
        hash(&p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
#[test]
fn python_parquet_roundtrip_preserves_null_empty_wkb_and_frame() {
    let p = fixture("base");
    let x = load(&p, &hash(&p.join("manifest.json"))).unwrap();
    assert_eq!(x.objects.len(), 3);
    assert!(x.objects[0]["label"].is_null());
    assert_eq!(x.objects[1]["label"], "");
    assert_eq!(x.objects[2]["enabled"], false);
    assert!(
        x.objects[0]["geometry_wkb"]
            .as_str()
            .unwrap()
            .starts_with("0103000000")
    );
    assert_eq!(x.manifest["frame"]["origin"], json!([100., 200.]));
    assert_eq!(x.model["history"], json!([]));
    assert_eq!(x.nodes.len(), 4);
    assert_eq!(x.edges.len(), 2);
    assert_eq!(
        x.travel_seconds(&path(&["n0", "n1", "n2"]), &assumptions())
            .unwrap(),
        3.5
    );
    assert_eq!(
        x.travel_seconds(&path(&["n2", "n1", "n0"]), &assumptions())
            .unwrap(),
        3.5
    );
    let mut scale = assumptions();
    scale.assumed_metres_per_source_unit = 2.;
    assert_eq!(
        x.travel_seconds(&path(&["n0", "n1", "n2"]), &scale)
            .unwrap(),
        7.
    );
    assert_eq!(x.manifest["units_verified"], false);
}
#[test]
fn policy_disconnection_and_missing_scale_reject() {
    let p = fixture("base");
    let x = load(&p, &hash(&p.join("manifest.json"))).unwrap();
    for nodes in [&["n0", "isolated"][..], &["missing", "n0"], &["n0"]] {
        assert!(x.travel_seconds(&path(nodes), &assumptions()).is_err());
    }
    let mut a = assumptions();
    a.allowed_edge_ids.clear();
    assert!(x.travel_seconds(&path(&["n0", "n1"]), &a).is_err());
    for v in [0., -1., f64::INFINITY, f64::NAN] {
        let mut a = assumptions();
        a.assumed_metres_per_source_unit = v;
        assert!(x.travel_seconds(&path(&["n0", "n1"]), &a).is_err());
        let mut a = assumptions();
        a.speed_m_per_s = v;
        assert!(x.travel_seconds(&path(&["n0", "n1"]), &a).is_err());
    }
    let mut a = assumptions();
    a.accept_provisional = false;
    assert!(x.travel_seconds(&path(&["n0", "n1"]), &a).is_err());
    let mut a = assumptions();
    a.assumed_metres_per_source_unit = f64::MAX;
    assert!(x.travel_seconds(&path(&["n0", "n1"]), &a).is_err());
}
#[test]
fn unsupported_schema_flags_units_counts_and_identity_reject() {
    for (key, value) in [
        ("schema_version", json!(99)),
        ("format", json!("other")),
        ("provisional", json!(false)),
        ("spatial_acceptance", json!(true)),
        ("units_verified", json!(true)),
        ("floor_id", json!("wrong")),
        ("source_sha256", json!("bad")),
        ("revision_sha256", json!("0".repeat(64))),
        ("graph_status", json!("unavailable")),
        ("classification", json!("public")),
        ("assumptions", json!([])),
        ("exclusions", json!([])),
    ] {
        let t = Temp::new();
        let h = t.manifest(|v| v[key] = value);
        assert!(load(&t.0, &h).is_err(), "{key}");
    }
    for (key, value) in [
        ("scale", json!(1000)),
        ("unit", json!("m")),
        ("origin", json!([1.])),
        ("forward", json!("bad")),
        ("units_verified", json!(true)),
    ] {
        let t = Temp::new();
        let h = t.manifest(|v| v["frame"][key] = value);
        assert!(load(&t.0, &h).is_err(), "{key}");
    }
    let t = Temp::new();
    let h = t.manifest(|v| v["row_counts"]["nodes"] = json!(1));
    assert!(load(&t.0, &h).is_err());
}
#[test]
fn corruption_partial_and_extra_file_fail_closed() {
    let t = Temp::new();
    let h = hash(&t.0.join("manifest.json"));
    assert!(load(&t.0, &"0".repeat(64)).is_err());
    std::fs::write(t.0.join("nodes.parquet"), b"partial").unwrap();
    assert!(load(&t.0, &h).is_err());
    let t = Temp::new();
    let h = hash(&t.0.join("manifest.json"));
    std::fs::write(t.0.join("extra"), b"unexpected").unwrap();
    assert!(load(&t.0, &h).is_err());
    let t = Temp::new();
    let h = hash(&t.0.join("manifest.json"));
    std::fs::remove_file(t.0.join("edges.parquet")).unwrap();
    assert!(load(&t.0, &h).is_err());
}
#[test]
fn symlink_artifact_is_rejected() {
    use std::os::unix::fs::symlink;
    let t = Temp::new();
    let h = hash(&t.0.join("manifest.json"));
    std::fs::remove_file(t.0.join("nodes.parquet")).unwrap();
    symlink(fixture("base/nodes.parquet"), t.0.join("nodes.parquet")).unwrap();
    assert!(load(&t.0, &h).is_err());
}
#[test]
fn failed_replacement_preserves_active_revision_and_checkpoint_rolls_back() {
    let base = fixture("base");
    let changed = fixture("changed");
    let h = hash(&base.join("manifest.json"));
    let h2 = hash(&changed.join("manifest.json"));
    let mut s = Session::default();
    assert!(s.checkpoint().is_err());
    s.replace(&base, &h).unwrap();
    let checkpoint = s.checkpoint().unwrap();
    let t = Temp::new();
    std::fs::write(t.0.join("objects.parquet"), b"interrupted transfer").unwrap();
    assert!(s.replace(&t.0, &h).is_err());
    assert_eq!(s.current.as_ref().unwrap().manifest_sha256, h);
    s.replace(&changed, &h2).unwrap();
    assert_eq!(s.current.as_ref().unwrap().objects[0]["label"], "Revised");
    assert!(s.restore(&base, b"{\"schema_version\":99}").is_err());
    assert_eq!(s.current.as_ref().unwrap().manifest_sha256, h2);
    s.restore(&base, &checkpoint).unwrap();
    assert_eq!(s.current.as_ref().unwrap().manifest_sha256, h);
    assert!(s.current.as_ref().unwrap().objects[0]["label"].is_null());
}
#[test]
fn unavailable_graph_remains_unavailable() {
    let p = fixture("unavailable");
    let x = load(&p, &hash(&p.join("manifest.json"))).unwrap();
    assert!(x.nodes.is_empty());
    assert!(
        x.travel_seconds(&path(&["n0", "n1"]), &assumptions())
            .is_err()
    );
}
#[test]
fn negative_parquet_contracts_are_rejected() {
    let root = fixture("negative");
    for entry in std::fs::read_dir(root).unwrap() {
        let p = entry.unwrap().path();
        assert!(
            load(&p, &hash(&p.join("manifest.json"))).is_err(),
            "{}",
            p.display()
        );
    }
}

#[test]
fn command_line_inspection_and_timing_preserve_explicit_assumptions() {
    let binary = env!("CARGO_BIN_EXE_hhs-spatial");
    let package = fixture("base");
    let digest = hash(&package.join("manifest.json"));
    let inspect = std::process::Command::new(binary)
        .arg(&package)
        .arg(&digest)
        .output()
        .unwrap();
    assert!(inspect.status.success());
    let value: Value = serde_json::from_slice(&inspect.stdout).unwrap();
    assert_eq!(value["manifest"]["provisional"], true);
    let temp = Temp::new();
    let route = temp.0.join("route.json");
    std::fs::write(
        &route,
        serde_json::to_vec(&json!({"path":["n0","n1","n2"],"assumptions":assumptions()})).unwrap(),
    )
    .unwrap();
    let result = std::process::Command::new(binary)
        .arg(&package)
        .arg(&digest)
        .arg(&route)
        .output()
        .unwrap();
    assert!(result.status.success());
    let value: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(value["travel_seconds"], 3.5);
    assert_eq!(value["units_verified"], false);
    assert_eq!(value["spatial_acceptance"], false);
    assert!(
        !std::process::Command::new(binary)
            .output()
            .unwrap()
            .status
            .success()
    );
}
fn write_parquet(path: &Path, schema: Arc<Schema>, columns: Vec<ArrayRef>) -> String {
    let file = std::fs::File::create(path).unwrap();
    let props = WriterProperties::builder()
        .set_compression(Compression::ZSTD(Default::default()))
        .build();
    let mut writer = ArrowWriter::try_new(file, schema.clone(), Some(props)).unwrap();
    let batch = RecordBatch::try_new(schema, columns).unwrap();
    writer.write(&batch).unwrap();
    writer.close().unwrap();
    hash(path)
}
fn write_campus_v2(dir: &Path) {
    let spaces_schema = Arc::new(Schema::new(vec![
        Field::new("globalId", DataType::Utf8, false),
        Field::new("storey", DataType::Utf8, false),
        Field::new("building", DataType::Utf8, false),
        Field::new("centroidX", DataType::Float32, false),
        Field::new("centroidY", DataType::Int8, false),
        Field::new("connectedSpaceCount", DataType::Int8, false),
        Field::new("connectedSpaceIds", DataType::Utf8, false),
        Field::new("floorId", DataType::Utf8, false),
        Field::new("sourceReadiness", DataType::Utf8, false),
        Field::new("area", DataType::Float32, true),
    ]));
    let spaces_hash = write_parquet(
        &dir.join("spaces.parquet"),
        spaces_schema,
        vec![
            Arc::new(StringArray::from(vec!["s1", "s2", "s_mh"])),
            Arc::new(StringArray::from(vec!["B L1", "B L1", "MH L0"])),
            Arc::new(StringArray::from(vec!["B", "B", "MH"])),
            Arc::new(Float32Array::from(vec![0.0, 6.0, 1.0])),
            Arc::new(Int8Array::from(vec![0, 8, 1])),
            Arc::new(Int8Array::from(vec![1, 1, 0])),
            Arc::new(StringArray::from(vec!["[\"s2\"]", "[\"s1\"]", "[]"])),
            Arc::new(StringArray::from(vec!["B-L1", "B-L1", "MH-L0"])),
            Arc::new(StringArray::from(vec![
                "available-plan",
                "available-plan",
                "missing-plan",
            ])),
            Arc::new(Float32Array::from(vec![None, Some(8.25), Some(1.5)])),
        ],
    );
    let doors_schema = Arc::new(Schema::new(vec![
        Field::new("globalId", DataType::Utf8, false),
        Field::new("storey", DataType::Utf8, false),
        Field::new("connectedSpaceCount", DataType::Int64, false),
        Field::new("connectedSpaceIds", DataType::Utf8, false),
        Field::new("isStairAccess", DataType::Boolean, false),
        Field::new("isInterSpaceDoor", DataType::Boolean, false),
        Field::new("floorId", DataType::Utf8, false),
        Field::new("sourceReadiness", DataType::Utf8, false),
    ]));
    let doors_hash = write_parquet(
        &dir.join("doors.parquet"),
        doors_schema,
        vec![
            Arc::new(StringArray::from(vec!["d1", "d_stair"])),
            Arc::new(StringArray::from(vec!["B L1", "B L1"])),
            Arc::new(Int64Array::from(vec![2, 1])),
            Arc::new(StringArray::from(vec!["[\"s1\", \"s2\"]", "[\"s1\"]"])),
            Arc::new(BooleanArray::from(vec![false, true])),
            Arc::new(BooleanArray::from(vec![true, false])),
            Arc::new(StringArray::from(vec!["B-L1", "B-L1"])),
            Arc::new(StringArray::from(vec!["available-plan", "available-plan"])),
        ],
    );
    let scopes = [
        "B L1", "A L5", "E L3", "E L4", "E L5", "E L6", "E L7", "MH L0", "MH L1", "MH L2", "MH L3",
    ];
    let floor_ids: Vec<String> = scopes.iter().map(|s| s.replace(' ', "-")).collect();
    let buildings: Vec<&str> = scopes
        .iter()
        .map(|s| s.split_whitespace().next().unwrap())
        .collect();
    let readiness: Vec<&str> = scopes
        .iter()
        .map(|s| {
            if *s == "B L1" {
                "available-plan"
            } else {
                "missing-plan"
            }
        })
        .collect();
    let dispositions: Vec<&str> = scopes
        .iter()
        .map(|s| {
            if *s == "B L1" {
                "provisional-available-plan-partial-coverage"
            } else if s.starts_with("MH ") {
                "provisional-missing-plan-full-cad-topology"
            } else {
                "provisional-missing-plan-shell-core-only"
            }
        })
        .collect();
    let export_status: Vec<&str> = scopes
        .iter()
        .map(|s| {
            if *s == "B L1" || s.starts_with("MH ") {
                "provisional"
            } else {
                "blocked-shell-core"
            }
        })
        .collect();
    let space_counts: Vec<i64> = scopes
        .iter()
        .map(|s| match *s {
            "B L1" => 2,
            "A L5" | "MH L0" => 1,
            _ => 0,
        })
        .collect();
    let door_counts: Vec<i64> = scopes
        .iter()
        .map(|s| match *s {
            "B L1" => 2,
            "A L5" => 1,
            _ => 0,
        })
        .collect();
    let floors_schema = Arc::new(Schema::new(vec![
        Field::new("floorId", DataType::Utf8, false),
        Field::new("scope", DataType::Utf8, false),
        Field::new("buildingCode", DataType::Utf8, false),
        Field::new("sourceReadiness", DataType::Utf8, false),
        Field::new("floorAcceptanceDisposition", DataType::Utf8, false),
        Field::new("topologySpaceCount", DataType::Int64, false),
        Field::new("topologyDoorCount", DataType::Int64, false),
        Field::new("finalAcceptanceReady", DataType::Boolean, false),
        Field::new("physicalVerification", DataType::Boolean, false),
        Field::new("exportStatus", DataType::Utf8, false),
    ]));
    let floors_hash = write_parquet(
        &dir.join("floors.parquet"),
        floors_schema,
        vec![
            Arc::new(StringArray::from(
                floor_ids.iter().map(String::as_str).collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(scopes.to_vec())),
            Arc::new(StringArray::from(buildings)),
            Arc::new(StringArray::from(readiness)),
            Arc::new(StringArray::from(dispositions)),
            Arc::new(Int64Array::from(space_counts)),
            Arc::new(Int64Array::from(door_counts)),
            Arc::new(BooleanArray::from(vec![false; scopes.len()])),
            Arc::new(BooleanArray::from(vec![false; scopes.len()])),
            Arc::new(StringArray::from(export_status)),
        ],
    );
    let src_hash = "a".repeat(64);
    let manifest = json!({
        "schema_version": 2,
        "format": "hhs-provisional-campus-spatial",
        "classification": "synthetic",
        "provisional": true,
        "spatial_acceptance": false,
        "finalAcceptanceReady": false,
        "physicalVerification": false,
        "units_verified": false,
        "frame": {"type": "local_cartesian", "unit": "source_units", "units_verified": false},
        "source_sha256": {"spaces": src_hash, "doors": src_hash, "floors": src_hash},
        "input_row_counts": {"spaces": 4, "doors": 3, "floors": 11},
        "row_counts": {"spaces": 3, "doors": 2, "floors": 11},
        "excluded_row_counts": {"spaces": 1, "doors": 1},
        "blocked_floor_scopes": ["A L5", "E L3", "E L4", "E L5", "E L6", "E L7"],
        "coordinate_mapping": null,
        "coordinate_semantics": "Space centroids only; local source representation coordinates.",
        "topology_semantics": "Provisional same-floor connectivity inferred from source geometry.",
        "files": {
            "doors.parquet": doors_hash,
            "floors.parquet": floors_hash,
            "spaces.parquet": spaces_hash,
        }
    });
    std::fs::write(
        dir.join("manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
}
#[test]
fn v2_campus_roundtrip_preserves_flags_exclusions_and_routes() {
    let temp = Temp::campus_v2();
    let p = &temp.0;
    let digest = hash(&p.join("manifest.json"));
    let x = load(p, &digest).unwrap();
    assert_eq!(x.manifest["schema_version"], 2);
    assert_eq!(x.manifest["format"], "hhs-provisional-campus-spatial");
    assert_eq!(x.manifest["provisional"], true);
    assert_eq!(x.manifest["spatial_acceptance"], false);
    assert_eq!(x.manifest["finalAcceptanceReady"], false);
    assert_eq!(x.manifest["physicalVerification"], false);
    assert_eq!(x.manifest["units_verified"], false);
    assert_eq!(x.spaces.len(), 3);
    assert!(x.spaces[0]["area"].is_null());
    assert_eq!(x.spaces[1]["area"], 8.25);
    assert_eq!(x.doors.len(), 2);
    assert_eq!(x.floors.len(), 11);
    assert_eq!(x.edges.len(), 1);
    assert_eq!(
        x.manifest["excluded_row_counts"],
        json!({"spaces": 1, "doors": 1})
    );
    let v2_assumptions = Assumptions {
        accept_provisional: true,
        assumed_metres_per_source_unit: 1.0,
        speed_m_per_s: 2.0,
        allowed_edge_ids: vec!["d1".into()],
    };
    assert_eq!(
        x.travel_seconds(&path(&["s1", "s2"]), &v2_assumptions)
            .unwrap(),
        5.0
    );
    let mut session = Session::default();
    session.replace(p, &digest).unwrap();
    let cp = session.checkpoint().unwrap();
    let mut restored = Session::default();
    restored.restore(p, &cp).unwrap();
    assert_eq!(restored.current.unwrap().manifest_sha256, digest);
}
#[test]
fn python_producer_v2_preserves_complete_campus_rows() {
    let expected: Value =
        serde_json::from_slice(include_bytes!("fixtures/campus-v2-expected.json")).unwrap();
    let package = load(
        &fixture("campus-v2"),
        expected["manifest_sha256"].as_str().unwrap(),
    )
    .unwrap();
    let observed = serde_json::to_value(&package).unwrap();
    for field in ["manifest_sha256", "manifest", "spaces", "doors", "floors"] {
        assert_eq!(observed[field], expected[field], "{field} parity");
    }
    assert!(package.spaces[0]["name"].is_null());
    assert_eq!(package.spaces[1]["name"], "");
    assert_eq!(package.nodes.len(), 3);
    assert_eq!(package.edges.len(), 1);
    let mut route = assumptions();
    route.allowed_edge_ids = vec!["d1".into()];
    assert_eq!(
        package
            .travel_seconds(&path(&["s1", "s2"]), &route)
            .unwrap(),
        2.0_f64.sqrt()
    );
}

#[test]
fn v2_campus_negative_contracts_and_epsg_frame() {
    let epsg = Temp::campus_v2();
    let epsg_hash = epsg.manifest(|m| {
        m["frame"] = json!({"type": "EPSG:7855", "unit": "metres", "units_verified": false});
        m["coordinate_mapping"] = json!({
            "schema_version": 1,
            "type": "EPSG:7855",
            "unit": "metres",
            "units_verified": false,
            "physicalVerification": false,
            "method": "fictional projected coordinate fixture",
            "spaces_sha256": m["source_sha256"]["spaces"],
            "floor_matrix_sha256": m["source_sha256"]["floors"],
            "coordinates_sha256": "b".repeat(64),
            "metadata_sha256": "c".repeat(64),
        });
    });
    let loaded = load(&epsg.0, &epsg_hash).unwrap();
    assert_eq!(loaded.manifest["frame"]["type"], "EPSG:7855");
    assert_eq!(loaded.manifest["frame"]["unit"], "metres");
    let mut route = assumptions();
    route.allowed_edge_ids = vec!["d1".into()];
    assert!(loaded.travel_seconds(&path(&["s1", "s2"]), &route).is_ok());
    route.assumed_metres_per_source_unit = 2.0;
    assert!(loaded.travel_seconds(&path(&["s1", "s2"]), &route).is_err());
    let changed_mapping = epsg.manifest(|m| {
        m["coordinate_mapping"]["spaces_sha256"] = json!("d".repeat(64));
    });
    assert!(load(&epsg.0, &changed_mapping).is_err());

    let bad_cases: [fn(&mut Value); 9] = [
        |m| m["finalAcceptanceReady"] = json!(true),
        |m| m["physicalVerification"] = json!(true),
        |m| m["units_verified"] = json!(true),
        |m| m["format"] = json!("hhs-provisional-spatial"),
        |m| {
            m["frame"] =
                json!({"type": "EPSG:7855", "unit": "source_units", "units_verified": false})
        },
        |m| m["frame"] = json!({"type": "EPSG:7855", "unit": "metres", "units_verified": false}),
        |m| m["excluded_row_counts"]["spaces"] = json!(99),
        |m| {
            m["input_row_counts"]["spaces"] = json!(3);
            m["excluded_row_counts"]["spaces"] = json!(0);
        },
        |m| m["blocked_floor_scopes"] = json!(["A L5"]),
    ];
    for change in bad_cases {
        let bad = Temp::campus_v2();
        let bad_hash = bad.manifest(change);
        assert!(load(&bad.0, &bad_hash).is_err());
    }
    let extra = Temp::campus_v2();
    std::fs::write(extra.0.join("extra.txt"), b"x").unwrap();
    assert!(load(&extra.0, &hash(&extra.0.join("manifest.json"))).is_err());
}
