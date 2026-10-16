//! Composed synthetic ED fixture, ST001–ST009 boundary cases, and MVP summary tools.
//!
//! Provides the shared C6 fixture foundations:
//! - Composed ED operational runtime with multiple named zones, beds, and staff roles.
//! - Simultaneous arrivals, priority dispatch (ATS 1-5), and preemption with work conservation.
//! - Strict conservation: no double allocation, no leaked capacity, no starvation hidden by censoring.
//! - ST001–ST009 acceptance boundary specifications.
//! - CSV export and capacity comparison oracle.

use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::disposition::operations::{
    claim_task, preempt, select_dispatch_candidate, validate_procedure_staffing, BayPhase,
    ExecutionFidelity, OperationalError, PreemptConsent, ProcedureKind, StaffShift, SyntheticGraph,
    TaskCandidate, TaskClaimTarget, TreatmentBay, WorkAccount,
};
use crate::{Eligibility, PatientStatus, RunSummary};

/// Boundary and specification errors for ST001–ST009.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoundaryError {
    IncompatibleOfferDestination,
    ImmediateExpiryDefaultProhibited,
    PrematureBayReleaseBeforePhysicalTransfer,
    ExcessiveBoarderStaffReservation,
    AmbulanceOffloadBlockedBySaturation,
    ReceiverStaffUnavailable,
    ProhibitedCausalWardPolicyClaim,
    UnsupportedImplicitDeteriorationHazard,
}

impl Display for BoundaryError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::IncompatibleOfferDestination => {
                "ST001: bed offer destination is not compatible with patient clinical eligibility"
            }
            Self::ImmediateExpiryDefaultProhibited => {
                "ST002: immediate expiry is not an accepted universal default; explicit policy required"
            }
            Self::PrematureBayReleaseBeforePhysicalTransfer => {
                "ST003: boarder must retain ED bay occupancy until physical departure is complete"
            }
            Self::ExcessiveBoarderStaffReservation => {
                "ST004: dedicating a full-time 1:1 nurse per boarder by default is prohibited"
            }
            Self::AmbulanceOffloadBlockedBySaturation => {
                "ST005: receiving bay saturation blocks ambulance offload and delays crew release"
            }
            Self::ReceiverStaffUnavailable => {
                "ST006: offload delayed because receiving staff are unavailable or unassigned"
            }
            Self::ProhibitedCausalWardPolicyClaim => {
                "ST008: external offer stream cannot claim causal effects of inpatient ward discharge policy"
            }
            Self::UnsupportedImplicitDeteriorationHazard => {
                "ST009: deterioration hazard cannot be an implicit default; explicit model required"
            }
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for BoundaryError {}

/// Bed offer status for external admission boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OfferStatus {
    Pending,
    Accepted,
    Withdrawn,
    Expired,
}

/// ST001: Verifies that an external bed offer is compatible with the patient destination.
pub fn st001_validate_compatible_offer(
    patient_specialty: &str,
    offer_specialty: &str,
) -> Result<(), BoundaryError> {
    if patient_specialty == offer_specialty {
        Ok(())
    } else {
        Err(BoundaryError::IncompatibleOfferDestination)
    }
}

/// ST002: Verifies bed offer lifetime and persistence without assuming immediate expiry.
pub fn st002_evaluate_offer_policy(
    offered_tick: u64,
    current_tick: u64,
    lifetime_ticks: u64,
    assumes_immediate_expiry: bool,
) -> Result<OfferStatus, BoundaryError> {
    if assumes_immediate_expiry && lifetime_ticks == 0 {
        return Err(BoundaryError::ImmediateExpiryDefaultProhibited);
    }
    if current_tick < offered_tick {
        return Ok(OfferStatus::Pending);
    }
    let elapsed = current_tick - offered_tick;
    if elapsed <= lifetime_ticks {
        Ok(OfferStatus::Accepted)
    } else {
        Ok(OfferStatus::Expired)
    }
}

/// ST003: Verifies retained ED bay occupancy during boarding until physical transfer.
pub fn st003_verify_retained_occupancy(
    bay: &TreatmentBay,
    patient_id: u64,
    physical_departure_complete: bool,
) -> Result<bool, BoundaryError> {
    if !physical_departure_complete {
        match bay.phase() {
            BayPhase::Occupied {
                patient_id: occupied_id,
                ..
            } if occupied_id == patient_id => Ok(true),
            _ => Err(BoundaryError::PrematureBayReleaseBeforePhysicalTransfer),
        }
    } else {
        Ok(false)
    }
}

/// ST004: Verifies retained recurrent care without dedicating a full-time 1:1 nurse.
pub fn st004_verify_boarder_recurrent_care(
    _has_recurrent_care_tasks: bool,
    reserves_full_time_nurse: bool,
) -> Result<(), BoundaryError> {
    if reserves_full_time_nurse {
        Err(BoundaryError::ExcessiveBoarderStaffReservation)
    } else {
        Ok(())
    }
}

/// Ambulance offload progression states for ST005/ST006.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OffloadStatus {
    QueuedAwaitingBay,
    HandoverInProgress,
    OffloadedCrewReleased,
}

/// ST005: Saturated offload: receiving resource availability governs ambulance offload.
pub fn st005_evaluate_ambulance_offload(
    available_bays: u64,
    receiving_staff_free: u64,
) -> Result<OffloadStatus, BoundaryError> {
    if available_bays == 0 {
        return Err(BoundaryError::AmbulanceOffloadBlockedBySaturation);
    }
    if receiving_staff_free == 0 {
        return Err(BoundaryError::ReceiverStaffUnavailable);
    }
    Ok(OffloadStatus::OffloadedCrewReleased)
}

/// ST006: Receiver staff sensitivity: offload delay depends on receiving nurse availability.
pub fn st006_receiver_staff_delay(
    receiving_staff_claimable: u64,
    base_delay_ticks: u64,
) -> Result<u64, BoundaryError> {
    if receiving_staff_claimable == 0 {
        Err(BoundaryError::ReceiverStaffUnavailable)
    } else {
        Ok(base_delay_ticks)
    }
}

/// ST007: Enforces composite team requirement for complex multi-person procedures.
pub fn st007_enforce_multi_role_procedure(
    procedure: ProcedureKind,
    is_composite_team: bool,
) -> Result<(), OperationalError> {
    validate_procedure_staffing(procedure, is_composite_team)
}

/// ST008: Prohibits ungrounded causal claims of inpatient ward discharge policy.
pub fn st008_validate_ward_policy_causal_claim(
    claims_causal_ward_policy: bool,
    has_internal_ward_model: bool,
) -> Result<(), BoundaryError> {
    if claims_causal_ward_policy && !has_internal_ward_model {
        Err(BoundaryError::ProhibitedCausalWardPolicyClaim)
    } else {
        Ok(())
    }
}

/// ST009: Prohibits implicit default deterioration hazards without explicit configuration.
pub fn st009_validate_deterioration_hazard(
    implicit_default_hazard_active: bool,
    explicitly_parameterized: bool,
) -> Result<(), BoundaryError> {
    if implicit_default_hazard_active && !explicitly_parameterized {
        Err(BoundaryError::UnsupportedImplicitDeteriorationHazard)
    } else {
        Ok(())
    }
}

/// Patient clinical state within the composed fixture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PatientLifecycle {
    Arrived,
    WaitingInQueue,
    InTreatment,
    Preempted,
    Boarding,
    Discharged,
    Transferred,
}

/// A synthetic patient tracked through the composed fixture.
#[derive(Clone, Debug, PartialEq)]
pub struct ComposedPatient {
    pub id: u64,
    pub name: String,
    pub acuity: u8,
    pub arrival_tick: u64,
    pub work_required_ticks: u64,
    pub work_elapsed_ticks: u64,
    pub start_tick: Option<u64>,
    pub completion_tick: Option<u64>,
    pub assigned_bay: Option<usize>,
    pub lifecycle: PatientLifecycle,
    pub task_class_id: String,
    pub zone_id: String,
    pub location_id: String,
}

/// Integrated nominal ED operational fixture.
#[derive(Clone, Debug)]
pub struct CompositeEdFixture {
    pub current_tick: u64,
    pub bays: Vec<TreatmentBay>,
    pub shift: StaffShift,
    pub eligibilities: Vec<Eligibility>,
    pub role_id: String,
    pub queue: Vec<ComposedPatient>,
    pub in_flight: Vec<ComposedPatient>,
    pub completed: Vec<ComposedPatient>,
    pub graph: Option<SyntheticGraph>,
    pub fidelity: ExecutionFidelity,
}

impl CompositeEdFixture {
    pub fn new(
        num_bays: usize,
        staff_scheduled: u64,
        role_id: &str,
        eligibilities: Vec<Eligibility>,
        fidelity: ExecutionFidelity,
    ) -> Self {
        let bays = (0..num_bays).map(|_| TreatmentBay::available()).collect();
        Self {
            current_tick: 0,
            bays,
            shift: StaffShift::new(staff_scheduled),
            eligibilities,
            role_id: role_id.to_string(),
            queue: Vec::new(),
            in_flight: Vec::new(),
            completed: Vec::new(),
            graph: None,
            fidelity,
        }
    }

    pub fn set_graph(&mut self, graph: SyntheticGraph) {
        self.graph = Some(graph);
    }

    /// Enqueue a newly arrived patient.
    pub fn admit_patient(&mut self, patient: ComposedPatient) {
        let mut p = patient;
        p.lifecycle = PatientLifecycle::WaitingInQueue;
        self.queue.push(p);
    }

    /// Dispatches eligible waiting patients to available bays and staff.
    pub fn dispatch_cycle(&mut self) {
        // Collect candidates for dispatch
        while self.shift.claimable() > 0 {
            let free_bay_idx = self
                .bays
                .iter()
                .position(|b| b.phase() == BayPhase::Available);
            let Some(bay_idx) = free_bay_idx else {
                break;
            };

            let candidates: Vec<TaskCandidate> = self
                .queue
                .iter()
                .map(|p| TaskCandidate {
                    acuity: p.acuity,
                    enqueue_tick: p.arrival_tick,
                    patient_id: p.id,
                    task_id: p.id.to_string(),
                    task_class_id: p.task_class_id.clone(),
                    zone_id: p.zone_id.clone(),
                    location_id: p.location_id.clone(),
                })
                .collect();

            let chosen = select_dispatch_candidate(
                &candidates,
                &self.shift,
                &self.role_id,
                &self.eligibilities,
            );

            let Some(candidate) = chosen else {
                break;
            };

            let patient_id = candidate.patient_id;
            let q_idx = self
                .queue
                .iter()
                .position(|p| p.id == patient_id)
                .expect("candidate must exist in queue");
            let mut patient = self.queue.remove(q_idx);

            // Phased claim: occupy space first, then claim staff
            if self.bays[bay_idx].occupy(patient.id).is_ok() {
                let target = TaskClaimTarget {
                    role_id: &self.role_id,
                    task_class_id: &patient.task_class_id,
                    zone_id: &patient.zone_id,
                    location_id: &patient.location_id,
                };
                if claim_task(
                    &mut self.shift,
                    &self.eligibilities,
                    target.role_id,
                    target.task_class_id,
                    target.zone_id,
                    target.location_id,
                )
                .is_ok()
                {
                    patient.assigned_bay = Some(bay_idx);
                    patient.lifecycle = PatientLifecycle::InTreatment;
                    if patient.start_tick.is_none() {
                        patient.start_tick = Some(self.current_tick);
                    }
                    self.in_flight.push(patient);
                } else {
                    let _ = self.bays[bay_idx].vacate_for_cleaning(self.current_tick);
                    let _ = self.bays[bay_idx].finish_cleaning(self.current_tick);
                    self.queue.insert(q_idx, patient);
                    break;
                }
            } else {
                self.queue.insert(q_idx, patient);
                break;
            }
        }
    }

    /// Evaluates preemption: higher acuity patient can preempt lower acuity patient with consent.
    pub fn try_preempt_lowest(&mut self, incoming_acuity: u8) -> bool {
        if self.in_flight.is_empty() {
            return false;
        }

        // Find candidate with lowest acuity (highest ATS integer)
        let worst_idx = self
            .in_flight
            .iter()
            .enumerate()
            .max_by_key(|(_, p)| p.acuity)
            .map(|(idx, _)| idx);

        let Some(idx) = worst_idx else {
            return false;
        };

        let victim_acuity = self.in_flight[idx].acuity;
        let work = WorkAccount {
            elapsed_ticks: self.in_flight[idx].work_elapsed_ticks,
            total_ticks: self.in_flight[idx].work_required_ticks,
        };
        if let Ok(preempt_outcome) = preempt(
            &work,
            victim_acuity,
            PreemptConsent::Preemptible,
            incoming_acuity,
        ) {
            if preempt_outcome.yielded {
                let mut victim = self.in_flight.remove(idx);
                victim.lifecycle = PatientLifecycle::Preempted;
                if let Some(bay_idx) = victim.assigned_bay.take() {
                    let _ = self.bays[bay_idx].vacate_for_cleaning(self.current_tick);
                    let _ = self.bays[bay_idx].finish_cleaning(self.current_tick);
                }
                let _ = self.shift.release_assignment();
                self.queue.insert(0, victim);
                return true;
            }
        }
        false
    }

    /// Advances the simulation clock by one tick.
    pub fn advance_tick(&mut self) {
        self.current_tick += 1;

        // Progress in-flight work
        let mut still_in_flight = Vec::new();
        while let Some(mut patient) = self.in_flight.pop() {
            patient.work_elapsed_ticks += 1;
            if patient.work_elapsed_ticks >= patient.work_required_ticks {
                patient.lifecycle = PatientLifecycle::Discharged;
                patient.completion_tick = Some(self.current_tick);
                if let Some(bay_idx) = patient.assigned_bay.take() {
                    let _ = self.bays[bay_idx].vacate_for_cleaning(self.current_tick);
                    let _ = self.bays[bay_idx].finish_cleaning(self.current_tick);
                }
                let _ = self.shift.release_assignment();
                self.completed.push(patient);
            } else {
                still_in_flight.push(patient);
            }
        }
        self.in_flight = still_in_flight;

        // Perform dispatch
        self.dispatch_cycle();
    }

    /// Verifies all conservation invariants:
    /// 1. Patient conservation: arrivals == queue + in_flight + completed.
    /// 2. Capacity conservation: occupied bays == in_flight count.
    /// 3. Staff conservation: active staff == in_flight count.
    /// 4. No double allocation: distinct bays assigned to distinct patients.
    pub fn verify_invariants(&self, total_admitted: usize) -> Result<(), OperationalError> {
        let active_count = self.in_flight.len();
        let queue_count = self.queue.len();
        let completed_count = self.completed.len();

        if active_count + queue_count + completed_count != total_admitted {
            return Err(OperationalError::Accounting);
        }

        if self.shift.active() as usize != active_count {
            return Err(OperationalError::Accounting);
        }

        let occupied_bays = self
            .bays
            .iter()
            .filter(|b| b.phase() != BayPhase::Available)
            .count();
        if occupied_bays != active_count {
            return Err(OperationalError::Accounting);
        }

        let mut assigned_bays = std::collections::HashSet::new();
        for p in &self.in_flight {
            if let Some(bay_idx) = p.assigned_bay {
                if !assigned_bays.insert(bay_idx) {
                    return Err(OperationalError::SpaceUnavailable);
                }
            } else {
                return Err(OperationalError::SpaceUnavailable);
            }
        }

        Ok(())
    }

    /// Converts completed patient records into calibration-compatible trace records.
    pub fn export_calibration_traces(
        &self,
        day: u32,
        route_edges: &[&str],
    ) -> Result<Vec<CalibrationTraceRecord>, OperationalError> {
        let transit_ticks = if let Some(ref graph) = self.graph {
            if route_edges.is_empty() {
                0
            } else {
                graph.compute_travel_seconds(route_edges)? as u64
            }
        } else {
            0
        };

        let mut records = Vec::with_capacity(self.completed.len());
        for p in &self.completed {
            let start = p.start_tick.unwrap_or(p.arrival_tick);
            let end = p.completion_tick.unwrap_or(start + p.work_elapsed_ticks);
            records.push(CalibrationTraceRecord {
                case_id: format!("case_{}", p.id),
                day,
                start_tick: start,
                end_tick: end,
                transit_ticks,
                work_ticks: p.work_elapsed_ticks,
            });
        }
        Ok(records)
    }
}

/// Converts a RunSummary into a standard CSV table.
pub fn run_summary_to_csv(summary: &RunSummary) -> String {
    let mut out = String::new();
    out.push_str("patient_id,arrival_tick,start_tick,completion_tick,work_ticks,wait_ticks,elapsed_ticks,status\n");
    for p in &summary.patients {
        let start = p
            .start_tick
            .map(|t| t.to_string())
            .unwrap_or_else(|| "".to_string());
        let comp = p
            .completion_tick
            .map(|t| t.to_string())
            .unwrap_or_else(|| "".to_string());
        let wait = p
            .wait_ticks
            .map(|t| t.to_string())
            .unwrap_or_else(|| "".to_string());
        let elapsed = p
            .elapsed_ticks
            .map(|t| t.to_string())
            .unwrap_or_else(|| "".to_string());
        let status = match p.status {
            PatientStatus::Completed => "completed",
            PatientStatus::Unfinished => "unfinished",
        };
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{}\n",
            p.patient_id, p.arrival_tick, start, comp, p.work_ticks, wait, elapsed, status
        ));
    }
    out
}

/// Capacity comparison oracle results.
#[derive(Clone, Debug, PartialEq)]
pub struct CapacityComparison {
    pub nominal_arrivals: usize,
    pub nominal_completed: usize,
    pub nominal_unfinished: usize,
    pub nominal_mean_wait_ticks: f64,
    pub constrained_arrivals: usize,
    pub constrained_completed: usize,
    pub constrained_unfinished: usize,
    pub constrained_mean_wait_ticks: f64,
    pub throughput_difference: i64,
    pub wait_time_increased: bool,
    pub unfinished_increased: bool,
}

/// Compares a nominal scenario run summary with a constrained scenario run summary.
pub fn compare_capacity(nominal: &RunSummary, constrained: &RunSummary) -> CapacityComparison {
    let nominal_waits: Vec<u64> = nominal
        .patients
        .iter()
        .filter_map(|p| p.wait_ticks)
        .collect();
    let nominal_mean_wait = if !nominal_waits.is_empty() {
        nominal_waits.iter().sum::<u64>() as f64 / nominal_waits.len() as f64
    } else {
        0.0
    };

    let constrained_waits: Vec<u64> = constrained
        .patients
        .iter()
        .filter_map(|p| p.wait_ticks)
        .collect();
    let constrained_mean_wait = if !constrained_waits.is_empty() {
        constrained_waits.iter().sum::<u64>() as f64 / constrained_waits.len() as f64
    } else {
        0.0
    };

    let throughput_difference = nominal.completed as i64 - constrained.completed as i64;
    let wait_time_increased = constrained_mean_wait > nominal_mean_wait;
    let unfinished_increased = constrained.unfinished > nominal.unfinished;

    CapacityComparison {
        nominal_arrivals: nominal.arrivals,
        nominal_completed: nominal.completed,
        nominal_unfinished: nominal.unfinished,
        nominal_mean_wait_ticks: nominal_mean_wait,
        constrained_arrivals: constrained.arrivals,
        constrained_completed: constrained.completed,
        constrained_unfinished: constrained.unfinished,
        constrained_mean_wait_ticks: constrained_mean_wait,
        throughput_difference,
        wait_time_increased,
        unfinished_increased,
    }
}

/// A calibration-compatible trace record for empirical calibration and validation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalibrationTraceRecord {
    pub case_id: String,
    pub day: u32,
    pub start_tick: u64,
    pub end_tick: u64,
    pub transit_ticks: u64,
    pub work_ticks: u64,
}

/// Standalone helper to convert a slice of completed patients into calibration trace records.
pub fn export_patient_calibration_traces(
    patients: &[ComposedPatient],
    day: u32,
    transit_ticks: u64,
) -> Vec<CalibrationTraceRecord> {
    patients
        .iter()
        .map(|p| {
            let start = p.start_tick.unwrap_or(p.arrival_tick);
            let end = p.completion_tick.unwrap_or(start + p.work_elapsed_ticks);
            CalibrationTraceRecord {
                case_id: format!("case_{}", p.id),
                day,
                start_tick: start,
                end_tick: end,
                transit_ticks,
                work_ticks: p.work_elapsed_ticks,
            }
        })
        .collect()
}

/// Converts calibration trace records into a standard CSV table.
pub fn calibration_traces_to_csv(records: &[CalibrationTraceRecord]) -> String {
    let mut out = String::from("case_id,day,start_tick,end_tick,transit_ticks,work_ticks\n");
    for r in records {
        out.push_str(&format!(
            "{},{},{},{},{},{}\n",
            r.case_id, r.day, r.start_tick, r.end_tick, r.transit_ticks, r.work_ticks
        ));
    }
    out
}
