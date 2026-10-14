//! Tests for E2.2 minimal agent dispatch, Macro/Micro route fidelity,
//! phased resource claims, and unsupported multi-role procedure limits.

use careops_ed::disposition::operations::{
    calculate_execution_duration, claim_atomic_multi_resource, execute_phased_claim,
    select_dispatch_candidate, validate_procedure_staffing, ExecutionFidelity, MovementEdge,
    OperationalError, ProcedureKind, StaffShift, SyntheticGraph, TaskCandidate, TaskClaimTarget,
    TreatmentBay,
};
use careops_ed::Eligibility;

fn sample_eligibilities() -> Vec<Eligibility> {
    vec![
        Eligibility {
            staff_role_id: "role.triage_nurse".into(),
            task_class_id: "task.triage".into(),
            zone_id: "zone.waiting".into(),
            location_id: Some("loc.triage_desk".into()),
        },
        Eligibility {
            staff_role_id: "role.resus_doctor".into(),
            task_class_id: "task.clinical_assessment".into(),
            zone_id: "zone.acute".into(),
            location_id: None,
        },
        Eligibility {
            staff_role_id: "role.ed_doctor".into(),
            task_class_id: "task.clinical_assessment".into(),
            zone_id: "zone.acute".into(),
            location_id: None,
        },
    ]
}

#[test]
fn test_dispatch_selection_urgency_fifo_and_ties() {
    let eligibilities = sample_eligibilities();
    let shift = StaffShift::new(2);

    // Queue of candidates arriving in mixed order
    let candidates = vec![
        TaskCandidate {
            task_id: "task_4_early".into(),
            patient_id: 401,
            acuity: 4, // ATS 4 (Semi-urgent)
            enqueue_tick: 10,
            task_class_id: "task.clinical_assessment".into(),
            zone_id: "zone.acute".into(),
            location_id: "bay_1".into(),
        },
        TaskCandidate {
            task_id: "task_2_late".into(),
            patient_id: 202,
            acuity: 2, // ATS 2 (Emergency)
            enqueue_tick: 25,
            task_class_id: "task.clinical_assessment".into(),
            zone_id: "zone.acute".into(),
            location_id: "bay_2".into(),
        },
        TaskCandidate {
            task_id: "task_2_early".into(),
            patient_id: 201,
            acuity: 2, // ATS 2 (Emergency)
            enqueue_tick: 15,
            task_class_id: "task.clinical_assessment".into(),
            zone_id: "zone.acute".into(),
            location_id: "bay_3".into(),
        },
        TaskCandidate {
            task_id: "task_1_resus".into(),
            patient_id: 101,
            acuity: 1, // ATS 1 (Immediate / Resuscitation)
            enqueue_tick: 30,
            task_class_id: "task.clinical_assessment".into(),
            zone_id: "zone.acute".into(),
            location_id: "resus_1".into(),
        },
    ];

    // ATS 1 must be dispatched first, despite arriving latest (tick 30)
    let selected =
        select_dispatch_candidate(&candidates, &shift, "role.ed_doctor", &eligibilities).unwrap();
    assert_eq!(selected.task_id, "task_1_resus");
    assert_eq!(selected.acuity, 1);

    // Exclude ATS 1: between the two ATS 2 patients, FIFO governs (tick 15 before tick 25)
    let sub_candidates = &candidates[0..3];
    let selected_fifo =
        select_dispatch_candidate(sub_candidates, &shift, "role.ed_doctor", &eligibilities)
            .unwrap();
    assert_eq!(selected_fifo.task_id, "task_2_early");
    assert_eq!(selected_fifo.enqueue_tick, 15);

    // Deterministic tie-breaker: identical acuity and identical enqueue_tick
    let tied_candidates = vec![
        TaskCandidate {
            task_id: "task_b".into(),
            patient_id: 502,
            acuity: 3,
            enqueue_tick: 50,
            task_class_id: "task.clinical_assessment".into(),
            zone_id: "zone.acute".into(),
            location_id: "bay_5".into(),
        },
        TaskCandidate {
            task_id: "task_a".into(),
            patient_id: 501,
            acuity: 3,
            enqueue_tick: 50,
            task_class_id: "task.clinical_assessment".into(),
            zone_id: "zone.acute".into(),
            location_id: "bay_4".into(),
        },
    ];
    let selected_tie =
        select_dispatch_candidate(&tied_candidates, &shift, "role.ed_doctor", &eligibilities)
            .unwrap();
    // Deterministic tie-break picks lower patient_id (501 before 502)
    assert_eq!(selected_tie.patient_id, 501);
    assert_eq!(selected_tie.task_id, "task_a");

    // Ineligible role cannot claim
    let ineligible =
        select_dispatch_candidate(&candidates, &shift, "role.triage_nurse", &eligibilities);
    assert!(ineligible.is_none());

    // Saturated / zero-claimable shift cannot claim
    let mut busy_shift = StaffShift::new(1);
    busy_shift.begin_break(1).unwrap();
    assert_eq!(busy_shift.claimable(), 0);
    let no_capacity =
        select_dispatch_candidate(&candidates, &busy_shift, "role.ed_doctor", &eligibilities);
    assert!(no_capacity.is_none());
}

#[test]
fn test_macro_micro_fidelity_authoritative_state_and_edge_transit() {
    let graph = SyntheticGraph::new(vec![
        MovementEdge {
            edge_id: "ab".into(),
            from: "A".into(),
            to: "B".into(),
            distance_m: 18.5,
            speed_m_per_s: 1.0,
            accessible: true,
        },
        MovementEdge {
            edge_id: "bc".into(),
            from: "B".into(),
            to: "C".into(),
            distance_m: 30.0,
            speed_m_per_s: 1.5,
            accessible: true,
        },
        MovementEdge {
            edge_id: "bd_closed".into(),
            from: "B".into(),
            to: "D".into(),
            distance_m: 9.0,
            speed_m_per_s: 1.0,
            accessible: false,
        },
    ]);

    let active_work_seconds = 12.0;

    // In Macro mode: transit is not modeled; duration equals active work time exactly
    let macro_duration =
        calculate_execution_duration(ExecutionFidelity::Macro, active_work_seconds, None, &[])
            .unwrap();
    assert_eq!(macro_duration, 12.0);

    // In Micro mode: edge travel is evaluated (18.5m / 1.0 m/s = 18.5s)
    let micro_duration = calculate_execution_duration(
        ExecutionFidelity::Micro,
        active_work_seconds,
        Some(&graph),
        &["ab"],
    )
    .unwrap();
    assert_eq!(micro_duration, 30.5); // 12.0 active + 18.5 travel

    // Multi-edge path: ab (18.5s) + bc (30m / 1.5m/s = 20.0s) = 38.5s travel + 12.0s = 50.5s
    let multi_duration = calculate_execution_duration(
        ExecutionFidelity::Micro,
        active_work_seconds,
        Some(&graph),
        &["ab", "bc"],
    )
    .unwrap();
    assert_eq!(multi_duration, 50.5);

    // Inaccessible edge is rejected
    let inacc_result = calculate_execution_duration(
        ExecutionFidelity::Micro,
        active_work_seconds,
        Some(&graph),
        &["bd_closed"],
    );
    assert_eq!(inacc_result, Err(OperationalError::EdgeInaccessible));

    // Non-existent route is rejected
    let unreach_result = calculate_execution_duration(
        ExecutionFidelity::Micro,
        active_work_seconds,
        Some(&graph),
        &["unknown_edge"],
    );
    assert_eq!(unreach_result, Err(OperationalError::RouteUnreachable));
}

#[test]
fn test_phased_resource_claims_rejects_atomic_multi_resource() {
    let eligibilities = sample_eligibilities();

    // 1. Direct atomic multi-resource acquisition is rejected
    assert_eq!(
        claim_atomic_multi_resource(),
        Err(OperationalError::AtomicMultiResourceUnsupported)
    );

    // 2. Phased acquisition: Space first, then Staff
    let mut bay = TreatmentBay::available();
    let mut shift = StaffShift::new(1);

    // Successful phased claim
    let result = execute_phased_claim(
        &mut bay,
        101,
        &mut shift,
        &eligibilities,
        TaskClaimTarget {
            role_id: "role.ed_doctor",
            task_class_id: "task.clinical_assessment",
            zone_id: "zone.acute",
            location_id: "bay_1",
        },
    );
    assert!(result.is_ok());
    assert_eq!(shift.active(), 1);

    // If staff is unavailable on next patient, bay is rolled back and not leaked
    let mut bay_2 = TreatmentBay::available();
    let fail_result = execute_phased_claim(
        &mut bay_2,
        102,
        &mut shift, // shift has active=1, scheduled=1 -> unavailable!
        &eligibilities,
        TaskClaimTarget {
            role_id: "role.ed_doctor",
            task_class_id: "task.clinical_assessment",
            zone_id: "zone.acute",
            location_id: "bay_2",
        },
    );
    assert_eq!(fail_result, Err(OperationalError::Unavailable));
}

#[test]
fn test_unsupported_multi_role_rejection_and_composite_team() {
    // Routine single-provider procedures succeed normally
    assert!(validate_procedure_staffing(ProcedureKind::RoutineAssessment, false).is_ok());
    assert!(validate_procedure_staffing(ProcedureKind::DiagnosticScan, false).is_ok());
    assert!(validate_procedure_staffing(ProcedureKind::MedicationAdministration, false).is_ok());

    // Multi-person procedures without dedicated composite team are rejected
    assert_eq!(
        validate_procedure_staffing(ProcedureKind::ProceduralSedation, false),
        Err(OperationalError::UnsupportedMultiRole)
    );
    assert_eq!(
        validate_procedure_staffing(ProcedureKind::Resuscitation, false),
        Err(OperationalError::UnsupportedMultiRole)
    );
    assert_eq!(
        validate_procedure_staffing(ProcedureKind::AssistedTransfer, false),
        Err(OperationalError::UnsupportedMultiRole)
    );

    // Multi-person procedures WITH dedicated composite team succeed
    assert!(validate_procedure_staffing(ProcedureKind::ProceduralSedation, true).is_ok());
    assert!(validate_procedure_staffing(ProcedureKind::Resuscitation, true).is_ok());
    assert!(validate_procedure_staffing(ProcedureKind::AssistedTransfer, true).is_ok());
}
