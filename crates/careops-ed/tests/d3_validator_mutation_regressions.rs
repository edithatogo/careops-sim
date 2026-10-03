//! Synthetic one-fault regression oracles for measured E0/P0 validator gaps.
use careops_ed::parse_scenario_json;
use serde_json::{json, Value};

fn scenario() -> Value {
    let mut input: Value =
        serde_json::from_str(include_str!("../examples/one_patient.json")).unwrap();
    // The accepted example has no routes, so include one valid synthetic route
    // in the common baseline before route tests mutate a single property.
    input["capacity_location"]["routes"] = json!([{
        "from_location_id": "location.synthetic",
        "to_location_id": "location.synthetic",
        "distance": {"value": 0, "unit": "m"}
    }]);
    input
}

fn rejects_with(change: impl FnOnce(&mut Value), expected: &str) {
    let mut input = scenario();
    change(&mut input);
    let error =
        parse_scenario_json(&input.to_string()).expect_err("invalid synthetic input accepted");
    assert!(
        error.to_string().contains(expected),
        "expected diagnostic {expected:?}, got {error}"
    );
}

fn accepts(change: impl FnOnce(&mut Value)) {
    let mut input = scenario();
    change(&mut input);
    parse_scenario_json(&input.to_string()).expect("valid synthetic input rejected");
}

fn first_staff_count(value: &mut Value) -> &mut Value {
    &mut value["capacity_location"]["staff_roles"][0]["counts"][0]
}

#[test]
fn profile_metadata_and_aggregate_shape_are_checked_one_field_at_a_time() {
    parse_scenario_json(&scenario().to_string()).unwrap();
    rejects_with(
        |v| v["capacity_location"]["schema_version"] = json!(2),
        "capacity_location must use proposed P0 schema version 3",
    );
    rejects_with(
        |v| v["capacity_location"]["status"] = json!("observed"),
        "capacity_location must use proposed P0 schema version 3",
    );
    rejects_with(
        |v| v["capacity_location"]["capacity"]["unit"] = json!("bed"),
        "capacity unit/completeness does not match P0 contract",
    );
    rejects_with(
        |v| v["capacity_location"]["capacity"]["bucket_completeness"] = json!("partial"),
        "capacity unit/completeness does not match P0 contract",
    );
}

#[test]
fn resource_known_availability_requires_both_counts_and_no_reason() {
    rejects_with(
        |v| v["capacity_location"]["resource_buckets"][0]["open_count"] = Value::Null,
        "resource availability known status requires open/staffed counts and no reason",
    );
    rejects_with(
        |v| v["capacity_location"]["resource_buckets"][0]["staffed_count"] = Value::Null,
        "resource availability known status requires open/staffed counts and no reason",
    );
    rejects_with(
        |v| {
            v["capacity_location"]["resource_buckets"][0]["availability_reason"] =
                json!("contradicts known status")
        },
        "resource availability known status requires open/staffed counts and no reason",
    );
}

#[test]
fn resource_unknown_availability_requires_null_counts_and_a_reason() {
    rejects_with(
        |v| {
            v["capacity_location"]["resource_buckets"][0]["availability_status"] = json!("unknown");
            v["capacity_location"]["resource_buckets"][0]["staffed_count"] = Value::Null;
            v["capacity_location"]["resource_buckets"][0]["availability_reason"] =
                json!("not established by this synthetic fixture");
        },
        "resource availability unknown status requires null counts and a reason",
    );
    rejects_with(
        |v| {
            v["capacity_location"]["resource_buckets"][0]["availability_status"] = json!("unknown");
            v["capacity_location"]["resource_buckets"][0]["open_count"] = Value::Null;
            v["capacity_location"]["resource_buckets"][0]["availability_reason"] =
                json!("not established by this synthetic fixture");
        },
        "resource availability unknown status requires null counts and a reason",
    );
    rejects_with(
        |v| {
            v["capacity_location"]["resource_buckets"][0]["availability_status"] = json!("unknown");
            v["capacity_location"]["resource_buckets"][0]["open_count"] = Value::Null;
            v["capacity_location"]["resource_buckets"][0]["staffed_count"] = Value::Null;
            v["capacity_location"]["resource_buckets"][0]
                .as_object_mut()
                .unwrap()
                .remove("availability_reason");
        },
        "resource availability unknown status requires null counts and a reason",
    );
    rejects_with(
        |v| {
            v["capacity_location"]["resource_buckets"][0]["availability_status"] = json!("unknown");
            v["capacity_location"]["resource_buckets"][0]["open_count"] = Value::Null;
            v["capacity_location"]["resource_buckets"][0]["staffed_count"] = Value::Null;
            v["capacity_location"]["resource_buckets"][0]["availability_reason"] = json!("  ");
        },
        "resource availability unknown status requires null counts and a reason",
    );
    rejects_with(
        |v| {
            v["capacity_location"]["resource_buckets"][0]["availability_status"] = json!("unknown");
            v["capacity_location"]["resource_buckets"][0]["open_count"] = Value::Null;
            v["capacity_location"]["resource_buckets"][0]["staffed_count"] = Value::Null;
            v["capacity_location"]["resource_buckets"][0]["availability_reason"] =
                json!("not established by this synthetic fixture");
        },
        "resource 'resource.synthetic' needs positive known open and staffed counts to run",
    );
}

#[test]
fn resource_counts_enforce_positive_and_physical_boundaries() {
    rejects_with(
        |v| v["capacity_location"]["resource_buckets"][0]["open_count"] = json!(0),
        "resource 'resource.synthetic' needs positive known open and staffed counts to run",
    );
    rejects_with(
        |v| v["capacity_location"]["resource_buckets"][0]["staffed_count"] = json!(0),
        "resource 'resource.synthetic' needs positive known open and staffed counts to run",
    );
    rejects_with(
        |v| v["capacity_location"]["resource_buckets"][0]["physical_count"] = Value::Null,
        "resource physical count is marked known but has no count",
    );
    rejects_with(
        |v| {
            v["capacity_location"]["resource_buckets"][0]["physical_status"] = json!("unknown");
        },
        "resource physical count is marked unknown but supplies a count",
    );
    rejects_with(
        |v| {
            v["capacity_location"]["resource_buckets"][0]["physical_count"] = json!(1);
            v["capacity_location"]["resource_buckets"][0]["open_count"] = json!(2);
        },
        "resource 'resource.synthetic' open_count exceeds physical_count",
    );
    rejects_with(
        |v| {
            v["capacity_location"]["resource_buckets"][0]["physical_count"] = json!(2);
            v["capacity_location"]["resource_buckets"][0]["open_count"] = json!(1);
            v["capacity_location"]["resource_buckets"][0]["staffed_count"] = json!(2);
        },
        "resource 'resource.synthetic' staffed_count exceeds open_count",
    );
    accepts(|v| {
        v["capacity_location"]["resource_buckets"][0]["physical_count"] = json!(2);
        v["capacity_location"]["resource_buckets"][0]["open_count"] = json!(2);
        v["capacity_location"]["resource_buckets"][0]["staffed_count"] = json!(2);
    });
}

#[test]
fn aggregate_physical_and_availability_counts_have_distinct_status_rules() {
    rejects_with(
        |v| v["capacity_location"]["capacity"]["physical_count"] = Value::Null,
        "aggregate physical count is marked known but has no count",
    );
    rejects_with(
        |v| v["capacity_location"]["capacity"]["physical_status"] = json!("unknown"),
        "aggregate physical count is marked unknown but supplies a count",
    );
    rejects_with(
        |v| {
            v["capacity_location"]["capacity"]["availability_reason"] =
                json!("contradicts known status")
        },
        "aggregate availability known status requires open/staffed counts and no reason",
    );
    rejects_with(
        |v| v["capacity_location"]["capacity"]["open_count"] = Value::Null,
        "aggregate availability known status requires open/staffed counts and no reason",
    );
    rejects_with(
        |v| v["capacity_location"]["capacity"]["staffed_count"] = Value::Null,
        "aggregate availability known status requires open/staffed counts and no reason",
    );
    rejects_with(
        |v| {
            v["capacity_location"]["capacity"]["availability_status"] = json!("unknown");
            v["capacity_location"]["capacity"]["open_count"] = Value::Null;
            v["capacity_location"]["capacity"]["staffed_count"] = Value::Null;
            v["capacity_location"]["capacity"]["availability_reason"] = json!("  ");
        },
        "aggregate availability unknown status requires null counts and a reason",
    );
    rejects_with(
        |v| {
            v["capacity_location"]["capacity"]["availability_status"] = json!("unknown");
            v["capacity_location"]["capacity"]["open_count"] = json!(1);
            v["capacity_location"]["capacity"]["staffed_count"] = Value::Null;
            v["capacity_location"]["capacity"]["availability_reason"] =
                json!("not established by this synthetic fixture");
        },
        "aggregate availability unknown status requires null counts and a reason",
    );
    rejects_with(
        |v| {
            v["capacity_location"]["capacity"]["availability_status"] = json!("unknown");
            v["capacity_location"]["capacity"]["open_count"] = Value::Null;
            v["capacity_location"]["capacity"]["staffed_count"] = json!(1);
            v["capacity_location"]["capacity"]["availability_reason"] =
                json!("not established by this synthetic fixture");
        },
        "aggregate availability unknown status requires null counts and a reason",
    );
    accepts(|v| {
        v["capacity_location"]["capacity"]["availability_status"] = json!("unknown");
        v["capacity_location"]["capacity"]["open_count"] = Value::Null;
        v["capacity_location"]["capacity"]["staffed_count"] = Value::Null;
        v["capacity_location"]["capacity"]["availability_reason"] =
            json!("not established by this synthetic fixture");
    });
    rejects_with(
        |v| {
            v["capacity_location"]["capacity"]["open_count"] = json!(2);
        },
        "aggregate capacity counts are inconsistent",
    );
    rejects_with(
        |v| {
            v["capacity_location"]["capacity"]["physical_count"] = json!(2);
            v["capacity_location"]["capacity"]["staffed_count"] = json!(2);
        },
        "aggregate capacity counts are inconsistent",
    );
    accepts(|v| {
        v["capacity_location"]["capacity"]["physical_count"] = json!(2);
        v["capacity_location"]["capacity"]["open_count"] = json!(2);
        v["capacity_location"]["capacity"]["staffed_count"] = json!(2);
    });
}

#[test]
fn equipment_reason_rules_match_each_status() {
    for status in ["available", "unavailable"] {
        accepts(|v| {
            v["capacity_location"]["capacity"]["equipment"] = json!([{
                "capability": "monitoring",
                "status": status
            }]);
        });
    }
    accepts(|v| {
        v["capacity_location"]["capacity"]["equipment"] = json!([{
            "capability": "monitoring",
            "status": "unknown",
            "reason": "not established by this synthetic fixture"
        }]);
    });
    rejects_with(
        |v| {
            v["capacity_location"]["capacity"]["equipment"] = json!([{
                "capability": "monitoring",
                "status": "unknown"
            }]);
        },
        "unknown equipment status requires a nonempty reason",
    );
    rejects_with(
        |v| {
            v["capacity_location"]["capacity"]["equipment"] = json!([{
                "capability": "monitoring",
                "status": "unknown",
                "reason": "  "
            }]);
        },
        "unknown equipment status requires a nonempty reason",
    );
    rejects_with(
        |v| {
            v["capacity_location"]["capacity"]["equipment"] = json!([{
                "capability": "monitoring",
                "status": "available",
                "reason": "contradicts known status"
            }]);
        },
        "known equipment status must not have a reason",
    );
    rejects_with(
        |v| {
            v["capacity_location"]["capacity"]["equipment"] = json!([{
                "capability": "monitoring",
                "status": "proposed"
            }]);
        },
        "unsupported capacity equipment status",
    );
}

#[test]
fn staff_count_status_requires_the_matching_value_and_reason() {
    accepts(|v| {
        first_staff_count(v)["status"] = json!("unknown");
        first_staff_count(v)["count"] = Value::Null;
        first_staff_count(v)["reason"] = json!("not established by this synthetic fixture");
    });
    rejects_with(
        |v| first_staff_count(v)["count"] = Value::Null,
        "known staff count requires an integer and no reason",
    );
    rejects_with(
        |v| first_staff_count(v)["reason"] = json!("contradicts known status"),
        "known staff count requires an integer and no reason",
    );
    rejects_with(
        |v| {
            first_staff_count(v)["status"] = json!("unknown");
            first_staff_count(v)["reason"] = json!("not established by this synthetic fixture");
        },
        "unknown staff count requires null and a reason",
    );
    rejects_with(
        |v| {
            first_staff_count(v)["status"] = json!("unknown");
            first_staff_count(v)["count"] = Value::Null;
            first_staff_count(v)
                .as_object_mut()
                .unwrap()
                .remove("reason");
        },
        "unknown staff count requires null and a reason",
    );
    rejects_with(
        |v| {
            first_staff_count(v)["status"] = json!("unknown");
            first_staff_count(v)["count"] = Value::Null;
            first_staff_count(v)["reason"] = json!("  ");
        },
        "unknown staff count requires null and a reason",
    );
}

#[test]
fn eligibility_and_required_resource_classes_are_checked_independently() {
    rejects_with(
        |v| v["capacity_location"]["eligibility"][0]["staff_role_id"] = json!("missing-staff"),
        "eligibility contains an unresolved staff, task, or zone reference",
    );
    rejects_with(
        |v| v["capacity_location"]["eligibility"][0]["task_class_id"] = json!("missing-task"),
        "eligibility contains an unresolved staff, task, or zone reference",
    );
    rejects_with(
        |v| v["capacity_location"]["eligibility"][0]["zone_id"] = json!("missing-zone"),
        "eligibility contains an unresolved staff, task, or zone reference",
    );
    rejects_with(
        |v| {
            v["capacity_location"]["task_classes"][0]["required_resource_classes"] =
                json!(["different_class"]);
        },
        "patient 'synthetic-patient-1' task class is incompatible with resource 'resource.synthetic'",
    );
    rejects_with(
        |v| {
            v["capacity_location"]["task_classes"]
                .as_array_mut()
                .unwrap()
                .push(json!({
                    "task_class_id": "task.other",
                    "name": "other synthetic task",
                    "required_resource_classes": []
                }));
            v["capacity_location"]["eligibility"][0]["task_class_id"] = json!("task.other");
        },
        "patient 'synthetic-patient-1' has no matching staff/task/zone eligibility",
    );
    rejects_with(
        |v| {
            v["capacity_location"]["zones"]
                .as_array_mut()
                .unwrap()
                .push(json!({
                    "zone_id": "zone.other",
                    "name": "other synthetic zone"
                }));
            v["capacity_location"]["eligibility"][0]["zone_id"] = json!("zone.other");
        },
        "patient 'synthetic-patient-1' has no matching staff/task/zone eligibility",
    );
    rejects_with(
        |v| {
            v["capacity_location"]["locations"]
                .as_array_mut()
                .unwrap()
                .push(json!({
                    "location_id": "location.other",
                    "name": "other synthetic location",
                    "zone_id": "zone.synthetic"
                }));
            v["capacity_location"]["eligibility"][0]["location_id"] = json!("location.other");
        },
        "patient 'synthetic-patient-1' has no matching staff/task/zone eligibility",
    );
}

#[test]
fn routes_require_resolved_locations_and_nonnegative_metre_distance() {
    accepts(|_| {});
    rejects_with(
        |v| v["capacity_location"]["routes"][0]["from_location_id"] = json!("missing-from"),
        "route contains an unresolved location reference",
    );
    rejects_with(
        |v| v["capacity_location"]["routes"][0]["to_location_id"] = json!("missing-to"),
        "route contains an unresolved location reference",
    );
    rejects_with(
        |v| v["capacity_location"]["routes"][0]["distance"]["unit"] = json!("km"),
        "route distance must be finite, nonnegative, and use unit 'm'",
    );
    rejects_with(
        |v| v["capacity_location"]["routes"][0]["distance"]["value"] = json!(-1),
        "route distance must be finite, nonnegative, and use unit 'm'",
    );
}

#[test]
fn identifiers_must_be_nonempty_and_already_trimmed() {
    parse_scenario_json(&scenario().to_string()).unwrap();
    rejects_with(
        |v| v["scenario_id"] = json!(""),
        "scenario_id must be a nonempty, trimmed identifier",
    );
    rejects_with(
        |v| v["scenario_id"] = json!("   "),
        "scenario_id must be a nonempty, trimmed identifier",
    );
    rejects_with(
        |v| v["scenario_id"] = json!(" padded "),
        "scenario_id must be a nonempty, trimmed identifier",
    );
}
