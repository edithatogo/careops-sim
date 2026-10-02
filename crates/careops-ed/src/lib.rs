//! Small deterministic ED-domain shell for E0.
//!
//! The only accepted runtime inputs are explicitly synthetic supplied work
//! items. This crate does not encode clinical pathways or empirically validate
//! an ED profile.

use std::collections::{HashMap, HashSet, VecDeque};
use std::error::Error;
use std::fmt::{Display, Formatter};

use kairo_ecs_core::Scheduler;
use kairo_ecs_types::{EntityId, EventKind, ScheduleRequest, SimTime, StepOutcome};
use serde::{Deserialize, Serialize};

const ARRIVAL_EVENT: u32 = 1;
const COMPLETION_EVENT: u32 = 2;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ScenarioConfig {
    pub schema_version: u32,
    pub scenario_id: String,
    pub seed: u64,
    pub time_unit: String,
    pub horizon_ticks: u64,
    pub provenance: Provenance,
    pub capacity_location: CapacityLocationProfile,
    pub patients: Vec<PatientWorkItem>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    pub class: String,
    pub notes: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CapacityLocationProfile {
    pub schema_version: u32,
    pub status: String,
    pub capacity: AggregateCapacity,
    pub zones: Vec<Zone>,
    pub locations: Vec<Location>,
    pub resource_buckets: Vec<ResourceBucket>,
    pub task_classes: Vec<TaskClass>,
    pub staff_roles: Vec<StaffRole>,
    pub eligibility: Vec<Eligibility>,
    #[serde(default)]
    pub routes: Vec<Route>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AggregateCapacity {
    pub unit: String,
    pub bucket_completeness: String,
    pub physical_status: KnowledgeStatus,
    pub physical_count: Option<u64>,
    pub availability_status: KnowledgeStatus,
    pub open_count: Option<u64>,
    pub staffed_count: Option<u64>,
    #[serde(default)]
    pub availability_reason: Option<String>,
    #[serde(default)]
    pub equipment: Vec<EquipmentCapability>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EquipmentCapability {
    pub capability: String,
    pub status: String,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeStatus {
    Known,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Zone {
    pub zone_id: String,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Location {
    pub location_id: String,
    pub name: String,
    pub zone_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ResourceBucket {
    pub resource_id: String,
    pub resource_class: String,
    pub location_id: String,
    pub zone_id: String,
    #[serde(default)]
    pub pool_group_id: Option<String>,
    pub physical_status: KnowledgeStatus,
    pub physical_count: Option<u64>,
    pub availability_status: KnowledgeStatus,
    pub open_count: Option<u64>,
    pub staffed_count: Option<u64>,
    #[serde(default)]
    pub availability_reason: Option<String>,
    pub capabilities: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TaskClass {
    pub task_class_id: String,
    pub name: String,
    #[serde(default)]
    pub required_resource_classes: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StaffRole {
    pub staff_role_id: String,
    pub name: String,
    pub counts: Vec<StaffCount>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StaffCount {
    pub basis: String,
    pub status: KnowledgeStatus,
    pub count: Option<u64>,
    pub reference: String,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Eligibility {
    pub staff_role_id: String,
    pub task_class_id: String,
    pub zone_id: String,
    #[serde(default)]
    pub location_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Route {
    pub from_location_id: String,
    pub to_location_id: String,
    pub distance: RouteDistance,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RouteDistance {
    pub value: f64,
    pub unit: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PatientWorkItem {
    pub patient_id: String,
    pub arrival_tick: u64,
    pub work_ticks: u64,
    pub resource_id: String,
    pub task_class_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct RunSummary {
    pub schema_version: u32,
    pub scenario_id: String,
    pub seed: u64,
    pub time_unit: String,
    pub horizon_ticks: u64,
    pub arrivals: usize,
    pub started: usize,
    pub completed: usize,
    pub unfinished: usize,
    pub patients: Vec<PatientResult>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PatientResult {
    pub patient_id: String,
    pub arrival_tick: u64,
    pub start_tick: Option<u64>,
    pub completion_tick: Option<u64>,
    pub work_ticks: u64,
    pub wait_ticks: Option<u64>,
    pub elapsed_ticks: Option<u64>,
    pub status: PatientStatus,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PatientStatus {
    Completed,
    Unfinished,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenarioError(String);

impl Display for ScenarioError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "invalid scenario: {}", self.0)
    }
}

impl Error for ScenarioError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimulationError(String);

impl Display for SimulationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "simulation failed: {}", self.0)
    }
}

impl Error for SimulationError {}

pub fn parse_scenario_json(source: &str) -> Result<ScenarioConfig, ScenarioError> {
    let scenario: ScenarioConfig =
        serde_json::from_str(source).map_err(|error| ScenarioError(error.to_string()))?;
    scenario.validate()?;
    Ok(scenario)
}

impl ScenarioConfig {
    pub fn validate(&self) -> Result<(), ScenarioError> {
        if self.schema_version != 1 {
            return invalid("schema_version must be 1");
        }
        require_id("scenario_id", &self.scenario_id)?;
        if self.time_unit != "tick" {
            return invalid("time_unit must be 'tick'; no clinical time conversion is defined");
        }
        if self.horizon_ticks == 0 {
            return invalid("horizon_ticks must be positive");
        }
        if self.provenance.class != "synthetic" {
            return invalid("E0 only accepts provenance.class 'synthetic'");
        }
        require_id("provenance.notes", &self.provenance.notes)?;
        if self.patients.is_empty() {
            return invalid("patients must contain at least one supplied work item");
        }

        let profile = &self.capacity_location;
        if profile.schema_version != 3 || profile.status != "proposed" {
            return invalid("capacity_location must use proposed P0 schema version 3");
        }
        validate_aggregate_capacity(&profile.capacity)?;

        let zone_ids = unique_ids(
            "zone_id",
            profile.zones.iter().map(|zone| zone.zone_id.as_str()),
        )?;
        let mut location_ids = HashSet::new();
        for location in &profile.locations {
            require_id("location_id", &location.location_id)?;
            require_id("location.name", &location.name)?;
            if !location_ids.insert(location.location_id.as_str()) {
                return invalid(format!("duplicate location_id '{}'", location.location_id));
            }
            if !zone_ids.contains(location.zone_id.as_str()) {
                return invalid(format!(
                    "location '{}' refers to unknown zone '{}'",
                    location.location_id, location.zone_id
                ));
            }
        }
        if profile.locations.is_empty() {
            return invalid("capacity_location.locations must not be empty");
        }

        let mut resource_ids = HashMap::new();
        for resource in &profile.resource_buckets {
            require_id("resource_id", &resource.resource_id)?;
            require_id("resource_class", &resource.resource_class)?;
            if resource_ids
                .insert(resource.resource_id.as_str(), resource)
                .is_some()
            {
                return invalid(format!("duplicate resource_id '{}'", resource.resource_id));
            }
            if !location_ids.contains(resource.location_id.as_str()) {
                return invalid(format!(
                    "resource '{}' refers to unknown location '{}'",
                    resource.resource_id, resource.location_id
                ));
            }
            if !zone_ids.contains(resource.zone_id.as_str()) {
                return invalid(format!(
                    "resource '{}' refers to unknown zone '{}'",
                    resource.resource_id, resource.zone_id
                ));
            }
            validate_known_count(
                "resource availability",
                resource.availability_status,
                resource.open_count,
                resource.staffed_count,
                resource.availability_reason.as_deref(),
            )?;
            if resource.open_count.unwrap_or(0) == 0 || resource.staffed_count.unwrap_or(0) == 0 {
                return invalid(format!(
                    "resource '{}' needs positive known open and staffed counts to run",
                    resource.resource_id
                ));
            }
            validate_physical_count(
                resource.physical_status,
                resource.physical_count,
                "resource physical count",
            )?;
            if let (Some(physical), Some(open)) = (resource.physical_count, resource.open_count) {
                if open > physical {
                    return invalid(format!(
                        "resource '{}' open_count exceeds physical_count",
                        resource.resource_id
                    ));
                }
            }
            if let (Some(open), Some(staffed)) = (resource.open_count, resource.staffed_count) {
                if staffed > open {
                    return invalid(format!(
                        "resource '{}' staffed_count exceeds open_count",
                        resource.resource_id
                    ));
                }
            }
            for capability in &resource.capabilities {
                require_id("resource capability", capability)?;
            }
        }
        if resource_ids.is_empty() {
            return invalid("capacity_location.resource_buckets must not be empty");
        }

        let task_ids = unique_ids(
            "task_class_id",
            profile
                .task_classes
                .iter()
                .map(|item| item.task_class_id.as_str()),
        )?;
        let mut task_classes = HashMap::new();
        for task in &profile.task_classes {
            require_id("task_class.name", &task.name)?;
            for class in &task.required_resource_classes {
                require_id("required_resource_class", class)?;
            }
            task_classes.insert(task.task_class_id.as_str(), task);
        }
        if task_ids.is_empty() {
            return invalid("capacity_location.task_classes must not be empty");
        }

        let role_ids = unique_ids(
            "staff_role_id",
            profile
                .staff_roles
                .iter()
                .map(|role| role.staff_role_id.as_str()),
        )?;
        for role in &profile.staff_roles {
            require_id("staff_role.name", &role.name)?;
            if role.counts.is_empty() {
                return invalid(format!(
                    "staff role '{}' must record at least one count",
                    role.staff_role_id
                ));
            }
            for count in &role.counts {
                if !["employed", "rostered", "present", "task_eligible"]
                    .contains(&count.basis.as_str())
                {
                    return invalid(format!(
                        "staff role '{}' has unsupported count basis '{}'",
                        role.staff_role_id, count.basis
                    ));
                }
                require_id("staff_count.reference", &count.reference)?;
                validate_staff_count(count)?;
            }
        }
        if role_ids.is_empty() {
            return invalid("capacity_location.staff_roles must not be empty");
        }

        for item in &profile.eligibility {
            if !role_ids.contains(item.staff_role_id.as_str())
                || !task_ids.contains(item.task_class_id.as_str())
                || !zone_ids.contains(item.zone_id.as_str())
            {
                return invalid(
                    "eligibility contains an unresolved staff, task, or zone reference",
                );
            }
            if let Some(location_id) = &item.location_id {
                if !location_ids.contains(location_id.as_str()) {
                    return invalid(format!(
                        "eligibility refers to unknown location '{location_id}'"
                    ));
                }
            }
        }
        if profile.eligibility.is_empty() {
            return invalid("capacity_location.eligibility must not be empty");
        }

        for route in &profile.routes {
            if !location_ids.contains(route.from_location_id.as_str())
                || !location_ids.contains(route.to_location_id.as_str())
            {
                return invalid("route contains an unresolved location reference");
            }
            if route.distance.unit != "m"
                || !route.distance.value.is_finite()
                || route.distance.value < 0.0
            {
                return invalid("route distance must be finite, nonnegative, and use unit 'm'");
            }
        }

        let patient_ids = unique_ids(
            "patient_id",
            self.patients
                .iter()
                .map(|patient| patient.patient_id.as_str()),
        )?;
        for patient in &self.patients {
            if patient.arrival_tick >= self.horizon_ticks {
                return invalid(format!(
                    "patient '{}' arrival_tick must be earlier than horizon_ticks",
                    patient.patient_id
                ));
            }
            if patient.work_ticks == 0 {
                return invalid(format!(
                    "patient '{}' work_ticks must be positive",
                    patient.patient_id
                ));
            }
            let resource = resource_ids
                .get(patient.resource_id.as_str())
                .ok_or_else(|| {
                    ScenarioError(format!(
                        "patient '{}' refers to unknown resource '{}'",
                        patient.patient_id, patient.resource_id
                    ))
                })?;
            let task = task_classes
                .get(patient.task_class_id.as_str())
                .ok_or_else(|| {
                    ScenarioError(format!(
                        "patient '{}' refers to unknown task class '{}'",
                        patient.patient_id, patient.task_class_id
                    ))
                })?;
            if !task.required_resource_classes.is_empty()
                && !task
                    .required_resource_classes
                    .contains(&resource.resource_class)
            {
                return invalid(format!(
                    "patient '{}' task class is incompatible with resource '{}'",
                    patient.patient_id, patient.resource_id
                ));
            }
            if !profile.eligibility.iter().any(|entry| {
                entry.task_class_id == patient.task_class_id
                    && entry.zone_id == resource.zone_id
                    && entry
                        .location_id
                        .as_ref()
                        .map_or(true, |location| location == &resource.location_id)
            }) {
                return invalid(format!(
                    "patient '{}' has no matching staff/task/zone eligibility",
                    patient.patient_id
                ));
            }
        }
        if patient_ids.is_empty() {
            return invalid("patients must contain at least one supplied work item");
        }
        Ok(())
    }
}

fn validate_aggregate_capacity(capacity: &AggregateCapacity) -> Result<(), ScenarioError> {
    if capacity.unit != "treatment_space"
        || !["complete", "incomplete", "unknown"].contains(&capacity.bucket_completeness.as_str())
    {
        return invalid("capacity unit/completeness does not match P0 contract");
    }
    let mut capabilities = HashSet::new();
    for item in &capacity.equipment {
        require_id("capacity.equipment.capability", &item.capability)?;
        if !capabilities.insert(item.capability.as_str()) {
            return invalid(format!(
                "duplicate capacity equipment capability '{}'",
                item.capability
            ));
        }
        match (item.status.as_str(), item.reason.as_deref()) {
            ("available" | "unavailable", None) => {}
            ("unknown", Some(reason)) if !reason.trim().is_empty() => {}
            ("available" | "unavailable", Some(_)) => {
                return invalid("known equipment status must not have a reason")
            }
            ("unknown", _) => {
                return invalid("unknown equipment status requires a nonempty reason")
            }
            _ => return invalid("unsupported capacity equipment status"),
        }
    }
    validate_physical_count(
        capacity.physical_status,
        capacity.physical_count,
        "aggregate physical count",
    )?;
    validate_known_count(
        "aggregate availability",
        capacity.availability_status,
        capacity.open_count,
        capacity.staffed_count,
        capacity.availability_reason.as_deref(),
    )?;
    if capacity.availability_status == KnowledgeStatus::Known {
        let open = capacity.open_count.unwrap_or(0);
        let staffed = capacity.staffed_count.unwrap_or(0);
        if open > capacity.physical_count.unwrap_or(u64::MAX) || staffed > open {
            return invalid("aggregate capacity counts are inconsistent");
        }
    }
    Ok(())
}

fn validate_physical_count(
    status: KnowledgeStatus,
    count: Option<u64>,
    label: &str,
) -> Result<(), ScenarioError> {
    match (status, count) {
        (KnowledgeStatus::Known, Some(_)) | (KnowledgeStatus::Unknown, None) => Ok(()),
        (KnowledgeStatus::Known, None) => {
            invalid(format!("{label} is marked known but has no count"))
        }
        (KnowledgeStatus::Unknown, Some(_)) => {
            invalid(format!("{label} is marked unknown but supplies a count"))
        }
    }
}

fn validate_known_count(
    label: &str,
    status: KnowledgeStatus,
    open_count: Option<u64>,
    staffed_count: Option<u64>,
    reason: Option<&str>,
) -> Result<(), ScenarioError> {
    match (status, open_count, staffed_count) {
        (KnowledgeStatus::Known, Some(_), Some(_)) if reason.is_none() => Ok(()),
        (KnowledgeStatus::Unknown, None, None)
            if reason.map_or(false, |value| !value.trim().is_empty()) =>
        {
            Ok(())
        }
        (KnowledgeStatus::Known, _, _) => invalid(format!(
            "{label} known status requires open/staffed counts and no reason"
        )),
        (KnowledgeStatus::Unknown, _, _) => invalid(format!(
            "{label} unknown status requires null counts and a reason"
        )),
    }
}

fn validate_staff_count(count: &StaffCount) -> Result<(), ScenarioError> {
    match (count.status, count.count, count.reason.as_deref()) {
        (KnowledgeStatus::Known, Some(_), None) => Ok(()),
        (KnowledgeStatus::Unknown, None, Some(reason)) if !reason.trim().is_empty() => Ok(()),
        (KnowledgeStatus::Known, _, _) => {
            invalid("known staff count requires an integer and no reason")
        }
        (KnowledgeStatus::Unknown, _, _) => {
            invalid("unknown staff count requires null and a reason")
        }
    }
}

fn unique_ids<'a>(
    label: &str,
    values: impl Iterator<Item = &'a str>,
) -> Result<HashSet<&'a str>, ScenarioError> {
    let mut result = HashSet::new();
    for value in values {
        require_id(label, value)?;
        if !result.insert(value) {
            return invalid(format!("duplicate {label} '{value}'"));
        }
    }
    if result.is_empty() {
        return invalid(format!("{label} collection must not be empty"));
    }
    Ok(result)
}

fn require_id(label: &str, value: &str) -> Result<(), ScenarioError> {
    if value.trim().is_empty() || value.trim() != value {
        return invalid(format!("{label} must be a nonempty, trimmed identifier"));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, ScenarioError> {
    Err(ScenarioError(message.into()))
}

#[derive(Clone, Debug)]
struct PatientState {
    arrival_tick: Option<u64>,
    start_tick: Option<u64>,
    completion_tick: Option<u64>,
}

#[derive(Clone, Debug)]
struct ResourceState {
    capacity: u64,
    busy: u64,
    waiting: VecDeque<usize>,
}

pub fn run_scenario(scenario: &ScenarioConfig) -> Result<RunSummary, SimulationError> {
    scenario
        .validate()
        .map_err(|error| SimulationError(error.to_string()))?;

    let mut patients = scenario.patients.clone();
    patients.sort_by(|left, right| {
        left.arrival_tick
            .cmp(&right.arrival_tick)
            .then_with(|| left.patient_id.cmp(&right.patient_id))
    });
    let mut states = vec![
        PatientState {
            arrival_tick: None,
            start_tick: None,
            completion_tick: None
        };
        patients.len()
    ];
    let mut resources = HashMap::new();
    for resource in &scenario.capacity_location.resource_buckets {
        let capacity = resource
            .open_count
            .unwrap_or(0)
            .min(resource.staffed_count.unwrap_or(0));
        resources.insert(
            resource.resource_id.clone(),
            ResourceState {
                capacity,
                busy: 0,
                waiting: VecDeque::new(),
            },
        );
    }

    let mut scheduler = Scheduler::new();
    for (index, patient) in patients.iter().enumerate() {
        let entity_index = u64::try_from(index).map_err(|_| {
            SimulationError("patient index exceeds Kairos entity ID range".to_owned())
        })?;
        scheduler.schedule(ScheduleRequest {
            at: SimTime::from_ticks(u128::from(patient.arrival_tick)),
            priority: 0,
            entity: Some(EntityId::new(entity_index, 0)),
            kind: EventKind::custom(ARRIVAL_EVENT),
        });
    }

    while let StepOutcome::Dispatched(event) = scheduler.step() {
        let index = event
            .entity
            .ok_or_else(|| SimulationError("Kairos event has no patient entity".to_owned()))?
            .index as usize;
        let patient = patients
            .get(index)
            .ok_or_else(|| SimulationError("Kairos event refers to unknown patient".to_owned()))?;
        match event.kind.code() {
            ARRIVAL_EVENT => {
                let arrival_tick = u64::try_from(event.at.ticks()).map_err(|_| {
                    SimulationError("arrival tick exceeds scenario range".to_owned())
                })?;
                states[index].arrival_tick = Some(arrival_tick);
                resources
                    .get_mut(&patient.resource_id)
                    .ok_or_else(|| {
                        SimulationError("patient resource disappeared after validation".to_owned())
                    })?
                    .waiting
                    .push_back(index);
                start_waiting(
                    &patient.resource_id,
                    event.at.ticks(),
                    scenario.horizon_ticks,
                    &patients,
                    &mut states,
                    &mut resources,
                    &mut scheduler,
                )?;
            }
            COMPLETION_EVENT => {
                let completion_tick = u64::try_from(event.at.ticks()).map_err(|_| {
                    SimulationError("completion tick exceeds scenario range".to_owned())
                })?;
                if states[index].completion_tick.is_some() || states[index].start_tick.is_none() {
                    return Err(SimulationError("invalid completion event state".to_owned()));
                }
                states[index].completion_tick = Some(completion_tick);
                let resource = resources.get_mut(&patient.resource_id).ok_or_else(|| {
                    SimulationError("patient resource disappeared after validation".to_owned())
                })?;
                if resource.busy == 0 {
                    return Err(SimulationError("resource busy count underflow".to_owned()));
                }
                resource.busy -= 1;
                start_waiting(
                    &patient.resource_id,
                    event.at.ticks(),
                    scenario.horizon_ticks,
                    &patients,
                    &mut states,
                    &mut resources,
                    &mut scheduler,
                )?;
            }
            _ => {
                return Err(SimulationError(
                    "Kairos scheduler emitted an unknown event kind".to_owned(),
                ))
            }
        }
    }

    let mut results = Vec::with_capacity(patients.len());
    for (patient, state) in patients.iter().zip(states.iter()) {
        let arrival_tick = state.arrival_tick.ok_or_else(|| {
            SimulationError("arrival was not processed before the horizon".to_owned())
        })?;
        let wait_ticks = state.start_tick.map(|start| start - arrival_tick);
        let elapsed_ticks = state
            .completion_tick
            .map(|completion| completion - arrival_tick);
        results.push(PatientResult {
            patient_id: patient.patient_id.clone(),
            arrival_tick,
            start_tick: state.start_tick,
            completion_tick: state.completion_tick,
            work_ticks: patient.work_ticks,
            wait_ticks,
            elapsed_ticks,
            status: if state.completion_tick.is_some() {
                PatientStatus::Completed
            } else {
                PatientStatus::Unfinished
            },
        });
    }
    let arrivals = results.len();
    let started = results
        .iter()
        .filter(|patient| patient.start_tick.is_some())
        .count();
    let completed = results
        .iter()
        .filter(|patient| patient.completion_tick.is_some())
        .count();
    let unfinished = arrivals - completed;
    if completed + unfinished != arrivals || started > arrivals {
        return Err(SimulationError(
            "patient count conservation invariant failed".to_owned(),
        ));
    }

    Ok(RunSummary {
        schema_version: 1,
        scenario_id: scenario.scenario_id.clone(),
        seed: scenario.seed,
        time_unit: scenario.time_unit.clone(),
        horizon_ticks: scenario.horizon_ticks,
        arrivals,
        started,
        completed,
        unfinished,
        patients: results,
    })
}

fn start_waiting(
    resource_id: &str,
    now: u128,
    horizon_ticks: u64,
    patients: &[PatientWorkItem],
    states: &mut [PatientState],
    resources: &mut HashMap<String, ResourceState>,
    scheduler: &mut Scheduler,
) -> Result<(), SimulationError> {
    loop {
        let next = {
            let resource = resources
                .get_mut(resource_id)
                .ok_or_else(|| SimulationError("unknown resource while dispatching".to_owned()))?;
            if resource.busy >= resource.capacity {
                None
            } else if let Some(index) = resource.waiting.pop_front() {
                resource.busy += 1;
                Some(index)
            } else {
                None
            }
        };
        let Some(index) = next else { break };
        if states[index].start_tick.is_some() {
            return Err(SimulationError("patient started more than once".to_owned()));
        }
        let start_tick = u64::try_from(now)
            .map_err(|_| SimulationError("start tick exceeds scenario range".to_owned()))?;
        states[index].start_tick = Some(start_tick);
        let completion = now
            .checked_add(u128::from(patients[index].work_ticks))
            .ok_or_else(|| SimulationError("completion time overflow".to_owned()))?;
        if completion <= u128::from(horizon_ticks) {
            let entity_index = u64::try_from(index).map_err(|_| {
                SimulationError("patient index exceeds Kairos entity ID range".to_owned())
            })?;
            scheduler.schedule(ScheduleRequest {
                at: SimTime::from_ticks(completion),
                priority: 0,
                entity: Some(EntityId::new(entity_index, 0)),
                kind: EventKind::custom(COMPLETION_EVENT),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_runs_are_identical() {
        let source = include_str!("../examples/one_patient.json");
        let scenario = parse_scenario_json(source).expect("example scenario should validate");
        assert_eq!(
            run_scenario(&scenario).unwrap(),
            run_scenario(&scenario).unwrap()
        );
    }

    #[test]
    fn processes_completion_at_the_horizon_and_leaves_late_work_unfinished() {
        let source = include_str!("../examples/one_patient.json");
        let mut scenario = parse_scenario_json(source).unwrap();
        scenario.horizon_ticks = 3;
        assert_eq!(run_scenario(&scenario).unwrap().completed, 1);
        scenario.horizon_ticks = 2;
        let summary = run_scenario(&scenario).unwrap();
        assert_eq!(summary.completed, 0);
        assert_eq!(summary.unfinished, 1);
        assert_eq!(summary.patients[0].start_tick, Some(1));
        assert_eq!(summary.patients[0].completion_tick, None);
    }

    #[test]
    fn accepted_e01_synthetic_case_maps_to_the_e0_single_work_slice() {
        let source = include_str!("../examples/one_patient.json");
        let scenario = parse_scenario_json(source).unwrap();
        let summary = run_scenario(&scenario).unwrap();
        let patient = &summary.patients[0];
        assert_eq!(patient.patient_id, "synthetic-patient-1");
        assert_eq!(patient.arrival_tick, 1);
        assert_eq!(patient.start_tick, Some(1));
        assert_eq!(patient.completion_tick, Some(3));
        assert_eq!(patient.wait_ticks, Some(0));
        assert_eq!(patient.work_ticks, 2);
        assert_eq!(patient.elapsed_ticks, Some(2));
        assert_eq!(summary.arrivals, 1);
        assert_eq!(summary.started, 1);
        assert_eq!(summary.completed, 1);
        assert_eq!(summary.unfinished, 0);
    }
}
