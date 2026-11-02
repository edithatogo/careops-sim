use careops_birthing::{BirthingEpisode, EventCategory, EventEffect, PatientState, ServiceState};
use careops_core::pathway::{
    CompetingRiskModel, DecisionPolicy, HazardRate, PathwayRngStreams, ScheduledEvent,
};
use serde_json::Value;
use std::collections::{BTreeSet, HashMap};

const FIXTURE: &str =
    include_str!("../../../model-inputs/birthing/fixtures/b1-progression-cases.synthetic.json");

fn patient(value: &str) -> PatientState {
    value
        .parse()
        .expect("fixture patient state must be supported")
}

fn service(value: &str) -> ServiceState {
    value
        .parse()
        .expect("fixture service state must be supported")
}

fn id_ranks<'a>(values: impl Iterator<Item = &'a str>) -> HashMap<String, u64> {
    let sorted: BTreeSet<String> = values.map(str::to_owned).collect();
    sorted
        .into_iter()
        .enumerate()
        .map(|(rank, value)| (value, rank as u64 + 1))
        .collect()
}

#[test]
fn accepted_b11_fixture_executes_through_independent_patient_and_service_states() {
    let fixture: Value = serde_json::from_str(FIXTURE).unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    let event_ids = id_ranks(cases.iter().flat_map(|case| {
        case["events"]
            .as_array()
            .unwrap()
            .iter()
            .map(|event| event["event_id"].as_str().unwrap())
            .chain(
                case["cancelled_event_ids"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|id| id.as_str().unwrap()),
            )
    }));
    let patient_ids = id_ranks(
        cases
            .iter()
            .map(|case| case["patient_id"].as_str().unwrap()),
    );

    for case in cases {
        let events = case["events"].as_array().unwrap();
        let first = &events[0];
        let patient_id = patient_ids[case["patient_id"].as_str().unwrap()];
        let mut episode = BirthingEpisode::new(
            patient_id,
            patient(first["patient_state"].as_str().unwrap()),
            service(first["service_state"].as_str().unwrap()),
            41,
        );
        let cancelled: Vec<&str> = case["cancelled_event_ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|id| id.as_str().unwrap())
            .collect();

        for event in events {
            let kind = event["kind"].as_str().unwrap();
            let label = event["event_id"].as_str().unwrap();
            let tick = event["tick"].as_u64().unwrap();
            let priority = event["priority"].as_u64().unwrap() as u8;
            let target_patient = patient(event["patient_state"].as_str().unwrap());
            let target_service = service(event["service_state"].as_str().unwrap());

            // Pre-schedule the explicitly cancelled progression before deterioration.
            if kind == "deterioration" {
                for stale_id in &cancelled {
                    episode
                        .schedule(
                            event_ids[*stale_id],
                            *stale_id,
                            tick + 1,
                            priority,
                            EventEffect::progression(PatientState::ActiveLabour),
                        )
                        .unwrap();
                }
            }

            let effect = match kind {
                "admission" => EventEffect::combined(
                    Some(target_patient),
                    Some(target_service),
                    EventCategory::Routine,
                ),
                "labour_progression" => EventEffect::progression(target_patient),
                "deterioration" => EventEffect::deterioration(target_patient, Some(target_service)),
                "intervention_induction_start" | "intervention_decision" => {
                    EventEffect::intervention(target_patient, Some(target_service))
                }
                "service_reservation" | "transfer_departure" | "transfer_arrival" => {
                    EventEffect::service(target_service)
                }
                other => panic!("unsupported fixture event kind: {other}"),
            };
            episode
                .schedule(event_ids[label], label, tick, priority, effect)
                .unwrap();
            episode.advance_through(tick).unwrap();
            let trace = episode.trace().last().unwrap();
            assert_eq!(trace.event_id, label);
            assert_eq!(trace.patient_state, target_patient);
            assert_eq!(trace.service_state, target_service);
        }

        if !cancelled.is_empty() {
            episode
                .advance_through(episode.observation().tick + 1)
                .unwrap();
            assert_eq!(episode.stale_event_ids(), cancelled.as_slice());
            assert!(episode.generation() > 0);
        }

        let horizon = episode.observation().tick + 2;
        let censored = episode.censor_at(horizon).unwrap();
        assert!(
            censored.censored,
            "unfinished synthetic episode should censor"
        );
        assert_eq!(censored.censor_tick, Some(horizon));
        assert_eq!(censored.patient_state, episode.observation().patient_state);
        assert_eq!(censored.service_state, episode.observation().service_state);
    }
}

#[test]
fn shuffled_ties_follow_tick_priority_event_id_then_patient_id() {
    let event = |event_id, tick, priority, patient_id| ScheduledEvent {
        event_id,
        tick,
        priority,
        patient_id,
        generation: 0,
        description: format!("event-{event_id}"),
    };
    let mut shuffled = [
        event(8, 10, 1, 5),
        event(8, 10, 1, 3),
        event(7, 10, 1, 9),
        event(20, 10, 0, 50),
        event(1, 9, 9, 1),
    ];
    shuffled.sort();
    let keys: Vec<_> = shuffled
        .iter()
        .map(|item| (item.tick, item.priority, item.event_id, item.patient_id))
        .collect();
    assert_eq!(
        keys,
        vec![
            (9, 9, 1, 1),
            (10, 0, 20, 50),
            (10, 1, 7, 9),
            (10, 1, 8, 3),
            (10, 1, 8, 5)
        ]
    );
}

#[test]
fn hazard_updates_sample_only_the_progression_stream() {
    let mut episode = BirthingEpisode::new(
        1,
        PatientState::LatentLabour,
        ServiceState::WaitingForSuite,
        77,
    );
    let covariates = HashMap::new();
    let cause_states = HashMap::from([("progress".to_string(), PatientState::ActiveLabour)]);

    // Zero hazard is a synthetic control and consumes no random draws.
    let zero = CompetingRiskModel::new();
    assert!(!episode
        .sample_progression_tick(1, "no-progression", 1, &zero, &covariates, &cause_states)
        .unwrap());
    assert_eq!(episode.draw_counts(), (0, 0, 0));

    // A test-only high rate makes this synthetic event certain; it is not a clinical value.
    let mut updated = CompetingRiskModel::new();
    updated.add_hazard(HazardRate::new("progress", 100.0));
    assert!(episode
        .sample_progression_tick(
            2,
            "updated-progression",
            1,
            &updated,
            &covariates,
            &cause_states
        )
        .unwrap());
    assert_eq!(episode.draw_counts(), (2, 0, 0));
    episode.advance_through(1).unwrap();
    assert_eq!(
        episode.observation().patient_state,
        PatientState::ActiveLabour
    );

    // The same progression seed produces the same cause despite independent service draws.
    let mut baseline = PathwayRngStreams::new(77);
    baseline.service_stream.next_uniform();
    let baseline_cause = updated
        .sample_event(&covariates, &mut baseline.progression_stream)
        .unwrap();
    assert_eq!(baseline_cause.as_deref(), Some("progress"));
    assert_eq!(baseline.progression_stream.draw_count, 2);
    assert_eq!(baseline.service_stream.draw_count, 1);
    assert_eq!(baseline.routing_stream.draw_count, 0);
}

#[test]
fn horizon_censoring_does_not_execute_future_events_or_change_states() {
    let mut episode =
        BirthingEpisode::new(9, PatientState::ActiveLabour, ServiceState::InTransit, 4);
    episode
        .schedule(
            1,
            "future-arrival",
            5,
            2,
            EventEffect::service(ServiceState::InSuite),
        )
        .unwrap();
    episode.advance_through(2).unwrap();
    let before = episode.observation();
    let censored = episode.censor_at(3).unwrap();
    assert!(censored.censored);
    assert_eq!(censored.censor_tick, Some(3));
    assert_eq!(censored.patient_state, before.patient_state);
    assert_eq!(censored.service_state, before.service_state);
    assert!(episode.trace().is_empty());
    assert!(episode.advance_through(5).is_err());
}

#[test]
fn updated_hazard_invalidates_the_old_progression_before_resampling() {
    let mut episode = BirthingEpisode::new(
        2,
        PatientState::ActiveLabour,
        ServiceState::WaitingForSuite,
        13,
    );
    let covariates = HashMap::new();
    let mut old_model = CompetingRiskModel::new();
    old_model.add_hazard(HazardRate::new("old-state", 100.0));
    let old_targets = HashMap::from([("old-state".to_string(), PatientState::LaterLabourStage)]);
    assert!(episode
        .sample_progression_tick(
            10,
            "old-progression",
            1,
            &old_model,
            &covariates,
            &old_targets
        )
        .unwrap());

    let mut updated_model = CompetingRiskModel::new();
    updated_model.add_hazard(HazardRate::new("new-state", 100.0));
    let updated_targets =
        HashMap::from([("new-state".to_string(), PatientState::DeteriorationObserved)]);
    assert!(episode
        .sample_progression_tick(
            11,
            "updated-progression",
            1,
            &updated_model,
            &covariates,
            &updated_targets
        )
        .unwrap());
    episode.advance_through(1).unwrap();

    assert_eq!(episode.stale_event_ids(), &["old-progression"]);
    assert_eq!(episode.trace().len(), 1);
    assert_eq!(episode.trace()[0].event_id, "updated-progression");
    assert_eq!(
        episode.observation().patient_state,
        PatientState::DeteriorationObserved
    );
}

fn synthetic_urgency(assessed_at_tick: u64) -> careops_core::pathway::VersionedUrgencyAssessment {
    careops_core::pathway::VersionedUrgencyAssessment {
        jurisdiction_id: "synthetic-test-only".to_string(),
        scheme_id: "synthetic-test-scheme".to_string(),
        scheme_version: "v1".to_string(),
        category: 3,
        assessed_at_tick,
        effective_date: "2099-01-01".to_string(),
        source_authority: "synthetic test fixture".to_string(),
        citation: "not a policy source".to_string(),
        source_sha256: None,
    }
}

struct EchoObservedPolicy;

impl careops_core::pathway::DecisionPolicy for EchoObservedPolicy {
    fn decide(
        &self,
        observed: &careops_core::pathway::ObservedPolicyState,
    ) -> careops_core::pathway::PolicyDecision {
        careops_core::pathway::PolicyDecision {
            urgency: careops_core::pathway::UrgencyCategory(observed.urgency.category),
            service_target: careops_core::pathway::ServiceTargetStatus::WithinTarget,
            dispatch_priority: Some(careops_core::pathway::DispatchPriority(
                observed.urgency.scheme_version.clone(),
            )),
            interruption: careops_core::pathway::InterruptionEligibility::NotEligible,
        }
    }
}

#[test]
fn policy_input_excludes_patient_service_and_future_event_state() {
    let mut with_future = BirthingEpisode::new(
        1,
        PatientState::LatentLabour,
        ServiceState::WaitingForSuite,
        1,
    );
    with_future
        .schedule(
            1,
            "future-deterioration",
            100,
            1,
            EventEffect::deterioration(PatientState::DeteriorationObserved, None),
        )
        .unwrap();
    let without_future = BirthingEpisode::new(
        2,
        PatientState::DeteriorationObserved,
        ServiceState::InTheatre,
        9,
    );

    let assessment = synthetic_urgency(0);
    let left = with_future.policy_input(assessment.clone(), 0).unwrap();
    let right = without_future.policy_input(assessment.clone(), 0).unwrap();
    assert_eq!(left, right);
    assert_eq!(left.urgency, assessment);
    let policy = EchoObservedPolicy;
    assert_eq!(policy.decide(&left), policy.decide(&right));
    assert!(with_future.policy_input(synthetic_urgency(1), 0).is_err());
}

#[test]
fn bounded_synthetic_trajectories_replay_within_horizon_and_event_limit() {
    const HORIZON: u64 = 32;
    const MAX_EVENTS: usize = 32;

    fn run(
        seed: u64,
    ) -> (
        Vec<careops_birthing::StateTraceEntry>,
        careops_birthing::EpisodeObservation,
    ) {
        let mut episode = BirthingEpisode::new(
            seed,
            PatientState::LatentLabour,
            ServiceState::WaitingForSuite,
            seed,
        );
        let mut model = CompetingRiskModel::new();
        model.add_hazard(HazardRate::new("synthetic-progress-only", 0.25));
        let covariates = HashMap::new();
        let cause_states = HashMap::from([(
            "synthetic-progress-only".to_string(),
            PatientState::ActiveLabour,
        )]);
        episode
            .schedule(
                10_000,
                "after-horizon-service",
                HORIZON + 1,
                2,
                EventEffect::service(ServiceState::InSuite),
            )
            .unwrap();

        for tick in 0..HORIZON {
            episode
                .sample_progression_tick(
                    tick + 1,
                    format!("synthetic-progression-{tick}"),
                    1,
                    &model,
                    &covariates,
                    &cause_states,
                )
                .unwrap();
            episode.advance_through(tick + 1).unwrap();
            assert_eq!(episode.observation().tick, tick + 1);
            assert!(episode.trace().len() <= MAX_EVENTS);
        }
        let observation = episode.censor_at(HORIZON).unwrap();
        assert!(observation.censored);
        assert_eq!(observation.censor_tick, Some(HORIZON));
        assert!(episode.trace().len() <= MAX_EVENTS);
        assert!(episode.trace().iter().all(|event| event.tick <= HORIZON));
        assert!(!episode
            .trace()
            .iter()
            .any(|event| event.event_id == "after-horizon-service"));
        assert!(episode
            .trace()
            .windows(2)
            .all(|pair| pair[0].tick < pair[1].tick));
        (episode.trace().to_vec(), observation)
    }

    for seed in 1..=16 {
        assert_eq!(run(seed), run(seed), "same seed must replay: {seed}");
    }
}
