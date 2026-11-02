//! Neutral staff calendars and time-bounded assignments.
//!
//! This module accounts for individual availability and assignments. Staff
//! roles, clinical permissions, eligibility and site policy remain caller-owned.

use crate::capacity::{Calendar, UtcInterval};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct StaffId(u64);

impl StaffId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BookingId(u64);

impl BookingId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EscortChainId(u64);

impl EscortChainId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AssignmentId(u64);

impl AssignmentId {
    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DutyKind {
    Service,
    Handover,
    Escort { chain: EscortChainId },
    Procedure { interruptible: bool },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaffAssignment {
    pub id: AssignmentId,
    pub staff: StaffId,
    pub booking: BookingId,
    pub interval: UtcInterval,
    pub kind: DutyKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StaffError {
    DuplicateStaff,
    UnknownStaff,
    UnknownAssignment,
    BreakOutsideRoster,
    OutsideRoster,
    BreakConflict,
    DoubleBooked,
    InvalidHandover,
    NotEscort,
    NonContiguousEscort,
    InvalidReleaseTime,
    NonInterruptible,
    TimeOverflow,
    CounterOverflow,
}

#[derive(Clone, Debug)]
struct StaffMember {
    roster: Calendar,
    breaks: Calendar,
    call_in_latency_seconds: u64,
}

#[derive(Clone, Debug, Default)]
pub struct StaffingRegistry {
    members: BTreeMap<StaffId, StaffMember>,
    assignments: BTreeMap<AssignmentId, StaffAssignment>,
    next_assignment: u64,
}

impl StaffingRegistry {
    pub fn register_staff(
        &mut self,
        id: StaffId,
        roster: Calendar,
        breaks: Calendar,
        call_in_latency_seconds: u64,
    ) -> Result<(), StaffError> {
        if self.members.contains_key(&id) {
            return Err(StaffError::DuplicateStaff);
        }
        if breaks
            .intervals()
            .iter()
            .any(|break_interval| !calendar_covers(&roster, *break_interval))
        {
            return Err(StaffError::BreakOutsideRoster);
        }
        self.members.insert(
            id,
            StaffMember {
                roster,
                breaks,
                call_in_latency_seconds,
            },
        );
        Ok(())
    }

    pub fn is_available_at(&self, staff: StaffId, utc_seconds: i64) -> Result<bool, StaffError> {
        let member = self.members.get(&staff).ok_or(StaffError::UnknownStaff)?;
        if !member.roster.contains(utc_seconds) || member.breaks.contains(utc_seconds) {
            return Ok(false);
        }
        Ok(!self.assignments.values().any(|assignment| {
            assignment.staff == staff && assignment.interval.contains(utc_seconds)
        }))
    }

    /// Return the earliest time call-in latency has elapsed. This is a
    /// readiness lower bound, not proof that a caller's clinical or role
    /// requirements are met.
    pub fn call_in_available_at(
        &self,
        staff: StaffId,
        requested_at: i64,
    ) -> Result<i64, StaffError> {
        let member = self.members.get(&staff).ok_or(StaffError::UnknownStaff)?;
        let delay =
            i64::try_from(member.call_in_latency_seconds).map_err(|_| StaffError::TimeOverflow)?;
        requested_at
            .checked_add(delay)
            .ok_or(StaffError::TimeOverflow)
    }

    pub fn assign(
        &mut self,
        staff: StaffId,
        booking: BookingId,
        interval: UtcInterval,
        kind: DutyKind,
    ) -> Result<AssignmentId, StaffError> {
        self.check_slot(staff, interval)?;
        self.insert_assignment(staff, booking, interval, kind)
    }

    /// Create the incoming handover overlap and following duty as one atomic
    /// operation. The outgoing assignment must end at handover completion.
    pub fn handover(
        &mut self,
        outgoing: AssignmentId,
        incoming_staff: StaffId,
        overlap: UtcInterval,
        continuation: UtcInterval,
    ) -> Result<(AssignmentId, AssignmentId), StaffError> {
        let outgoing_assignment = self
            .assignments
            .get(&outgoing)
            .cloned()
            .ok_or(StaffError::UnknownAssignment)?;
        if outgoing_assignment.staff == incoming_staff
            || matches!(
                outgoing_assignment.kind,
                DutyKind::Procedure {
                    interruptible: false
                }
            )
            || outgoing_assignment.interval.end() != overlap.end()
            || overlap.start() < outgoing_assignment.interval.start()
            || continuation.start() != overlap.end()
            || continuation.end() <= continuation.start()
        {
            return Err(StaffError::InvalidHandover);
        }

        // Validate both requested windows before mutating either one.
        self.check_slot(incoming_staff, overlap)?;
        self.check_slot(incoming_staff, continuation)?;
        let handover_id = self.next_id()?;
        let continuation_id = AssignmentId(
            handover_id
                .0
                .checked_add(1)
                .ok_or(StaffError::CounterOverflow)?,
        );
        self.next_assignment = continuation_id.0;
        self.assignments.insert(
            handover_id,
            StaffAssignment {
                id: handover_id,
                staff: incoming_staff,
                booking: outgoing_assignment.booking,
                interval: overlap,
                kind: DutyKind::Handover,
            },
        );
        self.assignments.insert(
            continuation_id,
            StaffAssignment {
                id: continuation_id,
                staff: incoming_staff,
                booking: outgoing_assignment.booking,
                interval: continuation,
                kind: outgoing_assignment.kind,
            },
        );
        Ok((handover_id, continuation_id))
    }

    /// Continue an escort with the same staff member over the immediately
    /// adjacent interval. The original claim remains recorded independently.
    pub fn continue_escort(
        &mut self,
        prior: AssignmentId,
        next_interval: UtcInterval,
    ) -> Result<AssignmentId, StaffError> {
        let prior_assignment = self
            .assignments
            .get(&prior)
            .cloned()
            .ok_or(StaffError::UnknownAssignment)?;
        let chain = match prior_assignment.kind {
            DutyKind::Escort { chain } => chain,
            _ => return Err(StaffError::NotEscort),
        };
        if prior_assignment.interval.end() != next_interval.start() {
            return Err(StaffError::NonContiguousEscort);
        }
        self.check_slot(prior_assignment.staff, next_interval)?;
        self.insert_assignment(
            prior_assignment.staff,
            prior_assignment.booking,
            next_interval,
            DutyKind::Escort { chain },
        )
    }

    /// Release an assignment early. Declared non-interruptible procedure phases
    /// retain their staff claim through their scheduled end.
    pub fn release_at(&mut self, assignment: AssignmentId, at: i64) -> Result<(), StaffError> {
        let current = self
            .assignments
            .get(&assignment)
            .cloned()
            .ok_or(StaffError::UnknownAssignment)?;
        if at < current.interval.start() || at >= current.interval.end() {
            return Err(StaffError::InvalidReleaseTime);
        }
        if matches!(
            current.kind,
            DutyKind::Procedure {
                interruptible: false
            }
        ) {
            return Err(StaffError::NonInterruptible);
        }
        if at == current.interval.start() {
            self.assignments.remove(&assignment);
        } else {
            self.assignments.get_mut(&assignment).unwrap().interval =
                UtcInterval::new(current.interval.start(), at)
                    .map_err(|_| StaffError::InvalidReleaseTime)?;
        }
        Ok(())
    }

    pub fn assignment(&self, id: AssignmentId) -> Option<&StaffAssignment> {
        self.assignments.get(&id)
    }

    pub fn assignments(&self) -> Vec<StaffAssignment> {
        self.assignments.values().cloned().collect()
    }

    pub fn assignment_count(&self) -> usize {
        self.assignments.len()
    }

    fn check_slot(&self, staff: StaffId, interval: UtcInterval) -> Result<(), StaffError> {
        let member = self.members.get(&staff).ok_or(StaffError::UnknownStaff)?;
        if !calendar_covers(&member.roster, interval) {
            return Err(StaffError::OutsideRoster);
        }
        if member
            .breaks
            .intervals()
            .iter()
            .any(|break_interval| overlaps(*break_interval, interval))
        {
            return Err(StaffError::BreakConflict);
        }
        if self
            .assignments
            .values()
            .any(|assignment| assignment.staff == staff && overlaps(assignment.interval, interval))
        {
            return Err(StaffError::DoubleBooked);
        }
        Ok(())
    }

    fn insert_assignment(
        &mut self,
        staff: StaffId,
        booking: BookingId,
        interval: UtcInterval,
        kind: DutyKind,
    ) -> Result<AssignmentId, StaffError> {
        let id = self.next_id()?;
        self.next_assignment = id.0;
        self.assignments.insert(
            id,
            StaffAssignment {
                id,
                staff,
                booking,
                interval,
                kind,
            },
        );
        Ok(id)
    }

    fn next_id(&self) -> Result<AssignmentId, StaffError> {
        self.next_assignment
            .checked_add(1)
            .map(AssignmentId)
            .ok_or(StaffError::CounterOverflow)
    }
}

fn overlaps(left: UtcInterval, right: UtcInterval) -> bool {
    left.start() < right.end() && right.start() < left.end()
}

fn calendar_covers(calendar: &Calendar, requested: UtcInterval) -> bool {
    let mut cursor = requested.start();
    for window in calendar.intervals() {
        if window.end() <= cursor {
            continue;
        }
        if window.start() > cursor {
            return false;
        }
        cursor = window.end();
        if cursor >= requested.end() {
            return true;
        }
    }
    false
}
