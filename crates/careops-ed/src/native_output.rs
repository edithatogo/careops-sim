//! Native ED output controls. Patient facts stay in typed standard files.
use crate::disposition::api_runner::{ExitCodeDef, ValidationError};
use crate::disposition::batch_runner::BatchRunDocument;
use crate::ScenarioConfig;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const SCHEMA: &str = "careops.ed.native-output.v1";
pub const DATA_REVISION: &str = "473ff2c0e640a7e457f500593f6ea1f06602ab35";
pub const POLARS_REVISION: &str = "96439685eb2f95108f51087d2b76091917f2593c";
pub const RUNTIME_REVISION: &str = "07410266bf54b6f73c59db2f8cbfbc5697c53734";
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeFormat {
    #[default]
    Parquet,
    Ipc,
    Csv,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportLimits {
    pub max_total_rows: usize,
    pub max_parts: usize,
    pub max_rows_per_part: usize,
    pub max_bytes_per_part: usize,
    pub max_manifest_bytes: usize,
}
impl Default for ExportLimits {
    fn default() -> Self {
        Self {
            max_total_rows: 1_000_000,
            max_parts: 4096,
            max_rows_per_part: 256,
            max_bytes_per_part: 4 * 1024 * 1024,
            max_manifest_bytes: 1024 * 1024,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColumnSchema {
    pub name: String,
    pub physical_type: String,
    pub nullable: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativePart {
    pub file: String,
    pub first_row: usize,
    pub rows: usize,
    pub bytes: usize,
    pub sha256: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeTable {
    pub name: String,
    pub schema: String,
    pub rows: usize,
    pub columns: Vec<ColumnSchema>,
    pub parts: Vec<NativePart>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeManifest {
    pub schema: String,
    pub status: String,
    pub format: NativeFormat,
    pub compression: Option<String>,
    pub batch_id: String,
    pub scenario_id: String,
    pub time_unit: String,
    pub horizon_ticks: u64,
    pub base_seed: u64,
    pub replication_seeds: Vec<u64>,
    pub effective_input_sha256: String,
    pub parsed_input_sha256: String,
    pub raw_input_sha256: Option<String>,
    pub model_contract_sha256: String,
    pub runtime_revision: String,
    pub data_revision: String,
    pub polars_revision: String,
    pub producer: String,
    pub limits: ExportLimits,
    pub tables: Vec<NativeTable>,
}

use crate::capture::{BoundedPartWriter, PartStore};
use crate::disposition::arrow_export::{native_tables, summarize_replications, MetricRecord};
use crate::parquet_capture::ParquetPartStore;
use polars::prelude::*;
use std::collections::BTreeMap;
use std::io::Write;
fn invalid(e: impl std::fmt::Display) -> ValidationError {
    ValidationError::new(ExitCodeDef::MalformedConfig, e.to_string())
}
fn output(e: impl std::fmt::Display) -> ValidationError {
    ValidationError::new(ExitCodeDef::OutputWriteFailed, e.to_string())
}
fn json_control(manifest: &NativeManifest) -> Result<Vec<u8>, ValidationError> {
    let mut writer = BoundedPartWriter::new(Vec::new(), manifest.limits.max_manifest_bytes);
    serde_json::to_writer_pretty(&mut writer, manifest).map_err(invalid)?;
    Ok(writer.into_inner())
}
impl NativeFormat {
    fn extension(self) -> &'static str {
        match self {
            Self::Parquet => "parquet",
            Self::Ipc => "arrow",
            Self::Csv => "csv",
        }
    }
    fn codec(self) -> &'static str {
        match self {
            Self::Parquet => "parquet-zstd",
            Self::Ipc => "arrow-ipc-stream",
            Self::Csv => "csv-necessary-quotes-empty-null",
        }
    }
}
impl ExportLimits {
    fn validate(self) -> Result<(), ValidationError> {
        let ceiling = Self::default();
        for (value, max) in [
            (self.max_total_rows, ceiling.max_total_rows),
            (self.max_parts, ceiling.max_parts),
            (self.max_rows_per_part, ceiling.max_rows_per_part),
            (self.max_bytes_per_part, ceiling.max_bytes_per_part),
            (self.max_manifest_bytes, ceiling.max_manifest_bytes),
        ] {
            if value == 0 || value > max {
                return Err(invalid(
                    "native export budget outside qualified finite envelope",
                ));
            }
        }
        Ok(())
    }
}
fn columns(frame: &DataFrame, table: &str) -> Result<Vec<ColumnSchema>, ValidationError> {
    let declared = reader::closed_columns(table)?;
    reader::check_frame(frame, &declared, frame.height())?;
    Ok(declared)
}
fn replication_frame(document: &BatchRunDocument) -> Result<DataFrame, ValidationError> {
    let rows = &document.replications;
    df!(
        "run_id"=>vec![document.batch_id.as_str();rows.len()],
        "replication_index"=>rows.iter().map(|r|r.replication_index as u64).collect::<Vec<_>>(),
        "seed"=>rows.iter().map(|r|r.seed).collect::<Vec<_>>(),
        "horizon_ticks"=>rows.iter().map(|r|r.horizon_ticks).collect::<Vec<_>>(),
        "arrivals"=>rows.iter().map(|r|r.arrivals as u64).collect::<Vec<_>>(),
        "started"=>rows.iter().map(|r|r.started as u64).collect::<Vec<_>>(),
        "completed"=>rows.iter().map(|r|r.completed as u64).collect::<Vec<_>>(),
        "unfinished"=>rows.iter().map(|r|r.unfinished as u64).collect::<Vec<_>>(),
        "mean_wait_ticks"=>rows.iter().map(|r|(r.started>0).then_some(r.mean_wait_ticks)).collect::<Vec<_>>()
    ).map_err(invalid)
}
fn metric_frame(rows: &[MetricRecord]) -> Result<DataFrame, ValidationError> {
    df!(
        "name"=>rows.iter().map(|r|r.name.as_str()).collect::<Vec<_>>(),
        "value"=>rows.iter().map(|r|r.value).collect::<Vec<_>>(),
        "unit"=>rows.iter().map(|r|r.unit.as_str()).collect::<Vec<_>>(),
        "denominator_name"=>rows.iter().map(|r|r.denominator_name.as_str()).collect::<Vec<_>>(),
        "denominator_value"=>rows.iter().map(|r|r.denominator_value).collect::<Vec<_>>(),
        "numerator_count"=>rows.iter().map(|r|r.numerator_count).collect::<Vec<_>>(),
        "denominator_count"=>rows.iter().map(|r|r.denominator_count).collect::<Vec<_>>(),
        "estimand"=>rows.iter().map(|r|r.estimand.as_str()).collect::<Vec<_>>(),
        "sample_unit"=>rows.iter().map(|r|r.sample_unit.as_str()).collect::<Vec<_>>(),
        "weighting"=>rows.iter().map(|r|r.weighting.as_str()).collect::<Vec<_>>(),
        "eligible_replications"=>rows.iter().map(|r|r.eligible_replications as u64).collect::<Vec<_>>(),
        "included_replications"=>rows.iter().map(|r|r.included_replications as u64).collect::<Vec<_>>(),
        "total_replications"=>rows.iter().map(|r|r.total_replications as u64).collect::<Vec<_>>(),
        "estimate"=>rows.iter().map(|r|r.uncertainty.estimate).collect::<Vec<_>>(),
        "variance"=>rows.iter().map(|r|r.uncertainty.variance).collect::<Vec<_>>(),
        "standard_error"=>rows.iter().map(|r|r.uncertainty.standard_error).collect::<Vec<_>>(),
        "ci_95_lower"=>rows.iter().map(|r|r.uncertainty.ci_95_lower).collect::<Vec<_>>(),
        "ci_95_upper"=>rows.iter().map(|r|r.uncertainty.ci_95_upper).collect::<Vec<_>>(),
        "uncertainty_replications"=>rows.iter().map(|r|r.uncertainty.replications_count as u64).collect::<Vec<_>>(),
        "ddof"=>rows.iter().map(|r|r.uncertainty.ddof.map(u32::from)).collect::<Vec<_>>(),
        "method"=>rows.iter().map(|r|r.uncertainty.method.as_str()).collect::<Vec<_>>(),
        "status"=>rows.iter().map(|r|r.uncertainty.status.as_str()).collect::<Vec<_>>(),
        "censored_count"=>rows.iter().map(|r|r.censored_count).collect::<Vec<_>>(),
        "unfinished_count"=>rows.iter().map(|r|r.unfinished_count).collect::<Vec<_>>()
    ).map_err(invalid)
}
fn file_metadata(
    manifest: &NativeManifest,
    table: &NativeTable,
    first: usize,
    rows: usize,
) -> BTreeMap<String, String> {
    [
        ("schema", table.schema.clone()),
        ("output_schema", SCHEMA.into()),
        ("codec", manifest.format.codec().into()),
        ("table", table.name.clone()),
        ("run_id", manifest.batch_id.clone()),
        ("scenario_id", manifest.scenario_id.clone()),
        ("time_unit", manifest.time_unit.clone()),
        ("first_row", first.to_string()),
        ("rows", rows.to_string()),
        (
            "effective_input_sha256",
            manifest.effective_input_sha256.clone(),
        ),
        ("parsed_input_sha256", manifest.parsed_input_sha256.clone()),
        (
            "model_contract_sha256",
            manifest.model_contract_sha256.clone(),
        ),
        ("runtime_revision", manifest.runtime_revision.clone()),
        ("data_revision", manifest.data_revision.clone()),
        ("polars_revision", manifest.polars_revision.clone()),
        ("producer", manifest.producer.clone()),
    ]
    .into_iter()
    .map(|(k, v)| (format!("careops.{k}"), v))
    .collect()
}
fn encode(
    frame: &mut DataFrame,
    format: NativeFormat,
    metadata: &BTreeMap<String, String>,
    writer: &mut dyn Write,
) -> Result<(), ValidationError> {
    match format {
        NativeFormat::Parquet => kairo_ecs_arrow::polars_interop::write_parquet_zstd_with_metadata(
            frame, writer, metadata,
        )
        .map_err(output),
        NativeFormat::Ipc => {
            kairo_ecs_arrow::polars_interop::write_ipc_stream_with_metadata(frame, writer, metadata)
                .map_err(output)
        }
        NativeFormat::Csv => {
            // CSV has no embedded metadata channel. The strict control manifest
            // binds its exact schema, types, nullable fields and actual bytes.
            CsvWriter::new(writer)
                .include_header(true)
                .with_quote_style(QuoteStyle::Necessary)
                .with_null_value(String::new().into())
                .finish(frame)
                .map_err(output)
        }
    }
}
fn plan_part(
    frame: &DataFrame,
    manifest: &NativeManifest,
    table: &mut NativeTable,
    first: usize,
    rows: usize,
    parts: &mut usize,
    budget: &mut reader::ProfileBudget,
) -> Result<(), ValidationError> {
    let metadata = file_metadata(manifest, table, first, rows);
    let mut writer = BoundedPartWriter::new(Vec::new(), manifest.limits.max_bytes_per_part);
    let result = encode(
        &mut frame.slice(first as i64, rows),
        manifest.format,
        &metadata,
        &mut writer,
    );
    if writer.limit_exceeded() {
        // Release the bounded encoded buffer before descending into a split.
        drop(writer);
        if rows <= 1 {
            return Err(invalid(
                "native single row or typed empty file exceeds byte budget",
            ));
        }
        let left = rows / 2;
        plan_part(frame, manifest, table, first, left, parts, budget)?;
        return plan_part(
            frame,
            manifest,
            table,
            first + left,
            rows - left,
            parts,
            budget,
        );
    }
    result?;
    if *parts >= manifest.limits.max_parts {
        return Err(invalid("native total part budget exceeded"));
    }
    let encoded_bytes = writer.written();
    let encoded = writer.into_inner();
    budget.add(&encoded, manifest.format, &table.columns, rows)?;
    if manifest.format == NativeFormat::Csv {
        reader::csv_part(&encoded, &table.columns, rows)?;
    }
    *parts += 1;
    table.parts.push(NativePart {
        file: format!(
            "{}-{:05}.{}",
            table.name,
            table.parts.len(),
            manifest.format.extension()
        ),
        first_row: first,
        rows,
        bytes: encoded_bytes,
        sha256: "0".repeat(64),
    });
    Ok(())
}
/// Export validated joined patient facts and their qualified metrics into a fresh
/// caller-owned directory. Budgets cover all tables and the control manifest.
/// Derived files do not rerun the simulation. Incomplete export is not resumable.
pub fn export_native_batch(
    scenario: &ScenarioConfig,
    document: &BatchRunDocument,
    destination: &Path,
    format: NativeFormat,
    limits: ExportLimits,
) -> Result<NativeManifest, ValidationError> {
    limits.validate()?;
    let destination_meta = std::fs::symlink_metadata(destination).map_err(output)?;
    if !destination_meta.is_dir()
        || destination_meta.file_type().is_symlink()
        || std::fs::read_dir(destination)
            .map_err(output)?
            .next()
            .is_some()
    {
        return Err(invalid(
            "native export destination must be a fresh empty real directory",
        ));
    }
    // Check authoritative declared row budgets BEFORE assembling a DataFrame.
    let patients = document.replications.iter().try_fold(0usize, |n, r| {
        n.checked_add(r.arrivals)
            .ok_or_else(|| invalid("native row count overflow"))
    })?;
    let total = patients
        .checked_add(document.replications.len())
        .and_then(|n| n.checked_add(5))
        .ok_or_else(|| invalid("native total row count overflow"))?;
    if total > limits.max_total_rows {
        return Err(invalid("native total row budget exceeded"));
    }
    let source = crate::captured_batch::export_source(scenario, document)?;
    let capture = source.2.clone();
    let facts = native_tables::patient_table_from_validated_capture(scenario, document, source)?;
    if facts.height() != patients {
        return Err(invalid("native patient count differs from capture"));
    }
    let hi = facts
        .column("elapsed_ticks_hi")
        .and_then(|c| c.u64())
        .map_err(invalid)?;
    let lo = facts
        .column("elapsed_ticks_lo")
        .and_then(|c| c.u64())
        .map_err(invalid)?;
    let durations = (0..facts.height())
        .map(|index| match (hi.get(index), lo.get(index)) {
            (Some(hi), Some(lo)) => Ok(Some((u128::from(hi) << 64) | u128::from(lo))),
            (None, None) => Ok(None),
            _ => Err(invalid("half-null elapsed limbs")),
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    let metrics = summarize_replications(&document.replications, &durations, &scenario.time_unit)?;
    if metrics.len() != 5 {
        return Err(invalid("native metric schema cardinality changed"));
    }
    let frames = [facts, replication_frame(document)?, metric_frame(&metrics)?];
    let manifest = NativeManifest {
        schema: SCHEMA.into(),
        status: "complete".into(),
        format,
        compression: (format == NativeFormat::Parquet).then(|| "zstd".into()),
        batch_id: document.batch_id.clone(),
        scenario_id: scenario.scenario_id.clone(),
        time_unit: scenario.time_unit.clone(),
        horizon_ticks: capture
            .config
            .horizon_override
            .unwrap_or(scenario.horizon_ticks),
        base_seed: capture.config.base_seed,
        replication_seeds: document.replications.iter().map(|r| r.seed).collect(),
        effective_input_sha256: capture.effective_input_sha256,
        parsed_input_sha256: capture.parsed_input_sha256,
        raw_input_sha256: None,
        model_contract_sha256: capture.model_contract_sha256,
        runtime_revision: RUNTIME_REVISION.into(),
        data_revision: DATA_REVISION.into(),
        polars_revision: POLARS_REVISION.into(),
        producer: format!("careops-ed/{}", env!("CARGO_PKG_VERSION")),
        limits,
        tables: Vec::new(),
    };
    publish_frames(manifest, frames, destination)
}

fn publish_frames(
    mut manifest: NativeManifest,
    frames: [DataFrame; 3],
    destination: &Path,
) -> Result<NativeManifest, ValidationError> {
    let limits = manifest.limits;
    let format = manifest.format;
    limits.validate()?;
    let meta = std::fs::symlink_metadata(destination).map_err(output)?;
    if !meta.is_dir()
        || meta.file_type().is_symlink()
        || std::fs::read_dir(destination)
            .map_err(output)?
            .next()
            .is_some()
    {
        return Err(invalid(
            "native export destination must be a fresh empty real directory",
        ));
    }
    let total = frames.iter().try_fold(0usize, |n, f| {
        n.checked_add(f.height())
            .ok_or_else(|| invalid("native total rows overflow"))
    })?;
    if total > limits.max_total_rows {
        return Err(invalid("native total row budget exceeded"));
    }
    manifest.tables.clear();
    reader::check_provenance(&frames, &manifest)?;
    let mut parts = 0;
    let mut budget = reader::ProfileBudget::default();
    for (frame, (name, schema)) in frames.iter().zip([
        ("patients", "careops.ed.patients.v2"),
        ("replications", "careops.ed.replications.v1"),
        ("metrics", "careops.ed.metrics.v1"),
    ]) {
        let mut table = NativeTable {
            name: name.into(),
            schema: schema.into(),
            rows: frame.height(),
            columns: columns(frame, name)?,
            parts: Vec::new(),
        };
        if frame.height() == 0 {
            plan_part(frame, &manifest, &mut table, 0, 0, &mut parts, &mut budget)?;
        } else {
            for first in (0..frame.height()).step_by(limits.max_rows_per_part) {
                plan_part(
                    frame,
                    &manifest,
                    &mut table,
                    first,
                    (frame.height() - first).min(limits.max_rows_per_part),
                    &mut parts,
                    &mut budget,
                )?;
            }
        }
        manifest.tables.push(table);
    }
    // Placeholder hashes have exactly the final hash width. All names, counts,
    // metadata and final control-size budgets are qualified before publication.
    json_control(&manifest)?;
    let store = ParquetPartStore::open(
        destination,
        crate::capture::CaptureLimits {
            max_rows: limits.max_rows_per_part,
            max_bytes: limits.max_bytes_per_part,
            max_patient_id_bytes: 4096,
        },
    )
    .map_err(crate::captured_batch::error)?;
    let mut actual_budget = reader::ProfileBudget::default();
    for (index, frame) in frames.iter().enumerate() {
        for part_index in 0..manifest.tables[index].parts.len() {
            let part = manifest.tables[index].parts[part_index].clone();
            let metadata = file_metadata(
                &manifest,
                &manifest.tables[index],
                part.first_row,
                part.rows,
            );
            store
                .publish(&part.file, |file| {
                    let mut bounded = BoundedPartWriter::new(file, limits.max_bytes_per_part);
                    encode(
                        &mut frame.slice(part.first_row as i64, part.rows),
                        format,
                        &metadata,
                        &mut bounded,
                    )
                    .map_err(|e| {
                        careops_core::experiment::ExperimentError::new(
                            careops_core::experiment::ExperimentErrorKind::OutputWriteFailed,
                            e.to_string(),
                        )
                    })
                })
                .map_err(crate::captured_batch::error)?;
            let bytes = store
                .read_part(&part.file, limits.max_bytes_per_part)
                .map_err(crate::captured_batch::error)?;
            if bytes.len() != part.bytes {
                return Err(output(
                    "native physical encoding differs from pure preflight",
                ));
            }
            actual_budget
                .add(&bytes, format, &manifest.tables[index].columns, part.rows)
                .map_err(output)?;
            if format == NativeFormat::Csv {
                reader::csv_part(&bytes, &manifest.tables[index].columns, part.rows)
                    .map_err(output)?;
            }
            manifest.tables[index].parts[part_index].sha256 =
                crate::disposition::api_runner::compute_sha256_hex(&bytes);
        }
    }
    let bytes = json_control(&manifest)?;
    store
        .publish("manifest.json", |file| {
            file.write_all(&bytes).map_err(|e| {
                careops_core::experiment::ExperimentError::new(
                    careops_core::experiment::ExperimentErrorKind::OutputWriteFailed,
                    e.to_string(),
                )
            })
        })
        .map_err(crate::captured_batch::error)?;
    Ok(manifest)
}

/// Stream the validated joined patient table through the qualified CSV writer.
/// A finite byte cap and all write/flush errors apply to the real sink. Partial
/// stdout is a failure, even when already-published disk artifacts are complete.
pub fn stream_patient_csv(
    scenario: &ScenarioConfig,
    document: &BatchRunDocument,
    writer: &mut dyn Write,
    max_bytes: usize,
) -> Result<(), ValidationError> {
    if max_bytes == 0 || max_bytes > 8 * 1024 * 1024 {
        return Err(invalid("patient CSV stream cap outside qualified budget"));
    }
    let rows = document.replications.iter().try_fold(0usize, |n, r| {
        n.checked_add(r.arrivals)
            .ok_or_else(|| invalid("patient stream row overflow"))
    })?;
    if rows > ExportLimits::default().max_total_rows {
        return Err(invalid("patient stream row budget exceeded"));
    }
    let mut patients = native_tables::patient_table(scenario, document)?;
    let mut bounded = BoundedPartWriter::new(writer, max_bytes);
    CsvWriter::new(&mut bounded)
        .include_header(true)
        .with_quote_style(QuoteStyle::Necessary)
        .with_null_value(String::new().into())
        .finish(&mut patients)
        .map_err(output)?;
    bounded.flush().map_err(output)
}

#[path = "native_reader.rs"]
mod reader;
pub use reader::{read_native_dataset, NativeDataset};
/// Convert a validated immutable native snapshot without rerunning its model.
pub fn convert_native_dataset(
    dataset: &NativeDataset,
    destination: &Path,
    format: NativeFormat,
    limits: ExportLimits,
) -> Result<NativeManifest, ValidationError> {
    let mut manifest = dataset.manifest.clone();
    manifest.format = format;
    manifest.compression = (format == NativeFormat::Parquet).then(|| "zstd".into());
    manifest.limits = limits;
    manifest.producer = format!("careops-ed/{}", env!("CARGO_PKG_VERSION"));
    let manifest = publish_frames(manifest, dataset.frames.clone(), destination)?;
    let target = reader::bounded_file(
        &destination.join("manifest.json"),
        limits.max_manifest_bytes,
    )?;
    let receipt = serde_json::json!({"schema":"careops.ed.native-conversion.v1","source_manifest_sha256":dataset.manifest_sha256,"target_manifest_sha256":crate::disposition::api_runner::compute_sha256_hex(&target)});
    let mut bounded = BoundedPartWriter::new(Vec::new(), 8192);
    serde_json::to_writer_pretty(&mut bounded, &receipt).map_err(output)?;
    let bytes = bounded.into_inner();
    let store = ParquetPartStore::open(
        destination,
        crate::capture::CaptureLimits {
            max_rows: limits.max_rows_per_part,
            max_bytes: limits.max_bytes_per_part,
            max_patient_id_bytes: 4096,
        },
    )
    .map_err(crate::captured_batch::error)?;
    store
        .publish("conversion.json", |file| {
            file.write_all(&bytes).map_err(|e| {
                careops_core::experiment::ExperimentError::new(
                    careops_core::experiment::ExperimentErrorKind::OutputWriteFailed,
                    e.to_string(),
                )
            })
        })
        .map_err(crate::captured_batch::error)?;
    Ok(manifest)
}
