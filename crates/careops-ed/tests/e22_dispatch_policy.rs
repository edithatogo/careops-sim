use careops_ed::dispatch::{
    validate_dispatch_snapshot, CalendarInterval, DispatchRequestInput, DispatchSelection,
    DispatchValidationCategory as Category, StaffSnapshotInput,
};

fn request(
    id: &str,
    urgency: i32,
    ready_tick: u64,
    zone: &str,
    skills: &[&str],
) -> DispatchRequestInput {
    DispatchRequestInput {
        request_id: id.into(),
        ready_tick,
        assigned_zone: zone.into(),
        required_skills: skills.iter().map(|value| (*value).into()).collect(),
        urgency,
        required_staff_count: 1,
    }
}

fn staff(id: &str, zone: &str, skills: &[&str]) -> StaffSnapshotInput {
    StaffSnapshotInput {
        staff_id: id.into(),
        assigned_zone: zone.into(),
        skills: skills.iter().map(|value| (*value).into()).collect(),
        shifts: vec![interval(0, 20)],
        breaks: vec![],
        idle: true,
    }
}

fn interval(start_tick: u64, end_tick: u64) -> CalendarInterval {
    CalendarInterval {
        start_tick,
        end_tick,
    }
}

fn selection<'a>(
    requests: &'a [DispatchRequestInput],
    staff: &'a [StaffSnapshotInput],
    now: u64,
    horizon: u64,
) -> Option<DispatchSelection<'a>> {
    let snapshot = validate_dispatch_snapshot(now, horizon, requests, staff).unwrap();
    careops_ed::dispatch::select_dispatch(&snapshot)
}

fn assert_error(
    result: Result<impl Sized, careops_ed::dispatch::DispatchValidationError>,
    category: Category,
    path: &str,
) {
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("expected dispatch validation error"),
    };
    assert_eq!(error.category, category);
    assert_eq!(error.path, path);
}

#[test]
fn empty_and_single_pair_are_supported() {
    assert_eq!(selection(&[], &[], 0, 10), None);
    let requests = [request("r", 0, 0, "z", &[])];
    let staff = [staff("s", "z", &[])];
    assert_eq!(
        selection(&requests, &staff, 0, 10),
        Some(DispatchSelection {
            request_id: "r",
            staff_id: "s"
        })
    );
}

#[test]
fn high_priority_without_eligible_staff_is_skipped() {
    let requests = [
        request("urgent", 100, 0, "closed", &[]),
        request("next", 0, 0, "open", &[]),
    ];
    let staff = [staff("available", "open", &[])];
    assert_eq!(
        selection(&requests, &staff, 0, 10).unwrap().request_id,
        "next"
    );
}

#[test]
fn request_priority_is_urgency_then_ready_time_then_ascii_id() {
    let requests = [
        request("A", 4, 2, "z", &[]),
        request("Z", 5, 3, "z", &[]),
        request("later", 4, 3, "z", &[]),
        request("B", 4, 2, "z", &[]),
    ];
    let staff = [staff("s", "z", &[])];
    assert_eq!(selection(&requests, &staff, 3, 10).unwrap().request_id, "Z");
    let tied = [request("B", 4, 2, "z", &[]), request("A", 4, 2, "z", &[])];
    assert_eq!(selection(&tied, &staff, 3, 10).unwrap().request_id, "A");
}

#[test]
fn only_already_waiting_requests_compete_and_staff_ties_use_ascii_id() {
    let requests = [request("Z", 8, 1, "z", &[]), request("A", 8, 2, "z", &[])];
    let staff = [staff("z-staff", "z", &[]), staff("A-staff", "z", &[])];
    let chosen = selection(&requests, &staff, 3, 10).unwrap();
    assert_eq!((chosen.request_id, chosen.staff_id), ("Z", "A-staff"));
}

#[test]
fn exact_zone_all_required_skills_and_extra_staff_skills_apply() {
    let request = [request("r", 1, 0, "Zone", &["a", "c"])];
    let staff = [
        staff("wrong-case", "zone", &["a", "c"]),
        staff("eligible", "Zone", &["a", "b", "c"]),
    ];
    assert_eq!(
        selection(&request, &staff, 0, 10).unwrap().staff_id,
        "eligible"
    );
}

#[test]
fn idle_shift_and_break_boundaries_are_half_open() {
    let requests = [request("r", 1, 0, "z", &[])];
    let mut member = staff("s", "z", &[]);
    member.shifts = vec![interval(2, 6)];
    member.breaks = vec![interval(4, 5)];
    assert_eq!(
        selection(&requests, &[member.clone()], 2, 10)
            .unwrap()
            .staff_id,
        "s"
    );
    assert_eq!(selection(&requests, &[member.clone()], 4, 10), None);
    assert_eq!(
        selection(&requests, &[member.clone()], 5, 10)
            .unwrap()
            .staff_id,
        "s"
    );
    assert_eq!(selection(&requests, &[member], 6, 10), None);
}

#[test]
fn busy_staff_and_future_requests_are_unavailable() {
    let mut busy = staff("busy", "z", &[]);
    busy.idle = false;
    let requests = [
        request("future", 10, 4, "z", &[]),
        request("ready", 0, 1, "z", &[]),
    ];
    assert_eq!(selection(&requests, &[busy], 3, 10), None);
}

#[test]
fn calendars_are_order_independent_adjacency_is_valid_and_inputs_are_unchanged() {
    let requests = [request("r", 1, 0, "z", &[])];
    let mut first = staff("s", "z", &[]);
    first.shifts = vec![interval(5, 8), interval(1, 3), interval(3, 5)];
    first.breaks = vec![interval(6, 7), interval(2, 2 + 1)];
    let second = StaffSnapshotInput {
        shifts: vec![interval(1, 3), interval(3, 5), interval(5, 8)],
        breaks: vec![interval(2, 3), interval(6, 7)],
        ..first.clone()
    };
    let first_staff = [first];
    let first_before = first_staff.clone();
    let second_staff = [second];
    let forward = selection(&requests, &first_staff, 1, 10);
    let reversed = selection(&requests, &second_staff, 1, 10);
    assert_eq!(forward, reversed);
    assert_eq!(first_staff, first_before);
}

#[test]
fn now_at_horizon_can_select_from_shift_starting_at_horizon() {
    let requests = [request("r", 1, 9, "z", &[])];
    let mut member = staff("s", "z", &[]);
    member.shifts = vec![interval(10, 11)];
    assert_eq!(
        selection(&requests, &[member], 10, 10).unwrap().staff_id,
        "s"
    );
}

#[test]
fn urgency_handles_minimum_zero_and_maximum_without_negation() {
    let requests = [
        request("min", i32::MIN, 0, "z", &[]),
        request("zero", 0, 0, "z", &[]),
        request("max", i32::MAX, 0, "z", &[]),
    ];
    assert_eq!(
        selection(&requests, &[staff("s", "z", &[])], 0, 10)
            .unwrap()
            .request_id,
        "max"
    );
}

#[test]
fn validation_reports_outer_limits_and_time_paths() {
    assert_error(
        validate_dispatch_snapshot(0, 0, &[], &[]),
        Category::Horizon,
        "horizon",
    );
    assert_error(
        validate_dispatch_snapshot(2, 1, &[], &[]),
        Category::CurrentTime,
        "now",
    );
    let ready = [request("r", 0, 10, "z", &[])];
    assert_error(
        validate_dispatch_snapshot(0, 10, &ready, &[]),
        Category::ReadyTime,
        "requests[0].ready_tick",
    );
    let too_many_staff = vec![staff("s", "z", &[]); 1_025];
    assert_error(
        validate_dispatch_snapshot(0, 10, &[], &too_many_staff),
        Category::Cardinality,
        "staff",
    );
}

#[test]
fn validation_checks_identities_skills_and_required_count_with_paths() {
    let bad_id = [request("é", 0, 0, "z", &[])];
    assert_error(
        validate_dispatch_snapshot(0, 10, &bad_id, &[]),
        Category::Identifier,
        "requests[0].request_id",
    );
    let duplicate = [
        request("same", 0, 0, "z", &[]),
        request("same", 0, 0, "z", &[]),
    ];
    assert_error(
        validate_dispatch_snapshot(0, 10, &duplicate, &[]),
        Category::DuplicateIdentifier,
        "requests[1].request_id",
    );
    let invalid_zone = [request("r", 0, 0, "", &[])];
    assert_error(
        validate_dispatch_snapshot(0, 10, &invalid_zone, &[]),
        Category::Identifier,
        "requests[0].assigned_zone",
    );
    let invalid_skills = [request("r", 0, 0, "z", &["b", "a"])];
    assert_error(
        validate_dispatch_snapshot(0, 10, &invalid_skills, &[]),
        Category::Skills,
        "requests[0].required_skills",
    );
    let mut invalid_count = request("r", 0, 0, "z", &[]);
    invalid_count.required_staff_count = 2;
    assert_error(
        validate_dispatch_snapshot(0, 10, &[invalid_count], &[]),
        Category::UnsupportedStaffCount,
        "requests[0].required_staff_count",
    );
    let too_many_skills: Vec<String> = (0..33).map(|i| format!("s{i:02}")).collect();
    let too_many = [DispatchRequestInput {
        required_skills: too_many_skills,
        ..request("r", 0, 0, "z", &[])
    }];
    assert_error(
        validate_dispatch_snapshot(0, 10, &too_many, &[]),
        Category::Skills,
        "requests[0].required_skills",
    );
}

#[test]
fn skill_cap_and_empty_skill_lists_are_valid() {
    let skills: Vec<String> = (0..32).map(|i| format!("s{i:02}")).collect();
    let member = StaffSnapshotInput {
        skills,
        ..staff("s", "z", &[])
    };
    assert!(validate_dispatch_snapshot(0, 10, &[], &[member]).is_ok());
    assert!(validate_dispatch_snapshot(
        0,
        10,
        &[request("r", 0, 0, "z", &[])],
        &[staff("s", "z", &[])]
    )
    .is_ok());
}

#[test]
fn calendar_validation_reports_original_indices_and_calendar_paths() {
    let mut member = staff("s", "z", &[]);
    member.shifts = vec![interval(4, 5), interval(1, 3)];
    member.breaks = vec![interval(2, 4)];
    assert_error(
        validate_dispatch_snapshot(0, 10, &[], &[member]),
        Category::BreakOutsideShift,
        "staff[0].breaks[0]",
    );
    let mut invalid = staff("s", "z", &[]);
    invalid.shifts = vec![interval(1, 1)];
    assert_error(
        validate_dispatch_snapshot(0, 10, &[], &[invalid]),
        Category::CalendarInterval,
        "staff[0].shifts[0]",
    );
    let mut overlap = staff("s", "z", &[]);
    overlap.shifts = vec![interval(1, 4), interval(3, 5)];
    assert_error(
        validate_dispatch_snapshot(0, 10, &[], &[overlap]),
        Category::CalendarOverlap,
        "staff[0].shifts",
    );
}

#[test]
fn calendar_combined_cap_is_checked_before_intervals() {
    let mut member = staff("s", "z", &[]);
    member.shifts = (0..1_025)
        .map(|index| interval(index * 2, index * 2 + 1))
        .collect();
    assert_error(
        validate_dispatch_snapshot(0, 10, &[], &[member]),
        Category::Cardinality,
        "staff[0]",
    );
}

#[test]
fn bitset_boundaries_and_joint_skill_intersection_select_correct_staff() {
    for eligible_index in [63_usize, 64, 1_023] {
        let mut staff: Vec<_> = (0..1_024)
            .map(|index| {
                let mut member = staff(&format!("s{index:04}"), "z", &["a", "b"]);
                member.idle = index == eligible_index;
                member
            })
            .collect();
        staff.reverse();
        let requests = [request("r", 1, 0, "z", &["a", "b"])];
        let picked = selection(&requests, &staff, 0, 10).unwrap();
        assert_eq!(picked.staff_id, format!("s{eligible_index:04}"));
    }
    let mut only_a = staff("a", "z", &["a"]);
    let only_b = staff("b", "z", &["b"]);
    only_a.staff_id = "a".into();
    let requests = [request("r", 1, 0, "z", &["a", "b"])];
    assert_eq!(selection(&requests, &[only_a, only_b], 0, 10), None);
}

#[test]
fn zone_and_skill_intersections_exclude_otherwise_matching_staff() {
    let requests = [request("r", 1, 0, "z", &["a", "b"])];
    let staff = [
        staff("a", "z", &["a"]),
        staff("b", "other", &["b"]),
        staff("c", "z", &["b"]),
    ];
    assert_eq!(selection(&requests, &staff, 0, 10), None);
}

#[test]
fn available_staff_waits_for_future_request_and_empty_shifts_are_unavailable() {
    let requests = [request("future", 9, 2, "z", &[])];
    let staff_list = [staff("available", "z", &[])];
    assert_eq!(selection(&requests, &staff_list, 1, 10), None);
    assert_eq!(
        selection(&requests, &staff_list, 2, 10).unwrap().request_id,
        "future"
    );

    let mut no_shift = staff("no-shift", "z", &[]);
    no_shift.shifts.clear();
    assert_eq!(
        selection(&[request("r", 1, 0, "z", &[])], &[no_shift], 0, 10),
        None
    );
}

#[test]
fn reversed_inputs_choose_same_pair_and_repeat_selection_without_mutation() {
    let requests = [
        request("Z", 5, 1, "z", &["a"]),
        request("A", 5, 1, "z", &["a"]),
        request("later", 5, 2, "z", &["a"]),
    ];
    let staff = [staff("z-staff", "z", &["a"]), staff("A-staff", "z", &["a"])];
    let requests_before = requests.clone();
    let staff_before = staff.clone();
    let snapshot = validate_dispatch_snapshot(3, 10, &requests, &staff).unwrap();
    let first = careops_ed::dispatch::select_dispatch(&snapshot);
    assert_eq!(first, careops_ed::dispatch::select_dispatch(&snapshot));

    let mut reversed_requests = requests.clone();
    reversed_requests.reverse();
    let mut reversed_staff = staff.clone();
    reversed_staff.reverse();
    let reversed_requests_before = reversed_requests.clone();
    let reversed_staff_before = reversed_staff.clone();
    let reversed_snapshot =
        validate_dispatch_snapshot(3, 10, &reversed_requests, &reversed_staff).unwrap();
    assert_eq!(
        first,
        careops_ed::dispatch::select_dispatch(&reversed_snapshot)
    );
    assert_eq!(
        first.unwrap(),
        DispatchSelection {
            request_id: "A",
            staff_id: "A-staff"
        }
    );
    assert_eq!(requests, requests_before);
    assert_eq!(staff, staff_before);
    assert_eq!(reversed_requests, reversed_requests_before);
    assert_eq!(reversed_staff, reversed_staff_before);
}

#[test]
fn request_outer_cardinality_rejects_100001_and_accepts_100000_unique_rows() {
    let requests: Vec<_> = (0..100_001)
        .map(|index| request(&format!("r{index:06}"), 0, 0, "z", &[]))
        .collect();
    assert_error(
        validate_dispatch_snapshot(0, 1, &requests, &[]),
        Category::Cardinality,
        "requests",
    );
    let valid: Vec<_> = requests[..100_000].to_vec();
    assert!(validate_dispatch_snapshot(0, 1, &valid, &[]).is_ok());
}

#[test]
fn request_and_staff_identity_limits_are_ascii_and_separate_namespaces() {
    let mut invalid_values = vec![String::new(), "é".into(), "x".repeat(65)];
    for invalid in invalid_values.drain(..) {
        let bad_request = [request(&invalid, 0, 0, "z", &[])];
        assert_error(
            validate_dispatch_snapshot(0, 10, &bad_request, &[]),
            Category::Identifier,
            "requests[0].request_id",
        );
        let bad_staff = [staff(&invalid, "z", &[])];
        assert_error(
            validate_dispatch_snapshot(0, 10, &[], &bad_staff),
            Category::Identifier,
            "staff[0].staff_id",
        );
    }
    let valid_id = "x".repeat(64);
    let same_identity = [request(&valid_id, 1, 0, "z", &[])];
    let same_staff = [staff(&valid_id, "z", &[])];
    assert_eq!(
        selection(&same_identity, &same_staff, 0, 10)
            .unwrap()
            .staff_id,
        valid_id
    );

    let duplicate_staff = [staff("same", "z", &[]), staff("same", "z", &[])];
    assert_error(
        validate_dispatch_snapshot(0, 10, &[], &duplicate_staff),
        Category::DuplicateIdentifier,
        "staff[1].staff_id",
    );
    let invalid_zone = [staff("s", "", &[])];
    assert_error(
        validate_dispatch_snapshot(0, 10, &[], &invalid_zone),
        Category::Identifier,
        "staff[0].assigned_zone",
    );
}

#[test]
fn request_and_staff_skill_lists_enforce_order_identity_and_both_caps() {
    let valid_32: Vec<String> = (0..32).map(|index| format!("s{index:02}")).collect();
    let request_32 = [DispatchRequestInput {
        required_skills: valid_32.clone(),
        ..request("r", 0, 0, "z", &[])
    }];
    let staff_32 = [StaffSnapshotInput {
        skills: valid_32.clone(),
        ..staff("s", "z", &[])
    }];
    assert!(validate_dispatch_snapshot(0, 10, &request_32, &staff_32).is_ok());
    let mut malformed_32 = valid_32.clone();
    malformed_32[31] = "é".into();
    let invalid_request = [DispatchRequestInput {
        required_skills: malformed_32.clone(),
        ..request("r", 0, 0, "z", &[])
    }];
    assert_error(
        validate_dispatch_snapshot(0, 10, &invalid_request, &[]),
        Category::Skills,
        "requests[0].required_skills",
    );
    let invalid_staff = [StaffSnapshotInput {
        skills: malformed_32,
        ..staff("s", "z", &[])
    }];
    assert_error(
        validate_dispatch_snapshot(0, 10, &[], &invalid_staff),
        Category::Skills,
        "staff[0].skills",
    );

    for (skills, expected_category) in [
        (vec!["a".to_string(), "a".to_string()], Category::Skills),
        (vec!["b".to_string(), "a".to_string()], Category::Skills),
        (vec!["é".to_string()], Category::Skills),
    ] {
        let invalid_request = [DispatchRequestInput {
            required_skills: skills.clone(),
            ..request("r", 0, 0, "z", &[])
        }];
        assert_error(
            validate_dispatch_snapshot(0, 10, &invalid_request, &[]),
            expected_category.clone(),
            "requests[0].required_skills",
        );
        let invalid_staff = [StaffSnapshotInput {
            skills,
            ..staff("s", "z", &[])
        }];
        assert_error(
            validate_dispatch_snapshot(0, 10, &[], &invalid_staff),
            expected_category,
            "staff[0].skills",
        );
    }
    let too_many: Vec<String> = (0..33).map(|index| format!("s{index:02}")).collect();
    let invalid_request = [DispatchRequestInput {
        required_skills: too_many.clone(),
        ..request("r", 0, 0, "z", &[])
    }];
    assert_error(
        validate_dispatch_snapshot(0, 10, &invalid_request, &[]),
        Category::Skills,
        "requests[0].required_skills",
    );
    let invalid_staff = [StaffSnapshotInput {
        skills: too_many,
        ..staff("s", "z", &[])
    }];
    assert_error(
        validate_dispatch_snapshot(0, 10, &[], &invalid_staff),
        Category::Skills,
        "staff[0].skills",
    );
}

#[test]
fn combined_calendar_limit_accepts_1024_and_rejects_1025() {
    let shifts: Vec<_> = (0..512)
        .map(|index| interval(index * 4, index * 4 + 3))
        .collect();
    let breaks: Vec<_> = (0..512)
        .map(|index| interval(index * 4 + 1, index * 4 + 2))
        .collect();
    let exactly_1024 = [StaffSnapshotInput {
        shifts,
        breaks,
        ..staff("s", "z", &[])
    }];
    assert!(validate_dispatch_snapshot(0, 1, &[], &exactly_1024).is_ok());

    let shifts: Vec<_> = (0..513)
        .map(|index| interval(index * 4, index * 4 + 3))
        .collect();
    let breaks: Vec<_> = (0..512)
        .map(|index| interval(index * 4 + 1, index * 4 + 2))
        .collect();
    let too_many = [StaffSnapshotInput {
        shifts,
        breaks,
        ..staff("s", "z", &[])
    }];
    assert_error(
        validate_dispatch_snapshot(0, 1, &[], &too_many),
        Category::Cardinality,
        "staff[0]",
    );
}

#[test]
fn break_overlap_and_containment_use_original_order_after_calendar_normalization() {
    let mut overlap = staff("s", "z", &[]);
    overlap.shifts = vec![interval(0, 10)];
    overlap.breaks = vec![interval(1, 4), interval(3, 5)];
    assert_error(
        validate_dispatch_snapshot(0, 10, &[], &[overlap]),
        Category::CalendarOverlap,
        "staff[0].breaks",
    );

    let mut original_order = staff("s", "z", &[]);
    original_order.shifts = vec![interval(5, 10)];
    original_order.breaks = vec![interval(11, 12), interval(0, 1)];
    assert_error(
        validate_dispatch_snapshot(0, 10, &[], &[original_order]),
        Category::BreakOutsideShift,
        "staff[0].breaks[0]",
    );

    let mut invalid_interval_at_one = staff("s", "z", &[]);
    invalid_interval_at_one.shifts = vec![interval(5, 10)];
    invalid_interval_at_one.breaks = vec![interval(6, 7), interval(8, 8)];
    assert_error(
        validate_dispatch_snapshot(0, 10, &[], &[invalid_interval_at_one]),
        Category::CalendarInterval,
        "staff[0].breaks[1]",
    );

    let mut containment_at_one = staff("s", "z", &[]);
    containment_at_one.shifts = vec![interval(5, 10)];
    containment_at_one.breaks = vec![interval(6, 7), interval(0, 1)];
    assert_error(
        validate_dispatch_snapshot(0, 10, &[], &[containment_at_one]),
        Category::BreakOutsideShift,
        "staff[0].breaks[1]",
    );

    let mut adjacent = staff("s", "z", &[]);
    adjacent.shifts = vec![interval(1, 3), interval(3, 5)];
    adjacent.breaks = vec![interval(2, 4)];
    assert_error(
        validate_dispatch_snapshot(0, 10, &[], &[adjacent]),
        Category::BreakOutsideShift,
        "staff[0].breaks[0]",
    );
}

#[test]
fn validation_checks_ineligible_rows_and_preserves_error_precedence() {
    let eligible = [request("valid", 1, 0, "z", &[])];
    let mut invalid_future = request("future", 100, 10, "", &[]);
    invalid_future.ready_tick = 10;
    assert_error(
        validate_dispatch_snapshot(
            0,
            10,
            &[eligible[0].clone(), invalid_future],
            &[staff("s", "z", &[])],
        ),
        Category::ReadyTime,
        "requests[1].ready_tick",
    );

    let mut ineligible_invalid_staff = staff("busy", "", &[]);
    ineligible_invalid_staff.idle = false;
    assert_error(
        validate_dispatch_snapshot(
            0,
            10,
            &eligible,
            &[staff("available", "z", &[]), ineligible_invalid_staff],
        ),
        Category::Identifier,
        "staff[1].assigned_zone",
    );

    assert_error(
        validate_dispatch_snapshot(0, 0, &[request("é", 0, 0, "", &[])], &[]),
        Category::Horizon,
        "horizon",
    );
    assert_error(
        validate_dispatch_snapshot(
            0,
            10,
            &[request("é", 0, 0, "z", &[])],
            &[staff("é", "z", &[])],
        ),
        Category::Identifier,
        "requests[0].request_id",
    );

    let mut ready_and_bad_zone = request("r", 0, 10, "", &[]);
    ready_and_bad_zone.ready_tick = 10;
    assert_error(
        validate_dispatch_snapshot(0, 10, &[ready_and_bad_zone], &[]),
        Category::ReadyTime,
        "requests[0].ready_tick",
    );

    let mut bad_skills_and_count = request("r", 0, 0, "z", &["b", "a"]);
    bad_skills_and_count.required_staff_count = 2;
    assert_error(
        validate_dispatch_snapshot(0, 10, &[bad_skills_and_count], &[]),
        Category::Skills,
        "requests[0].required_skills",
    );

    let mut bad_shift_and_break = staff("s", "z", &[]);
    bad_shift_and_break.shifts = vec![interval(3, 3)];
    bad_shift_and_break.breaks = vec![interval(5, 5)];
    assert_error(
        validate_dispatch_snapshot(0, 10, &[], &[bad_shift_and_break]),
        Category::CalendarInterval,
        "staff[0].shifts[0]",
    );
}
