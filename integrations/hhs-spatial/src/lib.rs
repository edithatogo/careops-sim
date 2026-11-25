//! Immutable provisional spatial loading and explicit-assumption CareOps routing.
use arrow_array::{
    Array, BinaryArray, BinaryViewArray, BooleanArray, Float32Array, Float64Array, Int8Array,
    Int16Array, Int32Array, Int64Array, LargeBinaryArray, LargeStringArray, RecordBatch,
    StringArray, StringViewArray, UInt8Array, UInt16Array, UInt32Array, UInt64Array,
};
use arrow_schema::DataType;
use bytes::Bytes;
use careops_ed::disposition::operations::{
    ExecutionFidelity, MovementEdge, SyntheticGraph, calculate_execution_duration,
};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashSet},
    error::Error,
    path::Path,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const MAX_FILE: u64 = 128 * 1024 * 1024;
const MAX_ROWS: usize = 1_000_000;
fn hash(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn require(ok: bool, message: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(message.to_owned().into())
    }
}
fn read(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let metadata = std::fs::symlink_metadata(path)?;
    require(
        metadata.is_file() && !metadata.file_type().is_symlink() && metadata.len() <= limit,
        "regular bounded file required",
    )?;
    let data = std::fs::read(path)?;
    require(data.len() as u64 <= limit, "file limit")?;
    Ok(data)
}
fn text(array: &dyn Array, row: usize) -> Result<String> {
    require(!array.is_null(row), "null identifier")?;
    let s = if let Some(a) = array.as_any().downcast_ref::<StringArray>() {
        a.value(row)
    } else if let Some(a) = array.as_any().downcast_ref::<LargeStringArray>() {
        a.value(row)
    } else if let Some(a) = array.as_any().downcast_ref::<StringViewArray>() {
        a.value(row)
    } else {
        return Err("invalid string type".into());
    };
    Ok(s.to_owned())
}
fn cell(array: &dyn Array, kind: &str, row: usize) -> Result<Value> {
    if kind == "label" && array.is_null(row) {
        return Ok(Value::Null);
    }
    require(!array.is_null(row), "unexpected null")?;
    match kind {
        "text" | "label" => Ok(json!(text(array, row)?)),
        "number" => {
            let v = if let Some(a) = array.as_any().downcast_ref::<Float64Array>() {
                a.value(row)
            } else if let Some(a) = array.as_any().downcast_ref::<Float32Array>() {
                f64::from(a.value(row))
            } else {
                return Err("float required".into());
            };
            require(v.is_finite(), "finite coordinate required")?;
            Ok(json!(v))
        }
        "bool" => Ok(json!(
            array
                .as_any()
                .downcast_ref::<BooleanArray>()
                .ok_or("boolean required")?
                .value(row)
        )),
        "int" => Ok(
            if let Some(a) = array.as_any().downcast_ref::<Int8Array>() {
                json!(a.value(row))
            } else if let Some(a) = array.as_any().downcast_ref::<Int16Array>() {
                json!(a.value(row))
            } else if let Some(a) = array.as_any().downcast_ref::<Int64Array>() {
                json!(a.value(row))
            } else if let Some(a) = array.as_any().downcast_ref::<Int32Array>() {
                json!(a.value(row))
            } else if let Some(a) = array.as_any().downcast_ref::<UInt64Array>() {
                json!(a.value(row))
            } else if let Some(a) = array.as_any().downcast_ref::<UInt32Array>() {
                json!(a.value(row))
            } else if let Some(a) = array.as_any().downcast_ref::<UInt8Array>() {
                json!(a.value(row))
            } else if let Some(a) = array.as_any().downcast_ref::<UInt16Array>() {
                json!(a.value(row))
            } else {
                return Err("integer required".into());
            },
        ),
        "binary" => {
            let data = if let Some(a) = array.as_any().downcast_ref::<BinaryArray>() {
                a.value(row)
            } else if let Some(a) = array.as_any().downcast_ref::<LargeBinaryArray>() {
                a.value(row)
            } else if let Some(a) = array.as_any().downcast_ref::<BinaryViewArray>() {
                a.value(row)
            } else {
                return Err("binary WKB required".into());
            };
            require(
                data.len() >= 5 && data.len() <= MAX_FILE as usize,
                "bounded WKB required",
            )?;
            Ok(json!(hex(data)))
        }
        _ => Err("unknown field type".into()),
    }
}
fn table(data: Vec<u8>, columns: &[(&str, &str)], count: usize) -> Result<Vec<Value>> {
    require(count <= MAX_ROWS, "row limit")?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(Bytes::from(data))?;
    require(
        builder.metadata().file_metadata().num_rows() == count as i64,
        "row count mismatch",
    )?;
    require(
        builder.schema().fields().len() == columns.len(),
        "column count mismatch",
    )?;
    for (field, (name, kind)) in builder.schema().fields().iter().zip(columns) {
        let valid = match *kind {
            "text" | "label" => matches!(
                field.data_type(),
                DataType::Utf8 | DataType::LargeUtf8 | DataType::Utf8View
            ),
            "number" => *field.data_type() == DataType::Float64,
            "bool" => *field.data_type() == DataType::Boolean,
            "binary" => matches!(
                field.data_type(),
                DataType::Binary | DataType::LargeBinary | DataType::BinaryView
            ),
            _ => false,
        };
        require(field.name() == name && valid, "column schema mismatch")?;
    }
    let mut values = Vec::new();
    let mut ids = HashSet::new();
    for batch in builder.with_batch_size(1024).build()? {
        let batch: RecordBatch = batch?;
        for row in 0..batch.num_rows() {
            let mut record = serde_json::Map::new();
            for (i, (name, kind)) in columns.iter().enumerate() {
                record.insert((*name).into(), cell(batch.column(i).as_ref(), kind, row)?);
            }
            let id = record["id"].as_str().ok_or("id required")?;
            require(
                !id.trim().is_empty() && ids.insert(id.to_owned()),
                "duplicate/empty id",
            )?;
            values.push(Value::Object(record));
        }
    }
    require(values.len() == count, "decoded count mismatch")?;
    Ok(values)
}
#[derive(Clone, Debug, Serialize)]
pub struct Package {
    pub manifest_sha256: String,
    pub manifest: Value,
    pub model: Value,
    pub objects: Vec<Value>,
    pub nodes: Vec<Value>,
    pub edges: Vec<Value>,
    pub spaces: Vec<Value>,
    pub doors: Vec<Value>,
    pub floors: Vec<Value>,
}
pub fn load(directory: &Path, expected_hash: &str) -> Result<Package> {
    require(
        !std::fs::symlink_metadata(directory)?
            .file_type()
            .is_symlink(),
        "package link rejected",
    )?;
    let bytes = read(&directory.join("manifest.json"), 1024 * 1024)?;
    require(hash(&bytes) == expected_hash, "manifest hash mismatch")?;
    let m: Value = serde_json::from_slice(&bytes)?;
    match m["schema_version"].as_u64() {
        Some(1) => load_v1(directory, expected_hash, m),
        Some(2) => load_v2(directory, expected_hash, m),
        _ => Err("unsupported schema".into()),
    }
}
fn load_v1(directory: &Path, expected_hash: &str, m: Value) -> Result<Package> {
    require(
        m["schema_version"] == 1 && m["format"] == "hhs-provisional-spatial",
        "unsupported schema",
    )?;
    require(
        m["provisional"] == true
            && m["spatial_acceptance"] == false
            && m["units_verified"] == false,
        "provisional flags required",
    )?;
    require(
        matches!(
            m["classification"].as_str(),
            Some("synthetic" | "private_source_derived")
        ),
        "classification required",
    )?;
    let frame = &m["frame"];
    require(
        frame["type"] == "local_cartesian"
            && frame["unit"] == "source_units"
            && frame["units_verified"] == false
            && frame["scale"].as_f64() == Some(1.0)
            && frame["forward"] == "local=(source-origin)*scale"
            && frame["inverse"] == "source=local/scale+origin",
        "unsupported coordinate frame",
    )?;
    let origin = frame["origin"].as_array().ok_or("origin required")?;
    require(
        origin.len() == 2
            && origin
                .iter()
                .all(|x| x.as_f64().is_some_and(f64::is_finite)),
        "invalid origin",
    )?;
    for name in ["source_sha256", "revision_sha256"] {
        let s = m[name]
            .as_str()
            .ok_or("source/revision identity required")?;
        require(
            s.len() == 64 && s.bytes().all(|c| c.is_ascii_hexdigit()),
            "invalid identity hash",
        )?;
    }
    require(
        m["floor_id"].as_str().is_some_and(|s| !s.is_empty())
            && frame["id"].as_str().is_some_and(|s| !s.is_empty()),
        "floor/frame id required",
    )?;
    for field in ["assumptions", "exclusions"] {
        require(
            m[field].as_array().is_some_and(|a| {
                !a.is_empty() && a.iter().all(|s| s.as_str().is_some_and(|s| !s.is_empty()))
            }),
            "assumptions/exclusions required",
        )?;
    }
    let expected = [
        "edges.parquet",
        "manifest.json",
        "model.json",
        "nodes.parquet",
        "objects.parquet",
    ];
    let mut names = std::fs::read_dir(directory)?
        .map(|p| p.map(|p| p.file_name().to_string_lossy().into_owned()))
        .collect::<std::io::Result<Vec<_>>>()?;
    names.sort();
    require(names == expected, "unexpected package files")?;
    let files = m["files"].as_object().ok_or("file inventory required")?;
    require(files.len() == 4, "file inventory mismatch")?;
    let mut data = BTreeMap::new();
    for name in expected.into_iter().filter(|n| *n != "manifest.json") {
        let bytes = read(&directory.join(name), MAX_FILE)?;
        require(
            files.get(name).and_then(Value::as_str) == Some(hash(&bytes).as_str()),
            "artifact hash mismatch",
        )?;
        data.insert(name, bytes);
    }
    let model: Value = serde_json::from_slice(&data.remove("model.json").ok_or("model missing")?)?;
    require(
        model["frame"] == m["frame"]
            && model["source_sha256"] == m["source_sha256"]
            && model["revision_sha256"] == m["revision_sha256"]
            && model["floor_id"] == m["floor_id"]
            && model["provisional"] == true
            && model["spatial_acceptance"] == false
            && model["schema_version"] == 1,
        "model identity mismatch",
    )?;
    let schemas: [(&str, &[(&str, &str)]); 3] = [
        (
            "objects",
            &[
                ("id", "text"),
                ("role", "text"),
                ("label", "label"),
                ("geometry_wkb", "binary"),
                ("enabled", "bool"),
            ],
        ),
        (
            "nodes",
            &[
                ("id", "text"),
                ("floor_id", "text"),
                ("x", "number"),
                ("y", "number"),
            ],
        ),
        (
            "edges",
            &[
                ("id", "text"),
                ("from_id", "text"),
                ("to_id", "text"),
                ("length", "number"),
            ],
        ),
    ];
    let mut tables = BTreeMap::new();
    for (name, schema) in schemas {
        let count = m["row_counts"][name].as_u64().ok_or("row count required")?;
        require(count <= MAX_ROWS as u64, "row limit")?;
        tables.insert(
            name,
            table(
                data.remove(format!("{name}.parquet").as_str())
                    .ok_or("table missing")?,
                schema,
                count as usize,
            )?,
        );
    }
    let nodes = tables.remove("nodes").unwrap();
    let edges = tables.remove("edges").unwrap();
    let objects = tables.remove("objects").unwrap();
    let originals = model["objects"]
        .as_array()
        .ok_or("model objects required")?;
    require(originals.len() == objects.len(), "object count mismatch")?;
    for object in &objects {
        require(
            matches!(object["role"].as_str(), Some("room" | "wall" | "label")),
            "unknown object role",
        )?;
        let original = originals
            .iter()
            .find(|v| v["id"] == object["id"])
            .ok_or("object identity missing")?;
        require(
            ["role", "label", "enabled"]
                .iter()
                .all(|k| original[k] == object[k]),
            "object metadata mismatch",
        )?;
    }
    for n in &nodes {
        require(n["floor_id"] == m["floor_id"], "mixed floor topology")?;
    }
    for e in &edges {
        let a = nodes
            .iter()
            .find(|n| n["id"] == e["from_id"])
            .ok_or("unresolved edge")?;
        let b = nodes
            .iter()
            .find(|n| n["id"] == e["to_id"])
            .ok_or("unresolved edge")?;
        let distance = (a["x"].as_f64().unwrap() - b["x"].as_f64().unwrap())
            .hypot(a["y"].as_f64().unwrap() - b["y"].as_f64().unwrap());
        require(
            e["length"].as_f64().unwrap() >= 0.0
                && e["length"].as_f64().unwrap() + 1e-8 >= distance,
            "invalid edge length",
        )?;
    }
    require(
        matches!(
            m["graph_status"].as_str(),
            Some("available" | "unavailable")
        ),
        "graph status required",
    )?;
    require(
        m["graph_status"] != "unavailable" || (nodes.is_empty() && edges.is_empty()),
        "unavailable topology",
    )?;
    Ok(Package {
        manifest_sha256: expected_hash.into(),
        manifest: m,
        model,
        objects,
        nodes,
        edges,
        spaces: Vec::new(),
        doors: Vec::new(),
        floors: Vec::new(),
    })
}
fn v2_table(
    data: Vec<u8>,
    required: &[(&str, &str)],
    id_key: &str,
    count: usize,
) -> Result<Vec<Value>> {
    require(count <= MAX_ROWS, "row limit")?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(Bytes::from(data))?;
    require(
        builder.metadata().file_metadata().num_rows() == count as i64,
        "row count mismatch",
    )?;
    let schema = builder.schema().clone();
    let required: BTreeMap<_, _> = required.iter().copied().collect();
    let mut names = HashSet::new();
    let mut columns = Vec::with_capacity(schema.fields().len());
    for field in schema.fields() {
        let name = field.name().as_str();
        require(names.insert(name), "duplicate column name")?;
        let kind = match field.data_type() {
            DataType::Utf8 | DataType::LargeUtf8 | DataType::Utf8View => "text",
            DataType::Float32 | DataType::Float64 => "number",
            DataType::Boolean => "bool",
            DataType::Int8
            | DataType::Int16
            | DataType::Int32
            | DataType::Int64
            | DataType::UInt8
            | DataType::UInt16
            | DataType::UInt32
            | DataType::UInt64 => "int",
            DataType::Null => "null",
            _ => return Err("unsupported campus column type".into()),
        };
        let expected = required.get(name).copied();
        require(
            expected
                .is_none_or(|expected| kind == expected || expected == "number" && kind == "int"),
            "column schema mismatch",
        )?;
        columns.push((name, kind, expected));
    }
    require(
        required.keys().all(|name| names.contains(name)),
        "column schema mismatch",
    )?;
    let mut values = Vec::new();
    let mut ids = HashSet::new();
    for batch in builder.with_batch_size(1024).build()? {
        let batch: RecordBatch = batch?;
        for row in 0..batch.num_rows() {
            let mut record = serde_json::Map::new();
            for (idx, &(name, kind, expected)) in columns.iter().enumerate() {
                let array = batch.column(idx).as_ref();
                let value = if expected.is_none() && array.is_null(row) {
                    Value::Null
                } else {
                    cell(array, kind, row)?
                };
                require(
                    expected != Some("int") || value.as_u64().is_some(),
                    "non-negative integer required",
                )?;
                record.insert(name.into(), value);
            }
            let id = record[id_key].as_str().ok_or("id required")?;
            require(
                !id.trim().is_empty() && ids.insert(id.to_owned()),
                "duplicate/empty id",
            )?;
            values.push(Value::Object(record));
        }
    }
    require(values.len() == count, "decoded count mismatch")?;
    Ok(values)
}
fn parse_connected_ids(row: &Value) -> Result<Vec<String>> {
    let raw = row["connectedSpaceIds"]
        .as_str()
        .ok_or("connectedSpaceIds required")?;
    let count = row["connectedSpaceCount"]
        .as_u64()
        .ok_or("connectedSpaceCount required")? as usize;
    let parsed: Vec<String> =
        serde_json::from_str(raw).map_err(|_| "invalid campus connectivity")?;
    let mut seen = HashSet::new();
    for item in &parsed {
        require(
            !item.is_empty() && seen.insert(item.as_str()),
            "invalid campus connectivity",
        )?;
    }
    require(parsed.len() == count, "invalid campus connectivity")?;
    Ok(parsed)
}
fn is_hex64(value: &Value) -> bool {
    value
        .as_str()
        .is_some_and(|s| s.len() == 64 && s.bytes().all(|c| c.is_ascii_hexdigit()))
}
fn load_v2(directory: &Path, expected_hash: &str, m: Value) -> Result<Package> {
    require(
        m["schema_version"] == 2 && m["format"] == "hhs-provisional-campus-spatial",
        "unsupported schema",
    )?;
    require(
        m["provisional"] == true
            && m["spatial_acceptance"] == false
            && m["finalAcceptanceReady"] == false
            && m["physicalVerification"] == false
            && m["units_verified"] == false,
        "provisional flags required",
    )?;
    require(
        matches!(
            m["classification"].as_str(),
            Some("synthetic" | "private_source_derived")
        ),
        "classification required",
    )?;
    let frame = &m["frame"];
    require(
        frame["units_verified"] == false
            && ((frame["type"] == "local_cartesian" && frame["unit"] == "source_units")
                || (frame["type"] == "EPSG:7855" && frame["unit"] == "metres")),
        "unsupported coordinate frame",
    )?;
    let src_hashes = m["source_sha256"]
        .as_object()
        .ok_or("source_sha256 required")?;
    for key in ["spaces", "doors", "floors"] {
        require(
            src_hashes.get(key).is_some_and(is_hex64),
            "invalid identity hash",
        )?;
    }
    let mapping = m
        .get("coordinate_mapping")
        .ok_or("coordinate mapping field required")?;
    if frame["type"] == "EPSG:7855" {
        require(
            mapping["schema_version"] == 1
                && mapping["type"] == "EPSG:7855"
                && mapping["unit"] == "metres"
                && mapping["units_verified"] == false
                && mapping["physicalVerification"] == false
                && mapping["method"]
                    .as_str()
                    .is_some_and(|s| !s.trim().is_empty())
                && mapping["spaces_sha256"] == src_hashes["spaces"]
                && mapping["floor_matrix_sha256"] == src_hashes["floors"]
                && is_hex64(&mapping["coordinates_sha256"])
                && is_hex64(&mapping["metadata_sha256"]),
            "invalid coordinate mapping or source hashes",
        )?;
    } else {
        require(mapping.is_null(), "coordinate mapping requires EPSG:7855")?;
    }
    for key in ["coordinate_semantics", "topology_semantics"] {
        require(
            m[key].as_str().is_some_and(|s| !s.trim().is_empty()),
            "semantics required",
        )?;
    }
    let expected = [
        "doors.parquet",
        "floors.parquet",
        "manifest.json",
        "spaces.parquet",
    ];
    let mut names = std::fs::read_dir(directory)?
        .map(|p| p.map(|p| p.file_name().to_string_lossy().into_owned()))
        .collect::<std::io::Result<Vec<_>>>()?;
    names.sort();
    require(names == expected, "unexpected package files")?;
    let files = m["files"].as_object().ok_or("file inventory required")?;
    require(files.len() == 3, "file inventory mismatch")?;
    let mut data = BTreeMap::new();
    for name in expected.into_iter().filter(|n| *n != "manifest.json") {
        let bytes = read(&directory.join(name), MAX_FILE)?;
        require(
            files.get(name).and_then(Value::as_str) == Some(hash(&bytes).as_str()),
            "artifact hash mismatch",
        )?;
        data.insert(name, bytes);
    }
    for key in ["spaces", "doors"] {
        let input_n = m["input_row_counts"][key]
            .as_u64()
            .ok_or("input row count required")?;
        let out_n = m["row_counts"][key].as_u64().ok_or("row count required")?;
        let excl_n = m["excluded_row_counts"][key]
            .as_u64()
            .ok_or("excluded row count required")?;
        require(
            input_n >= out_n && input_n - out_n == excl_n,
            "row count arithmetic mismatch",
        )?;
    }
    require(
        m["input_row_counts"]["floors"].as_u64() == m["row_counts"]["floors"].as_u64(),
        "floor row count mismatch",
    )?;
    let spaces_n = m["row_counts"]["spaces"]
        .as_u64()
        .ok_or("row count required")? as usize;
    let doors_n = m["row_counts"]["doors"]
        .as_u64()
        .ok_or("row count required")? as usize;
    let floors_n = m["row_counts"]["floors"]
        .as_u64()
        .ok_or("row count required")? as usize;
    let spaces = v2_table(
        data.remove("spaces.parquet").unwrap(),
        &[
            ("globalId", "text"),
            ("storey", "text"),
            ("building", "text"),
            ("centroidX", "number"),
            ("centroidY", "number"),
            ("connectedSpaceCount", "int"),
            ("connectedSpaceIds", "text"),
            ("floorId", "text"),
            ("sourceReadiness", "text"),
        ],
        "globalId",
        spaces_n,
    )?;
    let doors = v2_table(
        data.remove("doors.parquet").unwrap(),
        &[
            ("globalId", "text"),
            ("storey", "text"),
            ("connectedSpaceCount", "int"),
            ("connectedSpaceIds", "text"),
            ("isStairAccess", "bool"),
            ("isInterSpaceDoor", "bool"),
            ("floorId", "text"),
            ("sourceReadiness", "text"),
        ],
        "globalId",
        doors_n,
    )?;
    let floors = v2_table(
        data.remove("floors.parquet").unwrap(),
        &[
            ("floorId", "text"),
            ("scope", "text"),
            ("buildingCode", "text"),
            ("sourceReadiness", "text"),
            ("floorAcceptanceDisposition", "text"),
            ("topologySpaceCount", "int"),
            ("topologyDoorCount", "int"),
            ("finalAcceptanceReady", "bool"),
            ("physicalVerification", "bool"),
            ("exportStatus", "text"),
        ],
        "floorId",
        floors_n,
    )?;
    let shell_core: HashSet<&str> = ["A L5", "E L3", "E L4", "E L5", "E L6", "E L7"].into();
    let mh_missing: HashSet<&str> = ["MH L0", "MH L1", "MH L2", "MH L3"].into();
    let mut floors_by_scope = BTreeMap::new();
    let mut blocked_scopes = Vec::new();
    for f in &floors {
        let scope = f["scope"].as_str().unwrap();
        let readiness = f["sourceReadiness"].as_str().unwrap();
        let disp = f["floorAcceptanceDisposition"].as_str().unwrap();
        let export_status = f["exportStatus"].as_str().unwrap();
        let building = scope.split_whitespace().next().unwrap_or("");
        require(
            f["buildingCode"].as_str() == Some(building)
                && f["finalAcceptanceReady"] == false
                && f["physicalVerification"] == false
                && matches!(readiness, "available-plan" | "missing-plan")
                && (readiness != "available-plan"
                    || disp == "provisional-available-plan-partial-coverage")
                && (readiness != "missing-plan"
                    || matches!(
                        disp,
                        "provisional-missing-plan-shell-core-only"
                            | "provisional-missing-plan-full-cad-topology"
                    ))
                && (!shell_core.contains(scope)
                    || (readiness == "missing-plan"
                        && disp == "provisional-missing-plan-shell-core-only"
                        && export_status == "blocked-shell-core"))
                && (!mh_missing.contains(scope) || readiness == "missing-plan")
                && ((disp == "provisional-missing-plan-shell-core-only"
                    && export_status == "blocked-shell-core")
                    || (disp != "provisional-missing-plan-shell-core-only"
                        && export_status == "provisional")),
            "invalid or promoted campus floor status",
        )?;
        if export_status == "blocked-shell-core" {
            blocked_scopes.push(scope.to_owned());
        }
        require(
            floors_by_scope.insert(scope.to_owned(), f).is_none(),
            "duplicate campus floor scope",
        )?;
    }
    require(
        shell_core.iter().all(|s| floors_by_scope.contains_key(*s))
            && mh_missing.iter().all(|s| floors_by_scope.contains_key(*s)),
        "campus floor matrix is missing required floor scopes",
    )?;
    blocked_scopes.sort();
    let manifest_blocked: Vec<String> = m["blocked_floor_scopes"]
        .as_array()
        .ok_or("blocked_floor_scopes required")?
        .iter()
        .map(|v| v.as_str().map(str::to_owned).ok_or("invalid scope"))
        .collect::<std::result::Result<_, _>>()?;
    require(
        manifest_blocked == blocked_scopes,
        "blocked_floor_scopes mismatch",
    )?;
    for (table, count) in [
        ("spaces", "topologySpaceCount"),
        ("doors", "topologyDoorCount"),
    ] {
        let excluded = blocked_scopes.iter().try_fold(0u64, |total, scope| {
            total.checked_add(floors_by_scope[scope][count].as_u64().unwrap())
        });
        require(
            excluded == m["excluded_row_counts"][table].as_u64(),
            "blocked floor topology counts disagree",
        )?;
    }
    let mut space_by_id = BTreeMap::new();
    let mut space_neighbors: BTreeMap<String, HashSet<String>> = BTreeMap::new();
    let mut space_counts: BTreeMap<String, u64> = BTreeMap::new();
    let mut nodes = Vec::with_capacity(spaces.len());
    for s in &spaces {
        let id = s["globalId"].as_str().unwrap();
        let scope = s["storey"].as_str().unwrap();
        let floor = floors_by_scope.get(scope).ok_or("space floor mismatch")?;
        require(
            floor["exportStatus"] != "blocked-shell-core"
                && s["building"] == floor["buildingCode"]
                && s["floorId"] == floor["floorId"]
                && s["sourceReadiness"] == floor["sourceReadiness"],
            "space floor or building mismatch",
        )?;
        *space_counts.entry(scope.to_owned()).or_default() += 1;
        let ids: HashSet<String> = parse_connected_ids(s)?.into_iter().collect();
        space_neighbors.insert(id.to_owned(), ids);
        space_by_id.insert(id.to_owned(), s);
        nodes.push(json!({
            "id": id,
            "floor_id": s["floorId"],
            "x": s["centroidX"],
            "y": s["centroidY"],
        }));
    }
    let mut door_neighbors: BTreeMap<String, HashSet<String>> = space_by_id
        .keys()
        .map(|k| (k.clone(), HashSet::new()))
        .collect();
    let mut door_counts: BTreeMap<String, u64> = BTreeMap::new();
    let mut edges = Vec::new();
    for d in &doors {
        let scope = d["storey"].as_str().unwrap();
        let floor = floors_by_scope.get(scope).ok_or("door floor mismatch")?;
        require(
            floor["exportStatus"] != "blocked-shell-core"
                && d["floorId"] == floor["floorId"]
                && d["sourceReadiness"] == floor["sourceReadiness"],
            "door floor mismatch",
        )?;
        *door_counts.entry(scope.to_owned()).or_default() += 1;
        let ids = parse_connected_ids(d)?;
        let is_inter = d["isInterSpaceDoor"].as_bool().unwrap();
        require(
            ids.len() <= 2 && is_inter == (ids.len() == 2),
            "invalid door topology or stair flag",
        )?;
        for sid in &ids {
            let sp = space_by_id.get(sid).ok_or("unresolved door space")?;
            require(sp["storey"] == scope, "cross-floor door space")?;
        }
        if ids.len() == 2 {
            door_neighbors
                .get_mut(&ids[0])
                .unwrap()
                .insert(ids[1].clone());
            door_neighbors
                .get_mut(&ids[1])
                .unwrap()
                .insert(ids[0].clone());
            let a = space_by_id[&ids[0]];
            let b = space_by_id[&ids[1]];
            let distance = (a["centroidX"].as_f64().unwrap() - b["centroidX"].as_f64().unwrap())
                .hypot(a["centroidY"].as_f64().unwrap() - b["centroidY"].as_f64().unwrap());
            require(distance.is_finite(), "non-finite campus edge length")?;
            edges.push(json!({
                "id": d["globalId"],
                "from_id": ids[0],
                "to_id": ids[1],
                "length": distance,
            }));
        }
    }
    require(
        space_neighbors == door_neighbors,
        "space and door connectivity disagree",
    )?;
    for (scope, floor) in &floors_by_scope {
        if floor["exportStatus"] != "blocked-shell-core" {
            let exp_s = floor["topologySpaceCount"].as_u64().unwrap();
            let exp_d = floor["topologyDoorCount"].as_u64().unwrap();
            require(
                space_counts.get(scope).copied().unwrap_or(0) == exp_s
                    && door_counts.get(scope).copied().unwrap_or(0) == exp_d,
                "floor matrix topology counts disagree",
            )?;
        }
    }
    Ok(Package {
        manifest_sha256: expected_hash.into(),
        manifest: m,
        model: Value::Null,
        objects: Vec::new(),
        nodes,
        edges,
        spaces,
        doors,
        floors,
    })
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Assumptions {
    pub accept_provisional: bool,
    pub assumed_metres_per_source_unit: f64,
    pub speed_m_per_s: f64,
    pub allowed_edge_ids: Vec<String>,
}
impl Package {
    pub fn travel_seconds(&self, path: &[String], assumptions: &Assumptions) -> Result<f64> {
        let route_ready = match self.manifest["schema_version"].as_u64() {
            Some(1) => self.manifest["graph_status"] == "available",
            Some(2) => !self.edges.is_empty(),
            _ => false,
        };
        require(route_ready && path.len() >= 2, "route unavailable")?;
        require(
            assumptions.accept_provisional
                && assumptions.assumed_metres_per_source_unit.is_finite()
                && assumptions.assumed_metres_per_source_unit > 0.0
                && assumptions.speed_m_per_s.is_finite()
                && assumptions.speed_m_per_s > 0.0,
            "explicit provisional scale/speed required",
        )?;
        require(
            self.manifest["frame"]["unit"] != "metres"
                || assumptions.assumed_metres_per_source_unit == 1.0,
            "metre-frame scale must be one",
        )?;
        let mut movement = Vec::new();
        for pair in path.windows(2) {
            let matches: Vec<_> = self
                .edges
                .iter()
                .filter(|e| {
                    (e["from_id"] == pair[0] && e["to_id"] == pair[1])
                        || (e["from_id"] == pair[1] && e["to_id"] == pair[0])
                })
                .collect();
            require(matches.len() == 1, "disconnected or ambiguous route")?;
            let edge = matches[0];
            let id = edge["id"].as_str().unwrap();
            require(
                assumptions.allowed_edge_ids.iter().any(|x| x == id),
                "edge policy denied",
            )?;
            let distance =
                edge["length"].as_f64().unwrap() * assumptions.assumed_metres_per_source_unit;
            require(distance.is_finite(), "scaled distance overflow")?;
            movement.push(MovementEdge {
                edge_id: id.into(),
                from: pair[0].clone(),
                to: pair[1].clone(),
                distance_m: distance,
                speed_m_per_s: assumptions.speed_m_per_s,
                accessible: true,
            });
        }
        let ids: Vec<_> = movement.iter().map(|e| e.edge_id.as_str()).collect();
        let value = calculate_execution_duration(
            ExecutionFidelity::Micro,
            0.0,
            Some(&SyntheticGraph::new(movement.clone())),
            &ids,
        )
        .map_err(|e| format!("CareOps execution error: {e:?}"))?;
        require(value.is_finite(), "duration overflow")?;
        Ok(value)
    }
}
#[derive(Default)]
pub struct Session {
    pub current: Option<Package>,
}
impl Session {
    pub fn replace(&mut self, path: &Path, expected: &str) -> Result<()> {
        let validated = load(path, expected)?;
        self.current = Some(validated);
        Ok(())
    }
    pub fn checkpoint(&self) -> Result<Vec<u8>> {
        let current = self.current.as_ref().ok_or("no active revision")?;
        Ok(serde_json::to_vec(
            &json!({"schema_version":1,"manifest_sha256":current.manifest_sha256}),
        )?)
    }
    pub fn restore(&mut self, path: &Path, checkpoint: &[u8]) -> Result<()> {
        let c: Value = serde_json::from_slice(checkpoint)?;
        require(c["schema_version"] == 1, "unsupported checkpoint")?;
        self.replace(
            path,
            c["manifest_sha256"]
                .as_str()
                .ok_or("checkpoint identity required")?,
        )
    }
}
