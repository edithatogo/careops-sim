use std::collections::{BTreeMap, BTreeSet};

use careops_ed::disposition::boarding::{
    compute_endpoints, compute_endpoints_with_options, AbandonmentPolicy, BedOffer,
    BoarderCarePolicy, EndpointExecutionResult, ExitOptions, FacilityTransferPolicy,
};
use careops_ed::disposition::DispositionIntent;
use careops_ed::pathway::{run_pathway, PathwayConfig, PathwayPatient, StageKind, StageWork};
use careops_ed::{
    CapacityLocationProfile, Eligibility, KnowledgeStatus, Location, Provenance, ResourceBucket,
    StaffCount, StaffRole, TaskClass, Zone,
};
use serde_json::Value;

const ENDPOINTS: &str = include_str!("../../../conductor/design/ed/e1-endpoints.json");
type ResourceRow = (String, String, String, String, u64, u64, u64);

fn fixtures(source: &str, family: &str) -> Vec<Value> {
    let doc: Value = serde_json::from_str(source).expect("frozen fixture JSON parses");
    assert_eq!(doc["family"], family);
    assert_eq!(doc["runtime_status"], "unbound");
    doc["cases"]
        .as_array()
        .expect("fixture cases array")
        .clone()
}

fn kind(name: &str) -> StageKind {
    match name {
        "triage" => StageKind::Triage,
        "assessment" => StageKind::Assessment,
        "diagnostics" => StageKind::Diagnostics,
        "treatment" => StageKind::Treatment,
        _ => panic!("unmapped fixture stage {name}"),
    }
}

fn synthetic_profile(
    rows: &[(String, String, String, String, u64, u64, u64)],
) -> CapacityLocationProfile {
    let fallback = [(
        "resource.synthetic".into(),
        "nurse".into(),
        "task.synthetic".into(),
        "zone.synthetic".into(),
        1,
        1,
        1,
    )];
    let rows = if rows.is_empty() { &fallback[..] } else { rows };
    let mut zones = BTreeMap::new();
    let mut locations = BTreeMap::new();
    let mut resources = BTreeMap::new();
    let mut tasks = BTreeMap::new();
    let mut eligibilities = BTreeSet::new();

    for (resource, class, task, zone, physical, open, staffed) in rows {
        let location = format!("location.{zone}");
        zones.insert(
            zone.clone(),
            Zone {
                zone_id: zone.clone(),
                name: zone.clone(),
            },
        );
        locations.insert(
            location.clone(),
            Location {
                location_id: location.clone(),
                name: location.clone(),
                zone_id: zone.clone(),
            },
        );
        resources.insert(
            resource.clone(),
            ResourceBucket {
                resource_id: resource.clone(),
                resource_class: class.clone(),
                location_id: location.clone(),
                zone_id: zone.clone(),
                pool_group_id: None,
                physical_status: KnowledgeStatus::Known,
                physical_count: Some(*physical),
                availability_status: KnowledgeStatus::Known,
                open_count: Some(*open),
                staffed_count: Some(*staffed),
                availability_reason: None,
                capabilities: vec![],
            },
        );
        tasks.entry(task.clone()).or_insert_with(|| TaskClass {
            task_class_id: task.clone(),
            name: task.clone(),
            required_resource_classes: vec![class.clone()],
        });
        eligibilities.insert((task.clone(), zone.clone()));
    }

    let staff = StaffRole {
        staff_role_id: "staff.synthetic".into(),
        name: "synthetic staff".into(),
        counts: vec![StaffCount {
            basis: "present".into(),
            status: KnowledgeStatus::Known,
            count: Some(1),
            reference: "synthetic.fixture".into(),
            reason: None,
        }],
    };

    CapacityLocationProfile {
        schema_version: 3,
        status: "proposed".into(),
        capacity: careops_ed::AggregateCapacity {
            unit: "treatment_space".into(),
            bucket_completeness: "complete".into(),
            physical_status: KnowledgeStatus::Known,
            physical_count: Some(1),
            availability_status: KnowledgeStatus::Known,
            open_count: Some(1),
            staffed_count: Some(1),
            availability_reason: None,
            equipment: vec![],
        },
        zones: zones.into_values().collect(),
        locations: locations.into_values().collect(),
        resource_buckets: resources.into_values().collect(),
        task_classes: tasks.into_values().collect(),
        staff_roles: vec![staff],
        eligibility: eligibilities
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

fn cfg(horizon: u64, patients: Vec<PathwayPatient>, rows: &[ResourceRow]) -> PathwayConfig {
    PathwayConfig {
        version: 1,
        horizon_ticks: horizon,
        provenance: Provenance {
            class: "synthetic".into(),
            notes: "frozen synthetic component fixture".into(),
        },
        capacity_location: synthetic_profile(rows),
        patients,
    }
}

fn fixture_endpoint_patient(input: &Value) -> (PathwayPatient, Vec<ResourceRow>) {
    let mut rows = vec![];
    let stages = input["stages"]
        .as_array()
        .map(|stages| {
            stages
                .iter()
                .map(|stage| {
                    let stage_name = stage["stage"].as_str().expect("stage name");
                    let resource = stage["resource_id"]
                        .as_str()
                        .expect("resource id")
                        .to_owned();
                    let task = format!("task.{stage_name}");
                    rows.push((
                        resource.clone(),
                        "nurse".into(),
                        task.clone(),
                        format!("zone.{stage_name}"),
                        1,
                        1,
                        1,
                    ));
                    StageWork {
                        kind: kind(stage_name),
                        resource_id: resource,
                        task_class_id: task,
                        duration_ticks: stage["duration_ticks"]
                            .as_u64()
                            .expect("positive stage duration"),
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    (
        PathwayPatient {
            patient_id: input["patient_id"].as_str().unwrap().into(),
            arrival_tick: input["arrival_tick"].as_u64().unwrap(),
            mode: "synthetic".into(),
            acuity: input["latent_state"]["acuity"].as_u64().unwrap() as u8,
            stages,
        },
        rows,
    )
}

#[test]
fn test_all_23_endpoint_fixtures_match_oracle_exactly() {
    let cases = fixtures(ENDPOINTS, "endpoints");
    assert_eq!(cases.len(), 23);

    for case in &cases {
        let id = case["case_id"].as_str().unwrap();
        let inputs = &case["inputs"];
        let horizon_tick = inputs["horizon_tick"].as_u64().unwrap();

        let mut patients = vec![];
        let mut rows = vec![];
        let mut provisional_dispositions = BTreeMap::new();

        for input in inputs["patients"].as_array().unwrap() {
            let (patient, patient_rows) = fixture_endpoint_patient(input);
            if let Some(disp) = input["provisional_disposition"].as_str() {
                let intent = match disp {
                    "admission" => DispositionIntent::Admission,
                    "discharge" => DispositionIntent::Discharge,
                    other => panic!("unknown provisional disposition: {other}"),
                };
                provisional_dispositions.insert(patient.patient_id.clone(), intent);
            }
            patients.push(patient);
            rows.extend(patient_rows);
        }

        let bed_offers: Vec<BedOffer> = inputs["bed_offers"]
            .as_array()
            .unwrap()
            .iter()
            .map(|o| BedOffer {
                offer_id: o["offer_id"].as_str().unwrap().into(),
                patient_id: o["patient_id"].as_str().unwrap().into(),
                destination: o["destination"].as_str().unwrap().into(),
                available_tick: o["available_tick"].as_u64().unwrap(),
                withdrawn_tick: o["withdrawn_tick"].as_u64().unwrap(),
                transfer_duration_ticks: o["transfer_duration_ticks"].as_u64().unwrap(),
            })
            .collect();

        let pathway_results = run_pathway(&cfg(horizon_tick, patients.clone(), &rows)).unwrap();

        let endpoint_result: EndpointExecutionResult = compute_endpoints(
            &pathway_results,
            &patients,
            &provisional_dispositions,
            &bed_offers,
            horizon_tick,
        )
        .expect("compute_endpoints succeeds");

        let actual_json = serde_json::to_value(&endpoint_result).expect("serialize actual");
        let expected_json = &case["expected"];

        assert_eq!(
            actual_json["counts"], expected_json["counts"],
            "{id}: counts mismatch"
        );
        assert_eq!(
            actual_json["metrics_ticks"], expected_json["metrics_ticks"],
            "{id}: metrics_ticks mismatch"
        );
        assert_eq!(
            actual_json["denominators"], expected_json["denominators"],
            "{id}: denominators mismatch"
        );
        assert_eq!(
            actual_json["patients"], expected_json["patients"],
            "{id}: patients mismatch"
        );
    }
}

#[test]
fn test_single_bed_offer_cannot_be_consumed_twice() {
    let p1 = PathwayPatient {
        patient_id: "p1".into(),
        arrival_tick: 1,
        mode: "synthetic".into(),
        acuity: 3,
        stages: vec![
            StageWork {
                kind: StageKind::Triage,
                resource_id: "res.triage".into(),
                task_class_id: "task.triage".into(),
                duration_ticks: 2,
            },
            StageWork {
                kind: StageKind::Assessment,
                resource_id: "res.assessment".into(),
                task_class_id: "task.assessment".into(),
                duration_ticks: 2,
            },
            StageWork {
                kind: StageKind::Treatment,
                resource_id: "res.treatment".into(),
                task_class_id: "task.treatment".into(),
                duration_ticks: 2,
            },
        ],
    };
    let mut p2 = p1.clone();
    p2.patient_id = "p2".into();

    let rows = [
        (
            "res.triage".into(),
            "nurse".into(),
            "task.triage".into(),
            "zone.triage".into(),
            2,
            2,
            2,
        ),
        (
            "res.assessment".into(),
            "nurse".into(),
            "task.assessment".into(),
            "zone.assessment".into(),
            2,
            2,
            2,
        ),
        (
            "res.treatment".into(),
            "nurse".into(),
            "task.treatment".into(),
            "zone.treatment".into(),
            2,
            2,
            2,
        ),
    ];

    let patients = vec![p1, p2];
    let pathway_results = run_pathway(&cfg(30, patients.clone(), &rows)).unwrap();

    let mut provisional_dispositions = BTreeMap::new();
    provisional_dispositions.insert("p1".into(), DispositionIntent::Admission);
    provisional_dispositions.insert("p2".into(), DispositionIntent::Admission);

    let bed_offers = vec![BedOffer {
        offer_id: "offer-1".into(),
        patient_id: "p1".into(),
        destination: "ward-a".into(),
        available_tick: 10,
        withdrawn_tick: 20,
        transfer_duration_ticks: 2,
    }];

    let res = compute_endpoints(
        &pathway_results,
        &patients,
        &provisional_dispositions,
        &bed_offers,
        30,
    )
    .unwrap();

    assert_eq!(res.patients[0].endpoint_state, "terminal");
    assert_eq!(
        res.patients[0].realized_outcome.as_ref().unwrap().kind,
        "transferred"
    );

    assert_eq!(res.patients[1].endpoint_state, "unfinished_at_horizon");
    assert_eq!(
        res.patients[1].unfinished_reason.as_deref(),
        Some("boarding_without_compatible_offer")
    );
}

#[test]
fn test_expired_or_incompatible_bed_offer_is_not_consumed() {
    let p1 = PathwayPatient {
        patient_id: "p1".into(),
        arrival_tick: 1,
        mode: "synthetic".into(),
        acuity: 3,
        stages: vec![
            StageWork {
                kind: StageKind::Triage,
                resource_id: "res.triage".into(),
                task_class_id: "task.triage".into(),
                duration_ticks: 2,
            },
            StageWork {
                kind: StageKind::Assessment,
                resource_id: "res.assessment".into(),
                task_class_id: "task.assessment".into(),
                duration_ticks: 2,
            },
            StageWork {
                kind: StageKind::Treatment,
                resource_id: "res.treatment".into(),
                task_class_id: "task.treatment".into(),
                duration_ticks: 4,
            },
        ],
    };

    let rows = [
        (
            "res.triage".into(),
            "nurse".into(),
            "task.triage".into(),
            "zone.triage".into(),
            1,
            1,
            1,
        ),
        (
            "res.assessment".into(),
            "nurse".into(),
            "task.assessment".into(),
            "zone.assessment".into(),
            1,
            1,
            1,
        ),
        (
            "res.treatment".into(),
            "nurse".into(),
            "task.treatment".into(),
            "zone.treatment".into(),
            1,
            1,
            1,
        ),
    ];

    let patients = vec![p1];
    let pathway_results = run_pathway(&cfg(30, patients.clone(), &rows)).unwrap();

    let mut provisional_dispositions = BTreeMap::new();
    provisional_dispositions.insert("p1".into(), DispositionIntent::Admission);

    let bed_offers = vec![BedOffer {
        offer_id: "offer-expired".into(),
        patient_id: "p1".into(),
        destination: "ward-a".into(),
        available_tick: 5,
        withdrawn_tick: 8,
        transfer_duration_ticks: 2,
    }];

    let res = compute_endpoints(
        &pathway_results,
        &patients,
        &provisional_dispositions,
        &bed_offers,
        30,
    )
    .unwrap();

    assert_eq!(res.patients[0].endpoint_state, "unfinished_at_horizon");
    assert_eq!(
        res.patients[0].unfinished_reason.as_deref(),
        Some("boarding_without_compatible_offer")
    );
    assert_eq!(res.patients[0].realized_outcome, None);
    assert_eq!(
        res.patients[0]
            .boarding_interval
            .as_ref()
            .unwrap()
            .start_tick,
        9
    );
    assert_eq!(
        res.patients[0].boarding_interval.as_ref().unwrap().end_tick,
        30
    );
}

#[test]
fn test_disabled_or_empty_policy_never_silently_activates_transfer() {
    let p1 = PathwayPatient {
        patient_id: "p1".into(),
        arrival_tick: 1,
        mode: "synthetic".into(),
        acuity: 3,
        stages: vec![
            StageWork {
                kind: StageKind::Triage,
                resource_id: "res.triage".into(),
                task_class_id: "task.triage".into(),
                duration_ticks: 2,
            },
            StageWork {
                kind: StageKind::Assessment,
                resource_id: "res.assessment".into(),
                task_class_id: "task.assessment".into(),
                duration_ticks: 2,
            },
            StageWork {
                kind: StageKind::Treatment,
                resource_id: "res.treatment".into(),
                task_class_id: "task.treatment".into(),
                duration_ticks: 2,
            },
        ],
    };

    let rows = [
        (
            "res.triage".into(),
            "nurse".into(),
            "task.triage".into(),
            "zone.triage".into(),
            1,
            1,
            1,
        ),
        (
            "res.assessment".into(),
            "nurse".into(),
            "task.assessment".into(),
            "zone.assessment".into(),
            1,
            1,
            1,
        ),
        (
            "res.treatment".into(),
            "nurse".into(),
            "task.treatment".into(),
            "zone.treatment".into(),
            1,
            1,
            1,
        ),
    ];

    let patients = vec![p1];
    let pathway_results = run_pathway(&cfg(30, patients.clone(), &rows)).unwrap();

    let provisional_dispositions = BTreeMap::new();

    let bed_offers = vec![BedOffer {
        offer_id: "offer-unused".into(),
        patient_id: "p1".into(),
        destination: "ward-a".into(),
        available_tick: 10,
        withdrawn_tick: 20,
        transfer_duration_ticks: 2,
    }];

    let res = compute_endpoints(
        &pathway_results,
        &patients,
        &provisional_dispositions,
        &bed_offers,
        30,
    )
    .unwrap();

    assert_eq!(res.patients[0].endpoint_state, "unfinished_at_horizon");
    assert_eq!(res.patients[0].transfer_interval, None);
    assert_eq!(res.patients[0].realized_outcome, None);
    assert_eq!(res.patients[0].completed_los_ticks, None);
}

#[test]
fn test_wrong_destination_cannot_accept() {
    let p1 = PathwayPatient {
        patient_id: "p1".into(),
        arrival_tick: 1,
        mode: "synthetic".into(),
        acuity: 3,
        stages: vec![
            StageWork {
                kind: StageKind::Triage,
                resource_id: "res.triage".into(),
                task_class_id: "task.triage".into(),
                duration_ticks: 2,
            },
            StageWork {
                kind: StageKind::Assessment,
                resource_id: "res.assessment".into(),
                task_class_id: "task.assessment".into(),
                duration_ticks: 2,
            },
            StageWork {
                kind: StageKind::Treatment,
                resource_id: "res.treatment".into(),
                task_class_id: "task.treatment".into(),
                duration_ticks: 2,
            },
        ],
    };

    let rows = [
        (
            "res.triage".into(),
            "nurse".into(),
            "task.triage".into(),
            "zone.triage".into(),
            1,
            1,
            1,
        ),
        (
            "res.assessment".into(),
            "nurse".into(),
            "task.assessment".into(),
            "zone.assessment".into(),
            1,
            1,
            1,
        ),
        (
            "res.treatment".into(),
            "nurse".into(),
            "task.treatment".into(),
            "zone.treatment".into(),
            1,
            1,
            1,
        ),
    ];

    let patients = vec![p1];
    let pathway_results = run_pathway(&cfg(30, patients.clone(), &rows)).unwrap();

    let mut provisional_dispositions = BTreeMap::new();
    provisional_dispositions.insert("p1".into(), DispositionIntent::Admission);

    let bed_offers = vec![BedOffer {
        offer_id: "offer-bad-dest".into(),
        patient_id: "p1".into(),
        destination: "invalid destination with spaces!".into(),
        available_tick: 10,
        withdrawn_tick: 20,
        transfer_duration_ticks: 2,
    }];

    let err = compute_endpoints(
        &pathway_results,
        &patients,
        &provisional_dispositions,
        &bed_offers,
        30,
    )
    .expect_err("invalid destination must be rejected");

    assert!(err
        .to_string()
        .contains("invalid bed offer destination syntax"));
}

fn admission_patient(id: &str, treatment_ticks: u64) -> (PathwayPatient, Vec<ResourceRow>) {
    let stages = [
        ("triage", 2),
        ("assessment", 2),
        ("treatment", treatment_ticks),
    ];
    let mut rows = vec![];
    let mut work = vec![];
    for (stage_name, duration) in stages {
        let resource = format!("res.{stage_name}");
        let task = format!("task.{stage_name}");
        rows.push((
            resource.clone(),
            "nurse".into(),
            task.clone(),
            format!("zone.{stage_name}"),
            1,
            1,
            1,
        ));
        work.push(StageWork {
            kind: kind(stage_name),
            resource_id: resource,
            task_class_id: task,
            duration_ticks: duration,
        });
    }
    (
        PathwayPatient {
            patient_id: id.into(),
            arrival_tick: 1,
            mode: "synthetic".into(),
            acuity: 3,
            stages: work,
        },
        rows,
    )
}

#[test]
fn optional_abandonment_is_explicit_and_disabled_policy_stays_off() {
    let (patient, rows) = admission_patient("wait", 20);
    let pathways = run_pathway(&cfg(8, vec![patient.clone()], &rows)).unwrap();
    let censored =
        compute_endpoints(&pathways, &[patient.clone()], &BTreeMap::new(), &[], 8).unwrap();
    assert_eq!(censored.patients[0].endpoint_state, "unfinished_at_horizon");
    assert!(censored.patients[0].realized_outcome.is_none());

    let enabled = ExitOptions {
        abandonment: Some(AbandonmentPolicy {
            version: 1,
            patience_ticks: 2,
        }),
        ..ExitOptions::default()
    };
    let abandoned =
        compute_endpoints_with_options(&pathways, &[patient], &BTreeMap::new(), &[], 8, &enabled)
            .unwrap();
    assert_eq!(
        abandoned.patients[0]
            .realized_outcome
            .as_ref()
            .unwrap()
            .kind,
        "abandoned"
    );
    assert_eq!(abandoned.patients[0].terminal_tick, Some(3));
    assert!(!abandoned.claims.ward_policy);
    assert!(!abandoned.claims.community_ambulance);
}

#[test]
fn recurrent_care_continues_until_physical_departure_without_staff_reservation() {
    let (patient, rows) = admission_patient("p", 2);
    let pathways = run_pathway(&cfg(30, vec![patient.clone()], &rows)).unwrap();
    let offers = vec![BedOffer {
        offer_id: "offer-1".into(),
        patient_id: "p".into(),
        destination: "ward-a".into(),
        available_tick: 8,
        withdrawn_tick: 30,
        transfer_duration_ticks: 4,
    }];
    let options = ExitOptions {
        care: Some(BoarderCarePolicy {
            version: 1,
            interval_ticks: 2,
            duration_ticks: 1,
        }),
        ..ExitOptions::default()
    };
    let actual = compute_endpoints_with_options(
        &pathways,
        &[patient],
        &BTreeMap::from([("p".into(), DispositionIntent::Admission)]),
        &offers,
        30,
        &options,
    )
    .unwrap();
    let person = &actual.patients[0];
    let acceptance = person.transfer_interval.as_ref().unwrap().start_tick;
    let departure = person.terminal_tick.unwrap();
    assert!(departure > acceptance);
    assert_eq!(
        person.boarding_interval.as_ref().unwrap().end_tick,
        acceptance
    );
    assert!(actual.boarder_care.len() >= 2);
    assert!(actual.boarder_care.iter().all(|task| !task.reserves_staff));
    assert!(actual
        .boarder_care
        .iter()
        .all(|task| task.start_tick < departure));
}

#[test]
fn horizon_edge_censors_unfinished_transfer_without_causal_claims() {
    let (patient, rows) = admission_patient("p", 2);
    let pathways = run_pathway(&cfg(20, vec![patient.clone()], &rows)).unwrap();
    let offers = vec![BedOffer {
        offer_id: "late".into(),
        patient_id: "p".into(),
        destination: "ward-a".into(),
        available_tick: 7,
        withdrawn_tick: 40,
        transfer_duration_ticks: 4,
    }];
    let intents = BTreeMap::from([("p".into(), DispositionIntent::Admission)]);
    let finished = compute_endpoints(&pathways, &[patient.clone()], &intents, &offers, 11).unwrap();
    assert_eq!(finished.patients[0].endpoint_state, "terminal");
    assert_eq!(finished.patients[0].terminal_tick, Some(11));

    let censored = compute_endpoints(&pathways, &[patient.clone()], &intents, &offers, 10).unwrap();
    assert_eq!(censored.patients[0].endpoint_state, "unfinished_at_horizon");
    assert!(censored.patients[0].realized_outcome.is_none());
    assert_eq!(
        censored.patients[0].unfinished_reason.as_deref(),
        Some("censored_during_transfer")
    );
    assert_eq!(
        censored.counts.terminal + censored.counts.unfinished,
        censored.counts.arrivals
    );
    assert!(!censored.claims.ward_policy);
    assert!(!censored.claims.community_ambulance);

    let mut requests = BTreeMap::new();
    requests.insert("p".into(), "facility.other".into());
    let blocked = ExitOptions {
        transfer_requests: requests,
        ..ExitOptions::default()
    };
    let error =
        compute_endpoints_with_options(&pathways, &[patient.clone()], &intents, &[], 20, &blocked)
            .unwrap_err();
    assert!(error.to_string().contains("transfer policy is disabled"));

    let mut eligible_requests = BTreeMap::new();
    eligible_requests.insert("p".into(), "facility.other".into());
    let transfer = ExitOptions {
        transfer: Some(FacilityTransferPolicy {
            version: 1,
            transport_delay_ticks: 2,
            eligible_destinations: BTreeSet::from(["facility.other".into()]),
        }),
        transfer_requests: eligible_requests,
        ..ExitOptions::default()
    };
    let moved = compute_endpoints_with_options(&pathways, &[patient], &intents, &[], 20, &transfer)
        .unwrap();
    let outcome = moved.patients[0].realized_outcome.as_ref().unwrap();
    assert_eq!(outcome.kind, "transferred");
    assert_eq!(outcome.destination.as_deref(), Some("facility.other"));
    assert!(outcome.offer_id.is_none());
}
