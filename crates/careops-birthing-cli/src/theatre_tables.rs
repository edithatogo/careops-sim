//! Offline views of the bounded synthetic B7 capture. The pure constructor
//! validates declared capture rows; only `read_theatre_tables` proves native
//! manifest/model/input binding. No policy is reselected and no outcome inferred.
//! Resource-failure detail remains in the original checked event capture.
use crate::{
    hash, replication_events, BatchInput, Replication, ReplicationManifest, Result, Row, MAX_ROWS,
};
use careops_core::experiment::{ExperimentError, ExperimentErrorKind};
use careops_core::pathway::VersionedUrgencyAssessment;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::Path;
// Concrete typed-column construction repeats across the five domain views.
// This local helper only removes Series allocation boilerplate; all query,
// schema and validation decisions remain explicit in the functions below.
macro_rules! column {
    ($name:literal, $rows:expr, $map:expr) => {
        Series::new($name.into(), $rows.iter().map($map).collect::<Vec<_>>()).into()
    };
}
const MAX_VIEW_ROWS: usize = MAX_ROWS * 32;
const CASE_KEYS: [&str; 3] = ["run_id", "replication_index", "case_id"];
const POLICY_KEYS: [&str; 4] = [
    "policy_jurisdiction_id",
    "policy_scheme_id",
    "policy_scheme_version",
    "policy_revision",
];
fn invalid(e: impl std::fmt::Display) -> Box<dyn std::error::Error + Send + Sync> {
    Box::new(ExperimentError::new(
        ExperimentErrorKind::MalformedConfig,
        e.to_string(),
    ))
}
/// Caller-declared source context. Tuple entries retain original capture sequence
/// numbers, even when this is a case/callback projection of a larger event log.
#[derive(Clone, Debug)]
pub struct TheatreCapture {
    pub run_id: String,
    pub replication_index: u64,
    pub events: Vec<(u64, Row)>,
}
#[derive(Clone, Debug)]
pub struct TheatreTables {
    pub cases: DataFrame,
    pub assessments: DataFrame,
    pub policy_rules: DataFrame,
    pub metrics: DataFrame,
    pub callbacks: DataFrame,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Targets {
    transfer: Option<u64>,
    incision: Option<u64>,
    birth: Option<u64>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Rule {
    priority_rank: i32,
    targets: Targets,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PolicyRule {
    category: u8,
    priority_rank: i32,
    targets: Targets,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Policy {
    revision: String,
    jurisdiction_id: String,
    scheme_id: String,
    scheme_version: String,
    source_authority: String,
    citation: String,
    rules: Vec<PolicyRule>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Assessment {
    observed_at_tick: u64,
    elapsed_wait_ticks: u64,
    urgency: VersionedUrgencyAssessment,
    interruption: String,
    rule: Rule,
    policy: Policy,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Metric {
    assessment_tick: u64,
    elapsed: u64,
    target: Option<u64>,
    breached: Option<bool>,
    policy_revision: String,
    scheme_version: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Metrics {
    transfer: Option<Metric>,
    incision: Option<Metric>,
    birth: Option<Metric>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Timings {
    accepted: Option<u64>,
    departed: Option<u64>,
    arrived: Option<u64>,
    started: Option<u64>,
    incision: Option<u64>,
    birth: Option<u64>,
    completed: Option<u64>,
    cancelled: Option<u64>,
    cleaned: Option<u64>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Callback {
    callback: String,
    tick: u64,
    episode: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    #[serde(rename = "case")]
    id: u64,
    episode: u64,
    decision_tick: u64,
    timings: Timings,
    metrics: Metrics,
    assessments: Vec<Assessment>,
    callbacks: Vec<Callback>,
    #[serde(rename = "resource_failures")]
    _failures: serde::de::IgnoredAny,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TopCallback {
    #[serde(rename = "case")]
    id: u64,
    callback: String,
}
struct CaseSource<'a> {
    context: &'a TheatreCapture,
    sequence: u64,
    row: &'a Row,
    case: Case,
}
struct PolicySource<'a> {
    policy: &'a Policy,
    content: String,
    sha: String,
}
fn bound(current: usize, additional: usize) -> Result<()> {
    if current
        .checked_add(additional)
        .is_none_or(|n| n > MAX_VIEW_ROWS)
    {
        return Err(invalid("derived theatre row bound exceeded"));
    }
    Ok(())
}
fn required_keys(value: &Value, names: &[&str]) -> Result<()> {
    let object = value
        .as_object()
        .ok_or_else(|| invalid("expected captured object"))?;
    if object.len() != names.len() || names.iter().any(|name| !object.contains_key(*name)) {
        return Err(invalid("unsupported captured object fields"));
    }
    Ok(())
}
fn capture_shape(value: &Value) -> Result<()> {
    required_keys(
        &value["timings"],
        &[
            "accepted",
            "departed",
            "arrived",
            "started",
            "incision",
            "birth",
            "completed",
            "cancelled",
            "cleaned",
        ],
    )?;
    required_keys(&value["metrics"], &["transfer", "incision", "birth"])?;
    for name in ["transfer", "incision", "birth"] {
        let metric = &value["metrics"][name];
        if !metric.is_null() {
            required_keys(
                metric,
                &[
                    "assessment_tick",
                    "elapsed",
                    "target",
                    "breached",
                    "policy_revision",
                    "scheme_version",
                ],
            )?;
        }
    }
    for name in ["assessments", "callbacks", "resource_failures"] {
        let rows = value[name]
            .as_array()
            .ok_or_else(|| invalid("missing captured child array"))?;
        bound(0, rows.len())?;
    }
    for assessment in value["assessments"]
        .as_array()
        .ok_or_else(|| invalid("missing assessments"))?
    {
        required_keys(
            &assessment["rule"]["targets"],
            &["transfer", "incision", "birth"],
        )?;
        let rules = assessment["policy"]["rules"]
            .as_array()
            .ok_or_else(|| invalid("missing captured policy rules"))?;
        if rules.is_empty() || rules.len() > 256 {
            return Err(invalid("policy category bound"));
        }
        for rule in rules {
            required_keys(&rule["targets"], &["transfer", "incision", "birth"])?;
        }
    }
    Ok(())
}
// Objects use sorted keys; arrays retain captured order. All typed policy fields,
// including authority/citation/rules/targets, participate in the content digest.
fn canonical(value: Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.into_iter()
                .collect::<std::collections::BTreeMap<_, _>>()
                .into_iter()
                .map(|(k, v)| (k, canonical(v)))
                .collect(),
        ),
        Value::Array(a) => Value::Array(a.into_iter().map(canonical).collect()),
        v => v,
    }
}
fn ticks(columns: &mut Vec<Column>, name: &str, values: Vec<Option<u64>>) {
    columns.push(
        Series::new(
            format!("{name}_hi").into(),
            values.iter().map(|v| v.map(|_| 0u64)).collect::<Vec<_>>(),
        )
        .into(),
    );
    columns.push(Series::new(format!("{name}_lo").into(), values).into());
}
fn schema_column(name: &str, n: usize) -> Column {
    Series::new("schema".into(), vec![name; n]).into()
}
fn difference(end: u64, start: u64) -> Result<u64> {
    let (hi, lo) =
        kairo_ecs_arrow::polars_interop::subtract_tick_limbs(0, end, 0, start).map_err(invalid)?;
    if hi != 0 {
        return Err(invalid("unsupported source duration width"));
    }
    Ok(lo)
}
fn check_keys(frame: &DataFrame, keys: &[&str]) -> Result<()> {
    if frame.height() > MAX_VIEW_ROWS {
        return Err(invalid("native theatre row capacity"));
    }
    for key in keys {
        if frame.column(key).map_err(invalid)?.null_count() != 0 {
            return Err(invalid("null theatre key"));
        }
    }
    let groups = frame
        .clone()
        .lazy()
        .group_by(keys.iter().map(|key| col(*key)).collect::<Vec<_>>())
        .agg([len().alias("_rows")])
        .collect()
        .map_err(invalid)?;
    if groups.height() != frame.height() {
        return Err(invalid("duplicate theatre key"));
    }
    Ok(())
}
/// Normalize fixed synthetic capture payloads once, then perform offline grain,
/// joins, grouping and method validation in Polars. This does not validate a
/// caller's filesystem receipt or establish clinical/source authenticity.
pub fn theatre_tables(captures: &[TheatreCapture]) -> Result<TheatreTables> {
    crate::initialize_native_data()?;
    if captures.len() > 32 {
        return Err(invalid("replication context bound"));
    }
    let mut contexts = BTreeSet::new();
    let mut cases = Vec::new();
    let mut top = Vec::new();
    let mut source_run = Vec::new();
    let mut source_rep = Vec::new();
    let mut source_sequence = Vec::new();
    for context in captures {
        if context.run_id.trim().is_empty()
            || context.run_id.len() > 256
            || context.events.len() > MAX_ROWS
            || !contexts.insert((context.run_id.as_str(), context.replication_index))
        {
            return Err(invalid("invalid or duplicate capture context"));
        }
        for (sequence, row) in &context.events {
            if *sequence == 0 {
                return Err(invalid("zero capture sequence"));
            }
            source_run.push(context.run_id.as_str());
            source_rep.push(context.replication_index);
            source_sequence.push(*sequence);
            match row.kind.as_str() {
                "theatre_case" => {
                    capture_shape(&row.data)?;
                    let case = Case::deserialize(&row.data).map_err(invalid)?;
                    if case.id == 0
                        || case.episode == 0
                        || case.episode != row.episode
                        || case.assessments.is_empty()
                    {
                        return Err(invalid("invalid captured case identity/history"));
                    }
                    bound(cases.len(), 1)?;
                    cases.push(CaseSource {
                        context,
                        sequence: *sequence,
                        row,
                        case,
                    });
                }
                "theatre_callback" => {
                    let callback = TopCallback::deserialize(&row.data).map_err(invalid)?;
                    bound(top.len(), 1)?;
                    top.push((context, *sequence, row, callback));
                }
                _ => {}
            }
        }
    }
    check_keys(
        &df!("run_id"=>source_run,"replication_index"=>source_rep,"sequence"=>source_sequence)
            .map_err(invalid)?,
        &["run_id", "replication_index", "sequence"],
    )?;
    let mut assessments = Vec::new();
    let mut policies = Vec::new();
    let mut callbacks = Vec::new();
    let mut metrics = Vec::new();
    for source in &cases {
        bound(assessments.len(), source.case.assessments.len())?;
        bound(callbacks.len(), source.case.callbacks.len())?;
        for (ordinal, a) in source.case.assessments.iter().enumerate() {
            if [
                &a.policy.jurisdiction_id,
                &a.policy.scheme_id,
                &a.policy.scheme_version,
                &a.policy.revision,
            ]
            .iter()
            .any(|id| id.trim().is_empty())
                || a.urgency.jurisdiction_id != a.policy.jurisdiction_id
                || a.urgency.scheme_id != a.policy.scheme_id
                || a.urgency.scheme_version != a.policy.scheme_version
            {
                return Err(invalid("captured urgency/policy identity mismatch"));
            }
            // A captured policy is a small configuration map rendered as an
            // ordered array. Duplicated category keys are invalid even when the
            // repeated rule has identical values; repeated snapshots are deduped
            // only after this source-local configuration check.
            let mut categories = BTreeSet::new();
            if a.policy
                .rules
                .iter()
                .any(|rule| !categories.insert(rule.category))
            {
                return Err(invalid("duplicate category in captured policy snapshot"));
            }
            let content = serde_json::to_string(&canonical(
                serde_json::to_value(&a.policy).map_err(invalid)?,
            ))
            .map_err(invalid)?;
            policies.push(PolicySource {
                policy: &a.policy,
                sha: hash(content.as_bytes()),
                content,
            });
            assessments.push((source, ordinal as u64, a));
        }
        callbacks.extend(
            source
                .case
                .callbacks
                .iter()
                .enumerate()
                .map(|(i, c)| (source, i as u64, c)),
        );
        for (kind, metric, tick) in [
            (
                "transfer",
                source.case.metrics.transfer.as_ref(),
                source.case.timings.departed,
            ),
            (
                "incision",
                source.case.metrics.incision.as_ref(),
                source.case.timings.incision,
            ),
            (
                "birth",
                source.case.metrics.birth.as_ref(),
                source.case.timings.birth,
            ),
        ] {
            if let Some(metric) = metric {
                bound(metrics.len(), 1)?;
                let tick = tick.ok_or_else(|| invalid("metric without captured endpoint"))?;
                metrics.push((
                    source,
                    kind,
                    metric,
                    tick,
                    difference(tick, source.case.decision_tick)?,
                ));
            }
        }
    }
    let mut rules = Vec::new();
    for policy in &policies {
        bound(rules.len(), policy.policy.rules.len())?;
        rules.extend(policy.policy.rules.iter().map(|rule| (policy, rule)));
    }
    let n = cases.len();
    let mut columns = vec![
        schema_column("careops.birthing.theatre-cases.v2", n),
        column!("run_id", cases, |s| s.context.run_id.as_str()),
        column!("replication_index", cases, |s| s.context.replication_index),
        column!("case_id", cases, |s| s.case.id),
        column!("episode_id", cases, |s| s.case.episode),
        column!("maternal_person_id", cases, |s| (s.row.person != 0)
            .then_some(s.row.person)),
        column!("capture_sequence", cases, |s| s.sequence),
        column!("transfer_metric_recorded", cases, |s| s
            .case
            .metrics
            .transfer
            .is_some()),
        column!("incision_metric_recorded", cases, |s| s
            .case
            .metrics
            .incision
            .is_some()),
        column!("birth_metric_recorded", cases, |s| s
            .case
            .metrics
            .birth
            .is_some()),
    ];
    ticks(
        &mut columns,
        "capture_tick",
        cases.iter().map(|s| Some(s.row.tick)).collect(),
    );
    ticks(
        &mut columns,
        "decision_tick",
        cases.iter().map(|s| Some(s.case.decision_tick)).collect(),
    );
    for (name, values) in [
        (
            "accepted_tick",
            cases.iter().map(|s| s.case.timings.accepted).collect(),
        ),
        (
            "departed_tick",
            cases.iter().map(|s| s.case.timings.departed).collect(),
        ),
        (
            "arrived_tick",
            cases.iter().map(|s| s.case.timings.arrived).collect(),
        ),
        (
            "started_tick",
            cases.iter().map(|s| s.case.timings.started).collect(),
        ),
        (
            "incision_tick",
            cases.iter().map(|s| s.case.timings.incision).collect(),
        ),
        (
            "birth_tick",
            cases.iter().map(|s| s.case.timings.birth).collect(),
        ),
        (
            "completed_tick",
            cases.iter().map(|s| s.case.timings.completed).collect(),
        ),
        (
            "cancelled_tick",
            cases.iter().map(|s| s.case.timings.cancelled).collect(),
        ),
        (
            "cleaned_tick",
            cases.iter().map(|s| s.case.timings.cleaned).collect(),
        ),
    ] {
        ticks(&mut columns, name, values);
    }
    let case_frame = DataFrame::new(n, columns).map_err(invalid)?;
    let n = assessments.len();
    let mut columns = vec![
        schema_column("careops.birthing.theatre-assessments.v2", n),
        column!("run_id", assessments, |(s, _, _)| s.context.run_id.as_str()),
        column!("replication_index", assessments, |(s, _, _)| s
            .context
            .replication_index),
        column!("case_id", assessments, |(s, _, _)| s.case.id),
        column!("assessment_ordinal", assessments, |(_, i, _)| *i),
        column!("capture_sequence", assessments, |(s, _, _)| s.sequence),
        column!("urgency_category", assessments, |(_, _, a)| u64::from(
            a.urgency.category
        )),
        column!("urgency_jurisdiction_id", assessments, |(_, _, a)| a
            .urgency
            .jurisdiction_id
            .as_str()),
        column!("urgency_scheme_id", assessments, |(_, _, a)| a
            .urgency
            .scheme_id
            .as_str()),
        column!("urgency_scheme_version", assessments, |(_, _, a)| a
            .urgency
            .scheme_version
            .as_str()),
        column!("urgency_effective_date", assessments, |(_, _, a)| a
            .urgency
            .effective_date
            .as_str()),
        column!("urgency_source_authority", assessments, |(_, _, a)| a
            .urgency
            .source_authority
            .as_str()),
        column!("urgency_citation", assessments, |(_, _, a)| a
            .urgency
            .citation
            .as_str()),
        column!("urgency_source_sha256", assessments, |(_, _, a)| a
            .urgency
            .source_sha256
            .as_deref()),
        column!("interruption", assessments, |(_, _, a)| a
            .interruption
            .as_str()),
        column!("selected_priority_rank", assessments, |(_, _, a)| a
            .rule
            .priority_rank),
        column!("policy_jurisdiction_id", assessments, |(_, _, a)| a
            .policy
            .jurisdiction_id
            .as_str()),
        column!("policy_scheme_id", assessments, |(_, _, a)| a
            .policy
            .scheme_id
            .as_str()),
        column!("policy_scheme_version", assessments, |(_, _, a)| a
            .policy
            .scheme_version
            .as_str()),
        column!("policy_revision", assessments, |(_, _, a)| a
            .policy
            .revision
            .as_str()),
        column!("policy_content_sha256", policies, |p| p.sha.as_str()),
        Series::new(
            "_expected_wait".into(),
            assessments
                .iter()
                .map(|(s, _, a)| difference(a.observed_at_tick, s.case.decision_tick))
                .collect::<Result<Vec<_>>>()?,
        )
        .into(),
    ];
    for (name, values) in [
        (
            "observed_at_tick",
            assessments
                .iter()
                .map(|(_, _, a)| Some(a.observed_at_tick))
                .collect(),
        ),
        (
            "elapsed_wait_ticks",
            assessments
                .iter()
                .map(|(_, _, a)| Some(a.elapsed_wait_ticks))
                .collect(),
        ),
        (
            "urgency_assessed_tick",
            assessments
                .iter()
                .map(|(_, _, a)| Some(a.urgency.assessed_at_tick))
                .collect(),
        ),
        (
            "selected_transfer_target_ticks",
            assessments
                .iter()
                .map(|(_, _, a)| a.rule.targets.transfer)
                .collect(),
        ),
        (
            "selected_incision_target_ticks",
            assessments
                .iter()
                .map(|(_, _, a)| a.rule.targets.incision)
                .collect(),
        ),
        (
            "selected_birth_target_ticks",
            assessments
                .iter()
                .map(|(_, _, a)| a.rule.targets.birth)
                .collect(),
        ),
    ] {
        ticks(&mut columns, name, values);
    }
    let assessment_frame = DataFrame::new(n, columns).map_err(invalid)?;
    let n = rules.len();
    let mut columns = vec![
        schema_column("careops.birthing.captured-policy-rules.v2", n),
        column!("policy_jurisdiction_id", rules, |(p, _)| p
            .policy
            .jurisdiction_id
            .as_str()),
        column!("policy_scheme_id", rules, |(p, _)| p
            .policy
            .scheme_id
            .as_str()),
        column!("policy_scheme_version", rules, |(p, _)| p
            .policy
            .scheme_version
            .as_str()),
        column!("policy_revision", rules, |(p, _)| p
            .policy
            .revision
            .as_str()),
        column!("policy_content_sha256", rules, |(p, _)| p.sha.as_str()),
        column!("policy_content_json", rules, |(p, _)| p.content.as_str()),
        column!("policy_source_authority", rules, |(p, _)| p
            .policy
            .source_authority
            .as_str()),
        column!("policy_citation", rules, |(p, _)| p
            .policy
            .citation
            .as_str()),
        column!("urgency_category", rules, |(_, r)| u64::from(r.category)),
        column!("policy_priority_rank", rules, |(_, r)| r.priority_rank),
    ];
    for (name, values) in [
        (
            "policy_transfer_target_ticks",
            rules.iter().map(|(_, r)| r.targets.transfer).collect(),
        ),
        (
            "policy_incision_target_ticks",
            rules.iter().map(|(_, r)| r.targets.incision).collect(),
        ),
        (
            "policy_birth_target_ticks",
            rules.iter().map(|(_, r)| r.targets.birth).collect(),
        ),
    ] {
        ticks(&mut columns, name, values);
    }
    let policy_frame = DataFrame::new(n, columns).map_err(invalid)?;
    let n = metrics.len();
    let mut columns = vec![
        schema_column("careops.birthing.theatre-metrics.v2", n),
        column!("run_id", metrics, |(s, _, _, _, _)| s
            .context
            .run_id
            .as_str()),
        column!("replication_index", metrics, |(s, _, _, _, _)| s
            .context
            .replication_index),
        column!("case_id", metrics, |(s, _, _, _, _)| s.case.id),
        column!("metric_kind", metrics, |(_, k, _, _, _)| *k),
        column!("capture_sequence", metrics, |(s, _, _, _, _)| s.sequence),
        column!("captured_policy_revision", metrics, |(_, _, m, _, _)| m
            .policy_revision
            .as_str()),
        column!("captured_scheme_version", metrics, |(_, _, m, _, _)| m
            .scheme_version
            .as_str()),
        column!("breached", metrics, |(_, _, m, _, _)| m.breached),
        column!("_expected_elapsed", metrics, |(_, _, _, _, e)| *e),
    ];
    for (name, values) in [
        (
            "assessment_tick",
            metrics
                .iter()
                .map(|(_, _, m, _, _)| Some(m.assessment_tick))
                .collect(),
        ),
        (
            "elapsed_ticks",
            metrics
                .iter()
                .map(|(_, _, m, _, _)| Some(m.elapsed))
                .collect(),
        ),
        (
            "target_ticks",
            metrics.iter().map(|(_, _, m, _, _)| m.target).collect(),
        ),
        (
            "event_tick",
            metrics
                .iter()
                .map(|(_, _, _, tick, _)| Some(*tick))
                .collect(),
        ),
    ] {
        ticks(&mut columns, name, values);
    }
    let metric_frame = DataFrame::new(n, columns).map_err(invalid)?;
    let n = callbacks.len();
    let mut columns = vec![
        schema_column("careops.birthing.theatre-callbacks.v2", n),
        column!("run_id", callbacks, |(s, _, _)| s.context.run_id.as_str()),
        column!("replication_index", callbacks, |(s, _, _)| s
            .context
            .replication_index),
        column!("case_id", callbacks, |(s, _, _)| s.case.id),
        column!("callback_ordinal", callbacks, |(_, i, _)| *i),
        column!("episode_id", callbacks, |(_, _, c)| c.episode),
        column!("callback_kind", callbacks, |(_, _, c)| c.callback.as_str()),
        column!("case_capture_sequence", callbacks, |(s, _, _)| s.sequence),
        Series::new("capture_source".into(), vec!["embedded_theatre_case"; n]).into(),
    ];
    ticks(
        &mut columns,
        "callback_tick",
        callbacks.iter().map(|(_, _, c)| Some(c.tick)).collect(),
    );
    let callback_frame = DataFrame::new(n, columns).map_err(invalid)?;
    let mut top_frame=df!("run_id"=>top.iter().map(|(s,_,_,_)|s.run_id.as_str()).collect::<Vec<_>>(),"replication_index"=>top.iter().map(|(s,_,_,_)|s.replication_index).collect::<Vec<_>>(),"case_id"=>top.iter().map(|(_,_,_,c)|c.id).collect::<Vec<_>>(),"episode_id"=>top.iter().map(|(_,_,r,_)|r.episode).collect::<Vec<_>>(),"callback_kind"=>top.iter().map(|(_,_,_,c)|c.callback.as_str()).collect::<Vec<_>>()).map_err(invalid)?;
    let mut tick_columns = Vec::new();
    ticks(
        &mut tick_columns,
        "callback_tick",
        top.iter().map(|(_, _, r, _)| Some(r.tick)).collect(),
    );
    for column in tick_columns {
        top_frame.with_column(column).map_err(invalid)?;
    }
    qualify(
        case_frame,
        assessment_frame,
        policy_frame,
        metric_frame,
        callback_frame,
        top_frame,
    )
}

fn key_exprs(names: &[&str]) -> Vec<Expr> {
    names.iter().map(|name| col(*name)).collect()
}
fn case_join(child: DataFrame, cases: &DataFrame) -> Result<DataFrame> {
    let rows = child.height();
    let mut projection = key_exprs(&CASE_KEYS);
    projection.push(col("episode_id").alias("case_episode_id"));
    let result = child
        .lazy()
        .join(
            cases
                .clone()
                .lazy()
                .select(projection)
                .with_column(lit(true).alias("_case_present")),
            key_exprs(&CASE_KEYS),
            key_exprs(&CASE_KEYS),
            JoinArgs::new(JoinType::Left),
        )
        .map_err(invalid)?
        .collect()
        .map_err(invalid)?;
    if result.height() != rows
        || result
            .column("_case_present")
            .map_err(invalid)?
            .null_count()
            != 0
    {
        return Err(invalid("orphan theatre child or changed join cardinality"));
    }
    Ok(result.drop_many(["_case_present"]))
}
// Current B7 emits scheduled callbacks as top-level rows. Boundary callbacks
// are retained only in the embedded case history; this is capture coverage,
// not a change to the theatre model's supported callback kinds.
fn scheduled_callback() -> Expr {
    col("callback_kind")
        .eq(lit("ProcedureStarted"))
        .or(col("callback_kind").eq(lit("Birth")))
        .or(col("callback_kind").eq(lit("ServiceCompleted")))
        .or(col("callback_kind").eq(lit("RoomReady")))
}
fn qualify(
    cases: DataFrame,
    assessments: DataFrame,
    policies: DataFrame,
    metrics: DataFrame,
    callbacks: DataFrame,
    top: DataFrame,
) -> Result<TheatreTables> {
    check_keys(&cases, &CASE_KEYS)?;
    check_keys(
        &assessments,
        &[
            "run_id",
            "replication_index",
            "case_id",
            "assessment_ordinal",
        ],
    )?;
    check_keys(
        &metrics,
        &["run_id", "replication_index", "case_id", "metric_kind"],
    )?;
    check_keys(
        &callbacks,
        &["run_id", "replication_index", "case_id", "callback_ordinal"],
    )?;
    let conflict = policies
        .clone()
        .lazy()
        .group_by(key_exprs(&POLICY_KEYS))
        .agg([col("policy_content_sha256").n_unique().alias("_contents")])
        .filter(col("_contents").gt(lit(1u32)))
        .limit(1)
        .collect()
        .map_err(invalid)?;
    if conflict.height() != 0 {
        return Err(invalid("conflicting captured policy content"));
    }
    let all_columns = policies
        .get_column_names()
        .iter()
        .map(|name| col(name.as_str()))
        .collect::<Vec<_>>();
    let policies = policies
        .lazy()
        .group_by(all_columns)
        .agg(Vec::<Expr>::new())
        .collect()
        .map_err(invalid)?;
    let mut policy_join_keys = POLICY_KEYS.to_vec();
    policy_join_keys.extend(["policy_content_sha256", "urgency_category"]);
    check_keys(&policies, &policy_join_keys)?;
    let assessments = case_join(assessments, &cases)?;
    let expected = assessments.height();
    let policy_projection = policies
        .get_column_names()
        .iter()
        .map(|name| {
            if name.as_str() == "schema" {
                col("schema").alias("policy_schema")
            } else {
                col(name.as_str())
            }
        })
        .collect::<Vec<_>>();
    let mut assessments = assessments
        .lazy()
        .join(
            policies
                .clone()
                .lazy()
                .select(policy_projection)
                .with_column(lit(true).alias("_policy_present")),
            key_exprs(&policy_join_keys),
            key_exprs(&policy_join_keys),
            JoinArgs::new(JoinType::Left),
        )
        .map_err(invalid)?
        .collect()
        .map_err(invalid)?;
    if assessments.height() != expected
        || assessments
            .column("_policy_present")
            .map_err(invalid)?
            .null_count()
            != 0
    {
        return Err(invalid(
            "missing captured policy/category or changed cardinality",
        ));
    }
    let mut wrong_rule = col("selected_priority_rank")
        .neq(col("policy_priority_rank"))
        .or(col("elapsed_wait_ticks_lo").neq(col("_expected_wait")));
    for kind in ["transfer", "incision", "birth"] {
        for limb in ["hi", "lo"] {
            wrong_rule = wrong_rule.or(col(format!("selected_{kind}_target_ticks_{limb}"))
                .eq_missing(col(format!("policy_{kind}_target_ticks_{limb}")))
                .not());
        }
    }
    if assessments
        .clone()
        .lazy()
        .filter(wrong_rule)
        .limit(1)
        .collect()
        .map_err(invalid)?
        .height()
        != 0
    {
        return Err(invalid(
            "selected rule/wait differs from captured policy and clock",
        ));
    }
    assessments = assessments.drop_many(["_policy_present", "_expected_wait"]);
    let callbacks = case_join(callbacks, &cases)?
        .lazy()
        .with_column(
            when(scheduled_callback())
                .then(lit("scheduled_overlap"))
                .otherwise(lit("not_emitted_at_top_level"))
                .alias("top_level_capture"),
        )
        .collect()
        .map_err(invalid)?;
    let timing_fields = [
        ("Accepted", "accepted_tick"),
        ("TransferDeparted", "departed_tick"),
        ("TransferArrived", "arrived_tick"),
        ("ServiceStarted", "started_tick"),
        ("ProcedureStarted", "incision_tick"),
        ("Birth", "birth_tick"),
        ("ServiceCompleted", "completed_tick"),
        ("RoomReady", "cleaned_tick"),
        ("Cancelled", "cancelled_tick"),
    ];
    let mut projection = key_exprs(&CASE_KEYS);
    for (_, field) in timing_fields {
        for limb in ["hi", "lo"] {
            projection.push(col(format!("{field}_{limb}")));
        }
    }
    let rows = callbacks.height();
    let timed = callbacks
        .clone()
        .lazy()
        .join(
            cases.clone().lazy().select(projection),
            key_exprs(&CASE_KEYS),
            key_exprs(&CASE_KEYS),
            JoinArgs::new(JoinType::Left),
        )
        .map_err(invalid)?
        .collect()
        .map_err(invalid)?;
    if timed.height() != rows {
        return Err(invalid("callback timing join changed cardinality"));
    }
    let expected = |limb: &str| {
        let mut expression = lit(NULL).cast(DataType::UInt64);
        for (kind, field) in timing_fields.into_iter().rev() {
            expression = when(col("callback_kind").eq(lit(kind)))
                .then(col(format!("{field}_{limb}")))
                .otherwise(expression);
        }
        expression
    };
    let expected_hi = expected("hi");
    let expected_lo = expected("lo");
    if timed
        .lazy()
        .filter(
            expected_hi
                .clone()
                .is_null()
                .or(expected_lo.clone().is_null())
                .or(col("callback_tick_hi").neq(expected_hi))
                .or(col("callback_tick_lo").neq(expected_lo)),
        )
        .limit(1)
        .collect()
        .map_err(invalid)?
        .height()
        != 0
    {
        return Err(invalid("callback has no matching captured case timestamp"));
    }
    let top = case_join(top, &cases)?;
    for frame in [&callbacks, &top] {
        let mut known = col("callback_kind").eq(lit("Accepted"));
        for name in [
            "TransferDeparted",
            "TransferArrived",
            "ServiceStarted",
            "ProcedureStarted",
            "Birth",
            "ServiceCompleted",
            "RoomReady",
            "Cancelled",
        ] {
            known = known.or(col("callback_kind").eq(lit(name)));
        }
        if frame
            .clone()
            .lazy()
            .filter(
                known
                    .not()
                    .or(col("episode_id").neq(col("case_episode_id"))),
            )
            .limit(1)
            .collect()
            .map_err(invalid)?
            .height()
            != 0
        {
            return Err(invalid("invalid callback kind or case episode"));
        }
    }
    if top
        .clone()
        .lazy()
        .filter(scheduled_callback().not())
        .limit(1)
        .collect()
        .map_err(invalid)?
        .height()
        != 0
    {
        return Err(invalid("unsupported top-level callback coverage"));
    }
    let callback_keys = [
        "run_id",
        "replication_index",
        "case_id",
        "episode_id",
        "callback_kind",
        "callback_tick_hi",
        "callback_tick_lo",
    ];
    let counts = |frame: DataFrame| -> Result<DataFrame> {
        frame
            .lazy()
            .group_by(key_exprs(&callback_keys))
            .agg([len().alias("_rows")])
            .sort(callback_keys, SortMultipleOptions::default())
            .collect()
            .map_err(invalid)
    };
    let scheduled = callbacks
        .clone()
        .lazy()
        .filter(scheduled_callback())
        .collect()
        .map_err(invalid)?;
    if !counts(scheduled)?.equals_missing(&counts(top)?) {
        return Err(invalid("embedded/top-level callback multiset mismatch"));
    }
    let metrics = case_join(metrics, &cases)?;
    if metrics
        .clone()
        .lazy()
        .filter(col("assessment_tick_lo").gt(col("event_tick_lo")))
        .limit(1)
        .collect()
        .map_err(invalid)?
        .height()
        != 0
    {
        return Err(invalid("metric references a future assessment"));
    }
    // Verify the captured assessment time against the available history without
    // reselecting a policy or choosing an ordinal among equal-time candidates.
    let history_counts = assessments
        .clone()
        .lazy()
        .group_by(key_exprs(&CASE_KEYS))
        .agg([len().cast(DataType::UInt64).alias("_history_rows")])
        .collect()
        .map_err(invalid)?;
    let budget = metrics
        .clone()
        .lazy()
        .join(
            history_counts.lazy(),
            key_exprs(&CASE_KEYS),
            key_exprs(&CASE_KEYS),
            JoinArgs::new(JoinType::Left),
        )
        .map_err(invalid)?
        .collect()
        .map_err(invalid)?;
    if budget
        .column("_history_rows")
        .map_err(invalid)?
        .null_count()
        != 0
    {
        return Err(invalid("metric without assessment history"));
    }
    let expanded = budget
        .column("_history_rows")
        .map_err(invalid)?
        .u64()
        .map_err(invalid)?
        .sum()
        .unwrap_or(0);
    if expanded > MAX_VIEW_ROWS as u64 {
        return Err(invalid("assessment history join exceeds bound"));
    }
    let history = assessments.clone().lazy().select([
        col("run_id"),
        col("replication_index"),
        col("case_id"),
        col("observed_at_tick_lo").alias("_available_tick"),
    ]);
    let joined = metrics
        .clone()
        .lazy()
        .join(
            history,
            key_exprs(&CASE_KEYS),
            key_exprs(&CASE_KEYS),
            JoinArgs::new(JoinType::Left),
        )
        .map_err(invalid)?
        .collect()
        .map_err(invalid)?;
    if joined.height() != expanded as usize {
        return Err(invalid("changed history join cardinality"));
    }
    let metric_keys = ["run_id", "replication_index", "case_id", "metric_kind"];
    let latest = joined
        .lazy()
        .filter(col("_available_tick").lt(col("event_tick_lo")))
        .group_by(key_exprs(&metric_keys))
        .agg([col("_available_tick").max().alias("_latest_tick")]);
    let checked = metrics
        .clone()
        .lazy()
        .join(
            latest,
            key_exprs(&metric_keys),
            key_exprs(&metric_keys),
            JoinArgs::new(JoinType::Left),
        )
        .map_err(invalid)?
        .collect()
        .map_err(invalid)?;
    if checked.height() != metrics.height()
        || checked
            .lazy()
            .filter(
                col("_latest_tick")
                    .is_not_null()
                    .and(col("assessment_tick_lo").lt(col("_latest_tick"))),
            )
            .limit(1)
            .collect()
            .map_err(invalid)?
            .height()
            != 0
    {
        return Err(invalid(
            "metric predates a strictly earlier available assessment",
        ));
    }
    // An assessment at the event's own tick may have been captured after the
    // callback. Without that ordering field, do not force it onto an earlier
    // immutable metric or manufacture an assessment ordinal.

    let expected_breach = when(col("target_ticks_lo").is_null())
        .then(lit(NULL).cast(DataType::Boolean))
        .otherwise(col("elapsed_ticks_lo").gt(col("target_ticks_lo")));
    if metrics
        .clone()
        .lazy()
        .filter(
            col("elapsed_ticks_lo")
                .neq(col("_expected_elapsed"))
                .or(col("breached").eq_missing(expected_breach).not()),
        )
        .limit(1)
        .collect()
        .map_err(invalid)?
        .height()
        != 0
    {
        return Err(invalid("captured metric elapsed/target/breach mismatch"));
    }
    let reference_keys = [
        "run_id",
        "replication_index",
        "case_id",
        "assessment_tick_hi",
        "assessment_tick_lo",
        "captured_policy_revision",
        "captured_scheme_version",
    ];
    let mut projection = key_exprs(&CASE_KEYS);
    projection.extend([
        col("observed_at_tick_hi").alias("assessment_tick_hi"),
        col("observed_at_tick_lo").alias("assessment_tick_lo"),
        col("policy_revision").alias("captured_policy_revision"),
        col("policy_scheme_version").alias("captured_scheme_version"),
        col("assessment_ordinal"),
        col("policy_jurisdiction_id"),
        col("policy_scheme_id"),
        col("policy_scheme_version"),
        col("policy_revision"),
        col("policy_content_sha256"),
    ]);
    for kind in ["transfer", "incision", "birth"] {
        for limb in ["hi", "lo"] {
            projection.push(col(format!("selected_{kind}_target_ticks_{limb}")));
        }
    }
    let candidates = assessments
        .clone()
        .lazy()
        .select(projection)
        .collect()
        .map_err(invalid)?;
    let candidate_counts = candidates
        .clone()
        .lazy()
        .group_by(key_exprs(&reference_keys))
        .agg([len()
            .cast(DataType::UInt64)
            .alias("assessment_candidate_count")])
        .collect()
        .map_err(invalid)?;
    let rows = metrics.height();
    let metrics = metrics
        .lazy()
        .join(
            candidate_counts.clone().lazy(),
            key_exprs(&reference_keys),
            key_exprs(&reference_keys),
            JoinArgs::new(JoinType::Left),
        )
        .map_err(invalid)?
        .collect()
        .map_err(invalid)?;
    if metrics.height() != rows
        || metrics
            .column("assessment_candidate_count")
            .map_err(invalid)?
            .null_count()
            != 0
    {
        return Err(invalid("orphan captured metric assessment reference"));
    }
    // Bound the candidate expansion before materializing it. With both operands
    // capped at MAX_VIEW_ROWS, this UInt64 sum cannot overflow.
    let expanded = metrics
        .column("assessment_candidate_count")
        .map_err(invalid)?
        .u64()
        .map_err(invalid)?
        .sum()
        .unwrap_or(0);
    if expanded > MAX_VIEW_ROWS as u64 {
        return Err(invalid("metric candidate expansion exceeds bound"));
    }
    let joined = metrics
        .clone()
        .lazy()
        .join(
            candidates.clone().lazy(),
            key_exprs(&reference_keys),
            key_exprs(&reference_keys),
            JoinArgs::new(JoinType::Left),
        )
        .map_err(invalid)?
        .collect()
        .map_err(invalid)?;
    if joined.height() != expanded as usize {
        return Err(invalid("changed metric candidate cardinality"));
    }
    let selected_target = |limb: &str| {
        when(col("metric_kind").eq(lit("transfer")))
            .then(col(format!("selected_transfer_target_ticks_{limb}")))
            .when(col("metric_kind").eq(lit("incision")))
            .then(col(format!("selected_incision_target_ticks_{limb}")))
            .otherwise(col(format!("selected_birth_target_ticks_{limb}")))
    };
    let target_matches = col("target_ticks_hi")
        .eq_missing(selected_target("hi"))
        .and(col("target_ticks_lo").eq_missing(selected_target("lo")));
    let metric_keys = ["run_id", "replication_index", "case_id", "metric_kind"];
    if joined
        .lazy()
        .with_column(target_matches.alias("_target_matches"))
        .group_by(key_exprs(&metric_keys))
        .agg([col("_target_matches").sum().alias("_matches")])
        .filter(col("_matches").eq(lit(0u32)))
        .limit(1)
        .collect()
        .map_err(invalid)?
        .height()
        != 0
    {
        return Err(invalid(
            "metric target differs from every captured assessment candidate",
        ));
    }
    let mut projection = key_exprs(&reference_keys);
    projection.push(col("assessment_ordinal").alias("resolved_assessment_ordinal"));
    for name in POLICY_KEYS {
        projection.push(col(name).alias(format!("resolved_{name}")));
    }
    projection.push(col("policy_content_sha256").alias("resolved_policy_content_sha256"));
    let unique = candidates
        .lazy()
        .join(
            candidate_counts
                .lazy()
                .filter(col("assessment_candidate_count").eq(lit(1u64)))
                .select(key_exprs(&reference_keys)),
            key_exprs(&reference_keys),
            key_exprs(&reference_keys),
            JoinArgs::new(JoinType::Inner),
        )
        .map_err(invalid)?
        .select(projection)
        .collect()
        .map_err(invalid)?;
    check_keys(&unique, &reference_keys)?;
    let metrics = metrics
        .lazy()
        .join(
            unique.lazy(),
            key_exprs(&reference_keys),
            key_exprs(&reference_keys),
            JoinArgs::new(JoinType::Left),
        )
        .map_err(invalid)?
        .with_column(
            when(col("assessment_candidate_count").eq(lit(1u64)))
                .then(lit("resolved"))
                .otherwise(lit("ambiguous"))
                .alias("assessment_reference_status"),
        )
        .collect()
        .map_err(invalid)?
        .drop_many(["_expected_elapsed"]);
    if metrics.height() != rows {
        return Err(invalid("metric resolution changed row count"));
    }
    let sort = |frame: DataFrame, names: &[&str]| -> Result<DataFrame> {
        frame
            .sort(names.iter().copied(), SortMultipleOptions::default())
            .map_err(invalid)
    };
    Ok(TheatreTables {
        cases: sort(cases, &CASE_KEYS)?,
        assessments: sort(
            assessments,
            &[
                "run_id",
                "replication_index",
                "case_id",
                "assessment_ordinal",
            ],
        )?,
        policy_rules: sort(policies, &policy_join_keys)?,
        metrics: sort(
            metrics,
            &["run_id", "replication_index", "case_id", "metric_kind"],
        )?,
        callbacks: sort(
            callbacks,
            &["run_id", "replication_index", "case_id", "callback_ordinal"],
        )?,
    })
}

/// Checked full-batch entry: preserve original row sequences while Polars selects
/// the two captured payload kinds. Other details remain in replication_events.
pub fn read_theatre_tables(
    batch_directory: &Path,
    input: &BatchInput,
    replications: &[Replication],
) -> Result<(Vec<ReplicationManifest>, TheatreTables)> {
    input.validate().map_err(invalid)?;
    if replications.len() != input.replications
        || replications.iter().enumerate().any(|(i, r)| r.index != i)
    {
        return Err(invalid("incomplete or unordered replication selection"));
    }
    let declared: Vec<Replication> = serde_json::from_slice(
        &crate::read_bounded(&batch_directory.join("batch_manifest.json"), 262144)
            .map_err(invalid)?,
    )
    .map_err(invalid)?;
    if declared.as_slice() != replications {
        return Err(invalid("parent batch receipt declaration mismatch"));
    }
    let mut manifests = Vec::new();
    let mut captures = Vec::new();
    for replication in replications {
        let (manifest, events) = replication_events(
            batch_directory,
            &replication.directory,
            input,
            replication.index,
            replication.seed,
        )?;
        if manifest.replication != *replication {
            return Err(invalid("supplied replication receipt mismatch"));
        }
        let run_id = events
            .column("run_id")
            .map_err(invalid)?
            .str()
            .map_err(invalid)?
            .get(0)
            .ok_or_else(|| invalid("empty checked capture"))?
            .to_owned();
        let selected = events
            .lazy()
            .filter(
                col("event_kind")
                    .eq(lit("theatre_case"))
                    .or(col("event_kind").eq(lit("theatre_callback"))),
            )
            .select([col("sequence"), col("payload_ref")])
            .collect()
            .map_err(invalid)?;
        let sequence = selected
            .column("sequence")
            .map_err(invalid)?
            .u64()
            .map_err(invalid)?;
        let payloads = selected
            .column("payload_ref")
            .map_err(invalid)?
            .str()
            .map_err(invalid)?;
        let mut rows = Vec::with_capacity(selected.height());
        for i in 0..selected.height() {
            let payload = payloads
                .get(i)
                .ok_or_else(|| invalid("missing captured payload"))?;
            if payload.len() > 8 * 1024 * 1024 {
                return Err(invalid("captured theatre payload exceeds bound"));
            }
            rows.push((
                sequence
                    .get(i)
                    .ok_or_else(|| invalid("missing capture sequence"))?,
                serde_json::from_str(payload).map_err(invalid)?,
            ));
        }
        captures.push(TheatreCapture {
            run_id,
            replication_index: replication.index as u64,
            events: rows,
        });
        manifests.push(manifest);
    }
    let tables = theatre_tables(&captures)?;
    Ok((manifests, tables))
}
