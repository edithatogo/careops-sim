use std::collections::{BTreeMap, BTreeSet};

use careops_ed::arrivals::{generate_arrivals, ArrivalBin, ArrivalConfig, ArrivalModeCounts};
use careops_ed::pathway::{
    run_pathway, PathwayConfig, PathwayPatient, StageKind, StageWork,
};
use careops_ed::{
    AggregateCapacity, CapacityLocationProfile, Eligibility, KnowledgeStatus, Location,
    Provenance, ResourceBucket, StaffCount, StaffRole, TaskClass, Zone,
};

#[test]
fn test_seeded_time_varying_arrivals_determinism_and_stability() {
    let config1 = ArrivalConfig {
        version: 1,
        seed: 42,
        horizon_ticks: 20,
        provenance: "synthetic_tick_fixture".into(),
        bins: vec![
            ArrivalBin {
                start_tick: 0,
                end_tick: 10,
                modes: vec![ArrivalModeCounts {
                    mode: "walk_in".into(),
                    total: 3,
                    acuity_counts: [0, 1, 1, 1, 0],
                }],
            },
            ArrivalBin {
                start_tick: 10,
                end_tick: 20,
                modes: vec![ArrivalModeCounts {
                    mode: "ambulance".into(),
                    total: 2,
                    acuity_counts: [1, 1, 0, 0, 0],
                }],
            },
        ],
    };

    let arrivals_a = generate_arrivals(&config1).expect("valid arrivals generation");
    let arrivals_b = generate_arrivals(&config1).expect("reproducible generation");

    assert_eq!(arrivals_a.len(), 5);
    assert_eq!(
        arrivals_a,
        arrivals_b,
        "Seeded arrivals must be byte-for-byte deterministic"
    );

    for arr in &arrivals_a {
        assert!(arr.patient_id.starts_with("arrival-v1-"));
        assert!(arr.arrival_tick < 20);
    }
}

#[test]
fn test_clinical_states_pathway_execution_and_queue_isolation() {
    let rows = [
        (
            "res.triage".to_string(),
            "triage_desk".to_string(),
            "task.triage".to_string(),
            "zone.triage".to_string(),
            1u64,
            1u64,
            1u64,
        ),
        (
            "res.assessment".to_string(),
            "assessment_desk".to_string(),
            "task.assessment".to_string(),
            "zone.assessment".to_string(),
            1u64,
            1u64,
            1u64,
        ),
    ];

    let mut zones = BTreeMap::new();
    let mut locations = BTreeMap::new();
    let mut resources = BTreeMap::new();
    let mut tasks = BTreeMap::new();
    let mut eligibilities = BTreeSet::new();
    for (res_id, class, task_id, zone_id, phys, open, staffed) in &rows {
        let location = format!("loc.{zone_id}");
        zones.insert(
            zone_id.clone(),
            Zone {
                zone_id: zone_id.clone(),
                name: zone_id.clone(),
            },
        );
        locations.insert(
            location.clone(),
            Location {
                location_id: location.clone(),
                name: location.clone(),
                zone_id: zone_id.clone(),
            },
        );
        resources.insert(
            res_id.clone(),
            ResourceBucket {
                resource_id: res_id.clone(),
                resource_class: class.clone(),
                location_id: location.clone(),
                zone_id: zone_id.clone(),
                pool_group_id: None,
                physical_status: KnowledgeStatus::Known,
                physical_count: Some(*phys),
                availability_status: KnowledgeStatus::Known,
                open_count: Some(*open),
                staffed_count: Some(*staffed),
                availability_reason: None,
                capabilities: vec![],
            },
        );
        tasks.entry(task_id.clone()).or_insert_with(|| TaskClass {
            task_class_id: task_id.clone(),
            name: task_id.clone(),
            required_resource_classes: vec![class.clone()],
        });
        eligibilities.insert((
            "role.nurse".to_string(),
            task_id.clone(),
            zone_id.clone(),
            None,
        ));
    }

    let profile = CapacityLocationProfile {
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
        staff_roles: vec![StaffRole {
            staff_role_id: "role.nurse".into(),
            name: "Nurse".into(),
            counts: vec![StaffCount {
                basis: "present".into(),
                status: KnowledgeStatus::Known,
                count: Some(1),
                reference: "synthetic".into(),
                reason: None,
            }],
        }],
        eligibility: eligibilities
            .into_iter()
            .map(
                |(staff_role_id, task_class_id, zone_id, location_id)| Eligibility {
                    staff_role_id,
                    task_class_id,
                    zone_id,
                    location_id,
                },
            )
            .collect(),
        routes: vec![],
    };

    let triage_assessment = |triage_ticks: u64| {
        vec![
            StageWork {
                kind: StageKind::Triage,
                resource_id: "res.triage".into(),
                task_class_id: "task.triage".into(),
                duration_ticks: triage_ticks,
            },
            StageWork {
                kind: StageKind::Assessment,
                resource_id: "res.assessment".into(),
                task_class_id: "task.assessment".into(),
                duration_ticks: 1,
            },
        ]
    };

    let config = PathwayConfig {
        version: 1,
        horizon_ticks: 30,
        provenance: Provenance {
            class: "synthetic".into(),
            notes: "E1.2 states test".into(),
        },
        capacity_location: profile,
        patients: vec![
            PathwayPatient {
                patient_id: "pat-1".into(),
                arrival_tick: 0,
                mode: "walk_in".into(),
                acuity: 3,
                stages: triage_assessment(5),
            },
            PathwayPatient {
                patient_id: "pat-2".into(),
                arrival_tick: 0,
                mode: "walk_in".into(),
                acuity: 3,
                stages: triage_assessment(5),
            },
        ],
    };

    let results = run_pathway(&config).expect("pathway run succeeds");
    assert_eq!(results.len(), 2);

    let by_id: std::collections::BTreeMap<_, _> = results
        .iter()
        .map(|result| (result.patient_id.as_str(), result))
        .collect();

    let ep1 = &by_id["pat-1"].episodes[0];
    assert_eq!(ep1.queue_entry_tick, 0);
    assert_eq!(ep1.start_tick, Some(0));
    assert_eq!(ep1.completion_tick, Some(5));
    assert_eq!(ep1.observed_work_ticks, 5);

    let ep2 = &by_id["pat-2"].episodes[0];
    assert_eq!(ep2.queue_entry_tick, 0);
    assert_eq!(ep2.start_tick, Some(5));
    assert_eq!(ep2.completion_tick, Some(10));
    assert_eq!(ep2.observed_work_ticks, 5);
}
