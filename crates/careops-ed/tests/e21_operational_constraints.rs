use careops_ed::operations::{
    claim_task, handover, preempt, BayPhase, HeldWork, OperationalError, PreemptConsent,
    StaffShift, TreatmentBay, Ward, WorkAccount,
};
use careops_ed::{Eligibility, KnowledgeStatus, StaffCount, StaffRole, TaskClass};

fn triage_eligibility() -> Vec<Eligibility> {
    vec![Eligibility {
        staff_role_id: "role.triage_nurse".into(),
        task_class_id: "task.triage_assessment".into(),
        zone_id: "zone.triage".into(),
        location_id: Some("loc.triage_1".into()),
    }]
}

fn sample_roles() -> (StaffRole, StaffRole, TaskClass) {
    let counts = |count| {
        vec![StaffCount {
            basis: "present".into(),
            status: KnowledgeStatus::Known,
            count: Some(count),
            reference: "profile.fixture".into(),
            reason: None,
        }]
    };
    (
        StaffRole {
            staff_role_id: "role.general_nurse".into(),
            name: "General Nurse".into(),
            counts: counts(2),
        },
        StaffRole {
            staff_role_id: "role.triage_nurse".into(),
            name: "Triage Nurse".into(),
            counts: counts(1),
        },
        TaskClass {
            task_class_id: "task.triage_assessment".into(),
            name: "Triage Assessment".into(),
            required_resource_classes: vec!["triage_desk".into()],
        },
    )
}

#[test]
fn ineligible_or_off_zone_staff_cannot_claim() {
    let (general, triage, task) = sample_roles();
    let rows = triage_eligibility();
    let mut shift = StaffShift::new(1);

    assert_eq!(
        claim_task(
            &mut shift,
            &rows,
            &general.staff_role_id,
            &task.task_class_id,
            "zone.triage",
            "loc.triage_1",
        ),
        Err(OperationalError::Ineligible)
    );
    assert_eq!(shift.active(), 0);

    assert_eq!(
        claim_task(
            &mut shift,
            &rows,
            &triage.staff_role_id,
            &task.task_class_id,
            "zone.resus",
            "loc.resus_1",
        ),
        Err(OperationalError::Ineligible)
    );
    assert_eq!(
        claim_task(
            &mut shift,
            &rows,
            &triage.staff_role_id,
            &task.task_class_id,
            "zone.triage",
            "loc.triage_1",
        ),
        Ok(())
    );
    assert_eq!(shift.active(), 1);
    assert_eq!(
        claim_task(
            &mut shift,
            &rows,
            &triage.staff_role_id,
            &task.task_class_id,
            "zone.triage",
            "loc.triage_1",
        ),
        Err(OperationalError::Unavailable)
    );
}

#[test]
fn breaks_and_shift_shrink_keep_in_flight_work() {
    let rows = triage_eligibility();
    let mut shift = StaffShift::new(2);
    assert!(claim_task(
        &mut shift,
        &rows,
        "role.triage_nurse",
        "task.triage_assessment",
        "zone.triage",
        "loc.triage_1",
    )
    .is_ok());
    assert!(claim_task(
        &mut shift,
        &rows,
        "role.triage_nurse",
        "task.triage_assessment",
        "zone.triage",
        "loc.triage_1",
    )
    .is_ok());

    assert_eq!(shift.begin_break(1), Err(OperationalError::CapacityInUse));
    assert_eq!(
        shift.shrink_scheduled(1),
        Err(OperationalError::CapacityInUse)
    );
    assert_eq!(shift.scheduled(), 2);
    assert_eq!(shift.active(), 2);

    shift.release_assignment().unwrap();
    assert_eq!(shift.shrink_scheduled(1), Ok(()));
    assert_eq!(shift.active(), 1);
    assert_eq!(shift.claimable(), 0);
    assert_eq!(
        claim_task(
            &mut shift,
            &rows,
            "role.triage_nurse",
            "task.triage_assessment",
            "zone.triage",
            "loc.triage_1",
        ),
        Err(OperationalError::Unavailable)
    );

    shift.release_assignment().unwrap();
    assert_eq!(shift.begin_break(1), Ok(()));
    assert_eq!(shift.claimable(), 0);
    assert_eq!(shift.active(), 0);
    shift.end_break(1).unwrap();
    assert!(claim_task(
        &mut shift,
        &rows,
        "role.triage_nurse",
        "task.triage_assessment",
        "zone.triage",
        "loc.triage_1",
    )
    .is_ok());
    assert!(!claim_task(
        &mut shift,
        &rows,
        "role.triage_nurse",
        "task.triage_assessment",
        "zone.triage",
        "loc.triage_1",
    )
    .is_ok());
}

#[test]
fn preemption_needs_higher_acuity_and_consent_and_keeps_work() {
    let account = WorkAccount {
        elapsed_ticks: 3,
        total_ticks: 10,
    };

    let yielded = preempt(&account, 3, PreemptConsent::Preemptible, 1).unwrap();
    assert!(yielded.yielded);
    assert_eq!(yielded.retained_ticks, 7);
    assert_eq!(
        account.elapsed_ticks + yielded.retained_ticks,
        account.total_ticks
    );

    let protected = preempt(&account, 3, PreemptConsent::NonPreemptible, 1).unwrap();
    assert!(!protected.yielded);
    assert_eq!(protected.retained_ticks, 7);

    let same_or_lower = preempt(&account, 2, PreemptConsent::Preemptible, 3).unwrap();
    assert!(!same_or_lower.yielded);
    assert_eq!(account.elapsed_ticks, 3);

    let broken = WorkAccount {
        elapsed_ticks: 11,
        total_ticks: 10,
    };
    assert_eq!(
        preempt(&broken, 3, PreemptConsent::Preemptible, 1),
        Err(OperationalError::Accounting)
    );
}

#[test]
fn handover_moves_the_assignment_without_dropping_work() {
    let rows = triage_eligibility();
    let mut offgoing = StaffShift::new(1);
    let mut incoming = StaffShift::new(1);
    claim_task(
        &mut offgoing,
        &rows,
        "role.triage_nurse",
        "task.triage_assessment",
        "zone.triage",
        "loc.triage_1",
    )
    .unwrap();
    let work = HeldWork {
        elapsed_ticks: 4,
        total_ticks: 9,
        task_class_id: "task.triage_assessment".into(),
        zone_id: "zone.triage".into(),
        location_id: "loc.triage_1".into(),
    };

    let mut wrong_zone = work.clone();
    wrong_zone.zone_id = "zone.resus".into();
    assert_eq!(
        handover(
            &mut offgoing,
            &mut incoming,
            &wrong_zone,
            "role.triage_nurse",
            "role.triage_nurse",
            &rows,
        ),
        Err(OperationalError::Ineligible)
    );
    assert_eq!(offgoing.active(), 1);
    assert_eq!(incoming.active(), 0);

    incoming.begin_break(1).unwrap();
    assert_eq!(
        handover(
            &mut offgoing,
            &mut incoming,
            &work,
            "role.triage_nurse",
            "role.triage_nurse",
            &rows,
        ),
        Err(OperationalError::Unavailable)
    );
    incoming.end_break(1).unwrap();

    handover(
        &mut offgoing,
        &mut incoming,
        &work,
        "role.triage_nurse",
        "role.triage_nurse",
        &rows,
    )
    .unwrap();
    assert_eq!(offgoing.active(), 0);
    assert_eq!(incoming.active(), 1);
    assert_eq!(work.elapsed_ticks, 4);
    assert_eq!(work.total_ticks - work.elapsed_ticks, 5);
}

#[test]
fn cleaning_and_reservation_keep_a_bay_exclusive() {
    let mut bay = TreatmentBay::available();
    assert_eq!(bay.phase(), BayPhase::Available);

    bay.reserve(101).unwrap();
    assert_eq!(bay.reserve(102), Err(OperationalError::SpaceUnavailable));
    assert_eq!(bay.occupy(102), Err(OperationalError::ReservedForOther));
    bay.occupy(101).unwrap();
    assert!(matches!(
        bay.phase(),
        BayPhase::Occupied {
            patient_id: 101,
            diagnostic_blocked: false,
            boarding: false,
        }
    ));

    bay.vacate_for_cleaning(15).unwrap();
    assert_eq!(bay.occupy(103), Err(OperationalError::CleaningInProgress));
    assert_eq!(
        bay.finish_cleaning(13),
        Err(OperationalError::CleaningInProgress)
    );
    bay.finish_cleaning(15).unwrap();
    assert_eq!(bay.phase(), BayPhase::Available);
    bay.occupy(104).unwrap();
    assert_eq!(bay.occupy(105), Err(OperationalError::SpaceUnavailable));
}

#[test]
fn blocked_diagnostics_and_boarding_do_not_free_capacity() {
    let mut ward = Ward::new(4);
    for patient in 1..=4 {
        ward.place(patient).unwrap();
    }
    assert_eq!(ward.occupied_patients(), 4);
    assert_eq!(ward.free_bays(), 0);
    assert_eq!(ward.place(5), Err(OperationalError::SpaceUnavailable));

    let bay = ward.bay_mut(2).unwrap();
    bay.block_for_diagnostics().unwrap();
    assert_eq!(
        bay.vacate_for_cleaning(20),
        Err(OperationalError::DiagnosticBlocked)
    );
    assert!(matches!(
        bay.phase(),
        BayPhase::Occupied {
            patient_id: 3,
            diagnostic_blocked: true,
            ..
        }
    ));
    bay.clear_diagnostics().unwrap();
    bay.begin_boarding().unwrap();
    assert_eq!(
        bay.vacate_for_cleaning(20),
        Err(OperationalError::BoardingRetained)
    );
    assert_eq!(ward.occupied_patients(), 4);
    assert_eq!(ward.free_bays() + ward.occupied_patients(), ward.len());

    ward.bay_mut(2).unwrap().accept_transfer(20).unwrap();
    assert_eq!(ward.occupied_patients(), 3);
    assert_eq!(ward.free_bays(), 0);
    assert_eq!(ward.place(6), Err(OperationalError::SpaceUnavailable));
    ward.bay_mut(2).unwrap().finish_cleaning(20).unwrap();
    assert_eq!(ward.place(6).unwrap(), 2);
    assert_eq!(ward.occupied_patients() + ward.free_bays(), 4);
}
