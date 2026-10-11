//! Experimental synthetic ED triage and assessment runtime.
//!
//! Supplied stage durations are synthetic tick work, not clinical estimates.
//! This module is an experimental pathway fixture and makes no admission,
//! discharge, staffing-identity, or operational-validity claim.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

use kairo_ecs_core::Scheduler;
use kairo_ecs_types::{EntityId, EventKind, ScheduleRequest, SimTime, StepOutcome};
use serde::{Deserialize, Serialize};

use crate::arrivals::GeneratedArrival;
use crate::{validate_capacity_work, CapacityLocationProfile, PatientWorkItem, Provenance};

const ARRIVAL_EVENT: u32 = 1;
const TRIAGE_COMPLETE_EVENT: u32 = 2;
const ASSESSMENT_COMPLETE_EVENT: u32 = 3;
const MAX_PATIENTS: usize = 100_000;
const MAX_PROFILE_ROWS: usize = 1_024;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PathwayConfig {
    pub version: u32,
    pub horizon_ticks: u64,
    pub provenance: Provenance,
    pub capacity_location: CapacityLocationProfile,
    pub patients: Vec<PathwayPatient>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PathwayPatient {
    pub patient_id: String,
    pub arrival_tick: u64,
    pub mode: String,
    pub acuity: u8,
    pub stages: Vec<StageWork>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StageKind {
    Triage,
    Assessment,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageWork {
    pub kind: StageKind,
    pub resource_id: String,
    pub task_class_id: String,
    pub duration_ticks: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct StageEpisode {
    pub task_id: String,
    pub kind: StageKind,
    pub queue_entry_tick: u64,
    pub start_tick: Option<u64>,
    pub completion_tick: Option<u64>,
    pub observed_work_ticks: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PathwayState {
    Queued(StageKind),
    Active(StageKind),
    AssessmentComplete,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PathwayResult {
    pub patient_id: String,
    pub mode: String,
    pub acuity: u8,
    pub known_acuity: Option<u8>,
    pub arrival_tick: u64,
    pub episodes: Vec<StageEpisode>,
    pub state: PathwayState,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageTemplate {
    pub mode: String,
    pub acuity: u8,
    pub stages: Vec<StageWork>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PathwayError {
    InvalidInput(String),
    Runtime(String),
}

impl Display for PathwayError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(message) => write!(formatter, "invalid pathway: {message}"),
            Self::Runtime(message) => write!(formatter, "pathway runtime failed: {message}"),
        }
    }
}

impl Error for PathwayError {}

#[derive(Clone, Debug)]
struct ResourceRuntime {
    capacity: u64,
    busy: u64,
    waiting: BTreeSet<(u64, usize, usize)>,
}

/// Run supplied, synthetic triage and assessment durations on Kairos.
pub fn run_pathway(config: &PathwayConfig) -> Result<Vec<PathwayResult>, PathwayError> {
    validate_config(config)?;

    let mut patients = config.patients.clone();
    patients.sort_by(|left, right| left.patient_id.cmp(&right.patient_id));
    let mut results: Vec<PathwayResult> = patients
        .iter()
        .map(|patient| PathwayResult {
            patient_id: patient.patient_id.clone(),
            mode: patient.mode.clone(),
            acuity: patient.acuity,
            known_acuity: None,
            arrival_tick: patient.arrival_tick,
            episodes: Vec::with_capacity(2),
            state: PathwayState::Queued(StageKind::Triage),
        })
        .collect();
    let mut resources = BTreeMap::<String, ResourceRuntime>::new();
    for resource in &config.capacity_location.resource_buckets {
        let capacity = resource
            .open_count
            .unwrap_or(0)
            .min(resource.staffed_count.unwrap_or(0));
        resources.insert(
            resource.resource_id.clone(),
            ResourceRuntime {
                capacity,
                busy: 0,
                waiting: BTreeSet::new(),
            },
        );
    }

    let mut scheduler = Scheduler::new();
    for (index, patient) in patients.iter().enumerate() {
        scheduler.schedule(ScheduleRequest {
            at: SimTime::from_ticks(u128::from(patient.arrival_tick)),
            priority: 0,
            entity: Some(EntityId::new(index as u64, 0)),
            kind: EventKind::custom(ARRIVAL_EVENT),
        });
    }

    while let Some(next) = scheduler.peek_next() {
        let tick = next.at.ticks();
        if tick > u128::from(config.horizon_ticks) {
            break;
        }
        while let Some(preview) = scheduler.peek_next() {
            if preview.at.ticks() != tick {
                break;
            }
            let event = match scheduler.step() {
                StepOutcome::Dispatched(event) => event,
                StepOutcome::Empty | StepOutcome::LimitReached => {
                    return Err(PathwayError::Runtime(
                        "scheduler stopped during a tick batch".into(),
                    ));
                }
            };
            let patient_index = event
                .entity
                .ok_or_else(|| {
                    PathwayError::Runtime("scheduler event has no patient entity".into())
                })?
                .index as usize;
            let patient = patients.get(patient_index).ok_or_else(|| {
                PathwayError::Runtime("scheduler event has an unknown patient".into())
            })?;
            let now = u64::try_from(tick)
                .map_err(|_| PathwayError::Runtime("event tick exceeds pathway range".into()))?;
            match event.kind.code() {
                ARRIVAL_EVENT => {
                    enqueue_stage(
                        patient_index,
                        0,
                        now,
                        &patients,
                        &mut results,
                        &mut resources,
                    )?;
                }
                TRIAGE_COMPLETE_EVENT | ASSESSMENT_COMPLETE_EVENT => {
                    let stage_index = usize::from(event.kind.code() == ASSESSMENT_COMPLETE_EVENT);
                    let stage = patient.stages.get(stage_index).ok_or_else(|| {
                        PathwayError::Runtime("completion refers to missing stage".into())
                    })?;
                    let episode = results[patient_index].episodes.last_mut().ok_or_else(|| {
                        PathwayError::Runtime("completion has no active episode".into())
                    })?;
                    if episode.kind != stage.kind || episode.completion_tick.is_some() {
                        return Err(PathwayError::Runtime("completion state mismatch".into()));
                    }
                    episode.completion_tick = Some(now);
                    episode.observed_work_ticks = stage.duration_ticks;
                    let resource = resources.get_mut(&stage.resource_id).ok_or_else(|| {
                        PathwayError::Runtime("active resource disappeared".into())
                    })?;
                    resource.busy = resource.busy.checked_sub(1).ok_or_else(|| {
                        PathwayError::Runtime("resource busy count underflow".into())
                    })?;
                    if stage_index == 0 {
                        results[patient_index].known_acuity = Some(patient.acuity);
                        enqueue_stage(
                            patient_index,
                            1,
                            now,
                            &patients,
                            &mut results,
                            &mut resources,
                        )?;
                    } else {
                        results[patient_index].state = PathwayState::AssessmentComplete;
                    }
                }
                _ => return Err(PathwayError::Runtime("unknown scheduler event kind".into())),
            }
        }
        grant_waiting(
            tick,
            config.horizon_ticks,
            &patients,
            &mut results,
            &mut resources,
            &mut scheduler,
        )?;
    }

    let horizon = config.horizon_ticks;
    for result in &mut results {
        if let Some(episode) = result.episodes.last_mut() {
            if episode.completion_tick.is_none() {
                if let Some(start) = episode.start_tick {
                    episode.observed_work_ticks = horizon.saturating_sub(start);
                    result.state = PathwayState::Active(episode.kind);
                } else {
                    result.state = PathwayState::Queued(episode.kind);
                }
            }
        }
    }
    Ok(results)
}

fn enqueue_stage(
    patient_index: usize,
    stage_index: usize,
    tick: u64,
    patients: &[PathwayPatient],
    results: &mut [PathwayResult],
    resources: &mut BTreeMap<String, ResourceRuntime>,
) -> Result<(), PathwayError> {
    let patient = &patients[patient_index];
    let stage = &patient.stages[stage_index];
    let episode = StageEpisode {
        task_id: format!("{}:{}", patient.patient_id, stage_name(stage.kind)),
        kind: stage.kind,
        queue_entry_tick: tick,
        start_tick: None,
        completion_tick: None,
        observed_work_ticks: 0,
    };
    results[patient_index].episodes.push(episode);
    results[patient_index].state = PathwayState::Queued(stage.kind);
    let resource = resources
        .get_mut(&stage.resource_id)
        .ok_or_else(|| PathwayError::Runtime("validated resource disappeared".into()))?;
    resource.waiting.insert((tick, patient_index, stage_index));
    Ok(())
}

fn grant_waiting(
    now: u128,
    horizon: u64,
    patients: &[PathwayPatient],
    results: &mut [PathwayResult],
    resources: &mut BTreeMap<String, ResourceRuntime>,
    scheduler: &mut Scheduler,
) -> Result<(), PathwayError> {
    for resource in resources.values_mut() {
        while resource.busy < resource.capacity && !resource.waiting.is_empty() {
            let (_, patient_index, stage_index) =
                resource.waiting.pop_first().ok_or_else(|| {
                    PathwayError::Runtime("queued work disappeared while granting".into())
                })?;
            let patient = &patients[patient_index];
            let stage = &patient.stages[stage_index];
            let start = u64::try_from(now)
                .map_err(|_| PathwayError::Runtime("start tick overflow".into()))?;
            let completion = now + u128::from(stage.duration_ticks);
            let episode = results[patient_index]
                .episodes
                .last_mut()
                .ok_or_else(|| PathwayError::Runtime("grant has no queued episode".into()))?;
            if episode.start_tick.is_some() {
                return Err(PathwayError::Runtime(
                    "episode started more than once".into(),
                ));
            }
            episode.start_tick = Some(start);
            results[patient_index].state = PathwayState::Active(stage.kind);
            resource.busy += 1;
            if completion <= u128::from(horizon) {
                scheduler.schedule(ScheduleRequest {
                    at: SimTime::from_ticks(completion),
                    priority: 0,
                    entity: Some(EntityId::new(patient_index as u64, 0)),
                    kind: EventKind::custom(match stage.kind {
                        StageKind::Triage => TRIAGE_COMPLETE_EVENT,
                        StageKind::Assessment => ASSESSMENT_COMPLETE_EVENT,
                    }),
                });
            }
        }
    }
    Ok(())
}

fn validate_config(config: &PathwayConfig) -> Result<(), PathwayError> {
    let invalid = |message: &str| Err(PathwayError::InvalidInput(message.to_owned()));
    if config.version != 1 {
        return invalid("version must be 1");
    }
    if config.horizon_ticks == 0 {
        return invalid("horizon_ticks must be positive");
    }
    if config.provenance.class != "synthetic"
        || config.provenance.notes.trim().is_empty()
        || config.provenance.notes.trim() != config.provenance.notes
    {
        return invalid("provenance must be synthetic with nonempty notes");
    }
    if config.patients.len() > MAX_PATIENTS {
        return invalid("patient limit exceeded");
    }
    let profile = &config.capacity_location;
    if profile.zones.len() > MAX_PROFILE_ROWS
        || profile.locations.len() > MAX_PROFILE_ROWS
        || profile.resource_buckets.len() > MAX_PROFILE_ROWS
        || profile.task_classes.len() > MAX_PROFILE_ROWS
        || profile.staff_roles.len() > MAX_PROFILE_ROWS
        || profile.eligibility.len() > MAX_PROFILE_ROWS
        || profile.routes.len() > MAX_PROFILE_ROWS
        || profile
            .staff_roles
            .iter()
            .any(|role| role.counts.len() > 32)
        || profile
            .resource_buckets
            .iter()
            .any(|resource| resource.capabilities.len() > 32)
        || profile
            .task_classes
            .iter()
            .any(|task| task.required_resource_classes.len() > 32)
        || profile.capacity.equipment.len() > 32
    {
        return invalid("capacity profile limit exceeded");
    }
    if !valid_profile_identifiers(profile) {
        return invalid("capacity profile identifiers/references must be valid identifiers");
    }
    if profile.capacity.physical_status != crate::KnowledgeStatus::Known
        || profile
            .capacity
            .physical_count
            .is_none_or(|count| count == 0)
        || profile.capacity.availability_status != crate::KnowledgeStatus::Known
        || profile.capacity.open_count.is_none_or(|count| count == 0)
        || profile
            .capacity
            .staffed_count
            .is_none_or(|count| count == 0)
        || profile.resource_buckets.iter().any(|resource| {
            resource.physical_status != crate::KnowledgeStatus::Known
                || resource.physical_count.is_none_or(|count| count == 0)
                || resource.availability_status != crate::KnowledgeStatus::Known
                || resource.open_count.is_none_or(|count| count == 0)
                || resource.staffed_count.is_none_or(|count| count == 0)
        })
    {
        return invalid("E1 requires known positive physical, open, and staffed capacity");
    }

    let mut seen_patients = HashSet::new();
    let mut cross_kind = HashMap::<&str, StageKind>::new();
    let mut validation_work = Vec::new();
    for (index, patient) in config.patients.iter().enumerate() {
        if !valid_identifier(&patient.patient_id) || patient.patient_id.contains(':') {
            return invalid("patient_id must be a valid identifier without ':'");
        }
        if !seen_patients.insert(patient.patient_id.as_str()) {
            return invalid("patient IDs must be unique");
        }
        if !valid_identifier(&patient.mode) {
            return invalid("mode must be a valid identifier");
        }
        if !(1..=5).contains(&patient.acuity) {
            return invalid("acuity must be 1..5");
        }
        if patient.arrival_tick >= config.horizon_ticks {
            return invalid("arrival must be before horizon");
        }
        if patient.stages.len() != 2
            || patient.stages[0].kind != StageKind::Triage
            || patient.stages[1].kind != StageKind::Assessment
        {
            return invalid("stages must be exactly triage then assessment");
        }
        for (stage_index, stage) in patient.stages.iter().enumerate() {
            if !valid_identifier(&stage.resource_id) || !valid_identifier(&stage.task_class_id) {
                return invalid("stage resource and task references must be valid identifiers");
            }
            if stage.duration_ticks == 0 {
                return invalid("stage duration must be positive");
            }
            if let Some(previous) = cross_kind.insert(&stage.resource_id, stage.kind) {
                if previous != stage.kind {
                    return invalid("a resource cannot serve both stage kinds");
                }
            }
            validation_work.push(PatientWorkItem {
                patient_id: format!("validation-{index}-{stage_index}"),
                arrival_tick: patient.arrival_tick,
                work_ticks: 1,
                resource_id: stage.resource_id.clone(),
                task_class_id: stage.task_class_id.clone(),
            });
            let _completion = u128::from(patient.arrival_tick) + u128::from(stage.duration_ticks);
        }
    }
    validate_capacity_work(profile, &validation_work, config.horizon_ticks)
        .map_err(|error| PathwayError::InvalidInput(error.to_string()))?;
    for patient in &config.patients {
        for stage in &patient.stages {
            let resource = profile
                .resource_buckets
                .iter()
                .find(|item| item.resource_id == stage.resource_id)
                .ok_or_else(|| PathwayError::InvalidInput("unknown resource".into()))?;
            let task = profile
                .task_classes
                .iter()
                .find(|item| item.task_class_id == stage.task_class_id)
                .ok_or_else(|| PathwayError::InvalidInput("unknown task class".into()))?;
            if !task.required_resource_classes.is_empty()
                && !task
                    .required_resource_classes
                    .contains(&resource.resource_class)
            {
                return invalid("task class is incompatible with resource class");
            }
            if !profile.eligibility.iter().any(|entry| {
                entry.task_class_id == stage.task_class_id
                    && entry.zone_id == resource.zone_id
                    && entry
                        .location_id
                        .as_ref()
                        .is_none_or(|location| location == &resource.location_id)
            }) {
                return invalid("stage has no matching role/zone/location eligibility");
            }
        }
    }
    Ok(())
}

fn valid_identifier(value: &str) -> bool {
    let bytes = value.as_bytes();
    !bytes.is_empty()
        && bytes.len() <= 64
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(*byte, b'_' | b'.' | b'-'))
}

fn valid_profile_identifiers(profile: &CapacityLocationProfile) -> bool {
    let valid = |value: &str| valid_identifier(value);
    profile.zones.iter().all(|zone| valid(&zone.zone_id))
        && profile
            .locations
            .iter()
            .all(|location| valid(&location.location_id) && valid(&location.zone_id))
        && profile.resource_buckets.iter().all(|resource| {
            valid(&resource.resource_id)
                && valid(&resource.resource_class)
                && valid(&resource.location_id)
                && valid(&resource.zone_id)
                && resource.pool_group_id.as_deref().is_none_or(valid)
                && resource
                    .capabilities
                    .iter()
                    .all(|capability| valid(capability))
        })
        && profile.task_classes.iter().all(|task| {
            valid(&task.task_class_id)
                && task
                    .required_resource_classes
                    .iter()
                    .all(|class| valid(class))
        })
        && profile.staff_roles.iter().all(|role| {
            valid(&role.staff_role_id) && role.counts.iter().all(|count| valid(&count.reference))
        })
        && profile.eligibility.iter().all(|entry| {
            valid(&entry.staff_role_id)
                && valid(&entry.task_class_id)
                && valid(&entry.zone_id)
                && entry.location_id.as_deref().is_none_or(valid)
        })
        && profile
            .routes
            .iter()
            .all(|route| valid(&route.from_location_id) && valid(&route.to_location_id))
        && profile
            .capacity
            .equipment
            .iter()
            .all(|item| valid(&item.capability))
}

fn stage_name(kind: StageKind) -> &'static str {
    match kind {
        StageKind::Triage => "triage",
        StageKind::Assessment => "assessment",
    }
}

/// Bind generator output to explicit supplied templates; acuity mapping is never inferred.
pub fn bind_arrivals(
    arrivals: &[GeneratedArrival],
    templates: &[StageTemplate],
) -> Result<Vec<PathwayPatient>, PathwayError> {
    if arrivals.len() > MAX_PATIENTS {
        return Err(PathwayError::InvalidInput("patient limit exceeded".into()));
    }
    if templates.len() > MAX_PROFILE_ROWS {
        return Err(PathwayError::InvalidInput("template limit exceeded".into()));
    }
    let mut by_key = HashMap::new();
    for template in templates {
        if !valid_identifier(&template.mode)
            || !(1..=5).contains(&template.acuity)
            || !valid_template_stages(&template.stages)
        {
            return Err(PathwayError::InvalidInput(
                "template key or stages are invalid".into(),
            ));
        }
        if by_key
            .insert((template.mode.as_str(), template.acuity), template)
            .is_some()
        {
            return Err(PathwayError::InvalidInput(
                "duplicate mode/acuity template".into(),
            ));
        }
    }
    let mut ids = HashSet::new();
    let mut patients = Vec::with_capacity(arrivals.len());
    for arrival in arrivals {
        if !valid_identifier(&arrival.patient_id)
            || !valid_identifier(&arrival.mode)
            || !(1..=5).contains(&arrival.acuity)
            || !ids.insert(arrival.patient_id.as_str())
        {
            return Err(PathwayError::InvalidInput(
                "arrival has invalid ID/mode/acuity or duplicate ID".into(),
            ));
        }
        let template = by_key
            .get(&(arrival.mode.as_str(), arrival.acuity))
            .ok_or_else(|| {
                PathwayError::InvalidInput("arrival has no exact mode/acuity template".into())
            })?;
        patients.push(PathwayPatient {
            patient_id: arrival.patient_id.clone(),
            arrival_tick: arrival.arrival_tick,
            mode: arrival.mode.clone(),
            acuity: arrival.acuity,
            stages: template.stages.clone(),
        });
    }
    Ok(patients)
}

fn valid_template_stages(stages: &[StageWork]) -> bool {
    stages.len() == 2
        && stages[0].kind == StageKind::Triage
        && stages[1].kind == StageKind::Assessment
        && stages.iter().all(|stage| {
            valid_identifier(&stage.resource_id)
                && valid_identifier(&stage.task_class_id)
                && stage.duration_ticks > 0
        })
        && stages[0].resource_id != stages[1].resource_id
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse_scenario_json, ScenarioConfig};

    type ConfigMutation = Box<dyn Fn(&mut PathwayConfig)>;

    fn profile() -> CapacityLocationProfile {
        let scenario: ScenarioConfig =
            parse_scenario_json(include_str!("../examples/one_patient.json"))
                .expect("E0 profile fixture validates");
        let mut profile = scenario.capacity_location;
        profile.staff_roles[0].counts[0].reference = "synthetic.fixture".into();
        let mut assessment = profile.resource_buckets[0].clone();
        assessment.resource_id = "resource.assessment".into();
        profile.resource_buckets.push(assessment);
        profile
    }

    fn stage(kind: StageKind, resource_id: &str, duration_ticks: u64) -> StageWork {
        StageWork {
            kind,
            resource_id: resource_id.into(),
            task_class_id: "task.synthetic".into(),
            duration_ticks,
        }
    }

    fn patient(id: &str, arrival_tick: u64, acuity: u8) -> PathwayPatient {
        PathwayPatient {
            patient_id: id.into(),
            arrival_tick,
            mode: "alpha".into(),
            acuity,
            stages: vec![
                stage(StageKind::Triage, "resource.synthetic", 2),
                stage(StageKind::Assessment, "resource.assessment", 3),
            ],
        }
    }

    fn config(patients: Vec<PathwayPatient>, horizon_ticks: u64) -> PathwayConfig {
        PathwayConfig {
            version: 1,
            horizon_ticks,
            provenance: Provenance {
                class: "synthetic".into(),
                notes: "synthetic fixture".into(),
            },
            capacity_location: profile(),
            patients,
        }
    }

    fn interval(result: &PathwayResult, stage: usize) -> (Option<u64>, Option<u64>, u64) {
        let episode = &result.episodes[stage];
        (
            episode.start_tick,
            episode.completion_tick,
            episode.observed_work_ticks,
        )
    }

    #[test]
    fn supplied_two_stage_work_has_exact_non_queue_intervals_and_stable_ids() {
        let result = run_pathway(&config(vec![patient("p1", 1, 3)], 10)).unwrap();
        let result = &result[0];
        assert_eq!(interval(result, 0), (Some(1), Some(3), 2));
        assert_eq!(interval(result, 1), (Some(3), Some(6), 3));
        assert_eq!(
            result
                .episodes
                .iter()
                .map(|e| e.observed_work_ticks)
                .sum::<u64>(),
            5
        );
        assert_eq!(result.episodes[0].task_id, "p1:triage");
        assert_eq!(result.episodes[1].task_id, "p1:assessment");
        assert_eq!(result.state, PathwayState::AssessmentComplete);
    }

    #[test]
    fn triage_reveals_acuity_and_fcfs_ignores_acuity_priority() {
        let mut patients = (1..=5)
            .map(|acuity| patient(&format!("p{acuity}"), 1, acuity))
            .collect::<Vec<_>>();
        let mut cfg = config(patients.clone(), 30);
        cfg.capacity_location.resource_buckets[0].open_count = Some(5);
        cfg.capacity_location.resource_buckets[0].staffed_count = Some(5);
        cfg.capacity_location.resource_buckets[0].physical_count = Some(5);
        cfg.capacity_location.resource_buckets[1].open_count = Some(5);
        cfg.capacity_location.resource_buckets[1].staffed_count = Some(5);
        cfg.capacity_location.resource_buckets[1].physical_count = Some(5);
        let results = run_pathway(&cfg).unwrap();
        for result in &results {
            assert_eq!(result.known_acuity, Some(result.acuity));
            assert_eq!(interval(result, 0), (Some(1), Some(3), 2));
        }
        patients.reverse();
        cfg.patients = patients;
        assert_eq!(results, run_pathway(&cfg).unwrap());
        cfg.capacity_location.resource_buckets.reverse();
        assert_eq!(results, run_pathway(&cfg).unwrap());
    }

    #[test]
    fn same_tick_completion_and_arrivals_enqueue_before_deterministic_fcfs_grants() {
        let mut patients = vec![
            patient("A", 1, 5),
            patient("B", 1, 1),
            patient("C", 1, 3),
            patient("D", 1, 2),
            patient("E", 3, 4),
        ];
        let mut cfg = config(patients.clone(), 20);
        for resource in &mut cfg.capacity_location.resource_buckets {
            resource.open_count = Some(2);
            resource.staffed_count = Some(2);
            resource.physical_count = Some(2);
        }
        let results = run_pathway(&cfg).unwrap();
        let by_id: HashMap<_, _> = results
            .iter()
            .map(|item| (item.patient_id.as_str(), item))
            .collect();
        for id in ["A", "B"] {
            assert_eq!(interval(by_id[id], 0), (Some(1), Some(3), 2));
        }
        for id in ["C", "D"] {
            assert_eq!(interval(by_id[id], 0), (Some(3), Some(5), 2));
        }
        assert_eq!(interval(by_id["E"], 0), (Some(5), Some(7), 2));
        for id in ["A", "B"] {
            assert_eq!(interval(by_id[id], 1), (Some(3), Some(6), 3));
        }
        for id in ["C", "D"] {
            assert_eq!(interval(by_id[id], 1), (Some(6), Some(9), 3));
        }
        assert_eq!(interval(by_id["E"], 1), (Some(9), Some(12), 3));
        patients.reverse();
        cfg.patients = patients;
        assert_eq!(results, run_pathway(&cfg).unwrap());
    }

    #[test]
    fn exact_horizon_completion_grants_next_stage_at_horizon_and_censors_it_at_zero() {
        let result = run_pathway(&config(vec![patient("p", 1, 2)], 3)).unwrap();
        assert_eq!(interval(&result[0], 0), (Some(1), Some(3), 2));
        assert_eq!(interval(&result[0], 1), (Some(3), None, 0));
        assert_eq!(result[0].known_acuity, Some(2));
        assert_eq!(result[0].state, PathwayState::Active(StageKind::Assessment));
    }

    #[test]
    fn very_long_service_is_censored_without_completion_overflow() {
        let mut patient = patient("long", 1, 2);
        patient.stages[0].duration_ticks = u64::MAX;
        let result = run_pathway(&config(vec![patient], 10)).unwrap();
        assert_eq!(interval(&result[0], 0), (Some(1), None, 9));
        assert_eq!(result[0].episodes.len(), 1);
        assert_eq!(result[0].state, PathwayState::Active(StageKind::Triage));
    }

    #[test]
    fn queued_and_active_episodes_are_censored_at_horizon() {
        let cfg = config(vec![patient("a", 0, 1), patient("b", 0, 2)], 1);
        let results = run_pathway(&cfg).unwrap();
        assert_eq!(results[0].patient_id, "a");
        assert_eq!(interval(&results[0], 0), (Some(0), None, 1));
        assert_eq!(interval(&results[1], 0), (None, None, 0));
        assert_eq!(results[1].state, PathwayState::Queued(StageKind::Triage));
        assert_eq!(results[0].known_acuity, None);
        assert_eq!(results[1].known_acuity, None);
        assert!(results.iter().all(|item| item
            .episodes
            .iter()
            .all(|e| e.completion_tick.is_none_or(|t| t <= 1))));
    }

    #[test]
    fn empty_population_still_validates_capacity_profile() {
        let cfg = config(Vec::new(), 10);
        assert!(run_pathway(&cfg).unwrap().is_empty());
        let mut invalid = cfg;
        invalid.capacity_location.resource_buckets.clear();
        assert!(matches!(
            run_pathway(&invalid),
            Err(PathwayError::InvalidInput(_))
        ));
    }

    #[test]
    fn invalid_patient_and_stage_mutations_are_rejected() {
        let mutations: Vec<ConfigMutation> = vec![
            Box::new(|cfg| cfg.patients[0].patient_id = "bad:id".into()),
            Box::new(|cfg| cfg.patients[0].mode = "møde".into()),
            Box::new(|cfg| cfg.patients[0].acuity = 0),
            Box::new(|cfg| cfg.patients[0].stages[0].duration_ticks = 0),
            Box::new(|cfg| cfg.patients[0].stages[0].resource_id = "missing".into()),
            Box::new(|cfg| cfg.patients[0].stages.swap(0, 1)),
            Box::new(|cfg| cfg.patients[0].stages[1].resource_id = "resource.synthetic".into()),
            Box::new(|cfg| cfg.horizon_ticks = 0),
            Box::new(|cfg| cfg.provenance.notes = "  ".into()),
            Box::new(|cfg| cfg.provenance.notes = " synthetic fixture".into()),
        ];
        for mutation in mutations {
            let mut cfg = config(vec![patient("p", 1, 2)], 10);
            mutation(&mut cfg);
            assert!(matches!(
                run_pathway(&cfg),
                Err(PathwayError::InvalidInput(_))
            ));
        }
    }

    #[test]
    fn generated_arrivals_bind_only_exact_templates_and_preserve_fields() {
        let arrivals = vec![GeneratedArrival {
            patient_id: "g1".into(),
            arrival_tick: 2,
            mode: "alpha".into(),
            acuity: 4,
        }];
        let templates = vec![StageTemplate {
            mode: "alpha".into(),
            acuity: 4,
            stages: patient("x", 0, 4).stages,
        }];
        let bound = bind_arrivals(&arrivals, &templates).unwrap();
        assert_eq!(bound[0].patient_id, "g1");
        assert_eq!(bound[0].arrival_tick, 2);
        assert_eq!(bound[0].acuity, 4);
        assert_eq!(bound[0].stages[0].duration_ticks, 2);
        assert!(matches!(
            bind_arrivals(&arrivals, &[]),
            Err(PathwayError::InvalidInput(_))
        ));
        let duplicate = vec![templates[0].clone(), templates[0].clone()];
        assert!(matches!(
            bind_arrivals(&arrivals, &duplicate),
            Err(PathwayError::InvalidInput(_))
        ));
        let reordered = vec![
            StageTemplate {
                mode: "beta".into(),
                acuity: 2,
                stages: patient("y", 0, 2).stages,
            },
            templates[0].clone(),
        ];
        let rebound = bind_arrivals(&arrivals, &reordered).unwrap();
        assert_eq!(bound[0].patient_id, rebound[0].patient_id);
        assert_eq!(
            bound[0].stages[0].duration_ticks,
            rebound[0].stages[0].duration_ticks
        );
    }

    #[test]
    fn class_eligibility_and_capacity_consistency_mutations_are_rejected() {
        let mutations: Vec<ConfigMutation> = vec![
            Box::new(|cfg| {
                cfg.capacity_location.task_classes[0].required_resource_classes =
                    vec!["wrong".into()]
            }),
            Box::new(|cfg| cfg.capacity_location.eligibility.clear()),
            Box::new(|cfg| {
                cfg.capacity_location.resource_buckets[0].availability_status =
                    crate::KnowledgeStatus::Unknown
            }),
            Box::new(|cfg| cfg.capacity_location.resource_buckets[0].open_count = Some(0)),
            Box::new(|cfg| {
                cfg.capacity_location.resource_buckets[0].physical_status =
                    crate::KnowledgeStatus::Unknown;
                cfg.capacity_location.resource_buckets[0].physical_count = None;
            }),
            Box::new(|cfg| cfg.capacity_location.resource_buckets[0].physical_count = Some(0)),
            Box::new(|cfg| {
                cfg.capacity_location.capacity.physical_status = crate::KnowledgeStatus::Unknown;
                cfg.capacity_location.capacity.physical_count = None;
            }),
            Box::new(|cfg| cfg.capacity_location.resource_buckets[0].resource_id = "bad id".into()),
            Box::new(|cfg| {
                cfg.capacity_location.staff_roles[0].counts[0].reference = "bad reference".into()
            }),
        ];
        for mutation in mutations {
            let mut cfg = config(vec![patient("p", 1, 2)], 10);
            mutation(&mut cfg);
            assert!(matches!(
                run_pathway(&cfg),
                Err(PathwayError::InvalidInput(_))
            ));
        }
    }

    #[test]
    fn distinct_stage_pool_capacities_keep_fcfs_queues_independent() {
        let mut cfg = config(
            vec![patient("A", 1, 3), patient("B", 1, 2), patient("C", 1, 1)],
            20,
        );
        cfg.capacity_location.resource_buckets[0].open_count = Some(2);
        cfg.capacity_location.resource_buckets[0].staffed_count = Some(2);
        cfg.capacity_location.resource_buckets[0].physical_count = Some(2);
        let results = run_pathway(&cfg).unwrap();
        let by_id: HashMap<_, _> = results
            .iter()
            .map(|result| (result.patient_id.as_str(), result))
            .collect();
        assert_eq!(interval(by_id["A"], 0), (Some(1), Some(3), 2));
        assert_eq!(interval(by_id["B"], 0), (Some(1), Some(3), 2));
        assert_eq!(interval(by_id["C"], 0), (Some(3), Some(5), 2));
        assert_eq!(interval(by_id["A"], 1), (Some(3), Some(6), 3));
        assert_eq!(interval(by_id["B"], 1), (Some(6), Some(9), 3));
        assert_eq!(interval(by_id["C"], 1), (Some(9), Some(12), 3));
    }

    #[test]
    fn documented_patient_profile_and_template_bounds_are_enforced() {
        let too_many_patients = config(vec![patient("p", 1, 1); MAX_PATIENTS + 1], 10);
        assert!(matches!(
            run_pathway(&too_many_patients),
            Err(PathwayError::InvalidInput(_))
        ));

        type ProfileMutation = Box<dyn Fn(&mut CapacityLocationProfile)>;
        let collection_mutations: Vec<ProfileMutation> = vec![
            Box::new(|profile| {
                profile.zones = vec![profile.zones[0].clone(); MAX_PROFILE_ROWS + 1]
            }),
            Box::new(|profile| {
                profile.locations = vec![profile.locations[0].clone(); MAX_PROFILE_ROWS + 1]
            }),
            Box::new(|profile| {
                profile.resource_buckets =
                    vec![profile.resource_buckets[0].clone(); MAX_PROFILE_ROWS + 1]
            }),
            Box::new(|profile| {
                profile.task_classes = vec![profile.task_classes[0].clone(); MAX_PROFILE_ROWS + 1]
            }),
            Box::new(|profile| {
                profile.staff_roles = vec![profile.staff_roles[0].clone(); MAX_PROFILE_ROWS + 1]
            }),
            Box::new(|profile| {
                profile.eligibility = vec![profile.eligibility[0].clone(); MAX_PROFILE_ROWS + 1]
            }),
            Box::new(|profile| {
                profile.routes = vec![
                    crate::Route {
                        from_location_id: "location.synthetic".into(),
                        to_location_id: "location.synthetic".into(),
                        distance: crate::RouteDistance {
                            value: 0.0,
                            unit: "m".into()
                        },
                    };
                    MAX_PROFILE_ROWS + 1
                ]
            }),
        ];
        for mutate in collection_mutations {
            let mut cfg = config(Vec::new(), 10);
            mutate(&mut cfg.capacity_location);
            match validate_config(&cfg) {
                Err(PathwayError::InvalidInput(message)) => {
                    assert_eq!(message, "capacity profile limit exceeded")
                }
                other => {
                    panic!("profile collection over-limit did not return limit error: {other:?}")
                }
            }
        }
        let nested_mutations: Vec<ProfileMutation> = vec![
            Box::new(|profile| {
                profile.staff_roles[0].counts = vec![profile.staff_roles[0].counts[0].clone(); 33]
            }),
            Box::new(|profile| profile.resource_buckets[0].capabilities = vec!["cap".into(); 33]),
            Box::new(|profile| {
                profile.task_classes[0].required_resource_classes = vec!["class".into(); 33]
            }),
            Box::new(|profile| {
                profile.capacity.equipment = vec![
                    crate::EquipmentCapability {
                        capability: "cap".into(),
                        status: "available".into(),
                        reason: None,
                    };
                    33
                ]
            }),
        ];
        for mutate in nested_mutations {
            let mut cfg = config(Vec::new(), 10);
            mutate(&mut cfg.capacity_location);
            match validate_config(&cfg) {
                Err(PathwayError::InvalidInput(message)) => {
                    assert_eq!(message, "capacity profile limit exceeded")
                }
                other => panic!("nested profile over-limit did not return limit error: {other:?}"),
            }
        }

        let template = StageTemplate {
            mode: "alpha".into(),
            acuity: 1,
            stages: patient("template", 0, 1).stages,
        };
        let arrivals = vec![GeneratedArrival {
            patient_id: "p".into(),
            arrival_tick: 1,
            mode: "alpha".into(),
            acuity: 1,
        }];
        assert!(matches!(
            bind_arrivals(&arrivals, &vec![template.clone(); MAX_PROFILE_ROWS + 1]),
            Err(PathwayError::InvalidInput(_))
        ));
        let exact_templates = (0..MAX_PROFILE_ROWS)
            .map(|index| StageTemplate {
                mode: format!("mode{index}"),
                acuity: 1,
                stages: template.stages.clone(),
            })
            .collect::<Vec<_>>();
        let exact_template_arrival = GeneratedArrival {
            patient_id: "p".into(),
            arrival_tick: 1,
            mode: "mode0".into(),
            acuity: 1,
        };
        assert_eq!(
            bind_arrivals(&[exact_template_arrival], &exact_templates)
                .unwrap()
                .len(),
            1
        );
        let too_many_arrivals = vec![
            GeneratedArrival {
                patient_id: "p".into(),
                arrival_tick: 1,
                mode: "alpha".into(),
                acuity: 1,
            };
            MAX_PATIENTS + 1
        ];
        assert!(
            matches!(bind_arrivals(&too_many_arrivals, std::slice::from_ref(&template)), Err(PathwayError::InvalidInput(message)) if message == "patient limit exceeded")
        );
        let exact_arrivals = (0..MAX_PATIENTS)
            .map(|index| GeneratedArrival {
                patient_id: format!("patient{index}"),
                arrival_tick: 1,
                mode: "alpha".into(),
                acuity: 1,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            bind_arrivals(&exact_arrivals, &[template]).unwrap().len(),
            MAX_PATIENTS
        );
    }

    #[test]
    fn generator_adapter_preserves_values_through_template_and_runtime() {
        let generated = crate::arrivals::generate_arrivals(&crate::arrivals::ArrivalConfig {
            version: 1,
            seed: 17,
            horizon_ticks: 10,
            provenance: "synthetic_tick_fixture".into(),
            bins: vec![crate::arrivals::ArrivalBin {
                start_tick: 1,
                end_tick: 2,
                modes: vec![crate::arrivals::ArrivalModeCounts {
                    mode: "alpha".into(),
                    total: 1,
                    acuity_counts: [0, 0, 0, 1, 0],
                }],
            }],
        })
        .unwrap();
        let template = StageTemplate {
            mode: "alpha".into(),
            acuity: 4,
            stages: patient("template", 0, 4).stages,
        };
        let bound = bind_arrivals(&generated, &[template]).unwrap();
        assert_eq!(bound[0].mode, generated[0].mode);
        assert_eq!(bound[0].acuity, generated[0].acuity);
        assert_eq!(bound[0].patient_id, generated[0].patient_id);
        assert_eq!(bound[0].arrival_tick, generated[0].arrival_tick);
        assert_eq!(bound[0].stages[0].kind, StageKind::Triage);
        assert_eq!(bound[0].stages[0].resource_id, "resource.synthetic");
        assert_eq!(bound[0].stages[0].task_class_id, "task.synthetic");
        assert_eq!(bound[0].stages[0].duration_ticks, 2);
        assert_eq!(bound[0].stages[1].kind, StageKind::Assessment);
        assert_eq!(bound[0].stages[1].resource_id, "resource.assessment");
        assert_eq!(bound[0].stages[1].task_class_id, "task.synthetic");
        assert_eq!(bound[0].stages[1].duration_ticks, 3);
        let results = run_pathway(&config(bound, 10)).unwrap();
        assert_eq!(results[0].mode, "alpha");
        assert_eq!(results[0].acuity, generated[0].acuity);
        assert_eq!(results[0].known_acuity, Some(4));
    }
}
