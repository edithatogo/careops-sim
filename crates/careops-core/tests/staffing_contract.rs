use careops_core::capacity::{Calendar, UtcInterval};
use careops_core::staffing::{
    BookingId, DutyKind, EscortChainId, StaffError, StaffId, StaffingRegistry,
};

fn interval(start: i64, end: i64) -> UtcInterval {
    UtcInterval::new(start, end).unwrap()
}

fn roster(start: i64, end: i64) -> Calendar {
    Calendar::new(vec![interval(start, end)]).unwrap()
}

#[test]
fn roster_coverage_and_breaks_gate_staff_assignments() {
    let mut staff = StaffingRegistry::default();
    staff
        .register_staff(
            StaffId::new(1),
            roster(0, 100),
            Calendar::new(vec![interval(20, 30)]).unwrap(),
            10,
        )
        .unwrap();

    assert_eq!(staff.is_available_at(StaffId::new(1), 19), Ok(true));
    assert_eq!(staff.is_available_at(StaffId::new(1), 20), Ok(false));
    assert_eq!(
        staff.assign(
            StaffId::new(1),
            BookingId::new(1),
            interval(20, 25),
            DutyKind::Service,
        ),
        Err(StaffError::BreakConflict)
    );
    assert_eq!(
        staff.assign(
            StaffId::new(1),
            BookingId::new(1),
            interval(95, 105),
            DutyKind::Service,
        ),
        Err(StaffError::OutsideRoster)
    );
    assert_eq!(staff.assignment_count(), 0);
}

#[test]
fn overlapping_assignments_for_one_staff_member_are_rejected_without_mutation() {
    let mut staff = StaffingRegistry::default();
    staff
        .register_staff(StaffId::new(2), roster(0, 100), Calendar::default(), 0)
        .unwrap();
    let existing = staff
        .assign(
            StaffId::new(2),
            BookingId::new(10),
            interval(10, 30),
            DutyKind::Service,
        )
        .unwrap();
    let before = staff.assignments().to_vec();

    assert_eq!(
        staff.assign(
            StaffId::new(2),
            BookingId::new(11),
            interval(29, 40),
            DutyKind::Escort {
                chain: EscortChainId::new(1),
            },
        ),
        Err(StaffError::DoubleBooked)
    );
    assert_eq!(staff.assignments(), before);
    assert_eq!(
        staff.assignment(existing).unwrap().booking,
        BookingId::new(10)
    );
}

#[test]
fn handover_reserves_distinct_staff_for_the_overlap_and_contiguous_following_duty() {
    let mut staff = StaffingRegistry::default();
    staff
        .register_staff(StaffId::new(3), roster(0, 100), Calendar::default(), 0)
        .unwrap();
    staff
        .register_staff(StaffId::new(4), roster(0, 100), Calendar::default(), 0)
        .unwrap();
    let booking = BookingId::new(20);
    let outgoing = staff
        .assign(StaffId::new(3), booking, interval(0, 50), DutyKind::Service)
        .unwrap();

    let (handover, continuation) = staff
        .handover(
            outgoing,
            StaffId::new(4),
            interval(40, 50),
            interval(50, 80),
        )
        .unwrap();
    assert_eq!(staff.assignment(handover).unwrap().kind, DutyKind::Handover);
    assert_eq!(
        staff.assignment(continuation).unwrap().kind,
        DutyKind::Service
    );
    assert_eq!(staff.assignment(handover).unwrap().staff, StaffId::new(4));
    assert_eq!(
        staff.assignment(continuation).unwrap().staff,
        StaffId::new(4)
    );
    assert_eq!(staff.assignment(handover).unwrap().booking, booking);
    assert_eq!(staff.assignment(continuation).unwrap().booking, booking);
    assert_eq!(staff.assignment_count(), 3);
}

#[test]
fn failed_handover_does_not_leave_a_partial_incoming_assignment() {
    let mut staff = StaffingRegistry::default();
    staff
        .register_staff(StaffId::new(5), roster(0, 100), Calendar::default(), 0)
        .unwrap();
    staff
        .register_staff(StaffId::new(6), roster(0, 100), Calendar::default(), 0)
        .unwrap();
    let outgoing = staff
        .assign(
            StaffId::new(5),
            BookingId::new(21),
            interval(0, 50),
            DutyKind::Service,
        )
        .unwrap();
    staff
        .assign(
            StaffId::new(6),
            BookingId::new(22),
            interval(45, 55),
            DutyKind::Service,
        )
        .unwrap();
    let before = staff.assignments().to_vec();

    assert_eq!(
        staff.handover(
            outgoing,
            StaffId::new(6),
            interval(40, 50),
            interval(50, 80)
        ),
        Err(StaffError::DoubleBooked)
    );
    assert_eq!(staff.assignments(), before);
}

#[test]
fn escort_continuation_keeps_the_same_staff_and_requires_adjacent_intervals() {
    let mut staff = StaffingRegistry::default();
    staff
        .register_staff(StaffId::new(7), roster(0, 100), Calendar::default(), 0)
        .unwrap();
    staff
        .register_staff(StaffId::new(8), roster(0, 100), Calendar::default(), 0)
        .unwrap();
    let first = staff
        .assign(
            StaffId::new(7),
            BookingId::new(30),
            interval(10, 20),
            DutyKind::Escort {
                chain: EscortChainId::new(30),
            },
        )
        .unwrap();

    let second = staff.continue_escort(first, interval(20, 30)).unwrap();
    assert_eq!(staff.assignment(second).unwrap().staff, StaffId::new(7));
    assert_eq!(
        staff.continue_escort(first, interval(21, 30)),
        Err(StaffError::NonContiguousEscort)
    );
    assert_eq!(
        staff.assign(
            StaffId::new(7),
            BookingId::new(31),
            interval(25, 35),
            DutyKind::Service,
        ),
        Err(StaffError::DoubleBooked)
    );
    assert_eq!(staff.assignment_count(), 2);
    assert_eq!(
        staff.assignment(second).unwrap().staff,
        staff.assignment(first).unwrap().staff
    );
}

#[test]
fn on_call_latency_is_checked_and_never_means_immediate_availability() {
    let mut staff = StaffingRegistry::default();
    staff
        .register_staff(
            StaffId::new(9),
            Calendar::default(),
            Calendar::default(),
            15,
        )
        .unwrap();
    assert_eq!(staff.call_in_available_at(StaffId::new(9), 100), Ok(115));
    assert_eq!(
        staff.call_in_available_at(StaffId::new(9), i64::MAX - 5),
        Err(StaffError::TimeOverflow)
    );
    assert_eq!(staff.is_available_at(StaffId::new(9), 100), Ok(false));
}

#[test]
fn non_interruptible_procedure_assignment_cannot_be_released_early() {
    let mut staff = StaffingRegistry::default();
    staff
        .register_staff(StaffId::new(10), roster(0, 100), Calendar::default(), 0)
        .unwrap();
    let assignment = staff
        .assign(
            StaffId::new(10),
            BookingId::new(40),
            interval(10, 30),
            DutyKind::Procedure {
                interruptible: false,
            },
        )
        .unwrap();
    let before = staff.assignment(assignment).unwrap().clone();
    assert_eq!(
        staff.release_at(assignment, 20),
        Err(StaffError::NonInterruptible)
    );
    assert_eq!(staff.assignment(assignment).unwrap(), &before);
}
