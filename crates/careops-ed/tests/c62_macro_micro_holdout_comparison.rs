#![forbid(unsafe_code)]

//! C6.2 Macro/Micro Holdout Comparison and Uncertainty Reporting.
//!
//! Evaluates:
//! 1. `test_macro_micro_holdout_comparison_through_api_runner`:
//!    - Multi-bay, multi-zone held-out ED operational scenario (`zone.acute`, `zone.subacute`).
//!    - Staff assigned to zones via `Eligibility`.
//!    - Diverse patient cases with ATS 1-5 urgencies, diagnostic holds, urgent preemption, and boarding.
//!    - Synthetic physical graph connecting triage, bays, diagnostics, discharge.
//!    - Free-running Macro simulation (zero transit) and Micro simulation (graph transit added).
//!    - Generates `RunDocument` for both, compares via `careops_ed::api_runner::compare_run_documents`.
//!    - Verifies throughput, avg wait ticks, avg LOS/elapsed ticks, peak bay occupancy.
//!    - Verifies transit emission (0 for Macro, route duration for Micro).
//!    - Verifies interruption counts and conserved work.
//!    - Asserts empirical superiority is not presumed (neutral metric deltas without crowning either).
//!
//! 2. `test_orthogonal_exogenous_vs_generative_arrivals_macro_micro`:
//!    - 2x2 evaluation matrix:
//!      (1) Exogenous Arrivals x Macro
//!      (2) Exogenous Arrivals x Micro
//!      (3) Generative Arrivals x Macro
//!      (4) Generative Arrivals x Micro
//!    - Verifies exogenous arrivals evaluate operational mechanics with fixed demand,
//!      while generative arrivals validate the arrival generation process orthogonally.
//!
//! 3. `test_stochastic_and_structural_uncertainty_reporting`:
//!    - Multiple replications with fresh seeds.
//!    - Stochastic uncertainty (variance, 95% CI) across replications.
//!    - Comparison with structural uncertainty (Macro vs Micro delta).
//!    - Exclusions reporting (censored cases at horizon cutoff) and execution runtime.
//!    - Distance metrics (W1, KS, residuals) and marginal/joint conservation check.

use std::time::Instant;

use careops_ed::arrivals::{generate_arrivals, ArrivalBin, ArrivalConfig, ArrivalModeCounts};
use careops_ed::disposition::api_runner::compare_run_documents;
use careops_ed::disposition::composite_fixture::{
    st003_verify_retained_occupancy, ComposedPatient, CompositeEdFixture, PatientLifecycle,
};
use careops_ed::disposition::operations::{
    calculate_execution_duration, BayPhase, ExecutionFidelity, MovementEdge, OperationalError,
    SyntheticGraph,
};
use careops_ed::Eligibility;

/// Computes 1D Wasserstein-1 (Earth Mover's) distance between two empirical distributions.
fn compute_w1_distance(sample_a: &[f64], sample_b: &[f64]) -> f64 {
    if sample_a.is_empty() || sample_b.is_empty() {
        return 0.0;
    }
    let mut all_points: Vec<f64> = sample_a.iter().chain(sample_b.iter()).copied().collect();
    all_points.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    all_points.dedup();

    let mut w1 = 0.0;
    for window in all_points.windows(2) {
        let t0 = window[0];
        let t1 = window[1];
        let width = t1 - t0;
        let cdf_a = sample_a.iter().filter(|&&x| x <= t0).count() as f64 / sample_a.len() as f64;
        let cdf_b = sample_b.iter().filter(|&&x| x <= t0).count() as f64 / sample_b.len() as f64;
        w1 += (cdf_a - cdf_b).abs() * width;
    }
    w1
}

/// Computes two-sample Kolmogorov-Smirnov statistic D_KS between two empirical samples.
fn compute_ks_statistic(sample_a: &[f64], sample_b: &[f64]) -> f64 {
    if sample_a.is_empty() || sample_b.is_empty() {
        return 0.0;
    }
    let mut all_points: Vec<f64> = sample_a.iter().chain(sample_b.iter()).copied().collect();
    all_points.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    all_points.dedup();

    let mut max_d: f64 = 0.0;
    for &t in &all_points {
        let cdf_a = sample_a.iter().filter(|&&x| x <= t).count() as f64 / sample_a.len() as f64;
        let cdf_b = sample_b.iter().filter(|&&x| x <= t).count() as f64 / sample_b.len() as f64;
        let diff = (cdf_a - cdf_b).abs();
        if diff > max_d {
            max_d = diff;
        }
    }
    max_d
}

/// Paired residual metrics between two series.
#[derive(Debug, Clone, PartialEq)]
struct ResidualReport {
    mean_residual: f64,
    mae: f64,
    rmse: f64,
    min_residual: f64,
    max_residual: f64,
}

fn compute_paired_residuals(reference: &[f64], candidate: &[f64]) -> Option<ResidualReport> {
    if reference.is_empty() || reference.len() != candidate.len() {
        return None;
    }
    let n = reference.len() as f64;
    let residuals: Vec<f64> = candidate
        .iter()
        .zip(reference.iter())
        .map(|(c, r)| c - r)
        .collect();

    let sum: f64 = residuals.iter().sum();
    let mean_residual = sum / n;
    let abs_sum: f64 = residuals.iter().map(|r| r.abs()).sum();
    let mae = abs_sum / n;
    let sq_sum: f64 = residuals.iter().map(|r| r.powi(2)).sum();
    let rmse = (sq_sum / n).sqrt();
    let min_residual = residuals.iter().copied().fold(f64::INFINITY, f64::min);
    let max_residual = residuals.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    Some(ResidualReport {
        mean_residual,
        mae,
        rmse,
        min_residual,
        max_residual,
    })
}

/// Checks that joint length-of-stay satisfies LOS >= wait + active_work for each completed patient.
fn check_joint_queueing_conservation(
    wait_ticks: u64,
    work_ticks: u64,
    transit_ticks: u64,
    elapsed_ticks: u64,
) -> bool {
    elapsed_ticks >= wait_ticks + work_ticks + transit_ticks
}

/// Builds the synthetic spatial transit graph connecting triage, bays, diagnostics, discharge.
fn build_synthetic_graph() -> SyntheticGraph {
    let edges = vec![
        MovementEdge {
            edge_id: "edge.triage_to_acute".to_string(),
            from: "loc.triage".to_string(),
            to: "loc.acute.bay1".to_string(),
            distance_m: 30.0,
            speed_m_per_s: 1.5, // 20 ticks
            accessible: true,
        },
        MovementEdge {
            edge_id: "edge.triage_to_subacute".to_string(),
            from: "loc.triage".to_string(),
            to: "loc.subacute.bay1".to_string(),
            distance_m: 45.0,
            speed_m_per_s: 1.5, // 30 ticks
            accessible: true,
        },
        MovementEdge {
            edge_id: "edge.acute_to_diag".to_string(),
            from: "loc.acute.bay1".to_string(),
            to: "loc.diagnostics".to_string(),
            distance_m: 15.0,
            speed_m_per_s: 1.5, // 10 ticks
            accessible: true,
        },
        MovementEdge {
            edge_id: "edge.diag_to_acute".to_string(),
            from: "loc.diagnostics".to_string(),
            to: "loc.acute.bay1".to_string(),
            distance_m: 15.0,
            speed_m_per_s: 1.5, // 10 ticks
            accessible: true,
        },
        MovementEdge {
            edge_id: "edge.acute_to_discharge".to_string(),
            from: "loc.acute.bay1".to_string(),
            to: "loc.discharge".to_string(),
            distance_m: 15.0,
            speed_m_per_s: 1.5, // 10 ticks
            accessible: true,
        },
        MovementEdge {
            edge_id: "edge.subacute_to_discharge".to_string(),
            from: "loc.subacute.bay1".to_string(),
            to: "loc.discharge".to_string(),
            distance_m: 30.0,
            speed_m_per_s: 1.5, // 20 ticks
            accessible: true,
        },
    ];
    SyntheticGraph::new(edges)
}

/// Builds staff eligibilities spanning acute and subacute zones.
fn build_multi_zone_eligibilities() -> Vec<Eligibility> {
    vec![
        Eligibility {
            staff_role_id: "staff.ed_nurse".to_string(),
            task_class_id: "task.treatment".to_string(),
            zone_id: "zone.acute".to_string(),
            location_id: Some("loc.acute.bay1".to_string()),
        },
        Eligibility {
            staff_role_id: "staff.ed_nurse".to_string(),
            task_class_id: "task.treatment".to_string(),
            zone_id: "zone.subacute".to_string(),
            location_id: Some("loc.subacute.bay1".to_string()),
        },
    ]
}

/// Held-out patient case definitions across ATS 1-5 urgencies.
struct HeldoutCase {
    id: u64,
    name: &'static str,
    acuity: u8,
    arrival_tick: u64,
    work_ticks: u64,
    zone_id: &'static str,
    location_id: &'static str,
    route_edges: Vec<&'static str>,
}

fn create_heldout_cases() -> Vec<HeldoutCase> {
    vec![
        // Patient 1: ATS 5 non-urgent, zone.acute, will be preempted by ATS 1
        HeldoutCase {
            id: 501,
            name: "Case_501_ATS5",
            acuity: 5,
            arrival_tick: 0,
            work_ticks: 10,
            zone_id: "zone.acute",
            location_id: "loc.acute.bay1",
            route_edges: vec!["edge.triage_to_acute", "edge.acute_to_discharge"], // 20 + 10 = 30 ticks
        },
        // Patient 2: ATS 4 semi-urgent, zone.subacute, transitions to boarding
        HeldoutCase {
            id: 401,
            name: "Case_401_ATS4",
            acuity: 4,
            arrival_tick: 0,
            work_ticks: 6,
            zone_id: "zone.subacute",
            location_id: "loc.subacute.bay1",
            route_edges: vec!["edge.triage_to_subacute", "edge.subacute_to_discharge"], // 30 + 20 = 50 ticks
        },
        // Patient 3: ATS 3 urgent, zone.acute, undergoes diagnostic hold
        HeldoutCase {
            id: 301,
            name: "Case_301_ATS3",
            acuity: 3,
            arrival_tick: 1,
            work_ticks: 8,
            zone_id: "zone.acute",
            location_id: "loc.acute.bay1",
            route_edges: vec![
                "edge.triage_to_acute",
                "edge.acute_to_diag",
                "edge.diag_to_acute",
                "edge.acute_to_discharge",
            ], // 20 + 10 + 10 + 10 = 50 ticks
        },
        // Patient 4: ATS 2 emergency, zone.subacute
        HeldoutCase {
            id: 201,
            name: "Case_201_ATS2",
            acuity: 2,
            arrival_tick: 2,
            work_ticks: 6,
            zone_id: "zone.subacute",
            location_id: "loc.subacute.bay1",
            route_edges: vec!["edge.triage_to_subacute", "edge.subacute_to_discharge"], // 30 + 20 = 50 ticks
        },
        // Patient 5: ATS 1 immediate resuscitation, preempts ATS 5 at tick 4
        HeldoutCase {
            id: 101,
            name: "Case_101_ATS1",
            acuity: 1,
            arrival_tick: 4,
            work_ticks: 4,
            zone_id: "zone.acute",
            location_id: "loc.acute.bay1",
            route_edges: vec!["edge.triage_to_acute", "edge.acute_to_discharge"], // 20 + 10 = 30 ticks
        },
    ]
}

/// 1. `test_macro_micro_holdout_comparison_through_api_runner`
#[test]
fn test_macro_micro_holdout_comparison_through_api_runner() {
    let graph = build_synthetic_graph();
    let eligibilities = build_multi_zone_eligibilities();

    // Helper to run a free-running simulation with given fidelity
    let run_fixture = |fidelity: ExecutionFidelity| {
        let mut fixture =
            CompositeEdFixture::new(2, 2, "staff.ed_nurse", eligibilities.clone(), fidelity);
        fixture.set_graph(graph.clone());

        let cases = create_heldout_cases();
        let horizon_ticks = 150u64;
        let mut preemption_interruption_count = 0usize;
        let mut preemption_conserved_work = 0u64;

        for current_tick in 0..horizon_ticks {
            // Admit arriving patients at this tick
            let mut newly_admitted = false;
            for case in &cases {
                if case.arrival_tick == current_tick {
                    let effective_work = match fidelity {
                        ExecutionFidelity::Macro => calculate_execution_duration(
                            ExecutionFidelity::Macro,
                            case.work_ticks as f64,
                            None,
                            &[],
                        )
                        .unwrap() as u64,
                        ExecutionFidelity::Micro => calculate_execution_duration(
                            ExecutionFidelity::Micro,
                            case.work_ticks as f64,
                            Some(&graph),
                            &case.route_edges,
                        )
                        .unwrap() as u64,
                    };

                    let p = ComposedPatient {
                        id: case.id,
                        name: case.name.to_string(),
                        acuity: case.acuity,
                        arrival_tick: case.arrival_tick,
                        work_required_ticks: effective_work,
                        work_elapsed_ticks: 0,
                        start_tick: None,
                        completion_tick: None,
                        assigned_bay: None,
                        lifecycle: PatientLifecycle::Arrived,
                        task_class_id: "task.treatment".to_string(),
                        zone_id: case.zone_id.to_string(),
                        location_id: case.location_id.to_string(),
                    };
                    fixture.admit_patient(p);
                    newly_admitted = true;

                    // If ATS 1 arrives, trigger preemption check against lowest in-flight patient
                    if case.acuity == 1 {
                        // Find victim before preemption to record conserved work
                        let victim_work_before = fixture
                            .in_flight
                            .iter()
                            .filter(|p| p.acuity > 1)
                            .max_by_key(|p| p.acuity)
                            .map(|p| p.work_elapsed_ticks);

                        if fixture.try_preempt_lowest(1) {
                            preemption_interruption_count += 1;
                            if let Some(w) = victim_work_before {
                                preemption_conserved_work = w;
                            }
                        }
                    }
                }
            }

            if newly_admitted {
                fixture.dispatch_cycle();
            }

            // Diagnostic hold logic for Patient 301
            for p in &fixture.in_flight {
                if p.id == 301 {
                    if let Some(bay_idx) = p.assigned_bay {
                        // Hold diagnostics between tick 15 and 20
                        if (15..20).contains(&current_tick) {
                            if fixture.bays[bay_idx].phase()
                                == (BayPhase::Occupied {
                                    patient_id: 301,
                                    diagnostic_blocked: false,
                                    boarding: false,
                                })
                            {
                                assert!(fixture.bays[bay_idx].block_for_diagnostics().is_ok());
                                // Premature release must fail while diagnostics are blocked
                                assert_eq!(
                                    fixture.bays[bay_idx].vacate_for_cleaning(current_tick),
                                    Err(OperationalError::DiagnosticBlocked)
                                );
                            }
                        } else if current_tick == 20
                            && fixture.bays[bay_idx].phase()
                                == (BayPhase::Occupied {
                                    patient_id: 301,
                                    diagnostic_blocked: true,
                                    boarding: false,
                                })
                        {
                            assert!(fixture.bays[bay_idx].clear_diagnostics().is_ok());
                        }
                    }
                }
            }

            // Boarding logic for Patient 401
            for p in &fixture.in_flight {
                if p.id == 401 && p.work_elapsed_ticks >= p.work_required_ticks {
                    if let Some(bay_idx) = p.assigned_bay {
                        if fixture.bays[bay_idx].phase()
                            == (BayPhase::Occupied {
                                patient_id: 401,
                                diagnostic_blocked: false,
                                boarding: false,
                            })
                        {
                            assert!(fixture.bays[bay_idx].begin_boarding().is_ok());
                            // Boarding retains bay: vacate fails
                            assert_eq!(
                                fixture.bays[bay_idx].vacate_for_cleaning(current_tick),
                                Err(OperationalError::BoardingRetained)
                            );
                            // Verify retained occupancy oracle
                            let retained =
                                st003_verify_retained_occupancy(&fixture.bays[bay_idx], 401, false);
                            assert_eq!(retained, Ok(true));
                        }
                    }
                }
            }

            // Transfer boarding Patient 401 at tick 40
            if current_tick == 40 {
                let boarding_p401 = fixture
                    .in_flight
                    .iter()
                    .any(|p| p.id == 401 && p.lifecycle == PatientLifecycle::Boarding);
                if boarding_p401 {
                    let res = fixture.complete_boarding_transfer(401);
                    assert!(res.is_ok());
                }
            }

            fixture.advance_tick();
        }

        (
            fixture,
            preemption_interruption_count,
            preemption_conserved_work,
        )
    };

    // Run free-running Macro simulation
    let (macro_fixture, macro_interruptions, macro_conserved_work) =
        run_fixture(ExecutionFidelity::Macro);
    // Run free-running Micro simulation
    let (micro_fixture, micro_interruptions, micro_conserved_work) =
        run_fixture(ExecutionFidelity::Micro);

    // Verify interruption counts: preemption event conserved work in both runs
    assert_eq!(
        macro_interruptions, 1,
        "Macro run must experience exactly 1 preemption interruption"
    );
    assert_eq!(
        micro_interruptions, 1,
        "Micro run must experience exactly 1 preemption interruption"
    );
    assert_eq!(
        macro_conserved_work, 4,
        "Accrued work (4 ticks) strictly conserved in Macro"
    );
    assert_eq!(
        micro_conserved_work, 4,
        "Accrued work (4 ticks) strictly conserved in Micro"
    );

    // Verify transit emissions:
    // Macro emits 0 transit ticks
    let macro_traces = macro_fixture
        .export_calibration_traces(1, &["edge.triage_to_acute"])
        .expect("macro trace export succeeds");
    for trace in &macro_traces {
        assert_eq!(
            trace.transit_ticks, 0,
            "Macro fidelity must emit zero transit ticks"
        );
    }

    // Micro emits route transit ticks
    let micro_traces = micro_fixture
        .export_calibration_traces(1, &["edge.triage_to_acute"])
        .expect("micro trace export succeeds");
    for trace in &micro_traces {
        assert_eq!(
            trace.transit_ticks, 20,
            "Micro fidelity must emit route transit ticks (20s)"
        );
    }

    // Generate RunDocument for Macro and Micro
    let macro_doc = macro_fixture.to_run_document("ed-holdout-macro", 42, 150);
    let micro_doc = micro_fixture.to_run_document("ed-holdout-micro", 42, 150);

    // Compare them through public API runner
    let comparison = compare_run_documents(&macro_doc, &micro_doc)
        .expect("comparison report must succeed between valid run documents");

    assert_eq!(comparison.base_scenario_id, "ed-holdout-macro");
    assert_eq!(comparison.candidate_scenario_id, "ed-holdout-micro");
    assert_eq!(comparison.time_unit, "tick");

    // Verify compared metrics: throughput, avg wait ticks, avg LOS/elapsed ticks, peak bay occupancy
    let throughput_metric = comparison
        .metrics
        .iter()
        .find(|m| m.metric_name == "completed")
        .expect("throughput metric must be present");
    assert_eq!(throughput_metric.base_value, 5.0);
    assert_eq!(throughput_metric.candidate_value, 5.0);
    assert_eq!(throughput_metric.delta, 0.0);

    let wait_metric = comparison
        .metrics
        .iter()
        .find(|m| m.metric_name == "mean_wait")
        .expect("mean_wait metric must be present");
    assert!(
        wait_metric.candidate_value >= wait_metric.base_value,
        "Micro transit contention cannot reduce wait times: candidate ({}) >= base ({})",
        wait_metric.candidate_value,
        wait_metric.base_value
    );

    let los_metric = comparison
        .metrics
        .iter()
        .find(|m| m.metric_name == "mean_elapsed")
        .expect("mean_elapsed (LOS) metric must be present");
    assert!(
        los_metric.candidate_value > los_metric.base_value,
        "Micro transit strictly increases length of stay: micro ({}) > macro ({})",
        los_metric.candidate_value,
        los_metric.base_value
    );
    assert!(los_metric.delta > 0.0);
    assert!(los_metric.percent_change.unwrap() > 0.0);

    let peak_metric = comparison
        .metrics
        .iter()
        .find(|m| m.metric_name == "peak_occupancy")
        .expect("peak_occupancy metric must be present");
    assert_eq!(peak_metric.base_value, 3.0);
    assert_eq!(peak_metric.candidate_value, 3.0);
    assert_eq!(peak_metric.delta, 0.0);

    // Assert empirical superiority is not presumed:
    // Neither fidelity is crowned superior; both outputs and percent changes are reported neutrally.
    for m in &comparison.metrics {
        // Confirm all values and deltas are finite and neutral
        assert!(m.base_value.is_finite());
        assert!(m.candidate_value.is_finite());
        assert!(m.delta.is_finite());
        if let Some(pct) = m.percent_change {
            assert!(pct.is_finite());
        }
        // Denominator descriptions are objective
        assert!(!m.denominator_description.is_empty());
    }

    // Diagnostics report objective mathematical changes neutrally
    for diag in &comparison.diagnostics {
        assert!(
            !diag.contains("superior") && !diag.contains("inferior") && !diag.contains("better"),
            "Empirical superiority must not be presumed in diagnostics: {diag}"
        );
    }

    // Verify distance metrics between Macro and Micro LOS distributions
    let macro_los: Vec<f64> = macro_doc
        .summary
        .patients
        .iter()
        .filter_map(|p| p.elapsed_ticks.map(|e| e as f64))
        .collect();
    let micro_los: Vec<f64> = micro_doc
        .summary
        .patients
        .iter()
        .filter_map(|p| p.elapsed_ticks.map(|e| e as f64))
        .collect();

    let w1 = compute_w1_distance(&macro_los, &micro_los);
    let ks = compute_ks_statistic(&macro_los, &micro_los);
    let residuals = compute_paired_residuals(&macro_los, &micro_los).unwrap();

    assert!(
        w1 > 0.0,
        "W1 distance between Macro and Micro must be non-zero"
    );
    assert!(
        ks > 0.0,
        "KS statistic between Macro and Micro must be non-zero"
    );
    assert!(
        residuals.mean_residual > 0.0,
        "Micro LOS minus Macro LOS residual must be positive"
    );

    // Verify joint queueing conservation across all completed patients
    for p in &micro_doc.summary.patients {
        let wait = p.wait_ticks.unwrap_or(0);
        let work = p.work_ticks;
        let elapsed = p.elapsed_ticks.unwrap_or(0);
        assert!(
            check_joint_queueing_conservation(wait, work, 0, elapsed),
            "Joint queueing relationship must hold for patient {}",
            p.patient_id
        );
    }
}

/// 2. `test_orthogonal_exogenous_vs_generative_arrivals_macro_micro`
///
/// Tests the full 2x2 matrix:
/// (1) Exogenous Arrivals x Macro
/// (2) Exogenous Arrivals x Micro
/// (3) Endogenous/Generative Arrivals x Macro
/// (4) Endogenous/Generative Arrivals x Micro
#[test]
fn test_orthogonal_exogenous_vs_generative_arrivals_macro_micro() {
    let graph = build_synthetic_graph();
    let eligibilities = build_multi_zone_eligibilities();

    // 1. Exogenous historical holdout schedule (fixed demand)
    let exogenous_schedule = vec![
        (1001u64, 0u64, 2u8, 10u64, "zone.acute", "loc.acute.bay1"),
        (
            1002u64,
            5u64,
            4u8,
            8u64,
            "zone.subacute",
            "loc.subacute.bay1",
        ),
        (1003u64, 10u64, 1u8, 6u64, "zone.acute", "loc.acute.bay1"),
        (
            1004u64,
            15u64,
            3u8,
            8u64,
            "zone.subacute",
            "loc.subacute.bay1",
        ),
    ];

    let run_exogenous = |fidelity: ExecutionFidelity| {
        let mut fixture =
            CompositeEdFixture::new(2, 2, "staff.ed_nurse", eligibilities.clone(), fidelity);
        fixture.set_graph(graph.clone());

        for &(id, arr_tick, acuity, work_ticks, zone, loc) in &exogenous_schedule {
            let eff_work = match fidelity {
                ExecutionFidelity::Macro => work_ticks,
                ExecutionFidelity::Micro => work_ticks + 20, // 20 ticks transit added
            };
            fixture.admit_patient(ComposedPatient {
                id,
                name: format!("Exog_{id}"),
                acuity,
                arrival_tick: arr_tick,
                work_required_ticks: eff_work,
                work_elapsed_ticks: 0,
                start_tick: None,
                completion_tick: None,
                assigned_bay: None,
                lifecycle: PatientLifecycle::Arrived,
                task_class_id: "task.treatment".to_string(),
                zone_id: zone.to_string(),
                location_id: loc.to_string(),
            });
        }

        for _ in 0..100 {
            fixture.advance_tick();
        }
        fixture.to_run_document("exog-run", 42, 100)
    };

    // Cell (1): Exogenous x Macro
    let exog_macro = run_exogenous(ExecutionFidelity::Macro);
    // Cell (2): Exogenous x Micro
    let exog_micro = run_exogenous(ExecutionFidelity::Micro);

    // Verify fixed demand under exogenous arrivals:
    // Identical presentation count, identical arrival ticks, identical patient identities
    assert_eq!(exog_macro.summary.arrivals, 4);
    assert_eq!(exog_micro.summary.arrivals, 4);
    for (m, u) in exog_macro
        .summary
        .patients
        .iter()
        .zip(exog_micro.summary.patients.iter())
    {
        assert_eq!(m.patient_id, u.patient_id);
        assert_eq!(m.arrival_tick, u.arrival_tick);
    }

    // For uncongested patient Exog_1001 (arrived at 0, dispatched at 0), individual LOS delta is exactly 20 ticks
    let m_p1 = exog_macro
        .summary
        .patients
        .iter()
        .find(|p| p.patient_id == "Exog_1001")
        .unwrap();
    let u_p1 = exog_micro
        .summary
        .patients
        .iter()
        .find(|p| p.patient_id == "Exog_1001")
        .unwrap();
    assert_eq!(
        u_p1.elapsed_ticks.unwrap() - m_p1.elapsed_ticks.unwrap(),
        20,
        "Individual uncongested patient has exact synthetic transit delta"
    );

    // Overall mean LOS delta reflects route transit plus queueing capacity multiplier
    let exog_comp = compare_run_documents(&exog_macro, &exog_micro).unwrap();
    let exog_los = exog_comp
        .metrics
        .iter()
        .find(|m| m.metric_name == "mean_elapsed")
        .unwrap();
    assert!(
        exog_los.delta >= 20.0,
        "Under fixed exogenous demand, Micro LOS delta reflects synthetic route transit plus queueing"
    );

    // 2. Endogenous/Generative arrival model
    let arrival_config = ArrivalConfig {
        version: 1,
        seed: 42,
        horizon_ticks: 50,
        provenance: "synthetic_tick_fixture".to_string(),
        bins: vec![
            ArrivalBin {
                start_tick: 0,
                end_tick: 25,
                modes: vec![ArrivalModeCounts {
                    mode: "walk_in".to_string(),
                    total: 3,
                    acuity_counts: [0, 1, 1, 1, 0],
                }],
            },
            ArrivalBin {
                start_tick: 25,
                end_tick: 50,
                modes: vec![ArrivalModeCounts {
                    mode: "ambulance".to_string(),
                    total: 2,
                    acuity_counts: [1, 1, 0, 0, 0],
                }],
            },
        ],
    };

    let generated_arrivals =
        generate_arrivals(&arrival_config).expect("generative arrivals generation must succeed");
    assert_eq!(generated_arrivals.len(), 5);

    let run_generative = |fidelity: ExecutionFidelity| {
        let mut fixture =
            CompositeEdFixture::new(2, 2, "staff.ed_nurse", eligibilities.clone(), fidelity);
        fixture.set_graph(graph.clone());

        for (idx, gen) in generated_arrivals.iter().enumerate() {
            let zone = if gen.acuity <= 2 {
                "zone.acute"
            } else {
                "zone.subacute"
            };
            let loc = if gen.acuity <= 2 {
                "loc.acute.bay1"
            } else {
                "loc.subacute.bay1"
            };
            let eff_work = match fidelity {
                ExecutionFidelity::Macro => 6,
                ExecutionFidelity::Micro => 6 + 15, // 15 ticks transit added
            };

            fixture.admit_patient(ComposedPatient {
                id: (idx + 1) as u64,
                name: gen.patient_id.clone(),
                acuity: gen.acuity,
                arrival_tick: gen.arrival_tick,
                work_required_ticks: eff_work,
                work_elapsed_ticks: 0,
                start_tick: None,
                completion_tick: None,
                assigned_bay: None,
                lifecycle: PatientLifecycle::Arrived,
                task_class_id: "task.treatment".to_string(),
                zone_id: zone.to_string(),
                location_id: loc.to_string(),
            });
        }

        for _ in 0..100 {
            fixture.advance_tick();
        }
        fixture.to_run_document("gen-run", 42, 100)
    };

    // Cell (3): Generative x Macro
    let gen_macro = run_generative(ExecutionFidelity::Macro);
    // Cell (4): Generative x Micro
    let gen_micro = run_generative(ExecutionFidelity::Micro);

    // Verify generative arrivals validate arrival process:
    assert_eq!(gen_macro.summary.arrivals, 5);
    assert_eq!(gen_micro.summary.arrivals, 5);
    let gen_comp = compare_run_documents(&gen_macro, &gen_micro).unwrap();
    let gen_los = gen_comp
        .metrics
        .iter()
        .find(|m| m.metric_name == "mean_elapsed")
        .unwrap();
    assert!(
        gen_los.delta >= 15.0,
        "Generative arrivals validate arrival process with orthogonal fidelity scaling"
    );

    // Cross-comparison between exogenous and generative:
    // Exogenous arrivals had fixed 4 patients; Generative arrivals had 5 sampled patients across bins
    assert_ne!(exog_macro.summary.arrivals, gen_macro.summary.arrivals);
}

/// 3. `test_stochastic_and_structural_uncertainty_reporting`
///
/// Runs fresh-seed replications, evaluates stochastic uncertainty (variance, 95% CI)
/// and structural uncertainty (Macro vs Micro delta), and reports exclusions and runtime.
#[test]
fn test_stochastic_and_structural_uncertainty_reporting() {
    let start_instant = Instant::now();
    let graph = build_synthetic_graph();
    let eligibilities = build_multi_zone_eligibilities();

    // Fresh seeds for replications
    let replication_seeds = [101u64, 202, 303, 404, 505];
    let num_replications = replication_seeds.len();

    let mut macro_mean_los_series = Vec::with_capacity(num_replications);
    let mut micro_mean_los_series = Vec::with_capacity(num_replications);
    let mut structural_deltas = Vec::with_capacity(num_replications);
    let mut macro_exclusions = Vec::with_capacity(num_replications);
    let mut micro_exclusions = Vec::with_capacity(num_replications);

    // Tight horizon cutoff to induce and test exclusions (censored presentations)
    let tight_horizon_ticks = 30u64;

    for &seed in &replication_seeds {
        let arrival_config = ArrivalConfig {
            version: 1,
            seed,
            horizon_ticks: tight_horizon_ticks,
            provenance: "synthetic_tick_fixture".to_string(),
            bins: vec![
                ArrivalBin {
                    start_tick: 0,
                    end_tick: 15,
                    modes: vec![ArrivalModeCounts {
                        mode: "walk_in".to_string(),
                        total: 3,
                        acuity_counts: [1, 1, 1, 0, 0],
                    }],
                },
                ArrivalBin {
                    start_tick: 15,
                    end_tick: 30,
                    modes: vec![ArrivalModeCounts {
                        mode: "ambulance".to_string(),
                        total: 3,
                        acuity_counts: [1, 1, 1, 0, 0],
                    }],
                },
            ],
        };

        let arrivals = generate_arrivals(&arrival_config).expect("seeded arrivals must succeed");

        let run_rep = |fidelity: ExecutionFidelity| {
            let mut fixture =
                CompositeEdFixture::new(2, 2, "staff.ed_nurse", eligibilities.clone(), fidelity);
            fixture.set_graph(graph.clone());

            for (idx, gen) in arrivals.iter().enumerate() {
                let zone = if gen.acuity <= 2 {
                    "zone.acute"
                } else {
                    "zone.subacute"
                };
                let loc = if gen.acuity <= 2 {
                    "loc.acute.bay1"
                } else {
                    "loc.subacute.bay1"
                };
                // Substantial service work guarantees capacity saturation near horizon cutoff
                let eff_work = match fidelity {
                    ExecutionFidelity::Macro => 12,
                    ExecutionFidelity::Micro => 12 + 10, // 10 ticks transit added
                };

                fixture.admit_patient(ComposedPatient {
                    id: (idx + 1) as u64,
                    name: format!("Rep_{seed}_{}", gen.patient_id),
                    acuity: gen.acuity,
                    arrival_tick: gen.arrival_tick,
                    work_required_ticks: eff_work,
                    work_elapsed_ticks: 0,
                    start_tick: None,
                    completion_tick: None,
                    assigned_bay: None,
                    lifecycle: PatientLifecycle::Arrived,
                    task_class_id: "task.treatment".to_string(),
                    zone_id: zone.to_string(),
                    location_id: loc.to_string(),
                });
            }

            for current_tick in 0..tight_horizon_ticks {
                // Admit / progress
                let _ = current_tick;
                fixture.advance_tick();
            }
            fixture.to_run_document(&format!("rep-{seed}"), seed, tight_horizon_ticks)
        };

        let macro_doc = run_rep(ExecutionFidelity::Macro);
        let micro_doc = run_rep(ExecutionFidelity::Micro);

        let m_los: Vec<u64> = macro_doc
            .summary
            .patients
            .iter()
            .filter_map(|p| p.elapsed_ticks)
            .collect();
        let u_los: Vec<u64> = micro_doc
            .summary
            .patients
            .iter()
            .filter_map(|p| p.elapsed_ticks)
            .collect();

        let m_mean = if !m_los.is_empty() {
            m_los.iter().sum::<u64>() as f64 / m_los.len() as f64
        } else {
            0.0
        };
        let u_mean = if !u_los.is_empty() {
            u_los.iter().sum::<u64>() as f64 / u_los.len() as f64
        } else {
            0.0
        };

        macro_mean_los_series.push(m_mean);
        micro_mean_los_series.push(u_mean);
        structural_deltas.push(u_mean - m_mean);

        // Record exclusions (unfinished/censored cases at horizon cutoff)
        macro_exclusions.push(macro_doc.summary.unfinished);
        micro_exclusions.push(micro_doc.summary.unfinished);
    }

    // Measure stochastic uncertainty across replications
    let n = num_replications as f64;
    let macro_mean = macro_mean_los_series.iter().sum::<f64>() / n;
    let macro_variance = macro_mean_los_series
        .iter()
        .map(|&x| (x - macro_mean).powi(2))
        .sum::<f64>()
        / (n - 1.0);
    let macro_std_error = (macro_variance / n).sqrt();
    let macro_ci95_lower = macro_mean - 1.96 * macro_std_error;
    let macro_ci95_upper = macro_mean + 1.96 * macro_std_error;

    let micro_mean = micro_mean_los_series.iter().sum::<f64>() / n;
    let micro_variance = micro_mean_los_series
        .iter()
        .map(|&x| (x - micro_mean).powi(2))
        .sum::<f64>()
        / (n - 1.0);
    let micro_std_error = (micro_variance / n).sqrt();
    let micro_ci95_lower = micro_mean - 1.96 * micro_std_error;
    let micro_ci95_upper = micro_mean + 1.96 * micro_std_error;

    // Structural uncertainty: average Macro vs Micro difference
    let mean_structural_delta = structural_deltas.iter().sum::<f64>() / n;

    // Report exclusions (censored presentations at horizon cutoff)
    let total_macro_exclusions: usize = macro_exclusions.iter().sum();
    let total_micro_exclusions: usize = micro_exclusions.iter().sum();
    assert!(
        total_macro_exclusions > 0,
        "Tight horizon must record exclusions (unfinished presentations at cutoff)"
    );
    assert!(
        total_micro_exclusions >= total_macro_exclusions,
        "Micro transit congestion yields equal or higher horizon exclusions: micro ({}) >= macro ({})",
        total_micro_exclusions,
        total_macro_exclusions
    );

    // Execution runtime reporting
    let execution_duration = start_instant.elapsed();
    assert!(
        execution_duration.as_millis() < 5000,
        "Holdout evaluation runtime must complete within bounded budget"
    );

    // Verifications of stochastic and structural uncertainty properties:
    assert!(
        macro_mean > 0.0,
        "Macro mean LOS across replications must be positive"
    );
    assert!(
        micro_mean > macro_mean,
        "Micro mean LOS must strictly exceed Macro mean due to structural transit: micro ({}) > macro ({})",
        micro_mean,
        macro_mean
    );
    assert!(
        mean_structural_delta > 0.0,
        "Structural delta must be positive"
    );

    // Stochastic CI bounds are ordered
    assert!(macro_ci95_lower <= macro_ci95_upper);
    assert!(micro_ci95_lower <= micro_ci95_upper);

    // Compute distribution distance across replication summaries
    let w1 = compute_w1_distance(&macro_mean_los_series, &micro_mean_los_series);
    let ks = compute_ks_statistic(&macro_mean_los_series, &micro_mean_los_series);
    assert!(w1 > 0.0);
    assert!(ks > 0.0);
}
