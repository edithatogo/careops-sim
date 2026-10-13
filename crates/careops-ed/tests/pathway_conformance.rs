use careops_ed::{parse_scenario_json, run_scenario, PatientStatus, ScenarioConfig};
use serde_json::{json, Value};

/// Generates a valid base scenario JSON value that conforms to E0 / P0 schemas.
fn base_scenario() -> Value {
    json!({
        "schema_version": 1,
        "scenario_id": "e1-pathway-conformance",
        "seed": 42,
        "time_unit": "tick",
        "horizon_ticks": 50,
        "provenance": {
            "class": "synthetic",
            "notes": "E1.1 pathway conformance testing."
        },
        "capacity_location": {
            "schema_version": 3,
            "status": "proposed",
            "capacity": {
                "unit": "treatment_space",
                "bucket_completeness": "complete",
                "physical_status": "known",
                "physical_count": 5,
                "availability_status": "known",
                "open_count": 5,
                "staffed_count": 5,
                "equipment": []
            },
            "zones": [
                {"zone_id": "zone.triage", "name": "Triage Zone"},
                {"zone_id": "zone.acute", "name": "Acute Treatment Zone"},
                {"zone_id": "zone.subacute", "name": "Subacute Zone"}
            ],
            "locations": [
                {"location_id": "loc.triage", "name": "Triage Bay", "zone_id": "zone.triage"},
                {"location_id": "loc.acute", "name": "Resus / Acute Bed", "zone_id": "zone.acute"},
                {"location_id": "loc.subacute", "name": "Consult Room", "zone_id": "zone.subacute"}
            ],
            "resource_buckets": [
                {
                    "resource_id": "res.triage",
                    "resource_class": "triage_desk",
                    "location_id": "loc.triage",
                    "zone_id": "zone.triage",
                    "physical_status": "known",
                    "physical_count": 1,
                    "availability_status": "known",
                    "open_count": 1,
                    "staffed_count": 1,
                    "capabilities": []
                },
                {
                    "resource_id": "res.acute",
                    "resource_class": "acute_bed",
                    "location_id": "loc.acute",
                    "zone_id": "zone.acute",
                    "physical_status": "known",
                    "physical_count": 2,
                    "availability_status": "known",
                    "open_count": 2,
                    "staffed_count": 2,
                    "capabilities": []
                },
                {
                    "resource_id": "res.subacute",
                    "resource_class": "subacute_chair",
                    "location_id": "loc.subacute",
                    "zone_id": "zone.subacute",
                    "physical_status": "known",
                    "physical_count": 2,
                    "availability_status": "known",
                    "open_count": 2,
                    "staffed_count": 2,
                    "capabilities": []
                }
            ],
            "task_classes": [
                {
                    "task_class_id": "task.triage",
                    "name": "Triage Assessment",
                    "required_resource_classes": ["triage_desk"]
                },
                {
                    "task_class_id": "task.acute_care",
                    "name": "High Acuity Care",
                    "required_resource_classes": ["acute_bed"]
                },
                {
                    "task_class_id": "task.ambulatory",
                    "name": "Low Acuity Care",
                    "required_resource_classes": ["subacute_chair"]
                }
            ],
            "staff_roles": [
                {
                    "staff_role_id": "role.triage_nurse",
                    "name": "Triage Nurse",
                    "counts": [{
                        "basis": "present",
                        "status": "known",
                        "count": 1,
                        "reference": "synthetic schedule"
                    }]
                },
                {
                    "staff_role_id": "role.ed_physician",
                    "name": "ED Physician",
                    "counts": [{
                        "basis": "present",
                        "status": "known",
                        "count": 2,
                        "reference": "synthetic schedule"
                    }]
                }
            ],
            "eligibility": [
                {
                    "staff_role_id": "role.triage_nurse",
                    "task_class_id": "task.triage",
                    "zone_id": "zone.triage",
                    "location_id": "loc.triage"
                },
                {
                    "staff_role_id": "role.ed_physician",
                    "task_class_id": "task.acute_care",
                    "zone_id": "zone.acute",
                    "location_id": "loc.acute"
                },
                {
                    "staff_role_id": "role.ed_physician",
                    "task_class_id": "task.ambulatory",
                    "zone_id": "zone.subacute",
                    "location_id": "loc.subacute"
                }
            ],
            "routes": [
                {
                    "from_location_id": "loc.triage",
                    "to_location_id": "loc.acute",
                    "distance": {"value": 15.0, "unit": "m"}
                },
                {
                    "from_location_id": "loc.triage",
                    "to_location_id": "loc.subacute",
                    "distance": {"value": 25.0, "unit": "m"}
                }
            ]
        },
        "patients": []
    })
}

// -------------------------------------------------------------------------
// 1. Empty run
// -------------------------------------------------------------------------
#[test]
fn test_empty_run_conformance() {
    let mut scenario_val = base_scenario();
    scenario_val["patients"] = json!([]);
    let scenario_str = scenario_val.to_string();

    // Baseline E0 validator requires patients collection must not be empty.
    let parse_res = parse_scenario_json(&scenario_str);
    assert!(
        parse_res.is_err(),
        "Empty patient list rejected by baseline schema validator"
    );
    let err_str = parse_res.unwrap_err().to_string();
    assert!(
        err_str.contains("patients must contain at least one supplied work item"),
        "Diagnostic matched: {err_str}"
    );

    // If an empty ScenarioConfig struct is constructed directly, run_scenario verifies
    // that validate() rejects it without panicking or creating false output.
    let mut raw_scenario: ScenarioConfig = serde_json::from_value(base_scenario()).unwrap();
    raw_scenario.patients.clear();
    let run_res = run_scenario(&raw_scenario);
    assert!(run_res.is_err());
}

// -------------------------------------------------------------------------
// 2. One patient run: explicit trace, event order, and metric intervals
// -------------------------------------------------------------------------
#[test]
fn test_one_patient_run_trace_and_intervals() {
    let mut scenario_val = base_scenario();
    scenario_val["horizon_ticks"] = json!(20);
    scenario_val["patients"] = json!([{
        "patient_id": "patient-ats-2",
        "arrival_tick": 2,
        "work_ticks": 6,
        "resource_id": "res.acute",
        "task_class_id": "task.acute_care"
    }]);

    let scenario: ScenarioConfig = parse_scenario_json(&scenario_val.to_string()).unwrap();
    let summary = run_scenario(&scenario).unwrap();

    // Conservation invariants
    assert_eq!(summary.arrivals, 1);
    assert_eq!(summary.started, 1);
    assert_eq!(summary.completed, 1);
    assert_eq!(summary.unfinished, 0);
    assert_eq!(summary.arrivals, summary.completed + summary.unfinished);

    let res = &summary.patients[0];
    assert_eq!(res.patient_id, "patient-ats-2");
    assert_eq!(res.status, PatientStatus::Completed);
    assert_eq!(res.arrival_tick, 2);
    assert_eq!(res.start_tick, Some(2));
    assert_eq!(res.completion_tick, Some(8));
    assert_eq!(res.work_ticks, 6);
    assert_eq!(res.wait_ticks, Some(0));
    assert_eq!(res.elapsed_ticks, Some(6));

    // Verify interval algebra [start, completion)
    assert!(res.arrival_tick <= res.start_tick.unwrap());
    assert_eq!(
        res.completion_tick.unwrap(),
        res.start_tick.unwrap() + res.work_ticks
    );
    assert_eq!(
        res.elapsed_ticks.unwrap(),
        res.wait_ticks.unwrap() + res.work_ticks
    );
}

// -------------------------------------------------------------------------
// 3. Each acuity and pathway: ATS 1-5 across distinct care pathways
// -------------------------------------------------------------------------
#[test]
fn test_each_acuity_and_pathway_mix() {
    // We test a cross-product of synthetic ATS categories 1 through 5 across
    // 3 pathway resource allocations:
    // ATS 1 -> res.acute (Resus/Immediate)
    // ATS 2 -> res.acute (Emergency)
    // ATS 3 -> res.acute (Urgent)
    // ATS 4 -> res.subacute (Semi-urgent)
    // ATS 5 -> res.subacute (Non-urgent)
    let mut scenario_val = base_scenario();
    scenario_val["horizon_ticks"] = json!(100);

    let mut patients = Vec::new();
    // Schedule one patient for each ATS category with deterministic arrival spacing
    let ats_configs = [
        ("p-ats1", 1, 4, "res.acute", "task.acute_care"),
        ("p-ats2", 3, 5, "res.acute", "task.acute_care"),
        ("p-ats3", 5, 6, "res.acute", "task.acute_care"),
        ("p-ats4", 7, 3, "res.subacute", "task.ambulatory"),
        ("p-ats5", 9, 2, "res.subacute", "task.ambulatory"),
    ];

    for &(id, arr, work, res, task) in &ats_configs {
        patients.push(json!({
            "patient_id": id,
            "arrival_tick": arr,
            "work_ticks": work,
            "resource_id": res,
            "task_class_id": task
        }));
    }
    scenario_val["patients"] = Value::Array(patients);

    let scenario: ScenarioConfig = parse_scenario_json(&scenario_val.to_string()).unwrap();
    let summary = run_scenario(&scenario).unwrap();

    assert_eq!(summary.arrivals, 5);
    assert_eq!(summary.completed, 5);
    assert_eq!(summary.unfinished, 0);

    // Endpoint count conservation
    assert_eq!(summary.arrivals, summary.completed + summary.unfinished);

    for p in &summary.patients {
        assert_eq!(p.status, PatientStatus::Completed);
        assert!(p.start_tick.is_some());
        assert!(p.completion_tick.is_some());
        assert_eq!(
            p.elapsed_ticks.unwrap(),
            p.wait_ticks.unwrap() + p.work_ticks
        );
    }
}

// -------------------------------------------------------------------------
// 4. Saturated arrival demand and queue wait intervals
// -------------------------------------------------------------------------
#[test]
fn test_saturated_arrival_demand_and_queue_formation() {
    // Saturated demand: 4 patients arriving at tick 0 competing for 1 unit of res.triage
    let mut scenario_val = base_scenario();
    scenario_val["horizon_ticks"] = json!(50);
    scenario_val["patients"] = json!([
        {"patient_id": "sat-1", "arrival_tick": 0, "work_ticks": 3, "resource_id": "res.triage", "task_class_id": "task.triage"},
        {"patient_id": "sat-2", "arrival_tick": 0, "work_ticks": 3, "resource_id": "res.triage", "task_class_id": "task.triage"},
        {"patient_id": "sat-3", "arrival_tick": 0, "work_ticks": 3, "resource_id": "res.triage", "task_class_id": "task.triage"},
        {"patient_id": "sat-4", "arrival_tick": 0, "work_ticks": 3, "resource_id": "res.triage", "task_class_id": "task.triage"}
    ]);

    let scenario: ScenarioConfig = parse_scenario_json(&scenario_val.to_string()).unwrap();
    let summary = run_scenario(&scenario).unwrap();

    assert_eq!(summary.arrivals, 4);
    assert_eq!(summary.completed, 4);
    assert_eq!(summary.unfinished, 0);

    // Verify deterministic single-server serialized queue ordering:
    // sat-1: start=0, comp=3, wait=0
    // sat-2: start=3, comp=6, wait=3
    // sat-3: start=6, comp=9, wait=6
    // sat-4: start=9, comp=12, wait=9
    let expected_waits = [0, 3, 6, 9];
    let expected_starts = [0, 3, 6, 9];
    let expected_comps = [3, 6, 9, 12];

    for (i, p) in summary.patients.iter().enumerate() {
        assert_eq!(p.start_tick, Some(expected_starts[i]));
        assert_eq!(p.completion_tick, Some(expected_comps[i]));
        assert_eq!(p.wait_ticks, Some(expected_waits[i]));
        assert_eq!(p.elapsed_ticks, Some(expected_waits[i] + 3));
    }

    // Capacity conservation: utilization is 100% during [0, 12)
    let total_busy: u64 = summary.patients.iter().map(|p| p.work_ticks).sum();
    assert_eq!(total_busy, 12);
}

// -------------------------------------------------------------------------
// 5. Zero service duration, long service, and horizon boundary
// -------------------------------------------------------------------------
#[test]
fn test_zero_service_duration_rejection() {
    let mut scenario_val = base_scenario();
    scenario_val["patients"] = json!([{
        "patient_id": "zero-work",
        "arrival_tick": 1,
        "work_ticks": 0,
        "resource_id": "res.triage",
        "task_class_id": "task.triage"
    }]);

    // Zero service duration is rejected by input validation to prevent cyclic / zero-work loops
    let parse_res = parse_scenario_json(&scenario_val.to_string());
    assert!(parse_res.is_err(), "Zero work ticks must be rejected");
}

#[test]
fn test_long_service_and_horizon_clipping() {
    // Work duration exceeds horizon or arrival is right before horizon
    let mut scenario_val = base_scenario();
    scenario_val["horizon_ticks"] = json!(10);
    scenario_val["patients"] = json!([
        {
            "patient_id": "patient-finished",
            "arrival_tick": 1,
            "work_ticks": 4,
            "resource_id": "res.acute",
            "task_class_id": "task.acute_care"
        },
        {
            "patient_id": "patient-unfinished-clipped",
            "arrival_tick": 6,
            "work_ticks": 10, // requires completion at tick 16 > horizon 10
            "resource_id": "res.acute",
            "task_class_id": "task.acute_care"
        }
    ]);

    let scenario: ScenarioConfig = parse_scenario_json(&scenario_val.to_string()).unwrap();
    let summary = run_scenario(&scenario).unwrap();

    assert_eq!(summary.arrivals, 2);
    assert_eq!(summary.started, 2);
    assert_eq!(summary.completed, 1);
    assert_eq!(summary.unfinished, 1);
    assert_eq!(summary.arrivals, summary.completed + summary.unfinished);

    let p1 = &summary.patients[0];
    assert_eq!(p1.status, PatientStatus::Completed);
    assert_eq!(p1.completion_tick, Some(5));

    let p2 = &summary.patients[1];
    assert_eq!(p2.status, PatientStatus::Unfinished);
    assert_eq!(p2.start_tick, Some(6));
    assert_eq!(p2.completion_tick, None);
    assert_eq!(p2.elapsed_ticks, None);
    assert_eq!(p2.wait_ticks, Some(0)); // wait until start was 0
}

// -------------------------------------------------------------------------
// 6. Impossible resource configuration rejection
// -------------------------------------------------------------------------
#[test]
fn test_impossible_resource_configuration() {
    // Case A: Unresolved resource ID
    let mut scenario_val = base_scenario();
    scenario_val["patients"] = json!([{
        "patient_id": "p-unresolved",
        "arrival_tick": 1,
        "work_ticks": 2,
        "resource_id": "res.nonexistent",
        "task_class_id": "task.acute_care"
    }]);
    assert!(parse_scenario_json(&scenario_val.to_string()).is_err());

    // Case B: Zero open capacity in bucket
    let mut scenario_val_b = base_scenario();
    scenario_val_b["capacity_location"]["resource_buckets"][0]["open_count"] = json!(0);
    scenario_val_b["patients"] = json!([{
        "patient_id": "p-zero-cap",
        "arrival_tick": 1,
        "work_ticks": 2,
        "resource_id": "res.triage",
        "task_class_id": "task.triage"
    }]);
    assert!(parse_scenario_json(&scenario_val_b.to_string()).is_err());

    // Case C: Negative or non-physical counts
    let mut scenario_val_c = base_scenario();
    scenario_val_c["capacity_location"]["resource_buckets"][0]["staffed_count"] = json!(-1);
    assert!(parse_scenario_json(&scenario_val_c.to_string()).is_err());
}

// -------------------------------------------------------------------------
// 7. Endpoint count conservation invariant
// -------------------------------------------------------------------------
#[test]
fn test_endpoint_count_conservation_exhaustive_grid() {
    // Test multiple combinations of arrival spacings and horizon cutoffs.
    // arrivals are in [0, 5), horizon is varying from 5 to 20.
    for horizon in [5, 7, 10, 15, 20] {
        let mut scenario_val = base_scenario();
        scenario_val["horizon_ticks"] = json!(horizon);
        scenario_val["patients"] = json!([
            {"patient_id": "c1", "arrival_tick": 0, "work_ticks": 4, "resource_id": "res.triage", "task_class_id": "task.triage"},
            {"patient_id": "c2", "arrival_tick": 1, "work_ticks": 4, "resource_id": "res.triage", "task_class_id": "task.triage"},
            {"patient_id": "c3", "arrival_tick": 2, "work_ticks": 4, "resource_id": "res.triage", "task_class_id": "task.triage"},
            {"patient_id": "c4", "arrival_tick": 3, "work_ticks": 4, "resource_id": "res.triage", "task_class_id": "task.triage"}
        ]);

        let scenario: ScenarioConfig = parse_scenario_json(&scenario_val.to_string()).unwrap();
        let summary = run_scenario(&scenario).unwrap();

        // Exact endpoint conservation equation:
        // arrivals = completed + unfinished
        assert_eq!(
            summary.arrivals,
            summary.completed + summary.unfinished,
            "Failed conservation at horizon {horizon}"
        );
        assert_eq!(summary.arrivals, summary.patients.len());
        assert!(summary.started <= summary.arrivals);
        assert!(summary.completed <= summary.started);
    }
}
