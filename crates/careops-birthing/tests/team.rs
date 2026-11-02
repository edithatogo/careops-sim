use careops_birthing::{spaces::*, team::*};
use careops_core::capacity::{Calendar, CapabilitySet, UtcInterval};
use careops_core::staffing::{BookingId, DutyKind, EscortChainId, StaffId};

fn window(a: i64, b: i64) -> UtcInterval {
    UtcInterval::new(a, b).unwrap()
}
fn calendar(a: i64, b: i64) -> Calendar {
    Calendar::new(vec![window(a, b)]).unwrap()
}
fn caps(roles: &[&str]) -> CapabilitySet {
    CapabilitySet::new(roles.iter().copied()).unwrap()
}
fn member(id: u64, roles: &[&str], a: i64, b: i64) -> MemberProfile {
    MemberProfile {
        staff: StaffId::new(id),
        revision: "synthetic:v1".into(),
        roles: caps(roles),
        roster: calendar(a, b),
        unavailable: Calendar::default(),
        overtime_per_shift_seconds: 0,
        call_in_latency_seconds: 0,
        call_windows: None,
    }
}
fn request(booking: u64, patients: u32, roles: &[(&str, u32)], a: i64, b: i64) -> TeamRequest {
    TeamRequest {
        booking: BookingId::new(booking),
        patients,
        interval: window(a, b),
        kind: DutyKind::Service,
        profile: RatioProfile {
            revision: "synthetic:v1".into(),
            cohort_unit: "synthetic_person".into(),
            patients_per_staff: roles
                .iter()
                .map(|(role, n)| (role.to_string(), *n))
                .collect(),
        },
    }
}

#[test]
fn flexible_role_matching_ratios_and_shared_contention_are_deterministic() {
    let profiles = vec![
        member(1, &["role_a", "role_z"], 0, 100),
        member(2, &["role_a"], 0, 100),
    ];
    let mut team = TeamAllocator::new(profiles.clone()).unwrap();
    let req = request(1, 1, &[("role_a", 1), ("role_z", 1)], 0, 10);
    let receipt = team.allocate(req.clone()).unwrap();
    assert_eq!(receipt.assignments[0].staff, StaffId::new(2));
    assert_eq!(receipt.assignments[1].staff, StaffId::new(1));
    assert_eq!(team.allocate(req.clone()).unwrap(), receipt);
    assert!(team
        .allocate(request(2, 1, &[("role_a", 1)], 0, 10))
        .is_err());
    let mut reversed = profiles;
    reversed.reverse();
    assert_eq!(
        TeamAllocator::new(reversed).unwrap().allocate(req).unwrap(),
        receipt
    );
    let mut ratios = TeamAllocator::new(vec![
        member(1, &["staff"], 0, 100),
        member(2, &["staff"], 0, 100),
    ])
    .unwrap();
    assert_eq!(
        ratios
            .allocate(request(1, 3, &[("staff", 2)], 0, 10))
            .unwrap()
            .assignments
            .len(),
        2
    );
    assert_eq!(ratios.registry().assignment_count(), 2);
}

#[test]
fn whole_interval_break_absence_overtime_and_call_readiness_are_enforced() {
    let mut profile = member(1, &["staff"], 0, 100);
    profile.unavailable = Calendar::new(vec![window(20, 30), window(40, 60)]).unwrap();
    profile.overtime_per_shift_seconds = 10;
    let mut team = TeamAllocator::new(vec![profile]).unwrap();
    for (a, b) in [(19, 21), (45, 50), (110, 111)] {
        assert!(team.allocate(request(1, 1, &[("staff", 1)], a, b)).is_err());
        assert_eq!(team.registry().assignment_count(), 0);
    }
    team.allocate(request(1, 1, &[("staff", 1)], 100, 110))
        .unwrap();
    let mut called = member(1, &["staff"], 0, 100);
    called.call_in_latency_seconds = 20;
    called.call_windows = Some(vec![CallWindow {
        requested_at: 10,
        duty: window(0, 100),
    }]);
    let mut call_team = TeamAllocator::new(vec![called]).unwrap();
    assert!(call_team
        .allocate(request(1, 1, &[("staff", 1)], 29, 40))
        .is_err());
    call_team
        .allocate(request(1, 1, &[("staff", 1)], 30, 40))
        .unwrap();
}

#[test]
fn static_space_team_approximation_rolls_back_partial_admission_and_rejects_rebinding() {
    let make_space = |end| {
        SpacePool::new(vec![SpaceProfile {
            id: 1,
            revision: "synthetic:v1".into(),
            kind: SpaceKind::Ldr,
            capabilities: caps(&["ldr"]),
            open: calendar(0, end),
            staffed: Some(calendar(0, 100)),
            equipped: Some(true),
        }])
        .unwrap()
    };
    let mut team = TeamAllocator::new(vec![member(1, &["staff"], 0, 100)]).unwrap();
    let mut closed = make_space(5);
    let req = request(1, 1, &[("staff", 1)], 10, 20);
    assert!(team
        .admit(&mut closed, 10, caps(&["ldr"]), req.clone())
        .is_err());
    assert_eq!(team.registry().assignment_count(), 0);
    assert_eq!(closed.snapshot(1).unwrap().reserved, 0);
    let mut spaces = make_space(100);
    let receipt = team
        .admit(&mut spaces, 10, caps(&["ldr"]), req.clone())
        .unwrap();
    let mut unrelated = make_space(100);
    assert!(team
        .admit(&mut unrelated, 10, caps(&["ldr"]), req.clone())
        .is_err());
    assert_eq!(unrelated.snapshot(1).unwrap().occupied, 0);
    assert_eq!(
        team.admit(&mut spaces, 10, caps(&["ldr"]), req.clone())
            .unwrap(),
        receipt
    );
    assert!(team.admit(&mut spaces, 11, caps(&["ldr"]), req).is_err());
    assert_eq!(team.registry().assignment_count(), 1);
    assert_eq!(spaces.snapshot(1).unwrap().occupied, 1);
}

#[test]
fn handover_preserves_role_responsibility_and_escort_continuity() {
    let mut team = TeamAllocator::new(vec![
        member(1, &["staff"], 0, 50),
        member(2, &["staff"], 40, 100),
        member(3, &["other"], 40, 100),
    ])
    .unwrap();
    let original = team
        .allocate(request(1, 1, &[("staff", 1)], 0, 50))
        .unwrap()
        .assignments[0]
        .assignment;
    assert!(team
        .handover(original, StaffId::new(3), window(45, 50), window(50, 80))
        .is_err());
    assert_eq!(team.registry().assignment_count(), 1);
    let result = team
        .handover(original, StaffId::new(2), window(45, 50), window(50, 80))
        .unwrap();
    assert_eq!(
        team.handover(original, StaffId::new(2), window(45, 50), window(50, 80))
            .unwrap(),
        result
    );
    assert_eq!(team.registry().assignment_count(), 3);
    assert!(team.release_at(original, 46).is_err());
    assert!(team.release_at(result.0, 46).is_err());
    assert!(team
        .allocate(request(2, 1, &[("staff", 1)], 60, 70))
        .is_err());
    for (at, expected) in [(49, 2), (50, 1), (79, 1)] {
        assert_eq!(
            team.registry()
                .assignments()
                .iter()
                .filter(|a| a.interval.contains(at))
                .count(),
            expected
        );
    }
    let mut escort = TeamAllocator::new(vec![
        member(1, &["escort"], 0, 100),
        member(2, &["escort"], 0, 100),
    ])
    .unwrap();
    let mut req = request(1, 1, &[("escort", 1)], 0, 10);
    req.kind = DutyKind::Escort {
        chain: EscortChainId::new(8),
    };
    let prior = escort.allocate(req).unwrap().assignments[0].assignment;
    assert!(escort
        .handover(prior, StaffId::new(2), window(5, 10), window(10, 20))
        .is_err());
    assert!(escort.release_at(prior, 5).is_err());
    let next = escort.continue_escort(prior, window(10, 20)).unwrap();
    assert_eq!(escort.continue_escort(prior, window(10, 20)).unwrap(), next);
    assert!(escort.continue_escort(next, window(21, 30)).is_err());
    assert_eq!(
        escort.registry().assignment(next).unwrap().staff,
        StaffId::new(1)
    );
    assert_eq!(escort.registry().assignment_count(), 2);
}

#[test]
fn unsafe_procedure_interruption_and_invalid_profiles_fail_closed() {
    let mut team = TeamAllocator::new(vec![
        member(1, &["staff"], 0, 50),
        member(2, &["staff"], 40, 100),
    ])
    .unwrap();
    let mut req = request(1, 1, &[("staff", 1)], 0, 50);
    req.kind = DutyKind::Procedure {
        interruptible: false,
    };
    let assignment = team.allocate(req).unwrap().assignments[0].assignment;
    assert!(team
        .handover(assignment, StaffId::new(2), window(45, 50), window(50, 80))
        .is_err());
    assert!(team.release_at(assignment, 10).is_err());
    assert_eq!(
        team.registry()
            .assignment(assignment)
            .unwrap()
            .interval
            .end(),
        50
    );
    assert!(team
        .allocate(request(2, 1, &[("staff", 0)], 50, 60))
        .is_err());
    let profile = member(1, &["staff"], 0, 100);
    assert!(TeamAllocator::new(vec![profile.clone(), profile.clone()]).is_err());
    let mut invalid = profile;
    invalid.overtime_per_shift_seconds = u64::MAX;
    assert!(TeamAllocator::new(vec![invalid]).is_err());
}
