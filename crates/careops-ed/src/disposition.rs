//! Experimental synthetic provisional decision observations.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::pathway::{
    PathwayConfig, PathwayError, PathwayResult, ServiceBinding, ServiceProvenance, StageKind,
};

const MAX_DECISION_PATIENTS: usize = 100_000;
const MAX_DIAGNOSTIC_STAGES: usize = 16;

#[path = "boarding.rs"]
pub mod boarding;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DispositionIntent {
    Discharge,
    Admission,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionReveal {
    TreatmentCompletion,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionPolicy {
    pub version: u32,
    pub provenance: ServiceProvenance,
    pub reveal_at: DecisionReveal,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DispositionBinding {
    pub patient_id: String,
    pub latent_intent: DispositionIntent,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DecisionObservation {
    pub policy_version: u32,
    pub provenance: ServiceProvenance,
    pub observed_tick: u64,
    pub provisional_intent: DispositionIntent,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DispositionResult {
    pub pathway: PathwayResult,
    pub decision: Option<DecisionObservation>,
}

/// Run legacy supplied fixed work and reveal a provisional intent only when treatment completes.
pub fn run_with_disposition(
    config: &PathwayConfig,
    policy: &DecisionPolicy,
    decisions: &[DispositionBinding],
) -> Result<Vec<DispositionResult>, PathwayError> {
    let intents = validate_inputs(config, policy, decisions)?;
    let pathways = crate::pathway::run_pathway(config)?;
    attach_observations(pathways, policy, &intents)
}

/// Run configured fixed/discrete work and reveal a provisional intent at treatment completion.
pub fn run_with_disposition_and_services(
    config: &PathwayConfig,
    seed: u64,
    services: &[ServiceBinding],
    policy: &DecisionPolicy,
    decisions: &[DispositionBinding],
) -> Result<Vec<DispositionResult>, PathwayError> {
    let intents = validate_inputs(config, policy, decisions)?;
    let pathways = crate::pathway::run_pathway_with_services(config, seed, services)?;
    attach_observations(pathways, policy, &intents)
}

fn validate_inputs(
    config: &PathwayConfig,
    policy: &DecisionPolicy,
    decisions: &[DispositionBinding],
) -> Result<HashMap<String, DispositionIntent>, PathwayError> {
    if policy.version != 1
        || policy.provenance.class != "synthetic"
        || policy.provenance.notes.trim().is_empty()
        || policy.provenance.notes.trim() != policy.provenance.notes
    {
        return invalid("decision policy must use version 1 and trimmed synthetic provenance");
    }
    if config.patients.len() > MAX_DECISION_PATIENTS {
        return invalid("decision patient limit exceeded");
    }
    if decisions.len() > MAX_DECISION_PATIENTS {
        return invalid("decision binding limit exceeded");
    }
    if decisions.len() != config.patients.len() {
        return invalid("exactly one disposition binding is required per patient");
    }

    let mut patient_ids = HashSet::with_capacity(config.patients.len());
    for patient in &config.patients {
        if !valid_patient_id(&patient.patient_id) {
            return invalid("pathway patient ID is invalid");
        }
        if !patient_ids.insert(patient.patient_id.as_str()) {
            return invalid("pathway patient IDs must be unique");
        }
        if !has_full_treatment_pathway(patient) {
            return invalid("disposition requires a treatment-ending pathway");
        }
    }

    let mut intents = HashMap::with_capacity(decisions.len());
    for binding in decisions {
        if !valid_patient_id(&binding.patient_id) {
            return invalid("disposition binding patient ID is invalid");
        }
        if !patient_ids.contains(binding.patient_id.as_str()) {
            return invalid("disposition binding refers to an unknown patient");
        }
        if intents
            .insert(binding.patient_id.clone(), binding.latent_intent)
            .is_some()
        {
            return invalid("duplicate disposition binding");
        }
    }
    if intents.len() != config.patients.len() {
        return invalid("disposition bindings are missing configured patients");
    }
    Ok(intents)
}

fn has_full_treatment_pathway(patient: &crate::pathway::PathwayPatient) -> bool {
    let stages = &patient.stages;
    if stages.len() < 3
        || stages[0].kind != StageKind::Triage
        || stages[1].kind != StageKind::Assessment
        || stages.last().map(|stage| stage.kind) != Some(StageKind::Treatment)
    {
        return false;
    }
    let diagnostics = &stages[2..stages.len() - 1];
    diagnostics.len() <= MAX_DIAGNOSTIC_STAGES
        && diagnostics
            .iter()
            .all(|stage| stage.kind == StageKind::Diagnostics)
}

fn valid_patient_id(value: &str) -> bool {
    let bytes = value.as_bytes();
    !bytes.is_empty()
        && bytes.len() <= 64
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(*byte, b'_' | b'.' | b'-'))
}

fn attach_observations(
    pathways: Vec<PathwayResult>,
    policy: &DecisionPolicy,
    intents: &HashMap<String, DispositionIntent>,
) -> Result<Vec<DispositionResult>, PathwayError> {
    pathways
        .into_iter()
        .map(|pathway| {
            let decision = pathway
                .episodes
                .last()
                .filter(|episode| episode.kind == StageKind::Treatment)
                .and_then(|episode| episode.completion_tick)
                .map(|observed_tick| {
                    intents
                        .get(&pathway.patient_id)
                        .copied()
                        .map(|provisional_intent| DecisionObservation {
                            policy_version: policy.version,
                            provenance: policy.provenance.clone(),
                            observed_tick,
                            provisional_intent,
                        })
                        .ok_or_else(|| {
                            PathwayError::Runtime(
                                "completed treatment has no validated disposition binding".into(),
                            )
                        })
                })
                .transpose()?;
            Ok(DispositionResult { pathway, decision })
        })
        .collect()
}

fn invalid<T>(message: &str) -> Result<T, PathwayError> {
    Err(PathwayError::InvalidInput(message.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arrivals::{ArrivalBin, ArrivalConfig, ArrivalModeCounts};
    use crate::pathway::{
        bind_arrivals, PathwayPatient, ServiceLaw, ServiceOutcome, ServiceSpec, StageKind,
        StageTemplate, StageWork,
    };
    use crate::{parse_scenario_json, CapacityLocationProfile, Provenance, ScenarioConfig};

    fn profile() -> CapacityLocationProfile {
        let scenario: ScenarioConfig =
            parse_scenario_json(include_str!("../examples/one_patient.json")).unwrap();
        let mut profile = scenario.capacity_location;
        profile.staff_roles[0].counts[0].reference = "synthetic.fixture".into();
        for id in [
            "resource.assessment",
            "resource.diagnostics",
            "resource.treatment",
        ] {
            let mut resource = profile.resource_buckets[0].clone();
            resource.resource_id = id.into();
            profile.resource_buckets.push(resource);
        }
        profile
    }

    fn stage(kind: StageKind, resource_id: &str, duration_ticks: u64) -> StageWork {
        StageWork {
            kind,
            resource_id: resource_id.into(),
            task_class_id: "task.synthetic".into(),
            duration_ticks,
        }
    }

    fn patient(id: &str, arrival_tick: u64, diagnostics: usize) -> PathwayPatient {
        let mut stages = vec![
            stage(StageKind::Triage, "resource.synthetic", 1),
            stage(StageKind::Assessment, "resource.assessment", 1),
        ];
        stages.extend(
            (0..diagnostics).map(|_| stage(StageKind::Diagnostics, "resource.diagnostics", 2)),
        );
        stages.push(stage(StageKind::Treatment, "resource.treatment", 2));
        PathwayPatient {
            patient_id: id.into(),
            arrival_tick,
            mode: "alpha".into(),
            acuity: 3,
            stages,
        }
    }

    fn config(patients: Vec<PathwayPatient>, horizon_ticks: u64) -> PathwayConfig {
        PathwayConfig {
            version: 1,
            horizon_ticks,
            provenance: Provenance {
                class: "synthetic".into(),
                notes: "synthetic disposition fixture".into(),
            },
            capacity_location: profile(),
            patients,
        }
    }

    fn service(law: ServiceLaw) -> ServiceSpec {
        ServiceSpec {
            version: 1,
            provenance: ServiceProvenance {
                class: "synthetic".into(),
                notes: "synthetic service fixture".into(),
            },
            law,
        }
    }

    fn fixed_bindings(patients: &[PathwayPatient]) -> Vec<ServiceBinding> {
        patients
            .iter()
            .flat_map(|patient| {
                patient
                    .stages
                    .iter()
                    .enumerate()
                    .map(|(stage_index, work)| ServiceBinding {
                        patient_id: patient.patient_id.clone(),
                        stage_index,
                        service: service(ServiceLaw::Fixed {
                            duration_ticks: work.duration_ticks,
                        }),
                    })
            })
            .collect()
    }

    fn policy() -> DecisionPolicy {
        DecisionPolicy {
            version: 1,
            provenance: ServiceProvenance {
                class: "synthetic".into(),
                notes: "synthetic provisional decision policy".into(),
            },
            reveal_at: DecisionReveal::TreatmentCompletion,
        }
    }

    fn decision(patient_id: &str, latent_intent: DispositionIntent) -> DispositionBinding {
        DispositionBinding {
            patient_id: patient_id.into(),
            latent_intent,
        }
    }

    #[test]
    fn treatment_completion_reveals_only_at_exact_tick_and_never_changes_pathway() {
        let cfg = config(vec![patient("p", 1, 0)], 5);
        let bindings = fixed_bindings(&cfg.patients);
        let discharge = run_with_disposition_and_services(
            &cfg,
            0,
            &bindings,
            &policy(),
            &[decision("p", DispositionIntent::Discharge)],
        )
        .unwrap();
        let admission = run_with_disposition_and_services(
            &cfg,
            0,
            &bindings,
            &policy(),
            &[decision("p", DispositionIntent::Admission)],
        )
        .unwrap();
        assert_eq!(discharge[0].pathway, admission[0].pathway);
        assert_eq!(discharge[0].pathway.episodes[2].completion_tick, Some(5));
        assert_eq!(discharge[0].decision.as_ref().unwrap().observed_tick, 5);
        assert_eq!(
            discharge[0].decision.as_ref().unwrap().provisional_intent,
            DispositionIntent::Discharge
        );
        assert_eq!(
            admission[0].decision.as_ref().unwrap().provisional_intent,
            DispositionIntent::Admission
        );
        assert_eq!(discharge[0].pathway.known_acuity, Some(3));

        let before_horizon = config(vec![patient("p", 1, 0)], 4);
        let before_bindings = fixed_bindings(&before_horizon.patients);
        let before_discharge = run_with_disposition_and_services(
            &before_horizon,
            0,
            &before_bindings,
            &policy(),
            &[decision("p", DispositionIntent::Discharge)],
        )
        .unwrap();
        let before_admission = run_with_disposition_and_services(
            &before_horizon,
            0,
            &before_bindings,
            &policy(),
            &[decision("p", DispositionIntent::Admission)],
        )
        .unwrap();
        assert_eq!(before_discharge, before_admission);
        let before = &before_discharge;
        assert_eq!(
            before[0].pathway.state,
            crate::pathway::PathwayState::Active(StageKind::Treatment)
        );
        assert!(before[0].decision.is_none());
        assert_eq!(before[0].pathway.known_acuity, Some(3));
    }

    #[test]
    fn no_one_and_repeated_diagnostics_use_actual_completion_times() {
        for (count, completion) in [(0, 5), (1, 7), (2, 9)] {
            let cfg = config(vec![patient("p", 1, count)], completion);
            let output = run_with_disposition(
                &cfg,
                &policy(),
                &[decision("p", DispositionIntent::Admission)],
            )
            .unwrap();
            let result = &output[0];
            assert_eq!(result.decision.as_ref().unwrap().observed_tick, completion);
            assert_eq!(
                result.pathway.episodes.last().unwrap().completion_tick,
                Some(completion)
            );
            assert_eq!(
                result
                    .pathway
                    .episodes
                    .iter()
                    .filter(|episode| episode.kind == StageKind::Diagnostics)
                    .count(),
                count
            );
            assert_eq!(
                result.pathway.state,
                crate::pathway::PathwayState::TreatmentComplete
            );
        }
    }

    #[test]
    fn sampled_service_duration_drives_provisional_observation_tick() {
        let mut cfg = config(vec![patient("p", 1, 1)], 30);
        cfg.patients[0].stages[2].duration_ticks = 0;
        let mut bindings = fixed_bindings(&cfg.patients);
        bindings[2].service = service(ServiceLaw::Discrete {
            outcomes: vec![
                ServiceOutcome {
                    duration_ticks: 9,
                    weight: 4,
                },
                ServiceOutcome {
                    duration_ticks: 2,
                    weight: 1,
                },
                ServiceOutcome {
                    duration_ticks: 5,
                    weight: 2,
                },
            ],
        });
        let output = run_with_disposition_and_services(
            &cfg,
            11,
            &bindings,
            &policy(),
            &[decision("p", DispositionIntent::Discharge)],
        )
        .unwrap();
        let diagnostic = &output[0].pathway.episodes[2];
        assert_eq!(diagnostic.sampled_duration_ticks, 9);
        assert_eq!(diagnostic.completion_tick, Some(12));
        assert_eq!(output[0].decision.as_ref().unwrap().observed_tick, 14);
        assert_eq!(
            output[0].decision.as_ref().unwrap().observed_tick,
            output[0]
                .pathway
                .episodes
                .last()
                .unwrap()
                .completion_tick
                .unwrap()
        );
    }

    #[test]
    fn patient_service_and_decision_binding_permutations_preserve_output() {
        let cfg = config(vec![patient("B", 1, 1), patient("A", 1, 0)], 30);
        let services = fixed_bindings(&cfg.patients);
        let decisions = vec![
            decision("B", DispositionIntent::Admission),
            decision("A", DispositionIntent::Discharge),
        ];
        let expected =
            run_with_disposition_and_services(&cfg, 31, &services, &policy(), &decisions).unwrap();
        let reversed_cfg = config(vec![patient("A", 1, 0), patient("B", 1, 1)], 30);
        let mut reversed_services = fixed_bindings(&reversed_cfg.patients);
        reversed_services.reverse();
        let mut reversed_decisions = decisions;
        reversed_decisions.reverse();
        let permuted = run_with_disposition_and_services(
            &reversed_cfg,
            31,
            &reversed_services,
            &policy(),
            &reversed_decisions,
        )
        .unwrap();
        assert_eq!(expected, permuted);
        assert_eq!(
            expected
                .iter()
                .map(|item| item.pathway.patient_id.as_str())
                .collect::<Vec<_>>(),
            vec!["A", "B"]
        );
    }

    #[test]
    fn empty_population_validates_policy_and_returns_empty() {
        let empty = config(Vec::new(), 10);
        assert!(run_with_disposition(&empty, &policy(), &[])
            .unwrap()
            .is_empty());
        let mut invalid = policy();
        invalid.version = 2;
        assert!(matches!(
            run_with_disposition(&empty, &invalid, &[]),
            Err(PathwayError::InvalidInput(_))
        ));
    }

    #[test]
    fn invalid_policy_bindings_and_non_full_pathway_reject() {
        let cfg = config(vec![patient("p", 1, 0)], 10);
        let mut invalid_policy = policy();
        invalid_policy.version = 2;
        assert!(matches!(
            run_with_disposition(
                &cfg,
                &invalid_policy,
                &[decision("p", DispositionIntent::Admission)]
            ),
            Err(PathwayError::InvalidInput(_))
        ));
        let mut invalid_policy = policy();
        invalid_policy.provenance.class = "empirical".into();
        assert!(matches!(
            run_with_disposition(
                &cfg,
                &invalid_policy,
                &[decision("p", DispositionIntent::Admission)]
            ),
            Err(PathwayError::InvalidInput(_))
        ));
        let mut invalid_policy = policy();
        invalid_policy.provenance.notes = " padded ".into();
        assert!(matches!(
            run_with_disposition(
                &cfg,
                &invalid_policy,
                &[decision("p", DispositionIntent::Admission)]
            ),
            Err(PathwayError::InvalidInput(_))
        ));
        let legacy = config(
            vec![PathwayPatient {
                patient_id: "p".into(),
                arrival_tick: 1,
                mode: "alpha".into(),
                acuity: 3,
                stages: vec![
                    stage(StageKind::Triage, "resource.synthetic", 1),
                    stage(StageKind::Assessment, "resource.assessment", 1),
                ],
            }],
            10,
        );
        assert!(matches!(
            run_with_disposition(
                &legacy,
                &policy(),
                &[decision("p", DispositionIntent::Discharge)]
            ),
            Err(PathwayError::InvalidInput(_))
        ));

        let cases: Vec<Vec<DispositionBinding>> = vec![
            vec![],
            vec![
                decision("p", DispositionIntent::Admission),
                decision("p", DispositionIntent::Discharge),
            ],
            vec![decision("missing", DispositionIntent::Admission)],
            vec![decision("bad:id", DispositionIntent::Admission)],
        ];
        for bindings in cases {
            assert!(matches!(
                run_with_disposition(&cfg, &policy(), &bindings),
                Err(PathwayError::InvalidInput(_))
            ));
        }
        let malformed_path = config(vec![patient("bad:id", 1, 0)], 10);
        assert!(matches!(
            run_with_disposition(
                &malformed_path,
                &policy(),
                &[decision("bad:id", DispositionIntent::Admission)]
            ),
            Err(PathwayError::InvalidInput(_))
        ));
        let two_patients = config(vec![patient("A", 1, 0), patient("B", 1, 0)], 10);
        assert!(matches!(
            run_with_disposition(
                &two_patients,
                &policy(),
                &[
                    decision("A", DispositionIntent::Admission),
                    decision("A", DispositionIntent::Discharge),
                ]
            ),
            Err(PathwayError::InvalidInput(message)) if message.contains("duplicate")
        ));
    }

    #[test]
    fn generator_template_service_and_decision_compose() {
        let generated = crate::arrivals::generate_arrivals(&ArrivalConfig {
            version: 1,
            seed: 19,
            horizon_ticks: 30,
            provenance: "synthetic_tick_fixture".into(),
            bins: vec![ArrivalBin {
                start_tick: 1,
                end_tick: 2,
                modes: vec![ArrivalModeCounts {
                    mode: "alpha".into(),
                    total: 1,
                    acuity_counts: [1, 0, 0, 0, 0],
                }],
            }],
        })
        .unwrap();
        let template = StageTemplate {
            mode: "alpha".into(),
            acuity: 1,
            stages: vec![
                stage(StageKind::Triage, "resource.synthetic", 1),
                stage(StageKind::Assessment, "resource.assessment", 1),
            ],
        };
        let mut bound = bind_arrivals(&generated, &[template]).unwrap();
        for item in &mut bound {
            item.stages
                .push(stage(StageKind::Diagnostics, "resource.diagnostics", 2));
            item.stages
                .push(stage(StageKind::Treatment, "resource.treatment", 2));
        }
        let mut cfg = config(bound, 30);
        cfg.provenance.notes = "generated synthetic pathway fixture".into();
        assert_eq!(generated.len(), 1);
        assert_eq!(generated[0].patient_id, "arrival-v1-b0000-m00-a1-n000000");
        assert_eq!(generated[0].arrival_tick, 1);
        let mut services = fixed_bindings(&cfg.patients);
        services[2].service = service(ServiceLaw::Discrete {
            outcomes: vec![
                ServiceOutcome {
                    duration_ticks: 9,
                    weight: 4,
                },
                ServiceOutcome {
                    duration_ticks: 2,
                    weight: 1,
                },
                ServiceOutcome {
                    duration_ticks: 5,
                    weight: 2,
                },
            ],
        });
        cfg.patients[0].stages[2].duration_ticks = 0;
        let patient_id = generated[0].patient_id.clone();
        let output = run_with_disposition_and_services(
            &cfg,
            11,
            &services,
            &policy(),
            &[decision(&patient_id, DispositionIntent::Admission)],
        )
        .unwrap();
        assert_eq!(output[0].pathway.patient_id, patient_id);
        assert_eq!(output[0].pathway.mode, "alpha");
        assert_eq!(output[0].pathway.acuity, 1);
        assert_eq!(output[0].pathway.known_acuity, Some(1));
        assert_eq!(
            output[0].decision.as_ref().unwrap().provisional_intent,
            DispositionIntent::Admission
        );
        assert_eq!(output[0].pathway.arrival_tick, 1);
        assert_eq!(output[0].pathway.episodes[0].start_tick, Some(1));
        assert_eq!(output[0].pathway.episodes[0].completion_tick, Some(2));
        assert_eq!(output[0].pathway.episodes[1].start_tick, Some(2));
        assert_eq!(output[0].pathway.episodes[1].completion_tick, Some(3));
        assert_eq!(output[0].pathway.episodes[2].sampled_duration_ticks, 9);
        assert_eq!(output[0].pathway.episodes[2].start_tick, Some(3));
        assert_eq!(output[0].pathway.episodes[2].completion_tick, Some(12));
        assert_eq!(output[0].pathway.episodes[3].start_tick, Some(12));
        assert_eq!(output[0].pathway.episodes[3].completion_tick, Some(14));
        assert_eq!(output[0].decision.as_ref().unwrap().policy_version, 1);
        assert_eq!(
            output[0].decision.as_ref().unwrap().provenance,
            policy().provenance
        );
        assert_eq!(output[0].decision.as_ref().unwrap().observed_tick, 14);
        assert_eq!(
            output[0].pathway.episodes[2].service.law,
            ServiceLaw::Discrete {
                outcomes: vec![
                    ServiceOutcome {
                        duration_ticks: 2,
                        weight: 1
                    },
                    ServiceOutcome {
                        duration_ticks: 5,
                        weight: 2
                    },
                    ServiceOutcome {
                        duration_ticks: 9,
                        weight: 4
                    },
                ]
            }
        );
    }
}
