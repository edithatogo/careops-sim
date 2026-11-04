//! ED statistical records, native analysis and explicit trajectory interpolation.
//!
//! Native reductions use pinned Polars; checked scalar math preserves minimal
//! CPU execution. Original-run capture supplies native patient tables without
//! reruns. Caller-supplied transit interpolation is separate from observation.

#[cfg(feature = "native-data")]
#[path = "native_tables.rs"]
pub mod native_tables;

use serde::{Deserialize, Serialize};

use crate::disposition::api_runner::{ExitCodeDef, ValidationError};

/// Position measurement kind distinguishing physical/arrival confirmation,
/// simulated discrete event dispatch, and continuous in-transit trajectory interpolation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PositionKind {
    /// Confirmed location arrival or physical presence in a bay/unit.
    Observed,
    /// Discrete-event simulated waypoint or dispatch target.
    Simulated,
    /// Continuous in-transit position interpolated along a physical route between departure and arrival.
    Interpolated,
}

/// Statistical uncertainty estimates (standard error, variance, 95% confidence interval).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UncertaintyBounds {
    pub estimate: Option<f64>,
    pub variance: Option<f64>,
    pub standard_error: Option<f64>,
    pub ci_95_lower: Option<f64>,
    pub ci_95_upper: Option<f64>,
    pub replications_count: usize,
    pub ddof: Option<u8>,
    pub method: String,
    pub status: String,
}

/// Summary metric record with explicit units, denominators, and uncertainty.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MetricRecord {
    pub name: String,
    pub value: Option<f64>,
    pub unit: String,
    pub denominator_name: String,
    pub denominator_value: Option<f64>,
    pub numerator_count: Option<u64>,
    pub denominator_count: Option<u64>,
    pub estimand: String,
    pub sample_unit: String,
    pub weighting: String,
    pub eligible_replications: usize,
    pub included_replications: usize,
    pub total_replications: usize,
    pub uncertainty: UncertaintyBounds,
    pub censored_count: u64,
    pub unfinished_count: u64,
}

/// Movement/transit record representing a patient or agent traversing a spatial route.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LocationTransitRecord {
    pub patient_id: String,
    pub replication_index: usize,
    pub step_index: usize,
    pub from_location: String,
    pub to_location: String,
    pub distance_metres: f64,
    pub departure_tick: u64,
    pub arrival_tick: u64,
    pub transit_ticks: u64,
    pub position_kind: PositionKind,
    pub spatial_revision: String,
}

/// Interpolated patient trajectory position at a specific query tick.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrajectoryPosition {
    pub patient_id: String,
    pub query_tick: u64,
    pub current_location: String,
    pub destination_location: Option<String>,
    pub transit_progress: f64,
    pub position_kind: PositionKind,
    pub spatial_revision: String,
}

fn invalid_statistic(message: impl Into<String>) -> ValidationError {
    ValidationError::new(ExitCodeDef::MalformedConfig, message)
}

fn undefined_uncertainty(
    estimate: Option<f64>,
    replications: usize,
    status: &str,
) -> UncertaintyBounds {
    UncertaintyBounds {
        estimate,
        variance: None,
        standard_error: None,
        ci_95_lower: None,
        ci_95_upper: None,
        replications_count: replications,
        ddof: None,
        method: "unqualified".into(),
        status: status.into(),
    }
}

/// Sample uncertainty, explicitly undefined for fewer than two observations.
/// Native data uses pinned Polars reductions. The small non-native branch
/// preserves minimal CPU execution without linking a tabular runtime.
/// Neither branch may publish nonfinite inputs, derived values or intervals.
pub fn compute_uncertainty(values: &[f64]) -> Result<UncertaintyBounds, ValidationError> {
    if values.iter().any(|v| !v.is_finite()) {
        return Err(invalid_statistic("nonfinite statistical input"));
    }
    if values.is_empty() {
        return Ok(undefined_uncertainty(None, 0, "no_samples"));
    }
    #[cfg(feature = "native-data")]
    let (mean, variance) = {
        use polars::prelude::*;
        if values.len() > u32::MAX as usize {
            return Err(invalid_statistic("native row-index capacity exceeded"));
        }
        let frame = DataFrame::new(
            values.len(),
            vec![Series::new("value".into(), values).into()],
        )
        .map_err(|e| invalid_statistic(e.to_string()))?
        .lazy()
        .select([
            col("value").mean().alias("mean"),
            col("value").var(1).alias("variance"),
        ])
        .collect()
        .map_err(|e| invalid_statistic(e.to_string()))?;
        let number = |name: &str| -> Result<Option<f64>, ValidationError> {
            Ok(frame
                .column(name)
                .map_err(|e| invalid_statistic(e.to_string()))?
                .f64()
                .map_err(|e| invalid_statistic(e.to_string()))?
                .get(0))
        };
        (
            number("mean")?.ok_or_else(|| invalid_statistic("missing sample mean"))?,
            number("variance")?,
        )
    };
    #[cfg(not(feature = "native-data"))]
    let (mean, variance) = {
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = (values.len() > 1).then(|| {
            values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64
        });
        (mean, variance)
    };
    if !mean.is_finite() || variance.is_some_and(|v| !v.is_finite() || v < 0.0) {
        return Err(invalid_statistic(
            "nonfinite or invalid derived statistical result",
        ));
    }
    let Some(variance) = variance else {
        let mut result =
            undefined_uncertainty(Some(mean), values.len(), "insufficient_replications");
        result.ddof = Some(1);
        result.method = "normal_approximation_95".into();
        return Ok(result);
    };
    let se = (variance / values.len() as f64).sqrt();
    let margin = 1.959964 * se;
    let lower = mean - margin;
    let upper = mean + margin;
    if [se, lower, upper].iter().any(|v| !v.is_finite()) {
        return Err(invalid_statistic("nonfinite uncertainty interval"));
    }
    Ok(UncertaintyBounds {
        estimate: Some(mean),
        variance: Some(variance),
        standard_error: Some(se),
        ci_95_lower: Some(lower),
        ci_95_upper: Some(upper),
        replications_count: values.len(),
        ddof: Some(1),
        method: "normal_approximation_95".into(),
        status: "defined".into(),
    })
}

/// Interpolates a patient's position at a target query tick given their transit sequence.
pub fn interpolate_patient_position(
    records: &[LocationTransitRecord],
    query_tick: u64,
) -> Option<TrajectoryPosition> {
    if records.is_empty() {
        return None;
    }

    // Filter to this patient's records and sort by departure tick
    let mut patient_transits: Vec<&LocationTransitRecord> = records.iter().collect();
    patient_transits.sort_by_key(|r| (r.departure_tick, r.step_index));

    let first = patient_transits.first()?;
    let spatial_rev = first.spatial_revision.clone();
    let pat_id = first.patient_id.clone();

    // Before first departure
    if query_tick < first.departure_tick {
        return Some(TrajectoryPosition {
            patient_id: pat_id,
            query_tick,
            current_location: first.from_location.clone(),
            destination_location: None,
            transit_progress: 0.0,
            position_kind: PositionKind::Observed,
            spatial_revision: spatial_rev,
        });
    }

    // Check each transit segment
    for record in &patient_transits {
        if query_tick >= record.departure_tick && query_tick <= record.arrival_tick {
            let duration = record.arrival_tick.saturating_sub(record.departure_tick);
            let progress = if duration > 0 {
                (query_tick - record.departure_tick) as f64 / duration as f64
            } else {
                1.0
            };

            let kind = if query_tick == record.departure_tick || query_tick == record.arrival_tick {
                PositionKind::Observed
            } else {
                PositionKind::Interpolated
            };

            return Some(TrajectoryPosition {
                patient_id: pat_id,
                query_tick,
                current_location: if progress >= 1.0 {
                    record.to_location.clone()
                } else {
                    record.from_location.clone()
                },
                destination_location: if progress < 1.0 {
                    Some(record.to_location.clone())
                } else {
                    None
                },
                transit_progress: progress.clamp(0.0, 1.0),
                position_kind: kind,
                spatial_revision: spatial_rev,
            });
        }
    }

    // After all recorded transits, settled at final destination
    let last = patient_transits.last()?;
    Some(TrajectoryPosition {
        patient_id: pat_id,
        query_tick,
        current_location: last.to_location.clone(),
        destination_location: None,
        transit_progress: 1.0,
        position_kind: PositionKind::Observed,
        spatial_revision: spatial_rev,
    })
}

/// Fixed ED analysis over validated replication summaries. Native reductions
/// use Polars; no patient, event, policy or movement is inferred by statistics.
#[cfg(feature = "native-data")]
pub fn summarize_replications(
    results: &[crate::disposition::batch_runner::ReplicationResult],
    completed_elapsed_ticks: &[u128],
    time_unit: &str,
) -> Result<Vec<MetricRecord>, ValidationError> {
    let counts = crate::disposition::batch_runner::checked_replication_counts(results)?;
    if u64::try_from(completed_elapsed_ticks.len()).ok() != Some(counts[2]) || time_unit.is_empty()
    {
        return Err(invalid_statistic(
            "completed duration count or time unit mismatch",
        ));
    }
    let total = results.len();
    let started_reps = results.iter().filter(|r| r.started > 0).count();
    let arrival_reps = results.iter().filter(|r| r.arrivals > 0).count();
    let completed_reps = results.iter().filter(|r| r.completed > 0).count();
    let wait = if started_reps == 0 {
        undefined_uncertainty(None, total, "no_samples")
    } else {
        compute_uncertainty(
            &results
                .iter()
                .map(|r| r.mean_wait_ticks)
                .collect::<Vec<_>>(),
        )?
    };
    let rate = (counts[0] > 0).then(|| counts[2] as f64 / counts[0] as f64);
    let completion = undefined_uncertainty(
        rate,
        total,
        if rate.is_some() {
            "ratio_method_unqualified"
        } else {
            "no_samples"
        },
    );
    let throughput = compute_uncertainty(
        &results
            .iter()
            .map(|r| r.completed as f64 / r.horizon_ticks as f64)
            .collect::<Vec<_>>(),
    )?;
    let censor = if arrival_reps == 0 {
        undefined_uncertainty(None, total, "no_samples")
    } else {
        compute_uncertainty(
            &results
                .iter()
                .map(|r| {
                    if r.arrivals == 0 {
                        0.0
                    } else {
                        r.unfinished as f64 / r.arrivals as f64
                    }
                })
                .collect::<Vec<_>>(),
        )?
    };
    let elapsed_point = compute_uncertainty(
        &completed_elapsed_ticks
            .iter()
            .map(|v| *v as f64)
            .collect::<Vec<_>>(),
    )?
    .estimate;
    let elapsed = undefined_uncertainty(
        elapsed_point,
        total,
        if elapsed_point.is_some() {
            "cluster_method_unqualified"
        } else {
            "no_samples"
        },
    );
    let included_wait = if started_reps > 0 { total } else { 0 };
    let included_censor = if arrival_reps > 0 { total } else { 0 };
    let definitions = [
        (
            "mean_wait_ticks",
            time_unit,
            "included_replications",
            None,
            Some(included_wait as u64),
            "mean_of_replication_wait_means",
            "replication",
            "equal_replication_zero_for_empty",
            started_reps,
            included_wait,
            wait,
        ),
        (
            "mean_elapsed_ticks",
            time_unit,
            "completed_patients",
            None,
            Some(counts[2]),
            "pooled_completed_patient_mean",
            "completed_patient",
            "equal_completed_patient",
            completed_reps,
            completed_reps,
            elapsed,
        ),
        (
            "completion_rate",
            "ratio",
            "total_arrivals",
            Some(counts[2]),
            Some(counts[0]),
            "pooled_completed_over_arrivals",
            "arrival",
            "pooled_counts",
            arrival_reps,
            arrival_reps,
            completion,
        ),
        (
            "throughput_per_horizon_tick",
            "patients_per_tick",
            "per_replication_horizon_ticks",
            None,
            None,
            "mean_of_replication_throughputs",
            "replication",
            "equal_replication",
            total,
            total,
            throughput,
        ),
        (
            "censored_presentation_rate",
            "ratio",
            "included_replications",
            None,
            Some(included_censor as u64),
            "mean_of_replication_unfinished_rates",
            "replication",
            "equal_replication_zero_for_empty",
            arrival_reps,
            included_censor,
            censor,
        ),
    ];
    Ok(definitions
        .into_iter()
        .map(
            |(
                name,
                unit,
                denominator_name,
                numerator_count,
                denominator_count,
                estimand,
                sample_unit,
                weighting,
                eligible,
                included,
                uncertainty,
            )| MetricRecord {
                name: name.into(),
                value: uncertainty.estimate,
                unit: unit.into(),
                denominator_name: denominator_name.into(),
                denominator_value: denominator_count.map(|v| v as f64),
                numerator_count,
                denominator_count,
                estimand: estimand.into(),
                sample_unit: sample_unit.into(),
                weighting: weighting.into(),
                eligible_replications: eligible,
                included_replications: included,
                total_replications: total,
                uncertainty,
                censored_count: counts[3],
                unfinished_count: counts[3],
            },
        )
        .collect())
}
