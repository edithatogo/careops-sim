//! Minimum E2.1 operational constraints for synthetic ED staff and spaces.
//!
//! Shift reductions follow the Kairos shrink contract: a reduction that would
//! sit below in-flight claims is rejected, and active work is not evicted.
//! Space occupancy, cleaning, and reservation are separate from staff claims.
//! One bay holds one patient. Diagnostics and boarding retain that hold.

use std::fmt::{Display, Formatter};

use crate::Eligibility;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperationalError {
    Ineligible,
    Unavailable,
    CapacityInUse,
    NotAssigned,
    Accounting,
    SpaceUnavailable,
    ReservedForOther,
    CleaningInProgress,
    DiagnosticBlocked,
    BoardingRetained,
    AtomicMultiResourceUnsupported,
    UnsupportedMultiRole,
    RouteUnreachable,
    EdgeInaccessible,
}

impl Display for OperationalError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::Ineligible => "staff role is not eligible for this task and zone",
            Self::Unavailable => "no on-duty staff are free to claim",
            Self::CapacityInUse => "shift shrink would drop in-flight work",
            Self::NotAssigned => "handover requires an active assignment",
            Self::Accounting => "elapsed work exceeds the recorded duration",
            Self::SpaceUnavailable => "treatment space is not available",
            Self::ReservedForOther => "treatment space is reserved for another patient",
            Self::CleaningInProgress => "treatment space is still being cleaned",
            Self::DiagnosticBlocked => "blocked diagnostics retain the treatment space",
            Self::BoardingRetained => "boarding retains the treatment space until transfer",
            Self::AtomicMultiResourceUnsupported => {
                "atomic multi-resource acquisition is unsupported in v1; use phased sequential claims"
            }
            Self::UnsupportedMultiRole => {
                "multi-person procedure requires dedicated composite team approximation"
            }
            Self::RouteUnreachable => "target location is unreachable in directed movement graph",
            Self::EdgeInaccessible => "movement edge is currently marked inaccessible",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for OperationalError {}

/// Scheduled staff for one role. `on_break` and `active` never exceed `scheduled`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StaffShift {
    scheduled: u64,
    on_break: u64,
    active: u64,
}

impl StaffShift {
    pub fn new(scheduled: u64) -> Self {
        Self {
            scheduled,
            on_break: 0,
            active: 0,
        }
    }

    pub fn scheduled(&self) -> u64 {
        self.scheduled
    }

    pub fn on_break(&self) -> u64 {
        self.on_break
    }

    pub fn active(&self) -> u64 {
        self.active
    }

    pub fn claimable(&self) -> u64 {
        self.scheduled
            .saturating_sub(self.on_break)
            .saturating_sub(self.active)
    }

    /// Put idle staff on break. Busy staff stay assigned; this does not evict them.
    pub fn begin_break(&mut self, count: u64) -> Result<(), OperationalError> {
        if count > self.claimable() {
            return Err(OperationalError::CapacityInUse);
        }
        self.on_break += count;
        Ok(())
    }

    pub fn end_break(&mut self, count: u64) -> Result<(), OperationalError> {
        if count > self.on_break {
            return Err(OperationalError::Unavailable);
        }
        self.on_break -= count;
        Ok(())
    }

    /// Reduce the roster. Rejected while in-flight claims or breaks would exceed it.
    pub fn shrink_scheduled(&mut self, new_scheduled: u64) -> Result<(), OperationalError> {
        if self.active + self.on_break > new_scheduled {
            return Err(OperationalError::CapacityInUse);
        }
        self.scheduled = new_scheduled;
        Ok(())
    }

    pub fn release_assignment(&mut self) -> Result<(), OperationalError> {
        if self.active == 0 {
            return Err(OperationalError::NotAssigned);
        }
        self.active -= 1;
        Ok(())
    }

    fn take_claim(&mut self) -> Result<(), OperationalError> {
        if self.claimable() == 0 {
            return Err(OperationalError::Unavailable);
        }
        self.active += 1;
        Ok(())
    }
}

pub fn role_is_eligible(
    rows: &[Eligibility],
    role_id: &str,
    task_class_id: &str,
    zone_id: &str,
    location_id: &str,
) -> bool {
    rows.iter().any(|row| {
        row.staff_role_id == role_id
            && row.task_class_id == task_class_id
            && row.zone_id == zone_id
            && row
                .location_id
                .as_deref()
                .is_none_or(|location| location == location_id)
    })
}

pub fn claim_task(
    shift: &mut StaffShift,
    rows: &[Eligibility],
    role_id: &str,
    task_class_id: &str,
    zone_id: &str,
    location_id: &str,
) -> Result<(), OperationalError> {
    if !role_is_eligible(rows, role_id, task_class_id, zone_id, location_id) {
        return Err(OperationalError::Ineligible);
    }
    shift.take_claim()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeldWork {
    pub elapsed_ticks: u64,
    pub total_ticks: u64,
    pub task_class_id: String,
    pub zone_id: String,
    pub location_id: String,
}

/// Move one active assignment to another eligible, free staff member.
/// Elapsed and remaining work stay on the same account.
pub fn handover(
    from: &mut StaffShift,
    to: &mut StaffShift,
    work: &HeldWork,
    from_role: &str,
    to_role: &str,
    rows: &[Eligibility],
) -> Result<(), OperationalError> {
    if work.elapsed_ticks > work.total_ticks {
        return Err(OperationalError::Accounting);
    }
    if from.active == 0 {
        return Err(OperationalError::NotAssigned);
    }
    if !role_is_eligible(
        rows,
        from_role,
        &work.task_class_id,
        &work.zone_id,
        &work.location_id,
    ) || !role_is_eligible(
        rows,
        to_role,
        &work.task_class_id,
        &work.zone_id,
        &work.location_id,
    ) {
        return Err(OperationalError::Ineligible);
    }
    if to.claimable() == 0 {
        return Err(OperationalError::Unavailable);
    }
    from.active -= 1;
    to.active += 1;
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreemptConsent {
    NonPreemptible,
    Preemptible,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorkAccount {
    pub elapsed_ticks: u64,
    pub total_ticks: u64,
}

impl WorkAccount {
    pub fn remaining_ticks(self) -> Result<u64, OperationalError> {
        self.total_ticks
            .checked_sub(self.elapsed_ticks)
            .ok_or(OperationalError::Accounting)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Preemption {
    pub yielded: bool,
    pub retained_ticks: u64,
}

/// Urgent preemption needs a strictly higher acuity and explicit consent.
/// Lower ATS numbers are more urgent. Elapsed work is kept either way.
pub fn preempt(
    active: &WorkAccount,
    active_acuity: u8,
    consent: PreemptConsent,
    incoming_acuity: u8,
) -> Result<Preemption, OperationalError> {
    let retained_ticks = active.remaining_ticks()?;
    let clinical_priority = incoming_acuity < active_acuity;
    let consent_granted = consent == PreemptConsent::Preemptible;
    Ok(Preemption {
        yielded: clinical_priority && consent_granted,
        retained_ticks,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BayPhase {
    Available,
    Reserved(u64),
    Occupied {
        patient_id: u64,
        diagnostic_blocked: bool,
        boarding: bool,
    },
    Cleaning {
        ready_tick: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreatmentBay {
    phase: BayPhase,
}

impl TreatmentBay {
    pub fn available() -> Self {
        Self {
            phase: BayPhase::Available,
        }
    }

    pub fn phase(&self) -> BayPhase {
        self.phase
    }

    pub fn reserve(&mut self, patient_id: u64) -> Result<(), OperationalError> {
        match self.phase {
            BayPhase::Available => {
                self.phase = BayPhase::Reserved(patient_id);
                Ok(())
            }
            BayPhase::Reserved(_) | BayPhase::Occupied { .. } => {
                Err(OperationalError::SpaceUnavailable)
            }
            BayPhase::Cleaning { .. } => Err(OperationalError::CleaningInProgress),
        }
    }

    pub fn occupy(&mut self, patient_id: u64) -> Result<(), OperationalError> {
        match self.phase {
            BayPhase::Available => {
                self.phase = occupied(patient_id);
                Ok(())
            }
            BayPhase::Reserved(reserved) if reserved == patient_id => {
                self.phase = occupied(patient_id);
                Ok(())
            }
            BayPhase::Reserved(_) => Err(OperationalError::ReservedForOther),
            BayPhase::Occupied { .. } => Err(OperationalError::SpaceUnavailable),
            BayPhase::Cleaning { .. } => Err(OperationalError::CleaningInProgress),
        }
    }

    pub fn block_for_diagnostics(&mut self) -> Result<(), OperationalError> {
        match self.phase {
            BayPhase::Occupied {
                patient_id,
                boarding,
                ..
            } => {
                self.phase = BayPhase::Occupied {
                    patient_id,
                    diagnostic_blocked: true,
                    boarding,
                };
                Ok(())
            }
            _ => Err(OperationalError::SpaceUnavailable),
        }
    }

    pub fn clear_diagnostics(&mut self) -> Result<(), OperationalError> {
        match self.phase {
            BayPhase::Occupied {
                patient_id,
                diagnostic_blocked: true,
                boarding,
            } => {
                self.phase = BayPhase::Occupied {
                    patient_id,
                    diagnostic_blocked: false,
                    boarding,
                };
                Ok(())
            }
            BayPhase::Occupied { .. } => Err(OperationalError::SpaceUnavailable),
            _ => Err(OperationalError::SpaceUnavailable),
        }
    }

    pub fn begin_boarding(&mut self) -> Result<(), OperationalError> {
        match self.phase {
            BayPhase::Occupied {
                patient_id,
                diagnostic_blocked,
                boarding: false,
            } => {
                self.phase = BayPhase::Occupied {
                    patient_id,
                    diagnostic_blocked,
                    boarding: true,
                };
                Ok(())
            }
            BayPhase::Occupied { boarding: true, .. } => Err(OperationalError::BoardingRetained),
            _ => Err(OperationalError::SpaceUnavailable),
        }
    }

    pub fn vacate_for_cleaning(&mut self, ready_tick: u64) -> Result<(), OperationalError> {
        match self.phase {
            BayPhase::Occupied {
                diagnostic_blocked: true,
                ..
            } => Err(OperationalError::DiagnosticBlocked),
            BayPhase::Occupied { boarding: true, .. } => Err(OperationalError::BoardingRetained),
            BayPhase::Occupied { .. } => {
                self.phase = BayPhase::Cleaning { ready_tick };
                Ok(())
            }
            _ => Err(OperationalError::SpaceUnavailable),
        }
    }

    pub fn accept_transfer(&mut self, ready_tick: u64) -> Result<(), OperationalError> {
        match self.phase {
            BayPhase::Occupied {
                boarding: true,
                diagnostic_blocked: false,
                ..
            } => {
                self.phase = BayPhase::Cleaning { ready_tick };
                Ok(())
            }
            BayPhase::Occupied {
                diagnostic_blocked: true,
                ..
            } => Err(OperationalError::DiagnosticBlocked),
            BayPhase::Occupied { .. } => Err(OperationalError::SpaceUnavailable),
            _ => Err(OperationalError::SpaceUnavailable),
        }
    }

    pub fn finish_cleaning(&mut self, current_tick: u64) -> Result<(), OperationalError> {
        match self.phase {
            BayPhase::Cleaning { ready_tick } if current_tick >= ready_tick => {
                self.phase = BayPhase::Available;
                Ok(())
            }
            BayPhase::Cleaning { .. } => Err(OperationalError::CleaningInProgress),
            _ => Err(OperationalError::SpaceUnavailable),
        }
    }
}

fn occupied(patient_id: u64) -> BayPhase {
    BayPhase::Occupied {
        patient_id,
        diagnostic_blocked: false,
        boarding: false,
    }
}

/// Fixed cubicles. Occupied, reserved, and cleaning bays are not free capacity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ward {
    bays: Vec<TreatmentBay>,
}

impl Ward {
    pub fn new(cubicles: usize) -> Self {
        Self {
            bays: (0..cubicles).map(|_| TreatmentBay::available()).collect(),
        }
    }

    pub fn len(&self) -> usize {
        self.bays.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bays.is_empty()
    }

    pub fn bay(&self, index: usize) -> Option<&TreatmentBay> {
        self.bays.get(index)
    }

    pub fn bay_mut(&mut self, index: usize) -> Option<&mut TreatmentBay> {
        self.bays.get_mut(index)
    }

    pub fn occupied_patients(&self) -> usize {
        self.bays
            .iter()
            .filter(|bay| matches!(bay.phase, BayPhase::Occupied { .. }))
            .count()
    }

    pub fn free_bays(&self) -> usize {
        self.bays
            .iter()
            .filter(|bay| bay.phase == BayPhase::Available)
            .count()
    }

    pub fn place(&mut self, patient_id: u64) -> Result<usize, OperationalError> {
        let index = self
            .bays
            .iter()
            .position(|bay| bay.phase == BayPhase::Available)
            .ok_or(OperationalError::SpaceUnavailable)?;
        self.bays[index].occupy(patient_id)?;
        Ok(index)
    }
}

/// A queued patient task eligible for staff assignment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskCandidate {
    pub task_id: String,
    pub patient_id: u64,
    pub acuity: u8,
    pub enqueue_tick: u64,
    pub task_class_id: String,
    pub zone_id: String,
    pub location_id: String,
}

/// Dispatches the next task according to clinical urgency (acuity ascending, e.g. ATS 1 before ATS 5),
/// FIFO within the same acuity (enqueue_tick ascending), and deterministic tie-breaking (patient_id, task_id).
pub fn select_dispatch_candidate<'a>(
    candidates: &'a [TaskCandidate],
    shift: &StaffShift,
    role_id: &str,
    eligibilities: &[Eligibility],
) -> Option<&'a TaskCandidate> {
    if shift.claimable() == 0 {
        return None;
    }
    candidates
        .iter()
        .filter(|c| {
            role_is_eligible(
                eligibilities,
                role_id,
                &c.task_class_id,
                &c.zone_id,
                &c.location_id,
            )
        })
        .min_by(|a, b| {
            a.acuity
                .cmp(&b.acuity)
                .then_with(|| a.enqueue_tick.cmp(&b.enqueue_tick))
                .then_with(|| a.patient_id.cmp(&b.patient_id))
                .then_with(|| a.task_id.cmp(&b.task_id))
        })
}

/// Target identifiers for claiming a task.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskClaimTarget<'a> {
    pub role_id: &'a str,
    pub task_class_id: &'a str,
    pub zone_id: &'a str,
    pub location_id: &'a str,
}

/// Atomic multi-resource acquisition (e.g. implicitly granting bed + staff together)
/// is unsupported in v1; resource claims must be sequenced as explicit phases.
pub fn claim_atomic_multi_resource() -> Result<(), OperationalError> {
    Err(OperationalError::AtomicMultiResourceUnsupported)
}

/// Phased acquisition: reserve or occupy space first, then acquire staff assignment.
pub fn execute_phased_claim(
    bay: &mut TreatmentBay,
    patient_id: u64,
    shift: &mut StaffShift,
    eligibilities: &[Eligibility],
    target: TaskClaimTarget<'_>,
) -> Result<(), OperationalError> {
    bay.occupy(patient_id)?;
    if let Err(err) = claim_task(
        shift,
        eligibilities,
        target.role_id,
        target.task_class_id,
        target.zone_id,
        target.location_id,
    ) {
        let _ = bay.vacate_for_cleaning(0);
        return Err(err);
    }
    Ok(())
}

/// Simulation execution fidelity level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionFidelity {
    Macro,
    Micro,
}

/// A movement edge in the physical/spatial transit graph.
#[derive(Clone, Debug, PartialEq)]
pub struct MovementEdge {
    pub edge_id: String,
    pub from: String,
    pub to: String,
    pub distance_m: f64,
    pub speed_m_per_s: f64,
    pub accessible: bool,
}

/// Directed routing graph for spatial micro-simulation.
#[derive(Clone, Debug, PartialEq)]
pub struct SyntheticGraph {
    pub edges: Vec<MovementEdge>,
}

impl SyntheticGraph {
    pub fn new(edges: Vec<MovementEdge>) -> Self {
        Self { edges }
    }

    pub fn compute_travel_seconds(&self, edge_ids: &[&str]) -> Result<f64, OperationalError> {
        let mut total = 0.0;
        for edge_id in edge_ids {
            let edge = self
                .edges
                .iter()
                .find(|e| e.edge_id == *edge_id)
                .ok_or(OperationalError::RouteUnreachable)?;
            if !edge.accessible {
                return Err(OperationalError::EdgeInaccessible);
            }
            if edge.speed_m_per_s <= 0.0 {
                return Err(OperationalError::Accounting);
            }
            total += edge.distance_m / edge.speed_m_per_s;
        }
        Ok(total)
    }
}

/// Evaluates execution duration under Macro vs Micro fidelity.
/// In Macro mode: no hidden transit cost; total duration is active work time.
/// In Micro mode: total duration is active work + travel time along the route.
pub fn calculate_execution_duration(
    fidelity: ExecutionFidelity,
    active_work_seconds: f64,
    graph: Option<&SyntheticGraph>,
    edge_ids: &[&str],
) -> Result<f64, OperationalError> {
    match fidelity {
        ExecutionFidelity::Macro => Ok(active_work_seconds),
        ExecutionFidelity::Micro => {
            let g = graph.ok_or(OperationalError::RouteUnreachable)?;
            let travel = g.compute_travel_seconds(edge_ids)?;
            Ok(active_work_seconds + travel)
        }
    }
}

/// Clinical procedure categories.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcedureKind {
    RoutineAssessment,
    DiagnosticScan,
    MedicationAdministration,
    ProceduralSedation,
    Resuscitation,
    AssistedTransfer,
}

/// Enforces multi-role procedural staffing rules.
/// Multi-person sedation, resuscitation, and assisted transfer require an explicit dedicated
/// composite-team approximation; raw concurrent uncoordinated multi-role claims are rejected.
pub fn validate_procedure_staffing(
    procedure: ProcedureKind,
    is_composite_team: bool,
) -> Result<(), OperationalError> {
    match procedure {
        ProcedureKind::RoutineAssessment
        | ProcedureKind::DiagnosticScan
        | ProcedureKind::MedicationAdministration => Ok(()),
        ProcedureKind::ProceduralSedation
        | ProcedureKind::Resuscitation
        | ProcedureKind::AssistedTransfer => {
            if is_composite_team {
                Ok(())
            } else {
                Err(OperationalError::UnsupportedMultiRole)
            }
        }
    }
}
