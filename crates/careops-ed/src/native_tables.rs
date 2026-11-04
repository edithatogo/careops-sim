//! Offline ED tables. Configuration joins describe associations, not movement.
//! Capture validation remains authoritative for original-run and durable binding.
use std::io::Cursor;

use polars::prelude::*;

use crate::capture::{CaptureBatch, PartStore};
use crate::disposition::api_runner::{ExitCodeDef, ValidationError};
use crate::disposition::batch_runner::BatchRunDocument;
use crate::ScenarioConfig;

fn invalid(e: impl std::fmt::Display) -> ValidationError {
    ValidationError::new(ExitCodeDef::MalformedConfig, e.to_string())
}

fn unique_keys(frame: &DataFrame, keys: &[&str]) -> Result<(), ValidationError> {
    if frame.height() > u32::MAX as usize {
        return Err(invalid("native row index capacity exceeded"));
    }
    for key in keys {
        let column = frame.column(key).map_err(invalid)?;
        if column.null_count() != 0 {
            return Err(invalid(format!("null key: {key}")));
        }
    }
    let groups = frame
        .clone()
        .lazy()
        .group_by(keys.iter().map(|key| col(*key)).collect::<Vec<_>>())
        .agg([len().alias("_key_rows")])
        .collect()
        .map_err(invalid)?;
    if groups.height() != frame.height() {
        return Err(invalid("duplicate table key"));
    }
    Ok(())
}

/// Join an ED fact table with scenario-qualified configuration dimensions.
/// Required keys are non-null and unique; a left join may neither lose nor
/// multiply observations. Patient IDs may recur in distinct replications.
pub fn associate_patients(
    facts: DataFrame,
    patients: DataFrame,
    resources: DataFrame,
) -> Result<DataFrame, ValidationError> {
    unique_keys(&facts, &["run_id", "replication_index", "patient_id"])?;
    unique_keys(&facts, &["run_id", "replication_index", "sequence"])?;
    let sequences = facts
        .clone()
        .lazy()
        .group_by([col("run_id"), col("replication_index")])
        .agg([
            col("sequence").min().alias("first"),
            col("sequence").max().alias("last"),
            len().cast(DataType::UInt64).alias("rows"),
        ])
        .collect()
        .map_err(invalid)?;
    let first = sequences
        .column("first")
        .and_then(|c| c.u64())
        .map_err(invalid)?;
    let last = sequences
        .column("last")
        .and_then(|c| c.u64())
        .map_err(invalid)?;
    let rows = sequences
        .column("rows")
        .and_then(|c| c.u64())
        .map_err(invalid)?;
    for i in 0..sequences.height() {
        if first.get(i) != Some(0) || last.get(i).and_then(|v| v.checked_add(1)) != rows.get(i) {
            return Err(invalid("noncontiguous captured sequence"));
        }
    }
    unique_keys(&patients, &["scenario_id", "patient_id"])?;
    unique_keys(&resources, &["scenario_id", "resource_id"])?;
    // Explicit boundary schemas prevent Polars suffixes from hiding a supplied
    // resource/location column, and presence sentinels never escape this query.
    if patients.width() != 3 || resources.width() != 3 {
        return Err(invalid("unexpected dimension columns"));
    }
    for (dimension, names) in [
        (&patients, ["scenario_id", "patient_id", "resource_id"]),
        (&resources, ["scenario_id", "resource_id", "location_id"]),
    ] {
        for name in names {
            if dimension.column(name).map_err(invalid)?.dtype() != &DataType::String {
                return Err(invalid("dimension column must be String"));
            }
        }
    }
    for name in [
        "resource_id",
        "location_id",
        "_patient_present",
        "_resource_present",
    ] {
        if facts.column(name).is_ok() {
            return Err(invalid("fact contains reserved association column"));
        }
    }
    if patients
        .column("resource_id")
        .map_err(invalid)?
        .null_count()
        != 0
    {
        return Err(invalid("null patient resource reference"));
    }
    let rows = facts.height();
    let joined = facts
        .lazy()
        .join(
            patients
                .lazy()
                .with_column(lit(true).alias("_patient_present")),
            [col("scenario_id"), col("patient_id")],
            [col("scenario_id"), col("patient_id")],
            JoinArgs::new(JoinType::Left),
        )
        .map_err(invalid)?
        .join(
            resources
                .lazy()
                .with_column(lit(true).alias("_resource_present")),
            [col("scenario_id"), col("resource_id")],
            [col("scenario_id"), col("resource_id")],
            JoinArgs::new(JoinType::Left),
        )
        .map_err(invalid)?
        .sort(
            ["run_id", "replication_index", "sequence"],
            SortMultipleOptions::default(),
        )
        .collect()
        .map_err(invalid)?;
    if joined.height() != rows
        || joined
            .column("_patient_present")
            .map_err(invalid)?
            .null_count()
            != 0
        || joined
            .column("_resource_present")
            .map_err(invalid)?
            .null_count()
            != 0
    {
        return Err(invalid(
            "missing configuration reference or changed join cardinality",
        ));
    }
    Ok(joined.drop_many(["_patient_present", "_resource_present"]))
}

/// Read actual captured Parquet, retaining limb and null columns. No simulation
/// invocation, patient reconstruction or event reordering occurs at this boundary.
pub fn patient_table(
    scenario: &ScenarioConfig,
    document: &BatchRunDocument,
) -> Result<DataFrame, ValidationError> {
    patient_table_from_validated_capture(
        scenario,
        document,
        crate::captured_batch::export_source(scenario, document)?,
    )
}

// Reuse the same validation receipt within one export, rather than rereading and
// decoding every capture part twice. Only the checked entry points call this.
pub(crate) fn patient_table_from_validated_capture(
    scenario: &ScenarioConfig,
    document: &BatchRunDocument,
    source: (
        crate::parquet_capture::ParquetPartStore,
        Vec<crate::capture::CapturedReplication>,
        crate::capture::CaptureMetadata,
    ),
) -> Result<DataFrame, ValidationError> {
    let (store, captures, metadata) = source;
    let empty = CaptureBatch {
        schema: metadata.capture_schema,
        run_id: document.batch_id.clone(),
        input_sha256: metadata.effective_input_sha256,
        model_contract_sha256: metadata.model_contract_sha256,
        time_unit: scenario.time_unit.clone(),
        replication_index: 0,
        seed: 0,
        first_sequence: 0,
        rows: Vec::new(),
    };
    let mut facts = store
        .dataframe(&empty)
        .map_err(crate::captured_batch::error)?;
    for capture in captures {
        for part in capture.parts {
            let bytes = store
                .read_part(&part.name, metadata.limits.max_bytes)
                .map_err(crate::captured_batch::error)?;
            if crate::disposition::api_runner::compute_sha256_hex(&bytes) != part.sha256 {
                return Err(invalid("capture changed after validation"));
            }
            let frame = kairo_ecs_arrow::polars_interop::read_parquet(Cursor::new(bytes))
                .map_err(invalid)?;
            if facts
                .height()
                .checked_add(frame.height())
                .is_none_or(|n| n > u32::MAX as usize)
            {
                return Err(invalid("native row index capacity exceeded"));
            }
            facts.vstack_mut(&frame).map_err(invalid)?;
        }
    }
    facts = facts
        .lazy()
        .with_column(lit(scenario.scenario_id.as_str()).alias("scenario_id"))
        .collect()
        .map_err(invalid)?;
    let patients = df!(
        "scenario_id" => vec![scenario.scenario_id.as_str(); scenario.patients.len()],
        "patient_id" => scenario.patients.iter().map(|p| p.patient_id.as_str()).collect::<Vec<_>>(),
        "resource_id" => scenario.patients.iter().map(|p| p.resource_id.as_str()).collect::<Vec<_>>()
    ).map_err(invalid)?;
    let resources = &scenario.capacity_location.resource_buckets;
    let resources = df!(
        "scenario_id" => vec![scenario.scenario_id.as_str(); resources.len()],
        "resource_id" => resources.iter().map(|r| r.resource_id.as_str()).collect::<Vec<_>>(),
        "location_id" => resources.iter().map(|r| r.location_id.as_str()).collect::<Vec<_>>()
    )
    .map_err(invalid)?;
    let facts = associate_patients(facts, patients, resources)?;
    let grouped = patient_replication_summary(facts.clone())?;
    for rep in &document.replications {
        let group = grouped
            .clone()
            .lazy()
            .filter(col("replication_index").eq(lit(rep.replication_index as u64)))
            .collect()
            .map_err(invalid)?;
        if rep.arrivals == 0 && group.height() == 0 {
            continue;
        }
        if group.height() != 1 {
            return Err(invalid("missing captured replication group"));
        }
        for (name, expected) in [
            ("arrivals", rep.arrivals),
            ("started", rep.started),
            ("completed", rep.completed),
        ] {
            if group
                .column(name)
                .and_then(|c| c.u64())
                .map_err(invalid)?
                .get(0)
                != Some(expected as u64)
            {
                return Err(invalid("captured grouped count mismatch"));
            }
        }
        let mean = group
            .column("mean_wait_ticks")
            .and_then(|c| c.f64())
            .map_err(invalid)?
            .get(0)
            .unwrap_or(0.0);
        if !mean.is_finite()
            || (mean - rep.mean_wait_ticks).abs() > 1e-12 + 1e-10 * rep.mean_wait_ticks.abs()
        {
            return Err(invalid("captured grouped wait mean mismatch"));
        }
    }
    facts
        .lazy()
        .with_columns([
            col("schema").alias("capture_table_schema"),
            lit("careops.ed.patients.v2").alias("schema"),
        ])
        .collect()
        .map_err(invalid)
}

/// Captured patient aggregates, including censored/never-started rows. Empty
/// replications have no patient fact group; their horizons/seeds remain in the
/// authoritative batch control rather than synthetic patient rows.
pub fn patient_replication_summary(mut patients: DataFrame) -> Result<DataFrame, ValidationError> {
    unique_keys(&patients, &["run_id", "replication_index", "patient_id"])?;
    unique_keys(&patients, &["run_id", "replication_index", "sequence"])?;
    // The capture contract records endpoints as checked limbs. Subtract before
    // floating conversion, retaining raw limbs and nulls in the source table.
    let mut waits = Vec::with_capacity(patients.height());
    for i in 0..patients.height() {
        let pair = |name: &str| -> Result<Option<(u64, u64)>, ValidationError> {
            let hi = patients
                .column(&format!("{name}_hi"))
                .and_then(|c| c.u64())
                .map_err(invalid)?
                .get(i);
            let lo = patients
                .column(&format!("{name}_lo"))
                .and_then(|c| c.u64())
                .map_err(invalid)?
                .get(i);
            match (hi, lo) {
                (Some(hi), Some(lo)) => Ok(Some((hi, lo))),
                (None, None) => Ok(None),
                _ => Err(invalid("half-null tick pair")),
            }
        };
        let arrival = pair("arrival_tick")?.ok_or_else(|| invalid("null arrival"))?;
        let start = pair("start_tick")?;
        let completion = pair("completion_tick")?;
        let elapsed = pair("elapsed_ticks")?;
        match completion {
            Some(end) => {
                let start = start.ok_or_else(|| invalid("completion without start"))?;
                kairo_ecs_arrow::polars_interop::subtract_tick_limbs(
                    end.0, end.1, start.0, start.1,
                )
                .map_err(invalid)?;
                let difference = kairo_ecs_arrow::polars_interop::subtract_tick_limbs(
                    end.0, end.1, arrival.0, arrival.1,
                )
                .map_err(invalid)?;
                if elapsed != Some(difference) {
                    return Err(invalid("captured elapsed differs from endpoints"));
                }
            }
            None => {
                if elapsed.is_some() {
                    return Err(invalid("elapsed without completion"));
                }
            }
        }
        let wait = match start {
            Some(start) => {
                let (hi, lo) = kairo_ecs_arrow::polars_interop::subtract_tick_limbs(
                    start.0, start.1, arrival.0, arrival.1,
                )
                .map_err(invalid)?;
                if pair("wait_ticks")? != Some((hi, lo)) {
                    return Err(invalid("captured wait differs from endpoints"));
                }
                Some((((hi as u128) << 64) | (lo as u128)) as f64)
            }
            None => {
                if pair("wait_ticks")?.is_some() {
                    return Err(invalid("wait without start"));
                }
                None
            }
        };
        waits.push(wait);
    }
    patients
        .with_column(Series::new("_analytical_wait".into(), waits).into())
        .map_err(invalid)?;
    patients
        .lazy()
        .group_by([col("run_id"), col("replication_index")])
        .agg([
            len().cast(DataType::UInt64).alias("arrivals"),
            col("start_tick_lo")
                .count()
                .cast(DataType::UInt64)
                .alias("started"),
            col("completion_tick_lo")
                .count()
                .cast(DataType::UInt64)
                .alias("completed"),
            col("_analytical_wait").mean().alias("mean_wait_ticks"),
        ])
        .sort(
            ["run_id", "replication_index"],
            SortMultipleOptions::default(),
        )
        .collect()
        .map_err(invalid)
}
