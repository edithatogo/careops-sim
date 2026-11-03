//! ED statistical records and the temporary alpha export surface.
//!
//! Native reductions use pinned Polars; checked scalar math preserves minimal
//! CPU execution. Original-run capture supplies patient results without reruns.
//! Movement is explicitly not captured. Caller-supplied transit interpolation is
//! separate from observation. R13 replaces descriptors/JSON/manual CSV with
//! qualified native file outputs and migrates their reachable consumers.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::disposition::api_runner::{ExitCodeDef, ValidationError};
use crate::disposition::batch_runner::BatchRunDocument;
#[cfg(feature = "native-data")]
use crate::PatientStatus;
use crate::ScenarioConfig;

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

/// Execution and scenario provenance for complete reproducibility.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunProvenance {
    pub scenario_id: String,
    pub schema_version: u32,
    pub input_sha256: String,
    pub parsed_input_sha256: String,
    pub time_unit: String,
    pub horizon_ticks: u64,
    pub base_seed: u64,
    pub replication_seeds: Vec<u64>,
    pub num_workers: usize,
    pub total_replications: usize,
    pub resumed_from_checkpoint: bool,
    pub checkpoint_epoch_ms: Option<u64>,
    pub spatial_revision: String,
    pub toolchain: String,
    pub completed_epoch_ms: u64,
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

/// Tabular Arrow column field metadata.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArrowField {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub unit: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub denominator: Option<String>,
}

/// Structured Arrow schema definition.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArrowTableSchema {
    pub schema_name: String,
    pub version: String,
    pub fields: Vec<ArrowField>,
    pub metadata: BTreeMap<String, String>,
}

/// Per-patient tabular record matching Arrow IPC / Parquet schemas.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArrowPatientRow {
    pub patient_id: String,
    pub replication_index: usize,
    pub seed: u64,
    pub arrival_tick: u64,
    pub start_tick: Option<u64>,
    pub completion_tick: Option<u64>,
    pub work_ticks: u64,
    pub wait_ticks: Option<u64>,
    pub elapsed_ticks: Option<u64>,
    pub status: String,
    pub censor_status: String,
    pub resource_id: String,
    pub location_id: Option<String>,
    pub spatial_revision: String,
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

/// Spatial capture status. Unknown movement quantities stay null, not zero.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpatialSummary {
    pub spatial_revision: String,
    pub capture_status: String,
    pub observed_rows: usize,
    pub total_transits: Option<usize>,
    pub total_distance_metres: Option<f64>,
    pub mean_transit_duration_ticks: Option<f64>,
    pub location_visit_counts: BTreeMap<String, usize>,
    pub route_traversal_counts: BTreeMap<String, usize>,
    pub route_mean_duration_ticks: BTreeMap<String, f64>,
}

/// Complete export bundle containing Arrow/Parquet schemas, metrics, tabular data,
/// spatial records, and full provenance.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ArrowExportBundle {
    pub schema_version: u32,
    pub provenance: RunProvenance,
    pub patient_schema: ArrowTableSchema,
    pub metrics_schema: ArrowTableSchema,
    pub spatial_schema: ArrowTableSchema,
    pub parquet_schema: String,
    pub metrics: Vec<MetricRecord>,
    pub patient_records: Vec<ArrowPatientRow>,
    pub spatial_records: Vec<LocationTransitRecord>,
    pub spatial_summary: SpatialSummary,
}

/// Constructs the canonical Arrow table schema for patient presentation records.
pub fn build_patient_arrow_schema(scenario_id: &str, time_unit: &str) -> ArrowTableSchema {
    let mut metadata = BTreeMap::new();
    metadata.insert(
        "format".to_string(),
        "kairoecs.ed.patient_table".to_string(),
    );
    metadata.insert("scenario_id".to_string(), scenario_id.to_string());
    metadata.insert("time_unit".to_string(), time_unit.to_string());
    metadata.insert("byte_order".to_string(), "little".to_string());

    ArrowTableSchema {
        schema_name: "careops.ed.patient_records.v2".to_string(),
        version: "2.0".to_string(),
        fields: vec![
            ArrowField {
                name: "patient_id".to_string(),
                data_type: "utf8".to_string(),
                nullable: false,
                unit: "identifier".to_string(),
                denominator: None,
            },
            ArrowField {
                name: "replication_index".to_string(),
                data_type: "uint64".to_string(),
                nullable: false,
                unit: "index".to_string(),
                denominator: None,
            },
            ArrowField {
                name: "seed".to_string(),
                data_type: "uint64".to_string(),
                nullable: false,
                unit: "seed".to_string(),
                denominator: None,
            },
            ArrowField {
                name: "arrival_tick".to_string(),
                data_type: "uint64".to_string(),
                nullable: false,
                unit: time_unit.to_string(),
                denominator: None,
            },
            ArrowField {
                name: "start_tick".to_string(),
                data_type: "uint64".to_string(),
                nullable: true,
                unit: time_unit.to_string(),
                denominator: None,
            },
            ArrowField {
                name: "completion_tick".to_string(),
                data_type: "uint64".to_string(),
                nullable: true,
                unit: time_unit.to_string(),
                denominator: None,
            },
            ArrowField {
                name: "work_ticks".to_string(),
                data_type: "uint64".to_string(),
                nullable: false,
                unit: time_unit.to_string(),
                denominator: None,
            },
            ArrowField {
                name: "wait_ticks".to_string(),
                data_type: "uint64".to_string(),
                nullable: true,
                unit: time_unit.to_string(),
                denominator: Some("started_patients".to_string()),
            },
            ArrowField {
                name: "elapsed_ticks".to_string(),
                data_type: "uint64".to_string(),
                nullable: true,
                unit: time_unit.to_string(),
                denominator: Some("completed_patients".to_string()),
            },
            ArrowField {
                name: "status".to_string(),
                data_type: "utf8".to_string(),
                nullable: false,
                unit: "categorical".to_string(),
                denominator: None,
            },
            ArrowField {
                name: "censor_status".to_string(),
                data_type: "utf8".to_string(),
                nullable: false,
                unit: "categorical".to_string(),
                denominator: None,
            },
            ArrowField {
                name: "resource_id".to_string(),
                data_type: "utf8".to_string(),
                nullable: false,
                unit: "identifier".to_string(),
                denominator: None,
            },
            ArrowField {
                name: "location_id".to_string(),
                data_type: "utf8".to_string(),
                nullable: true,
                unit: "identifier".to_string(),
                denominator: None,
            },
            ArrowField {
                name: "spatial_revision".to_string(),
                data_type: "utf8".to_string(),
                nullable: false,
                unit: "version".to_string(),
                denominator: None,
            },
        ],
        metadata,
    }
}

/// Flat v2 metric-table descriptor; optional estimates/bounds remain nullable.
pub fn build_metrics_arrow_schema() -> ArrowTableSchema {
    let fields = [
        ("name", "utf8", false),
        ("value", "float64", true),
        ("unit", "utf8", false),
        ("denominator_name", "utf8", false),
        ("denominator_value", "float64", true),
        ("numerator_count", "uint64", true),
        ("denominator_count", "uint64", true),
        ("estimand", "utf8", false),
        ("sample_unit", "utf8", false),
        ("weighting", "utf8", false),
        ("eligible_replications", "uint64", false),
        ("included_replications", "uint64", false),
        ("total_replications", "uint64", false),
        ("variance", "float64", true),
        ("standard_error", "float64", true),
        ("ci_95_lower", "float64", true),
        ("ci_95_upper", "float64", true),
        ("replications_count", "uint64", false),
        ("ddof", "uint32", true),
        ("method", "utf8", false),
        ("status", "utf8", false),
        ("censored_count", "uint64", false),
        ("unfinished_count", "uint64", false),
    ]
    .into_iter()
    .map(|(name, data_type, nullable)| ArrowField {
        name: name.into(),
        data_type: data_type.into(),
        nullable,
        unit: "per_metric_unit_and_estimand".into(),
        denominator: None,
    })
    .collect();
    ArrowTableSchema {
        schema_name: "careops.ed.metrics.v2".into(),
        version: "2.0".into(),
        fields,
        metadata: BTreeMap::from([
            ("format".into(), "careops.ed.metrics.v2".into()),
            (
                "uncertainty".into(),
                "explicit method/status; undefined values are null".into(),
            ),
        ]),
    }
}

/// Constructs the canonical Arrow table schema for spatial transit records.
pub fn build_spatial_arrow_schema() -> ArrowTableSchema {
    let mut metadata = BTreeMap::new();
    metadata.insert(
        "format".to_string(),
        "kairoecs.ed.spatial_transit".to_string(),
    );
    metadata.insert("byte_order".to_string(), "little".to_string());

    ArrowTableSchema {
        schema_name: "careops.ed.spatial_transit.v2".to_string(),
        version: "2.0".to_string(),
        fields: vec![
            ArrowField {
                name: "patient_id".to_string(),
                data_type: "utf8".to_string(),
                nullable: false,
                unit: "identifier".to_string(),
                denominator: None,
            },
            ArrowField {
                name: "replication_index".to_string(),
                data_type: "uint64".to_string(),
                nullable: false,
                unit: "index".to_string(),
                denominator: None,
            },
            ArrowField {
                name: "step_index".to_string(),
                data_type: "uint64".to_string(),
                nullable: false,
                unit: "index".to_string(),
                denominator: None,
            },
            ArrowField {
                name: "from_location".to_string(),
                data_type: "utf8".to_string(),
                nullable: false,
                unit: "location_id".to_string(),
                denominator: None,
            },
            ArrowField {
                name: "to_location".to_string(),
                data_type: "utf8".to_string(),
                nullable: false,
                unit: "location_id".to_string(),
                denominator: None,
            },
            ArrowField {
                name: "distance_metres".to_string(),
                data_type: "float64".to_string(),
                nullable: false,
                unit: "m".to_string(),
                denominator: None,
            },
            ArrowField {
                name: "departure_tick".to_string(),
                data_type: "uint64".to_string(),
                nullable: false,
                unit: "ticks".to_string(),
                denominator: None,
            },
            ArrowField {
                name: "arrival_tick".to_string(),
                data_type: "uint64".to_string(),
                nullable: false,
                unit: "ticks".to_string(),
                denominator: None,
            },
            ArrowField {
                name: "transit_ticks".to_string(),
                data_type: "uint64".to_string(),
                nullable: false,
                unit: "ticks".to_string(),
                denominator: None,
            },
            ArrowField {
                name: "position_kind".to_string(),
                data_type: "utf8".to_string(),
                nullable: false,
                unit: "categorical".to_string(),
                denominator: None,
            },
            ArrowField {
                name: "spatial_revision".to_string(),
                data_type: "utf8".to_string(),
                nullable: false,
                unit: "version".to_string(),
                denominator: None,
            },
        ],
        metadata,
    }
}

/// Generates standard Parquet schema DDL.
pub fn generate_parquet_schema_ddl() -> String {
    "message careops_ed_patient_record {\n\
  required binary patient_id (UTF8);\n\
  required int64 replication_index (INTEGER(64,false));\n\
  required int64 seed (INTEGER(64,false));\n\
  required int64 arrival_tick (INTEGER(64,false));\n\
  optional int64 start_tick (INTEGER(64,false));\n\
  optional int64 completion_tick (INTEGER(64,false));\n\
  required int64 work_ticks (INTEGER(64,false));\n\
  optional int64 wait_ticks (INTEGER(64,false));\n\
  optional int64 elapsed_ticks (INTEGER(64,false));\n\
  required binary status (UTF8);\n\
  required binary censor_status (UTF8);\n\
  required binary resource_id (UTF8);\n\
  optional binary location_id (UTF8);\n\
  required binary spatial_revision (UTF8);\n\
}\n\
\n\
message careops_ed_metric_summary {\n\
  required binary name (UTF8);\n\
  optional double value;\n\
  required binary unit (UTF8);\n\
  required binary denominator_name (UTF8);\n\
  optional double denominator_value;\n\
  optional double variance;\n\
  optional double standard_error;\n\
  optional double ci_95_lower;\n\
  optional double ci_95_upper;\n\
  optional int64 numerator_count (INTEGER(64,false));\n\
  optional int64 denominator_count (INTEGER(64,false));\n\
  required binary estimand (UTF8);\n\
  required binary sample_unit (UTF8);\n\
  required binary weighting (UTF8);\n\
  required int64 eligible_replications (INTEGER(64,false));\n\
  required int64 included_replications (INTEGER(64,false));\n\
  required int64 total_replications (INTEGER(64,false));\n\
  optional int32 ddof (INTEGER(32,false));\n\
  required binary method (UTF8);\n\
  required binary status (UTF8);\n\
  required int64 replications_count (INTEGER(64,false));\n\
  required int64 censored_count (INTEGER(64,false));\n\
  required int64 unfinished_count (INTEGER(64,false));\n\
}\n\
\n\
message careops_ed_spatial_transit {\n\
  required binary patient_id (UTF8);\n\
  required int64 replication_index (INTEGER(64,false));\n\
  required int64 step_index (INTEGER(64,false));\n\
  required binary from_location (UTF8);\n\
  required binary to_location (UTF8);\n\
  required double distance_metres;\n\
  required int64 departure_tick (INTEGER(64,false));\n\
  required int64 arrival_tick (INTEGER(64,false));\n\
  required int64 transit_ticks (INTEGER(64,false));\n\
  required binary position_kind (UTF8);\n\
  required binary spatial_revision (UTF8);\n\
}"
    .to_string()
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

/// Generates complete Arrow/Parquet export bundle with explicit denominators, uncertainty,
/// spatial revision and provenance. Movement is explicitly not captured.
#[cfg(feature = "native-data")]
pub fn generate_arrow_export_bundle(
    scenario: &ScenarioConfig,
    batch_doc: &BatchRunDocument,
    spatial_revision_override: Option<&str>,
    resumed_info: Option<(bool, Option<u64>)>,
) -> Result<ArrowExportBundle, ValidationError> {
    let (capture_store, captured, capture_meta) =
        crate::captured_batch::export_source(scenario, batch_doc)?;
    let spatial_revision = spatial_revision_override
        .unwrap_or("not_captured")
        .to_string();

    let (resumed, chk_epoch) = resumed_info.unwrap_or((false, None));

    // Compile provenance
    let replication_seeds: Vec<u64> = batch_doc.replications.iter().map(|r| r.seed).collect();
    let provenance = RunProvenance {
        scenario_id: batch_doc.scenario_id.clone(),
        schema_version: 2,
        input_sha256: capture_meta.effective_input_sha256.clone(),
        parsed_input_sha256: capture_meta.parsed_input_sha256.clone(),
        time_unit: scenario.time_unit.clone(),
        horizon_ticks: batch_doc
            .replications
            .first()
            .map_or(scenario.horizon_ticks, |r| r.horizon_ticks),
        base_seed: capture_meta.config.base_seed,
        replication_seeds,
        num_workers: batch_doc.num_workers,
        total_replications: batch_doc.total_replications,
        resumed_from_checkpoint: resumed,
        checkpoint_epoch_ms: chk_epoch,
        spatial_revision: spatial_revision.clone(),
        toolchain: env!("CAREOPS_ED_BUILD_RUSTC").to_string(),
        completed_epoch_ms: batch_doc.completed_at_epoch_ms,
    };

    // Build resource map: resource_id -> location_id
    let mut resource_locations = BTreeMap::new();
    for rb in &scenario.capacity_location.resource_buckets {
        resource_locations.insert(rb.resource_id.clone(), rb.location_id.clone());
    }

    // Generate patient rows and spatial transit records across replications
    let mut patient_records = Vec::new();

    for rep in &batch_doc.replications {
        #[cfg(feature = "native-data")]
        let patient_rows = {
            use crate::capture::PartStore;
            let capture = captured
                .iter()
                .find(|r| r.summary.replication_index == rep.replication_index)
                .ok_or_else(|| {
                    ValidationError::new(
                        ExitCodeDef::MalformedConfig,
                        "missing captured replication",
                    )
                })?;
            let mut rows = Vec::new();
            for part in &capture.parts {
                let bytes = capture_store
                    .read_part(&part.name, capture_meta.limits.max_bytes)
                    .map_err(crate::captured_batch::error)?;
                rows.extend(
                    capture_store
                        .decode_part(&bytes)
                        .map_err(crate::captured_batch::error)?
                        .rows,
                );
            }
            rows
        };
        #[cfg(not(feature = "native-data"))]
        let patient_rows: Vec<crate::PatientResult> = Vec::new();
        for pat in &patient_rows {
            let res_id = scenario
                .patients
                .iter()
                .find(|w| w.patient_id == pat.patient_id)
                .map(|w| w.resource_id.clone())
                .ok_or_else(|| {
                    invalid_statistic("captured patient has no configuration reference")
                })?;

            let loc_id = resource_locations
                .get(&res_id)
                .cloned()
                .filter(|location| !location.is_empty());

            let status_str = match pat.status {
                PatientStatus::Completed => "completed".to_string(),
                PatientStatus::Unfinished => "unfinished".to_string(),
            };

            let censor_str = if pat.status == PatientStatus::Unfinished {
                "right_censored_at_horizon".to_string()
            } else {
                "uncensored".to_string()
            };

            patient_records.push(ArrowPatientRow {
                patient_id: pat.patient_id.clone(),
                replication_index: rep.replication_index,
                seed: rep.seed,
                arrival_tick: pat.arrival_tick,
                start_tick: pat.start_tick,
                completion_tick: pat.completion_tick,
                work_ticks: pat.work_ticks,
                wait_ticks: pat.wait_ticks,
                elapsed_ticks: pat.elapsed_ticks,
                status: status_str,
                censor_status: censor_str,
                resource_id: res_id.clone(),
                location_id: loc_id.clone(),
                spatial_revision: spatial_revision.clone(),
            });
        }
    }

    // Deterministic sorting of tabular rows
    patient_records.sort_by_key(|r| (r.replication_index, r.arrival_tick, r.patient_id.clone()));
    let spatial_records = Vec::new();
    let spatial_summary = SpatialSummary {
        spatial_revision: spatial_revision.clone(),
        capture_status: "not_captured".into(),
        observed_rows: 0,
        total_transits: None,
        total_distance_metres: None,
        mean_transit_duration_ticks: None,
        location_visit_counts: BTreeMap::new(),
        route_traversal_counts: BTreeMap::new(),
        route_mean_duration_ticks: BTreeMap::new(),
    };
    let elapsed: Vec<_> = patient_records
        .iter()
        .filter_map(|r| r.elapsed_ticks.map(u128::from))
        .collect();
    let metrics = summarize_replications(&batch_doc.replications, &elapsed, &scenario.time_unit)?;

    let patient_schema = build_patient_arrow_schema(&scenario.scenario_id, &scenario.time_unit);
    let metrics_schema = build_metrics_arrow_schema();
    let spatial_schema = build_spatial_arrow_schema();
    let parquet_schema = generate_parquet_schema_ddl();

    Ok(ArrowExportBundle {
        schema_version: 2,
        provenance,
        patient_schema,
        metrics_schema,
        spatial_schema,
        parquet_schema,
        metrics,
        patient_records,
        spatial_records,
        spatial_summary,
    })
}

/// Serializes the export bundle to pretty-printed JSON.
pub fn export_bundle_to_json(bundle: &ArrowExportBundle) -> Result<String, ValidationError> {
    serde_json::to_string_pretty(bundle)
        .map_err(|e| ValidationError::new(ExitCodeDef::MalformedConfig, e.to_string()))
}

/// Exports spatial route traversal summaries to CSV format.
pub fn export_spatial_summary_to_csv(summary: &SpatialSummary) -> String {
    let mut out = String::from("route,traversals,mean_duration_ticks,spatial_revision\n");
    for (route, count) in &summary.route_traversal_counts {
        let mean = summary
            .route_mean_duration_ticks
            .get(route)
            .copied()
            .unwrap_or(0.0);
        out.push_str(&format!(
            "\"{route}\",{count},{mean:.2},\"{}\"\n",
            summary.spatial_revision
        ));
    }
    out
}

#[cfg(not(feature = "native-data"))]
pub fn generate_arrow_export_bundle(
    _scenario: &ScenarioConfig,
    _batch_doc: &BatchRunDocument,
    _spatial_revision_override: Option<&str>,
    _resumed_info: Option<(bool, Option<u64>)>,
) -> Result<ArrowExportBundle, ValidationError> {
    Err(ValidationError::new(
        ExitCodeDef::MalformedConfig,
        "native-data capture required for export",
    ))
}
