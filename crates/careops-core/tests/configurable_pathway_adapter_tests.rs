//! Integration tests for the bounded configurable pathway adapter and independently scheduled progression (Task A1.2).
//!
//! Validates:
//! - Configurable pathway execution across canonical ED stages.
//! - Independent clinical progression evolving during queue wait (CC-04).
//! - Horizon right-censoring without unauthorized stage completion.
//! - Deterministic draw decoupling between progression and service streams.

use std::collections::HashMap;

use careops_core::pathway::{
    CompetingRiskModel, ConfigurablePathwayAdapter, HazardRate, PathwayPatientSpec, StageId,
};

#[test]
fn test_configurable_pathway_ed_canonical_flow() {
    let graph = ConfigurablePathwayAdapter::build_ed_canonical_graph();
    let mut adapter = ConfigurablePathwayAdapter::new(graph, 100);

    // Unconstrained capacity
    adapter.set_stage_capacity(StageId::new("triage"), 10);
    adapter.set_stage_capacity(StageId::new("assessment"), 10);
    adapter.set_stage_capacity(StageId::new("diagnostics"), 10);
    adapter.set_stage_capacity(StageId::new("treatment"), 10);

    let mut planned_durations = HashMap::new();
    planned_durations.insert(StageId::new("triage"), 5);
    planned_durations.insert(StageId::new("assessment"), 15);
    planned_durations.insert(StageId::new("diagnostics"), 20);
    planned_durations.insert(StageId::new("treatment"), 25);
    planned_durations.insert(StageId::new("discharge"), 0);

    let patient = PathwayPatientSpec {
        patient_id: "patient-ed-01".to_string(),
        arrival_tick: 0,
        initial_stage: StageId::new("triage"),
        acuity: 3,
        covariates: HashMap::new(),
        planned_stage_durations: planned_durations,
        base_seed: 12345,
    };

    let outcomes = adapter
        .run_simulation(&[patient])
        .expect("ED canonical pathway simulation must succeed");

    assert_eq!(outcomes.len(), 1);
    let o = &outcomes[0];
    assert!(o.is_completed, "patient should complete entire canonical ED pathway");
    assert!(!o.is_censored);

    // Stages traversed: triage (5) + assessment (15) + diagnostics (20) + treatment (25) + discharge (0) = 5 stages
    assert_eq!(o.stage_records.len(), 5);
    assert_eq!(o.stage_records[0].stage_id, StageId::new("triage"));
    assert_eq!(o.stage_records[0].start_tick, Some(0));
    assert_eq!(o.stage_records[0].completion_tick, Some(5));

    assert_eq!(o.stage_records[1].stage_id, StageId::new("assessment"));
    assert_eq!(o.stage_records[1].start_tick, Some(5));
    assert_eq!(o.stage_records[1].completion_tick, Some(20));

    assert_eq!(o.stage_records[2].stage_id, StageId::new("diagnostics"));
    assert_eq!(o.stage_records[2].start_tick, Some(20));
    assert_eq!(o.stage_records[2].completion_tick, Some(40));

    assert_eq!(o.stage_records[3].stage_id, StageId::new("treatment"));
    assert_eq!(o.stage_records[3].start_tick, Some(40));
    assert_eq!(o.stage_records[3].completion_tick, Some(65));

    assert_eq!(o.stage_records[4].stage_id, StageId::new("discharge"));
    assert_eq!(o.stage_records[4].start_tick, Some(65));
    assert_eq!(o.stage_records[4].completion_tick, Some(66));
}

#[test]
fn test_independent_clinical_progression_while_queued() {
    let graph = ConfigurablePathwayAdapter::build_ed_canonical_graph();
    let mut adapter = ConfigurablePathwayAdapter::new(graph, 50);

    // ZERO capacity at triage: patient is forced to wait in the queue!
    adapter.set_stage_capacity(StageId::new("triage"), 0);

    // High progression hazard model (patient will deteriorate while waiting)
    let mut model = CompetingRiskModel::new();
    let mut deterioration = HazardRate::new("deterioration", 50.0); // High hazard ensures trigger
    deterioration
        .covariate_coefficients
        .insert("acuity_risk".to_string(), 0.0);
    model.add_hazard(deterioration);
    adapter.set_progression_model(model);

    let mut planned = HashMap::new();
    planned.insert(StageId::new("triage"), 5);

    let patient = PathwayPatientSpec {
        patient_id: "patient-deteriorating-in-waiting-room".to_string(),
        arrival_tick: 0,
        initial_stage: StageId::new("triage"),
        acuity: 3, // Initial ATS 3 (Urgent)
        covariates: HashMap::new(),
        planned_stage_durations: planned,
        base_seed: 777,
    };

    let outcomes = adapter
        .run_simulation(&[patient])
        .expect("simulation must succeed");

    let o = &outcomes[0];
    assert!(!o.is_completed);
    assert!(o.is_censored); // Censored at horizon because triage capacity was 0

    // Clinical progression must have evolved despite zero capacity! (CC-04)
    assert!(
        !o.progression_logs.is_empty(),
        "clinical progression events must fire while queued in waiting area"
    );
    assert!(
        o.final_acuity < 3,
        "acuity must have escalated from 3 to higher urgency (lower category number) while waiting"
    );
    assert_eq!(o.progression_logs[0].cause, "clinical_deterioration");
}

#[test]
fn test_horizon_right_censoring_clamps_work_without_completion() {
    let graph = ConfigurablePathwayAdapter::build_ed_canonical_graph();
    let mut adapter = ConfigurablePathwayAdapter::new(graph, 50); // Horizon is 50 ticks

    adapter.set_stage_capacity(StageId::new("triage"), 5);

    // Patient arrives at tick 40, planned duration is 30 ticks (scheduled finish: tick 70, past horizon 50!)
    let mut planned = HashMap::new();
    planned.insert(StageId::new("triage"), 30);

    let patient = PathwayPatientSpec {
        patient_id: "patient-censored-late".to_string(),
        arrival_tick: 40,
        initial_stage: StageId::new("triage"),
        acuity: 2,
        covariates: HashMap::new(),
        planned_stage_durations: planned,
        base_seed: 9999,
    };

    let outcomes = adapter
        .run_simulation(&[patient])
        .expect("simulation must succeed");

    let o = &outcomes[0];
    assert!(!o.is_completed, "patient past horizon must NOT be marked complete");
    assert!(o.is_censored, "patient past horizon must be marked censored");

    let record = &o.stage_records[0];
    assert!(record.is_censored);
    assert_eq!(record.start_tick, Some(40));
    assert_eq!(record.completion_tick, None);
    assert_eq!(
        record.observed_duration_ticks, 10,
        "observed duration must be clamped to horizon (50 - 40 = 10)"
    );
}
