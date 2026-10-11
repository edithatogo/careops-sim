#[path = "support/e13.rs"]
pub mod e13;

use std::collections::{BTreeMap, BTreeSet};

use careops_ed::disposition::{
    DecisionPolicy, DecisionReveal, DispositionBinding, DispositionIntent,
};
use careops_ed::offers::{BedOffer, OfferPolicy, OfferStatus};
use careops_ed::pathway::{
    PathwayConfig, PathwayError, PathwayPatient, ServiceBinding, ServiceLaw, ServiceProvenance,
    ServiceSpec, StageKind, StageWork,
};
use careops_ed::terminal::{
    run_terminal, AbandonmentPolicy, AdmissionDestination, EndpointState, RealizedOutcome,
    TerminalConfig, TerminalPolicy, UnfinishedReason,
};
use careops_ed::{
    AggregateCapacity, CapacityLocationProfile, Eligibility, KnowledgeStatus, Location,
    PatientStatus, PatientWorkItem, Provenance, ResourceBucket, ScenarioConfig, StaffCount,
    StaffRole, TaskClass, Zone,
};
use serde_json::Value;

const ENDPOINTS: &str = include_str!("../../../conductor/design/ed/e1-endpoints.json");
const EXTREMES: &str = include_str!("../../../conductor/design/ed/e1-extremes.json");
const INVALID: &str = include_str!("../../../conductor/design/ed/e1-invalid.json");
fn cases(source: &str, family: &str) -> Vec<Value> {
    let document: Value = serde_json::from_str(source).expect("frozen fixture JSON parses");
    assert_eq!(document["family"], family);
    assert_eq!(document["runtime_status"], "unbound");
    document["cases"].as_array().expect("cases array").clone()
}

fn exact_ids(actual: &BTreeSet<String>, expected: &[&str], family: &str) {
    assert_eq!(actual.len(), expected.len(), "{family} case count");
    let expected = expected.iter().map(|value| (*value).to_owned()).collect();
    assert_eq!(*actual, expected, "{family} exact ID set");
}

fn kind(name: &str) -> StageKind {
    match name {
        "triage" => StageKind::Triage,
        "assessment" => StageKind::Assessment,
        "diagnostics" => StageKind::Diagnostics,
        "treatment" => StageKind::Treatment,
        _ => panic!("unknown frozen stage {name}"),
    }
}

fn intent(value: &str) -> DispositionIntent {
    match value {
        "discharge" => DispositionIntent::Discharge,
        "admission" => DispositionIntent::Admission,
        _ => panic!("unknown frozen intent {value}"),
    }
}

fn profile(triage: (KnowledgeStatus, Option<u64>, u64, u64, &str)) -> CapacityLocationProfile {
    let mut rows = vec![
        (
            "resource.triage",
            "nurse",
            "task.triage",
            "zone.triage",
            5,
            5,
            5,
        ),
        (
            "resource.assessment",
            "nurse",
            "task.assessment",
            "zone.assessment",
            5,
            5,
            5,
        ),
        (
            "resource.diagnostics",
            "nurse",
            "task.diagnostics",
            "zone.diagnostics",
            5,
            5,
            5,
        ),
        (
            "resource.treatment",
            "nurse",
            "task.treatment",
            "zone.treatment",
            5,
            5,
            5,
        ),
        (
            "resource.edspace",
            "space",
            "task.edspace",
            "zone.edspace",
            5,
            5,
            5,
        ),
        (
            "resource.transfer",
            "transfer",
            "task.transfer",
            "zone.transfer",
            5,
            5,
            5,
        ),
    ];
    rows[0] = (
        "resource.triage",
        triage.4,
        "task.triage",
        "zone.triage",
        triage.2,
        triage.3,
        triage.3,
    );
    let mut zones = BTreeMap::new();
    let mut locations = BTreeMap::new();
    let mut resources = BTreeMap::new();
    let mut tasks = BTreeMap::new();
    let mut eligibility = BTreeSet::new();
    for (resource_id, class, task_id, zone_id, physical, open, staffed) in rows {
        let resource_id = resource_id.to_owned();
        let class = class.to_owned();
        let task_id = task_id.to_owned();
        let zone_id = zone_id.to_owned();
        let location_id = format!("location.{}", zone_id.trim_start_matches("zone."));
        zones.entry(zone_id.clone()).or_insert_with(|| Zone {
            zone_id: zone_id.clone(),
            name: zone_id.clone(),
        });
        locations
            .entry(location_id.clone())
            .or_insert_with(|| Location {
                location_id: location_id.clone(),
                name: location_id.clone(),
                zone_id: zone_id.clone(),
            });
        resources.insert(
            resource_id.clone(),
            ResourceBucket {
                resource_id,
                resource_class: class.clone(),
                location_id,
                zone_id: zone_id.clone(),
                pool_group_id: None,
                physical_status: KnowledgeStatus::Known,
                physical_count: Some(physical),
                availability_status: KnowledgeStatus::Known,
                open_count: Some(open),
                staffed_count: Some(staffed),
                availability_reason: None,
                capabilities: vec![],
            },
        );
        tasks.entry(task_id.clone()).or_insert_with(|| TaskClass {
            task_class_id: task_id.clone(),
            name: task_id.clone(),
            required_resource_classes: vec![class],
        });
        eligibility.insert((task_id, zone_id));
    }
    let mut triage_resource = resources.get("resource.triage").unwrap().clone();
    triage_resource.physical_status = triage.0;
    triage_resource.physical_count = triage.1;
    resources.insert("resource.triage".into(), triage_resource);
    CapacityLocationProfile {
        schema_version: 3,
        status: "proposed".into(),
        capacity: AggregateCapacity {
            unit: "treatment_space".into(),
            bucket_completeness: "complete".into(),
            physical_status: KnowledgeStatus::Known,
            physical_count: Some(30),
            availability_status: KnowledgeStatus::Known,
            open_count: Some(30),
            staffed_count: Some(30),
            availability_reason: None,
            equipment: vec![],
        },
        zones: zones.into_values().collect(),
        locations: locations.into_values().collect(),
        resource_buckets: resources.into_values().collect(),
        task_classes: tasks.into_values().collect(),
        staff_roles: vec![StaffRole {
            staff_role_id: "staff.synthetic".into(),
            name: "synthetic".into(),
            counts: vec![StaffCount {
                basis: "present".into(),
                status: KnowledgeStatus::Known,
                count: Some(30),
                reference: "synthetic.fixture".into(),
                reason: None,
            }],
        }],
        eligibility: eligibility
            .into_iter()
            .map(|(task_class_id, zone_id)| Eligibility {
                staff_role_id: "staff.synthetic".into(),
                task_class_id,
                zone_id,
                location_id: None,
            })
            .collect(),
        routes: vec![],
    }
}

fn empty_terminal(horizon: u64) -> TerminalConfig {
    TerminalConfig {
        version: 1,
        pathway: PathwayConfig {
            version: 1,
            horizon_ticks: horizon,
            provenance: Provenance {
                class: "synthetic".into(),
                notes: "frozen terminal fixture".into(),
            },
            capacity_location: profile((KnowledgeStatus::Known, Some(5), 5, 5, "nurse")),
            patients: vec![],
        },
        seed: 0,
        services: vec![],
        decision_policy: DecisionPolicy {
            version: 1,
            provenance: ServiceProvenance {
                class: "synthetic".into(),
                notes: "fixture decision policy".into(),
            },
            reveal_at: DecisionReveal::TreatmentCompletion,
        },
        decisions: vec![],
        admissions: vec![],
        offer_policy: OfferPolicy {
            version: 1,
            provenance: ServiceProvenance {
                class: "synthetic".into(),
                notes: "fixture offer policy".into(),
            },
        },
        offers: vec![],
        policy: TerminalPolicy {
            version: 1,
            provenance: ServiceProvenance {
                class: "synthetic".into(),
                notes: "fixture terminal policy".into(),
            },
            ed_space_resource_id: "resource.edspace".into(),
            transfer_resource_id: "resource.transfer".into(),
            transfer_task_class_id: "task.transfer".into(),
            discharge_delay_ticks: 0,
            abandonment: AbandonmentPolicy::Disabled,
            boarder_care: None,
        },
    }
}

fn fixed_service(duration_ticks: u64) -> ServiceSpec {
    ServiceSpec {
        version: 1,
        provenance: ServiceProvenance {
            class: "synthetic".into(),
            notes: "fixed fixture service".into(),
        },
        law: ServiceLaw::Fixed { duration_ticks },
    }
}

fn path_patient(id: &str, arrival: u64, acuity: u8, stages: Vec<StageWork>) -> PathwayPatient {
    PathwayPatient {
        patient_id: id.into(),
        arrival_tick: arrival,
        mode: "synthetic".into(),
        acuity,
        stages,
    }
}

fn append_fixture_patient(config: &mut TerminalConfig, row: &Value, include_diagnostics: bool) {
    let id = row["patient_id"].as_str().unwrap();
    let arrival = row["arrival_tick"].as_u64().unwrap();
    let acuity = row["latent_state"]["acuity"].as_u64().unwrap() as u8;
    let mut stage_rows = Vec::new();
    for stage in row["stages"].as_array().unwrap() {
        let name = stage["stage"].as_str().unwrap();
        if name == "diagnostics" && !include_diagnostics {
            continue;
        }
        let resource_id = stage["resource_id"].as_str().unwrap().to_owned();
        let task_class_id = format!("task.{}", name);
        stage_rows.push(StageWork {
            kind: kind(name),
            resource_id,
            task_class_id,
            duration_ticks: stage["duration_ticks"].as_u64().unwrap(),
        });
    }
    let patient = path_patient(id, arrival, acuity, stage_rows.clone());
    config
        .services
        .extend(
            stage_rows
                .iter()
                .enumerate()
                .map(|(stage_index, stage)| ServiceBinding {
                    patient_id: id.into(),
                    stage_index,
                    service: fixed_service(stage.duration_ticks),
                }),
        );
    config.pathway.patients.push(patient);
    let disposition = intent(row["provisional_disposition"].as_str().unwrap());
    config.decisions.push(DispositionBinding {
        patient_id: id.into(),
        latent_intent: disposition,
    });
    if disposition == DispositionIntent::Admission {
        config.admissions.push(AdmissionDestination {
            patient_id: id.into(),
            destination: "destination.synthetic".into(),
        });
    }
}

fn endpoint_config(case: &Value) -> TerminalConfig {
    let input = &case["inputs"];
    let mut config = empty_terminal(input["horizon_tick"].as_u64().unwrap());
    for row in input["patients"].as_array().unwrap() {
        append_fixture_patient(&mut config, row, true);
    }
    config.offers = input["bed_offers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|offer| BedOffer {
            offer_id: offer["offer_id"].as_str().unwrap().into(),
            destination: offer["destination"].as_str().unwrap().into(),
            patient_id: offer
                .get("patient_id")
                .and_then(Value::as_str)
                .map(str::to_owned),
            available_tick: offer["available_tick"].as_u64().unwrap(),
            withdrawn_tick: offer["withdrawn_tick"].as_u64().unwrap(),
            transfer_duration_ticks: offer["transfer_duration_ticks"].as_u64().unwrap(),
        })
        .collect();
    config
}

fn assert_endpoint(case: &Value) {
    let id = case["case_id"].as_str().unwrap();
    let config = endpoint_config(case);
    let result = run_terminal(&config).unwrap_or_else(|e| panic!("{id} terminal run: {e}"));
    e13::assert_conservation(&result);
    e13::assert_interval_accounting(&config, &result, config.pathway.horizon_ticks);
    let expected = &case["expected"];
    assert_eq!(
        result.counts.arrived,
        expected["counts"]["arrivals"].as_u64().unwrap(),
        "{id} arrivals"
    );
    assert_eq!(
        result.counts.discharged + result.counts.transferred,
        expected["counts"]["terminal"].as_u64().unwrap(),
        "{id} terminal count"
    );
    assert_eq!(
        result.counts.discharged + result.counts.transferred,
        expected["counts"]["throughput_count"].as_u64().unwrap(),
        "{id} throughput count"
    );
    assert_eq!(
        result.counts.unfinished,
        expected["counts"]["unfinished"].as_u64().unwrap(),
        "{id} unfinished"
    );
    assert_eq!(result.counts.abandoned, 0, "{id} Disabled abandonment");
    let started = result
        .patients
        .iter()
        .filter(|p| {
            p.pathway
                .episodes
                .first()
                .and_then(|e| e.start_tick)
                .is_some()
        })
        .count();
    assert_eq!(
        started as u64,
        expected["counts"]["started"].as_u64().unwrap(),
        "{id} started"
    );
    for row in expected["patients"].as_array().unwrap() {
        let patient_id = row["patient_id"].as_str().unwrap();
        let actual = result
            .patients
            .iter()
            .find(|p| p.pathway.patient_id == patient_id)
            .unwrap_or_else(|| panic!("{id} missing patient {patient_id}"));
        assert_eq!(
            actual.pathway.acuity,
            row["latent_state"]["acuity"].as_u64().unwrap() as u8,
            "{id} latent acuity"
        );
        assert_eq!(
            actual.pathway.known_acuity,
            row["agent_knowledge"]["triage_acuity"]
                .as_u64()
                .map(|v| v as u8),
            "{id} known acuity"
        );
        assert_eq!(
            actual.pathway.arrival_tick,
            row["arrival_tick"].as_u64().unwrap(),
            "{id} arrival"
        );
        assert_eq!(
            actual.pathway.episodes.len(),
            row["stage_intervals"].as_array().unwrap().len(),
            "{id} episode count"
        );
        for (stage_index, (episode, expected_stage)) in actual
            .pathway
            .episodes
            .iter()
            .zip(row["stage_intervals"].as_array().unwrap())
            .enumerate()
        {
            assert_eq!(
                config
                    .pathway
                    .patients
                    .iter()
                    .find(|p| p.patient_id == patient_id)
                    .unwrap()
                    .stages[stage_index]
                    .resource_id,
                expected_stage["resource_id"].as_str().unwrap(),
                "{id} frozen stage resource binding"
            );
            assert_eq!(
                format!("{:?}", episode.kind).to_lowercase(),
                expected_stage["stage"].as_str().unwrap(),
                "{id} stage"
            );
            assert_eq!(episode.service.provenance.class, "synthetic");
            assert_eq!(
                episode.start_tick,
                expected_stage["start_tick"].as_u64(),
                "{id} stage start"
            );
            assert_eq!(
                episode.completion_tick,
                expected_stage["end_tick"].as_u64(),
                "{id} stage completion"
            );
            assert_eq!(
                episode.observed_work_ticks,
                expected_stage["end_tick"].as_u64().unwrap()
                    - expected_stage["start_tick"].as_u64().unwrap(),
                "{id} stage work"
            );
            assert_eq!(
                episode.task_id,
                if episode.kind == StageKind::Diagnostics {
                    format!("{}:diagnostics:{}", patient_id, episode.episode_index)
                } else {
                    format!(
                        "{}:{}",
                        patient_id,
                        expected_stage["stage"].as_str().unwrap()
                    )
                }
            );
            assert_eq!(
                episode.kind,
                kind(expected_stage["stage"].as_str().unwrap())
            );
            assert_eq!(
                episode.service.law,
                ServiceLaw::Fixed {
                    duration_ticks: episode.sampled_duration_ticks
                }
            );
            assert_eq!(episode.sampled_duration_ticks, episode.observed_work_ticks);
        }
        assert_eq!(
            actual.decision.as_ref().map(|d| d.observed_tick),
            row["stage_intervals"]
                .as_array()
                .unwrap()
                .last()
                .and_then(|v| v["end_tick"].as_u64()),
            "{id} decision reveal at treatment completion"
        );
        assert_eq!(
            actual
                .decision
                .as_ref()
                .map(|d| format!("{:?}", d.provisional_intent).to_lowercase()),
            Some(row["provisional_disposition"].as_str().unwrap().into()),
            "{id} decision"
        );
        assert_eq!(
            actual.endpoint_state,
            match row["endpoint_state"].as_str().unwrap() {
                "terminal" => EndpointState::Terminal,
                "unfinished_at_horizon" => EndpointState::Boarding,
                other => panic!("unmapped endpoint state {other}"),
            }
        );
        match row["unfinished_reason"].as_str() {
            Some("boarding_without_compatible_offer") => assert_eq!(
                actual.unfinished_reason,
                Some(UnfinishedReason::AwaitingOfferAllocation),
                "{id} frozen unfinished classification"
            ),
            None => assert!(actual.unfinished_reason.is_none(), "{id} terminal reason"),
            Some(other) => panic!("unmapped frozen unfinished classification {other}"),
        }
        assert_eq!(
            actual.terminal_tick,
            row["terminal_tick"].as_u64(),
            "{id} terminal tick"
        );
        assert_eq!(
            actual.completed_los_ticks,
            row["completed_los_ticks"].as_u64(),
            "{id} completed LOS"
        );
        assert_eq!(
            actual.observed_elapsed_ticks,
            row["observed_elapsed_ticks"].as_u64().unwrap(),
            "{id} observed elapsed"
        );
        let active_work: u64 = actual
            .pathway
            .episodes
            .iter()
            .map(|e| e.observed_work_ticks)
            .sum();
        assert_eq!(
            active_work,
            row["active_work_ticks"].as_u64().unwrap(),
            "{id} active work"
        );
        let first_wait = actual
            .pathway
            .episodes
            .first()
            .map(|e| e.start_tick.unwrap() - actual.pathway.arrival_tick)
            .unwrap_or(0);
        assert_eq!(
            first_wait,
            row["first_care_wait_ticks"].as_u64().unwrap(),
            "{id} first-care wait"
        );
        let stage_wait: u64 = actual
            .pathway
            .episodes
            .iter()
            .map(|e| e.start_tick.unwrap() - e.queue_entry_tick)
            .sum();
        assert_eq!(
            stage_wait,
            row["total_stage_queue_wait_ticks"].as_u64().unwrap(),
            "{id} stage queue wait"
        );
        if row["provisional_disposition"] == "admission" {
            if let Some(interval) = row["boarding_interval"].as_object() {
                let boarding = actual.boarding.as_ref().unwrap();
                assert_eq!(
                    boarding.start_tick,
                    interval["start_tick"].as_u64().unwrap()
                );
                assert_eq!(
                    boarding.observed_ticks,
                    interval["duration_ticks"].as_u64().unwrap()
                );
                assert_eq!(
                    boarding.end_tick,
                    if actual.allocation.is_some() {
                        interval["end_tick"].as_u64()
                    } else {
                        None
                    }
                );
            } else {
                assert!(actual.boarding.as_ref().unwrap().end_tick.is_none());
            }
            if let Some(interval) = row["transfer_interval"].as_object() {
                let transfer = actual.transfer.as_ref().unwrap();
                assert_eq!(transfer.start_tick, interval["start_tick"].as_u64());
                assert_eq!(transfer.completion_tick, interval["end_tick"].as_u64());
                assert_eq!(
                    transfer.required_work_ticks,
                    interval["duration_ticks"].as_u64().unwrap()
                );
                assert_eq!(
                    actual.allocation.as_ref().unwrap().allocated_tick,
                    interval["start_tick"].as_u64().unwrap()
                );
                assert_eq!(
                    actual.boarding_time_ticks,
                    Some(
                        interval["end_tick"].as_u64().unwrap()
                            - row["boarding_interval"]["start_tick"].as_u64().unwrap()
                    )
                );
                assert_eq!(
                    actual.allocation.as_ref().unwrap().offer_id,
                    interval["offer_id"].as_str().unwrap()
                );
                let pending = actual.pending_transfer.as_ref().unwrap();
                assert_eq!(pending.start_tick, interval["start_tick"].as_u64().unwrap());
                assert_eq!(
                    pending.end_tick,
                    Some(interval["start_tick"].as_u64().unwrap())
                );
                assert_eq!(pending.observed_ticks, 0);
            } else {
                assert!(actual.transfer.is_none());
                assert!(actual.pending_transfer.is_none());
                assert!(actual.allocation.is_none());
                assert!(actual.realized_outcome.is_none());
            }
        }
        match row["realized_outcome"]["kind"].as_str() {
            Some("discharged") => assert_eq!(
                actual.realized_outcome,
                Some(RealizedOutcome::Discharged),
                "{id} discharge"
            ),
            Some("transferred") => {
                assert!(
                    matches!(
                        actual.realized_outcome,
                        Some(RealizedOutcome::Transferred { .. })
                    ),
                    "{id} transfer"
                );
                if let Some(RealizedOutcome::Transferred {
                    destination,
                    offer_id,
                }) = &actual.realized_outcome
                {
                    assert_eq!(
                        destination,
                        row["realized_outcome"]["destination"].as_str().unwrap()
                    );
                    assert_eq!(
                        offer_id,
                        row["realized_outcome"]["offer_id"].as_str().unwrap()
                    );
                }
            }
            None => assert!(actual.realized_outcome.is_none(), "{id} unfinished outcome"),
            Some(other) => panic!("unmapped expected outcome {other}"),
        }
        assert_eq!(
            actual.space.acquired_tick,
            Some(actual.pathway.arrival_tick)
        );
        assert_eq!(actual.space.resource_id, "resource.edspace");
        assert_eq!(actual.space.released_tick, actual.terminal_tick);
        assert_eq!(
            actual.space.observed_occupancy_ticks,
            actual.observed_elapsed_ticks
        );
    }
    assert_eq!(
        result.offers.len(),
        config.offers.len(),
        "{id} offer row count"
    );
    for (actual_offer, configured_offer) in result.offers.iter().zip(&config.offers) {
        assert_eq!(
            actual_offer.offer, *configured_offer,
            "{id} actual offer echo"
        );
        let transferred = result.patients.iter().find(|p| {
            p.allocation
                .as_ref()
                .is_some_and(|a| a.offer_id == configured_offer.offer_id)
        });
        if let Some(patient) = transferred {
            assert_eq!(
                actual_offer.status,
                OfferStatus::Consumed,
                "{id} consumed offer"
            );
            assert_eq!(
                actual_offer.allocation.as_ref().unwrap().patient_id,
                patient.pathway.patient_id
            );
            assert_eq!(
                actual_offer.allocation.as_ref().unwrap(),
                patient.allocation.as_ref().unwrap()
            );
        } else {
            assert_ne!(
                actual_offer.status,
                OfferStatus::Consumed,
                "{id} unused offer"
            );
            assert!(actual_offer.allocation.is_none(), "{id} unused allocation");
        }
    }
    let actual_active: u64 = result
        .patients
        .iter()
        .map(|p| {
            p.pathway
                .episodes
                .iter()
                .map(|e| e.observed_work_ticks)
                .sum::<u64>()
        })
        .sum();
    let completed_los: Vec<u64> = result
        .patients
        .iter()
        .filter_map(|p| p.completed_los_ticks)
        .collect();
    let first_waits: Vec<u64> = result
        .patients
        .iter()
        .filter_map(|p| {
            p.pathway
                .episodes
                .first()
                .and_then(|e| e.start_tick.map(|s| s - p.pathway.arrival_tick))
        })
        .collect();
    let elapsed: Vec<u64> = result
        .patients
        .iter()
        .map(|p| p.observed_elapsed_ticks)
        .collect();
    let queue_waits: Vec<u64> = result
        .patients
        .iter()
        .filter(|p| !p.pathway.episodes.is_empty())
        .map(|p| {
            p.pathway
                .episodes
                .iter()
                .map(|e| e.start_tick.unwrap() - e.queue_entry_tick)
                .sum()
        })
        .collect();
    let denominators = [
        result
            .patients
            .iter()
            .filter(|p| !p.pathway.episodes.is_empty())
            .count(),
        completed_los.len(),
        first_waits.len(),
        elapsed.len(),
        queue_waits.len(),
    ];
    for (index, key) in [
        "active_work",
        "completed_los",
        "first_care_wait",
        "observed_elapsed",
        "total_stage_queue_wait",
    ]
    .iter()
    .enumerate()
    {
        assert_eq!(
            denominators[index] as u64,
            expected["denominators"][key].as_u64().unwrap(),
            "{id} denominator {key}"
        );
    }
    let sums = [
        actual_active,
        completed_los.iter().sum(),
        first_waits.iter().sum(),
        elapsed.iter().sum(),
        queue_waits.iter().sum(),
    ];
    for (index, key) in [
        "mean_active_work",
        "mean_completed_los",
        "mean_first_care_wait",
        "mean_observed_elapsed",
        "mean_total_stage_queue_wait",
    ]
    .iter()
    .enumerate()
    {
        let expected_mean = &expected["metrics_ticks"][key];
        if denominators[index] == 0 {
            assert!(expected_mean.is_null(), "{id} undefined {key}");
        } else {
            let mean = sums[index] as f64 / denominators[index] as f64;
            assert_eq!(
                mean,
                expected_mean.as_f64().unwrap(),
                "{id} independent {key}"
            );
        }
    }
    println!(
        "E13_HORIZON_ENDPOINT_ACTUAL {id} {}",
        serde_json::to_string(&result).unwrap()
    );
}

fn e0_profile(capacity: u64) -> CapacityLocationProfile {
    let mut result = profile((
        KnowledgeStatus::Known,
        Some(capacity),
        capacity,
        capacity,
        "nurse",
    ));
    result
        .resource_buckets
        .retain(|r| r.resource_id == "resource.triage");
    result.zones.retain(|z| z.zone_id == "zone.triage");
    result
        .locations
        .retain(|l| l.location_id == "location.triage");
    result
        .task_classes
        .retain(|t| t.task_class_id == "task.triage");
    result
        .eligibility
        .retain(|e| e.task_class_id == "task.triage");
    result
}

fn e0_config(case: &Value, reverse: bool) -> ScenarioConfig {
    let input = &case["inputs"];
    let capacity = input["capacity"].as_u64().unwrap();
    let mut patients: Vec<_> = input["patients"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| PatientWorkItem {
            patient_id: p["patient_id"].as_str().unwrap().into(),
            arrival_tick: p["arrival_tick"].as_u64().unwrap(),
            work_ticks: p["service_ticks"].as_u64().unwrap(),
            resource_id: "resource.triage".into(),
            task_class_id: "task.triage".into(),
        })
        .collect();
    if reverse {
        patients.reverse();
    }
    ScenarioConfig {
        schema_version: 1,
        scenario_id: case["case_id"].as_str().unwrap().into(),
        seed: 0,
        time_unit: "tick".into(),
        horizon_ticks: input["horizon_ticks"].as_u64().unwrap(),
        provenance: Provenance {
            class: "synthetic".into(),
            notes: "frozen E0 extreme projection".into(),
        },
        capacity_location: e0_profile(capacity),
        patients,
    }
}

fn assert_extreme(case: &Value) {
    let id = case["case_id"].as_str().unwrap();
    if id == "extremes.zero_service_rejected" {
        let error = careops_ed::run_scenario(&e0_config(case, false)).expect_err(id);
        assert!(
            error.to_string().contains("work_ticks must be positive"),
            "{id}: {error}"
        );
        return;
    }
    let config = e0_config(case, false);
    let actual = careops_ed::run_scenario(&config).unwrap_or_else(|e| panic!("{id}: {e}"));
    let expected = &case["expected"];
    assert_eq!(
        actual.arrivals as u64,
        expected["counts"]["arrivals"].as_u64().unwrap(),
        "{id} arrivals"
    );
    assert_eq!(
        actual.started as u64,
        expected["counts"]["started"].as_u64().unwrap(),
        "{id} starts"
    );
    assert_eq!(
        actual.completed as u64,
        expected["counts"]["completed"].as_u64().unwrap(),
        "{id} completions"
    );
    assert_eq!(
        actual.unfinished as u64,
        expected["counts"]["unfinished"].as_u64().unwrap(),
        "{id} unfinished"
    );
    for row in expected["patients"].as_array().unwrap() {
        let p = actual
            .patients
            .iter()
            .find(|p| p.patient_id == row["patient_id"].as_str().unwrap())
            .unwrap();
        assert_eq!(p.start_tick, row["start_tick"].as_u64());
        assert_eq!(p.completion_tick, row["completion_tick"].as_u64());
        let configured = config
            .patients
            .iter()
            .find(|patient| patient.patient_id == p.patient_id)
            .unwrap()
            .work_ticks;
        assert_eq!(p.work_ticks, configured, "{id} configured work");
        let observed_work = p
            .start_tick
            .map(|start| {
                p.completion_tick
                    .unwrap_or(config.horizon_ticks)
                    .min(config.horizon_ticks)
                    .saturating_sub(start)
                    .min(p.work_ticks)
            })
            .unwrap_or(0);
        assert_eq!(observed_work, row["observed_work_ticks"].as_u64().unwrap());
        let derived_wait = p
            .start_tick
            .map(|start| start.saturating_sub(p.arrival_tick));
        assert_eq!(p.wait_ticks, derived_wait);
        assert_eq!(derived_wait, row["first_care_wait_ticks"].as_u64());
        let derived_elapsed = p
            .completion_tick
            .unwrap_or(config.horizon_ticks)
            .min(config.horizon_ticks)
            .saturating_sub(p.arrival_tick);
        assert_eq!(
            derived_elapsed,
            row["observed_elapsed_ticks"].as_u64().unwrap()
        );
        assert_eq!(p.elapsed_ticks, row["completed_los_ticks"].as_u64());
        assert_eq!(
            p.status,
            if row["status"] == "completed" {
                PatientStatus::Completed
            } else {
                PatientStatus::Unfinished
            }
        );
        if let Some(los) = row["completed_los_ticks"].as_u64() {
            assert_eq!(p.completion_tick.unwrap() - p.arrival_tick, los);
        } else {
            assert!(p.completion_tick.is_none());
        }
    }
    if id == "extremes.saturated" {
        let permuted = careops_ed::run_scenario(&e0_config(case, true)).unwrap();
        assert_eq!(actual, permuted, "saturated input permutation");
    }
}

fn valid_invalid_config(case: &Value) -> TerminalConfig {
    let input = &case["inputs"];
    let case_id = case["case_id"].as_str().unwrap();
    let horizon = if case_id == "invalid.horizon_nonpositive" {
        10
    } else {
        input["horizon"].as_u64().unwrap()
    };
    let mut config = empty_terminal(horizon);
    let ids: Vec<String> = if case["case_id"] == "invalid.patient_id_duplicate" {
        vec!["A".into(), "B".into()]
    } else {
        vec![input["patient_id"].as_str().unwrap_or("p1").into()]
    };
    let arrival = if case_id == "invalid.arrival_at_horizon" {
        1
    } else {
        input["arrival_tick"].as_u64().unwrap()
    };
    let acuity = if case_id == "invalid.acuity_invalid" {
        3
    } else {
        input["acuity"].as_u64().unwrap() as u8
    };
    for id in ids {
        let stages = vec![
            StageWork {
                kind: StageKind::Triage,
                resource_id: "resource.triage".into(),
                task_class_id: "task.triage".into(),
                duration_ticks: 2,
            },
            StageWork {
                kind: StageKind::Assessment,
                resource_id: "resource.assessment".into(),
                task_class_id: "task.assessment".into(),
                duration_ticks: 3,
            },
            StageWork {
                kind: StageKind::Treatment,
                resource_id: "resource.treatment".into(),
                task_class_id: "task.treatment".into(),
                duration_ticks: 5,
            },
        ];
        config
            .services
            .extend(
                stages
                    .iter()
                    .enumerate()
                    .map(|(stage_index, stage)| ServiceBinding {
                        patient_id: id.clone(),
                        stage_index,
                        service: fixed_service(stage.duration_ticks),
                    }),
            );
        config
            .pathway
            .patients
            .push(path_patient(&id, arrival, acuity, stages));
        config.decisions.push(DispositionBinding {
            patient_id: id.clone(),
            latent_intent: DispositionIntent::Discharge,
        });
    }
    config
}

fn mutate_invalid(id: &str, config: &mut TerminalConfig, input: &Value) {
    let person = if id == "invalid.patient_id_duplicate" {
        &mut config.pathway.patients[1]
    } else {
        &mut config.pathway.patients[0]
    };
    match id {
        "invalid.acuity_invalid" => person.acuity = input["acuity"].as_u64().unwrap() as u8,
        "invalid.arrival_at_horizon" => {
            person.arrival_tick = input["arrival_tick"].as_u64().unwrap()
        }
        "invalid.capacity_inconsistent"
        | "invalid.capacity_open_zero"
        | "invalid.capacity_staffed_over_open"
        | "invalid.capacity_staffed_zero"
        | "invalid.capacity_unknown"
        | "invalid.capacity_zero" => {
            let source = &input["resources"]["triage"];
            let r = config
                .pathway
                .capacity_location
                .resource_buckets
                .iter_mut()
                .find(|r| r.resource_id == "resource.triage")
                .unwrap();
            r.physical_status = if source["physical_count"].is_null() {
                KnowledgeStatus::Unknown
            } else {
                KnowledgeStatus::Known
            };
            r.physical_count = source["physical_count"].as_u64();
            r.open_count = source["open_count"].as_u64();
            r.staffed_count = source["staffed_count"].as_u64();
        }
        "invalid.duration_nonpositive" => {
            assert_eq!(input["stages"][0]["duration"].as_u64(), Some(0));
            person.stages[0].duration_ticks = input["stages"][0]["duration"].as_u64().unwrap();
            let b = config
                .services
                .iter_mut()
                .find(|b| b.patient_id == person.patient_id && b.stage_index == 0)
                .unwrap();
            b.service = fixed_service(0);
        }
        "invalid.eligibility_missing" => config
            .pathway
            .capacity_location
            .eligibility
            .retain(|e| !(e.task_class_id == "task.triage" && e.zone_id == "zone.triage")),
        "invalid.horizon_nonpositive" => config.pathway.horizon_ticks = 0,
        "invalid.patient_id_duplicate" => person.patient_id = "A".into(),
        "invalid.resource_class_incompatible" => {
            config
                .pathway
                .capacity_location
                .resource_buckets
                .iter_mut()
                .find(|r| r.resource_id == "resource.triage")
                .unwrap()
                .resource_class = input["resources"]["triage"]["class"]
                .as_str()
                .unwrap()
                .into()
        }
        "invalid.resource_unresolved" => {
            person.stages[0].resource_id = format!(
                "resource.{}",
                input["stages"][0]["resource"].as_str().unwrap()
            )
        }
        "invalid.stage_disordered" => {
            person.stages[0].kind = kind(input["stages"][0]["name"].as_str().unwrap());
            person.stages[1].kind = kind(input["stages"][1]["name"].as_str().unwrap());
        }
        "invalid.stage_repeated" => {
            person.stages[1].kind = kind(input["stages"][1]["name"].as_str().unwrap())
        }
        "invalid.task_unresolved" => {
            person.stages[0].task_class_id =
                format!("task.{}", input["stages"][0]["task"].as_str().unwrap())
        }
        other => panic!("unmapped invalid fixture {other}"),
    }
}

fn repair_invalid(id: &str, config: &mut TerminalConfig) {
    let person = if id == "invalid.patient_id_duplicate" {
        &mut config.pathway.patients[1]
    } else {
        &mut config.pathway.patients[0]
    };
    match id {
        "invalid.acuity_invalid" => person.acuity = 3,
        "invalid.arrival_at_horizon" => person.arrival_tick = 1,
        "invalid.capacity_inconsistent"
        | "invalid.capacity_open_zero"
        | "invalid.capacity_staffed_over_open"
        | "invalid.capacity_staffed_zero"
        | "invalid.capacity_unknown"
        | "invalid.capacity_zero" => {
            let r = config
                .pathway
                .capacity_location
                .resource_buckets
                .iter_mut()
                .find(|r| r.resource_id == "resource.triage")
                .unwrap();
            r.physical_status = KnowledgeStatus::Known;
            r.physical_count = Some(5);
            r.availability_status = KnowledgeStatus::Known;
            r.open_count = Some(5);
            r.staffed_count = Some(5);
        }
        "invalid.duration_nonpositive" => {
            person.stages[0].duration_ticks = 2;
            config
                .services
                .iter_mut()
                .find(|b| b.patient_id == person.patient_id && b.stage_index == 0)
                .unwrap()
                .service = fixed_service(2);
        }
        "invalid.eligibility_missing" => {
            config
                .pathway
                .capacity_location
                .eligibility
                .push(Eligibility {
                    staff_role_id: "staff.synthetic".into(),
                    task_class_id: "task.triage".into(),
                    zone_id: "zone.triage".into(),
                    location_id: None,
                })
        }
        "invalid.horizon_nonpositive" => config.pathway.horizon_ticks = 10,
        "invalid.patient_id_duplicate" => person.patient_id = "B".into(),
        "invalid.resource_class_incompatible" => {
            config
                .pathway
                .capacity_location
                .resource_buckets
                .iter_mut()
                .find(|r| r.resource_id == "resource.triage")
                .unwrap()
                .resource_class = "nurse".into()
        }
        "invalid.resource_unresolved" => person.stages[0].resource_id = "resource.triage".into(),
        "invalid.stage_disordered" => {
            person.stages[0].kind = StageKind::Triage;
            person.stages[1].kind = StageKind::Assessment;
        }
        "invalid.stage_repeated" => person.stages[1].kind = StageKind::Assessment,
        "invalid.task_unresolved" => person.stages[0].task_class_id = "task.triage".into(),
        other => panic!("unmapped invalid repair {other}"),
    }
}

fn invalid_message(id: &str) -> &'static str {
    match id {
        "invalid.acuity_invalid" => "acuity must be 1..5",
        "invalid.arrival_at_horizon" => "arrival must be before horizon",
        "invalid.capacity_inconsistent" => "open_count exceeds physical_count",
        "invalid.capacity_open_zero"
        | "invalid.capacity_staffed_zero"
        | "invalid.capacity_unknown"
        | "invalid.capacity_zero" => {
            "E1 requires known positive physical, open, and staffed capacity"
        }
        "invalid.capacity_staffed_over_open" => "staffed_count exceeds open_count",
        "invalid.duration_nonpositive" => {
            "fixed service duration must be positive and equal StageWork.duration_ticks"
        }
        "invalid.eligibility_missing" => "has no matching staff/task/zone eligibility",
        "invalid.horizon_nonpositive" => "horizon_ticks must be positive",
        "invalid.patient_id_duplicate" => {
            "decision bindings must be unique and name configured patients"
        }
        "invalid.resource_class_incompatible" => "task class is incompatible with resource",
        "invalid.resource_unresolved" => "refers to unknown resource",
        "invalid.stage_disordered" | "invalid.stage_repeated" => {
            "terminal runtime requires treatment-ending clinical paths"
        }
        "invalid.task_unresolved" => "refers to unknown task class",
        _ => panic!("unmapped invalid diagnostic {id}"),
    }
}

fn invalid_metadata(id: &str) -> (&'static str, &'static str) {
    match id {
        "invalid.acuity_invalid" => ("acuity", "acuity_invalid"),
        "invalid.arrival_at_horizon" => ("arrival_tick", "arrival_out_of_range"),
        "invalid.capacity_inconsistent" => ("resources.triage.open_count", "capacity_inconsistent"),
        "invalid.capacity_open_zero" => ("resources.triage.open_count", "capacity_invalid"),
        "invalid.capacity_staffed_over_open" => {
            ("resources.triage.staffed_count", "capacity_inconsistent")
        }
        "invalid.capacity_staffed_zero" => ("resources.triage.staffed_count", "capacity_invalid"),
        "invalid.capacity_unknown" | "invalid.capacity_zero" => {
            ("resources.triage.physical_count", "capacity_invalid")
        }
        "invalid.duration_nonpositive" => ("stages[0].duration", "duration_invalid"),
        "invalid.eligibility_missing" => ("stages[0].resource", "eligibility_missing"),
        "invalid.horizon_nonpositive" => ("horizon", "time_domain_invalid"),
        "invalid.patient_id_duplicate" => ("patient_ids", "patient_identity_invalid"),
        "invalid.resource_class_incompatible" => {
            ("stages[0].required_class", "resource_class_incompatible")
        }
        "invalid.resource_unresolved" => ("stages[0].resource", "resource_unresolved"),
        "invalid.stage_disordered" | "invalid.stage_repeated" => {
            ("stages", "stage_sequence_invalid")
        }
        "invalid.task_unresolved" => ("stages[0].task", "task_unresolved"),
        _ => panic!("unmapped invalid fixture metadata {id}"),
    }
}

#[test]
fn all_frozen_endpoint_cases_bind_to_native_terminal_rows_and_reducers() {
    let rows = cases(ENDPOINTS, "endpoints");
    assert_eq!(rows.len(), 23);
    let mut ids = BTreeSet::new();
    for row in &rows {
        let id = row["case_id"].as_str().unwrap().to_owned();
        assert!(ids.insert(id.clone()), "duplicate {id}");
        assert_endpoint(row);
    }
    exact_ids(
        &ids,
        &[
            "endpoints_empty",
            "endpoints_one_discharge",
            "endpoints_acuity_1_discharge_no_diagnostics",
            "endpoints_acuity_1_discharge_diagnostics",
            "endpoints_acuity_1_admission_no_diagnostics",
            "endpoints_acuity_1_admission_diagnostics",
            "endpoints_acuity_2_discharge_no_diagnostics",
            "endpoints_acuity_2_discharge_diagnostics",
            "endpoints_acuity_2_admission_no_diagnostics",
            "endpoints_acuity_2_admission_diagnostics",
            "endpoints_acuity_3_discharge_no_diagnostics",
            "endpoints_acuity_3_discharge_diagnostics",
            "endpoints_acuity_3_admission_no_diagnostics",
            "endpoints_acuity_3_admission_diagnostics",
            "endpoints_acuity_4_discharge_no_diagnostics",
            "endpoints_acuity_4_discharge_diagnostics",
            "endpoints_acuity_4_admission_no_diagnostics",
            "endpoints_acuity_4_admission_diagnostics",
            "endpoints_acuity_5_discharge_no_diagnostics",
            "endpoints_acuity_5_discharge_diagnostics",
            "endpoints_acuity_5_admission_no_diagnostics",
            "endpoints_acuity_5_admission_diagnostics",
            "endpoints_admission_no_offer",
        ],
        "endpoints",
    );
}

#[test]
fn all_frozen_extremes_use_the_actual_e0_runner() {
    let rows = cases(EXTREMES, "extremes");
    assert_eq!(rows.len(), 6);
    let mut ids = BTreeSet::new();
    for row in &rows {
        let id = row["case_id"].as_str().unwrap().to_owned();
        assert!(ids.insert(id.clone()), "duplicate {id}");
        assert_extreme(row);
    }
    exact_ids(
        &ids,
        &[
            "extremes.saturated",
            "extremes.saturated_permuted",
            "extremes.long_censored",
            "extremes.long_exact_horizon",
            "extremes.queued_censored",
            "extremes.zero_service_rejected",
        ],
        "extremes",
    );
}

#[test]
fn all_frozen_invalid_cases_reject_full_terminal_input_and_single_mutation_repairs() {
    let rows = cases(INVALID, "invalid");
    assert_eq!(rows.len(), 17);
    let mut ids = BTreeSet::new();
    for row in &rows {
        let id = row["case_id"].as_str().unwrap().to_owned();
        assert!(ids.insert(id.clone()), "duplicate {id}");
        assert_eq!(row["runtime_owner"], "E1.2");
        assert_eq!(row["expected"]["validation"], "reject_before_scheduling");
        let metadata_path = row["expected"]["offending_path"].as_str().unwrap();
        let metadata_category = row["expected"]["diagnostic_category"].as_str().unwrap();
        let (expected_path, expected_category) = invalid_metadata(&id);
        assert_eq!(metadata_path, expected_path, "{id} frozen offending path");
        assert_eq!(metadata_category, expected_category, "{id} frozen category");
        assert!(!row["expected"]["detail"].as_str().unwrap().is_empty());
        let baseline = valid_invalid_config(row);
        let baseline_result = run_terminal(&baseline)
            .unwrap_or_else(|e| panic!("{id} valid starting control failed before mutation: {e}"));
        e13::assert_conservation(&baseline_result);
        e13::assert_interval_accounting(
            &baseline,
            &baseline_result,
            baseline.pathway.horizon_ticks,
        );
        let mut malformed = baseline;
        mutate_invalid(&id, &mut malformed, &row["inputs"]);
        let error = run_terminal(&malformed).expect_err(&id);
        match error {
            PathwayError::InvalidInput(message) => assert!(
                message.contains(invalid_message(&id)),
                "{id} path={metadata_path} category={metadata_category}: {message}"
            ),
            other => panic!("{id} returned non-input error: {other}"),
        }
        let mut repaired = malformed.clone();
        repair_invalid(&id, &mut repaired);
        let output = run_terminal(&repaired)
            .unwrap_or_else(|e| panic!("{id} repaired control (only {metadata_path}) failed: {e}"));
        assert_eq!(output.counts.abandoned, 0, "{id} disabled abandonment");
        e13::assert_conservation(&output);
        e13::assert_interval_accounting(&repaired, &output, repaired.pathway.horizon_ticks);
    }
    exact_ids(
        &ids,
        &[
            "invalid.acuity_invalid",
            "invalid.arrival_at_horizon",
            "invalid.capacity_inconsistent",
            "invalid.capacity_open_zero",
            "invalid.capacity_staffed_over_open",
            "invalid.capacity_staffed_zero",
            "invalid.capacity_unknown",
            "invalid.capacity_zero",
            "invalid.duration_nonpositive",
            "invalid.eligibility_missing",
            "invalid.horizon_nonpositive",
            "invalid.patient_id_duplicate",
            "invalid.resource_class_incompatible",
            "invalid.resource_unresolved",
            "invalid.stage_disordered",
            "invalid.stage_repeated",
            "invalid.task_unresolved",
        ],
        "invalid",
    );
}
