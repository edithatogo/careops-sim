//! Closed native ED file-profile validation; standard libraries decode the data.
use super::*;
/// An immutable snapshot of typed native tables and their actual manifest hash.
/// Historical model identity permits analysis, not execution recovery.
pub struct NativeDataset {
    pub(super) manifest: NativeManifest,
    pub(super) manifest_sha256: String,
    pub(super) frames: [DataFrame; 3],
}
impl NativeDataset {
    pub fn manifest(&self) -> &NativeManifest {
        &self.manifest
    }
    pub fn manifest_sha256(&self) -> &str {
        &self.manifest_sha256
    }
    pub fn patients(&self) -> &DataFrame {
        &self.frames[0]
    }
    pub fn replications(&self) -> &DataFrame {
        &self.frames[1]
    }
    pub fn metrics(&self) -> &DataFrame {
        &self.frames[2]
    }
}
pub(super) fn closed_columns(table: &str) -> Result<Vec<ColumnSchema>, ValidationError> {
    let names = match table {
        "patients"=>"time_unit schema capture_schema run_id input_sha256 model_contract_sha256 replication_index seed sequence patient_id status arrival_tick_hi arrival_tick_lo start_tick_hi start_tick_lo completion_tick_hi completion_tick_lo work_ticks_hi work_ticks_lo wait_ticks_hi wait_ticks_lo elapsed_ticks_hi elapsed_ticks_lo scenario_id resource_id location_id capture_table_schema",
        "replications"=>"run_id replication_index seed horizon_ticks arrivals started completed unfinished mean_wait_ticks",
        "metrics"=>"name value unit denominator_name denominator_value numerator_count denominator_count estimand sample_unit weighting eligible_replications included_replications total_replications estimate variance standard_error ci_95_lower ci_95_upper uncertainty_replications ddof method status censored_count unfinished_count",
        _=>return Err(invalid("unknown native table")),
    };
    Ok(names
        .split_whitespace()
        .map(|name| {
            let (dtype, nullable) = match table {
                "patients" => {
                    let string = [
                        "time_unit",
                        "schema",
                        "capture_schema",
                        "run_id",
                        "input_sha256",
                        "model_contract_sha256",
                        "patient_id",
                        "status",
                        "scenario_id",
                        "resource_id",
                        "location_id",
                        "capture_table_schema",
                    ]
                    .contains(&name);
                    (
                        if string { "String" } else { "UInt64" },
                        [
                            "start_tick_hi",
                            "start_tick_lo",
                            "completion_tick_hi",
                            "completion_tick_lo",
                            "wait_ticks_hi",
                            "wait_ticks_lo",
                            "elapsed_ticks_hi",
                            "elapsed_ticks_lo",
                        ]
                        .contains(&name),
                    )
                }
                "replications" => (
                    if name == "run_id" {
                        "String"
                    } else if name == "mean_wait_ticks" {
                        "Float64"
                    } else {
                        "UInt64"
                    },
                    name == "mean_wait_ticks",
                ),
                _ => {
                    let nullable = [
                        "value",
                        "denominator_value",
                        "numerator_count",
                        "denominator_count",
                        "estimate",
                        "variance",
                        "standard_error",
                        "ci_95_lower",
                        "ci_95_upper",
                        "ddof",
                    ]
                    .contains(&name);
                    (
                        if [
                            "name",
                            "unit",
                            "denominator_name",
                            "estimand",
                            "sample_unit",
                            "weighting",
                            "method",
                            "status",
                        ]
                        .contains(&name)
                        {
                            "String"
                        } else if name == "ddof" {
                            "UInt32"
                        } else if [
                            "numerator_count",
                            "denominator_count",
                            "eligible_replications",
                            "included_replications",
                            "total_replications",
                            "uncertainty_replications",
                            "censored_count",
                            "unfinished_count",
                        ]
                        .contains(&name)
                        {
                            "UInt64"
                        } else {
                            "Float64"
                        },
                        nullable,
                    )
                }
            };
            ColumnSchema {
                name: name.into(),
                physical_type: dtype.into(),
                nullable,
            }
        })
        .collect())
}
pub(super) fn bounded_file(path: &Path, maximum: usize) -> Result<Vec<u8>, ValidationError> {
    use std::io::Read;
    let meta = std::fs::symlink_metadata(path).map_err(invalid)?;
    if !meta.is_file() || meta.len() > maximum as u64 {
        return Err(invalid("native file type or byte bound exceeded"));
    }
    let mut file = std::fs::File::open(path).map_err(invalid)?;
    let mut bytes = vec![0; meta.len() as usize];
    file.read_exact(&mut bytes).map_err(invalid)?;
    if file.read(&mut [0]).map_err(invalid)? != 0 {
        return Err(invalid("native file grew beyond byte bound"));
    }
    Ok(bytes)
}
pub(super) fn check_frame(
    frame: &DataFrame,
    declared: &[ColumnSchema],
    rows: usize,
) -> Result<(), ValidationError> {
    if frame.height() != rows || frame.width() != declared.len() {
        return Err(invalid("native decoded shape mismatch"));
    }
    for (s, c) in frame.columns().iter().zip(declared) {
        if s.name().as_str() != c.name
            || format!("{:?}", s.dtype()) != c.physical_type
            || (!c.nullable && s.null_count() != 0)
        {
            return Err(invalid("native decoded schema or requiredness mismatch"));
        }
        if s.dtype() == &DataType::Float64
            && (0..s.len()).any(|i| {
                s.f64()
                    .expect("checked Float64")
                    .get(i)
                    .is_some_and(|v| !v.is_finite())
            })
        {
            return Err(invalid("nonfinite native observation"));
        }
    }
    Ok(())
}
pub fn read_native_dataset(root: &Path) -> Result<NativeDataset, ValidationError> {
    use std::io::Cursor;
    let root_meta = std::fs::symlink_metadata(root).map_err(invalid)?;
    if !root_meta.is_dir() || root_meta.file_type().is_symlink() {
        return Err(invalid("native dataset root must be a real directory"));
    }
    let bytes = bounded_file(&root.join("manifest.json"), 1024 * 1024)?;
    let manifest: NativeManifest = serde_json::from_slice(&bytes).map_err(invalid)?;
    manifest.limits.validate()?;
    let hash_ok = |s: &str| {
        s.len() == 64
            && s.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    };
    if manifest.schema != SCHEMA
        || manifest.status != "complete"
        || manifest.tables.len() != 3
        || bytes.len() > manifest.limits.max_manifest_bytes
        || manifest.compression != (manifest.format == NativeFormat::Parquet).then(|| "zstd".into())
        || manifest.runtime_revision != RUNTIME_REVISION
        || ![DATA_REVISION, "f34750533c07e9e868481606e5fe5c89ec9b76be"]
            .contains(&manifest.data_revision.as_str())
        || manifest.polars_revision != POLARS_REVISION
        || manifest.producer != format!("careops-ed/{}", env!("CARGO_PKG_VERSION"))
        || manifest.time_unit.is_empty()
        || manifest.time_unit.len() > 256
        || manifest.batch_id.is_empty()
        || manifest.scenario_id.is_empty()
        || !hash_ok(&manifest.effective_input_sha256)
        || !hash_ok(&manifest.parsed_input_sha256)
        || !hash_ok(&manifest.model_contract_sha256)
        || manifest
            .raw_input_sha256
            .as_ref()
            .is_some_and(|s| !hash_ok(s))
    {
        return Err(invalid("unsupported native manifest identity/profile"));
    }
    let mut snapshots = Vec::new();
    let mut encoded = 0usize;
    let mut total_rows = 0usize;
    let mut parts = 0usize;
    for (index, (name, schema)) in [
        ("patients", "careops.ed.patients.v2"),
        ("replications", "careops.ed.replications.v1"),
        ("metrics", "careops.ed.metrics.v1"),
    ]
    .into_iter()
    .enumerate()
    {
        let table = &manifest.tables[index];
        if table.name != name
            || table.schema != schema
            || table.columns != closed_columns(name)?
            || table.parts.is_empty()
            || (name == "metrics" && table.rows != 5)
            || (name == "replications" && table.rows != manifest.replication_seeds.len())
        {
            return Err(invalid("native closed table contract mismatch"));
        }
        total_rows = total_rows
            .checked_add(table.rows)
            .ok_or_else(|| invalid("native total rows overflow"))?;
        if total_rows > manifest.limits.max_total_rows {
            return Err(invalid("native total row bound exceeded"));
        }
        let mut offset = 0usize;
        for (part_index, part) in table.parts.iter().enumerate() {
            parts += 1;
            encoded = encoded
                .checked_add(part.bytes)
                .ok_or_else(|| invalid("native encoded bytes overflow"))?;
            if parts > manifest.limits.max_parts
                || encoded > 64 * 1024 * 1024
                || part.file != format!("{name}-{part_index:05}.{}", manifest.format.extension())
                || part.first_row != offset
                || part.rows > manifest.limits.max_rows_per_part
                || (part.rows == 0 && (table.rows != 0 || table.parts.len() != 1))
                || part.bytes > manifest.limits.max_bytes_per_part
                || !hash_ok(&part.sha256)
            {
                return Err(invalid("native part ordering or bound mismatch"));
            }
            offset = offset
                .checked_add(part.rows)
                .ok_or_else(|| invalid("native row offset overflow"))?;
            let content = bounded_file(&root.join(&part.file), part.bytes)?;
            if content.len() != part.bytes
                || crate::disposition::api_runner::compute_sha256_hex(&content) != part.sha256
            {
                return Err(invalid("native part length or digest mismatch"));
            }
            snapshots.push((index, part_index, content));
        }
        if offset != table.rows {
            return Err(invalid("native part row coverage mismatch"));
        }
    }
    // Every encoded part is now owned and hash-checked. Validate all allocation
    // profiles before constructing any DataFrame from these immutable snapshots.
    let mut budget = ProfileBudget::default();
    for (index, part_index, content) in &snapshots {
        let table = &manifest.tables[*index];
        let part = &table.parts[*part_index];
        budget.add(content, manifest.format, &table.columns, part.rows)?;
    }
    let mut frames: [Option<DataFrame>; 3] = [None, None, None];
    for (index, part_index, content) in snapshots {
        let table = &manifest.tables[index];
        let part = &table.parts[part_index];
        let expected = file_metadata(&manifest, table, part.first_row, part.rows);
        let frame = match manifest.format {
            NativeFormat::Parquet => {
                let mut reader = ParquetReader::new(Cursor::new(content));
                let metadata = reader.get_metadata().map_err(invalid)?;
                let mut actual = BTreeMap::new();
                for kv in metadata.key_value_metadata.iter().flatten() {
                    if kv.key == "ARROW:schema" {
                        continue;
                    }
                    if actual
                        .insert(
                            kv.key.clone(),
                            kv.value
                                .clone()
                                .ok_or_else(|| invalid("missing Parquet metadata value"))?,
                        )
                        .is_some()
                    {
                        return Err(invalid("duplicate Parquet metadata key"));
                    }
                }
                if actual != expected {
                    return Err(invalid("Parquet provenance metadata mismatch"));
                }
                reader.finish().map_err(invalid)?
            }
            NativeFormat::Ipc => {
                let mut reader = IpcStreamReader::new(Cursor::new(content));
                let metadata = reader
                    .custom_metadata()
                    .map_err(invalid)?
                    .ok_or_else(|| invalid("missing IPC provenance metadata"))?;
                let actual: BTreeMap<String, String> = metadata
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect();
                if actual != expected {
                    return Err(invalid("IPC provenance metadata mismatch"));
                }
                reader.finish().map_err(invalid)?
            }
            NativeFormat::Csv => csv_part(&content, &table.columns, part.rows)?,
        };
        check_frame(&frame, &table.columns, part.rows)?;
        if let Some(existing) = frames[index].as_mut() {
            existing.vstack_mut(&frame).map_err(invalid)?;
        } else {
            frames[index] = Some(frame);
        }
    }
    let frames = frames.map(|f| f.expect("validated nonempty part lists"));
    for (frame, table) in frames.iter().zip(&manifest.tables) {
        check_frame(frame, &table.columns, table.rows)?;
    }
    check_provenance(&frames, &manifest)?;
    Ok(NativeDataset {
        manifest,
        manifest_sha256: crate::disposition::api_runner::compute_sha256_hex(&bytes),
        frames,
    })
}

/// Read only the finite necessary-quote CSV profile emitted by this module.
/// Encoded size and quote-aware rows are bounded before typed parser allocation;
/// canonical standard-writer readback rejects short/ragged or ambiguous records.
fn csv_schema(
    bytes: &[u8],
    declared: &[ColumnSchema],
    rows: usize,
) -> Result<Schema, ValidationError> {
    use std::collections::BTreeSet;
    if bytes.len() > 4 * 1024 * 1024 || rows > 256 || declared.is_empty() || declared.len() > 64 {
        return Err(invalid("CSV profile bound exceeded"));
    }
    let mut schema = Schema::with_capacity(declared.len());
    let mut names = BTreeSet::new();
    for column in declared {
        if column.name.is_empty()
            || column.name.len() > 64
            || !column
                .name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
            || !names.insert(column.name.as_str())
        {
            return Err(invalid("unsupported CSV header"));
        }
        let dtype = match column.physical_type.as_str() {
            "String" => DataType::String,
            "UInt64" => DataType::UInt64,
            "UInt32" => DataType::UInt32,
            "Float64" => DataType::Float64,
            "Boolean" => DataType::Boolean,
            _ => return Err(invalid("unsupported CSV physical type")),
        };
        schema.insert(column.name.clone().into(), dtype);
    }
    let header = declared
        .iter()
        .map(|c| c.name.as_str())
        .collect::<Vec<_>>()
        .join(",")
        + "\n";
    if !bytes.starts_with(header.as_bytes()) {
        return Err(invalid("CSV exact header mismatch"));
    }
    let counted = polars::io::csv::read::count_rows_from_slice_par(
        bytes.to_vec().into(),
        Some(b'"'),
        None,
        b'\n',
        true,
        0,
        0,
        0,
        true,
    )
    .map_err(invalid)?;
    if counted != rows {
        return Err(invalid("CSV quote-aware row count mismatch"));
    }
    Ok(schema)
}

pub(super) fn csv_part(
    bytes: &[u8],
    declared: &[ColumnSchema],
    rows: usize,
) -> Result<DataFrame, ValidationError> {
    use std::{io::Cursor, sync::Arc};
    let schema = csv_schema(bytes, declared, rows)?;
    let parse = CsvParseOptions::default()
        .with_encoding(CsvEncoding::Utf8)
        .with_missing_is_null(true)
        .with_null_values(None)
        .with_truncate_ragged_lines(false);
    let mut frame = CsvReadOptions::default()
        .with_schema(Some(Arc::new(schema)))
        .with_infer_schema_length(Some(0))
        .with_n_threads(Some(1))
        .with_ignore_errors(false)
        .with_parse_options(parse)
        .into_reader_with_file_handle(Cursor::new(bytes))
        .finish()
        .map_err(invalid)?;
    if frame.height() != rows
        || declared
            .iter()
            .zip(frame.columns())
            .any(|(c, s)| !c.nullable && s.null_count() != 0)
    {
        return Err(invalid("CSV typed rows or requiredness mismatch"));
    }
    let mut canonical = BoundedPartWriter::new(Vec::new(), bytes.len());
    CsvWriter::new(&mut canonical)
        .include_header(true)
        .with_quote_style(QuoteStyle::Necessary)
        .with_null_value(String::new().into())
        .finish(&mut frame)
        .map_err(invalid)?;
    if canonical.into_inner() != bytes {
        return Err(invalid("CSV is not the canonical native writer profile"));
    }
    Ok(frame)
}

#[cfg(test)]
mod csv_profile_tests {
    use super::*;
    fn schema() -> Vec<ColumnSchema> {
        vec![
            ColumnSchema {
                name: "label".into(),
                physical_type: "String".into(),
                nullable: true,
            },
            ColumnSchema {
                name: "number".into(),
                physical_type: "UInt64".into(),
                nullable: false,
            },
        ]
    }
    #[test]
    fn csv_preserves_quoted_empty_null_unicode_and_embedded_newline() {
        let raw = "label,number\n,1\n\"\",2\n\"é,\r\nquoted\"\"value\",3\n".as_bytes();
        let frame = csv_part(raw, &schema(), 3).unwrap();
        let labels = frame.column("label").unwrap().str().unwrap();
        assert_eq!(labels.get(0), None);
        assert_eq!(labels.get(1), Some(""));
        assert_eq!(labels.get(2), Some("é,\r\nquoted\"value"));
        assert_eq!(
            csv_part(b"label,number\n", &schema(), 0).unwrap().height(),
            0
        );
    }
    #[test]
    fn csv_rejects_actual_malformed_or_noncanonical_records_before_acceptance() {
        for raw in [
            b"label,number\nx\n".as_slice(),
            b"label,number\nx,1,2\n",
            b"label,number\nx,-1\n",
            b"label,number\nx,\n",
            b"label,number\n\"x,1\n",
            b"label,number\n\"x\",1\n",
            b"label,number\nx,1\r\n",
        ] {
            assert!(csv_part(raw, &schema(), 1).is_err(), "accepted {raw:?}");
        }
        assert!(csv_part(b"label,number\nx,1\n", &schema(), 2).is_err());
        assert!(csv_part(b"label,number\nx,1\n", &schema(), 257).is_err());
        let mut changed = schema();
        changed[0].nullable = false;
        assert!(csv_part(b"label,number\n,1\n", &changed, 1).is_err());
    }
}

/// Inspect the producer's uncompressed flat Arrow stream before Polars allocates
/// arrays. Standard generated metadata accessors own protocol interpretation.
pub(super) fn ipc_preflight(
    bytes: &[u8],
    declared: &[ColumnSchema],
    rows: usize,
) -> Result<(), ValidationError> {
    use polars_arrow_format::ipc::{
        planus::ReadAsRoot, Endianness, MessageHeaderRef, MessageRef, MetadataVersion, Precision,
        TypeRef,
    };
    if bytes.len() > 4 * 1024 * 1024 || rows > 256 || declared.is_empty() || declared.len() > 64 {
        return Err(invalid("IPC profile bound exceeded"));
    }
    let mut at = 0usize;
    let mut saw_schema = false;
    let mut total = 0usize;
    let mut batches = 0usize;
    loop {
        let prefix = bytes
            .get(at..at + 8)
            .ok_or_else(|| invalid("truncated IPC frame"))?;
        if prefix[..4] != [255; 4] {
            return Err(invalid("unsupported IPC frame prefix"));
        }
        let size = u32::from_le_bytes(prefix[4..8].try_into().unwrap()) as usize;
        at += 8;
        if size == 0 {
            if !saw_schema || total != rows || at != bytes.len() {
                return Err(invalid("IPC final rows or trailing bytes mismatch"));
            }
            return Ok(());
        }
        if size > 65536 {
            return Err(invalid("IPC metadata bound exceeded"));
        }
        let end = at
            .checked_add(size)
            .ok_or_else(|| invalid("IPC metadata overflow"))?;
        let message = MessageRef::read_as_root(
            bytes
                .get(at..end)
                .ok_or_else(|| invalid("truncated IPC metadata"))?,
        )
        .map_err(invalid)?;
        if message.version().map_err(invalid)? != MetadataVersion::V5 {
            return Err(invalid("unsupported IPC metadata version"));
        }
        let length = usize::try_from(message.body_length().map_err(invalid)?).map_err(invalid)?;
        let body_end = end
            .checked_add(length)
            .ok_or_else(|| invalid("IPC body overflow"))?;
        let body = bytes
            .get(end..body_end)
            .ok_or_else(|| invalid("truncated IPC body"))?;
        match message
            .header()
            .map_err(invalid)?
            .ok_or_else(|| invalid("missing IPC header"))?
        {
            MessageHeaderRef::Schema(schema) => {
                if saw_schema
                    || total != 0
                    || !body.is_empty()
                    || schema.endianness().map_err(invalid)? != Endianness::Little
                {
                    return Err(invalid("unsupported IPC schema envelope"));
                }
                let mut metadata_keys = std::collections::BTreeSet::new();
                if let Some(entries) = schema.custom_metadata().map_err(invalid)? {
                    for entry in entries.iter() {
                        let entry = entry.map_err(invalid)?;
                        let key = entry
                            .key()
                            .map_err(invalid)?
                            .ok_or_else(|| invalid("missing IPC metadata key"))?;
                        if !metadata_keys.insert(key) {
                            return Err(invalid("duplicate IPC schema metadata key"));
                        }
                    }
                }
                let fields = schema
                    .fields()
                    .map_err(invalid)?
                    .ok_or_else(|| invalid("missing IPC fields"))?;
                if fields.len() != declared.len() {
                    return Err(invalid("IPC field count mismatch"));
                }
                for (field, column) in fields.iter().zip(declared) {
                    let field = field.map_err(invalid)?;
                    if field
                        .custom_metadata()
                        .map_err(invalid)?
                        .is_some_and(|v| !v.is_empty())
                    {
                        return Err(invalid("unsupported IPC field metadata"));
                    }
                    if field.name().map_err(invalid)? != Some(column.name.as_str())
                        || field.dictionary().map_err(invalid)?.is_some()
                        || field
                            .children()
                            .map_err(invalid)?
                            .is_some_and(|c| !c.is_empty())
                    {
                        return Err(invalid("unsupported IPC field structure"));
                    }
                    let kind = field
                        .type_()
                        .map_err(invalid)?
                        .ok_or_else(|| invalid("missing IPC physical type"))?;
                    let supported = match (kind, column.physical_type.as_str()) {
                        (TypeRef::LargeUtf8(_), "String") | (TypeRef::Bool(_), "Boolean") => true,
                        (TypeRef::Int(int), "UInt64") => {
                            int.bit_width().map_err(invalid)? == 64
                                && !int.is_signed().map_err(invalid)?
                        }
                        (TypeRef::Int(int), "UInt32") => {
                            int.bit_width().map_err(invalid)? == 32
                                && !int.is_signed().map_err(invalid)?
                        }
                        (TypeRef::FloatingPoint(float), "Float64") => {
                            float.precision().map_err(invalid)? == Precision::Double
                        }
                        _ => false,
                    };
                    if !supported {
                        return Err(invalid("IPC physical type mismatch"));
                    }
                }
                saw_schema = true;
            }
            MessageHeaderRef::RecordBatch(batch) => {
                let n = usize::try_from(batch.length().map_err(invalid)?).map_err(invalid)?;
                batches += 1;
                if batches > rows.max(1) || (n == 0 && rows != 0) {
                    return Err(invalid("IPC batch count exceeds finite row profile"));
                }
                total = total
                    .checked_add(n)
                    .ok_or_else(|| invalid("IPC row overflow"))?;
                if !saw_schema
                    || total > rows
                    || batch.compression().map_err(invalid)?.is_some()
                    || batch
                        .variadic_buffer_counts()
                        .map_err(invalid)?
                        .is_some_and(|v| !v.is_empty())
                {
                    return Err(invalid("unsupported IPC record batch"));
                }
                let nodes = batch
                    .nodes()
                    .map_err(invalid)?
                    .ok_or_else(|| invalid("missing IPC nodes"))?;
                let buffers = batch
                    .buffers()
                    .map_err(invalid)?
                    .ok_or_else(|| invalid("missing IPC buffers"))?;
                let expected_buffers: usize = declared
                    .iter()
                    .map(|c| if c.physical_type == "String" { 3 } else { 2 })
                    .sum();
                if nodes.len() != declared.len() || buffers.len() != expected_buffers {
                    return Err(invalid("IPC flat node/buffer count mismatch"));
                }
                let mut slices = Vec::with_capacity(buffers.len());
                let mut previous_end = 0usize;
                for buffer in buffers.iter() {
                    let offset = usize::try_from(buffer.offset()).map_err(invalid)?;
                    let count = usize::try_from(buffer.length()).map_err(invalid)?;
                    let stop = offset
                        .checked_add(count)
                        .ok_or_else(|| invalid("IPC buffer overflow"))?;
                    if offset < previous_end || offset % 8 != 0 {
                        return Err(invalid("IPC overlapping or misaligned buffers"));
                    }
                    slices.push(
                        body.get(offset..stop)
                            .ok_or_else(|| invalid("IPC buffer outside body"))?,
                    );
                    previous_end = stop;
                }
                let mut index = 0;
                for (node, column) in nodes.iter().zip(declared) {
                    let nulls = usize::try_from(node.null_count()).map_err(invalid)?;
                    if node.length() != n as i64 || nulls > n || (!column.nullable && nulls != 0) {
                        return Err(invalid("IPC node length or nullability mismatch"));
                    }
                    let validity = slices[index];
                    if validity.is_empty() {
                        if nulls != 0 {
                            return Err(invalid("IPC absent null bitmap"));
                        }
                    } else {
                        if validity.len() != n.div_ceil(8) {
                            return Err(invalid("IPC bitmap length mismatch"));
                        }
                        let actual = (0..n)
                            .filter(|i| validity[i / 8] & (1 << (i % 8)) == 0)
                            .count();
                        if actual != nulls {
                            return Err(invalid("IPC bitmap null count mismatch"));
                        }
                    }
                    if column.physical_type == "String" {
                        let offsets = slices[index + 1];
                        let values = slices[index + 2];
                        if offsets.len() != (n + 1) * 8 {
                            return Err(invalid("IPC string offset length mismatch"));
                        }
                        let text = std::str::from_utf8(values).map_err(invalid)?;
                        let mut previous = 0usize;
                        for offset in offsets.as_chunks::<8>().0 {
                            let offset =
                                usize::try_from(i64::from_le_bytes(*offset)).map_err(invalid)?;
                            if offset < previous
                                || offset > values.len()
                                || !text.is_char_boundary(offset)
                            {
                                return Err(invalid("IPC invalid string offsets"));
                            }
                            previous = offset;
                        }
                        index += 3;
                    } else {
                        let width = match column.physical_type.as_str() {
                            "Boolean" => n.div_ceil(8),
                            "UInt32" => n * 4,
                            _ => n * 8,
                        };
                        if slices[index + 1].len() != width {
                            return Err(invalid("IPC primitive buffer length mismatch"));
                        }
                        index += 2;
                    }
                }
            }
            _ => return Err(invalid("unsupported IPC message type")),
        }
        at = body_end;
    }
}

#[cfg(test)]
mod ipc_profile_tests {
    use super::*;
    fn source() -> (Vec<u8>, Vec<ColumnSchema>) {
        let mut frame = df!("label" => [Some("é"), Some("")], "number" => [1u64, 2]).unwrap();
        let columns = vec![
            ColumnSchema {
                name: "label".into(),
                physical_type: "String".into(),
                nullable: true,
            },
            ColumnSchema {
                name: "number".into(),
                physical_type: "UInt64".into(),
                nullable: false,
            },
        ];
        let mut bytes = Vec::new();
        encode(&mut frame, NativeFormat::Ipc, &BTreeMap::new(), &mut bytes).unwrap();
        (bytes, columns)
    }
    #[test]
    fn actual_ipc_schema_rejects_extension_fields_and_duplicate_metadata_keys() {
        use polars_arrow_format::ipc::{
            planus::{Builder, ReadAsRoot},
            KeyValue, Message, MessageHeader, MessageRef,
        };
        let (bytes, columns) = source();
        let old_size = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
        let original: Message = MessageRef::read_as_root(&bytes[8..8 + old_size])
            .unwrap()
            .try_into()
            .unwrap();
        for extension in [true, false] {
            let mut message = original.clone();
            let Some(MessageHeader::Schema(schema)) = message.header.as_mut() else {
                panic!("expected schema")
            };
            let entry = KeyValue {
                key: Some(
                    if extension {
                        "ARROW:extension:name"
                    } else {
                        "careops.table"
                    }
                    .into(),
                ),
                value: Some("unqualified".into()),
            };
            if extension {
                schema.fields.as_mut().unwrap()[0].custom_metadata = Some(vec![entry]);
            } else {
                schema.custom_metadata = Some(vec![entry.clone(), entry]);
            }
            let mut builder = Builder::new();
            let metadata = builder.finish(&message, None);
            let padded = metadata.len().div_ceil(8) * 8;
            let mut changed = vec![255; 4];
            changed.extend_from_slice(&(padded as u32).to_le_bytes());
            changed.extend_from_slice(metadata);
            changed.resize(8 + padded, 0);
            changed.extend_from_slice(&bytes[8 + old_size..]);
            let error = ipc_preflight(&changed, &columns, 2).unwrap_err();
            assert!(
                error.message.contains(if extension {
                    "field metadata"
                } else {
                    "duplicate IPC schema metadata"
                }),
                "{error:?}"
            );
        }
    }
    #[test]
    fn actual_ipc_frames_validate_before_decode_and_reject_bad_utf8_offsets_and_framing() {
        use polars_arrow_format::ipc::{planus::ReadAsRoot, MessageHeaderRef, MessageRef};
        let (bytes, columns) = source();
        ipc_preflight(&bytes, &columns, 2).unwrap();
        let mut changed = bytes.clone();
        let text = changed
            .windows(2)
            .position(|b| b == "é".as_bytes())
            .unwrap();
        changed[text] = 255;
        assert!(ipc_preflight(&changed, &columns, 2).is_err());
        let mut at = 0;
        loop {
            let length = u32::from_le_bytes(bytes[at + 4..at + 8].try_into().unwrap()) as usize;
            let body = at + 8 + length;
            let message = MessageRef::read_as_root(&bytes[at + 8..body]).unwrap();
            if let Some(MessageHeaderRef::RecordBatch(batch)) = message.header().unwrap() {
                let offset = batch.buffers().unwrap().unwrap().get(1).unwrap().offset() as usize;
                for value in [-1i64, 1, i64::MAX] {
                    let mut changed = bytes.clone();
                    changed[body + offset + 8..body + offset + 16]
                        .copy_from_slice(&value.to_le_bytes());
                    assert!(ipc_preflight(&changed, &columns, 2).is_err());
                }
                break;
            }
            at = body + message.body_length().unwrap() as usize;
        }
        for stop in [0, 4, bytes.len() - 1] {
            assert!(ipc_preflight(&bytes[..stop], &columns, 2).is_err());
        }
        assert!(ipc_preflight(&bytes, &columns, 1).is_err());
        let mut changed = bytes.clone();
        changed.push(0);
        assert!(ipc_preflight(&changed, &columns, 2).is_err());
        let mut columns = columns;
        columns[1].physical_type = "UInt32".into();
        assert!(ipc_preflight(&bytes, &columns, 2).is_err());
    }
}

/// Inspect the finite flat Parquet profile, including every actual page header.
/// Footer totals alone cannot bound decompressor allocation.
pub(super) fn parquet_preflight(
    bytes: &[u8],
    declared: &[ColumnSchema],
    rows: usize,
) -> Result<usize, ValidationError> {
    use polars_parquet_format::{
        thrift::protocol::TCompactInputProtocol, CompressionCodec, Encoding, FieldRepetitionType,
        FileMetaData, PageHeader, PageType, Type,
    };
    use std::io::Cursor;
    const MAX_DECODED: usize = 64 * 1024 * 1024;
    if bytes.len() > 4 * 1024 * 1024
        || bytes.len() < 12
        || rows > 256
        || declared.is_empty()
        || declared.len() > 64
        || &bytes[..4] != b"PAR1"
        || &bytes[bytes.len() - 4..] != b"PAR1"
    {
        return Err(invalid("unsupported Parquet envelope"));
    }
    let footer_len =
        u32::from_le_bytes(bytes[bytes.len() - 8..bytes.len() - 4].try_into().unwrap()) as usize;
    if footer_len > 1024 * 1024 || footer_len > bytes.len() - 12 {
        return Err(invalid("Parquet footer bound exceeded"));
    }
    let footer_at = bytes.len() - 8 - footer_len;
    let mut footer_cursor = Cursor::new(&bytes[footer_at..bytes.len() - 8]);
    let metadata = FileMetaData::read_from_in_protocol(&mut TCompactInputProtocol::new(
        &mut footer_cursor,
        1024 * 1024,
    ))
    .map_err(invalid)?;
    if footer_cursor.position() as usize != footer_len
        || metadata.num_rows != rows as i64
        || ![1, 2].contains(&metadata.version)
        || metadata.encryption_algorithm.is_some()
        || metadata.footer_signing_key_metadata.is_some()
        || metadata.schema.len() != declared.len() + 1
        || metadata.schema[0].num_children != Some(declared.len() as i32)
        || metadata.row_groups.len() > rows.max(1)
    {
        return Err(invalid("unsupported Parquet footer profile"));
    }
    // Bind embedded Arrow schema to this exact pinned producer's flat types.
    // Comparing the standard encoder output avoids decoding untrusted nested
    // Arrow metadata (and requires no parallel Base64/FlatBuffer implementation).
    let mut expected_columns = Vec::with_capacity(declared.len());
    for column in declared {
        if column.name.is_empty()
            || column.name.len() > 64
            || !column
                .name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return Err(invalid("unsupported Parquet field name"));
        }
        let dtype = match column.physical_type.as_str() {
            "String" => DataType::String,
            "UInt64" => DataType::UInt64,
            "UInt32" => DataType::UInt32,
            "Float64" => DataType::Float64,
            "Boolean" => DataType::Boolean,
            _ => return Err(invalid("unsupported Parquet physical type")),
        };
        expected_columns.push(Series::new_empty(column.name.clone().into(), &dtype).into());
    }
    let mut empty = DataFrame::new(0, expected_columns).map_err(invalid)?;
    let mut encoded = BoundedPartWriter::new(Vec::new(), 65536);
    encode(
        &mut empty,
        NativeFormat::Parquet,
        &BTreeMap::new(),
        &mut encoded,
    )?;
    let expected = encoded.into_inner();
    let expected_footer = u32::from_le_bytes(
        expected[expected.len() - 8..expected.len() - 4]
            .try_into()
            .unwrap(),
    ) as usize;
    let expected_metadata = FileMetaData::read_from_in_protocol(&mut TCompactInputProtocol::new(
        Cursor::new(&expected[expected.len() - 8 - expected_footer..expected.len() - 8]),
        65536,
    ))
    .map_err(invalid)?;
    let schema_values = |m: &FileMetaData| -> Vec<Option<String>> {
        m.key_value_metadata
            .iter()
            .flatten()
            .filter(|kv| kv.key == "ARROW:schema")
            .map(|kv| kv.value.clone())
            .collect()
    };
    let expected_schema = schema_values(&expected_metadata);
    if expected_schema.len() != 1
        || expected_schema[0].is_none()
        || schema_values(&metadata) != expected_schema
    {
        return Err(invalid(
            "Parquet embedded Arrow schema differs from closed producer profile",
        ));
    }
    let physical = |name: &str| -> Result<Type, ValidationError> {
        Ok(match name {
            "String" => Type::BYTE_ARRAY,
            "UInt64" => Type::INT64,
            "UInt32" => Type::INT32,
            "Float64" => Type::DOUBLE,
            "Boolean" => Type::BOOLEAN,
            _ => return Err(invalid("unsupported Parquet physical type")),
        })
    };
    for (field, column) in metadata.schema[1..].iter().zip(declared) {
        if field.name != column.name
            || field.type_ != Some(physical(&column.physical_type)?)
            || field.type_length.is_some()
            || field.num_children.is_some_and(|n| n != 0)
            || !matches!(
                field.repetition_type,
                Some(FieldRepetitionType::OPTIONAL | FieldRepetitionType::REQUIRED)
            )
        {
            return Err(invalid("unsupported Parquet flat field"));
        }
    }
    let mut group_rows = 0usize;
    let mut decoded = 0usize;
    let mut ranges = Vec::new();
    for group in metadata.row_groups {
        let n = usize::try_from(group.num_rows).map_err(invalid)?;
        group_rows = group_rows
            .checked_add(n)
            .ok_or_else(|| invalid("Parquet row overflow"))?;
        if group_rows > rows || group.columns.len() != declared.len() || (n == 0 && rows != 0) {
            return Err(invalid("Parquet row group mismatch"));
        }
        for (chunk, column) in group.columns.into_iter().zip(declared) {
            if chunk.file_path.is_some()
                || chunk.crypto_metadata.is_some()
                || chunk.encrypted_column_metadata.is_some()
            {
                return Err(invalid("external or encrypted Parquet column"));
            }
            let meta = chunk
                .meta_data
                .ok_or_else(|| invalid("missing Parquet column metadata"))?;
            if meta.path_in_schema != [column.name.clone()]
                || meta.type_ != physical(&column.physical_type)?
                || meta.codec != CompressionCodec::ZSTD
                || meta.num_values != n as i64
                || meta.index_page_offset.is_some()
                || meta.encodings.iter().any(|e| {
                    ![Encoding::PLAIN, Encoding::RLE, Encoding::RLE_DICTIONARY].contains(e)
                })
            {
                return Err(invalid("unsupported Parquet column profile"));
            }
            let start =
                usize::try_from(meta.dictionary_page_offset.unwrap_or(meta.data_page_offset))
                    .map_err(invalid)?;
            let compressed = usize::try_from(meta.total_compressed_size).map_err(invalid)?;
            let declared_decoded =
                usize::try_from(meta.total_uncompressed_size).map_err(invalid)?;
            let stop = start
                .checked_add(compressed)
                .ok_or_else(|| invalid("Parquet column overflow"))?;
            if start < 4 || stop > footer_at || declared_decoded > MAX_DECODED {
                return Err(invalid("Parquet column range or decoded bound exceeded"));
            }
            ranges.push((start, stop));
            let mut at = start;
            let mut values = 0usize;
            let mut pages = 0usize;
            let mut dictionary = false;
            let mut dictionary_bytes = 0usize;
            let mut actual_decoded = 0usize;
            while at < stop {
                pages += 1;
                if pages > n.max(1) + 1 {
                    return Err(invalid("Parquet page count bound exceeded"));
                }
                let mut cursor = Cursor::new(&bytes[at..stop.min(at.saturating_add(65536))]);
                let page = PageHeader::read_from_in_protocol(&mut TCompactInputProtocol::new(
                    &mut cursor,
                    65536,
                ))
                .map_err(invalid)?;
                let header_bytes = cursor.position() as usize;
                let packed = usize::try_from(page.compressed_page_size).map_err(invalid)?;
                let unpacked = usize::try_from(page.uncompressed_page_size).map_err(invalid)?;
                if unpacked > MAX_DECODED || page.index_page_header.is_some() {
                    return Err(invalid("Parquet page decoded bound exceeded"));
                }
                actual_decoded = actual_decoded
                    .checked_add(header_bytes)
                    .and_then(|v| v.checked_add(unpacked))
                    .ok_or_else(|| invalid("Parquet decoded overflow"))?;
                decoded = decoded
                    .checked_add(unpacked)
                    .ok_or_else(|| invalid("Parquet decoded overflow"))?;
                if decoded > MAX_DECODED {
                    return Err(invalid("Parquet part decoded bound exceeded"));
                }
                let count = match page.type_ {
                    PageType::DICTIONARY_PAGE => {
                        if dictionary
                            || values != 0
                            || page.data_page_header.is_some()
                            || page.data_page_header_v2.is_some()
                        {
                            return Err(invalid("unsupported Parquet dictionary order"));
                        }
                        dictionary = true;
                        let h = page
                            .dictionary_page_header
                            .ok_or_else(|| invalid("missing Parquet dictionary header"))?;
                        if h.encoding != Encoding::PLAIN {
                            return Err(invalid("unsupported Parquet dictionary encoding"));
                        }
                        dictionary_bytes = unpacked;
                        let entries = usize::try_from(h.num_values).map_err(invalid)?;
                        if entries > n.max(1) {
                            return Err(invalid("Parquet dictionary value bound exceeded"));
                        }
                        0
                    }
                    PageType::DATA_PAGE => {
                        if let Some(h) = &page.data_page_header {
                            if ![Encoding::PLAIN, Encoding::RLE_DICTIONARY].contains(&h.encoding)
                                || h.definition_level_encoding != Encoding::RLE
                                || h.repetition_level_encoding != Encoding::RLE
                            {
                                return Err(invalid("unsupported Parquet data encoding"));
                            }
                        }
                        if page.dictionary_page_header.is_some()
                            || page.data_page_header_v2.is_some()
                        {
                            return Err(invalid("ambiguous Parquet page header"));
                        }
                        usize::try_from(
                            page.data_page_header
                                .ok_or_else(|| invalid("missing Parquet data header"))?
                                .num_values,
                        )
                        .map_err(invalid)?
                    }
                    PageType::DATA_PAGE_V2 => {
                        if page.dictionary_page_header.is_some() || page.data_page_header.is_some()
                        {
                            return Err(invalid("ambiguous Parquet page header"));
                        }
                        let h = page
                            .data_page_header_v2
                            .ok_or_else(|| invalid("missing Parquet V2 header"))?;
                        if ![Encoding::PLAIN, Encoding::RLE_DICTIONARY].contains(&h.encoding) {
                            return Err(invalid("unsupported Parquet data encoding"));
                        }
                        let count = usize::try_from(h.num_values).map_err(invalid)?;
                        let nulls = usize::try_from(h.num_nulls).map_err(invalid)?;
                        let definitions =
                            usize::try_from(h.definition_levels_byte_length).map_err(invalid)?;
                        let repetitions =
                            usize::try_from(h.repetition_levels_byte_length).map_err(invalid)?;
                        if h.num_rows != h.num_values
                            || nulls > count
                            || repetitions != 0
                            || definitions > packed
                            || definitions > unpacked
                            || (h.is_compressed == Some(false) && packed != unpacked)
                        {
                            return Err(invalid("unsupported Parquet V2 lengths"));
                        }
                        count
                    }
                    _ => return Err(invalid("unsupported Parquet page type")),
                };
                values = values
                    .checked_add(count)
                    .ok_or_else(|| invalid("Parquet value count overflow"))?;
                if values > n {
                    return Err(invalid("Parquet actual page values exceed rows"));
                }
                at = at
                    .checked_add(header_bytes)
                    .and_then(|v| v.checked_add(packed))
                    .ok_or_else(|| invalid("Parquet page range overflow"))?;
                if at > stop {
                    return Err(invalid("Parquet page exceeds column"));
                }
            }
            if column.physical_type == "String" && dictionary {
                decoded = decoded
                    .checked_add(
                        dictionary_bytes
                            .checked_mul(n)
                            .ok_or_else(|| invalid("Parquet dictionary expansion overflow"))?,
                    )
                    .ok_or_else(|| invalid("Parquet dictionary expansion overflow"))?;
                if decoded > MAX_DECODED {
                    return Err(invalid(
                        "Parquet expanded dictionary exceeds decoded part bound",
                    ));
                }
            }
            if at != stop || values != n || actual_decoded != declared_decoded {
                return Err(invalid("Parquet actual column totals mismatch"));
            }
        }
    }
    if group_rows != rows {
        return Err(invalid("Parquet row total mismatch"));
    }
    ranges.sort_unstable();
    if ranges.windows(2).any(|w| w[0].1 > w[1].0) {
        return Err(invalid("overlapping Parquet columns"));
    }
    Ok(decoded)
}

#[cfg(test)]
mod parquet_profile_tests {
    use super::*;
    use polars_parquet_format::{
        thrift::protocol::{TCompactInputProtocol, TCompactOutputProtocol},
        FileMetaData, PageHeader,
    };
    use std::io::Cursor;
    fn source(empty: bool) -> (Vec<u8>, Vec<ColumnSchema>) {
        let mut frame = if empty {
            DataFrame::new(
                0,
                vec![
                    Series::new("label".into(), Vec::<Option<&str>>::new()).into(),
                    Series::new("number".into(), Vec::<u64>::new()).into(),
                ],
            )
            .unwrap()
        } else {
            df!("label"=>[Some("é"),None],"number"=>[1u64,2]).unwrap()
        };
        let columns = vec![
            ColumnSchema {
                name: "label".into(),
                physical_type: "String".into(),
                nullable: true,
            },
            ColumnSchema {
                name: "number".into(),
                physical_type: "UInt64".into(),
                nullable: false,
            },
        ];
        let mut bytes = Vec::new();
        encode(
            &mut frame,
            NativeFormat::Parquet,
            &BTreeMap::new(),
            &mut bytes,
        )
        .unwrap();
        (bytes, columns)
    }
    #[test]
    fn actual_parquet_pages_bound_allocation_independently_of_footer_totals() {
        let (bytes, columns) = source(false);
        assert!(parquet_preflight(&bytes, &columns, 2).unwrap() > 0);
        let footer_len =
            u32::from_le_bytes(bytes[bytes.len() - 8..bytes.len() - 4].try_into().unwrap())
                as usize;
        let footer_at = bytes.len() - 8 - footer_len;
        let metadata = FileMetaData::read_from_in_protocol(&mut TCompactInputProtocol::new(
            Cursor::new(&bytes[footer_at..bytes.len() - 8]),
            1024 * 1024,
        ))
        .unwrap();
        let meta = metadata.row_groups[0].columns[0]
            .meta_data
            .as_ref()
            .unwrap();
        let start = meta.dictionary_page_offset.unwrap_or(meta.data_page_offset) as usize;
        let mut page = PageHeader::read_from_in_protocol(&mut TCompactInputProtocol::new(
            Cursor::new(&bytes[start..footer_at]),
            65536,
        ))
        .unwrap();
        let original_page = page.clone();
        page.uncompressed_page_size = i32::MAX;
        let mut header = Vec::new();
        page.write_to_out_protocol(&mut TCompactOutputProtocol::new(&mut header))
            .unwrap();
        let mut changed = bytes.clone();
        changed[start..start + header.len()].copy_from_slice(&header);
        let error = parquet_preflight(&changed, &columns, 2).unwrap_err();
        assert!(error.message.contains("page decoded bound"), "{error:?}");
        let mut bad_page = original_page;
        if let Some(h) = bad_page.dictionary_page_header.as_mut() {
            h.encoding = polars_parquet_format::Encoding::DELTA_BYTE_ARRAY;
        }
        if let Some(h) = bad_page.data_page_header.as_mut() {
            h.encoding = polars_parquet_format::Encoding::DELTA_BYTE_ARRAY;
        }
        if let Some(h) = bad_page.data_page_header_v2.as_mut() {
            h.encoding = polars_parquet_format::Encoding::DELTA_BYTE_ARRAY;
        }
        let mut header = Vec::new();
        bad_page
            .write_to_out_protocol(&mut TCompactOutputProtocol::new(&mut header))
            .unwrap();
        let mut changed = bytes.clone();
        changed[start..start + header.len()].copy_from_slice(&header);
        assert!(parquet_preflight(&changed, &columns, 2)
            .unwrap_err()
            .message
            .contains("encoding"));
        let mut bad_metadata = metadata;
        for kv in bad_metadata.key_value_metadata.as_mut().unwrap() {
            if kv.key == "ARROW:schema" {
                kv.value = Some("unqualified embedded schema".into());
            }
        }
        let mut footer = Vec::new();
        bad_metadata
            .write_to_out_protocol(&mut TCompactOutputProtocol::new(&mut footer))
            .unwrap();
        let mut changed = bytes[..footer_at].to_vec();
        changed.extend_from_slice(&footer);
        changed.extend_from_slice(&(footer.len() as u32).to_le_bytes());
        changed.extend_from_slice(b"PAR1");
        assert!(parquet_preflight(&changed, &columns, 2)
            .unwrap_err()
            .message
            .contains("embedded Arrow schema"));
        assert!(parquet_preflight(&bytes, &columns, 1).is_err());
        for stop in [0, 4, bytes.len() - 1] {
            assert!(parquet_preflight(&bytes[..stop], &columns, 2).is_err());
        }
        let (empty, columns) = source(true);
        parquet_preflight(&empty, &columns, 0).unwrap();
    }
}

#[derive(Default)]
pub(super) struct ProfileBudget {
    encoded: usize,
    decoded: usize,
}
impl ProfileBudget {
    pub(super) fn add(
        &mut self,
        bytes: &[u8],
        format: NativeFormat,
        columns: &[ColumnSchema],
        rows: usize,
    ) -> Result<(), ValidationError> {
        self.encoded = self
            .encoded
            .checked_add(bytes.len())
            .ok_or_else(|| invalid("native encoded dataset overflow"))?;
        if self.encoded > 64 * 1024 * 1024 {
            return Err(invalid("native encoded dataset profile exceeded"));
        }
        let payload = match format {
            NativeFormat::Parquet => parquet_preflight(bytes, columns, rows)?,
            NativeFormat::Ipc => {
                ipc_preflight(bytes, columns, rows)?;
                bytes.len()
            }
            NativeFormat::Csv => {
                csv_schema(bytes, columns, rows)?;
                bytes.len()
            }
        };
        let bound = rows
            .checked_mul(columns.len())
            .and_then(|v| v.checked_mul(16))
            .and_then(|v| v.checked_add(payload))
            .ok_or_else(|| invalid("native decoded profile overflow"))?;
        self.decoded = self
            .decoded
            .checked_add(bound)
            .ok_or_else(|| invalid("native decoded dataset overflow"))?;
        if bound > 64 * 1024 * 1024 || self.decoded > 256 * 1024 * 1024 {
            return Err(invalid("native decoded allocation profile exceeded"));
        }
        Ok(())
    }
}
pub(super) fn check_provenance(
    frames: &[DataFrame; 3],
    m: &NativeManifest,
) -> Result<(), ValidationError> {
    let patients = &frames[0];
    let replications = &frames[1];
    if m.replication_seeds.is_empty() || replications.height() != m.replication_seeds.len() {
        return Err(invalid(
            "native replication provenance cardinality mismatch",
        ));
    }
    let string_matches =
        |frame: &DataFrame, name: &str, expected: &str| -> Result<(), ValidationError> {
            let column = frame.column(name).and_then(|c| c.str()).map_err(invalid)?;
            if (0..frame.height()).any(|i| column.get(i) != Some(expected)) {
                return Err(invalid(format!("native row provenance mismatch: {name}")));
            }
            Ok(())
        };
    for (name, value) in [
        ("run_id", m.batch_id.as_str()),
        ("input_sha256", &m.effective_input_sha256),
        ("model_contract_sha256", &m.model_contract_sha256),
        ("scenario_id", &m.scenario_id),
        ("time_unit", &m.time_unit),
        ("schema", "careops.ed.patients.v2"),
        ("capture_schema", "careops.ed.capture-boundary.v2"),
        ("capture_table_schema", "careops.ed.patient-result.v2"),
    ] {
        string_matches(patients, name, value)?;
    }
    string_matches(replications, "run_id", &m.batch_id)?;
    let indices = replications
        .column("replication_index")
        .and_then(|c| c.u64())
        .map_err(invalid)?;
    let seeds = replications
        .column("seed")
        .and_then(|c| c.u64())
        .map_err(invalid)?;
    let horizons = replications
        .column("horizon_ticks")
        .and_then(|c| c.u64())
        .map_err(invalid)?;
    for (index, seed) in m.replication_seeds.iter().enumerate() {
        if indices.get(index) != Some(index as u64)
            || seeds.get(index) != Some(*seed)
            || horizons.get(index) != Some(m.horizon_ticks)
        {
            return Err(invalid(
                "native replication index/seed/horizon provenance mismatch",
            ));
        }
    }
    let indices = patients
        .column("replication_index")
        .and_then(|c| c.u64())
        .map_err(invalid)?;
    let seeds = patients
        .column("seed")
        .and_then(|c| c.u64())
        .map_err(invalid)?;
    for row in 0..patients.height() {
        let index = usize::try_from(
            indices
                .get(row)
                .ok_or_else(|| invalid("missing patient replication"))?,
        )
        .map_err(invalid)?;
        if m.replication_seeds.get(index).copied() != seeds.get(row)
            || index >= m.replication_seeds.len()
        {
            return Err(invalid(
                "native patient replication/seed provenance mismatch",
            ));
        }
    }
    let total = frames[2]
        .column("total_replications")
        .and_then(|c| c.u64())
        .map_err(invalid)?;
    if (0..frames[2].height()).any(|i| total.get(i) != Some(m.replication_seeds.len() as u64)) {
        return Err(invalid("native metric replication provenance mismatch"));
    }
    Ok(())
}
