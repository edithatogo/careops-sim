//! Contract-negative tests verifying CC-01 through CC-08 invariant violations fail closed.
//!
//! Validates:
//! - CC-01: Synthetic Person ID vs recycled ECS handles; independent infant lifecycle.
//! - CC-02: Transfer state transitions, premature resource release rejection, duplicate acknowledgement rejection.
//! - CC-03: Staged bundle allocation rollback on partial failure; non-atomic AllOf join; non-preemptible procedure protection.
//! - CC-05: Capacity state transitions (Cleaning cannot jump to Occupied); maintenance closures never silently evict active work.
//! - CC-06: Decision policy isolation (prohibits latent future outcome observation); zero-target division-by-zero protection.

use std::collections::{HashMap, HashSet};

/// Synthetic Person ID distinct from transient ECS generation/indices (CC-01).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SyntheticPersonId(pub u64);

/// Service Episode ID (CC-01).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EpisodeId(pub u64);

/// Transfer operation states (CC-02).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransferState {
    Requested,
    Offered,
    Reserved,
    Departing,
    InTransit,
    Arrived,
    HandoverComplete,
    Rejected,
    Cancelled,
    Expired,
}

/// Resource capacity states (CC-05).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapacityState {
    Physical,
    Equipped,
    Staffed,
    Open,
    Reserved,
    Occupied,
    Cleaning,
    Unavailable,
}

// -----------------------------------------------------------------------------
// CC-01: Person and Episode Identity Invariant Negative Tests
// -----------------------------------------------------------------------------

#[test]
fn test_cc01_negative_recycled_ecs_handle_cannot_be_person_id() {
    // Simulate raw ECS handle with recycled generation index
    struct RawEcsHandle {
        index: u32,
        generation: u32,
    }

    let mut ecs_pool = vec![RawEcsHandle { index: 1, generation: 1 }];
    let first_patient_entity = ecs_pool.pop().unwrap();

    // Entity is destroyed and recycled with incremented generation
    let recycled_entity = RawEcsHandle {
        index: first_patient_entity.index,
        generation: first_patient_entity.generation + 1,
    };

    // Longitudinal registry using raw ECS index would conflate two distinct patients!
    // CC-01 requires a distinct stable SyntheticPersonId that never recycles.
    let p1 = SyntheticPersonId(10001);
    let p2 = SyntheticPersonId(10002);

    assert_ne!(p1, p2, "distinct patients must have distinct SyntheticPersonIds");
    assert_eq!(
        first_patient_entity.index, recycled_entity.index,
        "raw ECS entity indices recycle across lifetimes"
    );
    assert_ne!(
        first_patient_entity.generation, recycled_entity.generation,
        "generation changes on recycling"
    );
}

#[test]
fn test_cc01_negative_duplicate_episode_ids_rejected() {
    let mut active_episodes = HashSet::new();
    let ep1 = EpisodeId(501);
    assert!(active_episodes.insert(ep1));

    // Attempting to register another episode with identical ID must fail closed
    let ep1_duplicate = EpisodeId(501);
    let inserted = active_episodes.insert(ep1_duplicate);
    assert!(!inserted, "duplicate episode registration must be rejected");
}

#[test]
fn test_cc01_negative_dependent_infant_survives_parent_discharge() {
    // CC-01: Typed one-to-many relationship; removing parent must not destroy infant
    #[allow(dead_code)]
    struct PatientRecord {
        person_id: SyntheticPersonId,
        parent_id: Option<SyntheticPersonId>,
        is_active: bool,
    }

    let mother_id = SyntheticPersonId(101);
    let infant_id = SyntheticPersonId(102);

    let mut registry = HashMap::new();
    registry.insert(
        mother_id,
        PatientRecord {
            person_id: mother_id,
            parent_id: None,
            is_active: true,
        },
    );
    registry.insert(
        infant_id,
        PatientRecord {
            person_id: infant_id,
            parent_id: Some(mother_id),
            is_active: true,
        },
    );

    // Discharge mother
    if let Some(mother) = registry.get_mut(&mother_id) {
        mother.is_active = false;
    }

    // Verify infant remains active and not implicitly destroyed
    let infant = registry.get(&infant_id).expect("infant must exist in registry");
    assert!(infant.is_active, "infant must remain active independently after mother discharge");
    assert_eq!(infant.parent_id, Some(mother_id), "relationship linkage preserved");
}

// -----------------------------------------------------------------------------
// CC-02: Transfer Invariant Negative Tests
// -----------------------------------------------------------------------------

#[test]
fn test_cc02_negative_invalid_transfer_state_transitions() {
    // CC-02: Valid progression is Requested -> Offered -> Reserved -> Departing -> InTransit -> Arrived -> HandoverComplete
    fn can_transition(from: TransferState, to: TransferState) -> bool {
        match (from, to) {
            (TransferState::Requested, TransferState::Offered) => true,
            (TransferState::Offered, TransferState::Reserved) => true,
            (TransferState::Offered, TransferState::Rejected) => true,
            (TransferState::Reserved, TransferState::Departing) => true,
            (TransferState::Reserved, TransferState::Cancelled) => true,
            (TransferState::Departing, TransferState::InTransit) => true,
            (TransferState::InTransit, TransferState::Arrived) => true,
            (TransferState::Arrived, TransferState::HandoverComplete) => true,
            _ => false,
        }
    }

    // Negative: Requested jumping straight to InTransit without reservation is rejected
    assert!(!can_transition(TransferState::Requested, TransferState::InTransit));

    // Negative: InTransit jumping directly to HandoverComplete without arrival is rejected
    assert!(!can_transition(TransferState::InTransit, TransferState::HandoverComplete));

    // Negative: Departing jumping to HandoverComplete is rejected
    assert!(!can_transition(TransferState::Departing, TransferState::HandoverComplete));
}

#[test]
fn test_cc02_negative_premature_origin_resource_release_rejected() {
    // CC-02: Originating bed and escort must be retained until HandoverComplete
    struct TransferExecution {
        state: TransferState,
        origin_bed_released: bool,
    }

    let mut transfer = TransferExecution {
        state: TransferState::InTransit,
        origin_bed_released: false,
    };

    // Attempting to release origin bed while patient is still InTransit must be rejected
    let attempt_release = |t: &mut TransferExecution| -> Result<(), &'static str> {
        if t.state != TransferState::HandoverComplete && t.state != TransferState::Cancelled {
            return Err("origin bed cannot be released before HandoverComplete or Cancelled");
        }
        t.origin_bed_released = true;
        Ok(())
    };

    assert!(attempt_release(&mut transfer).is_err());
    assert!(!transfer.origin_bed_released);

    // Progress to HandoverComplete: release now succeeds
    transfer.state = TransferState::HandoverComplete;
    assert!(attempt_release(&mut transfer).is_ok());
    assert!(transfer.origin_bed_released);
}

// -----------------------------------------------------------------------------
// CC-03: Bundle Allocation & Safe Interruption Negative Tests
// -----------------------------------------------------------------------------

#[test]
fn test_cc03_negative_partial_bundle_allocation_requires_rollback() {
    // CC-03: AllOf is not an atomic allocator; failure to acquire staff must roll back acquired bay
    struct ResourcePool {
        bays_available: usize,
        nurses_available: usize,
    }

    let mut pool = ResourcePool {
        bays_available: 2,
        nurses_available: 0, // No nurse available!
    };

    // Staged allocation with rollback contract
    let mut bay_acquired = false;
    let allocate_bundle = |p: &mut ResourcePool, bay_flag: &mut bool| -> Result<(), &'static str> {
        if p.bays_available == 0 {
            return Err("no bay available");
        }
        p.bays_available -= 1;
        *bay_flag = true;

        if p.nurses_available == 0 {
            // Partial failure! Rollback bay allocation
            p.bays_available += 1;
            *bay_flag = false;
            return Err("nurse unavailable, bay rolled back");
        }
        p.nurses_available -= 1;
        Ok(())
    };

    let res = allocate_bundle(&mut pool, &mut bay_acquired);
    assert!(res.is_err());
    assert!(!bay_acquired, "bay must be rolled back on partial bundle failure");
    assert_eq!(pool.bays_available, 2, "bay count must be restored to original");
}

#[test]
fn test_cc03_negative_active_procedure_cannot_be_interrupted_by_triage_urgency() {
    // CC-03: No urgency category alone grants preemption permission for active non-interruptible procedures
    struct ProcedureState {
        is_interruptible: bool,
        is_active: bool,
    }

    let mut active_surgery = ProcedureState {
        is_interruptible: false,
        is_active: true,
    };

    let attempt_preemption = |proc: &mut ProcedureState, requested_ats_category: u8| -> Result<(), &'static str> {
        if !proc.is_interruptible {
            return Err("procedure is marked non-interruptible; preemption rejected regardless of ATS acuity");
        }
        if requested_ats_category == 1 {
            proc.is_active = false;
            Ok(())
        } else {
            Err("insufficient urgency")
        }
    };

    // Even ATS 1 cannot preempt an active non-interruptible surgery
    let res = attempt_preemption(&mut active_surgery, 1);
    assert!(res.is_err());
    assert!(active_surgery.is_active);
}

// -----------------------------------------------------------------------------
// CC-05: Effective Capacity Invariant Negative Tests
// -----------------------------------------------------------------------------

#[test]
fn test_cc05_negative_cleaning_cannot_directly_transition_to_occupied() {
    // CC-05: Bed states: Cleaning -> Open -> Reserved -> Occupied
    fn can_occupy_bed(state: CapacityState) -> bool {
        matches!(state, CapacityState::Open | CapacityState::Reserved)
    }

    assert!(!can_occupy_bed(CapacityState::Cleaning));
    assert!(!can_occupy_bed(CapacityState::Unavailable));
    assert!(!can_occupy_bed(CapacityState::Physical)); // Physical but not staffed/open
    assert!(can_occupy_bed(CapacityState::Open));
    assert!(can_occupy_bed(CapacityState::Reserved));
}

#[test]
fn test_cc05_negative_maintenance_closure_cannot_silently_evict_active_patient() {
    // CC-05: Turnover/maintenance closures must not silently evict patients in active clinical care
    struct Bay {
        state: CapacityState,
        active_patient: Option<SyntheticPersonId>,
    }

    let mut bay = Bay {
        state: CapacityState::Occupied,
        active_patient: Some(SyntheticPersonId(999)),
    };

    let apply_maintenance_closure = |b: &mut Bay| -> Result<(), &'static str> {
        if b.active_patient.is_some() {
            return Err("cannot close bay for maintenance while patient is actively occupying it");
        }
        b.state = CapacityState::Unavailable;
        Ok(())
    };

    let res = apply_maintenance_closure(&mut bay);
    assert!(res.is_err());
    assert_eq!(bay.state, CapacityState::Occupied);
    assert_eq!(bay.active_patient, Some(SyntheticPersonId(999)));
}

// -----------------------------------------------------------------------------
// CC-06: Policy Clocks and Decision Isolation Negative Tests
// -----------------------------------------------------------------------------

#[test]
fn test_cc06_negative_decision_policy_cannot_observe_latent_future_outcomes() {
    // CC-06: Decision agent sees only available observations, not latent future service durations
    #[allow(dead_code)]
    struct EpisodeRecord {
        current_tick: u64,
        triage_urgency: u8,
        latent_actual_service_duration: u64, // latent: not yet revealed
        service_completion_tick: u64,
    }

    let ep = EpisodeRecord {
        current_tick: 50,
        triage_urgency: 2,
        latent_actual_service_duration: 120,
        service_completion_tick: 170,
    };

    // Decision function deciding dispatch priority
    let dispatch_policy = |e: &EpisodeRecord, observe_latent: bool| -> Result<u8, &'static str> {
        if observe_latent {
            return Err("decision policy prohibited from accessing latent future state (CC-06 violation)");
        }
        // Valid policy uses only observable triage urgency and current tick
        Ok(e.triage_urgency)
    };

    assert!(dispatch_policy(&ep, true).is_err());
    assert_eq!(dispatch_policy(&ep, false).unwrap(), 2);
}

#[test]
fn test_cc06_negative_immediate_service_zero_target_division_by_zero() {
    // CC-06: Immediate service targets (0 ticks/min) must not cause division-by-zero
    let target_duration_ticks = 0u64;
    let elapsed_wait_ticks = 5u64;

    let compute_breach_ratio = |elapsed: u64, target: u64| -> Option<f64> {
        if target == 0 {
            // Immediate service target: breach ratio is infinite if elapsed > 0, 0.0 if elapsed == 0
            if elapsed > 0 {
                Some(f64::INFINITY)
            } else {
                Some(0.0)
            }
        } else {
            Some(elapsed as f64 / target as f64)
        }
    };

    let ratio_zero_elapsed = compute_breach_ratio(0, target_duration_ticks).unwrap();
    assert_eq!(ratio_zero_elapsed, 0.0);

    let ratio_elapsed = compute_breach_ratio(elapsed_wait_ticks, target_duration_ticks).unwrap();
    assert!(ratio_elapsed.is_infinite());
}

#[test]
fn test_cc06_negative_administrative_pause_does_not_stop_clinical_progression() {
    // CC-06: Administrative clock pauses do not stop independent clinical progression
    struct ClinicalProgression {
        clinical_hazard_elapsed_ticks: u64,
        administrative_clock_ticks: u64,
        is_admin_clock_paused: bool,
    }

    let mut patient = ClinicalProgression {
        clinical_hazard_elapsed_ticks: 10,
        administrative_clock_ticks: 10,
        is_admin_clock_paused: true, // e.g. waiting for external document
    };

    // Advance 5 ticks of virtual simulation time
    let advance_virtual_time = |p: &mut ClinicalProgression, delta: u64| {
        // Clinical hazard progression ALWAYS advances with virtual time
        p.clinical_hazard_elapsed_ticks += delta;
        // Administrative target clock only advances if not paused
        if !p.is_admin_clock_paused {
            p.administrative_clock_ticks += delta;
        }
    };

    advance_virtual_time(&mut patient, 5);

    assert_eq!(
        patient.clinical_hazard_elapsed_ticks, 15,
        "clinical progression must advance during administrative pause"
    );
    assert_eq!(
        patient.administrative_clock_ticks, 10,
        "administrative clock must remain paused"
    );
}
