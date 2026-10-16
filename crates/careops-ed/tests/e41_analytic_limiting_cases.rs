#![forbid(unsafe_code)]

//! E4.1 Analytic Limiting Cases, Differential Invariants, and Queueing Law Verification.
//!
//! # Analytic Foundations and Verification vs. Validation Distinction
//!
//! This test suite provides rigorous formal verification of the discrete-event and agent-based
//! operational engine (`careops-ed`) against exact analytic limiting cases, queueing conservation
//! laws, and differential execution contracts.
//!
//! ## Software Verification vs. Domain Validation
//!
//! A foundational principle of simulation science (Sargent 2013; Law 2015; Kleijnen 1995) is the
//! strict distinction between **Software Verification** and **Domain Validation**:
//!
//! - **Software Verification ("Building the model right")**:
//!   Ensures that the programmed mathematical and logical equations are implemented correctly
//!   without coding errors, race conditions, memory corruption, capacity leaks, or numerical drift.
//!   Verification is evaluated against closed-form analytic solutions (Kendall D/D/1, zero-contention
//!   infinite capacity limits, Little's Law, Kleinrock work conservation, sample-path monotonicity,
//!   and zero-transit fidelity equivalence).
//!
//! - **Domain Validation ("Building the right model")**:
//!   Ensures that the conceptual model adequately mirrors clinical reality in an actual Emergency
//!   Department (e.g., triage distributions matching Australasian Triage Scale [ATS] emergency data,
//!   transfer delays matching hospital bed-manager records, and clinical deterioration hazards).
//!   Domain validation is conducted via empirical calibration and holdout comparison (e.g., in
//!   C6.2, C6.3, and E1.3), and must never be conflated with software verification.
//!
//! ## Public Reference Standards and Analytic Oracles Verified
//!
//! 1. **No-Contention / Infinite-Capacity Limiting Case**:
//!    When resource capacity (treatment bays and nursing staff) is unconstrained ($C \ge A(t)$),
//!    all queue wait times are identically zero ($W = 0$), each patient's service start coincides
//!    with arrival ($S_i = A_i$), and total elapsed time equals service duration (in Macro mode)
//!    or service duration plus spatial transit (in Micro mode).
//!
//! 2. **Kendall D/D/1 Deterministic Single-Server Queue (Report 12 Oracle)**:
//!    A single-server queue with deterministic arrivals and deterministic service durations.
//!    Verifies exact bit-for-bit equivalence with the Report 12 hand-computable schedule:
//!    - Patient 0: arrival=0, duration=2 -> start=0, wait=0, completion=2
//!    - Patient 1: arrival=1, duration=2 -> start=2, wait=1, completion=4
//!    - Patient 2: arrival=2, duration=3 -> start=4, wait=2, completion=7
//!    - Patient 3: arrival=7, duration=1 -> start=7, wait=0, completion=8
//!
//! 3. **Kleinrock Work Conservation and Priority Preemption**:
//!    Under preemptive-resume priority dispatch (ATS 1 emergency preempting ATS 5 non-urgent),
//!    the system satisfies the Kleinrock conservation law: total server busy duration equals the
//!    exact sum of required patient work ($\sum W_i$). Work already performed is strictly preserved;
//!    no service ticks are lost, duplicated, or destroyed.
//!    State conservation holds at every tick:
//!    $$\text{Admitted} = \text{Completed} + \text{InFlight} + \text{Preempted} + \text{InQueue}$$.
//!
//! 4. **Differential Micro Zero-Route Equivalence to Macro**:
//!    When spatial travel distances in the `SyntheticGraph` are set to zero (or travel speed produces
//!    0 transit ticks), `ExecutionFidelity::Micro` reduces identically to `ExecutionFidelity::Macro`.
//!    All start ticks, completion ticks, wait ticks, and elapsed ticks are 100% bit-for-bit identical.
//!
//! 5. **Sample-Path Capacity Monotonicity**:
//!    For an identical patient cohort, increasing capacity from $C=1$ to $C=2$ to $C=10$
//!    (unconstrained) yields monotonically non-increasing wait times:
//!    $$\bar{W}(C=1) \ge \bar{W}(C=2) \ge \bar{W}(C=10) = 0$$
//!    $$\max W(C=1) \ge \max W(C=2) \ge \max W(C=10) = 0$$
//!    and monotonically non-decreasing throughput within a bounded horizon.
//!
//! 6. **Terminal Patient Conservation Law**:
//!    Under arbitrary arrival patterns and finite horizon cutoff $H$, every admitted patient is
//!    accounted for deterministically:
//!    $$\text{Admitted} = \text{Completed} + \text{CensoredUnfinished}$$.
//!    No patient is lost or duplicated.

use careops_ed::disposition::composite_fixture::{
    ComposedPatient, CompositeEdFixture, PatientLifecycle,
};
use careops_ed::disposition::operations::{
    calculate_execution_duration, ExecutionFidelity, MovementEdge, SyntheticGraph,
};
use careops_ed::{Eligibility, PatientStatus};

/// Helper to construct standard nurse eligibility for tests.
fn standard_nurse_eligibility() -> Vec<Eligibility> {
    vec![Eligibility {
        staff_role_id: "staff.ed_nurse".to_string(),
        task_class_id: "task.treatment".to_string(),
        zone_id: "zone.acute".to_string(),
        location_id: Some("loc.acute.1".to_string()),
    }]
}

/// Helper to create a nominal composed patient.
fn make_test_patient(
    id: u64,
    name: &str,
    acuity: u8,
    arrival_tick: u64,
    work_ticks: u64,
) -> ComposedPatient {
    ComposedPatient {
        id,
        name: name.to_string(),
        acuity,
        arrival_tick,
        work_required_ticks: work_ticks,
        work_elapsed_ticks: 0,
        start_tick: None,
        completion_tick: None,
        assigned_bay: None,
        lifecycle: PatientLifecycle::Arrived,
        task_class_id: "task.treatment".to_string(),
        zone_id: "zone.acute".to_string(),
        location_id: "loc.acute.1".to_string(),
    }
}

// ---------------------------------------------------------------------------
// 1. Limiting Case: No Contention / Infinite Capacity
// ---------------------------------------------------------------------------

#[test]
fn test_limiting_case_no_contention_infinite_capacity() {
    let eligibilities = standard_nurse_eligibility();

    // Capacity (10 bays, 10 staff) >= arrivals at every tick (max 1 arrival per tick).
    let num_bays = 10;
    let num_staff = 10;

    let patient_cohort = [
        (101u64, "Patient_101", 3u8, 0u64, 5u64),
        (102u64, "Patient_102", 2u8, 1u64, 3u64),
        (103u64, "Patient_103", 4u8, 2u64, 4u64),
        (104u64, "Patient_104", 1u8, 3u64, 2u64),
    ];

    // --- (A) Macro Mode Evaluation ---
    {
        let mut fixture = CompositeEdFixture::new(
            num_bays,
            num_staff,
            "staff.ed_nurse",
            eligibilities.clone(),
            ExecutionFidelity::Macro,
        );

        let horizon = 12u64;
        for current_tick in 0..horizon {
            for &(id, name, acuity, arr, work) in &patient_cohort {
                if arr == current_tick {
                    fixture.admit_patient(make_test_patient(id, name, acuity, arr, work));
                }
            }
            fixture.dispatch_cycle();
            fixture.advance_tick();
        }

        assert_eq!(fixture.completed.len(), patient_cohort.len());
        assert_eq!(fixture.queue.len(), 0);
        assert_eq!(fixture.in_flight.len(), 0);

        let summary = fixture.to_run_summary("infinite_cap_macro", 42, horizon);
        assert_eq!(summary.arrivals, patient_cohort.len());
        assert_eq!(summary.completed, patient_cohort.len());
        assert_eq!(summary.unfinished, 0);

        for &(id, _, _, arr, work) in &patient_cohort {
            let p = summary
                .patients
                .iter()
                .find(|p| p.patient_id == format!("Patient_{id}"))
                .expect("patient must exist in summary");

            // Verify: queue wait ticks are strictly zero for all patients (wait_ticks == 0)
            assert_eq!(
                p.wait_ticks,
                Some(0),
                "wait_ticks must be strictly 0 under infinite capacity"
            );
            // Verify: each patient's start_tick == arrival_tick
            assert_eq!(
                p.start_tick,
                Some(arr),
                "start_tick must coincide with arrival_tick"
            );
            // In Macro mode: elapsed_ticks == work_ticks
            assert_eq!(
                p.elapsed_ticks,
                Some(work),
                "elapsed_ticks must equal work_ticks in Macro mode"
            );
            assert_eq!(p.work_ticks, work);
            assert_eq!(p.completion_tick, Some(arr + work));
            assert_eq!(p.status, PatientStatus::Completed);
        }
    }

    // --- (B) Micro Mode Evaluation with Non-Zero Graph Transit ---
    {
        // 30 meters at 1.5 m/s = exactly 20 ticks transit duration
        let transit_distance_m = 30.0;
        let transit_speed_m_s = 1.5;
        let expected_transit_ticks = 20u64;

        let edge = MovementEdge {
            edge_id: "edge.triage_to_acute".to_string(),
            from: "loc.triage".to_string(),
            to: "loc.acute.1".to_string(),
            distance_m: transit_distance_m,
            speed_m_per_s: transit_speed_m_s,
            accessible: true,
        };
        let graph = SyntheticGraph::new(vec![edge]);

        let mut fixture = CompositeEdFixture::new(
            num_bays,
            num_staff,
            "staff.ed_nurse",
            eligibilities,
            ExecutionFidelity::Micro,
        );
        fixture.set_graph(graph.clone());

        let horizon = 35u64;
        for current_tick in 0..horizon {
            for &(id, name, acuity, arr, base_work) in &patient_cohort {
                if arr == current_tick {
                    let effective_work = calculate_execution_duration(
                        ExecutionFidelity::Micro,
                        base_work as f64,
                        Some(&graph),
                        &["edge.triage_to_acute"],
                    )
                    .expect("route must be resolvable")
                        as u64;

                    assert_eq!(effective_work, base_work + expected_transit_ticks);

                    fixture.admit_patient(make_test_patient(id, name, acuity, arr, effective_work));
                }
            }
            fixture.dispatch_cycle();
            fixture.advance_tick();
        }

        assert_eq!(fixture.completed.len(), patient_cohort.len());
        assert_eq!(fixture.queue.len(), 0);
        assert_eq!(fixture.in_flight.len(), 0);

        let summary = fixture.to_run_summary("infinite_cap_micro", 42, horizon);
        assert_eq!(summary.arrivals, patient_cohort.len());
        assert_eq!(summary.completed, patient_cohort.len());
        assert_eq!(summary.unfinished, 0);

        for &(id, _, _, arr, base_work) in &patient_cohort {
            let p = summary
                .patients
                .iter()
                .find(|p| p.patient_id == format!("Patient_{id}"))
                .expect("patient must exist in summary");

            // Verify: queue wait ticks are strictly zero for all patients (wait_ticks == 0)
            assert_eq!(
                p.wait_ticks,
                Some(0),
                "wait_ticks must be strictly 0 under infinite capacity"
            );
            // Verify: each patient's start_tick == arrival_tick
            assert_eq!(
                p.start_tick,
                Some(arr),
                "start_tick must coincide with arrival_tick"
            );
            // In Micro mode with non-zero graph transit: wait_ticks == 0, and elapsed_ticks == work_ticks + transit_ticks
            let expected_elapsed = base_work + expected_transit_ticks;
            assert_eq!(
                p.elapsed_ticks,
                Some(expected_elapsed),
                "elapsed_ticks must equal base work_ticks + transit_ticks in Micro mode"
            );
            assert_eq!(p.completion_tick, Some(arr + expected_elapsed));
            assert_eq!(p.status, PatientStatus::Completed);
        }

        // Trace export confirmation
        let traces = fixture
            .export_calibration_traces(1, &["edge.triage_to_acute"])
            .expect("calibration traces export must succeed");
        for trace in &traces {
            assert_eq!(trace.transit_ticks, expected_transit_ticks);
        }
    }
}

// ---------------------------------------------------------------------------
// 2. Limiting Case: Deterministic Single-Server Queue (D/D/1 Report 12 Oracle)
// ---------------------------------------------------------------------------

#[test]
fn test_limiting_case_deterministic_single_server_dd1() {
    let eligibilities = standard_nurse_eligibility();

    // Exactly 1 bay and 1 staff: strict deterministic single server
    let mut fixture = CompositeEdFixture::new(
        1,
        1,
        "staff.ed_nurse",
        eligibilities,
        ExecutionFidelity::Macro,
    );

    // Analytic hand-computable schedule:
    // Report 12 hand-computable oracle:
    //   Patient 0: arrival=0, duration=2 -> start=0, wait=0, completion=2
    //   Patient 1: arrival=1, duration=2 -> start=2, wait=1, completion=4
    // Extended hand-computable schedule:
    //   Patient 2: arrival=2, duration=3 -> start=4, wait=2, completion=7
    //   Patient 3: arrival=7, duration=1 -> start=7, wait=0, completion=8
    let oracle_schedule = [
        (0u64, "Patient_0", 0u64, 2u64, 0u64, 0u64, 2u64),
        (1u64, "Patient_1", 1u64, 2u64, 2u64, 1u64, 4u64),
        (2u64, "Patient_2", 2u64, 3u64, 4u64, 2u64, 7u64),
        (3u64, "Patient_3", 7u64, 1u64, 7u64, 0u64, 8u64),
    ];

    let horizon = 8u64;
    for current_tick in 0..horizon {
        for &(id, name, arr, dur, _, _, _) in &oracle_schedule {
            if arr == current_tick {
                fixture.admit_patient(make_test_patient(id, name, 3, arr, dur));
            }
        }
        fixture.dispatch_cycle();
        fixture.advance_tick();
    }

    assert_eq!(fixture.completed.len(), 4);
    assert_eq!(fixture.in_flight.len(), 0);
    assert_eq!(fixture.queue.len(), 0);

    let summary = fixture.to_run_summary("single_server_dd1", 42, horizon);

    // Assert all start, wait, and completion ticks match the analytic oracle bit-for-bit
    for &(id, name, expected_arr, expected_dur, expected_start, expected_wait, expected_comp) in
        &oracle_schedule
    {
        let p = summary
            .patients
            .iter()
            .find(|p| p.patient_id == name)
            .unwrap_or_else(|| panic!("patient {name} must exist in summary"));

        assert_eq!(
            p.arrival_tick, expected_arr,
            "patient {id} arrival tick mismatch"
        );
        assert_eq!(
            p.start_tick,
            Some(expected_start),
            "patient {id} start tick mismatch against analytic oracle"
        );
        assert_eq!(
            p.wait_ticks,
            Some(expected_wait),
            "patient {id} wait ticks mismatch against analytic oracle"
        );
        assert_eq!(
            p.completion_tick,
            Some(expected_comp),
            "patient {id} completion tick mismatch against analytic oracle"
        );
        assert_eq!(
            p.work_ticks, expected_dur,
            "patient {id} work ticks mismatch"
        );
        assert_eq!(
            p.elapsed_ticks,
            Some(expected_comp - expected_arr),
            "patient {id} elapsed ticks mismatch"
        );
        assert_eq!(p.status, PatientStatus::Completed);
    }

    assert!(
        fixture.verify_invariants(4).is_ok(),
        "fixture invariants must hold for single server D/D/1"
    );
}

// ---------------------------------------------------------------------------
// 3. Known Queue Fixture: Priority Preemption and Work Conservation
// ---------------------------------------------------------------------------

#[test]
fn test_known_queue_fixture_priority_preemption_work_conservation() {
    let eligibilities = standard_nurse_eligibility();

    // 1 bay, 1 staff
    let mut fixture = CompositeEdFixture::new(
        1,
        1,
        "staff.ed_nurse",
        eligibilities,
        ExecutionFidelity::Macro,
    );

    // Patient A (ATS 5, non-urgent) arrives at tick 0, required work = 6 ticks
    let patient_a = make_test_patient(10, "Patient_ATS5", 5, 0, 6);
    // Patient B (ATS 1, resuscitation/immediate) arrives at tick 2, required work = 3 ticks
    let patient_b = make_test_patient(20, "Patient_ATS1", 1, 2, 3);

    let sum_of_required_work = patient_a.work_required_ticks + patient_b.work_required_ticks; // 6 + 3 = 9 ticks
    let mut server_busy_ticks = 0u64;
    let mut admitted_count = 0usize;

    // Helper closure to verify state conservation invariant:
    // admitted == completed + in_flight + preempted + in_queue
    let verify_state_conservation = |f: &CompositeEdFixture, admitted: usize| {
        let completed = f.completed.len();
        let in_flight = f.in_flight.len();
        let preempted = f
            .queue
            .iter()
            .filter(|p| p.lifecycle == PatientLifecycle::Preempted)
            .count();
        let in_queue = f
            .queue
            .iter()
            .filter(|p| p.lifecycle == PatientLifecycle::WaitingInQueue)
            .count();

        assert_eq!(
            admitted,
            completed + in_flight + preempted + in_queue,
            "State conservation invariant violated: admitted ({admitted}) != completed ({completed}) + in_flight ({in_flight}) + preempted ({preempted}) + in_queue ({in_queue})"
        );
        assert!(
            f.verify_invariants(admitted).is_ok(),
            "Composite fixture operational invariants must hold"
        );
    };

    // --- Tick 0: Patient A arrives ---
    fixture.admit_patient(patient_a);
    admitted_count += 1;
    verify_state_conservation(&fixture, admitted_count);

    fixture.dispatch_cycle();
    assert_eq!(fixture.in_flight.len(), 1);
    assert_eq!(fixture.in_flight[0].id, 10);
    assert_eq!(fixture.in_flight[0].start_tick, Some(0));
    verify_state_conservation(&fixture, admitted_count);

    // Tick 0 -> 1
    if !fixture.in_flight.is_empty() {
        server_busy_ticks += 1;
    }
    fixture.advance_tick();
    assert_eq!(fixture.current_tick, 1);
    assert_eq!(fixture.in_flight[0].work_elapsed_ticks, 1);
    verify_state_conservation(&fixture, admitted_count);

    // Tick 1 -> 2
    if !fixture.in_flight.is_empty() {
        server_busy_ticks += 1;
    }
    fixture.advance_tick();
    assert_eq!(fixture.current_tick, 2);
    assert_eq!(fixture.in_flight[0].work_elapsed_ticks, 2);
    verify_state_conservation(&fixture, admitted_count);

    // --- Tick 2: High-urgency Patient B arrives and preempts Patient A ---
    // Preempt lowest in-flight patient (ATS 1 preempts ATS 5)
    let preempted = fixture.try_preempt_lowest(1);
    assert!(
        preempted,
        "ATS 1 patient must successfully preempt ATS 5 patient"
    );

    // Work preservation guarantee: Patient A retains exactly 2 accrued ticks
    assert_eq!(fixture.queue.len(), 1);
    assert_eq!(fixture.queue[0].id, 10);
    assert_eq!(fixture.queue[0].lifecycle, PatientLifecycle::Preempted);
    assert_eq!(
        fixture.queue[0].work_elapsed_ticks, 2,
        "Work conservation violated: accrued work must not be lost upon preemption"
    );

    // Admit Patient B
    fixture.admit_patient(patient_b);
    admitted_count += 1;
    verify_state_conservation(&fixture, admitted_count);

    // Dispatch ATS 1 patient
    fixture.dispatch_cycle();
    assert_eq!(fixture.in_flight.len(), 1);
    assert_eq!(fixture.in_flight[0].id, 20);
    assert_eq!(fixture.in_flight[0].start_tick, Some(2));
    verify_state_conservation(&fixture, admitted_count);

    // Advance through Patient B's service (3 ticks: ticks 2->3, 3->4, 4->5)
    for expected_elapsed in [1, 2, 3] {
        if !fixture.in_flight.is_empty() {
            server_busy_ticks += 1;
        }
        fixture.advance_tick();
        if expected_elapsed < 3 {
            assert_eq!(fixture.in_flight[0].id, 20);
            assert_eq!(fixture.in_flight[0].work_elapsed_ticks, expected_elapsed);
        }
        verify_state_conservation(&fixture, admitted_count);
    }

    // At tick 5: Patient B completed, Patient A resumed!
    assert_eq!(fixture.completed.len(), 1);
    assert_eq!(fixture.completed[0].id, 20);
    assert_eq!(fixture.completed[0].completion_tick, Some(5));

    // Preempted patient accurately resumes with accrued work intact
    assert_eq!(fixture.in_flight.len(), 1);
    assert_eq!(fixture.in_flight[0].id, 10);
    assert_eq!(
        fixture.in_flight[0].work_elapsed_ticks, 2,
        "Resumed patient must resume from previous accrued work"
    );

    // Patient A finishes remaining 4 ticks (ticks 5->6, 6->7, 7->8, 8->9)
    for _ in 0..4 {
        if !fixture.in_flight.is_empty() {
            server_busy_ticks += 1;
        }
        fixture.advance_tick();
        verify_state_conservation(&fixture, admitted_count);
    }

    // Both patients are now completed
    assert_eq!(fixture.completed.len(), 2);
    assert_eq!(fixture.in_flight.len(), 0);
    assert_eq!(fixture.queue.len(), 0);
    assert_eq!(fixture.current_tick, 9);

    let p_a_done = fixture
        .completed
        .iter()
        .find(|p| p.id == 10)
        .expect("Patient A must be completed");
    assert_eq!(
        p_a_done.work_elapsed_ticks, 6,
        "Patient A total work elapsed must equal required work (6)"
    );
    assert_eq!(p_a_done.completion_tick, Some(9));

    // Analytic guarantee: total system server busy ticks == sum of patient work required
    assert_eq!(
        server_busy_ticks, sum_of_required_work,
        "Kleinrock work conservation violated: total server busy ticks ({server_busy_ticks}) != required work sum ({sum_of_required_work})"
    );
}

// ---------------------------------------------------------------------------
// 4. Invariant: Differential Micro Zero-Route Equivalence to Macro
// ---------------------------------------------------------------------------

#[test]
fn test_invariant_micro_zero_route_equivalence_to_macro() {
    let eligibilities = standard_nurse_eligibility();

    // Identical patient cohort with varying arrival ticks, acuities, and durations
    let cohort = [
        (1u64, "Patient_1", 3u8, 0u64, 4u64),
        (2u64, "Patient_2", 2u8, 1u64, 5u64),
        (3u64, "Patient_3", 4u8, 3u64, 3u64),
        (4u64, "Patient_4", 1u8, 4u64, 6u64),
        (5u64, "Patient_5", 5u8, 7u64, 2u64),
    ];

    let horizon = 30u64;

    // --- (A) Run with ExecutionFidelity::Macro ---
    let macro_summary = {
        let mut fixture = CompositeEdFixture::new(
            2,
            2,
            "staff.ed_nurse",
            eligibilities.clone(),
            ExecutionFidelity::Macro,
        );

        for current_tick in 0..horizon {
            for &(id, name, acuity, arr, work) in &cohort {
                if arr == current_tick {
                    let dur = calculate_execution_duration(
                        ExecutionFidelity::Macro,
                        work as f64,
                        None,
                        &[],
                    )
                    .unwrap() as u64;

                    fixture.admit_patient(make_test_patient(id, name, acuity, arr, dur));
                }
            }
            fixture.dispatch_cycle();
            fixture.advance_tick();
        }

        fixture.to_run_summary("macro_zero_eq", 42, horizon)
    };

    // --- (B) Run with ExecutionFidelity::Micro with Zero Route Distance ---
    let micro_summary = {
        // All route edge distances in SyntheticGraph are 0.0 -> transit duration is strictly 0
        let zero_edge = MovementEdge {
            edge_id: "edge.zero_distance".to_string(),
            from: "loc.triage".to_string(),
            to: "loc.acute.1".to_string(),
            distance_m: 0.0,
            speed_m_per_s: 1.5,
            accessible: true,
        };
        let graph = SyntheticGraph::new(vec![zero_edge]);

        let mut fixture = CompositeEdFixture::new(
            2,
            2,
            "staff.ed_nurse",
            eligibilities,
            ExecutionFidelity::Micro,
        );
        fixture.set_graph(graph.clone());

        for current_tick in 0..horizon {
            for &(id, name, acuity, arr, work) in &cohort {
                if arr == current_tick {
                    let dur = calculate_execution_duration(
                        ExecutionFidelity::Micro,
                        work as f64,
                        Some(&graph),
                        &["edge.zero_distance"],
                    )
                    .unwrap() as u64;

                    assert_eq!(dur, work, "zero distance route must produce zero transit");
                    fixture.admit_patient(make_test_patient(id, name, acuity, arr, dur));
                }
            }
            fixture.dispatch_cycle();
            fixture.advance_tick();
        }

        fixture.to_run_summary("micro_zero_eq", 42, horizon)
    };

    // Assert that every patient's wait ticks, start ticks, elapsed ticks, and completion ticks
    // are 100% bit-for-bit identical between Macro and Micro.
    assert_eq!(
        macro_summary.arrivals, micro_summary.arrivals,
        "arrivals must be identical"
    );
    assert_eq!(
        macro_summary.started, micro_summary.started,
        "started count must be identical"
    );
    assert_eq!(
        macro_summary.completed, micro_summary.completed,
        "completed count must be identical"
    );
    assert_eq!(
        macro_summary.unfinished, micro_summary.unfinished,
        "unfinished count must be identical"
    );
    assert_eq!(
        macro_summary.patients.len(),
        micro_summary.patients.len(),
        "patient count must be identical"
    );

    for (macro_p, micro_p) in macro_summary
        .patients
        .iter()
        .zip(micro_summary.patients.iter())
    {
        assert_eq!(macro_p.patient_id, micro_p.patient_id);
        assert_eq!(macro_p.arrival_tick, micro_p.arrival_tick);
        assert_eq!(
            macro_p.start_tick, micro_p.start_tick,
            "start_tick mismatch for patient {}",
            macro_p.patient_id
        );
        assert_eq!(
            macro_p.completion_tick, micro_p.completion_tick,
            "completion_tick mismatch for patient {}",
            macro_p.patient_id
        );
        assert_eq!(
            macro_p.work_ticks, micro_p.work_ticks,
            "work_ticks mismatch for patient {}",
            macro_p.patient_id
        );
        assert_eq!(
            macro_p.wait_ticks, micro_p.wait_ticks,
            "wait_ticks mismatch for patient {}",
            macro_p.patient_id
        );
        assert_eq!(
            macro_p.elapsed_ticks, micro_p.elapsed_ticks,
            "elapsed_ticks mismatch for patient {}",
            macro_p.patient_id
        );
        assert_eq!(
            macro_p.status, micro_p.status,
            "status mismatch for patient {}",
            macro_p.patient_id
        );
    }
}

// ---------------------------------------------------------------------------
// 5. Differential Test: Capacity Monotonicity Law
// ---------------------------------------------------------------------------

#[test]
fn test_differential_capacity_monotonicity() {
    let eligibilities = standard_nurse_eligibility();

    // 8 patients arrive simultaneously at tick 0, each requiring 4 ticks of work
    let cohort_size = 8usize;
    let work_ticks = 4u64;

    let run_with_capacity = |cap: usize, horizon: u64| {
        let mut fixture = CompositeEdFixture::new(
            cap,
            cap as u64,
            "staff.ed_nurse",
            eligibilities.clone(),
            ExecutionFidelity::Macro,
        );

        for id in 1..=cohort_size {
            let p = make_test_patient(
                id as u64,
                &format!("P_{id}"),
                3,
                0, // All arrive at tick 0
                work_ticks,
            );
            fixture.admit_patient(p);
        }

        fixture.dispatch_cycle();

        for _ in 0..horizon {
            fixture.advance_tick();
        }

        fixture.to_run_summary(&format!("cap_{cap}"), 42, horizon)
    };

    // Evaluate same patient cohort under capacity C=1, C=2, and C=10 (unconstrained)
    // Run to completion horizon (H=35) to measure full wait time distribution
    let full_horizon = 35u64;
    let summary_c1 = run_with_capacity(1, full_horizon);
    let summary_c2 = run_with_capacity(2, full_horizon);
    let summary_c10 = run_with_capacity(10, full_horizon);

    assert_eq!(summary_c1.completed, cohort_size);
    assert_eq!(summary_c2.completed, cohort_size);
    assert_eq!(summary_c10.completed, cohort_size);

    let extract_wait_metrics = |summary: &careops_ed::RunSummary| -> (f64, u64) {
        let waits: Vec<u64> = summary
            .patients
            .iter()
            .map(|p| {
                p.wait_ticks
                    .expect("all completed patients have wait ticks")
            })
            .collect();
        let avg_wait = waits.iter().sum::<u64>() as f64 / waits.len() as f64;
        let max_wait = *waits.iter().max().expect("waits non-empty");
        (avg_wait, max_wait)
    };

    let (avg_wait_c1, max_wait_c1) = extract_wait_metrics(&summary_c1);
    let (avg_wait_c2, max_wait_c2) = extract_wait_metrics(&summary_c2);
    let (avg_wait_c10, max_wait_c10) = extract_wait_metrics(&summary_c10);

    // Under C=1: waits are 0, 4, 8, 12, 16, 20, 24, 28 -> mean = 14.0, max = 28
    assert_eq!(avg_wait_c1, 14.0);
    assert_eq!(max_wait_c1, 28);

    // Under C=2: waits are 0, 0, 4, 4, 8, 8, 12, 12 -> mean = 6.0, max = 12
    assert_eq!(avg_wait_c2, 6.0);
    assert_eq!(max_wait_c2, 12);

    // Under C=10 (unconstrained): all start immediately -> mean = 0.0, max = 0
    assert_eq!(avg_wait_c10, 0.0);
    assert_eq!(max_wait_c10, 0);

    // Assert: average and max wait times monotonically non-increasing:
    // Wait(C=1) >= Wait(C=2) >= Wait(C=10) == 0
    assert!(
        avg_wait_c1 >= avg_wait_c2,
        "Average wait must be monotonically non-increasing from C=1 ({avg_wait_c1}) to C=2 ({avg_wait_c2})"
    );
    assert!(
        avg_wait_c2 >= avg_wait_c10,
        "Average wait must be monotonically non-increasing from C=2 ({avg_wait_c2}) to C=10 ({avg_wait_c10})"
    );
    assert_eq!(
        avg_wait_c10, 0.0,
        "Average wait must be exactly 0 under unconstrained capacity"
    );

    assert!(
        max_wait_c1 >= max_wait_c2,
        "Max wait must be monotonically non-increasing from C=1 ({max_wait_c1}) to C=2 ({max_wait_c2})"
    );
    assert!(
        max_wait_c2 >= max_wait_c10,
        "Max wait must be monotonically non-increasing from C=2 ({max_wait_c2}) to C=10 ({max_wait_c10})"
    );
    assert_eq!(
        max_wait_c10, 0,
        "Max wait must be exactly 0 under unconstrained capacity"
    );

    // Assert: throughput within horizon monotonically non-decreasing
    // Check bounded horizon H = 10 ticks:
    let bounded_h = 10u64;
    let bounded_c1 = run_with_capacity(1, bounded_h);
    let bounded_c2 = run_with_capacity(2, bounded_h);
    let bounded_c10 = run_with_capacity(10, bounded_h);

    // C=1 completes 2 patients by tick 10 (at tick 4 and 8)
    // C=2 completes 4 patients by tick 10 (pairs at tick 4 and 8)
    // C=10 completes all 8 patients by tick 10 (all at tick 4)
    assert!(
        bounded_c1.completed <= bounded_c2.completed,
        "Throughput must be monotonically non-decreasing from C=1 ({}) to C=2 ({})",
        bounded_c1.completed,
        bounded_c2.completed
    );
    assert!(
        bounded_c2.completed <= bounded_c10.completed,
        "Throughput must be monotonically non-decreasing from C=2 ({}) to C=10 ({})",
        bounded_c2.completed,
        bounded_c10.completed
    );
    assert_eq!(bounded_c1.completed, 2);
    assert_eq!(bounded_c2.completed, 4);
    assert_eq!(bounded_c10.completed, 8);
}

// ---------------------------------------------------------------------------
// 6. Conservation Law: Terminal Patient State Conservation
// ---------------------------------------------------------------------------

#[test]
fn test_terminal_patient_conservation_law() {
    let eligibilities = standard_nurse_eligibility();

    // 2 bays, 2 nurses
    let mut fixture = CompositeEdFixture::new(
        2,
        2,
        "staff.ed_nurse",
        eligibilities,
        ExecutionFidelity::Macro,
    );

    // Arbitrary arrival schedule with diverse arrival ticks, durations, and urgencies
    let arrival_schedule = [
        (1u64, "P01", 3u8, 0u64, 4u64),
        (2u64, "P02", 2u8, 1u64, 6u64),
        (3u64, "P03", 4u8, 2u64, 5u64),
        (4u64, "P04", 1u8, 3u64, 3u64),
        (5u64, "P05", 5u8, 5u64, 7u64),
        (6u64, "P06", 2u8, 6u64, 4u64),
        (7u64, "P07", 3u8, 8u64, 5u64),
        (8u64, "P08", 4u8, 10u64, 6u64),
        (9u64, "P09", 1u8, 12u64, 3u64),
        (10u64, "P10", 5u8, 14u64, 8u64),
    ];

    // Horizon cutoff H = 15 ticks: ensures some patients complete, some in-flight, some queued
    let horizon = 15u64;
    let mut total_admitted = 0usize;

    for current_tick in 0..horizon {
        for &(id, name, acuity, arr, work) in &arrival_schedule {
            if arr == current_tick {
                fixture.admit_patient(make_test_patient(id, name, acuity, arr, work));
                total_admitted += 1;
            }
        }
        fixture.dispatch_cycle();
        fixture.advance_tick();
    }

    assert_eq!(
        total_admitted, 10,
        "all 10 patients should have arrived by tick 15"
    );

    let summary = fixture.to_run_summary("terminal_conservation", 12345, horizon);

    // Conservation: admitted == completed + censored_unfinished
    assert_eq!(
        summary.arrivals, total_admitted,
        "summary arrivals must match admitted count"
    );
    assert_eq!(
        summary.arrivals,
        summary.completed + summary.unfinished,
        "Terminal conservation violated: admitted ({}) != completed ({}) + unfinished ({})",
        summary.arrivals,
        summary.completed,
        summary.unfinished
    );

    // Non-trivial censoring verification: both completed and unfinished must exist
    assert!(
        summary.completed > 0,
        "at least one patient must have completed"
    );
    assert!(
        summary.unfinished > 0,
        "at least one patient must be unfinished (censored) at horizon cutoff"
    );

    // Every admitted patient is deterministically accounted for with valid status (Completed or Unfinished)
    let completed_partition = summary
        .patients
        .iter()
        .filter(|p| p.status == PatientStatus::Completed)
        .count();
    let unfinished_partition = summary
        .patients
        .iter()
        .filter(|p| p.status == PatientStatus::Unfinished)
        .count();

    assert_eq!(completed_partition, summary.completed);
    assert_eq!(unfinished_partition, summary.unfinished);
    assert_eq!(
        completed_partition + unfinished_partition,
        total_admitted,
        "every patient must be deterministically partitioned into Completed or Unfinished"
    );

    // Inspect individual patient status contracts
    for p in &summary.patients {
        match p.status {
            PatientStatus::Completed => {
                assert!(p.start_tick.is_some(), "completed patient must have start");
                assert!(
                    p.completion_tick.is_some(),
                    "completed patient must have completion"
                );
                assert!(
                    p.completion_tick.unwrap() <= horizon,
                    "completed patient completion_tick must be within horizon"
                );
                assert!(
                    p.start_tick.unwrap() >= p.arrival_tick,
                    "start_tick cannot precede arrival"
                );
                assert!(
                    p.completion_tick.unwrap() >= p.start_tick.unwrap() + p.work_ticks,
                    "completion_tick must be >= start_tick + work_ticks"
                );
                assert!(p.wait_ticks.is_some());
                assert!(p.elapsed_ticks.is_some());
            }
            PatientStatus::Unfinished => {
                assert!(
                    p.completion_tick.is_none(),
                    "unfinished patient must have no completion_tick"
                );
                assert!(
                    p.elapsed_ticks.is_none(),
                    "unfinished patient must have no elapsed_ticks"
                );
            }
        }
    }

    assert!(
        fixture.verify_invariants(total_admitted).is_ok(),
        "fixture invariants must strictly hold at horizon cutoff"
    );
}
