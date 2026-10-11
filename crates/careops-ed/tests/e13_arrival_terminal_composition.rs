#[allow(dead_code)]
#[path = "support/e13.rs"]
mod e13;

use std::collections::BTreeMap;

use careops_ed::arrivals::{generate_arrivals, ArrivalBin, ArrivalConfig, ArrivalModeCounts};
use careops_ed::disposition::{DispositionBinding, DispositionIntent};
use careops_ed::pathway::{
    bind_arrivals, ServiceBinding, ServiceLaw, ServiceProvenance, ServiceSpec, StageKind,
    StageTemplate, StageWork,
};
use careops_ed::terminal::{run_terminal, TerminalConfig};
use careops_ed::{Eligibility, ResourceBucket, TaskClass};
use e13::{assert_conservation, assert_interval_accounting, build, BuildOptions};

fn golden_arrival_config() -> ArrivalConfig {
    ArrivalConfig {
        version: 1,
        seed: 11,
        horizon_ticks: 10,
        provenance: "synthetic_tick_fixture".into(),
        bins: vec![ArrivalBin {
            start_tick: 0,
            end_tick: 10,
            modes: vec![
                ArrivalModeCounts {
                    mode: "beta".into(),
                    total: 3,
                    acuity_counts: [0, 1, 0, 2, 0],
                },
                ArrivalModeCounts {
                    mode: "alpha".into(),
                    total: 2,
                    acuity_counts: [1, 0, 0, 0, 1],
                },
            ],
        }],
    }
}

fn two_stage_templates() -> Vec<StageTemplate> {
    ["alpha", "beta"]
        .into_iter()
        .flat_map(|mode| {
            (1..=5).map(move |acuity| StageTemplate {
                mode: mode.into(),
                acuity,
                stages: vec![
                    StageWork {
                        kind: StageKind::Triage,
                        resource_id: "res.triage".into(),
                        task_class_id: "task.triage".into(),
                        duration_ticks: 1,
                    },
                    StageWork {
                        kind: StageKind::Assessment,
                        resource_id: "res.assessment".into(),
                        task_class_id: "task.assessment".into(),
                        duration_ticks: 1,
                    },
                ],
            })
        })
        .collect()
}

fn terminal_from_arrivals(
    arrivals: &[careops_ed::arrivals::GeneratedArrival],
    templates: &[StageTemplate],
) -> TerminalConfig {
    let mut config = build(BuildOptions {
        horizon: 10,
        patients: vec![],
        offers: vec![],
        discharge_delay: 0,
        care: None,
        capacities: BTreeMap::from([
            ("res.triage".into(), 5),
            ("res.assessment".into(), 5),
            ("res.treatment".into(), 5),
            ("res.space".into(), 5),
            ("res.transfer".into(), 5),
        ]),
    });
    let mut diagnostic_resource: ResourceBucket = config
        .pathway
        .capacity_location
        .resource_buckets
        .iter()
        .find(|resource| resource.resource_id == "res.treatment")
        .expect("treatment resource exists")
        .clone();
    diagnostic_resource.resource_id = "res.diagnostics".into();
    diagnostic_resource.location_id = "location.diagnostics".into();
    diagnostic_resource.zone_id = "zone.diagnostics".into();
    config
        .pathway
        .capacity_location
        .resource_buckets
        .push(diagnostic_resource);
    config
        .pathway
        .capacity_location
        .zones
        .push(careops_ed::Zone {
            zone_id: "zone.diagnostics".into(),
            name: "synthetic diagnostics".into(),
        });
    config
        .pathway
        .capacity_location
        .locations
        .push(careops_ed::Location {
            location_id: "location.diagnostics".into(),
            name: "synthetic diagnostics".into(),
            zone_id: "zone.diagnostics".into(),
        });
    config
        .pathway
        .capacity_location
        .task_classes
        .push(TaskClass {
            task_class_id: "task.diagnostics".into(),
            name: "synthetic diagnostics".into(),
            required_resource_classes: vec!["nurse".into()],
        });
    config
        .pathway
        .capacity_location
        .eligibility
        .push(Eligibility {
            staff_role_id: "staff.synthetic".into(),
            task_class_id: "task.diagnostics".into(),
            zone_id: "zone.diagnostics".into(),
            location_id: None,
        });

    let mut patients = bind_arrivals(arrivals, templates).expect("generated arrivals bind");
    for patient in &mut patients {
        patient.stages.extend([
            StageWork {
                kind: StageKind::Diagnostics,
                resource_id: "res.diagnostics".into(),
                task_class_id: "task.diagnostics".into(),
                duration_ticks: 1,
            },
            StageWork {
                kind: StageKind::Treatment,
                resource_id: "res.treatment".into(),
                task_class_id: "task.treatment".into(),
                duration_ticks: 1,
            },
        ]);
        config.decisions.push(DispositionBinding {
            patient_id: patient.patient_id.clone(),
            latent_intent: DispositionIntent::Discharge,
        });
        for stage_index in 0..patient.stages.len() {
            config.services.push(ServiceBinding {
                patient_id: patient.patient_id.clone(),
                stage_index,
                service: ServiceSpec {
                    version: 1,
                    provenance: ServiceProvenance {
                        class: "synthetic".into(),
                        notes: "one tick per generated pathway stage".into(),
                    },
                    law: ServiceLaw::Fixed { duration_ticks: 1 },
                },
            });
        }
    }
    config.pathway.patients = patients;
    config
}

#[test]
fn seed11_arrivals_compose_through_four_stages_and_censor_at_horizon() {
    let arrival_config = golden_arrival_config();
    let arrivals = generate_arrivals(&arrival_config).expect("seeded arrivals generate");
    assert_eq!(
        arrivals,
        vec![
            careops_ed::arrivals::GeneratedArrival {
                patient_id: "arrival-v1-b0000-m00-a5-n000000".into(),
                arrival_tick: 5,
                mode: "alpha".into(),
                acuity: 5,
            },
            careops_ed::arrivals::GeneratedArrival {
                patient_id: "arrival-v1-b0000-m01-a4-n000000".into(),
                arrival_tick: 5,
                mode: "beta".into(),
                acuity: 4,
            },
            careops_ed::arrivals::GeneratedArrival {
                patient_id: "arrival-v1-b0000-m01-a4-n000001".into(),
                arrival_tick: 6,
                mode: "beta".into(),
                acuity: 4,
            },
            careops_ed::arrivals::GeneratedArrival {
                patient_id: "arrival-v1-b0000-m00-a1-n000000".into(),
                arrival_tick: 9,
                mode: "alpha".into(),
                acuity: 1,
            },
            careops_ed::arrivals::GeneratedArrival {
                patient_id: "arrival-v1-b0000-m01-a2-n000000".into(),
                arrival_tick: 9,
                mode: "beta".into(),
                acuity: 2,
            },
        ]
    );
    assert_eq!(generate_arrivals(&arrival_config).unwrap(), arrivals);
    let mut permuted_arrival_config = golden_arrival_config();
    permuted_arrival_config.bins[0].modes.reverse();
    assert_eq!(
        generate_arrivals(&permuted_arrival_config).unwrap(),
        arrivals
    );

    let templates = two_stage_templates();
    let config = terminal_from_arrivals(&arrivals, &templates);
    for resource_id in [
        "res.triage",
        "res.assessment",
        "res.diagnostics",
        "res.treatment",
        "res.space",
        "res.transfer",
    ] {
        let resource = config
            .pathway
            .capacity_location
            .resource_buckets
            .iter()
            .find(|resource| resource.resource_id == resource_id)
            .expect("composition resource exists");
        assert_eq!(resource.physical_count, Some(5), "{resource_id} capacity");
        assert_eq!(resource.open_count, Some(5), "{resource_id} open capacity");
        assert_eq!(
            resource.staffed_count,
            Some(5),
            "{resource_id} staffed capacity"
        );
    }
    let result = run_terminal(&config).expect("complete generated cohort runs through terminal");
    println!(
        "E13_ARRIVAL_TERMINAL_COMPOSITION {}",
        serde_json::to_string(&serde_json::json!({
            "case_id": "seed11-arrivals-four-stage-capacity5-horizon10",
            "result": &result
        }))
        .unwrap()
    );

    assert_eq!(result.counts.arrived, 5);
    assert_eq!(result.counts.discharged, 3);
    assert_eq!(result.counts.transferred, 0);
    assert_eq!(result.counts.abandoned, 0);
    assert_eq!(result.counts.unfinished, 2);
    assert_conservation(&result);
    assert_interval_accounting(&config, &result, 10);
    assert_eq!(result, run_terminal(&config).unwrap());

    let expected = [
        ("arrival-v1-b0000-m00-a5-n000000", 5, "alpha", 5, Some(9)),
        ("arrival-v1-b0000-m01-a4-n000000", 5, "beta", 4, Some(9)),
        ("arrival-v1-b0000-m01-a4-n000001", 6, "beta", 4, Some(10)),
        ("arrival-v1-b0000-m00-a1-n000000", 9, "alpha", 1, None),
        ("arrival-v1-b0000-m01-a2-n000000", 9, "beta", 2, None),
    ];
    for (id, arrival, mode, acuity, terminal_tick) in expected {
        let patient = e13::patient(&result, id);
        assert_eq!(patient.pathway.patient_id, id);
        assert_eq!(patient.pathway.arrival_tick, arrival, "{id} arrival");
        assert_eq!(patient.pathway.mode, mode, "{id} mode");
        assert_eq!(patient.pathway.acuity, acuity, "{id} acuity");
        assert_eq!(
            patient.pathway.known_acuity,
            Some(acuity),
            "{id} known acuity"
        );
        assert_eq!(patient.terminal_tick, terminal_tick, "{id} terminal tick");
        assert_eq!(
            patient.space.acquired_tick,
            Some(arrival),
            "{id} space acquired"
        );
        assert_eq!(patient.pathway.episodes[0].kind, StageKind::Triage);
        assert_eq!(patient.pathway.episodes[0].start_tick, Some(arrival));
        assert_eq!(
            patient.pathway.episodes[0].completion_tick,
            Some(arrival + 1)
        );
        assert_eq!(patient.pathway.episodes[0].observed_work_ticks, 1);
        if let Some(terminal_tick) = terminal_tick {
            assert_eq!(
                patient.decision.as_ref().unwrap().observed_tick,
                terminal_tick
            );
            assert_eq!(patient.space.released_tick, Some(terminal_tick));
            assert_eq!(
                patient.space.observed_occupancy_ticks,
                terminal_tick - arrival
            );
        } else {
            assert_eq!(patient.decision, None);
            assert_eq!(patient.space.released_tick, None);
            assert_eq!(patient.space.observed_occupancy_ticks, 1);
        }
    }
    for id in [
        "arrival-v1-b0000-m00-a5-n000000",
        "arrival-v1-b0000-m01-a4-n000000",
    ] {
        let patient = e13::patient(&result, id);
        assert_eq!(patient.completed_los_ticks, Some(4));
        assert_eq!(patient.pathway.episodes.len(), 4);
        for (index, episode) in patient.pathway.episodes.iter().enumerate() {
            assert_eq!(episode.start_tick, Some(5 + index as u64));
            assert_eq!(episode.completion_tick, Some(6 + index as u64));
            assert_eq!(episode.observed_work_ticks, 1);
        }
    }
    let at_six = e13::patient(&result, "arrival-v1-b0000-m01-a4-n000001");
    assert_eq!(at_six.completed_los_ticks, Some(4));
    assert_eq!(at_six.pathway.episodes.len(), 4);
    for (index, episode) in at_six.pathway.episodes.iter().enumerate() {
        assert_eq!(episode.start_tick, Some(6 + index as u64));
        assert_eq!(episode.completion_tick, Some(7 + index as u64));
        assert_eq!(episode.observed_work_ticks, 1);
    }
    for id in [
        "arrival-v1-b0000-m00-a1-n000000",
        "arrival-v1-b0000-m01-a2-n000000",
    ] {
        let patient = e13::patient(&result, id);
        assert_eq!(patient.pathway.episodes.len(), 2);
        assert_eq!(patient.pathway.episodes[1].kind, StageKind::Assessment);
        assert_eq!(patient.pathway.episodes[1].start_tick, Some(10));
        assert_eq!(patient.pathway.episodes[1].completion_tick, None);
        assert_eq!(patient.pathway.episodes[1].observed_work_ticks, 0);
        assert_eq!(patient.decision, None);
        assert_eq!(patient.realized_outcome, None);
        assert_eq!(patient.terminal_tick, None);
        assert_eq!(patient.completed_los_ticks, None);
        assert_eq!(
            patient.unfinished_reason,
            Some(careops_ed::terminal::UnfinishedReason::ClinicalActive {
                stage: StageKind::Assessment
            })
        );
    }

    let mut permuted_arrivals = arrivals.clone();
    permuted_arrivals.reverse();
    let mut permuted_templates = two_stage_templates();
    permuted_templates.reverse();
    let mut permuted_config = terminal_from_arrivals(&permuted_arrivals, &permuted_templates);
    permuted_config.pathway.patients.reverse();
    permuted_config.services.reverse();
    permuted_config.decisions.reverse();
    permuted_config.admissions.reverse();
    permuted_config.offers.reverse();
    permuted_config
        .pathway
        .capacity_location
        .resource_buckets
        .reverse();
    assert_eq!(result, run_terminal(&permuted_config).unwrap());
}
