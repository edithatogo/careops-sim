use careops_core::capacity::{
    deduplicated_assignable_capacity, map_local_time, Calendar, CapabilitySet,
    CapacityPoolDeclaration, CapacitySnapshot, LocalTimeMapping, ResolvedLocalInstant, UtcInterval,
};

fn snapshot() -> CapacitySnapshot {
    CapacitySnapshot {
        physical: Some(6),
        equipped: Some(5),
        open: Some(4),
        staffed: Some(3),
        reserved: 1,
        occupied: 1,
        cleaning: 0,
        unavailable: 0,
    }
}

#[test]
fn capacity_dispositions_conserve_equipped_inventory() {
    let mut value = snapshot();
    value.reserved = 2;
    value.occupied = 2;
    value.cleaning = 1;
    assert!(value.validate().is_ok());
    assert_eq!(value.assignable_capacity().unwrap(), Some(0));

    value.unavailable = 1;
    assert!(value.validate().is_err());
}

#[test]
fn equipped_and_open_cannot_exceed_physical_inventory() {
    let mut value = snapshot();
    value.equipped = Some(7);
    assert!(value.validate().is_err());
    value.equipped = Some(5);
    value.open = Some(7);
    assert!(value.validate().is_err());
}

#[test]
fn unknown_capacity_is_distinct_from_known_zero_and_zero_demand() {
    let mut value = snapshot();
    value.open = None;
    assert_eq!(value.assignable_capacity().unwrap(), None);

    value.open = Some(0);
    assert_eq!(value.assignable_capacity().unwrap(), Some(0));
    assert!(value.validate().is_ok());
}

#[test]
fn disposition_sum_overflow_is_rejected() {
    let mut value = snapshot();
    value.reserved = u64::MAX;
    value.occupied = 1;
    assert!(value.validate().is_err());
}

#[test]
fn known_physical_inventory_constrains_dispositions_when_equipped_is_unknown() {
    let mut value = snapshot();
    value.physical = Some(0);
    value.equipped = None;
    value.open = Some(0);
    value.staffed = None;
    value.occupied = 1;
    value.reserved = 0;
    assert!(value.validate().is_err());
    assert!(value.assignable_capacity().is_err());
}

#[test]
fn any_known_zero_effective_ceiling_dominates_unknown_ceilings() {
    let mut cases = Vec::new();

    let mut physical_zero = snapshot();
    physical_zero.physical = Some(0);
    physical_zero.equipped = None;
    physical_zero.open = None;
    physical_zero.staffed = None;
    cases.push(physical_zero);

    let mut equipped_zero = snapshot();
    equipped_zero.equipped = Some(0);
    equipped_zero.open = None;
    equipped_zero.staffed = None;
    cases.push(equipped_zero);

    let mut open_zero = snapshot();
    open_zero.open = Some(0);
    open_zero.staffed = None;
    cases.push(open_zero);

    let mut staffed_zero = snapshot();
    staffed_zero.open = None;
    staffed_zero.staffed = Some(0);
    cases.push(staffed_zero);

    for mut value in cases {
        value.reserved = 0;
        value.occupied = 0;
        value.cleaning = 0;
        value.unavailable = 0;
        assert_eq!(value.assignable_capacity().unwrap(), Some(0));
        assert_eq!(value.unmet_demand(2).unwrap(), Some(2));
    }
}

fn pool(snapshot: CapacitySnapshot) -> CapacityPoolDeclaration {
    CapacityPoolDeclaration {
        pool_id: "pool-a".into(),
        unit: "bay".into(),
        revision: "r1".into(),
        as_of_utc_seconds: 100,
        snapshot,
    }
}

#[test]
fn shared_pool_aliases_are_counted_once() {
    let declaration = pool(snapshot());
    assert_eq!(
        deduplicated_assignable_capacity(&[declaration.clone(), declaration]).unwrap(),
        Some(1)
    );
}

#[test]
fn conflicting_shared_pool_declarations_fail_closed() {
    let first = pool(snapshot());
    let mut conflicting = first.clone();
    conflicting.snapshot.open = Some(3);
    assert!(deduplicated_assignable_capacity(&[first, conflicting]).is_err());
}

#[test]
fn shared_pool_identity_includes_unit_revision_and_snapshot_time() {
    let first = pool(snapshot());
    let mutations: [fn(&mut CapacityPoolDeclaration); 3] = [
        |value: &mut CapacityPoolDeclaration| value.unit = "room".into(),
        |value: &mut CapacityPoolDeclaration| value.revision = "r2".into(),
        |value: &mut CapacityPoolDeclaration| value.as_of_utc_seconds += 1,
    ];
    for mutate in mutations {
        let mut conflicting = first.clone();
        mutate(&mut conflicting);
        assert!(deduplicated_assignable_capacity(&[first.clone(), conflicting]).is_err());
    }
}

#[test]
fn distinct_pools_aggregate_once_each_and_overflow_rejects() {
    let first = pool(snapshot());
    let mut second = first.clone();
    second.pool_id = "pool-b".into();
    assert_eq!(
        deduplicated_assignable_capacity(&[first, second]).unwrap(),
        Some(2)
    );

    let maximum = CapacitySnapshot {
        physical: Some(u64::MAX),
        equipped: Some(u64::MAX),
        open: Some(u64::MAX),
        staffed: Some(u64::MAX),
        reserved: 0,
        occupied: 0,
        cleaning: 0,
        unavailable: 0,
    };
    let first = pool(maximum.clone());
    let mut second = first.clone();
    second.pool_id = "pool-b".into();
    assert!(deduplicated_assignable_capacity(&[first, second]).is_err());
}

#[test]
fn distinct_pool_totals_reject_mixed_units_or_snapshot_times() {
    let first = pool(snapshot());
    let mut different_unit = first.clone();
    different_unit.pool_id = "pool-b".into();
    different_unit.unit = "room".into();
    assert!(deduplicated_assignable_capacity(&[first.clone(), different_unit]).is_err());

    let mut different_time = first.clone();
    different_time.pool_id = "pool-b".into();
    different_time.as_of_utc_seconds += 1;
    assert!(deduplicated_assignable_capacity(&[first, different_time]).is_err());
}

#[test]
fn unknown_pool_capacity_propagates_in_aggregate() {
    let known = pool(snapshot());
    let mut unknown = pool(snapshot());
    unknown.pool_id = "pool-b".into();
    unknown.snapshot.staffed = None;
    assert_eq!(
        deduplicated_assignable_capacity(&[known, unknown]).unwrap(),
        None
    );
}

#[test]
fn capability_matching_is_set_based_and_requires_every_capability() {
    let offered = CapabilitySet::new(["oxygen", "monitor", "oxygen"]).unwrap();
    let reordered = CapabilitySet::new(["monitor", "oxygen"]).unwrap();
    let required = CapabilitySet::new(["oxygen", "monitor"]).unwrap();
    let missing = CapabilitySet::new(["oxygen", "isolation"]).unwrap();
    assert_eq!(offered, reordered);
    assert!(offered.satisfies(&required));
    assert!(!offered.satisfies(&missing));
}

#[test]
fn capability_identifiers_must_not_be_blank() {
    assert!(CapabilitySet::new(["  "]).is_err());
}

#[test]
fn utc_intervals_are_half_open_and_adjacent_windows_are_valid() {
    let first = UtcInterval::new(10, 20).unwrap();
    let second = UtcInterval::new(20, 30).unwrap();
    let calendar = Calendar::new(vec![first, second]).unwrap();
    assert!(calendar.contains(10));
    assert!(calendar.contains(19));
    assert!(calendar.contains(20));
    assert!(!calendar.contains(30));
}

#[test]
fn zero_reversed_and_overlapping_calendar_intervals_reject() {
    assert!(UtcInterval::new(5, 5).is_err());
    assert!(UtcInterval::new(6, 5).is_err());
    let first = UtcInterval::new(10, 20).unwrap();
    let overlapping = UtcInterval::new(19, 30).unwrap();
    assert!(Calendar::new(vec![first, overlapping]).is_err());
}

#[test]
fn repeated_fall_back_wall_time_uses_each_explicit_offset() {
    let local = 1_800_000_000;
    let daylight = ResolvedLocalInstant {
        local_epoch_seconds: local,
        utc_offset_seconds: -4 * 60 * 60,
    };
    let standard = ResolvedLocalInstant {
        local_epoch_seconds: local,
        utc_offset_seconds: -5 * 60 * 60,
    };
    let daylight_utc = daylight.to_utc_seconds().unwrap();
    let standard_utc = standard.to_utc_seconds().unwrap();
    assert_eq!(standard_utc - daylight_utc, 60 * 60);
}

#[test]
fn nonexistent_spring_time_requires_explicit_reject_or_skip() {
    assert!(map_local_time(LocalTimeMapping::RejectedNonexistent).is_err());
    assert_eq!(
        map_local_time(LocalTimeMapping::SkippedNonexistent).unwrap(),
        None
    );
}

#[test]
fn explicit_offsets_map_local_days_to_23_and_25_hour_utc_intervals() {
    let local_day_start = 0;
    let local_day_end = 24 * 60 * 60;
    let spring = UtcInterval::from_resolved_local(
        ResolvedLocalInstant {
            local_epoch_seconds: local_day_start,
            utc_offset_seconds: -5 * 60 * 60,
        },
        ResolvedLocalInstant {
            local_epoch_seconds: local_day_end,
            utc_offset_seconds: -4 * 60 * 60,
        },
    )
    .unwrap();
    let fall = UtcInterval::from_resolved_local(
        ResolvedLocalInstant {
            local_epoch_seconds: local_day_start,
            utc_offset_seconds: -4 * 60 * 60,
        },
        ResolvedLocalInstant {
            local_epoch_seconds: local_day_end,
            utc_offset_seconds: -5 * 60 * 60,
        },
    )
    .unwrap();
    assert_eq!(spring.duration_seconds(), 23 * 60 * 60);
    assert_eq!(fall.duration_seconds(), 25 * 60 * 60);
}

#[test]
fn explicit_offset_conversion_rejects_overflow() {
    let local = ResolvedLocalInstant {
        local_epoch_seconds: i64::MIN,
        utc_offset_seconds: 1,
    };
    assert!(local.to_utc_seconds().is_err());
}

#[test]
fn closure_and_staffing_changes_preserve_active_dispositions() {
    let mut active = snapshot();
    active.reserved = 1;
    active.occupied = 1;
    let closed = active.with_availability(Some(0), Some(0)).unwrap();

    assert_eq!(closed.reserved, 1);
    assert_eq!(closed.occupied, 1);
    assert_eq!(closed.cleaning, 0);
    assert_eq!(closed.assignable_capacity().unwrap(), Some(0));
    assert_eq!(active.reserved, 1);
    assert_eq!(active.occupied, 1);
}

#[test]
fn reservation_service_cleaning_turnover_releases_capacity_in_order() {
    let mut value = snapshot();
    value.start_reserved_work(1).unwrap();
    assert_eq!(value.reserved, 0);
    assert_eq!(value.occupied, 2);

    value.complete_occupied_work(1, true).unwrap();
    assert_eq!(value.occupied, 1);
    assert_eq!(value.cleaning, 1);
    assert_eq!(value.assignable_capacity().unwrap(), Some(1));

    value.finish_cleaning(1).unwrap();
    assert_eq!(value.cleaning, 0);
    assert_eq!(value.assignable_capacity().unwrap(), Some(2));
}

#[test]
fn maintenance_marks_only_an_explicit_unavailable_disposition() {
    let mut value = snapshot();
    value.reserved = 0;
    value.occupied = 1;
    value.mark_unavailable(1).unwrap();
    assert_eq!(value.occupied, 1);
    assert_eq!(value.unavailable, 1);
    assert_eq!(value.assignable_capacity().unwrap(), Some(1));

    value.restore_unavailable(1).unwrap();
    assert_eq!(value.unavailable, 0);
    assert_eq!(value.occupied, 1);
    assert_eq!(value.assignable_capacity().unwrap(), Some(2));
}

#[test]
fn rejected_lifecycle_transition_does_not_mutate_snapshot() {
    let mut value = snapshot();
    let before = value.clone();
    assert!(value.finish_cleaning(1).is_err());
    assert_eq!(value, before);
    assert!(value.complete_occupied_work(2, false).is_err());
    assert_eq!(value, before);
    assert!(value.start_reserved_work(2).is_err());
    assert_eq!(value, before);
    assert!(value.restore_unavailable(1).is_err());
    assert_eq!(value, before);

    let mut maximum = CapacitySnapshot {
        physical: Some(u64::MAX),
        equipped: Some(u64::MAX),
        open: Some(u64::MAX),
        staffed: Some(u64::MAX),
        reserved: 0,
        occupied: 0,
        cleaning: 0,
        unavailable: u64::MAX,
    };
    maximum.validate().unwrap();
    let before_overflow = maximum.clone();
    assert!(maximum.mark_unavailable(1).is_err());
    assert_eq!(maximum, before_overflow);
}

#[test]
fn unavailable_capacity_zero_demand_and_unknown_capacity_remain_distinct() {
    let mut unavailable = snapshot();
    unavailable = unavailable.with_availability(Some(0), Some(0)).unwrap();
    assert_eq!(unavailable.assignable_capacity().unwrap(), Some(0));
    assert_eq!(unavailable.unmet_demand(2).unwrap(), Some(2));

    let mut unknown = snapshot();
    unknown.staffed = None;
    assert_eq!(unknown.assignable_capacity().unwrap(), None);
    assert_eq!(unknown.unmet_demand(2).unwrap(), None);
    assert_eq!(unknown.unmet_demand(0).unwrap(), Some(0));
}
