use careops_birthing::spaces::*;
use careops_core::capacity::{Calendar, CapabilitySet, UtcInterval};

fn calendar(start: i64, end: i64) -> Calendar {
    Calendar::new(vec![UtcInterval::new(start, end).unwrap()]).unwrap()
}
fn caps(values: &[&str]) -> CapabilitySet {
    CapabilitySet::new(values.iter().copied()).unwrap()
}
fn room(id: u64, kind: SpaceKind, capability: &str) -> SpaceProfile {
    SpaceProfile {
        id,
        revision: "synthetic:v1".into(),
        kind,
        capabilities: caps(&[capability]),
        open: calendar(0, 100),
        staffed: Some(calendar(0, 100)),
        equipped: Some(true),
    }
}

#[test]
fn configured_spaces_match_capabilities_and_gate_unknown_or_unavailable_capacity() {
    let profiles = [
        room(1, SpaceKind::Ldr, "ldr"),
        room(2, SpaceKind::Pool, "water"),
        room(3, SpaceKind::Monitoring, "monitor"),
        room(4, SpaceKind::Stabilisation, "stabilise"),
    ];
    let mut spaces = SpacePool::new(profiles.to_vec()).unwrap();
    for (id, requirement) in [(1, "ldr"), (2, "water"), (3, "monitor"), (4, "stabilise")] {
        assert_eq!(spaces.reserve(id, caps(&[requirement]), 1).unwrap(), id);
    }
    assert!(spaces.reserve(5, caps(&["ldr"]), 1).is_err());
    assert!(spaces.reserve(6, caps(&["unsupported"]), 1).is_err());
    for mode in 0..4 {
        let mut profile = room(1, SpaceKind::Ldr, "ldr");
        match mode {
            0 => profile.equipped = Some(false),
            1 => profile.equipped = None,
            2 => profile.staffed = None,
            _ => profile.open = calendar(20, 30),
        }
        let mut limited = SpacePool::new(vec![profile]).unwrap();
        assert!(limited.reserve(1, caps(&["ldr"]), 1).is_err());
        assert_eq!(limited.snapshot(1).unwrap().reserved, 0);
        if mode == 1 || mode == 2 {
            assert_eq!(
                limited.snapshot(1).unwrap().assignable_capacity().unwrap(),
                None
            );
        }
    }
}

#[test]
fn reservation_start_boarding_departure_and_cleaning_conserve_one_room() {
    let mut spaces = SpacePool::new(vec![room(1, SpaceKind::Ldr, "ldr")]).unwrap();
    assert_eq!(spaces.reserve(10, caps(&["ldr"]), 1).unwrap(), 1);
    assert_eq!(spaces.reserve(10, caps(&["ldr"]), 1).unwrap(), 1);
    assert!(spaces.reserve(10, caps(&["different"]), 1).is_err());
    assert_eq!(spaces.snapshot(1).unwrap().reserved, 1);
    spaces.start(10, 2).unwrap();
    spaces.start(10, 2).unwrap();
    spaces.advance_to(50).unwrap(); // still boarding while downstream unavailable
    assert_eq!(spaces.snapshot(1).unwrap().occupied, 1);
    assert_eq!(spaces.occupied_seconds(10).unwrap(), 48);
    assert!(spaces.reserve(11, caps(&["ldr"]), 50).is_err());
    spaces.depart(10, 60).unwrap();
    assert_eq!(spaces.snapshot(1).unwrap().occupied, 0);
    assert_eq!(spaces.snapshot(1).unwrap().cleaning, 1);
    assert!(spaces.reserve(11, caps(&["ldr"]), 60).is_err());
    spaces.finish_cleaning(10, 65).unwrap();
    assert_eq!(spaces.reserve(11, caps(&["ldr"]), 66).unwrap(), 1);
    spaces.start(11, 67).unwrap();
    spaces.depart(11, 68).unwrap();
    // An old cleanup replay cannot free the second episode's cleaning hold.
    spaces.finish_cleaning(10, 65).unwrap();
    assert_eq!(spaces.snapshot(1).unwrap().cleaning, 1);
    assert_eq!(spaces.occupied_seconds(10).unwrap(), 58);
}

#[test]
fn closure_and_horizon_keep_occupants_and_cancel_only_unstarted_reservations() {
    let mut spaces = SpacePool::new(vec![
        room(1, SpaceKind::Ldr, "ldr"),
        room(2, SpaceKind::Ldr, "ldr"),
    ])
    .unwrap();
    spaces.reserve(10, caps(&["ldr"]), 1).unwrap();
    spaces.start(10, 2).unwrap();
    spaces.set_open(1, calendar(0, 3), 3).unwrap();
    spaces.reserve(11, caps(&["ldr"]), 3).unwrap();
    spaces.cancel(11, 4).unwrap();
    spaces.advance_to(10).unwrap();
    assert_eq!(spaces.snapshot(1).unwrap().occupied, 1);
    assert_eq!(
        spaces.snapshot(1).unwrap().assignable_capacity().unwrap(),
        Some(0)
    );
    assert_eq!(spaces.snapshot(2).unwrap().reserved, 0);
    assert_eq!(spaces.occupied_seconds(10).unwrap(), 8);
    assert!(spaces.cancel(10, 10).is_err());
    assert!(spaces.start(11, 10).is_err());
    assert!(spaces.depart(10, 9).is_err());
    assert_eq!(spaces.snapshot(1).unwrap().occupied, 1);
}

#[test]
fn malformed_profile_ids_or_versions_reject_instead_of_overwriting_rooms() {
    let valid = room(1, SpaceKind::Ldr, "ldr");
    assert!(SpacePool::new(vec![valid.clone(), valid.clone()]).is_err());
    let mut invalid = valid.clone();
    invalid.id = 0;
    assert!(SpacePool::new(vec![invalid]).is_err());
    let mut invalid = valid;
    invalid.revision.clear();
    assert!(SpacePool::new(vec![invalid]).is_err());
}

#[test]
fn a_reservation_is_retained_but_cannot_start_after_opening_or_staffing_ends() {
    for close_staffing in [false, true] {
        let mut profile = room(1, SpaceKind::Ldr, "ldr");
        if close_staffing {
            profile.staffed = Some(calendar(0, 3));
        } else {
            profile.open = calendar(0, 3);
        }
        let mut spaces = SpacePool::new(vec![profile]).unwrap();
        spaces.reserve(10, caps(&["ldr"]), 1).unwrap();
        spaces.advance_to(4).unwrap();
        assert!(spaces.start(10, 4).is_err());
        assert_eq!(spaces.snapshot(1).unwrap().reserved, 1);
        assert_eq!(spaces.snapshot(1).unwrap().occupied, 0);
    }
}
