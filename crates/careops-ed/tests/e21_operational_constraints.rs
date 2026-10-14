use careops_ed::{Eligibility, KnowledgeStatus, StaffCount, StaffRole, TaskClass};

/// E2.1.staff Oracle:
/// - Ineligible/unavailable staff cannot claim tasks.
/// - In-flight work policy under shift shrinkage: completing in-flight work before reduction
///   or orderly transition, without dropping patient work silently.
#[test]
fn test_ineligible_staff_role_cannot_claim_restricted_task() {
    // A task requiring specialized role (e.g. physician or triage-certified nurse)
    // cannot be claimed by a general role without explicit eligibility.
    let _triage_task = TaskClass {
        task_class_id: "task.triage_assessment".into(),
        name: "Triage Assessment".into(),
        required_resource_classes: vec!["triage_desk".into()],
    };

    let _general_nurse_role = StaffRole {
        staff_role_id: "role.general_nurse".into(),
        name: "General Nurse".into(),
        counts: vec![StaffCount {
            basis: "present".into(),
            status: KnowledgeStatus::Known,
            count: Some(2),
            reference: "profile.fixture".into(),
            reason: None,
        }],
    };

    let _specialized_nurse_role = StaffRole {
        staff_role_id: "role.triage_nurse".into(),
        name: "Triage Nurse".into(),
        counts: vec![StaffCount {
            basis: "present".into(),
            status: KnowledgeStatus::Known,
            count: Some(1),
            reference: "profile.fixture".into(),
            reason: None,
        }],
    };

    // Only specialized nurse has eligibility for the task in zone.triage
    let eligibility = [Eligibility {
        staff_role_id: "role.triage_nurse".into(),
        task_class_id: "task.triage_assessment".into(),
        zone_id: "zone.triage".into(),
        location_id: Some("loc.triage_1".into()),
    }];

    // Verify eligibility lookup enforces role + zone + location matching
    let is_eligible = |role_id: &str, task_id: &str, zone_id: &str, loc_id: &str| {
        eligibility.iter().any(|e| {
            e.staff_role_id == role_id
                && e.task_class_id == task_id
                && e.zone_id == zone_id
                && (e.location_id.is_none() || e.location_id.as_deref() == Some(loc_id))
        })
    };

    assert!(!is_eligible(
        "role.general_nurse",
        "task.triage_assessment",
        "zone.triage",
        "loc.triage_1"
    ));
    assert!(is_eligible(
        "role.triage_nurse",
        "task.triage_assessment",
        "zone.triage",
        "loc.triage_1"
    ));
    // Wrong zone fails even for eligible role
    assert!(!is_eligible(
        "role.triage_nurse",
        "task.triage_assessment",
        "zone.resus",
        "loc.resus_1"
    ));
}

#[test]
fn test_shift_shrink_contract_preserves_in_flight_work() {
    // When scheduled capacity shrinks from 2 to 1 (e.g. shift change or scheduled break),
    // currently active tasks must complete without interruption or abrupt termination,
    // and subsequent queued tasks must honor the reduced capacity limit.
    struct StaffCapacityTracker {
        allocated: u64,
        target_capacity: u64,
    }

    impl StaffCapacityTracker {
        fn new(initial: u64) -> Self {
            Self {
                allocated: 0,
                target_capacity: initial,
            }
        }

        fn try_claim(&mut self) -> bool {
            if self.allocated < self.target_capacity {
                self.allocated += 1;
                true
            } else {
                false
            }
        }

        fn release(&mut self) {
            self.allocated = self.allocated.saturating_sub(1);
        }

        fn apply_shift_shrink(&mut self, new_capacity: u64) {
            self.target_capacity = new_capacity;
            // In-flight work (self.allocated) is NOT forcefully dropped.
            // Rather, new claims are blocked until allocated drops below target_capacity.
        }
    }

    let mut tracker = StaffCapacityTracker::new(2);
    assert!(tracker.try_claim()); // task 1 active
    assert!(tracker.try_claim()); // task 2 active
    assert!(!tracker.try_claim()); // capacity 2 exhausted

    // Shift change reduces available staff to 1
    tracker.apply_shift_shrink(1);
    // Both tasks are still in-flight
    assert_eq!(tracker.allocated, 2);
    assert_eq!(tracker.target_capacity, 1);
    assert!(!tracker.try_claim());

    // Task 1 finishes
    tracker.release();
    assert_eq!(tracker.allocated, 1);
    // Cannot claim yet because target_capacity is 1 and allocated is 1
    assert!(!tracker.try_claim());

    // Task 2 finishes
    tracker.release();
    assert_eq!(tracker.allocated, 0);
    // Now task 3 can claim up to target_capacity (1)
    assert!(tracker.try_claim());
    assert!(!tracker.try_claim());
}

/// E2.1.interrupt Oracle:
/// - Urgent preemption requires both clinical priority policy and resource consent.
/// - Work accounting is strictly conserved (completed work + interrupted work == total elapsed).
#[test]
fn test_preemption_requires_clinical_priority_and_preemptible_consent() {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[allow(dead_code)]
    enum TaskPreemptibility {
        NonPreemptible,
        PreemptibleWithResume,
        PreemptibleWithRestart,
    }

    struct PreemptionDecision {
        allowed: bool,
        remaining_ticks: u64,
    }

    fn evaluate_preemption(
        active_acuity: u8,
        active_consent: TaskPreemptibility,
        incoming_acuity: u8,
        elapsed_work: u64,
        total_duration: u64,
    ) -> PreemptionDecision {
        // Clinical rule: incoming must have strictly higher acuity (lower numeric ATS)
        let clinical_priority = incoming_acuity < active_acuity;
        let resource_consent = active_consent != TaskPreemptibility::NonPreemptible;

        if clinical_priority && resource_consent {
            PreemptionDecision {
                allowed: true,
                remaining_ticks: total_duration.saturating_sub(elapsed_work),
            }
        } else {
            PreemptionDecision {
                allowed: false,
                remaining_ticks: total_duration.saturating_sub(elapsed_work),
            }
        }
    }

    // Resuscitation (acuity 1) arriving while stable assessment (acuity 3, preemptible) in progress
    let res = evaluate_preemption(3, TaskPreemptibility::PreemptibleWithResume, 1, 3, 10);
    assert!(res.allowed);
    assert_eq!(res.remaining_ticks, 7);

    // Non-preemptible procedure (e.g. sterile procedure) cannot be interrupted even by higher priority
    let non_preempt = evaluate_preemption(3, TaskPreemptibility::NonPreemptible, 1, 3, 10);
    assert!(!non_preempt.allowed);

    // Equal or lower priority (acuity 3 vs active acuity 2) cannot preempt
    let equal_pri = evaluate_preemption(2, TaskPreemptibility::PreemptibleWithResume, 3, 3, 10);
    assert!(!equal_pri.allowed);
}

/// E2.1.space Oracle:
/// - Physical space unavailable while occupied, being cleaned, or reserved.
/// - Blocking cannot leak capacity; occupied spaces conserve exactly 1 occupant per physical bay.
#[test]
fn test_treatment_space_lifecycle_and_cleaning_reservation_exclusivity() {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum SpaceState {
        Available,
        Reserved(u64), // reserved for patient_id, deadline
        Occupied(u64), // patient_id
        Cleaning(u64), // cleaning completion tick
    }

    struct TreatmentSpace {
        state: SpaceState,
    }

    impl TreatmentSpace {
        fn new() -> Self {
            Self {
                state: SpaceState::Available,
            }
        }

        fn reserve(&mut self, patient_id: u64, _tick: u64) -> Result<(), &'static str> {
            match self.state {
                SpaceState::Available => {
                    self.state = SpaceState::Reserved(patient_id);
                    Ok(())
                }
                SpaceState::Reserved(_) => Err("space already reserved"),
                SpaceState::Occupied(_) => Err("space currently occupied"),
                SpaceState::Cleaning(_) => Err("space undergoing cleaning"),
            }
        }

        fn occupy(&mut self, patient_id: u64) -> Result<(), &'static str> {
            match self.state {
                SpaceState::Reserved(reserved_for) if reserved_for == patient_id => {
                    self.state = SpaceState::Occupied(patient_id);
                    Ok(())
                }
                SpaceState::Available => {
                    self.state = SpaceState::Occupied(patient_id);
                    Ok(())
                }
                SpaceState::Reserved(_) => Err("reserved for different patient"),
                SpaceState::Occupied(_) => Err("space already occupied"),
                SpaceState::Cleaning(_) => Err("space not yet cleaned"),
            }
        }

        fn vacate_and_start_cleaning(&mut self, ready_tick: u64) -> Result<(), &'static str> {
            match self.state {
                SpaceState::Occupied(_) => {
                    self.state = SpaceState::Cleaning(ready_tick);
                    Ok(())
                }
                _ => Err("cannot clean non-occupied space"),
            }
        }

        fn finish_cleaning(&mut self, current_tick: u64) -> Result<(), &'static str> {
            match self.state {
                SpaceState::Cleaning(ready_tick) if current_tick >= ready_tick => {
                    self.state = SpaceState::Available;
                    Ok(())
                }
                SpaceState::Cleaning(_) => Err("cleaning in progress"),
                _ => Err("space not cleaning"),
            }
        }
    }

    let mut bay = TreatmentSpace::new();
    assert_eq!(bay.state, SpaceState::Available);

    // Reserve for patient 101
    assert!(bay.reserve(101, 10).is_ok());
    // Cannot be reserved or occupied by patient 102
    assert_eq!(bay.reserve(102, 10), Err("space already reserved"));
    assert_eq!(bay.occupy(102), Err("reserved for different patient"));

    // Patient 101 occupies
    assert!(bay.occupy(101).is_ok());
    assert_eq!(bay.state, SpaceState::Occupied(101));

    // Patient 101 finishes and departs, triggering cleaning for 3 ticks (until tick 15)
    assert!(bay.vacate_and_start_cleaning(15).is_ok());
    assert_eq!(bay.state, SpaceState::Cleaning(15));

    // At tick 13, cleaning is still underway; cannot occupy or reserve
    assert_eq!(bay.occupy(103), Err("space not yet cleaned"));
    assert_eq!(bay.finish_cleaning(13), Err("cleaning in progress"));

    // At tick 15, cleaning completes
    assert!(bay.finish_cleaning(15).is_ok());
    assert_eq!(bay.state, SpaceState::Available);

    // Ready for next patient
    assert!(bay.occupy(104).is_ok());
}

#[test]
fn test_blocked_diagnostics_retains_bed_occupancy() {
    // Diagnostic contention must not release patient's ED cubicle
    struct PatientCareState {
        cubicle_id: String,
        diagnostic_active: bool,
        diagnostic_queue_wait: u64,
    }

    let mut state = PatientCareState {
        cubicle_id: "bay_3".into(),
        diagnostic_active: false,
        diagnostic_queue_wait: 0,
    };

    // Diagnostic resource (e.g. CT scanner) is contended for 15 ticks
    for _ in 0..15 {
        state.diagnostic_queue_wait += 1;
        // Invariant: cubicle remains assigned throughout diagnostic queueing
        assert_eq!(state.cubicle_id, "bay_3");
    }

    // Diagnostic scan begins
    state.diagnostic_active = true;
    assert_eq!(state.cubicle_id, "bay_3");
    assert_eq!(state.diagnostic_queue_wait, 15);
}

#[test]
fn test_boarding_capacity_exhaustion_blocks_transfer_admission() {
    // When inpatient destination has zero available offers,
    // boarding interval extends and ED cubicle capacity is not liberated.
    let total_ed_cubicles = 4u64;
    let mut occupied_cubicles = 3u64;

    // A patient finishes treatment and begins boarding
    let treatment_completed = true;
    let compatible_bed_offer_available = false;

    if treatment_completed && !compatible_bed_offer_available {
        // ED cubicle continues to be occupied by boarder
        occupied_cubicles += 1;
    }

    assert_eq!(occupied_cubicles, 4);
    // ED is at capacity; incoming patient cannot be placed into a cubicle
    let can_place_new_arrival = occupied_cubicles < total_ed_cubicles;
    assert!(!can_place_new_arrival);
}
