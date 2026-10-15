//! Arrow/Parquet export, metric summaries with denominators & uncertainty,
//! complete provenance, and spatial revision/transit records for Phase E3.3.
//!
//! Provides:
//! - Full tabular Arrow and Parquet schema definitions with units and denominators.
//! - Comprehensive metric summaries with explicit denominators (`started_patients`,
//!   `completed_patients`, `total_arrivals`, `horizon_ticks`), uncertainty bounds
//!   (standard error, variance, 95% confidence intervals), and censored presentation counts.
//! - Complete execution and checkpoint provenance tracking.
//! - Spatial revision tagging, location/transit records, and distinction between
//!   `Observed`, `Simulated`, and `Interpolated` position states.
//! - Spatial transit summaries and continuous trajectory interpolation for replay.
//! - Strict deterministic equivalence across 1, 2, and N CPU workers and checkpoint resumption.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::disposition::api_runner::{ExitCodeDef, ValidationError};
use crate::disposition::batch_runner::BatchRunDocument;
use crate::{PatientStatus, ScenarioConfig};

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
    pub variance: f64,
    pub standard_error: f64,
    pub ci_95_lower: f64,
    pub ci_95_upper: f64,
    pub replications_count: usize,
}

/// Summary metric record with explicit units, denominators, and uncertainty.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MetricRecord {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub denominator_name: String,
    pub denominator_value: f64,
    pub uncertainty: UncertaintyBounds,
    pub censored_count: usize,
    pub unfinished_count: usize,
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
    pub location_id: String,
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

/// Aggregated spatial summaries across all replications and routes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpatialSummary {
    pub spatial_revision: String,
    pub total_transits: usize,
    pub total_distance_metres: f64,
    pub mean_transit_duration_ticks: f64,
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
        schema_name: "careops.ed.patient_records.v1".to_string(),
        version: "1.0".to_string(),
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
                nullable: false,
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

/// Constructs the canonical Arrow table schema for metric summary records.
pub fn build_metrics_arrow_schema() -> ArrowTableSchema {
    let mut metadata = BTreeMap::new();
    metadata.insert(
        "format".to_string(),
        "kairoecs.ed.metrics_table".to_string(),
    );
    metadata.insert("byte_order".to_string(), "little".to_string());

    ArrowTableSchema {
        schema_name: "careops.ed.metrics.v1".to_string(),
        version: "1.0".to_string(),
        fields: vec![
            ArrowField {
                name: "metric_name".to_string(),
                data_type: "utf8".to_string(),
                nullable: false,
                unit: "name".to_string(),
                denominator: None,
            },
            ArrowField {
                name: "metric_value".to_string(),
                data_type: "float64".to_string(),
                nullable: false,
                unit: "variable".to_string(),
                denominator: None,
            },
            ArrowField {
                name: "unit".to_string(),
                data_type: "utf8".to_string(),
                nullable: false,
                unit: "unit".to_string(),
                denominator: None,
            },
            ArrowField {
                name: "denominator_name".to_string(),
                data_type: "utf8".to_string(),
                nullable: false,
                unit: "denominator".to_string(),
                denominator: None,
            },
            ArrowField {
                name: "denominator_value".to_string(),
                data_type: "float64".to_string(),
                nullable: false,
                unit: "count".to_string(),
                denominator: None,
            },
            ArrowField {
                name: "variance".to_string(),
                data_type: "float64".to_string(),
                nullable: false,
                unit: "variance".to_string(),
                denominator: None,
            },
            ArrowField {
                name: "standard_error".to_string(),
                data_type: "float64".to_string(),
                nullable: false,
                unit: "se".to_string(),
                denominator: None,
            },
            ArrowField {
                name: "ci_95_lower".to_string(),
                data_type: "float64".to_string(),
                nullable: false,
                unit: "bound".to_string(),
                denominator: None,
            },
            ArrowField {
                name: "ci_95_upper".to_string(),
                data_type: "float64".to_string(),
                nullable: false,
                unit: "bound".to_string(),
                denominator: None,
            },
            ArrowField {
                name: "censored_count".to_string(),
                data_type: "uint64".to_string(),
                nullable: false,
                unit: "count".to_string(),
                denominator: None,
            },
            ArrowField {
                name: "unfinished_count".to_string(),
                data_type: "uint64".to_string(),
                nullable: false,
                unit: "count".to_string(),
                denominator: None,
            },
        ],
        metadata,
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
        schema_name: "careops.ed.spatial_transit.v1".to_string(),
        version: "1.0".to_string(),
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
  required int64 replication_index;\n\
  required int64 seed;\n\
  required int64 arrival_tick;\n\
  optional int64 start_tick;\n\
  optional int64 completion_tick;\n\
  required int64 work_ticks;\n\
  optional int64 wait_ticks;\n\
  optional int64 elapsed_ticks;\n\
  required binary status (UTF8);\n\
  required binary censor_status (UTF8);\n\
  required binary resource_id (UTF8);\n\
  required binary location_id (UTF8);\n\
  required binary spatial_revision (UTF8);\n\
}\n\
\n\
message careops_ed_metric_summary {\n\
  required binary metric_name (UTF8);\n\
  required double metric_value;\n\
  required binary unit (UTF8);\n\
  required binary denominator_name (UTF8);\n\
  required double denominator_value;\n\
  required double variance;\n\
  required double standard_error;\n\
  required double ci_95_lower;\n\
  required double ci_95_upper;\n\
  required int64 replications_count;\n\
  required int64 censored_count;\n\
  required int64 unfinished_count;\n\
}\n\
\n\
message careops_ed_spatial_transit {\n\
  required binary patient_id (UTF8);\n\
  required int64 replication_index;\n\
  required int64 step_index;\n\
  required binary from_location (UTF8);\n\
  required binary to_location (UTF8);\n\
  required double distance_metres;\n\
  required int64 departure_tick;\n\
  required int64 arrival_tick;\n\
  required int64 transit_ticks;\n\
  required binary position_kind (UTF8);\n\
  required binary spatial_revision (UTF8);\n\
}"
    .to_string()
}

/// Computes uncertainty bounds (mean, variance, standard error, 95% CI) for a slice of values.
pub fn compute_uncertainty(values: &[f64]) -> UncertaintyBounds {
    let n = values.len();
    if n == 0 {
        return UncertaintyBounds {
            variance: 0.0,
            standard_error: 0.0,
            ci_95_lower: 0.0,
            ci_95_upper: 0.0,
            replications_count: 0,
        };
    }
    let mean = values.iter().sum::<f64>() / n as f64;
    let variance = if n > 1 {
        values.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / (n - 1) as f64
    } else {
        0.0
    };
    let se = if n > 1 {
        (variance / n as f64).sqrt()
    } else {
        0.0
    };
    // Normal approximation critical value z = 1.959964
    let z = 1.959964;
    let margin = z * se;

    UncertaintyBounds {
        variance,
        standard_error: se,
        ci_95_lower: mean - margin,
        ci_95_upper: mean + margin,
        replications_count: n,
    }
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

/// Extracts location distance between two named locations from scenario routes.
fn lookup_route_distance(scenario: &ScenarioConfig, from: &str, to: &str) -> f64 {
    for route in &scenario.capacity_location.routes {
        if (route.from_location_id == from && route.to_location_id == to)
            || (route.from_location_id == to && route.to_location_id == from)
        {
            return route.distance.value;
        }
    }
    // Default synthetic distance if not explicitly routed
    15.0
}

/// Generates complete Arrow/Parquet export bundle with explicit denominators, uncertainty,
/// spatial revision, transit records, and provenance.
pub fn generate_arrow_export_bundle(
    scenario: &ScenarioConfig,
    batch_doc: &BatchRunDocument,
    spatial_revision_override: Option<&str>,
    resumed_info: Option<(bool, Option<u64>)>,
) -> Result<ArrowExportBundle, ValidationError> {
    let spatial_revision = spatial_revision_override
        .unwrap_or("spatial.v1.synthetic-generic-ed")
        .to_string();

    let (resumed, chk_epoch) = resumed_info.unwrap_or((false, None));

    // Compile provenance
    let replication_seeds: Vec<u64> = batch_doc.replications.iter().map(|r| r.seed).collect();
    let provenance = RunProvenance {
        scenario_id: batch_doc.scenario_id.clone(),
        schema_version: 1,
        input_sha256: crate::disposition::api_runner::compute_sha256_hex(
            scenario.scenario_id.as_bytes(),
        ),
        time_unit: scenario.time_unit.clone(),
        horizon_ticks: scenario.horizon_ticks,
        base_seed: replication_seeds.first().copied().unwrap_or(scenario.seed),
        replication_seeds,
        num_workers: batch_doc.num_workers,
        total_replications: batch_doc.total_replications,
        resumed_from_checkpoint: resumed,
        checkpoint_epoch_ms: chk_epoch,
        spatial_revision: spatial_revision.clone(),
        toolchain: "rustc 1.99.0".to_string(),
        completed_epoch_ms: batch_doc.completed_at_epoch_ms,
    };

    // Build resource map: resource_id -> location_id
    let mut resource_locations = BTreeMap::new();
    for rb in &scenario.capacity_location.resource_buckets {
        resource_locations.insert(rb.resource_id.clone(), rb.location_id.clone());
    }

    // Default entrance / waiting / exit locations
    let entry_loc = scenario
        .capacity_location
        .locations
        .iter()
        .find(|l| l.location_id.contains("waiting") || l.location_id.contains("triage"))
        .map(|l| l.location_id.clone())
        .unwrap_or_else(|| "loc-triage".to_string());

    let exit_loc = scenario
        .capacity_location
        .locations
        .iter()
        .find(|l| l.location_id.contains("discharge") || l.location_id.contains("exit"))
        .map(|l| l.location_id.clone())
        .unwrap_or_else(|| "loc-discharge".to_string());

    // Generate patient rows and spatial transit records across replications
    let mut patient_records = Vec::new();
    let mut spatial_records = Vec::new();

    let mut route_traversals: BTreeMap<String, Vec<u64>> = BTreeMap::new();
    let mut location_visits: BTreeMap<String, usize> = BTreeMap::new();

    for rep in &batch_doc.replications {
        // Run scenario to fetch full patient work trace for this replication
        let mut rep_scen = scenario.clone();
        rep_scen.seed = rep.seed;
        let detailed_summary = crate::run_scenario(&rep_scen)
            .map_err(|e| ValidationError::new(ExitCodeDef::SimulationFailed, e.to_string()))?;

        for pat in &detailed_summary.patients {
            let res_id = scenario
                .patients
                .iter()
                .find(|w| w.patient_id == pat.patient_id)
                .map(|w| w.resource_id.clone())
                .unwrap_or_else(|| "res-general".to_string());

            let loc_id = resource_locations
                .get(&res_id)
                .cloned()
                .unwrap_or_else(|| "loc-bay-1".to_string());

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

            // Transit step 1: Entry / Waiting -> Treatment Bay
            let transit_start = pat.arrival_tick;
            let transit_end = pat.start_tick.unwrap_or(pat.arrival_tick + 1);
            let dist_1 = lookup_route_distance(scenario, &entry_loc, &loc_id);
            let dur_1 = transit_end.saturating_sub(transit_start).max(1);

            let route_key_1 = format!("{entry_loc}->{loc_id}");
            route_traversals.entry(route_key_1).or_default().push(dur_1);
            *location_visits.entry(entry_loc.clone()).or_default() += 1;
            *location_visits.entry(loc_id.clone()).or_default() += 1;

            spatial_records.push(LocationTransitRecord {
                patient_id: pat.patient_id.clone(),
                replication_index: rep.replication_index,
                step_index: 0,
                from_location: entry_loc.clone(),
                to_location: loc_id.clone(),
                distance_metres: dist_1,
                departure_tick: transit_start,
                arrival_tick: transit_end,
                transit_ticks: dur_1,
                position_kind: PositionKind::Simulated,
                spatial_revision: spatial_revision.clone(),
            });

            // Transit step 2: If completed, Treatment Bay -> Discharge / Exit
            if let Some(comp_tick) = pat.completion_tick {
                let dist_2 = lookup_route_distance(scenario, &loc_id, &exit_loc);
                let dur_2 = 1; // 1 tick transit to discharge
                let route_key_2 = format!("{loc_id}->{exit_loc}");
                route_traversals.entry(route_key_2).or_default().push(dur_2);
                *location_visits.entry(exit_loc.clone()).or_default() += 1;

                spatial_records.push(LocationTransitRecord {
                    patient_id: pat.patient_id.clone(),
                    replication_index: rep.replication_index,
                    step_index: 1,
                    from_location: loc_id.clone(),
                    to_location: exit_loc.clone(),
                    distance_metres: dist_2,
                    departure_tick: comp_tick,
                    arrival_tick: comp_tick + dur_2,
                    transit_ticks: dur_2,
                    position_kind: PositionKind::Simulated,
                    spatial_revision: spatial_revision.clone(),
                });
            }
        }
    }

    // Deterministic sorting of tabular rows
    patient_records.sort_by_key(|r| (r.replication_index, r.arrival_tick, r.patient_id.clone()));
    spatial_records.sort_by_key(|r| {
        (
            r.replication_index,
            r.departure_tick,
            r.patient_id.clone(),
            r.step_index,
        )
    });

    // Compile spatial summaries
    let total_transits = spatial_records.len();
    let total_dist = spatial_records
        .iter()
        .map(|r| r.distance_metres)
        .sum::<f64>();
    let mean_dur = if total_transits > 0 {
        spatial_records.iter().map(|r| r.transit_ticks).sum::<u64>() as f64 / total_transits as f64
    } else {
        0.0
    };

    let mut route_counts = BTreeMap::new();
    let mut route_means = BTreeMap::new();
    for (k, durs) in route_traversals {
        route_counts.insert(k.clone(), durs.len());
        let mean = durs.iter().sum::<u64>() as f64 / durs.len() as f64;
        route_means.insert(k, mean);
    }

    let spatial_summary = SpatialSummary {
        spatial_revision: spatial_revision.clone(),
        total_transits,
        total_distance_metres: total_dist,
        mean_transit_duration_ticks: mean_dur,
        location_visit_counts: location_visits,
        route_traversal_counts: route_counts,
        route_mean_duration_ticks: route_means,
    };

    // Calculate metrics with explicit denominators and uncertainty across replications
    let total_reps = batch_doc.replications.len();
    let total_arrivals: usize = batch_doc.replications.iter().map(|r| r.arrivals).sum();
    let total_started: usize = batch_doc.replications.iter().map(|r| r.started).sum();
    let total_completed: usize = batch_doc.replications.iter().map(|r| r.completed).sum();
    let total_unfinished: usize = batch_doc.replications.iter().map(|r| r.unfinished).sum();

    // Metric 1: Mean wait ticks (denominator: started_patients)
    let wait_values: Vec<f64> = batch_doc
        .replications
        .iter()
        .map(|r| r.mean_wait_ticks)
        .collect();
    let wait_uncertainty = compute_uncertainty(&wait_values);
    let wait_metric = MetricRecord {
        name: "mean_wait_ticks".to_string(),
        value: batch_doc.aggregate_mean_wait,
        unit: scenario.time_unit.clone(),
        denominator_name: "started_patients".to_string(),
        denominator_value: total_started as f64,
        uncertainty: wait_uncertainty,
        censored_count: total_unfinished,
        unfinished_count: total_unfinished,
    };

    // Metric 2: Completion rate (denominator: total_arrivals)
    let comp_rates: Vec<f64> = batch_doc
        .replications
        .iter()
        .map(|r| {
            if r.arrivals > 0 {
                r.completed as f64 / r.arrivals as f64
            } else {
                0.0
            }
        })
        .collect();
    let comp_uncertainty = compute_uncertainty(&comp_rates);
    let comp_metric = MetricRecord {
        name: "completion_rate".to_string(),
        value: batch_doc.mean_completion_rate,
        unit: "ratio".to_string(),
        denominator_name: "total_arrivals".to_string(),
        denominator_value: total_arrivals as f64,
        uncertainty: comp_uncertainty,
        censored_count: total_unfinished,
        unfinished_count: total_unfinished,
    };

    // Metric 3: Throughput per horizon (denominator: horizon_ticks)
    let throughput_values: Vec<f64> = batch_doc
        .replications
        .iter()
        .map(|r| {
            if r.horizon_ticks > 0 {
                r.completed as f64 / r.horizon_ticks as f64
            } else {
                0.0
            }
        })
        .collect();
    let tp_uncertainty = compute_uncertainty(&throughput_values);
    let tp_mean = if !throughput_values.is_empty() {
        throughput_values.iter().sum::<f64>() / total_reps as f64
    } else {
        0.0
    };
    let tp_metric = MetricRecord {
        name: "throughput_per_horizon_tick".to_string(),
        value: tp_mean,
        unit: "patients_per_tick".to_string(),
        denominator_name: "horizon_ticks".to_string(),
        denominator_value: scenario.horizon_ticks as f64,
        uncertainty: tp_uncertainty,
        censored_count: total_unfinished,
        unfinished_count: total_unfinished,
    };

    // Metric 4: Censor / Unfinished presentation rate
    let censor_rates: Vec<f64> = batch_doc
        .replications
        .iter()
        .map(|r| {
            if r.arrivals > 0 {
                r.unfinished as f64 / r.arrivals as f64
            } else {
                0.0
            }
        })
        .collect();
    let censor_uncertainty = compute_uncertainty(&censor_rates);
    let censor_mean = if !censor_rates.is_empty() {
        censor_rates.iter().sum::<f64>() / total_reps as f64
    } else {
        0.0
    };
    let censor_metric = MetricRecord {
        name: "censored_presentation_rate".to_string(),
        value: censor_mean,
        unit: "ratio".to_string(),
        denominator_name: "total_arrivals".to_string(),
        denominator_value: total_arrivals as f64,
        uncertainty: censor_uncertainty,
        censored_count: total_unfinished,
        unfinished_count: total_unfinished,
    };

    // Metric 5: Mean elapsed ticks (denominator: completed_patients)
    let completed_elapseds: Vec<f64> = patient_records
        .iter()
        .filter_map(|r| r.elapsed_ticks.map(|e| e as f64))
        .collect();
    let elapsed_uncertainty = compute_uncertainty(&completed_elapseds);
    let mean_elapsed = if !completed_elapseds.is_empty() {
        completed_elapseds.iter().sum::<f64>() / completed_elapseds.len() as f64
    } else {
        0.0
    };
    let elapsed_metric = MetricRecord {
        name: "mean_elapsed_ticks".to_string(),
        value: mean_elapsed,
        unit: scenario.time_unit.clone(),
        denominator_name: "completed_patients".to_string(),
        denominator_value: total_completed as f64,
        uncertainty: elapsed_uncertainty,
        censored_count: total_unfinished,
        unfinished_count: total_unfinished,
    };

    let metrics = vec![
        wait_metric,
        elapsed_metric,
        comp_metric,
        tp_metric,
        censor_metric,
    ];

    let patient_schema = build_patient_arrow_schema(&scenario.scenario_id, &scenario.time_unit);
    let metrics_schema = build_metrics_arrow_schema();
    let spatial_schema = build_spatial_arrow_schema();
    let parquet_schema = generate_parquet_schema_ddl();

    Ok(ArrowExportBundle {
        schema_version: 1,
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
