//! E2.2 minimal agent task dispatch: urgency, FIFO, eligibility, and stable ties.

use careops_ed::disposition::operations::{
    select_dispatch_candidate, StaffShift, TaskCandidate,
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
            staff_role_id: "role.ed_doctor".into(),
            task_class_id: "task.clinical_assessment".into(),
            zone_id: "zone.acute".into(),
            location_id: None,
        },
    ]
}

fn acute_assessment(patient_id: u64, task_id: &str, acuity: u8, enqueue_tick: u64) -> TaskCandidate {
    TaskCandidate {
        task_id: task_id.into(),
        patient_id,
        acuity,
        enqueue_tick,
        task_class_id: "task.clinical_assessment".into(),
        zone_id: "zone.acute".into(),
        location_id: format!("bay_{patient_id}"),
    }
}

#[test]
fn urgency_precedes_fifo_and_input_order() {
    let eligibilities = sample_eligibilities();
    let shift = StaffShift::new(2);
    let candidates = vec![
        acute_assessment(401, "task_4_early", 4, 10),
        acute_assessment(202, "task_2_late", 2, 25),
        acute_assessment(201, "task_2_early", 2, 15),
        acute_assessment(101, "task_1_resus", 1, 30),
    ];

    let selected =
        select_dispatch_candidate(&candidates, &shift, "role.ed_doctor", &eligibilities)
            .expect("ATS 1 resuscitation task must win over later arrivals");
    assert_eq!(selected.task_id, "task_1_resus");
    assert_eq!(selected.acuity, 1);

    let without_resus = &candidates[0..3];
    let fifo =
        select_dispatch_candidate(without_resus, &shift, "role.ed_doctor", &eligibilities)
            .expect("two ATS-2 tasks remain");
    assert_eq!(fifo.task_id, "task_2_early");
    assert_eq!(fifo.enqueue_tick, 15);
}

#[test]
fn fifo_then_patient_and_task_ids_are_stable_tie_breakers() {
    let eligibilities = sample_eligibilities();
    let shift = StaffShift::new(1);
    let tied = vec![
        acute_assessment(502, "task_b", 3, 50),
        acute_assessment(501, "task_a", 3, 50),
    ];
    let selected =
        select_dispatch_candidate(&tied, &shift, "role.ed_doctor", &eligibilities).unwrap();
    assert_eq!(selected.patient_id, 501);
    assert_eq!(selected.task_id, "task_a");

    let same_patient = vec![
        TaskCandidate {
            task_id: "task_z".into(),
            patient_id: 600,
            acuity: 3,
            enqueue_tick: 50,
            task_class_id: "task.clinical_assessment".into(),
            zone_id: "zone.acute".into(),
            location_id: "bay_600".into(),
        },
        TaskCandidate {
            task_id: "task_a".into(),
            patient_id: 600,
            acuity: 3,
            enqueue_tick: 50,
            task_class_id: "task.clinical_assessment".into(),
            zone_id: "zone.acute".into(),
            location_id: "bay_600".into(),
        },
    ];
    let by_task_id =
        select_dispatch_candidate(&same_patient, &shift, "role.ed_doctor", &eligibilities)
            .unwrap();
    assert_eq!(by_task_id.task_id, "task_a");
}

#[test]
fn eligibility_uses_exact_zone_and_optional_location() {
    let eligibilities = sample_eligibilities();
    let shift = StaffShift::new(1);
    let triage_only = vec![TaskCandidate {
        task_id: "triage_1".into(),
        patient_id: 1,
        acuity: 3,
        enqueue_tick: 1,
        task_class_id: "task.triage".into(),
        zone_id: "zone.waiting".into(),
        location_id: "loc.triage_desk".into(),
    }];

    assert!(
        select_dispatch_candidate(&triage_only, &shift, "role.triage_nurse", &eligibilities)
            .is_some()
    );
    assert!(
        select_dispatch_candidate(&triage_only, &shift, "role.ed_doctor", &eligibilities).is_none()
    );

    let wrong_desk = vec![TaskCandidate {
        task_id: "triage_wrong".into(),
        patient_id: 2,
        acuity: 3,
        enqueue_tick: 1,
        task_class_id: "task.triage".into(),
        zone_id: "zone.waiting".into(),
        location_id: "loc.other_desk".into(),
    }];
    assert!(
        select_dispatch_candidate(&wrong_desk, &shift, "role.triage_nurse", &eligibilities).is_none()
    );
}

#[test]
fn no_eligible_staff_or_zero_claimable_returns_none() {
    let eligibilities = sample_eligibilities();
    let candidates = vec![acute_assessment(1, "only", 2, 0)];

    let mut busy = StaffShift::new(1);
    busy.begin_break(1).unwrap();
    assert_eq!(busy.claimable(), 0);
    assert!(
        select_dispatch_candidate(&candidates, &busy, "role.ed_doctor", &eligibilities).is_none()
    );

    let shift = StaffShift::new(1);
    assert!(
        select_dispatch_candidate(&[], &shift, "role.ed_doctor", &eligibilities).is_none()
    );
}

#[test]
fn wait_is_distinct_from_dispatch_selection() {
    let eligibilities = sample_eligibilities();
    let shift = StaffShift::new(1);
    let queue = vec![
        acute_assessment(701, "blocked_zone", 1, 5),
        acute_assessment(702, "eligible", 3, 10),
    ];
    let mut ineligible_first = queue;
    ineligible_first[0].zone_id = "zone.resus".into();

    let selected = select_dispatch_candidate(
        &ineligible_first,
        &shift,
        "role.ed_doctor",
        &eligibilities,
    )
    .expect("later eligible task must not be masked by an ineligible urgent head");
    assert_eq!(selected.task_id, "eligible");
    assert_eq!(selected.acuity, 3);
}
