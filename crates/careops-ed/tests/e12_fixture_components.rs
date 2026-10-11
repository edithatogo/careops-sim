use std::collections::{BTreeMap, BTreeSet};

use careops_ed::pathway::{
    PathwayConfig, PathwayError, PathwayPatient, PathwayResult, PathwayState, ServiceBinding,
    ServiceLaw, ServiceProvenance, ServiceSpec, StageKind, StageWork,
};
use careops_ed::{
    AggregateCapacity, CapacityLocationProfile, Eligibility, KnowledgeStatus, Location, Provenance,
    ResourceBucket, StaffCount, StaffRole, TaskClass, Zone,
};
use serde_json::Value;

const ENDPOINTS: &str = include_str!("../../../conductor/design/ed/e1-endpoints.json");
const EXTREMES: &str = include_str!("../../../conductor/design/ed/e1-extremes.json");
const INVALID: &str = include_str!("../../../conductor/design/ed/e1-invalid.json");
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

fn assert_exact_ids<T: AsRef<str> + Ord>(actual: &BTreeSet<T>, expected: &[&str]) {
    let expected: BTreeSet<String> = expected.iter().map(|id| (*id).to_owned()).collect();
    let actual: BTreeSet<String> = actual.iter().map(|id| id.as_ref().to_owned()).collect();
    assert_eq!(actual, expected, "fixture case ID set changed");
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
        capacity: AggregateCapacity {
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

fn cfg(
    horizon: u64,
    patients: Vec<PathwayPatient>,
    rows: &[(String, String, String, String, u64, u64, u64)],
) -> PathwayConfig {
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
fn endpoint_fixtures_bind_native_component_projections() {
    let cases = fixtures(ENDPOINTS, "endpoints");
    assert_eq!(cases.len(), 23);
    let mut ids = BTreeSet::new();
    for case in &cases {
        let id = case["case_id"].as_str().unwrap();
        assert!(ids.insert(id), "duplicate endpoint fixture {id}");
        let owner = if id.contains("admission") {
            "E1.3"
        } else {
            "E1.2"
        };
        assert_eq!(case["runtime_owner"], owner, "{id}: frozen owner metadata");
        let inputs = &case["inputs"];
        let mut patients = vec![];
        let mut rows = vec![];
        for input in inputs["patients"].as_array().unwrap() {
            let (patient, patient_rows) = fixture_endpoint_patient(input);
            patients.push(patient);
            rows.extend(patient_rows);
        }
        let result = careops_ed::pathway::run_pathway(&cfg(
            inputs["horizon_tick"].as_u64().unwrap(),
            patients.clone(),
            &rows,
        ))
        .unwrap();
        let expected = &case["expected"]["patients"];
        assert_eq!(
            result.len(),
            expected.as_array().unwrap().len(),
            "{id}: native result cardinality"
        );
        for (actual, expected) in result.iter().zip(expected.as_array().unwrap()) {
            assert_eq!(
                actual.patient_id,
                expected["patient_id"].as_str().unwrap(),
                "{id}"
            );
            assert_eq!(
                actual.acuity,
                expected["latent_state"]["acuity"].as_u64().unwrap() as u8,
                "{id}: latent acuity"
            );
            let known = expected["agent_knowledge"]["triage_acuity"]
                .as_u64()
                .map(|n| n as u8);
            assert_eq!(
                actual.known_acuity, known,
                "{id}: knowledge is realized by native triage completion"
            );
            let input = inputs["patients"]
                .as_array()
                .unwrap()
                .iter()
                .find(|p| p["patient_id"] == actual.patient_id)
                .unwrap();
            let expected_intervals = expected["stage_intervals"].as_array().unwrap();
            assert_eq!(
                actual.episodes.len(),
                expected_intervals.len(),
                "{id}: stage count"
            );
            let mut active = 0u64;
            let mut queue_wait = 0u64;
            for ((episode, interval), stage_input) in actual
                .episodes
                .iter()
                .zip(expected_intervals)
                .zip(input["stages"].as_array().unwrap())
            {
                let stage = interval["stage"].as_str().unwrap();
                assert_eq!(episode.kind, kind(stage), "{id}");
                assert_eq!(
                    stage_input["resource_id"].as_str().unwrap(),
                    interval["resource_id"].as_str().unwrap(),
                    "{id}: input mapping only; StageEpisode exposes no resource identity"
                );
                let start = interval["start_tick"].as_u64().unwrap();
                let end = interval["end_tick"].as_u64().unwrap();
                assert_eq!(episode.start_tick, Some(start), "{id}");
                assert_eq!(episode.completion_tick, Some(end), "{id}");
                assert_eq!(episode.observed_work_ticks, end - start, "{id}");
                assert_eq!(
                    episode.sampled_duration_ticks,
                    stage_input["duration_ticks"].as_u64().unwrap(),
                    "{id}"
                );
                active += episode.observed_work_ticks;
                queue_wait += start - episode.queue_entry_tick;
            }
            assert_eq!(
                active,
                expected["active_work_ticks"].as_u64().unwrap(),
                "{id}: active work only"
            );
            assert_eq!(
                queue_wait,
                expected["total_stage_queue_wait_ticks"].as_u64().unwrap(),
                "{id}: sum of stage queue waits"
            );
            let first_wait = actual
                .episodes
                .first()
                .and_then(|episode| episode.start_tick)
                .map(|start| start - actual.arrival_tick);
            assert_eq!(
                first_wait,
                expected["first_care_wait_ticks"].as_u64(),
                "{id}: first-care wait"
            );
            assert_eq!(
                actual.arrival_tick,
                expected["arrival_tick"].as_u64().unwrap()
            );
            assert_eq!(actual.state, PathwayState::TreatmentComplete);
            assert_eq!(
                actual.episodes.last().unwrap().kind,
                StageKind::Treatment,
                "{id}: all nonempty endpoint paths end at treatment only"
            );
        }
    }
    assert_exact_ids(
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
    );
}

fn extreme_config(case: &Value) -> (PathwayConfig, Vec<ServiceBinding>) {
    let inputs = &case["inputs"];
    let horizon = inputs["horizon_ticks"].as_u64().unwrap();
    let mut patients = vec![];
    let mut bindings = vec![];
    for patient in inputs["patients"].as_array().unwrap() {
        let patient_id = patient["patient_id"].as_str().unwrap().to_owned();
        let service = patient["service_ticks"].as_u64().unwrap();
        patients.push(PathwayPatient {
            patient_id: patient_id.clone(),
            arrival_tick: patient["arrival_tick"].as_u64().unwrap(),
            mode: "synthetic".into(),
            acuity: 3,
            stages: vec![
                StageWork {
                    kind: StageKind::Triage,
                    resource_id: "resource.triage".into(),
                    task_class_id: "task.triage".into(),
                    duration_ticks: service,
                },
                StageWork {
                    kind: StageKind::Assessment,
                    resource_id: "resource.assessment".into(),
                    task_class_id: "task.assessment".into(),
                    duration_ticks: 1,
                },
            ],
        });
        bindings.push(ServiceBinding {
            patient_id,
            stage_index: 0,
            service: ServiceSpec {
                version: 1,
                provenance: ServiceProvenance {
                    class: "synthetic".into(),
                    notes: "extreme service fixture".into(),
                },
                law: ServiceLaw::Fixed {
                    duration_ticks: service,
                },
            },
        });
        bindings.push(ServiceBinding {
            patient_id: patient["patient_id"].as_str().unwrap().into(),
            stage_index: 1,
            service: ServiceSpec {
                version: 1,
                provenance: ServiceProvenance {
                    class: "synthetic".into(),
                    notes: "positive later stage".into(),
                },
                law: ServiceLaw::Fixed { duration_ticks: 1 },
            },
        });
    }
    let rows = vec![
        (
            "resource.triage".into(),
            "nurse".into(),
            "task.triage".into(),
            "zone.triage".into(),
            inputs["capacity"].as_u64().unwrap_or(1),
            inputs["capacity"].as_u64().unwrap_or(1),
            inputs["capacity"].as_u64().unwrap_or(1),
        ),
        (
            "resource.assessment".into(),
            "nurse".into(),
            "task.assessment".into(),
            "zone.assessment".into(),
            100_000,
            100_000,
            100_000,
        ),
    ];
    (cfg(horizon, patients, &rows), bindings)
}

#[test]
fn extreme_fixtures_bind_only_triage_projection() {
    let cases = fixtures(EXTREMES, "extremes");
    assert_eq!(cases.len(), 6);
    let mut ids = BTreeSet::new();
    for case in cases {
        let id = case["case_id"].as_str().unwrap().to_owned();
        assert!(ids.insert(id.clone()));
        assert_eq!(case["runtime_owner"], "E1.2");
        if id == "extremes.zero_service_rejected" {
            let (config, bindings) = extreme_config(&case);
            assert!(matches!(
                careops_ed::pathway::run_pathway_with_services(&config, 0, &bindings),
                Err(PathwayError::InvalidInput(_))
            ));
            continue;
        }
        let (config, bindings) = extreme_config(&case);
        let results =
            careops_ed::pathway::run_pathway_with_services(&config, 0, &bindings).unwrap();
        let expected = case["expected"]["patients"].as_array().unwrap();
        assert_eq!(results.len(), expected.len(), "{id}");
        for (result, expected) in results.iter().zip(expected) {
            let triage = result
                .episodes
                .iter()
                .find(|episode| episode.kind == StageKind::Triage)
                .unwrap();
            assert_eq!(result.patient_id, expected["patient_id"].as_str().unwrap());
            assert_eq!(triage.start_tick, expected["start_tick"].as_u64());
            assert_eq!(triage.completion_tick, expected["completion_tick"].as_u64());
            assert_eq!(
                triage.observed_work_ticks,
                expected["observed_work_ticks"].as_u64().unwrap()
            );
            assert_eq!(triage.queue_entry_tick, result.arrival_tick);
            assert_eq!(
                triage
                    .start_tick
                    .map(|start| start - triage.queue_entry_tick),
                expected["first_care_wait_ticks"].as_u64()
            );
        }
        if id == "extremes.saturated" {
            let mut permuted = case.clone();
            permuted["inputs"]["patients"]
                .as_array_mut()
                .unwrap()
                .reverse();
            let (permuted_config, permuted_bindings) = extreme_config(&permuted);
            let permuted_results = careops_ed::pathway::run_pathway_with_services(
                &permuted_config,
                0,
                &permuted_bindings,
            )
            .unwrap();
            let projection = |rows: &[PathwayResult]| {
                rows.iter()
                    .map(|row| {
                        let triage = row
                            .episodes
                            .iter()
                            .find(|episode| episode.kind == StageKind::Triage)
                            .unwrap();
                        (
                            row.patient_id.clone(),
                            triage.start_tick,
                            triage.completion_tick,
                            triage.observed_work_ticks,
                        )
                    })
                    .collect::<Vec<_>>()
            };
            assert_eq!(projection(&results), projection(&permuted_results));
        }
    }
    assert_exact_ids(
        &ids,
        &[
            "extremes.saturated",
            "extremes.saturated_permuted",
            "extremes.long_censored",
            "extremes.long_exact_horizon",
            "extremes.queued_censored",
            "extremes.zero_service_rejected",
        ],
    );
}

fn invalid_native(case: &Value) -> (PathwayConfig, String, String) {
    let inputs = &case["inputs"];
    let id = case["case_id"].as_str().unwrap();
    let category = case["expected"]["diagnostic_category"].as_str().unwrap();
    let mut rows = vec![];
    let resources = inputs["resources"].as_object().unwrap();
    let tasks = inputs["tasks"].as_object().unwrap();
    let fixture_stages = inputs["stages"].as_array().unwrap();
    for (resource_id, resource) in resources {
        let zone = resource["zone"].as_str().unwrap().to_owned();
        let stage = fixture_stages
            .iter()
            .find(|stage| stage["resource"].as_str() == Some(resource_id))
            .unwrap_or(&fixture_stages[0]);
        let task_id = stage["task"].as_str().unwrap();
        let count = |field: &str| resource.get(field).and_then(Value::as_u64).unwrap_or(1);
        rows.push((
            format!("resource.{resource_id}"),
            resource["class"].as_str().unwrap().into(),
            format!("task.{task_id}"),
            format!("zone.{zone}"),
            count("physical_count"),
            count("open_count"),
            count("staffed_count"),
        ));
    }
    for (task_id, task) in tasks {
        for resource_id in task["eligible_resources"].as_array().unwrap_or(&vec![]) {
            let resource_id = resource_id.as_str().unwrap();
            let row = rows
                .iter_mut()
                .find(|row| row.0 == format!("resource.{resource_id}"))
                .unwrap();
            row.2 = format!("task.{task_id}");
        }
    }
    // Preserve the frozen resource values/capacities while making each native stage's pool distinct.
    rows.push((
        "resource.assessment".into(),
        "nurse".into(),
        "task.assessment".into(),
        "zone.assessment".into(),
        1,
        1,
        1,
    ));
    let first = &fixture_stages[0];
    let triage = StageWork {
        kind: StageKind::Triage,
        resource_id: format!("resource.{}", first["resource"].as_str().unwrap()),
        task_class_id: format!("task.{}", first["task"].as_str().unwrap()),
        duration_ticks: first["duration"].as_u64().unwrap(),
    };
    let assessment = StageWork {
        kind: StageKind::Assessment,
        resource_id: "resource.assessment".into(),
        task_class_id: "task.assessment".into(),
        duration_ticks: 1,
    };
    let mut stages = vec![triage, assessment];
    if id == "invalid.stage_disordered" || id == "invalid.stage_repeated" {
        stages = fixture_stages
            .iter()
            .enumerate()
            .map(|(index, stage)| StageWork {
                kind: match stage["name"].as_str().unwrap() {
                    "triage" => StageKind::Triage,
                    "assessment" => StageKind::Assessment,
                    _ => panic!("unmapped fixture stage"),
                },
                resource_id: if index == 0 {
                    "resource.triage"
                } else {
                    "resource.assessment"
                }
                .into(),
                task_class_id: if index == 0 {
                    "task.triage_task"
                } else {
                    "task.assessment"
                }
                .into(),
                duration_ticks: stage["duration"].as_u64().unwrap(),
            })
            .collect();
    }
    if id == "invalid.resource_unresolved" {
        stages[0].resource_id = format!("resource.{}", first["resource"].as_str().unwrap());
    }
    if id == "invalid.task_unresolved" {
        stages[0].task_class_id = format!("task.{}", first["task"].as_str().unwrap());
    }
    let patient = PathwayPatient {
        patient_id: inputs["patient_id"].as_str().unwrap().into(),
        arrival_tick: inputs["arrival_tick"].as_u64().unwrap(),
        mode: "synthetic".into(),
        acuity: inputs["acuity"].as_u64().unwrap() as u8,
        stages,
    };
    let mut patients = vec![patient];
    if inputs["patient_ids"].as_array().unwrap().len() > 1 {
        patients = inputs["patient_ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| {
                let mut p = patients[0].clone();
                p.patient_id = value.as_str().unwrap().into();
                p
            })
            .collect();
    }
    let horizon = inputs["horizon"].as_u64().unwrap();
    let mut profile = synthetic_profile(&rows);
    // Copy the fixture's capacity values/status exactly onto its native resource row.
    for (resource_id, raw) in resources {
        let resource = profile
            .resource_buckets
            .iter_mut()
            .find(|row| row.resource_id == format!("resource.{resource_id}"))
            .unwrap();
        resource.physical_count = raw.get("physical_count").and_then(Value::as_u64);
        resource.physical_status = if resource.physical_count.is_some() {
            KnowledgeStatus::Known
        } else {
            KnowledgeStatus::Unknown
        };
        resource.open_count = raw.get("open_count").and_then(Value::as_u64);
        resource.staffed_count = raw.get("staffed_count").and_then(Value::as_u64);
        resource.availability_status =
            if resource.open_count.is_some() && resource.staffed_count.is_some() {
                KnowledgeStatus::Known
            } else {
                KnowledgeStatus::Unknown
            };
    }
    if id == "invalid.eligibility_missing" {
        profile
            .eligibility
            .retain(|entry| entry.task_class_id != "task.triage_task");
    }
    if id == "invalid.resource_class_incompatible" {
        let required_class = first["required_class"].as_str().unwrap().to_owned();
        profile
            .task_classes
            .iter_mut()
            .filter(|task| task.task_class_id == "task.triage_task")
            .for_each(|task| task.required_resource_classes = vec![required_class.clone()]);
    }
    let config = PathwayConfig {
        version: 1,
        horizon_ticks: horizon,
        provenance: Provenance {
            class: "synthetic".into(),
            notes: "invalid fixture baseline".into(),
        },
        capacity_location: profile,
        patients,
    };
    let path = case["expected"]["offending_path"]
        .as_str()
        .unwrap()
        .to_owned();
    let (message, expected_category) = match id {
        "invalid.acuity_invalid" => ("acuity must be 1..5", "acuity_invalid"),
        "invalid.arrival_at_horizon" => ("arrival must be before horizon", "arrival_out_of_range"),
        "invalid.capacity_inconsistent" => {
            ("open_count exceeds physical_count", "capacity_inconsistent")
        }
        "invalid.capacity_open_zero"
        | "invalid.capacity_staffed_zero"
        | "invalid.capacity_unknown"
        | "invalid.capacity_zero" => (
            "known positive physical, open, and staffed capacity",
            "capacity_invalid",
        ),
        "invalid.capacity_staffed_over_open" => {
            ("staffed_count exceeds open_count", "capacity_inconsistent")
        }
        "invalid.duration_nonpositive" => ("stage duration must be positive", "duration_invalid"),
        "invalid.eligibility_missing" => (
            "no matching staff/task/zone eligibility",
            "eligibility_missing",
        ),
        "invalid.horizon_nonpositive" => ("horizon_ticks must be positive", "time_domain_invalid"),
        "invalid.patient_id_duplicate" => {
            ("patient IDs must be unique", "patient_identity_invalid")
        }
        "invalid.resource_class_incompatible" => {
            ("incompatible with resource", "resource_class_incompatible")
        }
        "invalid.resource_unresolved" => ("unknown resource", "resource_unresolved"),
        "invalid.stage_disordered" | "invalid.stage_repeated" => {
            ("stages must be triage", "stage_sequence_invalid")
        }
        "invalid.task_unresolved" => ("unknown task class", "task_unresolved"),
        _ => panic!("unmapped invalid fixture {id}"),
    };
    let expected_path = match id {
        "invalid.acuity_invalid" => "acuity",
        "invalid.arrival_at_horizon" => "arrival_tick",
        "invalid.capacity_inconsistent" | "invalid.capacity_open_zero" => {
            "resources.triage.open_count"
        }
        "invalid.capacity_staffed_over_open" | "invalid.capacity_staffed_zero" => {
            "resources.triage.staffed_count"
        }
        "invalid.capacity_unknown" | "invalid.capacity_zero" => "resources.triage.physical_count",
        "invalid.duration_nonpositive" => "stages[0].duration",
        "invalid.eligibility_missing" | "invalid.resource_unresolved" => "stages[0].resource",
        "invalid.horizon_nonpositive" => "horizon",
        "invalid.patient_id_duplicate" => "patient_ids",
        "invalid.resource_class_incompatible" => "stages[0].required_class",
        "invalid.stage_disordered" | "invalid.stage_repeated" => "stages",
        "invalid.task_unresolved" => "stages[0].task",
        _ => panic!("unmapped offending path {id}"),
    };
    assert_eq!(path, expected_path, "{id}: offending path metadata");
    assert_eq!(
        category, expected_category,
        "{id}: fixture diagnostic category"
    );
    (config, path, message.into())
}

#[test]
fn invalid_fixture_cases_reject_and_single_mutation_repairs() {
    let cases = fixtures(INVALID, "invalid");
    assert_eq!(cases.len(), 17);
    let mut ids = BTreeSet::new();
    for case in cases {
        let id = case["case_id"].as_str().unwrap().to_owned();
        assert!(ids.insert(id.clone()), "duplicate invalid case {id}");
        assert_eq!(case["runtime_owner"], "E1.2");
        let (mut malformed, path, expected_message) = invalid_native(&case);
        let mut legal_baseline = malformed.clone();
        repair_invalid(&id, &mut legal_baseline);
        careops_ed::pathway::run_pathway(&legal_baseline)
            .unwrap_or_else(|error| panic!("{id} legal baseline failed: {error}"));
        let error = careops_ed::pathway::run_pathway(&malformed).expect_err(&id);
        match error {
            PathwayError::InvalidInput(message) => assert!(
                message.contains(&expected_message),
                "{id} ({path}): {message:?}"
            ),
            other => panic!("{id} ({path}) returned non-input error: {other}"),
        }
        repair_invalid(&id, &mut malformed);
        careops_ed::pathway::run_pathway(&malformed)
            .unwrap_or_else(|error| panic!("{id} repair failed: {error}"));
    }
    assert_exact_ids(
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
    );
}

fn repair_invalid(id: &str, config: &mut PathwayConfig) {
    // Change only the offending field/group from the frozen fixture.
    match id {
        "invalid.acuity_invalid" => config.patients[0].acuity = 3,
        "invalid.arrival_at_horizon" => config.patients[0].arrival_tick = 1,
        "invalid.capacity_inconsistent" => {
            let r = config
                .capacity_location
                .resource_buckets
                .iter_mut()
                .find(|r| r.resource_id == "resource.triage")
                .unwrap();
            r.physical_count = Some(3);
            r.open_count = Some(2);
            r.staffed_count = Some(1);
        }
        "invalid.capacity_open_zero"
        | "invalid.capacity_staffed_over_open"
        | "invalid.capacity_staffed_zero"
        | "invalid.capacity_zero"
        | "invalid.capacity_unknown" => {
            let r = config
                .capacity_location
                .resource_buckets
                .iter_mut()
                .find(|r| r.resource_id == "resource.triage")
                .unwrap();
            r.physical_status = KnowledgeStatus::Known;
            r.physical_count = Some(1);
            r.availability_status = KnowledgeStatus::Known;
            r.open_count = Some(1);
            r.staffed_count = Some(1);
        }
        "invalid.duration_nonpositive" => config.patients[0].stages[0].duration_ticks = 2,
        "invalid.eligibility_missing" => {
            let resource = config
                .capacity_location
                .resource_buckets
                .iter()
                .find(|r| r.resource_id == "resource.triage")
                .unwrap();
            config
                .capacity_location
                .eligibility
                .push(careops_ed::Eligibility {
                    staff_role_id: "staff.synthetic".into(),
                    task_class_id: "task.triage_task".into(),
                    zone_id: resource.zone_id.clone(),
                    location_id: None,
                });
        }
        "invalid.horizon_nonpositive" => config.horizon_ticks = 10,
        "invalid.patient_id_duplicate" => config.patients[1].patient_id = "p2".into(),
        "invalid.resource_class_incompatible" => {
            config
                .capacity_location
                .resource_buckets
                .iter_mut()
                .find(|r| r.resource_id == "resource.triage")
                .unwrap()
                .resource_class = "nurse".into()
        }
        "invalid.resource_unresolved" => {
            config.patients[0].stages[0].resource_id = "resource.triage".into()
        }
        "invalid.stage_disordered" => {
            config.patients[0].stages[0].kind = StageKind::Triage;
            config.patients[0].stages[1].kind = StageKind::Assessment;
        }
        "invalid.stage_repeated" => config.patients[0].stages[1].kind = StageKind::Assessment,
        "invalid.task_unresolved" => {
            config.patients[0].stages[0].task_class_id = "task.triage_task".into()
        }
        _ => panic!("unmapped repair control {id}"),
    }
}
