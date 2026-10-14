//! Minimum E2.1/E2.2 operational constraints for synthetic ED staff and spaces.
//!
//! Shift reductions follow the Kairos shrink contract: a reduction that would
//! sit below in-flight claims is rejected, and active work is not evicted.
//! Space occupancy, cleaning, and reservation are separate from staff claims.
//! One bay holds one patient. Diagnostics and boarding retain that hold.
//!
//! E2.2 dispatch ranks eligible tasks by clinical urgency (lower acuity first),
//! FIFO within acuity, then stable patient and task identifiers.

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

/// Select the next dispatchable task for `role_id`.
///
/// Returns `None` when no staff are claimable, the queue is empty, or no candidate
/// matches declared role/task/zone/location eligibility.
pub fn select_dispatch_candidate<'a>(
    candidates: &'a [TaskCandidate],
    shift: &StaffShift,
    role_id: &str,
    eligibilities: &[Eligibility],
) -> Option<&'a TaskCandidate> {
    if shift.claimable() == 0 || candidates.is_empty() {
        return None;
    }
    candidates
        .iter()
        .filter(|candidate| {
            role_is_eligible(
                eligibilities,
                role_id,
                &candidate.task_class_id,
                &candidate.zone_id,
                &candidate.location_id,
            )
        })
        .min_by(|left, right| {
            left.acuity
                .cmp(&right.acuity)
                .then_with(|| left.enqueue_tick.cmp(&right.enqueue_tick))
                .then_with(|| left.patient_id.cmp(&right.patient_id))
                .then_with(|| left.task_id.cmp(&right.task_id))
        })
}
