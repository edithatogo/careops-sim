//! Native synthetic fixture projection, distinct from canonical empirical traces.
//! One invocation belongs to the caller-declared fixture identity. It neither
//! establishes input-hash provenance nor qualifies statistical/clinical fit.
use super::{CalibrationTraceRecord, CalibrationTransitSource};
use crate::disposition::api_runner::{ExitCodeDef, ValidationError};
use polars::prelude::*;
fn invalid(e: impl std::fmt::Display) -> ValidationError {
    ValidationError::new(ExitCodeDef::MalformedConfig, e.to_string())
}
/// Preserve independently absent endpoints. Route-derived seconds require an
/// explicit one-second tick mapping. The source cohort is synthetic fixture
/// records; this is not a declaration of measured movement or EHR observations.
pub fn calibration_table(
    records: &[CalibrationTraceRecord],
    fixture_id: &str,
    time_unit: &str,
) -> Result<DataFrame, ValidationError> {
    if records.len() > u32::MAX as usize
        || fixture_id.trim().is_empty()
        || fixture_id.len() > 256
        || time_unit.trim().is_empty()
        || time_unit.len() > 64
    {
        return Err(invalid(
            "invalid fixture identity, time unit or native row capacity",
        ));
    }
    let mut elapsed = Vec::with_capacity(records.len());
    for record in records {
        if record.case_id.is_empty() {
            return Err(invalid("empty case key"));
        }
        match (record.transit_source, record.transit_source_seconds) {
            (CalibrationTransitSource::RouteModel, Some(seconds)) => {
                if time_unit != "second"
                    || !seconds.is_finite()
                    || seconds < 0.0
                    || seconds >= u64::MAX as f64
                    || seconds.floor() as u64 != record.transit_ticks
                {
                    return Err(invalid("route seconds mapping is invalid or unqualified"));
                }
            }
            (CalibrationTransitSource::CallerSupplied, None) => {}
            (CalibrationTransitSource::MacroModel, None) if record.transit_ticks == 0 => {}
            _ => return Err(invalid("inconsistent transit provenance")),
        }
        elapsed.push(match (record.start_tick, record.end_tick) {
            (Some(start), Some(end)) => Some(
                kairo_ecs_arrow::polars_interop::subtract_tick_limbs(0, end, 0, start)
                    .map_err(invalid)?,
            ),
            _ => None,
        });
    }
    let n = records.len();
    let mut columns: Vec<Column> = vec![
        Series::new(
            "schema".into(),
            vec!["careops.ed.synthetic-fixture-calibration.v2"; n],
        )
        .into(),
        Series::new("fixture_id".into(), vec![fixture_id; n]).into(),
        Series::new("source_kind".into(), vec!["synthetic_fixture"; n]).into(),
        Series::new("time_unit".into(), vec![time_unit; n]).into(),
        Series::new("interval_basis".into(), vec!["end_minus_start"; n]).into(),
        Series::new(
            "case_id".into(),
            records
                .iter()
                .map(|r| r.case_id.as_str())
                .collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "day".into(),
            records.iter().map(|r| r.day).collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "transit_source".into(),
            records
                .iter()
                .map(|r| r.transit_source.as_str())
                .collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "transit_source_seconds".into(),
            records
                .iter()
                .map(|r| r.transit_source_seconds)
                .collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "transit_conversion".into(),
            records
                .iter()
                .map(|r| {
                    if r.transit_source == CalibrationTransitSource::RouteModel {
                        "floor_seconds"
                    } else {
                        "not_applicable"
                    }
                })
                .collect::<Vec<_>>(),
        )
        .into(),
    ];
    for (name, values) in [
        (
            "start_tick",
            records.iter().map(|r| r.start_tick).collect::<Vec<_>>(),
        ),
        ("end_tick", records.iter().map(|r| r.end_tick).collect()),
        (
            "transit_ticks",
            records.iter().map(|r| Some(r.transit_ticks)).collect(),
        ),
        (
            "work_ticks",
            records.iter().map(|r| Some(r.work_ticks)).collect(),
        ),
    ] {
        columns.push(
            Series::new(
                format!("{name}_hi").into(),
                values.iter().map(|v| v.map(|_| 0u64)).collect::<Vec<_>>(),
            )
            .into(),
        );
        columns.push(Series::new(format!("{name}_lo").into(), values).into());
    }
    columns.push(
        Series::new(
            "interval_ticks_hi".into(),
            elapsed
                .iter()
                .map(|v| v.map(|(hi, _)| hi))
                .collect::<Vec<_>>(),
        )
        .into(),
    );
    columns.push(
        Series::new(
            "interval_ticks_lo".into(),
            elapsed
                .iter()
                .map(|v| v.map(|(_, lo)| lo))
                .collect::<Vec<_>>(),
        )
        .into(),
    );
    let frame = DataFrame::new(n, columns).map_err(invalid)?;
    let keys = frame
        .clone()
        .lazy()
        .group_by([col("fixture_id"), col("day"), col("case_id")])
        .agg([len().alias("rows")])
        .collect()
        .map_err(invalid)?;
    if keys.height() != n {
        return Err(invalid("duplicate fixture/day/case key"));
    }
    frame
        .lazy()
        .sort(
            ["fixture_id", "day", "case_id"],
            SortMultipleOptions::default(),
        )
        .collect()
        .map_err(invalid)
}
