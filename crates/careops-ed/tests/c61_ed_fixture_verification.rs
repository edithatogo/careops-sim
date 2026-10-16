//! C6.1 ED Fixture Verification and Companion Tests.
//!
//! Verifies:
//! 1. All six operational dynamics modeled by `CompositeEdFixture` and ED operations:
//!    (1) arrivals/acuity: simultaneous arrivals of ATS 1-5 patients; priority dispatch orders by clinical urgency.
//!    (2) staff zones: zone-based staff eligibility and task allocation.
//!    (3) diagnostics: bay held during diagnostics (`diagnostic_blocked: true`); premature release rejected.
//!    (4) urgent preemption: ATS 1 arrival preempts ATS 5 in-flight patient; accrued work ticks are strictly preserved (`work_elapsed_ticks` conserved, no work lost!); preempted patient resumes and finishes.
//!    (5) bed cleaning: bay vacates to cleaning, unavailable during cleaning duration, becomes available once cleaning finishes.
//!    (6) boarding: boarding status retains bay occupancy until transfer (`st003_verify_retained_occupancy`).
//! 2. Hand-computable one-server queue oracle:
//!    One server, arrivals at 0 and 1; durations 2 -> starts 0, 2; waits 0, 1; completions 2, 4.
//! 3. Trace export:
//!    Convert completed patient records into calibration-compatible trace records (case_id, day, start_tick, end_tick, transit_ticks),
//!    confirming that transit ticks (route duration) and work ticks (service duration) are non-zero, correctly measured, and match the known parameters.

use careops_ed::disposition::composite_fixture::{
    calibration_traces_to_csv, export_patient_calibration_traces, st003_verify_retained_occupancy,
    BoundaryError, ComposedPatient, CompositeEdFixture, PatientLifecycle,
};
use careops_ed::disposition::operations::{
    claim_task, select_dispatch_candidate, BayPhase, ExecutionFidelity, MovementEdge,
    OperationalError, StaffShift, SyntheticGraph, TaskCandidate, TreatmentBay,
};
use careops_ed::Eligibility;

/// (1) arrivals/acuity: simultaneous arrivals of ATS 1-5 patients; priority dispatch orders by clinical urgency.
#[test]
fn test_c61_dynamic_1_arrivals_and_acuity_priority_dispatch() {
    let eligibilities = vec![Eligibility {
        staff_role_id: "staff.ed_nurse".to_string(),
        task_class_id: "task.treatment".to_string(),
        zone_id: "zone.acute".to_string(),
        location_id: Some("loc.acute.1".to_string()),
    }];

    // 1 bay, 1 nurse so dispatch order is strictly serialized
    let mut fixture = CompositeEdFixture::new(
        1,
        1,
        "staff.ed_nurse",
        eligibilities.clone(),
        ExecutionFidelity::Macro,
    );

    // Simultaneous arrivals of ATS 1-5 patients at tick 0
    // Shuffled arrival insertion order to prove clinical urgency (acuity ascending) governs
    let acuities = [3, 5, 1, 4, 2];
    for &acuity in &acuities {
        let p = ComposedPatient {
            id: acuity as u64,
            name: format!("Patient_ATS_{acuity}"),
            acuity,
            arrival_tick: 0,
            work_required_ticks: 2,
            work_elapsed_ticks: 0,
            start_tick: None,
            completion_tick: None,
            assigned_bay: None,
            lifecycle: PatientLifecycle::Arrived,
            task_class_id: "task.treatment".to_string(),
            zone_id: "zone.acute".to_string(),
            location_id: "loc.acute.1".to_string(),
        };
        fixture.admit_patient(p);
    }

    assert_eq!(fixture.queue.len(), 5);

    // Also directly verify candidate selection ordering with all candidates
    let shift = StaffShift::new(1);
    let candidates: Vec<TaskCandidate> = fixture
        .queue
        .iter()
        .map(|p| TaskCandidate {
            task_id: p.id.to_string(),
            patient_id: p.id,
            acuity: p.acuity,
            enqueue_tick: p.arrival_tick,
            task_class_id: p.task_class_id.clone(),
            zone_id: p.zone_id.clone(),
            location_id: p.location_id.clone(),
        })
        .collect();

    let chosen =
        select_dispatch_candidate(&candidates, &shift, "staff.ed_nurse", &eligibilities).unwrap();
    assert_eq!(chosen.acuity, 1);
    assert_eq!(chosen.patient_id, 1);

    // First dispatch cycle must pick ATS 1 (clinical urgency: ATS 1 is highest priority)
    fixture.dispatch_cycle();
    assert_eq!(fixture.in_flight.len(), 1);
    assert_eq!(fixture.in_flight[0].acuity, 1);
    assert_eq!(fixture.in_flight[0].id, 1);

    // Run each to completion and verify sequential dispatch order: ATS 1, 2, 3, 4, 5
    let mut dispatch_order = Vec::new();
    while fixture.completed.len() < 5 {
        if let Some(patient) = fixture.in_flight.first() {
            if !dispatch_order.contains(&patient.acuity) {
                dispatch_order.push(patient.acuity);
            }
        }
        fixture.advance_tick();
        assert!(fixture.verify_invariants(5).is_ok());
    }

    assert_eq!(dispatch_order, vec![1, 2, 3, 4, 5]);

    // All 5 completed in strict clinical urgency order
    let completed_acuities: Vec<u8> = fixture.completed.iter().map(|p| p.acuity).collect();
    assert_eq!(completed_acuities, vec![1, 2, 3, 4, 5]);
}

/// (2) staff zones: zone-based staff eligibility and task allocation.
#[test]
fn test_c61_dynamic_2_staff_zones_and_task_allocation() {
    let eligibilities = vec![Eligibility {
        staff_role_id: "staff.ed_nurse".to_string(),
        task_class_id: "task.treatment".to_string(),
        zone_id: "zone.acute".to_string(),
        location_id: Some("loc.acute.1".to_string()),
    }];

    let mut fixture = CompositeEdFixture::new(
        1,
        1,
        "staff.ed_nurse",
        eligibilities.clone(),
        ExecutionFidelity::Macro,
    );

    // Patient arriving in zone.subacute (staff is NOT eligible for this zone)
    let p_off_zone = ComposedPatient {
        id: 101,
        name: "Patient_Subacute".to_string(),
        acuity: 3,
        arrival_tick: 0,
        work_required_ticks: 5,
        work_elapsed_ticks: 0,
        start_tick: None,
        completion_tick: None,
        assigned_bay: None,
        lifecycle: PatientLifecycle::Arrived,
        task_class_id: "task.treatment".to_string(),
        zone_id: "zone.subacute".to_string(), // Ineligible zone
        location_id: "loc.subacute.1".to_string(),
    };
    fixture.admit_patient(p_off_zone);

    // Dispatch cycle fails to allocate: patient remains in queue
    fixture.dispatch_cycle();
    assert_eq!(fixture.in_flight.len(), 0);
    assert_eq!(fixture.queue.len(), 1);
    assert_eq!(fixture.queue[0].id, 101);

    // Direct claim check confirms Ineligible error for off-zone task
    let mut shift = StaffShift::new(1);
    let off_zone_claim = claim_task(
        &mut shift,
        &eligibilities,
        "staff.ed_nurse",
        "task.treatment",
        "zone.subacute",
        "loc.subacute.1",
    );
    assert_eq!(off_zone_claim, Err(OperationalError::Ineligible));
    assert_eq!(shift.active(), 0);

    // Now admit patient in matching zone.acute
    let p_in_zone = ComposedPatient {
        id: 102,
        name: "Patient_Acute".to_string(),
        acuity: 3,
        arrival_tick: 0,
        work_required_ticks: 3,
        work_elapsed_ticks: 0,
        start_tick: None,
        completion_tick: None,
        assigned_bay: None,
        lifecycle: PatientLifecycle::Arrived,
        task_class_id: "task.treatment".to_string(),
        zone_id: "zone.acute".to_string(), // Eligible zone
        location_id: "loc.acute.1".to_string(),
    };
    fixture.admit_patient(p_in_zone);

    // Dispatch cycle allocates to the eligible in-zone patient
    fixture.dispatch_cycle();
    assert_eq!(fixture.in_flight.len(), 1);
    assert_eq!(fixture.in_flight[0].id, 102);

    // Direct claim on matching zone succeeds
    let in_zone_claim = claim_task(
        &mut shift,
        &eligibilities,
        "staff.ed_nurse",
        "task.treatment",
        "zone.acute",
        "loc.acute.1",
    );
    assert!(in_zone_claim.is_ok());
    assert_eq!(shift.active(), 1);
}

/// (3) diagnostics: bay held during diagnostics (`diagnostic_blocked: true`); premature release rejected.
#[test]
fn test_c61_dynamic_3_diagnostics_bay_held_and_premature_release_rejected() {
    let mut bay = TreatmentBay::available();
    assert_eq!(bay.phase(), BayPhase::Available);

    // Occupy bay with patient 301
    assert!(bay.occupy(301).is_ok());
    assert_eq!(
        bay.phase(),
        BayPhase::Occupied {
            patient_id: 301,
            diagnostic_blocked: false,
            boarding: false,
        }
    );

    // Patient undergoes diagnostic testing: bay is blocked for diagnostics
    assert!(bay.block_for_diagnostics().is_ok());
    assert_eq!(
        bay.phase(),
        BayPhase::Occupied {
            patient_id: 301,
            diagnostic_blocked: true,
            boarding: false,
        }
    );

    // Premature release to cleaning is rejected while diagnostics are pending
    let release_err = bay.vacate_for_cleaning(10);
    assert_eq!(release_err, Err(OperationalError::DiagnosticBlocked));

    // Premature transfer is also rejected while diagnostics are pending
    let transfer_err = bay.accept_transfer(10);
    assert_eq!(transfer_err, Err(OperationalError::DiagnosticBlocked));

    // Bay remains securely occupied by patient 301
    assert_eq!(
        bay.phase(),
        BayPhase::Occupied {
            patient_id: 301,
            diagnostic_blocked: true,
            boarding: false,
        }
    );

    // Once diagnostics results return and block is cleared:
    assert!(bay.clear_diagnostics().is_ok());
    assert_eq!(
        bay.phase(),
        BayPhase::Occupied {
            patient_id: 301,
            diagnostic_blocked: false,
            boarding: false,
        }
    );

    // Now vacation to cleaning succeeds
    assert!(bay.vacate_for_cleaning(10).is_ok());
    assert_eq!(bay.phase(), BayPhase::Cleaning { ready_tick: 10 });
}

/// (4) urgent preemption: ATS 1 arrival preempts ATS 5 in-flight patient;
/// accrued work ticks are strictly preserved (`work_elapsed_ticks` conserved, no work lost!);
/// preempted patient resumes and finishes.
#[test]
fn test_c61_dynamic_4_urgent_preemption_work_conservation_and_resume() {
    let eligibilities = vec![Eligibility {
        staff_role_id: "staff.ed_nurse".to_string(),
        task_class_id: "task.treatment".to_string(),
        zone_id: "zone.acute".to_string(),
        location_id: Some("loc.acute.1".to_string()),
    }];

    let mut fixture = CompositeEdFixture::new(
        1,
        1,
        "staff.ed_nurse",
        eligibilities,
        ExecutionFidelity::Macro,
    );

    // In-flight low-urgency patient: ATS 5 (non-urgent), requires 10 work ticks
    let p_ats5 = ComposedPatient {
        id: 501,
        name: "ATS_5_Patient".to_string(),
        acuity: 5,
        arrival_tick: 0,
        work_required_ticks: 10,
        work_elapsed_ticks: 0,
        start_tick: None,
        completion_tick: None,
        assigned_bay: None,
        lifecycle: PatientLifecycle::Arrived,
        task_class_id: "task.treatment".to_string(),
        zone_id: "zone.acute".to_string(),
        location_id: "loc.acute.1".to_string(),
    };
    fixture.admit_patient(p_ats5);
    fixture.dispatch_cycle();
    assert_eq!(fixture.in_flight.len(), 1);
    assert_eq!(fixture.in_flight[0].id, 501);

    // Progress 4 ticks of active work
    for _ in 0..4 {
        fixture.advance_tick();
    }
    assert_eq!(fixture.in_flight[0].work_elapsed_ticks, 4);

    // Urgent ATS 1 patient arrives at tick 4
    let p_ats1 = ComposedPatient {
        id: 101,
        name: "ATS_1_Resus_Patient".to_string(),
        acuity: 1,
        arrival_tick: 4,
        work_required_ticks: 3,
        work_elapsed_ticks: 0,
        start_tick: None,
        completion_tick: None,
        assigned_bay: None,
        lifecycle: PatientLifecycle::Arrived,
        task_class_id: "task.treatment".to_string(),
        zone_id: "zone.acute".to_string(),
        location_id: "loc.acute.1".to_string(),
    };
    fixture.admit_patient(p_ats1);

    // Preemption triggered: ATS 1 preempts ATS 5
    let preempted = fixture.try_preempt_lowest(1);
    assert!(preempted, "ATS 1 should preempt ATS 5");

    // Preempted patient returned to front of queue
    assert_eq!(fixture.queue.len(), 2);
    let victim = &fixture.queue[0];
    assert_eq!(victim.id, 501);
    assert_eq!(victim.lifecycle, PatientLifecycle::Preempted);

    // CRITICAL: Accrued work ticks are strictly preserved (no work lost!)
    assert_eq!(
        victim.work_elapsed_ticks, 4,
        "accrued work ticks must be strictly conserved upon preemption"
    );

    // Urgent ATS 1 patient dispatched
    fixture.dispatch_cycle();
    assert_eq!(fixture.in_flight.len(), 1);
    assert_eq!(fixture.in_flight[0].id, 101);

    // Advance 3 ticks: ATS 1 completes
    for _ in 0..3 {
        fixture.advance_tick();
    }
    assert_eq!(fixture.completed.len(), 1);
    assert_eq!(fixture.completed[0].id, 101);
    assert_eq!(fixture.completed[0].work_elapsed_ticks, 3);

    // ATS 5 resumed automatically in the next dispatch
    assert_eq!(fixture.in_flight.len(), 1);
    assert_eq!(fixture.in_flight[0].id, 501);
    assert_eq!(fixture.in_flight[0].work_elapsed_ticks, 4);

    // Advance remaining 6 ticks to finish total 10 ticks
    for _ in 0..6 {
        fixture.advance_tick();
    }
    assert_eq!(fixture.completed.len(), 2);
    let p501_done = fixture.completed.iter().find(|p| p.id == 501).unwrap();
    assert_eq!(p501_done.work_elapsed_ticks, 10);
    assert_eq!(p501_done.lifecycle, PatientLifecycle::Discharged);

    assert!(fixture.verify_invariants(2).is_ok());
}

/// (5) bed cleaning: bay vacates to cleaning, unavailable during cleaning duration, becomes available once cleaning finishes.
#[test]
fn test_c61_dynamic_5_bed_cleaning_unavailability_and_restoration() {
    let mut bay = TreatmentBay::available();
    assert!(bay.occupy(1).is_ok());

    // Vacate to cleaning, ready at tick 25
    assert!(bay.vacate_for_cleaning(25).is_ok());
    assert_eq!(bay.phase(), BayPhase::Cleaning { ready_tick: 25 });

    // Bay is unavailable during cleaning duration (ticks 0..24)
    for tick in [0, 10, 20, 24] {
        assert_eq!(bay.occupy(2), Err(OperationalError::CleaningInProgress));
        assert_eq!(bay.reserve(2), Err(OperationalError::CleaningInProgress));
        assert_eq!(
            bay.finish_cleaning(tick),
            Err(OperationalError::CleaningInProgress)
        );
    }

    // At ready_tick (25), cleaning completes and bay becomes available
    assert!(bay.finish_cleaning(25).is_ok());
    assert_eq!(bay.phase(), BayPhase::Available);

    // Bay is now available for new admissions
    assert!(bay.occupy(2).is_ok());
    assert_eq!(
        bay.phase(),
        BayPhase::Occupied {
            patient_id: 2,
            diagnostic_blocked: false,
            boarding: false,
        }
    );
}

/// (6) boarding: boarding status retains bay occupancy until transfer (`st003_verify_retained_occupancy`).
#[test]
fn test_c61_dynamic_6_boarding_retained_occupancy_until_transfer() {
    let mut bay = TreatmentBay::available();
    assert!(bay.occupy(701).is_ok());

    // Patient transitions to boarding
    assert!(bay.begin_boarding().is_ok());
    assert_eq!(
        bay.phase(),
        BayPhase::Occupied {
            patient_id: 701,
            diagnostic_blocked: false,
            boarding: true,
        }
    );

    // Vacate for cleaning is rejected because boarding status retains bay
    assert_eq!(
        bay.vacate_for_cleaning(10),
        Err(OperationalError::BoardingRetained)
    );

    // st003 verifies retained occupancy while physical departure is incomplete
    let retained = st003_verify_retained_occupancy(&bay, 701, false);
    assert_eq!(retained, Ok(true));

    // Premature bay release before departure triggers BoundaryError
    let empty_bay = TreatmentBay::available();
    let premature_release = st003_verify_retained_occupancy(&empty_bay, 701, false);
    assert_eq!(
        premature_release,
        Err(BoundaryError::PrematureBayReleaseBeforePhysicalTransfer)
    );

    // Transfer accepted: bay transitions to cleaning
    assert!(bay.accept_transfer(40).is_ok());
    assert_eq!(bay.phase(), BayPhase::Cleaning { ready_tick: 40 });

    // Once physical departure is complete, retained occupancy is released (returns Ok(false))
    let departure_complete = st003_verify_retained_occupancy(&bay, 701, true);
    assert_eq!(departure_complete, Ok(false));
}

/// Hand-computable one-server queue oracle:
/// One server, arrivals at 0 and 1; durations 2 -> starts 0, 2; waits 0, 1; completions 2, 4.
#[test]
fn test_c61_hand_computable_one_server_queue_oracle() {
    let eligibilities = vec![Eligibility {
        staff_role_id: "staff.ed_nurse".to_string(),
        task_class_id: "task.treatment".to_string(),
        zone_id: "zone.acute".to_string(),
        location_id: Some("loc.acute.1".to_string()),
    }];

    // Exactly one server: 1 treatment bay, 1 rostered nurse
    let mut fixture = CompositeEdFixture::new(
        1,
        1,
        "staff.ed_nurse",
        eligibilities,
        ExecutionFidelity::Macro,
    );

    // Hand-computable oracle specification:
    // Patient A arrives at tick 0, service duration = 2
    // Patient B arrives at tick 1, service duration = 2
    // Hand calculation:
    //   Patient A: arrival = 0, start = 0, wait = 0, completion = 2
    //   Patient B: arrival = 1, start = 2, wait = 1, completion = 4
    let patient_a = ComposedPatient {
        id: 1,
        name: "Patient_A".to_string(),
        acuity: 3,
        arrival_tick: 0,
        work_required_ticks: 2,
        work_elapsed_ticks: 0,
        start_tick: None,
        completion_tick: None,
        assigned_bay: None,
        lifecycle: PatientLifecycle::Arrived,
        task_class_id: "task.treatment".to_string(),
        zone_id: "zone.acute".to_string(),
        location_id: "loc.acute.1".to_string(),
    };

    let patient_b = ComposedPatient {
        id: 2,
        name: "Patient_B".to_string(),
        acuity: 3,
        arrival_tick: 1,
        work_required_ticks: 2,
        work_elapsed_ticks: 0,
        start_tick: None,
        completion_tick: None,
        assigned_bay: None,
        lifecycle: PatientLifecycle::Arrived,
        task_class_id: "task.treatment".to_string(),
        zone_id: "zone.acute".to_string(),
        location_id: "loc.acute.1".to_string(),
    };

    // Tick 0: Patient A arrives and is admitted
    fixture.admit_patient(patient_a);
    fixture.dispatch_cycle();

    assert_eq!(fixture.in_flight.len(), 1);
    assert_eq!(fixture.in_flight[0].id, 1);
    assert_eq!(fixture.in_flight[0].start_tick, Some(0)); // starts at 0

    // Tick 0 -> 1: Advance tick (current_tick becomes 1)
    fixture.advance_tick();
    assert_eq!(fixture.current_tick, 1);
    assert_eq!(fixture.in_flight[0].work_elapsed_ticks, 1);

    // Tick 1: Patient B arrives and is admitted
    fixture.admit_patient(patient_b);
    fixture.dispatch_cycle(); // Server busy, B stays in queue
    assert_eq!(fixture.queue.len(), 1);
    assert_eq!(fixture.queue[0].id, 2);

    // Tick 1 -> 2: Advance tick (current_tick becomes 2)
    // Patient A completes 2nd tick of work -> finishes at 2!
    // Server freed, Patient B dispatched -> starts at 2!
    fixture.advance_tick();
    assert_eq!(fixture.current_tick, 2);
    assert_eq!(fixture.completed.len(), 1);
    assert_eq!(fixture.completed[0].id, 1);
    assert_eq!(fixture.completed[0].completion_tick, Some(2)); // completes at 2

    assert_eq!(fixture.in_flight.len(), 1);
    assert_eq!(fixture.in_flight[0].id, 2);
    assert_eq!(fixture.in_flight[0].start_tick, Some(2)); // starts at 2

    // Tick 2 -> 3: Advance tick (current_tick becomes 3)
    fixture.advance_tick();
    assert_eq!(fixture.current_tick, 3);
    assert_eq!(fixture.in_flight[0].work_elapsed_ticks, 1);

    // Tick 3 -> 4: Advance tick (current_tick becomes 4)
    // Patient B completes 2nd tick of work -> finishes at 4!
    fixture.advance_tick();
    assert_eq!(fixture.current_tick, 4);
    assert_eq!(fixture.completed.len(), 2);
    assert_eq!(fixture.in_flight.len(), 0);
    assert_eq!(fixture.queue.len(), 0);

    let done_a = fixture.completed.iter().find(|p| p.id == 1).unwrap();
    let done_b = fixture.completed.iter().find(|p| p.id == 2).unwrap();

    let wait_a = done_a.start_tick.unwrap() - done_a.arrival_tick;
    let wait_b = done_b.start_tick.unwrap() - done_b.arrival_tick;

    // Verify oracle starts: 0, 2
    assert_eq!(done_a.start_tick, Some(0));
    assert_eq!(done_b.start_tick, Some(2));

    // Verify oracle waits: 0, 1
    assert_eq!(wait_a, 0);
    assert_eq!(wait_b, 1);

    // Verify oracle completions: 2, 4
    assert_eq!(done_a.completion_tick, Some(2));
    assert_eq!(done_b.completion_tick, Some(4));

    // Conserved invariants
    assert!(fixture.verify_invariants(2).is_ok());
}

/// Trace export:
/// Convert completed patient records into calibration-compatible trace records (case_id, day, start_tick, end_tick, transit_ticks),
/// confirming that transit ticks (route duration) and work ticks (service duration) are non-zero, correctly measured, and match the known parameters.
#[test]
fn test_c61_trace_export_calibration_compatible_records() {
    let edge = MovementEdge {
        edge_id: "edge.waiting_to_bay".to_string(),
        from: "loc.waiting".to_string(),
        to: "loc.acute.1".to_string(),
        distance_m: 45.0,
        speed_m_per_s: 1.5, // 45m / 1.5m/s = 30 seconds / ticks transit
        accessible: true,
    };
    let graph = SyntheticGraph::new(vec![edge]);

    let eligibilities = vec![Eligibility {
        staff_role_id: "staff.ed_nurse".to_string(),
        task_class_id: "task.treatment".to_string(),
        zone_id: "zone.acute".to_string(),
        location_id: Some("loc.acute.1".to_string()),
    }];

    let mut fixture = CompositeEdFixture::new(
        2,
        2,
        "staff.ed_nurse",
        eligibilities,
        ExecutionFidelity::Micro,
    );
    fixture.set_graph(graph);

    // Known parameters:
    // Known service duration = 15 ticks
    // Known route duration = 30 ticks (45m at 1.5m/s)
    let known_work_ticks = 15;
    let known_transit_ticks = 30;

    let p1 = ComposedPatient {
        id: 1001,
        name: "TracePatient_1".to_string(),
        acuity: 2,
        arrival_tick: 5,
        work_required_ticks: known_work_ticks,
        work_elapsed_ticks: 0,
        start_tick: None,
        completion_tick: None,
        assigned_bay: None,
        lifecycle: PatientLifecycle::Arrived,
        task_class_id: "task.treatment".to_string(),
        zone_id: "zone.acute".to_string(),
        location_id: "loc.acute.1".to_string(),
    };

    let p2 = ComposedPatient {
        id: 1002,
        name: "TracePatient_2".to_string(),
        acuity: 3,
        arrival_tick: 5,
        work_required_ticks: known_work_ticks,
        work_elapsed_ticks: 0,
        start_tick: None,
        completion_tick: None,
        assigned_bay: None,
        lifecycle: PatientLifecycle::Arrived,
        task_class_id: "task.treatment".to_string(),
        zone_id: "zone.acute".to_string(),
        location_id: "loc.acute.1".to_string(),
    };

    fixture.admit_patient(p1);
    fixture.admit_patient(p2);
    fixture.dispatch_cycle();

    // Run until completion
    while fixture.completed.len() < 2 {
        fixture.advance_tick();
    }

    // Convert completed patient records into calibration-compatible trace records
    let traces = fixture
        .export_calibration_traces(1, &["edge.waiting_to_bay"])
        .expect("trace export should succeed");

    assert_eq!(traces.len(), 2);

    for trace in &traces {
        // Confirm non-zero, correctly measured, and matching known parameters
        assert!(trace.transit_ticks > 0, "transit_ticks must be non-zero");
        assert!(trace.work_ticks > 0, "work_ticks must be non-zero");

        assert_eq!(
            trace.transit_ticks, known_transit_ticks,
            "transit ticks must match known route parameter"
        );
        assert_eq!(
            trace.work_ticks, known_work_ticks,
            "work ticks must match known service duration parameter"
        );

        assert_eq!(trace.day, 1);
        assert!(trace.case_id.starts_with("case_"));
        assert_eq!(
            trace.end_tick - trace.start_tick,
            trace.work_ticks,
            "service duration matches tick difference"
        );
    }

    // Also test standalone helper export_patient_calibration_traces
    let standalone_traces =
        export_patient_calibration_traces(&fixture.completed, 1, known_transit_ticks);
    assert_eq!(traces, standalone_traces);

    // Verify CSV export
    let csv = calibration_traces_to_csv(&traces);
    assert!(csv.contains("case_id,day,start_tick,end_tick,transit_ticks,work_ticks"));
    assert!(csv.contains("case_1001,1,"));
    assert!(csv.contains("case_1002,1,"));
}
