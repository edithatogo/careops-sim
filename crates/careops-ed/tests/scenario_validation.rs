use careops_ed::parse_scenario_json;
use serde_json::{json, Value};

fn valid_scenario() -> Value {
    json!({
        "schema_version": 1,
        "scenario_id": "e0-one-patient",
        "seed": 7,
        "time_unit": "tick",
        "horizon_ticks": 10,
        "provenance": {
            "class": "synthetic",
            "notes": "Invented E0 smoke fixture; not operational or clinical data."
        },
        "capacity_location": {
            "schema_version": 3,
            "status": "proposed",
            "capacity": {
                "unit": "treatment_space",
                "bucket_completeness": "complete",
                "physical_status": "known",
                "physical_count": 1,
                "availability_status": "known",
                "open_count": 1,
                "staffed_count": 1,
                "equipment": []
            },
            "zones": [{"zone_id": "zone.synthetic", "name": "synthetic zone"}],
            "locations": [{
                "location_id": "location.synthetic",
                "name": "synthetic location",
                "zone_id": "zone.synthetic"
            }],
            "resource_buckets": [{
                "resource_id": "resource.synthetic",
                "resource_class": "synthetic_slot",
                "location_id": "location.synthetic",
                "zone_id": "zone.synthetic",
                "physical_status": "known",
                "physical_count": 1,
                "availability_status": "known",
                "open_count": 1,
                "staffed_count": 1,
                "capabilities": []
            }],
            "task_classes": [{
                "task_class_id": "task.synthetic",
                "name": "synthetic work",
                "required_resource_classes": ["synthetic_slot"]
            }],
            "staff_roles": [{
                "staff_role_id": "staff.synthetic",
                "name": "synthetic role",
                "counts": [{
                    "basis": "present",
                    "status": "known",
                    "count": 1,
                    "reference": "synthetic fixture"
                }]
            }],
            "eligibility": [{
                "staff_role_id": "staff.synthetic",
                "task_class_id": "task.synthetic",
                "zone_id": "zone.synthetic",
                "location_id": "location.synthetic"
            }],
            "routes": [{
                "from_location_id": "location.synthetic",
                "to_location_id": "location.synthetic",
                "distance": {"value": 0, "unit": "m"}
            }]
        },
        "patients": [{
            "patient_id": "synthetic-patient-1",
            "arrival_tick": 1,
            "work_ticks": 2,
            "resource_id": "resource.synthetic",
            "task_class_id": "task.synthetic"
        }]
    })
}

fn rejects(mut scenario: Value, mutate: impl FnOnce(&mut Value)) {
    mutate(&mut scenario);
    let error = parse_scenario_json(&scenario.to_string()).unwrap_err();
    assert!(!error.to_string().trim().is_empty());
}

#[test]
fn rejects_unsupported_time_unit() {
    rejects(valid_scenario(), |value| {
        value["time_unit"] = json!("seconds")
    });
}

#[test]
fn rejects_zero_horizon_and_zero_work() {
    rejects(valid_scenario(), |value| value["horizon_ticks"] = json!(0));
    rejects(valid_scenario(), |value| {
        value["patients"][0]["work_ticks"] = json!(0)
    });
}

#[test]
fn rejects_missing_required_inputs_and_extra_fields() {
    rejects(valid_scenario(), |value| {
        value.as_object_mut().unwrap().remove("seed");
    });
    rejects(valid_scenario(), |value| {
        value["unreviewed_policy"] = json!("default")
    });
    rejects(valid_scenario(), |value| value["patients"] = json!([]));
}

#[test]
fn rejects_unsupported_schema_version_with_clear_diagnostic() {
    let mut scenario = valid_scenario();
    scenario["schema_version"] = json!(999);
    let error = parse_scenario_json(&scenario.to_string()).unwrap_err();
    assert!(error.to_string().contains("schema_version must be 1"));
}

#[test]
fn rejects_nonpositive_capacity_and_unresolved_references() {
    rejects(valid_scenario(), |value| {
        value["capacity_location"]["resource_buckets"][0]["open_count"] = json!(0)
    });
    rejects(valid_scenario(), |value| {
        value["patients"][0]["resource_id"] = json!("missing-resource")
    });
    rejects(valid_scenario(), |value| {
        value["capacity_location"]["locations"][0]["zone_id"] = json!("missing-zone")
    });
}

#[test]
fn rejects_bad_route_units_and_unknown_capacity() {
    rejects(valid_scenario(), |value| {
        value["capacity_location"]["routes"][0]["distance"]["unit"] = json!("km")
    });
    rejects(valid_scenario(), |value| {
        value["capacity_location"]["resource_buckets"][0]["availability_status"] = json!("unknown");
        value["capacity_location"]["resource_buckets"][0]["open_count"] = Value::Null;
        value["capacity_location"]["resource_buckets"][0]["staffed_count"] = Value::Null;
        value["capacity_location"]["resource_buckets"][0]["availability_reason"] =
            json!("not established by synthetic fixture");
    });
}

#[test]
fn validates_optional_p0_equipment_status_and_reason() {
    let mut scenario = valid_scenario();
    scenario["capacity_location"]["capacity"]["equipment"] = json!([{
        "capability": "monitoring",
        "status": "unknown",
        "reason": "not observed in synthetic fixture"
    }]);
    assert!(parse_scenario_json(&scenario.to_string()).is_ok());

    rejects(valid_scenario(), |value| {
        value["capacity_location"]["capacity"]["equipment"] = json!([{
            "capability": "monitoring",
            "status": "unknown"
        }]);
    });
    rejects(valid_scenario(), |value| {
        value["capacity_location"]["capacity"]["equipment"] = json!([{
            "capability": "monitoring",
            "status": "available",
            "reason": "contradictory reason"
        }]);
    });
}
