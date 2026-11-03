use careops_birthing_cli::theatre_tables::{theatre_tables, TheatreCapture};
use careops_birthing_cli::{model, BatchInput};
fn capture() -> TheatreCapture {
    let mut input: BatchInput =
        serde_json::from_str(include_str!("../fixtures/synthetic.json")).unwrap();
    input.scenario.days = 1;
    let rows = model::simulate(&input.scenario, 41, 1, 0).unwrap();
    TheatreCapture {
        run_id: "r".into(),
        replication_index: 0,
        events: rows
            .into_iter()
            .enumerate()
            .map(|(i, row)| ((i + 1) as u64, row))
            .collect(),
    }
}
#[test]
fn actual_producer_populates_native_views_without_double_counting_callbacks() {
    let source = capture();
    let tables = theatre_tables(std::slice::from_ref(&source)).unwrap();
    let cases = source
        .events
        .iter()
        .filter(|(_, r)| r.kind == "theatre_case")
        .collect::<Vec<_>>();
    assert_eq!(tables.cases.height(), cases.len());
    assert_eq!(
        tables.callbacks.height(),
        cases
            .iter()
            .map(|(_, r)| r.data["callbacks"].as_array().unwrap().len())
            .sum::<usize>()
    );
    assert!(tables.assessments.height() > 0);
    assert!(tables.policy_rules.height() > 0);
    assert!(tables.metrics.height() > 0);
}

use careops_birthing_cli::theatre_tables::read_theatre_tables;
use careops_birthing_cli::{run_batch, Row};
use careops_core::experiment::{ExperimentError, ExperimentErrorKind};
use serde_json::{json, Value};
fn vectors() -> Value {
    serde_json::from_str(include_str!(
        "../../careops-ed/tests/fixtures/r121_analytical.json"
    ))
    .unwrap()
}
fn malformed(
    result: careops_birthing_cli::Result<careops_birthing_cli::theatre_tables::TheatreTables>,
) {
    let error = result.unwrap_err();
    assert_eq!(
        error
            .downcast_ref::<ExperimentError>()
            .expect("typed validation error")
            .kind,
        ExperimentErrorKind::MalformedConfig
    );
}
// The independent vectors use symbolic case labels. This fixed bijection renders
// them as the domain's positive UInt64 identifiers without changing equality,
// multiplicity or expected counts. Generic event entity0 is a different type.
fn case_id(label: &str) -> u64 {
    match label {
        "c0" => 1,
        "missing" => 2,
        _ => panic!("unknown frozen case label"),
    }
}
fn fragment() -> TheatreCapture {
    let mut c = capture();
    let id = c
        .events
        .iter()
        .find(|(_, r)| r.kind == "theatre_case")
        .unwrap()
        .1
        .data["case"]
        .as_u64()
        .unwrap();
    c.events.retain(|(_, r)| {
        matches!(r.kind.as_str(), "theatre_case" | "theatre_callback")
            && r.data["case"].as_u64() == Some(id)
    });
    for (_, row) in &mut c.events {
        row.data["case"] = json!(1u64);
    }
    c
}
fn case_mut(c: &mut TheatreCapture) -> &mut Row {
    &mut c
        .events
        .iter_mut()
        .find(|(_, r)| r.kind == "theatre_case")
        .unwrap()
        .1
}
fn partial() -> TheatreCapture {
    let mut c = fragment();
    c.events.retain(|(_, r)| r.kind == "theatre_case");
    let row = case_mut(&mut c);
    row.tick = 0;
    row.data["decision_tick"] = json!(0u64);
    for name in [
        "accepted",
        "departed",
        "arrived",
        "started",
        "incision",
        "birth",
        "completed",
        "cancelled",
        "cleaned",
    ] {
        row.data["timings"][name] = Value::Null;
    }
    for kind in ["transfer", "incision", "birth"] {
        row.data["metrics"][kind] = Value::Null;
    }
    row.data["callbacks"] = json!([]);
    let mut assessment = row.data["assessments"][0].clone();
    assessment["observed_at_tick"] = json!(0);
    assessment["elapsed_wait_ticks"] = json!(0);
    assessment["urgency"]["assessed_at_tick"] = json!(0);
    row.data["assessments"] = json!([assessment]);
    c
}
fn add_scheduled(c: &mut TheatreCapture, kind: &str, tick: u64) {
    let row = case_mut(c);
    let episode = row.episode;
    let id = row.data["case"].as_u64().unwrap();
    row.tick = row.tick.max(tick);
    row.data["callbacks"]
        .as_array_mut()
        .unwrap()
        .push(json!({"callback":kind,"tick":tick,"episode":episode}));
    let field = match kind {
        "ProcedureStarted" => "incision",
        "Birth" => "birth",
        "ServiceCompleted" => "completed",
        "RoomReady" => "cleaned",
        _ => panic!("test scheduled kind"),
    };
    row.data["timings"][field] = json!(tick);
    let sequence = c.events.iter().map(|(s, _)| *s).max().unwrap() + 1;
    c.events.push((
        sequence,
        Row {
            tick,
            person: 0,
            episode,
            kind: "theatre_callback".into(),
            data: json!({"case":id,"callback":kind}),
        },
    ));
}
#[test]
fn frozen_cross_replication_case_identity_is_valid() {
    let v = vectors();
    let mut contexts = Vec::new();
    for key in v["birthing"]["valid_cross_replication"].as_array().unwrap() {
        let mut c = fragment();
        c.run_id = key[0].as_str().unwrap().into();
        c.replication_index = key[1].as_u64().unwrap();
        for (_, r) in &mut c.events {
            r.data["case"] = json!(case_id(key[2].as_str().unwrap()));
        }
        contexts.push(c);
    }
    let t = theatre_tables(&contexts).unwrap();
    assert_eq!(t.cases.height(), 2);
    assert_eq!(t.metrics.height(), 6);
}
#[test]
fn duplicate_case_and_duplicate_capture_sequence_reject() {
    let mut c = fragment();
    let row = case_mut(&mut c).clone();
    let seq = c.events.iter().map(|(s, _)| *s).max().unwrap() + 1;
    c.events.push((seq, row));
    malformed(theatre_tables(&[c]));
    let mut c = fragment();
    c.events.push(c.events[0].clone());
    malformed(theatre_tables(&[c]));
}
#[test]
fn frozen_equal_tick_assessments_survive_and_metric_reference_is_ambiguous() {
    let v = vectors();
    let mut c = fragment();
    let row = case_mut(&mut c);
    let template = row.data["assessments"][0].clone();
    row.data["decision_tick"] = json!(0);
    let assessments = v["birthing"]["equal_tick_assessments"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| {
            assert_eq!(case_id(a["case_id"].as_str().unwrap()), 1);
            let tick = a["tick"].as_u64().unwrap();
            let mut row = template.clone();
            row["observed_at_tick"] = json!(tick);
            row["elapsed_wait_ticks"] = json!(tick);
            row["urgency"]["assessed_at_tick"] = json!(tick);
            row
        })
        .collect::<Vec<_>>();
    row.data["assessments"] = json!(assessments);
    for (kind, field) in [
        ("transfer", "departed"),
        ("incision", "incision"),
        ("birth", "birth"),
    ] {
        let tick = row.data["timings"][field].as_u64().unwrap();
        assert!(tick >= 7);
        let metric = &mut row.data["metrics"][kind];
        metric["assessment_tick"] = json!(7);
        metric["elapsed"] = json!(tick);
        metric["breached"] = metric["target"]
            .as_u64()
            .map_or(Value::Null, |target| json!(tick > target));
    }
    let t = theatre_tables(&[c]).unwrap();
    assert_eq!(
        t.assessments.height(),
        v["birthing"]["expected_assessment_count"].as_u64().unwrap() as usize
    );
    for i in 0..t.metrics.height() {
        assert_eq!(
            t.metrics
                .column("assessment_candidate_count")
                .unwrap()
                .u64()
                .unwrap()
                .get(i),
            Some(2)
        );
        assert_eq!(
            t.metrics
                .column("assessment_reference_status")
                .unwrap()
                .str()
                .unwrap()
                .get(i),
            Some("ambiguous")
        );
        assert_eq!(
            t.metrics
                .column("resolved_assessment_ordinal")
                .unwrap()
                .u64()
                .unwrap()
                .get(i),
            None
        );
        assert_eq!(
            t.metrics
                .column("resolved_policy_jurisdiction_id")
                .unwrap()
                .str()
                .unwrap()
                .get(i),
            None
        );
    }
}
#[test]
fn frozen_orphan_callback_child_is_typed_error() {
    let v = vectors();
    let mut c = partial();
    add_scheduled(&mut c, "ProcedureStarted", 7);
    c.events.last_mut().unwrap().1.data["case"] = json!(case_id(
        v["birthing"]["orphan_child"]["case_id"].as_str().unwrap()
    ));
    malformed(theatre_tables(&[c]));
}
#[test]
fn frozen_null_and_zero_targets_remain_distinct() {
    for vector in vectors()["birthing"]["targets"].as_array().unwrap() {
        let mut c = partial();
        let row = case_mut(&mut c);
        let target = vector["target"].clone();
        let elapsed = vector["elapsed"].as_u64().unwrap();
        let a = &mut row.data["assessments"][0];
        a["rule"]["targets"]["transfer"] = target.clone();
        a["policy"]["rules"][0]["targets"]["transfer"] = target.clone();
        let revision = a["policy"]["revision"].clone();
        let version = a["policy"]["scheme_version"].clone();
        row.data["timings"]["accepted"] = json!(0);
        row.data["timings"]["departed"] = json!(elapsed);
        row.data["metrics"]["transfer"] = json!({"assessment_tick":0,"elapsed":elapsed,"target":target,"breached":vector["breach"],"policy_revision":revision,"scheme_version":version});
        row.tick = elapsed;
        let t = theatre_tables(&[c]).unwrap();
        assert_eq!(t.metrics.height(), 1);
        assert_eq!(
            t.metrics
                .column("target_ticks_lo")
                .unwrap()
                .u64()
                .unwrap()
                .get(0),
            vector["target"].as_u64()
        );
        assert_eq!(
            t.metrics.column("breached").unwrap().bool().unwrap().get(0),
            vector["breach"].as_bool()
        );
        assert_eq!(
            t.cases
                .column("birth_metric_recorded")
                .unwrap()
                .bool()
                .unwrap()
                .get(0),
            Some(false)
        );
    }
}
#[test]
fn frozen_policy_identity_with_different_canonical_content_refuses() {
    let v = vectors();
    let fixture = &v["birthing"]["conflicting_policy_content"];
    let mut contexts = Vec::new();
    for (i, content) in fixture["contents"].as_array().unwrap().iter().enumerate() {
        let mut c = partial();
        c.replication_index = i as u64;
        let a = &mut case_mut(&mut c).data["assessments"][0];
        for (name, index) in [
            ("jurisdiction_id", 0),
            ("scheme_id", 1),
            ("scheme_version", 2),
            ("revision", 3),
        ] {
            a["policy"][name] = fixture["identity"][index].clone();
            if name != "revision" {
                a["urgency"][name] = fixture["identity"][index].clone();
            }
        }
        a["policy"]["citation"] = content.clone();
        a["urgency"]["citation"] = content.clone();
        contexts.push(c);
    }
    malformed(theatre_tables(&contexts));
}
#[test]
fn frozen_callback_overlap_counts_authoritative_source_once() {
    let v = vectors();
    assert_eq!(
        v["birthing"]["callback_overlap"]["embedded_sequences"],
        json!([0])
    );
    let mut c = partial();
    add_scheduled(&mut c, "ProcedureStarted", 7);
    let t = theatre_tables(&[c]).unwrap();
    assert_eq!(
        t.callbacks.height(),
        v["birthing"]["callback_overlap"]["authoritative_count"]
            .as_u64()
            .unwrap() as usize
    );
    assert_eq!(
        t.callbacks
            .column("callback_ordinal")
            .unwrap()
            .u64()
            .unwrap()
            .get(0),
        Some(0)
    );
}
#[test]
fn missing_duplicate_or_unsupported_top_overlap_rejects() {
    let mut c = partial();
    add_scheduled(&mut c, "ProcedureStarted", 7);
    let mut missing = c.clone();
    missing.events.retain(|(_, r)| r.kind != "theatre_callback");
    malformed(theatre_tables(&[missing]));
    let mut duplicate = c.clone();
    let seq = duplicate.events.iter().map(|(s, _)| *s).max().unwrap() + 1;
    let mut row = duplicate.events.last().unwrap().clone();
    row.0 = seq;
    duplicate.events.push(row);
    malformed(theatre_tables(&[duplicate]));
    c.events.last_mut().unwrap().1.data["callback"] = json!("Accepted");
    malformed(theatre_tables(&[c]));
}
#[test]
fn equal_tick_callbacks_keep_distinct_captured_ordinals() {
    let mut c = partial();
    add_scheduled(&mut c, "ProcedureStarted", 7);
    add_scheduled(&mut c, "Birth", 7);
    let t = theatre_tables(&[c]).unwrap();
    assert_eq!(t.callbacks.height(), 2);
    assert_eq!(
        t.callbacks
            .column("callback_ordinal")
            .unwrap()
            .u64()
            .unwrap()
            .get(1),
        Some(1)
    );
}
#[test]
fn absent_and_missing_target_fields_are_not_equivalent() {
    let mut c = partial();
    case_mut(&mut c).data["assessments"][0]["rule"]["targets"]
        .as_object_mut()
        .unwrap()
        .remove("transfer");
    malformed(theatre_tables(&[c]));
}
#[test]
fn selected_rule_and_missing_policy_category_refuse() {
    let mut c = partial();
    case_mut(&mut c).data["assessments"][0]["rule"]["priority_rank"] = json!(9);
    malformed(theatre_tables(&[c]));
    let mut c = partial();
    case_mut(&mut c).data["assessments"][0]["policy"]["rules"][0]["category"] = json!(2);
    malformed(theatre_tables(&[c]));
}
#[test]
fn future_and_stale_metric_assessment_references_refuse() {
    let mut c = fragment();
    let row = case_mut(&mut c);
    let event = row.data["timings"]["departed"].as_u64().unwrap();
    row.data["metrics"]["transfer"]["assessment_tick"] = json!(event + 1);
    malformed(theatre_tables(&[c]));
    let mut c = fragment();
    let row = case_mut(&mut c);
    let event = row.data["timings"]["birth"].as_u64().unwrap();
    let mut a = row.data["assessments"][0].clone();
    let decision = row.data["decision_tick"].as_u64().unwrap();
    a["observed_at_tick"] = json!(event - 1);
    a["elapsed_wait_ticks"] = json!(event - 1 - decision);
    a["urgency"]["assessed_at_tick"] = json!(event - 1);
    row.data["assessments"].as_array_mut().unwrap().push(a);
    malformed(theatre_tables(&[c]));
}
#[test]
fn typed_empty_and_owned_views_do_not_invent_rows() {
    let empty = theatre_tables(&[]).unwrap();
    assert_eq!(empty.cases.height(), 0);
    assert_eq!(empty.metrics.height(), 0);
    assert_eq!(empty.callbacks.height(), 0);
    let c = fragment();
    let table = theatre_tables(std::slice::from_ref(&c)).unwrap();
    drop(c);
    assert!(table.cases.height() > 0);
    assert!(table.metrics.height() > 0);
}
#[test]
fn actual_checked_io_views_match_across_worker_counts() {
    let mut input: BatchInput =
        serde_json::from_str(include_str!("../fixtures/synthetic.json")).unwrap();
    input.replications = 2;
    input.scenario.days = 1;
    let mut previous = None;
    for workers in [1, 2, 4] {
        let root = tempfile::tempdir().unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        let reps = run_batch(root.path(), &input, workers, false, None).unwrap();
        let (manifests, t) =
            read_theatre_tables(&root.path().join(&input.batch_id), &input, &reps).unwrap();
        assert_eq!(manifests.len(), 2);
        if let Some(p) = previous {
            let p: careops_birthing_cli::theatre_tables::TheatreTables = p;
            assert!(p.cases.equals_missing(&t.cases));
            assert!(p.assessments.equals_missing(&t.assessments));
            assert!(p.metrics.equals_missing(&t.metrics));
            assert!(p.callbacks.equals_missing(&t.callbacks));
            assert!(p.policy_rules.equals_missing(&t.policy_rules));
        }
        previous = Some(t);
    }
}

#[test]
fn derived_array_bound_refuses_before_child_deserialization() {
    let mut c = partial();
    case_mut(&mut c).data["assessments"] = Value::Array(vec![Value::Null; 131073]);
    let error = theatre_tables(&[c]).unwrap_err();
    assert!(error.to_string().contains("derived theatre row bound"));
    assert_eq!(
        error.downcast_ref::<ExperimentError>().unwrap().kind,
        ExperimentErrorKind::MalformedConfig
    );
}
#[test]
fn canonical_policy_objects_ignore_key_order_but_arrays_retain_order() {
    fn reverse_objects(value: Value) -> Value {
        match value {
            Value::Object(object) => {
                let entries = object.into_iter().collect::<Vec<_>>();
                Value::Object(
                    entries
                        .into_iter()
                        .rev()
                        .map(|(k, v)| (k, reverse_objects(v)))
                        .collect(),
                )
            }
            Value::Array(array) => Value::Array(array.into_iter().map(reverse_objects).collect()),
            v => v,
        }
    }
    let first = partial();
    let mut second = first.clone();
    second.replication_index = 1;
    for (_, row) in &mut second.events {
        row.data = reverse_objects(row.data.clone());
    }
    let table = theatre_tables(&[first.clone(), second]).unwrap();
    assert_eq!(table.policy_rules.height(), 1);
    let mut first = first;
    let a = &mut case_mut(&mut first).data["assessments"][0];
    let mut other = a["policy"]["rules"][0].clone();
    other["category"] = json!(2);
    a["policy"]["rules"].as_array_mut().unwrap().push(other);
    let mut second = first.clone();
    second.replication_index = 1;
    case_mut(&mut second).data["assessments"][0]["policy"]["rules"]
        .as_array_mut()
        .unwrap()
        .reverse();
    malformed(theatre_tables(&[first, second]));
}
#[test]
fn full_policy_fk_exists_only_for_a_unique_captured_reference() {
    let t = theatre_tables(&[fragment()]).unwrap();
    for i in 0..t.metrics.height() {
        assert_eq!(
            t.metrics
                .column("assessment_reference_status")
                .unwrap()
                .str()
                .unwrap()
                .get(i),
            Some("resolved")
        );
        assert_eq!(
            t.metrics
                .column("assessment_candidate_count")
                .unwrap()
                .u64()
                .unwrap()
                .get(i),
            Some(1)
        );
        for name in [
            "resolved_policy_jurisdiction_id",
            "resolved_policy_scheme_id",
            "resolved_policy_scheme_version",
            "resolved_policy_revision",
            "resolved_policy_content_sha256",
        ] {
            assert!(t
                .metrics
                .column(name)
                .unwrap()
                .str()
                .unwrap()
                .get(i)
                .is_some());
        }
    }
    assert!(t.assessments.column("schema_right").is_err());
    assert!(t.assessments.column("policy_schema").is_ok());
}
#[test]
fn maximum_supported_b7_batch_projects_without_row_loss() {
    let mut input: BatchInput =
        serde_json::from_str(include_str!("../fixtures/synthetic.json")).unwrap();
    input.scenario.days = 7;
    let captures = (0..32)
        .map(|rep| TheatreCapture {
            run_id: "r".into(),
            replication_index: rep,
            events: model::simulate(&input.scenario, 41 + rep, rep + 1, 0)
                .unwrap()
                .into_iter()
                .enumerate()
                .map(|(i, row)| (i as u64 + 1, row))
                .collect(),
        })
        .collect::<Vec<_>>();
    let expected = captures
        .iter()
        .flat_map(|c| &c.events)
        .filter(|(_, r)| r.kind == "theatre_case")
        .count();
    let t = theatre_tables(&captures).unwrap();
    assert_eq!(t.cases.height(), expected);
    assert_eq!(t.metrics.height(), expected * 3);
    assert_eq!(t.callbacks.height(), expected * 8);
}

#[test]
fn review_actual_domain_callback_then_same_tick_reassessment_remains_valid() {
    use careops_birthing::team::RatioProfile;
    use careops_birthing::theatre::{
        QueuePolicy, QueueRule, Targets, TheatreProfile, TheatreRequest, TheatreService,
    };
    use careops_core::pathway::{ObservedPolicyState, VersionedUrgencyAssessment};
    use careops_core::staffing::BookingId;
    use std::collections::BTreeMap;
    let policy = QueuePolicy {
        revision: "r1".into(),
        jurisdiction_id: "j".into(),
        scheme_id: "s".into(),
        scheme_version: "v1".into(),
        source_authority: "synthetic:domain-order".into(),
        citation: "fixture:domain-order".into(),
        rules: BTreeMap::from([(
            1,
            QueueRule {
                priority_rank: 0,
                targets: Targets {
                    transfer: Some(0),
                    incision: Some(0),
                    birth: None,
                },
            },
        )]),
    };
    let observed = |tick| ObservedPolicyState {
        observed_at_tick: tick,
        elapsed_wait_ticks: tick,
        urgency: VersionedUrgencyAssessment {
            jurisdiction_id: "j".into(),
            scheme_id: "s".into(),
            scheme_version: "v1".into(),
            category: 1,
            assessed_at_tick: tick,
            effective_date: "2026-11-03".into(),
            source_authority: "synthetic:domain-order".into(),
            citation: "fixture:domain-order".into(),
            source_sha256: None,
        },
    };
    let mut service = TheatreService::new(TheatreProfile {
        revision: "p1".into(),
        preparation_ticks: 1,
        birth_after_incision_ticks: 1,
        completion_after_birth_ticks: 1,
        cleaning_ticks: 1,
        required_capabilities: vec!["theatre".into()],
        staffing: RatioProfile {
            revision: "staff1".into(),
            cohort_unit: "person".into(),
            patients_per_staff: BTreeMap::from([("r".into(), 1)]),
        },
    })
    .unwrap();
    service
        .submit(TheatreRequest {
            id: 1,
            episode: 1,
            booking: BookingId::new(1),
            decision_tick: 0,
            cohort_size: 1,
            observed: observed(0),
            policy: policy.clone(),
        })
        .unwrap();
    service.accept(1, 0).unwrap();
    service.transfer_departed(1, 10).unwrap();
    let mut revised = policy;
    revised.revision = "r2".into();
    revised.rules.get_mut(&1).unwrap().targets.transfer = Some(20);
    service.reassess(1, observed(10), revised).unwrap();
    let case = service.case(1).unwrap();
    assert_eq!(case.metrics.transfer.as_ref().unwrap().assessment_tick, 0);
    assert_eq!(case.assessments[1].observed.observed_at_tick, 10);
    let targets =
        |t: &Targets| json!({"transfer":t.transfer,"incision":t.incision,"birth":t.birth});
    let assessments=case.assessments.iter().map(|a|json!({"observed_at_tick":a.observed.observed_at_tick,"elapsed_wait_ticks":a.observed.elapsed_wait_ticks,"urgency":a.observed.urgency,"interruption":format!("{:?}",a.interruption),"rule":{"priority_rank":a.rule.priority_rank,"targets":targets(&a.rule.targets)},"policy":{"revision":a.policy.revision,"jurisdiction_id":a.policy.jurisdiction_id,"scheme_id":a.policy.scheme_id,"scheme_version":a.policy.scheme_version,"source_authority":a.policy.source_authority,"citation":a.policy.citation,"rules":a.policy.rules.iter().map(|(category,r)|json!({"category":category,"priority_rank":r.priority_rank,"targets":targets(&r.targets)})).collect::<Vec<_>>()}})).collect::<Vec<_>>();
    let metric = case.metrics.transfer.as_ref().unwrap();
    let callbacks = service
        .events()
        .iter()
        .map(|c| json!({"callback":format!("{:?}",c.kind),"tick":c.tick,"episode":c.episode}))
        .collect::<Vec<_>>();
    let t = &case.timings;
    let row = Row {
        tick: 10,
        person: 1,
        episode: 1,
        kind: "theatre_case".into(),
        data: json!({"case":1,"episode":1,"decision_tick":0,"timings":{"accepted":t.accepted,"departed":t.departed,"arrived":t.arrived,"started":t.started,"incision":t.incision,"birth":t.birth,"completed":t.completed,"cancelled":t.cancelled,"cleaned":t.cleaned},"metrics":{"transfer":{"assessment_tick":metric.assessment_tick,"elapsed":metric.elapsed,"target":metric.target,"breached":metric.breached,"policy_revision":metric.policy_revision,"scheme_version":metric.scheme_version},"incision":null,"birth":null},"assessments":assessments,"callbacks":callbacks,"resource_failures":[]}),
    };
    let table = theatre_tables(&[TheatreCapture {
        run_id: "r".into(),
        replication_index: 0,
        events: vec![(1, row)],
    }])
    .unwrap();
    assert_eq!(
        table
            .metrics
            .column("resolved_assessment_ordinal")
            .unwrap()
            .u64()
            .unwrap()
            .get(0),
        Some(0)
    );
    assert_eq!(
        table
            .metrics
            .column("resolved_policy_revision")
            .unwrap()
            .str()
            .unwrap()
            .get(0),
        Some("r1")
    );
    assert_eq!(
        table
            .metrics
            .column("target_ticks_lo")
            .unwrap()
            .u64()
            .unwrap()
            .get(0),
        Some(0)
    );
}
#[test]
fn review_callback_must_match_the_corresponding_case_timing() {
    let mut c = fragment();
    let row = case_mut(&mut c);
    let cleaned = row.data["timings"]["cleaned"].as_u64().unwrap();
    row.data["timings"]["cleaned"] = json!(cleaned + 1);
    malformed(theatre_tables(&[c]));
}
#[test]
fn review_checked_batch_rejects_invalid_zero_replications_before_io() {
    let mut input: BatchInput =
        serde_json::from_str(include_str!("../fixtures/synthetic.json")).unwrap();
    input.replications = 0;
    assert!(
        read_theatre_tables(std::path::Path::new("nonexistent-unread-path"), &input, &[]).is_err()
    );
}
#[test]
fn review_checked_batch_binds_every_supplied_receipt_field() {
    let mut input: BatchInput =
        serde_json::from_str(include_str!("../fixtures/synthetic.json")).unwrap();
    input.replications = 1;
    input.scenario.days = 1;
    let root = tempfile::tempdir().unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    let reps = run_batch(root.path(), &input, 1, false, None).unwrap();
    let batch = root.path().join(&input.batch_id);
    let mut rejected = Vec::new();
    for field in ["rows", "canonical_sha256", "model_contract_sha256"] {
        let mut wrong = reps.clone();
        match field {
            "rows" => wrong[0].rows += 1,
            "canonical_sha256" => wrong[0].canonical_sha256 = "0".repeat(64),
            _ => wrong[0].model_contract_sha256 = "a".repeat(64),
        };
        // Keep the parent declaration consistent with the supplied object so the
        // independently checked child receipt must enforce this binding.
        std::fs::write(
            batch.join("batch_manifest.json"),
            serde_json::to_vec(&wrong).unwrap(),
        )
        .unwrap();
        rejected.push(read_theatre_tables(&batch, &input, &wrong).is_err());
    }
    assert_eq!(rejected, [true, true, true]);
}

#[test]
fn duplicate_policy_category_in_one_snapshot_refuses_instead_of_deduplicating() {
    let mut c = partial();
    let rules = case_mut(&mut c).data["assessments"][0]["policy"]["rules"]
        .as_array_mut()
        .unwrap();
    rules.push(rules[0].clone());
    malformed(theatre_tables(&[c]));
}
#[test]
fn missing_case_timestamp_does_not_get_imputed_from_a_callback() {
    let mut c = fragment();
    case_mut(&mut c).data["timings"]["cleaned"] = Value::Null;
    malformed(theatre_tables(&[c]));
}
#[test]
fn missing_parent_completion_manifest_is_not_a_checked_complete_batch() {
    let mut input: BatchInput =
        serde_json::from_str(include_str!("../fixtures/synthetic.json")).unwrap();
    input.replications = 1;
    input.scenario.days = 1;
    let root = tempfile::tempdir().unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    let reps = run_batch(root.path(), &input, 1, false, None).unwrap();
    let batch = root.path().join(&input.batch_id);
    std::fs::remove_file(batch.join("batch_manifest.json")).unwrap();
    assert!(read_theatre_tables(&batch, &input, &reps).is_err());
}

#[test]
fn pure_native_view_configures_its_declared_budget_without_a_shell_preset() {
    let source = partial();
    theatre_tables(&[source]).unwrap();
    let budget = kairo_ecs_arrow::run_output::native_thread_budget().unwrap();
    assert_eq!(
        budget.map(|b| b.worker_threads.get()),
        Some(1),
        "all native entry points must cross the shared initialization barrier"
    );
}
