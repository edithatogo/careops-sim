#![forbid(unsafe_code)]

//! C6.4 Handoff Contracts, Visualization Invariance, and Distributed Compatibility.
//!
//! Verifies:
//! 1. `test_visualization_render_rate_invariance`:
//!    - Representative ED scenario with multiple treatment bays, staff zones, and patient cases.
//!    - Executes simulation under 4 visualization snapshot capture conditions:
//!      (1) Headless / 0 FPS (no snapshots extracted)
//!      (2) 30 FPS snapshot capture (sampling every 33 ticks)
//!      (3) 60 FPS snapshot capture (sampling every 16 ticks)
//!      (4) Jittered / slow consumer (intermittent frame sampling and drops)
//!    - Proves bit-for-bit identical RunSummary, patient outcomes, mean wait ticks, mean LOS,
//!      and SHA-256 hashes across all 4 capture conditions.
//!
//! 2. `test_snapshot_stream_and_slow_consumer_backpressure`:
//!    - Bounded snapshot queue (ring buffer / capacity-bounded channel) for UI streaming.
//!    - Slow consumer UI lagging behind simulation ticks.
//!    - Snapshot queue drops/coalesces display frames without blocking or stalling the simulation loop.
//!    - Event recording remains 100% lossless despite display frame drops.
//!
//! 3. `test_wasm_compatible_payload_and_tick_bounds`:
//!    - Validates that all entity IDs, timestamps, and position coordinates fit within Wasm-safe representations:
//!      - Ticks within JavaScript safe integer range (2^53 - 1).
//!      - Float coordinates are finite and non-NaN.
//!      - Entity IDs are alphanumeric strings.
//!    - Tests serialization round-trip for browser/PixiJS consumption.
//!
//! 4. `test_distributed_and_pdes_export_bundle_compatibility`:
//!    - Validates that Arrow export bundles (`generate_arrow_export_bundle`) and batch runs
//!      (`run_batch_with_workers`) can be partitioned by replication ID, sent across distributed/PDES
//!      worker channels, and reassembled into identical aggregate metrics and output files.

use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use careops_ed::disposition::api_runner::{
    allocate_replication_seeds, compute_sha256_hex, RunDocument,
};
use careops_ed::disposition::arrow_export::generate_arrow_export_bundle;
use careops_ed::disposition::batch_runner::{
    compute_batch_aggregates, execute_single_replication, run_batch_with_workers, BatchConfig,
    BatchRunDocument, ReplicationResult,
};
use careops_ed::disposition::composite_fixture::{
    st003_verify_retained_occupancy, ComposedPatient, CompositeEdFixture, PatientLifecycle,
};
use careops_ed::disposition::operations::{BayPhase, ExecutionFidelity};
use careops_ed::{parse_scenario_json, Eligibility, PatientStatus, RunSummary, ScenarioConfig};
use serde::{Deserialize, Serialize};

/// Maximum safe integer in JavaScript / WebAssembly IEEE 754 float representation (2^53 - 1).
pub const JS_MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

// ---------------------------------------------------------------------------
// Visualization Snapshot and Stream Models
// ---------------------------------------------------------------------------

/// Read-only snapshot of simulation state extracted for visual rendering.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VisualizationSnapshot {
    pub schema_version: u32,
    pub scenario_id: String,
    pub sequence_number: u64,
    pub tick: u64,
    pub bays: Vec<BayVisualizationState>,
    pub active_patients: Vec<PatientVisualizationState>,
    pub queue_count: usize,
    pub in_flight_count: usize,
    pub completed_count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BayVisualizationState {
    pub bay_index: usize,
    pub zone_id: String,
    pub location_id: String,
    pub is_occupied: bool,
    pub patient_id: Option<u64>,
    pub diagnostic_blocked: bool,
    pub boarding: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PatientVisualizationState {
    pub patient_id: u64,
    pub name: String,
    pub acuity: u8,
    pub lifecycle: String,
    pub assigned_bay: Option<usize>,
    pub work_elapsed_ticks: u64,
    pub work_required_ticks: u64,
    pub x: f64,
    pub y: f64,
}

/// Bounded queue for streaming visualization snapshots to a UI with frame-dropping backpressure.
pub struct BoundedSnapshotQueue<T> {
    capacity: usize,
    buffer: VecDeque<T>,
    total_produced: u64,
    total_dropped: u64,
}

impl<T> BoundedSnapshotQueue<T> {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Queue capacity must be greater than zero");
        Self {
            capacity,
            buffer: VecDeque::with_capacity(capacity),
            total_produced: 0,
            total_dropped: 0,
        }
    }

    /// Pushes the latest frame. If capacity is exceeded, drops the oldest unconsumed
    /// frame so that the simulation loop never blocks or stalls.
    pub fn push_latest(&mut self, item: T) {
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front();
            self.total_dropped += 1;
        }
        self.buffer.push_back(item);
        self.total_produced += 1;
    }

    pub fn pop_frame(&mut self) -> Option<T> {
        self.buffer.pop_front()
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn total_produced(&self) -> u64 {
        self.total_produced
    }

    pub fn total_dropped(&self) -> u64 {
        self.total_dropped
    }
}

// ---------------------------------------------------------------------------
// Lossless Simulation Event Log
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SimEventKind {
    PatientAdmitted,
    PatientDispatched,
    PatientPreempted,
    DiagnosticsBlocked,
    DiagnosticsCleared,
    BoardingStarted,
    BoardingTransferred,
    PatientCompleted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LosslessSimEvent {
    pub sequence: u64,
    pub tick: u64,
    pub patient_id: u64,
    pub kind: SimEventKind,
}

#[derive(Default)]
pub struct LosslessEventLog {
    events: Vec<LosslessSimEvent>,
}

impl LosslessEventLog {
    pub fn record(&mut self, tick: u64, patient_id: u64, kind: SimEventKind) {
        let sequence = self.events.len() as u64;
        self.events.push(LosslessSimEvent {
            sequence,
            tick,
            patient_id,
            kind,
        });
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn events(&self) -> &[LosslessSimEvent] {
        &self.events
    }
}

// ---------------------------------------------------------------------------
// Wasm / Browser / PixiJS Payload Models
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PixiEntityRenderPayload {
    pub entity_id: String,
    pub kind: String,
    pub tick: u64,
    pub x: f64,
    pub y: f64,
    pub rotation: f64,
    pub alpha: f64,
    pub visible: bool,
    pub status_label: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PixiSceneSnapshotPayload {
    pub schema_version: u32,
    pub run_id: String,
    pub tick: u64,
    pub entities: Vec<PixiEntityRenderPayload>,
}

/// Validates that all fields in a scene payload satisfy Wasm / JS safety rules.
pub fn validate_wasm_payload(payload: &PixiSceneSnapshotPayload) -> Result<(), String> {
    if payload.tick > JS_MAX_SAFE_INTEGER {
        return Err(format!(
            "Scene tick {} exceeds JavaScript MAX_SAFE_INTEGER",
            payload.tick
        ));
    }
    if payload.run_id.is_empty()
        || !payload
            .run_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        return Err(format!("Run ID '{}' is not alphanumeric", payload.run_id));
    }
    for entity in &payload.entities {
        if entity.tick > JS_MAX_SAFE_INTEGER {
            return Err(format!(
                "Entity tick {} exceeds JavaScript MAX_SAFE_INTEGER",
                entity.tick
            ));
        }
        if !entity.x.is_finite() || entity.x.is_nan() {
            return Err(format!("Invalid x coordinate: {}", entity.x));
        }
        if !entity.y.is_finite() || entity.y.is_nan() {
            return Err(format!("Invalid y coordinate: {}", entity.y));
        }
        if !entity.rotation.is_finite() || entity.rotation.is_nan() {
            return Err(format!("Invalid rotation: {}", entity.rotation));
        }
        if !entity.alpha.is_finite() || !(0.0..=1.0).contains(&entity.alpha) {
            return Err(format!("Invalid alpha: {}", entity.alpha));
        }
        if entity.entity_id.is_empty()
            || !entity
                .entity_id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
        {
            return Err(format!(
                "Entity ID '{}' is not alphanumeric",
                entity.entity_id
            ));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Fixture and Scenario Helpers
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct TestPatientCase {
    id: u64,
    name: &'static str,
    acuity: u8,
    arrival_tick: u64,
    work_ticks: u64,
    zone_id: &'static str,
    location_id: &'static str,
}

fn create_representative_patient_cases() -> Vec<TestPatientCase> {
    vec![
        TestPatientCase {
            id: 101,
            name: "Patient_101_Subacute_A4",
            acuity: 4,
            arrival_tick: 0,
            work_ticks: 20,
            zone_id: "zone.subacute",
            location_id: "loc.subacute.bay1",
        },
        TestPatientCase {
            id: 102,
            name: "Patient_102_Acute_A2",
            acuity: 2,
            arrival_tick: 0,
            work_ticks: 25,
            zone_id: "zone.acute",
            location_id: "loc.acute.bay1",
        },
        TestPatientCase {
            id: 201,
            name: "Patient_201_Acute_Preemptor_A1",
            acuity: 1,
            arrival_tick: 5,
            work_ticks: 12,
            zone_id: "zone.acute",
            location_id: "loc.acute.bay1",
        },
        TestPatientCase {
            id: 301,
            name: "Patient_301_DiagnosticHold_A3",
            acuity: 3,
            arrival_tick: 8,
            work_ticks: 22,
            zone_id: "zone.subacute",
            location_id: "loc.subacute.bay2",
        },
        TestPatientCase {
            id: 401,
            name: "Patient_401_Boarding_A2",
            acuity: 2,
            arrival_tick: 10,
            work_ticks: 15,
            zone_id: "zone.acute",
            location_id: "loc.acute.bay2",
        },
        TestPatientCase {
            id: 501,
            name: "Patient_501_LateArrival_A5",
            acuity: 5,
            arrival_tick: 18,
            work_ticks: 16,
            zone_id: "zone.subacute",
            location_id: "loc.subacute.bay1",
        },
        TestPatientCase {
            id: 502,
            name: "Patient_502_Urgent_A1",
            acuity: 1,
            arrival_tick: 25,
            work_ticks: 10,
            zone_id: "zone.acute",
            location_id: "loc.acute.bay1",
        },
    ]
}

fn create_representative_eligibilities() -> Vec<Eligibility> {
    vec![
        Eligibility {
            staff_role_id: "staff.ed_nurse".to_string(),
            task_class_id: "task.treatment".to_string(),
            zone_id: "zone.acute".to_string(),
            location_id: Some("loc.acute.bay1".to_string()),
        },
        Eligibility {
            staff_role_id: "staff.ed_nurse".to_string(),
            task_class_id: "task.treatment".to_string(),
            zone_id: "zone.acute".to_string(),
            location_id: Some("loc.acute.bay2".to_string()),
        },
        Eligibility {
            staff_role_id: "staff.ed_nurse".to_string(),
            task_class_id: "task.treatment".to_string(),
            zone_id: "zone.subacute".to_string(),
            location_id: Some("loc.subacute.bay1".to_string()),
        },
        Eligibility {
            staff_role_id: "staff.ed_nurse".to_string(),
            task_class_id: "task.treatment".to_string(),
            zone_id: "zone.subacute".to_string(),
            location_id: Some("loc.subacute.bay2".to_string()),
        },
    ]
}

/// Extracts a read-only visualization snapshot from the composite ED fixture.
fn extract_visualization_snapshot(
    fixture: &CompositeEdFixture,
    scenario_id: &str,
    seq: u64,
    tick: u64,
) -> VisualizationSnapshot {
    let mut bays = Vec::new();
    for (idx, bay) in fixture.bays.iter().enumerate() {
        let (is_occupied, patient_id, diagnostic_blocked, boarding) = match bay.phase() {
            BayPhase::Occupied {
                patient_id,
                diagnostic_blocked,
                boarding,
            } => (true, Some(patient_id), diagnostic_blocked, boarding),
            _ => (false, None, false, false),
        };
        bays.push(BayVisualizationState {
            bay_index: idx,
            zone_id: if idx < 2 {
                "zone.acute".to_string()
            } else {
                "zone.subacute".to_string()
            },
            location_id: format!("loc.bay.{}", idx + 1),
            is_occupied,
            patient_id,
            diagnostic_blocked,
            boarding,
        });
    }

    let mut active_patients = Vec::new();
    for p in &fixture.in_flight {
        let (x, y) = if let Some(bay_idx) = p.assigned_bay {
            (100.0 + (bay_idx as f64) * 50.0, 200.0)
        } else {
            (50.0, 50.0)
        };
        active_patients.push(PatientVisualizationState {
            patient_id: p.id,
            name: p.name.clone(),
            acuity: p.acuity,
            lifecycle: format!("{:?}", p.lifecycle),
            assigned_bay: p.assigned_bay,
            work_elapsed_ticks: p.work_elapsed_ticks,
            work_required_ticks: p.work_required_ticks,
            x,
            y,
        });
    }

    VisualizationSnapshot {
        schema_version: 1,
        scenario_id: scenario_id.to_string(),
        sequence_number: seq,
        tick,
        bays,
        active_patients,
        queue_count: fixture.queue.len(),
        in_flight_count: fixture.in_flight.len(),
        completed_count: fixture.completed.len(),
    }
}

/// Runs the representative ED simulation under a given snapshot sampling predicate.
fn run_simulation_under_snapshot_policy<F>(
    scenario_id: &str,
    seed: u64,
    horizon_ticks: u64,
    mut should_capture: F,
    event_log: Option<&mut LosslessEventLog>,
) -> (RunDocument, Vec<VisualizationSnapshot>)
where
    F: FnMut(u64) -> bool,
{
    let eligibilities = create_representative_eligibilities();
    let cases = create_representative_patient_cases();
    let mut fixture = CompositeEdFixture::new(
        4,
        4,
        "staff.ed_nurse",
        eligibilities,
        ExecutionFidelity::Macro,
    );

    let mut snapshots = Vec::new();
    let mut next_seq = 0u64;
    let mut total_admitted = 0usize;

    let mut log_ref = event_log;

    for current_tick in 0..horizon_ticks {
        let mut newly_admitted = false;
        for case in &cases {
            if case.arrival_tick == current_tick {
                let p = ComposedPatient {
                    id: case.id,
                    name: case.name.to_string(),
                    acuity: case.acuity,
                    arrival_tick: case.arrival_tick,
                    work_required_ticks: case.work_ticks,
                    work_elapsed_ticks: 0,
                    start_tick: None,
                    completion_tick: None,
                    assigned_bay: None,
                    lifecycle: PatientLifecycle::Arrived,
                    task_class_id: "task.treatment".to_string(),
                    zone_id: case.zone_id.to_string(),
                    location_id: case.location_id.to_string(),
                };
                fixture.admit_patient(p);
                newly_admitted = true;
                total_admitted += 1;

                if let Some(ref mut log) = log_ref {
                    log.record(current_tick, case.id, SimEventKind::PatientAdmitted);
                }

                if case.acuity == 1 && fixture.try_preempt_lowest(1) {
                    if let Some(ref mut log) = log_ref {
                        log.record(current_tick, case.id, SimEventKind::PatientPreempted);
                    }
                }
            }
        }

        if newly_admitted || !fixture.queue.is_empty() {
            fixture.dispatch_cycle();
            if let Some(ref mut log) = log_ref {
                for p in &fixture.in_flight {
                    if p.start_tick == Some(current_tick) {
                        log.record(current_tick, p.id, SimEventKind::PatientDispatched);
                    }
                }
            }
        }

        // Diagnostic hold on Patient 301 between tick 15 and 20
        for p in &fixture.in_flight {
            if p.id == 301 {
                if let Some(bay_idx) = p.assigned_bay {
                    if (15..20).contains(&current_tick) {
                        if fixture.bays[bay_idx].phase()
                            == (BayPhase::Occupied {
                                patient_id: 301,
                                diagnostic_blocked: false,
                                boarding: false,
                            })
                        {
                            assert!(fixture.bays[bay_idx].block_for_diagnostics().is_ok());
                            if let Some(ref mut log) = log_ref {
                                log.record(current_tick, 301, SimEventKind::DiagnosticsBlocked);
                            }
                        }
                    } else if current_tick == 20
                        && fixture.bays[bay_idx].phase()
                            == (BayPhase::Occupied {
                                patient_id: 301,
                                diagnostic_blocked: true,
                                boarding: false,
                            })
                    {
                        assert!(fixture.bays[bay_idx].clear_diagnostics().is_ok());
                        if let Some(ref mut log) = log_ref {
                            log.record(current_tick, 301, SimEventKind::DiagnosticsCleared);
                        }
                    }
                }
            }
        }

        // Boarding on Patient 401 when work finishes
        for p in &fixture.in_flight {
            if p.id == 401 && p.work_elapsed_ticks + 1 >= p.work_required_ticks {
                if let Some(bay_idx) = p.assigned_bay {
                    if fixture.bays[bay_idx].phase()
                        == (BayPhase::Occupied {
                            patient_id: 401,
                            diagnostic_blocked: false,
                            boarding: false,
                        })
                    {
                        assert!(fixture.bays[bay_idx].begin_boarding().is_ok());
                        if let Some(ref mut log) = log_ref {
                            log.record(current_tick, 401, SimEventKind::BoardingStarted);
                        }
                    }
                }
            }
        }

        // Boarding transfer at tick 42
        if current_tick == 42 {
            let boarding_p401 = fixture
                .in_flight
                .iter()
                .any(|p| p.id == 401 && p.lifecycle == PatientLifecycle::Boarding);
            if boarding_p401 {
                assert!(fixture.complete_boarding_transfer(401).is_ok());
                if let Some(ref mut log) = log_ref {
                    log.record(current_tick, 401, SimEventKind::BoardingTransferred);
                }
            }
        }

        // Capture visualization snapshot if requested
        if should_capture(current_tick) {
            let snapshot =
                extract_visualization_snapshot(&fixture, scenario_id, next_seq, current_tick);
            next_seq += 1;
            snapshots.push(snapshot);
        }

        let completed_before = fixture.completed.len();
        fixture.advance_tick();
        let completed_after = fixture.completed.len();

        if let Some(ref mut log) = log_ref {
            if completed_after > completed_before {
                for p in &fixture.completed[completed_before..completed_after] {
                    log.record(current_tick, p.id, SimEventKind::PatientCompleted);
                }
            }
        }

        assert!(fixture.verify_invariants(total_admitted).is_ok());
    }

    let doc = fixture.to_run_document(scenario_id, seed, horizon_ticks);
    (doc, snapshots)
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn load_nominal_scenario() -> ScenarioConfig {
    let p = repo_root().join("crates/careops-ed/examples/nominal_mvp.json");
    let content = fs::read_to_string(&p).expect("read nominal_mvp.json");
    parse_scenario_json(&content).expect("parse nominal_mvp.json")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// 1. `test_visualization_render_rate_invariance`:
///
/// Assert that rendering/snapshot rate has ZERO effect on simulation physics and state transitions:
/// - 4 capture conditions:
///   (1) Headless / 0 FPS (no snapshots extracted)
///   (2) 30 FPS snapshot capture (sampling every 33 ticks)
///   (3) 60 FPS snapshot capture (sampling every 16 ticks)
///   (4) Jittered / slow consumer (intermittent frame sampling and drops)
/// - Simulation summary (RunSummary), patient outcomes, mean wait ticks, and mean LOS are bit-for-bit identical.
/// - Serialized JSON run documents and SHA-256 hashes match identically.
#[test]
fn test_visualization_render_rate_invariance() {
    let scenario_id = "c64-render-invariance-ed";
    let seed = 42u64;
    let horizon_ticks = 150u64;

    // Condition 1: Headless / 0 FPS
    let (doc_headless, snaps_headless) =
        run_simulation_under_snapshot_policy(scenario_id, seed, horizon_ticks, |_tick| false, None);

    // Condition 2: 30 FPS snapshot capture (~every 33 ticks)
    let (doc_30fps, snaps_30fps) = run_simulation_under_snapshot_policy(
        scenario_id,
        seed,
        horizon_ticks,
        |tick| tick % 33 == 0,
        None,
    );

    // Condition 3: 60 FPS snapshot capture (~every 16 ticks)
    let (doc_60fps, snaps_60fps) = run_simulation_under_snapshot_policy(
        scenario_id,
        seed,
        horizon_ticks,
        |tick| tick % 16 == 0,
        None,
    );

    // Condition 4: Jittered / slow consumer (intermittent irregular ticks)
    let (doc_jittered, snaps_jittered) = run_simulation_under_snapshot_policy(
        scenario_id,
        seed,
        horizon_ticks,
        |tick| matches!(tick, 0 | 12 | 13 | 47 | 50 | 85 | 110 | 111 | 145),
        None,
    );

    // Verify snapshot sample counts differ according to sampling predicates
    assert_eq!(snaps_headless.len(), 0, "Headless must capture 0 snapshots");
    assert_eq!(
        snaps_30fps.len(),
        5,
        "30 FPS must capture 5 snapshots across 150 ticks"
    );
    assert_eq!(
        snaps_60fps.len(),
        10,
        "60 FPS must capture 10 snapshots across 150 ticks"
    );
    assert_eq!(
        snaps_jittered.len(),
        9,
        "Jittered condition must capture 9 snapshots"
    );

    // 1. Bit-for-bit identical RunSummary across all 4 conditions
    assert_eq!(doc_headless.summary, doc_30fps.summary);
    assert_eq!(doc_headless.summary, doc_60fps.summary);
    assert_eq!(doc_headless.summary, doc_jittered.summary);

    // 2. Bit-for-bit identical patient outcomes across all 4 conditions
    assert_eq!(doc_headless.summary.patients.len(), 7);
    for i in 0..doc_headless.summary.patients.len() {
        let p_head = &doc_headless.summary.patients[i];
        let p_30 = &doc_30fps.summary.patients[i];
        let p_60 = &doc_60fps.summary.patients[i];
        let p_jit = &doc_jittered.summary.patients[i];

        assert_eq!(p_head, p_30);
        assert_eq!(p_head, p_60);
        assert_eq!(p_head, p_jit);
    }

    // 3. Compute and compare mean wait ticks and mean LOS across completed patients
    let compute_metrics = |summary: &RunSummary| -> (f64, f64) {
        let completed: Vec<_> = summary
            .patients
            .iter()
            .filter(|p| p.status == PatientStatus::Completed)
            .collect();
        assert!(!completed.is_empty());
        let wait_sum: u64 = completed.iter().filter_map(|p| p.wait_ticks).sum();
        let los_sum: u64 = completed.iter().filter_map(|p| p.elapsed_ticks).sum();
        let count = completed.len() as f64;
        (wait_sum as f64 / count, los_sum as f64 / count)
    };

    let (mean_wait_head, mean_los_head) = compute_metrics(&doc_headless.summary);
    let (mean_wait_30, mean_los_30) = compute_metrics(&doc_30fps.summary);
    let (mean_wait_60, mean_los_60) = compute_metrics(&doc_60fps.summary);
    let (mean_wait_jit, mean_los_jit) = compute_metrics(&doc_jittered.summary);

    assert_eq!(mean_wait_head, mean_wait_30);
    assert_eq!(mean_wait_head, mean_wait_60);
    assert_eq!(mean_wait_head, mean_wait_jit);

    assert_eq!(mean_los_head, mean_los_30);
    assert_eq!(mean_los_head, mean_los_60);
    assert_eq!(mean_los_head, mean_los_jit);

    // 4. Bit-for-bit identical serialized JSON run documents and SHA-256 hashes
    let json_head = serde_json::to_string_pretty(&doc_headless).unwrap();
    let json_30 = serde_json::to_string_pretty(&doc_30fps).unwrap();
    let json_60 = serde_json::to_string_pretty(&doc_60fps).unwrap();
    let json_jit = serde_json::to_string_pretty(&doc_jittered).unwrap();

    assert_eq!(json_head, json_30);
    assert_eq!(json_head, json_60);
    assert_eq!(json_head, json_jit);

    let hash_head = compute_sha256_hex(json_head.as_bytes());
    let hash_30 = compute_sha256_hex(json_30.as_bytes());
    let hash_60 = compute_sha256_hex(json_60.as_bytes());
    let hash_jit = compute_sha256_hex(json_jit.as_bytes());

    assert_eq!(hash_head, hash_30);
    assert_eq!(hash_head, hash_60);
    assert_eq!(hash_head, hash_jit);

    println!(
        "Render-rate invariance verified: SHA256={hash_head}, Completed={}, MeanWait={mean_wait_head:.2}, MeanLOS={mean_los_head:.2}",
        doc_headless.summary.completed
    );
}

/// 2. `test_snapshot_stream_and_slow_consumer_backpressure`:
/// - Bounded snapshot queue (ring buffer / capacity-bounded channel) for UI streaming.
/// - Slow consumer UI lagging behind simulation ticks.
/// - Assert that the snapshot queue drops/coalesces display frames without blocking or stalling the simulation loop.
/// - Assert that event recording remains 100% lossless despite display frame drops.
#[test]
fn test_snapshot_stream_and_slow_consumer_backpressure() {
    let scenario_id = "c64-backpressure-ed";
    let seed = 42u64;
    let horizon_ticks = 150u64;
    let queue_capacity = 4usize;

    let mut snapshot_queue = BoundedSnapshotQueue::new(queue_capacity);
    let mut lossless_log = LosslessEventLog::default();

    let eligibilities = create_representative_eligibilities();
    let cases = create_representative_patient_cases();
    let mut fixture = CompositeEdFixture::new(
        4,
        4,
        "staff.ed_nurse",
        eligibilities,
        ExecutionFidelity::Macro,
    );

    let mut consumed_frames = Vec::new();
    let mut last_consumed_seq = None;

    // Simulation loop running at full speed
    for current_tick in 0..horizon_ticks {
        // 1. Process arrivals and events
        let mut newly_admitted = false;
        for case in &cases {
            if case.arrival_tick == current_tick {
                let p = ComposedPatient {
                    id: case.id,
                    name: case.name.to_string(),
                    acuity: case.acuity,
                    arrival_tick: case.arrival_tick,
                    work_required_ticks: case.work_ticks,
                    work_elapsed_ticks: 0,
                    start_tick: None,
                    completion_tick: None,
                    assigned_bay: None,
                    lifecycle: PatientLifecycle::Arrived,
                    task_class_id: "task.treatment".to_string(),
                    zone_id: case.zone_id.to_string(),
                    location_id: case.location_id.to_string(),
                };
                fixture.admit_patient(p);
                newly_admitted = true;
                lossless_log.record(current_tick, case.id, SimEventKind::PatientAdmitted);

                if case.acuity == 1 && fixture.try_preempt_lowest(1) {
                    lossless_log.record(current_tick, case.id, SimEventKind::PatientPreempted);
                }
            }
        }

        if newly_admitted || !fixture.queue.is_empty() {
            fixture.dispatch_cycle();
            for p in &fixture.in_flight {
                if p.start_tick == Some(current_tick) {
                    lossless_log.record(current_tick, p.id, SimEventKind::PatientDispatched);
                }
            }
        }

        // Diagnostic hold on Patient 301 between tick 15 and 20
        for p in &fixture.in_flight {
            if p.id == 301 {
                if let Some(bay_idx) = p.assigned_bay {
                    if (15..20).contains(&current_tick) {
                        if fixture.bays[bay_idx].phase()
                            == (BayPhase::Occupied {
                                patient_id: 301,
                                diagnostic_blocked: false,
                                boarding: false,
                            })
                        {
                            assert!(fixture.bays[bay_idx].block_for_diagnostics().is_ok());
                            lossless_log.record(
                                current_tick,
                                301,
                                SimEventKind::DiagnosticsBlocked,
                            );
                        }
                    } else if current_tick == 20
                        && fixture.bays[bay_idx].phase()
                            == (BayPhase::Occupied {
                                patient_id: 301,
                                diagnostic_blocked: true,
                                boarding: false,
                            })
                    {
                        assert!(fixture.bays[bay_idx].clear_diagnostics().is_ok());
                        lossless_log.record(current_tick, 301, SimEventKind::DiagnosticsCleared);
                    }
                }
            }
        }

        // Boarding on Patient 401 when work finishes
        for p in &fixture.in_flight {
            if p.id == 401 && p.work_elapsed_ticks + 1 >= p.work_required_ticks {
                if let Some(bay_idx) = p.assigned_bay {
                    if fixture.bays[bay_idx].phase()
                        == (BayPhase::Occupied {
                            patient_id: 401,
                            diagnostic_blocked: false,
                            boarding: false,
                        })
                    {
                        assert!(fixture.bays[bay_idx].begin_boarding().is_ok());
                        lossless_log.record(current_tick, 401, SimEventKind::BoardingStarted);
                        let retained =
                            st003_verify_retained_occupancy(&fixture.bays[bay_idx], 401, false);
                        assert_eq!(retained, Ok(true));
                    }
                }
            }
        }

        if current_tick == 42 {
            let boarding_p401 = fixture
                .in_flight
                .iter()
                .any(|p| p.id == 401 && p.lifecycle == PatientLifecycle::Boarding);
            if boarding_p401 {
                assert!(fixture.complete_boarding_transfer(401).is_ok());
                lossless_log.record(current_tick, 401, SimEventKind::BoardingTransferred);
            }
        }

        // Capture snapshot every single tick (simulating high-frequency 60-120 Hz publisher)
        let snapshot = extract_visualization_snapshot(
            &fixture,
            scenario_id,
            snapshot_queue.total_produced(),
            current_tick,
        );
        snapshot_queue.push_latest(snapshot);

        // Advance simulation physics
        let completed_before = fixture.completed.len();
        fixture.advance_tick();
        let completed_after = fixture.completed.len();
        if completed_after > completed_before {
            for p in &fixture.completed[completed_before..completed_after] {
                lossless_log.record(current_tick, p.id, SimEventKind::PatientCompleted);
            }
        }

        // Slow consumer UI simulation: polls only every 30 ticks and reads 1 frame
        if current_tick > 0 && current_tick % 30 == 0 {
            if let Some(frame) = snapshot_queue.pop_frame() {
                if let Some(last_seq) = last_consumed_seq {
                    assert!(
                        frame.sequence_number > last_seq,
                        "Consumed frames must have monotonically increasing sequence numbers"
                    );
                }
                last_consumed_seq = Some(frame.sequence_number);
                consumed_frames.push(frame);
            }
        }
    }

    // 1. Verify simulation ran smoothly to completion without stalling
    assert_eq!(fixture.current_tick, horizon_ticks);
    let summary = fixture.to_run_summary(scenario_id, seed, horizon_ticks);
    assert_eq!(summary.arrivals, cases.len());
    assert_eq!(summary.completed, cases.len());
    assert_eq!(summary.unfinished, 0);

    // 2. Verify snapshot queue produced 150 frames, bounded buffer never exceeded capacity (4)
    assert_eq!(snapshot_queue.total_produced(), 150);
    assert!(snapshot_queue.len() <= queue_capacity);

    // 3. Verify that frames were dropped/coalesced under backpressure
    assert!(
        snapshot_queue.total_dropped() > 100,
        "Snapshot queue must drop surplus frames for slow consumer (dropped={})",
        snapshot_queue.total_dropped()
    );
    assert_eq!(
        snapshot_queue.total_dropped()
            + (snapshot_queue.len() as u64)
            + (consumed_frames.len() as u64),
        snapshot_queue.total_produced(),
        "Conservation of produced frames: dropped + buffered + consumed == produced"
    );

    // 4. Verify 100% lossless event recording despite frame drops
    assert!(!lossless_log.is_empty(), "Event log must not be empty");
    let admitted_events = lossless_log
        .events()
        .iter()
        .filter(|e| e.kind == SimEventKind::PatientAdmitted)
        .count();
    let completed_events = lossless_log
        .events()
        .iter()
        .filter(|e| e.kind == SimEventKind::PatientCompleted)
        .count();
    let boarding_transferred_events = lossless_log
        .events()
        .iter()
        .filter(|e| e.kind == SimEventKind::BoardingTransferred)
        .count();
    let preemption_events = lossless_log
        .events()
        .iter()
        .filter(|e| e.kind == SimEventKind::PatientPreempted)
        .count();
    let boarding_events = lossless_log
        .events()
        .iter()
        .filter(|e| e.kind == SimEventKind::BoardingStarted)
        .count();

    assert_eq!(
        admitted_events,
        cases.len(),
        "All arrivals losslessly recorded"
    );
    assert_eq!(
        completed_events, 6,
        "6 discharge completions losslessly recorded"
    );
    assert_eq!(
        boarding_transferred_events, 1,
        "1 boarding transfer completion losslessly recorded"
    );
    assert_eq!(
        completed_events + boarding_transferred_events,
        cases.len(),
        "All 7 patients reached completed state"
    );
    assert_eq!(
        preemption_events, 2,
        "Preemption events losslessly recorded"
    );
    assert_eq!(
        boarding_events, 1,
        "Boarding start event losslessly recorded"
    );

    println!(
        "Slow consumer backpressure verified: Produced=150, Dropped={}, Consumed={}, LosslessEvents={}",
        snapshot_queue.total_dropped(),
        consumed_frames.len(),
        lossless_log.len()
    );
}

/// 3. `test_wasm_compatible_payload_and_tick_bounds`:
/// - Validate that all entity IDs, timestamps, and position coordinates fit within Wasm-safe representations:
///   - Ticks within safe integer range (2^53 - 1).
///   - Float coordinates are finite and non-NaN.
///   - Entity IDs are alphanumeric strings.
/// - Test serialization round-trip for browser/PixiJS consumption.
#[test]
fn test_wasm_compatible_payload_and_tick_bounds() {
    let scenario = load_nominal_scenario();

    // 1. Verify nominal scenario inputs adhere to Wasm bounds
    assert!(
        scenario.horizon_ticks <= JS_MAX_SAFE_INTEGER,
        "Horizon ticks must be within JS_MAX_SAFE_INTEGER"
    );
    for p in &scenario.patients {
        assert!(
            p.arrival_tick <= JS_MAX_SAFE_INTEGER,
            "Patient arrival tick must be within JS_MAX_SAFE_INTEGER"
        );
        assert!(
            p.work_ticks <= JS_MAX_SAFE_INTEGER,
            "Patient work ticks must be within JS_MAX_SAFE_INTEGER"
        );
        assert!(
            p.patient_id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.'),
            "Patient ID must be alphanumeric string"
        );
    }

    // 2. Build a representative Pixi scene snapshot payload from simulation
    let valid_payload = PixiSceneSnapshotPayload {
        schema_version: 1,
        run_id: "run_wasm_001".to_string(),
        tick: 48,
        entities: vec![
            PixiEntityRenderPayload {
                entity_id: "patient_101".to_string(),
                kind: "patient".to_string(),
                tick: 48,
                x: 120.5,
                y: 200.0,
                rotation: 0.0,
                alpha: 1.0,
                visible: true,
                status_label: "Treatment".to_string(),
            },
            PixiEntityRenderPayload {
                entity_id: "bay_acute_1".to_string(),
                kind: "bay".to_string(),
                tick: 48,
                x: 120.0,
                y: 195.0,
                rotation: std::f64::consts::FRAC_PI_2,
                alpha: 0.85,
                visible: true,
                status_label: "Occupied".to_string(),
            },
            PixiEntityRenderPayload {
                entity_id: "staff_nurse_01".to_string(),
                kind: "staff".to_string(),
                tick: 48,
                x: 125.0,
                y: 205.0,
                rotation: 0.0,
                alpha: 1.0,
                visible: true,
                status_label: "Assisting".to_string(),
            },
        ],
    };

    // Validate payload passes Wasm safety check
    assert!(
        validate_wasm_payload(&valid_payload).is_ok(),
        "Valid payload must pass Wasm safety validation"
    );

    // 3. Test serialization round-trip for browser/PixiJS consumption
    let serialized_json =
        serde_json::to_string(&valid_payload).expect("JSON serialization must succeed");
    let deserialized_payload: PixiSceneSnapshotPayload =
        serde_json::from_str(&serialized_json).expect("JSON deserialization must succeed");

    assert_eq!(valid_payload, deserialized_payload);

    // 4. Verify boundary rejection cases
    // (a) Tick exceeding JavaScript MAX_SAFE_INTEGER
    let mut bad_tick_payload = valid_payload.clone();
    bad_tick_payload.tick = JS_MAX_SAFE_INTEGER + 1;
    assert!(
        validate_wasm_payload(&bad_tick_payload).is_err(),
        "Tick exceeding MAX_SAFE_INTEGER must be rejected"
    );

    // (b) Entity tick exceeding JavaScript MAX_SAFE_INTEGER
    let mut bad_entity_tick_payload = valid_payload.clone();
    bad_entity_tick_payload.entities[0].tick = JS_MAX_SAFE_INTEGER + 100;
    assert!(
        validate_wasm_payload(&bad_entity_tick_payload).is_err(),
        "Entity tick exceeding MAX_SAFE_INTEGER must be rejected"
    );

    // (c) NaN float coordinate
    let mut bad_nan_coord = valid_payload.clone();
    bad_nan_coord.entities[0].x = f64::NAN;
    assert!(
        validate_wasm_payload(&bad_nan_coord).is_err(),
        "NaN coordinate must be rejected"
    );

    // (d) Infinite float coordinate
    let mut bad_inf_coord = valid_payload.clone();
    bad_inf_coord.entities[0].y = f64::INFINITY;
    assert!(
        validate_wasm_payload(&bad_inf_coord).is_err(),
        "Infinite coordinate must be rejected"
    );

    // (e) Invalid entity ID containing control characters or spaces
    let mut bad_entity_id = valid_payload.clone();
    bad_entity_id.entities[0].entity_id = "patient with spaces".to_string();
    assert!(
        validate_wasm_payload(&bad_entity_id).is_err(),
        "Entity ID with spaces must be rejected"
    );

    // (f) Invalid alpha out of [0.0, 1.0]
    let mut bad_alpha = valid_payload;
    bad_alpha.entities[0].alpha = 1.5;
    assert!(
        validate_wasm_payload(&bad_alpha).is_err(),
        "Alpha > 1.0 must be rejected"
    );

    println!("Wasm-compatible payload and tick bounds verified");
}

/// 4. `test_distributed_and_pdes_export_bundle_compatibility`:
/// - Validate that Arrow export bundles (`generate_arrow_export_bundle`) and batch runs
///   (`run_batch_with_workers`) can be partitioned by replication ID, sent across distributed/PDES
///   worker channels, and reassembled into identical aggregate metrics and output files.
#[test]
fn test_distributed_and_pdes_export_bundle_compatibility() {
    let scenario = load_nominal_scenario();
    let total_replications = 6usize;
    let base_seed = 42u64;
    let horizon_override = Some(60u64);

    // 1. Monolithic reference run
    let monolithic_config = BatchConfig {
        schema_version: 1,
        batch_id: "batch_monolithic_ref".to_string(),
        scenario_id: scenario.scenario_id.clone(),
        base_seed,
        replications: total_replications,
        num_workers: 2,
        horizon_override,
        checkpoint_interval: None,
    };

    let monolithic_doc = run_batch_with_workers(&scenario, &monolithic_config, None, None, None)
        .expect("monolithic batch run must succeed");
    let monolithic_bundle = generate_arrow_export_bundle(&scenario, &monolithic_doc, None, None)
        .expect("monolithic arrow export bundle must succeed");

    assert_eq!(monolithic_doc.replications.len(), total_replications);
    assert_eq!(monolithic_doc.total_replications, total_replications);

    // 2. Distributed / PDES Execution over worker channels
    // Partition 6 replications across 2 distributed worker nodes:
    // Node 0: replications 0, 1, 2
    // Node 1: replications 3, 4, 5
    let seeds = allocate_replication_seeds(base_seed, total_replications);
    let partition_0_tasks: Vec<(usize, u64)> = (0..3).map(|i| (i, seeds[i])).collect();
    let partition_1_tasks: Vec<(usize, u64)> = (3..6).map(|i| (i, seeds[i])).collect();

    let (tx, rx) = mpsc::channel::<(usize, Vec<ReplicationResult>)>();

    // Spawn Worker Node 0
    let tx0 = tx.clone();
    let scen0 = scenario.clone();
    let tasks0 = partition_0_tasks;
    let h_override0 = horizon_override;
    let handle0 = thread::spawn(move || {
        let mut results = Vec::new();
        for (idx, seed) in tasks0 {
            let res = execute_single_replication(&scen0, idx, seed, h_override0)
                .expect("worker 0 replication execution");
            results.push(res);
        }
        tx0.send((0, results)).expect("worker 0 send to channel");
    });

    // Spawn Worker Node 1
    let tx1 = tx;
    let scen1 = scenario.clone();
    let tasks1 = partition_1_tasks;
    let h_override1 = horizon_override;
    let handle1 = thread::spawn(move || {
        let mut results = Vec::new();
        for (idx, seed) in tasks1 {
            let res = execute_single_replication(&scen1, idx, seed, h_override1)
                .expect("worker 1 replication execution");
            results.push(res);
        }
        tx1.send((1, results)).expect("worker 1 send to channel");
    });

    // Wait for distributed worker threads to complete
    handle0.join().expect("node 0 join");
    handle1.join().expect("node 1 join");

    // Coordinator reassembles partitioned results from distributed channels
    let mut all_received_results = Vec::new();
    while let Ok((node_id, batch_partition)) = rx.try_recv() {
        assert!(node_id < 2);
        all_received_results.extend(batch_partition);
    }

    assert_eq!(
        all_received_results.len(),
        total_replications,
        "All partitioned replications must be collected by coordinator"
    );

    // Canonical sorting by replication_index to reconstruct deterministic batch sequence
    all_received_results.sort_by_key(|r| r.replication_index);

    // Recompute aggregate metrics across all reassembled replications
    let (mean_wait, std_error, completion_rate) = compute_batch_aggregates(&all_received_results);

    // Reassemble BatchRunDocument
    let reassembled_doc = BatchRunDocument {
        schema_version: 1,
        batch_id: monolithic_doc.batch_id.clone(),
        scenario_id: monolithic_doc.scenario_id.clone(),
        status: "complete".to_string(),
        num_workers: monolithic_doc.num_workers,
        total_replications,
        replications: all_received_results,
        aggregate_mean_wait: mean_wait,
        aggregate_std_error: std_error,
        mean_completion_rate: completion_rate,
        completed_at_epoch_ms: monolithic_doc.completed_at_epoch_ms,
    };

    // Generate Arrow export bundle from reassembled document
    let reassembled_bundle = generate_arrow_export_bundle(&scenario, &reassembled_doc, None, None)
        .expect("reassembled arrow export bundle must succeed");

    // 3. Verify exact equivalence between Monolithic and Distributed/Reassembled runs
    // (a) Replication results match bit-for-bit
    assert_eq!(
        reassembled_doc.replications, monolithic_doc.replications,
        "Reassembled replication results must match monolithic baseline exactly"
    );

    // (b) Aggregated metrics match bit-for-bit
    assert_eq!(
        reassembled_doc.aggregate_mean_wait, monolithic_doc.aggregate_mean_wait,
        "Aggregate mean wait ticks must match"
    );
    assert_eq!(
        reassembled_doc.aggregate_std_error, monolithic_doc.aggregate_std_error,
        "Aggregate standard error must match"
    );
    assert_eq!(
        reassembled_doc.mean_completion_rate, monolithic_doc.mean_completion_rate,
        "Mean completion rate must match"
    );

    // (c) Arrow export bundle patient records match bit-for-bit
    assert_eq!(
        reassembled_bundle.patient_records.len(),
        monolithic_bundle.patient_records.len()
    );
    assert_eq!(
        reassembled_bundle.patient_records, monolithic_bundle.patient_records,
        "Patient records must be bit-for-bit identical"
    );

    // (d) Arrow export bundle spatial records match bit-for-bit
    assert_eq!(
        reassembled_bundle.spatial_records.len(),
        monolithic_bundle.spatial_records.len()
    );
    assert_eq!(
        reassembled_bundle.spatial_records, monolithic_bundle.spatial_records,
        "Spatial records must be bit-for-bit identical"
    );

    // (e) Arrow export bundle metrics match bit-for-bit
    assert_eq!(
        reassembled_bundle.metrics, monolithic_bundle.metrics,
        "Metrics records must be bit-for-bit identical"
    );

    // (f) Serialized JSON and SHA-256 hashes match identically
    let json_monolithic = serde_json::to_string_pretty(&monolithic_bundle).unwrap();
    let json_reassembled = serde_json::to_string_pretty(&reassembled_bundle).unwrap();
    assert_eq!(json_monolithic, json_reassembled);

    let hash_monolithic = compute_sha256_hex(json_monolithic.as_bytes());
    let hash_reassembled = compute_sha256_hex(json_reassembled.as_bytes());
    assert_eq!(hash_monolithic, hash_reassembled);

    println!(
        "Distributed/PDES compatibility verified: Replications={total_replications}, Patients={}, Transits={}, SHA256={hash_reassembled}",
        reassembled_bundle.patient_records.len(),
        reassembled_bundle.spatial_records.len()
    );
}
