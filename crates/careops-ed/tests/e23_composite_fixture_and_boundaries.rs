use careops_ed::disposition::composite_fixture::{
    compare_capacity, run_summary_to_csv, st001_validate_compatible_offer,
    st002_evaluate_offer_policy, st003_verify_retained_occupancy,
    st004_verify_boarder_recurrent_care, st005_evaluate_ambulance_offload,
    st006_receiver_staff_delay, st007_enforce_multi_role_procedure,
    st008_validate_ward_policy_causal_claim, st009_validate_deterioration_hazard,
    BoundaryError, ComposedPatient, CompositeEdFixture, OfferStatus, PatientLifecycle,
};
use careops_ed::disposition::operations::{
    ExecutionFidelity, MovementEdge, OperationalError, ProcedureKind, SyntheticGraph, TreatmentBay,
};
use careops_ed::{parse_scenario_json, run_scenario, Eligibility};

#[test]
fn test_composite_ed_fixture_simultaneous_arrivals_and_invariants() {
    let eligibilities = vec![Eligibility {
        staff_role_id: "staff.ed_nurse".to_string(),
        task_class_id: "task.treatment".to_string(),
        zone_id: "zone.acute".to_string(),
        location_id: Some("loc.acute.1".to_string()),
    }];

    // 2 bays, 2 nurses
    let mut fixture = CompositeEdFixture::new(
        2,
        2,
        "staff.ed_nurse",
        eligibilities,
        ExecutionFidelity::Macro,
    );

    // Three patients arrive simultaneously at tick 1:
    // Patient 1: ATS 2 (high urgency), work 3 ticks
    // Patient 2: ATS 4 (lower urgency), work 4 ticks
    // Patient 3: ATS 3 (medium urgency), work 2 ticks
    let p1 = ComposedPatient {
        id: 1,
        name: "P1".to_string(),
        acuity: 2,
        arrival_tick: 1,
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
    let p2 = ComposedPatient {
        id: 2,
        name: "P2".to_string(),
        acuity: 4,
        arrival_tick: 1,
        work_required_ticks: 4,
        work_elapsed_ticks: 0,
        start_tick: None,
        completion_tick: None,
        assigned_bay: None,
        lifecycle: PatientLifecycle::Arrived,
        task_class_id: "task.treatment".to_string(),
        zone_id: "zone.acute".to_string(),
        location_id: "loc.acute.1".to_string(),
    };
    let p3 = ComposedPatient {
        id: 3,
        name: "P3".to_string(),
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

    fixture.admit_patient(p1);
    fixture.admit_patient(p2);
    fixture.admit_patient(p3);

    // Initial dispatch: only 2 can be dispatched because there are 2 bays
    // ATS 2 (P1) and ATS 3 (P3) should be chosen over ATS 4 (P2)
    fixture.dispatch_cycle();

    assert_eq!(fixture.in_flight.len(), 2);
    assert_eq!(fixture.queue.len(), 1);
    assert_eq!(fixture.queue[0].id, 2); // P2 remains in queue

    // Verify invariants: no double allocation, patient and capacity conserved
    assert!(fixture.verify_invariants(3).is_ok());

    // Advance 2 ticks: P3 requires 2 work ticks, should finish at tick 2
    fixture.advance_tick();
    assert!(fixture.verify_invariants(3).is_ok());
    fixture.advance_tick();
    assert!(fixture.verify_invariants(3).is_ok());

    // P3 completed! P2 should now be dispatched into the freed bay
    assert_eq!(fixture.completed.len(), 1);
    assert_eq!(fixture.completed[0].id, 3);
    assert_eq!(fixture.in_flight.len(), 2); // P1 still in flight, P2 now in flight

    // Run until all complete
    for _ in 0..10 {
        if fixture.completed.len() == 3 {
            break;
        }
        fixture.advance_tick();
        assert!(fixture.verify_invariants(3).is_ok());
    }

    assert_eq!(fixture.completed.len(), 3);
    assert_eq!(fixture.in_flight.len(), 0);
    assert_eq!(fixture.queue.len(), 0);
    assert!(fixture.verify_invariants(3).is_ok());
}

#[test]
fn test_composite_ed_fixture_preemption_and_work_conservation() {
    let eligibilities = vec![Eligibility {
        staff_role_id: "staff.ed_nurse".to_string(),
        task_class_id: "task.treatment".to_string(),
        zone_id: "zone.acute".to_string(),
        location_id: Some("loc.acute.1".to_string()),
    }];

    // 1 bay, 1 nurse
    let mut fixture = CompositeEdFixture::new(
        1,
        1,
        "staff.ed_nurse",
        eligibilities,
        ExecutionFidelity::Macro,
    );

    // Patient 1 (ATS 5, non-urgent) arrives at tick 0, work 6 ticks
    let p1 = ComposedPatient {
        id: 10,
        name: "P10".to_string(),
        acuity: 5,
        arrival_tick: 0,
        work_required_ticks: 6,
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
    fixture.dispatch_cycle();
    assert_eq!(fixture.in_flight.len(), 1);

    // Advance 2 ticks: P10 has done 2 elapsed work ticks
    fixture.advance_tick();
    fixture.advance_tick();
    assert_eq!(fixture.in_flight[0].work_elapsed_ticks, 2);

    // Urgent Patient 2 (ATS 1, resus) arrives at tick 2
    let p2 = ComposedPatient {
        id: 20,
        name: "P20-urgent".to_string(),
        acuity: 1,
        arrival_tick: 2,
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
    fixture.admit_patient(p2);

    // Try preemption: ATS 1 preempts ATS 5
    let preempted = fixture.try_preempt_lowest(1);
    assert!(preempted);

    // P10 is returned to queue, retains its 2 elapsed ticks (no work lost!)
    assert_eq!(fixture.queue[0].id, 10);
    assert_eq!(fixture.queue[0].work_elapsed_ticks, 2);
    assert_eq!(fixture.queue[0].lifecycle, PatientLifecycle::Preempted);

    // Dispatch cycle assigns bay and nurse to urgent P20
    fixture.dispatch_cycle();
    assert_eq!(fixture.in_flight.len(), 1);
    assert_eq!(fixture.in_flight[0].id, 20);

    // Advance until P20 finishes (2 ticks)
    fixture.advance_tick();
    fixture.advance_tick();
    assert_eq!(fixture.completed[0].id, 20);

    // P10 resumes treatment and finishes remaining 4 ticks
    for _ in 0..6 {
        if fixture.completed.len() == 2 {
            break;
        }
        fixture.advance_tick();
    }

    assert_eq!(fixture.completed.len(), 2);
    let p10_done = fixture.completed.iter().find(|p| p.id == 10).unwrap();
    assert_eq!(p10_done.work_elapsed_ticks, 6);
    assert!(fixture.verify_invariants(2).is_ok());
}

#[test]
fn test_st001_compatible_offers() {
    // Bed offer matching patient destination is accepted
    assert!(st001_validate_compatible_offer("cardiology", "cardiology").is_ok());

    // Mismatched bed offer is rejected
    assert_eq!(
        st001_validate_compatible_offer("cardiology", "general_surgical"),
        Err(BoundaryError::IncompatibleOfferDestination)
    );
}

#[test]
fn test_st002_unused_offers_and_lifetime_policy() {
    // Immediate expiry as universal default is prohibited
    assert_eq!(
        st002_evaluate_offer_policy(10, 10, 0, true),
        Err(BoundaryError::ImmediateExpiryDefaultProhibited)
    );

    // Valid offer within lifetime policy window is accepted
    let status = st002_evaluate_offer_policy(10, 15, 10, false);
    assert_eq!(status, Ok(OfferStatus::Accepted));

    // Offer beyond lifetime expires cleanly
    let expired = st002_evaluate_offer_policy(10, 25, 10, false);
    assert_eq!(expired, Ok(OfferStatus::Expired));
}

#[test]
fn test_st003_retained_occupancy_until_departure() {
    let mut bay = TreatmentBay::available();
    assert!(bay.occupy(42).is_ok());

    // While physical transfer is incomplete, bay remains retained
    assert_eq!(st003_verify_retained_occupancy(&bay, 42, false), Ok(true));

    // If bay is cleared prematurely before departure, error is reported
    let empty_bay = TreatmentBay::available();
    assert_eq!(
        st003_verify_retained_occupancy(&empty_bay, 42, false),
        Err(BoundaryError::PrematureBayReleaseBeforePhysicalTransfer)
    );
}

#[test]
fn test_st004_retained_care_and_no_fulltime_nurse_default() {
    // Boarder care is allowed without full-time dedicated nurse
    assert!(st004_verify_boarder_recurrent_care(true, false).is_ok());

    // Reserving full-time 1:1 nurse per boarder by default is prohibited
    assert_eq!(
        st004_verify_boarder_recurrent_care(true, true),
        Err(BoundaryError::ExcessiveBoarderStaffReservation)
    );
}

#[test]
fn test_st005_saturated_offload_and_crew_release() {
    // Saturated ED bays block ambulance offload and crew release
    assert_eq!(
        st005_evaluate_ambulance_offload(0, 2),
        Err(BoundaryError::AmbulanceOffloadBlockedBySaturation)
    );

    // Available bay and nurse permit offload and crew release
    assert_eq!(
        st005_evaluate_ambulance_offload(1, 1),
        Ok(careops_ed::disposition::composite_fixture::OffloadStatus::OffloadedCrewReleased)
    );
}

#[test]
fn test_st006_receiver_staff_sensitivity() {
    // When receiving staff are unavailable, offload delay fails closed
    assert_eq!(
        st006_receiver_staff_delay(0, 5),
        Err(BoundaryError::ReceiverStaffUnavailable)
    );

    // When staff are available, base delay is observed
    assert_eq!(st006_receiver_staff_delay(1, 5), Ok(5));
}

#[test]
fn test_st007_unsupported_multi_role_procedure_enforcement() {
    // Procedural sedation without dedicated composite team is rejected
    assert_eq!(
        st007_enforce_multi_role_procedure(ProcedureKind::ProceduralSedation, false),
        Err(OperationalError::UnsupportedMultiRole)
    );

    // Procedural sedation with dedicated composite team approximation is accepted
    assert!(st007_enforce_multi_role_procedure(ProcedureKind::ProceduralSedation, true).is_ok());

    // Routine assessment requires no composite team
    assert!(st007_enforce_multi_role_procedure(ProcedureKind::RoutineAssessment, false).is_ok());
}

#[test]
fn test_st008_prohibited_causal_ward_policy_claims() {
    // External offer scenario cannot claim causal effects of inpatient ward policies
    assert_eq!(
        st008_validate_ward_policy_causal_claim(true, false),
        Err(BoundaryError::ProhibitedCausalWardPolicyClaim)
    );

    // Permitted when internal ward model is explicitly configured
    assert!(st008_validate_ward_policy_causal_claim(true, true).is_ok());
    assert!(st008_validate_ward_policy_causal_claim(false, false).is_ok());
}

#[test]
fn test_st009_no_implicit_deterioration_hazard_default() {
    // Implicit background deterioration hazard by default is prohibited
    assert_eq!(
        st009_validate_deterioration_hazard(true, false),
        Err(BoundaryError::UnsupportedImplicitDeteriorationHazard)
    );

    // Permitted only when explicitly parameterized
    assert!(st009_validate_deterioration_hazard(true, true).is_ok());
    assert!(st009_validate_deterioration_hazard(false, false).is_ok());
}

#[test]
fn test_macro_micro_synthetic_graph_transit() {
    let edge = MovementEdge {
        edge_id: "edge.triage_to_acute".to_string(),
        from: "loc.triage.1".to_string(),
        to: "loc.acute.1".to_string(),
        distance_m: 30.0,
        speed_m_per_s: 1.5,
        accessible: true,
    };
    let graph = SyntheticGraph::new(vec![edge]);

    let mut fixture = CompositeEdFixture::new(
        2,
        2,
        "staff.ed_nurse",
        Vec::new(),
        ExecutionFidelity::Micro,
    );
    fixture.set_graph(graph);

    let travel = fixture
        .graph
        .as_ref()
        .unwrap()
        .compute_travel_seconds(&["edge.triage_to_acute"])
        .unwrap();
    assert_eq!(travel, 20.0); // 30m / 1.5m/s = 20s
}

#[test]
fn test_csv_export_and_capacity_comparison_oracle() {
    let nominal_source = include_str!("../examples/nominal_mvp.json");
    let constrained_source = include_str!("../examples/constrained_mvp.json");

    let nominal_cfg = parse_scenario_json(nominal_source).unwrap();
    let constrained_cfg = parse_scenario_json(constrained_source).unwrap();

    let nominal_summary = run_scenario(&nominal_cfg).unwrap();
    let constrained_summary = run_scenario(&constrained_cfg).unwrap();

    // Verify CSV output
    let csv = run_summary_to_csv(&nominal_summary);
    assert!(csv.contains("patient_id,arrival_tick,start_tick"));
    assert!(csv.contains("patient-001,1,1,4,3,0,3,completed"));

    // Verify Capacity Comparison
    let comparison = compare_capacity(&nominal_summary, &constrained_summary);
    assert_eq!(comparison.nominal_completed, 5);
    assert_eq!(comparison.constrained_completed, 5);
    assert!(comparison.wait_time_increased);
    assert_eq!(comparison.nominal_mean_wait_ticks, 0.4);
    assert_eq!(comparison.constrained_mean_wait_ticks, 4.2);
}
