//! Experimental synthetic ED triage and assessment runtime.
//!
//! Supplied stage durations are synthetic tick work, not clinical estimates.
//! This module is an experimental pathway fixture and makes no admission,
//! discharge, staffing-identity, or operational-validity claim.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

use kairo_ecs_core::Scheduler;
use kairo_ecs_rng::{derive_entity_seed, DeterministicStream};
use kairo_ecs_types::{EntityId, EventKind, ScheduleRequest, SimTime, StepOutcome};
use serde::{Deserialize, Serialize};

use crate::arrivals::GeneratedArrival;
use crate::{validate_capacity_work, CapacityLocationProfile, PatientWorkItem, Provenance};

const ARRIVAL_EVENT: u32 = 1;
const COMPLETION_EVENT_BASE: u32 = 2;
const TERMINAL_DISCHARGE_EVENT: u32 = 100;
const TERMINAL_CARE_DUE_EVENT: u32 = 101;
const TERMINAL_CARE_COMPLETE_EVENT: u32 = 102;
const TERMINAL_TRANSFER_COMPLETE_EVENT: u32 = 103;
const TERMINAL_OFFER_BOUNDARY_EVENT: u32 = 104;
const TERMINAL_ABANDONMENT_EVENT: u32 = 105;
const SERVICE_DOMAIN_INDEX: u64 = 0x4341_5245_4F50_5353;
const MAX_PATIENTS: usize = 100_000;
const MAX_PROFILE_ROWS: usize = 1_024;
const MAX_DIAGNOSTICS: usize = 16;
const MAX_STAGES: usize = 19;
const MAX_DRAWS: usize = 128;
const MAX_TERMINAL_SCHEDULED_EVENTS: u64 = 4_400_000;

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
    Diagnostics,
    Treatment,
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
    pub episode_index: u32,
    pub queue_entry_tick: u64,
    pub start_tick: Option<u64>,
    pub completion_tick: Option<u64>,
    pub observed_work_ticks: u64,
    pub sampled_duration_ticks: u64,
    pub service: ServiceSpec,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PathwayState {
    Queued(StageKind),
    Active(StageKind),
    AssessmentComplete,
    TreatmentComplete,
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceSpec {
    pub version: u32,
    pub provenance: ServiceProvenance,
    pub law: ServiceLaw,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceProvenance {
    pub class: String,
    pub notes: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceLaw {
    Fixed { duration_ticks: u64 },
    Discrete { outcomes: Vec<ServiceOutcome> },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceOutcome {
    pub duration_ticks: u64,
    pub weight: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceBinding {
    pub patient_id: String,
    pub stage_index: usize,
    pub service: ServiceSpec,
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
    waiting: BTreeSet<(u64, usize, u8, usize)>,
}

struct SamplingContext<'a> {
    seed: u64,
    patients: &'a [PathwayPatient],
    services: &'a [Vec<ServiceSpec>],
}

#[derive(Default)]
struct TerminalPatientRuntime {
    decision: Option<crate::disposition::DecisionObservation>,
    destination: Option<String>,
    space_queue_tick: Option<u64>,
    space_start: Option<u64>,
    space_end: Option<u64>,
    boarding_start: Option<u64>,
    allocation: Option<crate::offers::OfferAllocation>,
    transfer: Option<crate::terminal::TerminalWorkEpisode>,
    care: Vec<crate::terminal::TerminalWorkEpisode>,
    care_active: bool,
    terminal_tick: Option<u64>,
    outcome: Option<crate::terminal::RealizedOutcome>,
    discharge_pending: bool,
    abandonment_deadline: Option<u64>,
}

struct TerminalRuntime {
    patients: Vec<TerminalPatientRuntime>,
    patient_indices: HashMap<String, usize>,
    patient_ids: Vec<String>,
    space_waiting: BTreeSet<(u64, usize)>,
    offer_book: crate::offers::OfferBook,
    destination_by_patient: HashMap<String, String>,
    intent_by_patient: HashMap<String, crate::disposition::DispositionIntent>,
    last_metric_tick: u64,
    busy_ticks: BTreeMap<String, u128>,
    peak_busy: BTreeMap<String, u64>,
}

impl TerminalRuntime {
    fn new(
        config: &crate::terminal::TerminalConfig,
        patients: &[PathwayPatient],
    ) -> Result<Self, PathwayError> {
        let patient_indices: HashMap<String, usize> = patients
            .iter()
            .enumerate()
            .map(|(index, patient)| (patient.patient_id.clone(), index))
            .collect();
        let mut destination_by_patient = HashMap::new();
        for item in &config.admissions {
            destination_by_patient.insert(item.patient_id.clone(), item.destination.clone());
        }
        let intent_by_patient = config
            .decisions
            .iter()
            .map(|decision| (decision.patient_id.clone(), decision.latent_intent))
            .collect();
        let mut runtime_patients: Vec<TerminalPatientRuntime> = (0..patients.len())
            .map(|_| TerminalPatientRuntime::default())
            .collect();
        for (patient_id, destination) in &destination_by_patient {
            if let Some(index) = patient_indices.get(patient_id) {
                runtime_patients[*index].destination = Some(destination.clone());
            }
        }
        Ok(Self {
            patients: runtime_patients,
            patient_indices,
            patient_ids: patients
                .iter()
                .map(|patient| patient.patient_id.clone())
                .collect(),
            space_waiting: BTreeSet::new(),
            offer_book: crate::offers::OfferBook::new(&config.offer_policy, &config.offers)?,
            destination_by_patient,
            intent_by_patient,
            last_metric_tick: 0,
            busy_ticks: BTreeMap::new(),
            peak_busy: BTreeMap::new(),
        })
    }

    fn advance_metrics(&mut self, tick: u64, resources: &BTreeMap<String, ResourceRuntime>) {
        let elapsed = u128::from(tick.saturating_sub(self.last_metric_tick));
        if elapsed > 0 {
            for (id, resource) in resources {
                *self.busy_ticks.entry(id.clone()).or_default() +=
                    u128::from(resource.busy) * elapsed;
            }
            self.last_metric_tick = tick;
        }
        for (id, resource) in resources {
            self.peak_busy
                .entry(id.clone())
                .and_modify(|peak| *peak = (*peak).max(resource.busy))
                .or_insert(resource.busy);
        }
    }
}

/// Run the legacy supplied-duration path. Every stage duration remains a fixed service.
pub fn run_pathway(config: &PathwayConfig) -> Result<Vec<PathwayResult>, PathwayError> {
    validate_config(config, false)?;
    let mut bindings = Vec::new();
    for patient in &config.patients {
        for (stage_index, stage) in patient.stages.iter().enumerate() {
            bindings.push(ServiceBinding {
                patient_id: patient.patient_id.clone(),
                stage_index,
                service: ServiceSpec {
                    version: 1,
                    provenance: ServiceProvenance {
                        class: config.provenance.class.clone(),
                        notes: config.provenance.notes.clone(),
                    },
                    law: ServiceLaw::Fixed {
                        duration_ticks: stage.duration_ticks,
                    },
                },
            });
        }
    }
    run_pathway_with_services(config, 0, &bindings)
}

/// Run a pathway using explicit fixed or discrete synthetic service laws.
pub fn run_pathway_with_services(
    config: &PathwayConfig,
    seed: u64,
    bindings: &[ServiceBinding],
) -> Result<Vec<PathwayResult>, PathwayError> {
    Ok(run_pathway_internal(config, seed, bindings, None)?.pathways)
}

pub(crate) fn run_terminal_live(
    terminal: &crate::terminal::TerminalConfig,
) -> Result<crate::terminal::TerminalRunResult, PathwayError> {
    let output = run_pathway_internal(
        &terminal.pathway,
        terminal.seed,
        &terminal.services,
        Some(terminal),
    )?;
    output
        .terminal
        .ok_or_else(|| PathwayError::Runtime("terminal runtime output is missing".into()))
}

struct SharedOutput {
    pathways: Vec<PathwayResult>,
    terminal: Option<crate::terminal::TerminalRunResult>,
}

fn run_pathway_internal(
    config: &PathwayConfig,
    seed: u64,
    bindings: &[ServiceBinding],
    terminal_config: Option<&crate::terminal::TerminalConfig>,
) -> Result<SharedOutput, PathwayError> {
    validate_config(config, true)?;
    let service_map = validate_service_bindings(config, bindings)?;

    let mut patients = config.patients.clone();
    patients.sort_by(|left, right| left.patient_id.cmp(&right.patient_id));
    let mut services = Vec::with_capacity(patients.len());
    for patient in &patients {
        let mut patient_services = Vec::with_capacity(patient.stages.len());
        for stage_index in 0..patient.stages.len() {
            patient_services.push(
                service_map
                    .get(&(patient.patient_id.clone(), stage_index))
                    .ok_or_else(|| {
                        PathwayError::Runtime("validated service binding disappeared".into())
                    })?
                    .clone(),
            );
        }
        services.push(patient_services);
    }

    let sampling = SamplingContext {
        seed,
        patients: &patients,
        services: &services,
    };
    let mut results: Vec<PathwayResult> = patients
        .iter()
        .map(|patient| PathwayResult {
            patient_id: patient.patient_id.clone(),
            mode: patient.mode.clone(),
            acuity: patient.acuity,
            known_acuity: None,
            arrival_tick: patient.arrival_tick,
            episodes: Vec::with_capacity(patient.stages.len()),
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

    let mut terminal_runtime = terminal_config
        .map(|terminal| TerminalRuntime::new(terminal, &patients))
        .transpose()?;
    let mut scheduled_event_count = 0u64;

    let mut scheduler = Scheduler::new();
    for (index, patient) in patients.iter().enumerate() {
        if terminal_config.is_some() {
            count_terminal_event(&mut scheduled_event_count)?;
        }
        scheduler.schedule(ScheduleRequest {
            at: SimTime::from_ticks(u128::from(patient.arrival_tick)),
            priority: 0,
            entity: Some(EntityId::new(index as u64, 0)),
            kind: EventKind::custom(ARRIVAL_EVENT),
        });
        if let Some(terminal) = terminal_config {
            if let crate::terminal::AbandonmentPolicy::BeforeFirstCare { patience_ticks } =
                terminal.policy.abandonment
            {
                let deadline = u128::from(patient.arrival_tick) + u128::from(patience_ticks);
                if deadline <= u128::from(config.horizon_ticks) {
                    count_terminal_event(&mut scheduled_event_count)?;
                    terminal_runtime
                        .as_mut()
                        .ok_or_else(|| PathwayError::Runtime("terminal runtime is missing".into()))?
                        .patients[index]
                        .abandonment_deadline = Some(deadline as u64);
                    scheduler.schedule(ScheduleRequest {
                        at: SimTime::from_ticks(deadline),
                        priority: 0,
                        entity: Some(EntityId::new(index as u64, 0)),
                        kind: EventKind::custom(TERMINAL_ABANDONMENT_EVENT),
                    });
                }
            }
        }
    }
    if let (Some(_terminal), Some(runtime)) = (terminal_config, terminal_runtime.as_ref()) {
        for boundary in runtime.offer_book.boundary_ticks() {
            if boundary > config.horizon_ticks {
                continue;
            }
            count_terminal_event(&mut scheduled_event_count)?;
            scheduler.schedule(ScheduleRequest {
                at: SimTime::from_ticks(u128::from(boundary)),
                priority: 0,
                entity: None,
                kind: EventKind::custom(TERMINAL_OFFER_BOUNDARY_EVENT),
            });
        }
    }

    while let Some(next) = scheduler.peek_next() {
        let tick = next.at.ticks();
        if tick > u128::from(config.horizon_ticks) {
            break;
        }
        let now = u64::try_from(tick)
            .map_err(|_| PathwayError::Runtime("event tick exceeds pathway range".into()))?;
        if let (Some(runtime), Some(_)) = (terminal_runtime.as_mut(), terminal_config) {
            runtime.advance_metrics(now, &resources);
        }
        let mut abandonment_candidates = Vec::new();
        while let Some(preview) = scheduler.peek_next() {
            if preview.at.ticks() != tick {
                break;
            }
            let event = match scheduler.step() {
                StepOutcome::Dispatched(event) => event,
                StepOutcome::Empty | StepOutcome::LimitReached => {
                    return Err(PathwayError::Runtime(
                        "scheduler stopped during a tick batch".into(),
                    ))
                }
            };
            let code = event.kind.code();
            if code == TERMINAL_OFFER_BOUNDARY_EVENT {
                continue;
            }
            let patient_index = event.entity.map(|entity| entity.index as usize);
            let Some(patient_index) = patient_index else {
                return Err(PathwayError::Runtime(
                    "scheduler event has no patient entity".into(),
                ));
            };
            let patient = patients.get(patient_index).ok_or_else(|| {
                PathwayError::Runtime("scheduler event has an unknown patient".into())
            })?;
            if code == TERMINAL_ABANDONMENT_EVENT {
                abandonment_candidates.push(patient_index);
            } else if code == ARRIVAL_EVENT {
                if let (Some(runtime), Some(terminal)) =
                    (terminal_runtime.as_mut(), terminal_config)
                {
                    runtime.space_waiting.insert((now, patient_index));
                    runtime.patients[patient_index].space_queue_tick = Some(now);
                    let _ = terminal;
                } else {
                    enqueue_stage(
                        patient_index,
                        0,
                        now,
                        &sampling,
                        &mut results,
                        &mut resources,
                    )?;
                }
            } else if code == TERMINAL_DISCHARGE_EVENT {
                terminal_departure(
                    patient_index,
                    now,
                    crate::terminal::RealizedOutcome::Discharged,
                    terminal_runtime.as_mut().ok_or_else(|| {
                        PathwayError::Runtime("discharge event outside terminal mode".into())
                    })?,
                    &mut resources,
                    terminal_config
                        .ok_or_else(|| PathwayError::Runtime("terminal config missing".into()))?,
                )?;
            } else if code == TERMINAL_CARE_DUE_EVENT {
                enqueue_boarder_care(
                    patient_index,
                    now,
                    terminal_runtime.as_mut().ok_or_else(|| {
                        PathwayError::Runtime("care event outside terminal mode".into())
                    })?,
                    terminal_config
                        .ok_or_else(|| PathwayError::Runtime("terminal config missing".into()))?,
                    &mut resources,
                )?;
            } else if code == TERMINAL_CARE_COMPLETE_EVENT {
                complete_terminal_work(
                    patient_index,
                    now,
                    true,
                    terminal_runtime.as_mut().ok_or_else(|| {
                        PathwayError::Runtime("care event outside terminal mode".into())
                    })?,
                    &mut resources,
                    &mut scheduler,
                    config.horizon_ticks,
                    terminal_config
                        .ok_or_else(|| PathwayError::Runtime("terminal config missing".into()))?,
                    &mut scheduled_event_count,
                )?;
            } else if code == TERMINAL_TRANSFER_COMPLETE_EVENT {
                complete_terminal_work(
                    patient_index,
                    now,
                    false,
                    terminal_runtime.as_mut().ok_or_else(|| {
                        PathwayError::Runtime("transfer event outside terminal mode".into())
                    })?,
                    &mut resources,
                    &mut scheduler,
                    config.horizon_ticks,
                    terminal_config
                        .ok_or_else(|| PathwayError::Runtime("terminal config missing".into()))?,
                    &mut scheduled_event_count,
                )?;
            } else if (COMPLETION_EVENT_BASE..TERMINAL_DISCHARGE_EVENT).contains(&code) {
                let stage_index = usize::try_from(code - COMPLETION_EVENT_BASE)
                    .map_err(|_| PathwayError::Runtime("completion stage index overflow".into()))?;
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
                episode.observed_work_ticks = episode.sampled_duration_ticks;
                let resource = resources
                    .get_mut(&stage.resource_id)
                    .ok_or_else(|| PathwayError::Runtime("active resource disappeared".into()))?;
                resource.busy = resource
                    .busy
                    .checked_sub(1)
                    .ok_or_else(|| PathwayError::Runtime("resource busy count underflow".into()))?;
                if stage.kind == StageKind::Triage {
                    results[patient_index].known_acuity = Some(patient.acuity);
                }
                if stage_index + 1 < patient.stages.len() {
                    enqueue_stage(
                        patient_index,
                        stage_index + 1,
                        now,
                        &sampling,
                        &mut results,
                        &mut resources,
                    )?;
                } else {
                    results[patient_index].state = match stage.kind {
                        StageKind::Assessment => PathwayState::AssessmentComplete,
                        StageKind::Treatment => PathwayState::TreatmentComplete,
                        _ => {
                            return Err(PathwayError::Runtime(
                                "pathway ended on an intermediate stage".into(),
                            ))
                        }
                    };
                    if stage.kind == StageKind::Treatment {
                        if let (Some(runtime), Some(terminal)) =
                            (terminal_runtime.as_mut(), terminal_config)
                        {
                            let intent = *runtime
                                .intent_by_patient
                                .get(&patient.patient_id)
                                .ok_or_else(|| {
                                    PathwayError::Runtime("validated decision disappeared".into())
                                })?;
                            runtime.patients[patient_index].decision =
                                Some(crate::disposition::DecisionObservation {
                                    policy_version: terminal.decision_policy.version,
                                    provenance: terminal.decision_policy.provenance.clone(),
                                    observed_tick: now,
                                    provisional_intent: intent,
                                });
                            if intent == crate::disposition::DispositionIntent::Admission {
                                runtime.patients[patient_index].boarding_start = Some(now);
                                if let Some(care) = &terminal.policy.boarder_care {
                                    schedule_terminal_event(
                                        &mut scheduler,
                                        now,
                                        care.interval_ticks,
                                        config.horizon_ticks,
                                        patient_index,
                                        TERMINAL_CARE_DUE_EVENT,
                                        &mut scheduled_event_count,
                                    )?;
                                }
                            } else {
                                runtime.patients[patient_index].discharge_pending =
                                    terminal.policy.discharge_delay_ticks > 0;
                                schedule_terminal_event(
                                    &mut scheduler,
                                    now,
                                    terminal.policy.discharge_delay_ticks,
                                    config.horizon_ticks,
                                    patient_index,
                                    TERMINAL_DISCHARGE_EVENT,
                                    &mut scheduled_event_count,
                                )?;
                            }
                        }
                    }
                }
            } else {
                return Err(PathwayError::Runtime("unknown scheduler event kind".into()));
            }
        }
        if let (Some(runtime), Some(terminal)) = (terminal_runtime.as_mut(), terminal_config) {
            grant_terminal_phase(
                now,
                runtime,
                terminal,
                true,
                &patients,
                &mut results,
                &mut resources,
                &mut scheduler,
                config.horizon_ticks,
                &sampling,
                &mut scheduled_event_count,
            )?;
            let eligible = abandonment_candidates
                .into_iter()
                .filter(|patient_index| {
                    let state = &runtime.patients[*patient_index];
                    state.terminal_tick.is_none()
                        && state.abandonment_deadline == Some(now)
                        && state.decision.is_none()
                        && state.space_queue_tick.is_some()
                        && results[*patient_index]
                            .episodes
                            .first()
                            .is_none_or(|episode| episode.start_tick.is_none())
                })
                .collect::<Vec<_>>();
            for patient_index in eligible.iter().copied() {
                cancel_waiting_patient(
                    now,
                    patient_index,
                    &patients,
                    runtime,
                    &mut results,
                    &mut resources,
                    terminal,
                )?;
            }
            if !eligible.is_empty() {
                grant_terminal_phase(
                    now,
                    runtime,
                    terminal,
                    false,
                    &patients,
                    &mut results,
                    &mut resources,
                    &mut scheduler,
                    config.horizon_ticks,
                    &sampling,
                    &mut scheduled_event_count,
                )?;
            }
        } else {
            grant_waiting(
                tick,
                config.horizon_ticks,
                &patients,
                &mut results,
                &mut resources,
                &mut scheduler,
            )?;
        }
    }

    for result in &mut results {
        if let Some(episode) = result.episodes.last_mut() {
            if episode.completion_tick.is_none() {
                if let Some(start) = episode.start_tick {
                    episode.observed_work_ticks = config
                        .horizon_ticks
                        .saturating_sub(start)
                        .min(episode.sampled_duration_ticks);
                    result.state = PathwayState::Active(episode.kind);
                } else {
                    result.state = PathwayState::Queued(episode.kind);
                }
            }
        }
    }
    let (pathways, terminal) = match (terminal_config, terminal_runtime.take()) {
        (Some(terminal_config), Some(mut runtime)) => {
            runtime.advance_metrics(config.horizon_ticks, &resources);
            let terminal = build_terminal_result(terminal_config, results, runtime, resources)?;
            let pathways = terminal
                .patients
                .iter()
                .map(|patient| patient.pathway.clone())
                .collect();
            (pathways, Some(terminal))
        }
        _ => (results, None),
    };
    Ok(SharedOutput { pathways, terminal })
}

fn build_terminal_result(
    config: &crate::terminal::TerminalConfig,
    pathways: Vec<PathwayResult>,
    mut runtime: TerminalRuntime,
    resources: BTreeMap<String, ResourceRuntime>,
) -> Result<crate::terminal::TerminalRunResult, PathwayError> {
    let horizon = config.pathway.horizon_ticks;
    let mut patients = Vec::with_capacity(pathways.len());
    let mut discharged = 0u64;
    let mut transferred = 0u64;
    let mut abandoned = 0u64;
    for (index, pathway) in pathways.into_iter().enumerate() {
        let state = &mut runtime.patients[index];
        if let Some(episode) = state.transfer.as_mut() {
            if let Some(start) = episode.start_tick {
                episode.observed_work_ticks = episode
                    .completion_tick
                    .map(|end| end.saturating_sub(start))
                    .unwrap_or_else(|| {
                        horizon
                            .saturating_sub(start)
                            .min(episode.required_work_ticks)
                    });
            }
        }
        for episode in &mut state.care {
            if let Some(start) = episode.start_tick {
                episode.observed_work_ticks = episode
                    .completion_tick
                    .map(|end| end.saturating_sub(start))
                    .unwrap_or_else(|| {
                        horizon
                            .saturating_sub(start)
                            .min(episode.required_work_ticks)
                    });
            }
        }
        let arrival = pathway.arrival_tick;
        let decision_tick = state
            .decision
            .as_ref()
            .map(|decision| decision.observed_tick);
        let space = crate::terminal::SpaceEpisode {
            resource_id: config.policy.ed_space_resource_id.clone(),
            queue_entry_tick: arrival,
            acquired_tick: state.space_start,
            released_tick: state.space_end,
            observed_occupancy_ticks: state.space_start.map_or(0, |start| {
                state.space_end.unwrap_or(horizon).saturating_sub(start)
            }),
        };
        let boarding = state
            .boarding_start
            .map(|start| crate::terminal::ObservedInterval {
                start_tick: start,
                end_tick: state
                    .allocation
                    .as_ref()
                    .map(|allocation| allocation.allocated_tick),
                observed_ticks: state
                    .allocation
                    .as_ref()
                    .map(|allocation| allocation.allocated_tick.saturating_sub(start))
                    .unwrap_or_else(|| horizon.saturating_sub(start)),
            });
        let pending_transfer = state.allocation.as_ref().map(|allocation| {
            let start = allocation.allocated_tick;
            let end = state
                .transfer
                .as_ref()
                .and_then(|episode| episode.start_tick);
            crate::terminal::ObservedInterval {
                start_tick: start,
                end_tick: end,
                observed_ticks: end
                    .map(|end| end.saturating_sub(start))
                    .unwrap_or_else(|| horizon.saturating_sub(start)),
            }
        });
        let endpoint_state = if state.terminal_tick.is_some() {
            crate::terminal::EndpointState::Terminal
        } else if state.space_start.is_none() {
            crate::terminal::EndpointState::WaitingForSpace
        } else if state.decision.is_none() {
            crate::terminal::EndpointState::Clinical
        } else if state.discharge_pending {
            crate::terminal::EndpointState::DischargePending
        } else if state.allocation.is_none() {
            crate::terminal::EndpointState::Boarding
        } else if state
            .transfer
            .as_ref()
            .is_some_and(|episode| episode.start_tick.is_none())
        {
            crate::terminal::EndpointState::PendingTransfer
        } else {
            crate::terminal::EndpointState::InTransfer
        };
        let unfinished_reason = if state.terminal_tick.is_some() {
            None
        } else if state.space_start.is_none() {
            Some(crate::terminal::UnfinishedReason::WaitingForSpace)
        } else if let Some(episode) = pathway
            .episodes
            .last()
            .filter(|episode| episode.completion_tick.is_none())
        {
            Some(if episode.start_tick.is_some() {
                crate::terminal::UnfinishedReason::ClinicalActive {
                    stage: episode.kind,
                }
            } else {
                crate::terminal::UnfinishedReason::ClinicalQueued {
                    stage: episode.kind,
                }
            })
        } else if state.discharge_pending {
            Some(crate::terminal::UnfinishedReason::DischargePending)
        } else if state.allocation.is_some() {
            if state
                .transfer
                .as_ref()
                .is_some_and(|episode| episode.start_tick.is_some())
            {
                Some(crate::terminal::UnfinishedReason::TransferActive)
            } else {
                Some(crate::terminal::UnfinishedReason::TransferQueued)
            }
        } else if state.care_active {
            Some(crate::terminal::UnfinishedReason::BoarderCareActive)
        } else if state.care.iter().any(|episode| {
            episode.start_tick.is_none()
                && episode.completion_tick.is_none()
                && episode.cancelled_tick.is_none()
        }) {
            Some(crate::terminal::UnfinishedReason::BoarderCareQueued)
        } else {
            Some(crate::terminal::UnfinishedReason::AwaitingOfferAllocation)
        };
        let departure = state.terminal_tick;
        let observed_end = departure.unwrap_or(horizon).min(horizon);
        let observed_elapsed_ticks = observed_end.saturating_sub(arrival);
        let boarding_time_ticks = match (&state.outcome, decision_tick, departure) {
            (
                Some(crate::terminal::RealizedOutcome::Transferred { .. }),
                Some(start),
                Some(end),
            ) => Some(end.saturating_sub(start)),
            _ => None,
        };
        let observed_boarding_ticks = decision_tick
            .filter(|_| state.boarding_start.is_some())
            .map(|start| observed_end.saturating_sub(start));
        match state.outcome {
            Some(crate::terminal::RealizedOutcome::Discharged) => discharged += 1,
            Some(crate::terminal::RealizedOutcome::Transferred { .. }) => transferred += 1,
            Some(crate::terminal::RealizedOutcome::Abandoned) => abandoned += 1,
            None => {}
        }
        patients.push(crate::terminal::TerminalPatientResult {
            pathway,
            decision: state.decision.clone(),
            space,
            boarding,
            allocation: state.allocation.clone(),
            pending_transfer,
            transfer: state.transfer.clone(),
            care: state.care.clone(),
            endpoint_state,
            unfinished_reason,
            realized_outcome: state.outcome.clone(),
            terminal_tick: state.terminal_tick,
            completed_los_ticks: state.terminal_tick.map(|end| end.saturating_sub(arrival)),
            observed_elapsed_ticks,
            boarding_time_ticks,
            observed_boarding_ticks,
        });
    }
    let mut resource_usage = Vec::with_capacity(resources.len());
    for (resource_id, resource) in resources {
        let mut queued_at_horizon = resource.waiting.len() as u64;
        if resource_id == config.policy.ed_space_resource_id {
            queued_at_horizon =
                queued_at_horizon.saturating_add(runtime.space_waiting.len() as u64);
        }
        resource_usage.push(crate::terminal::ResourceUsage {
            resource_id: resource_id.clone(),
            capacity: resource.capacity,
            busy_ticks: runtime.busy_ticks.get(&resource_id).copied().unwrap_or(0),
            capacity_ticks: u128::from(resource.capacity) * u128::from(horizon),
            busy_at_horizon: resource.busy,
            queued_at_horizon,
            peak_busy: runtime
                .peak_busy
                .get(&resource_id)
                .copied()
                .unwrap_or(resource.busy),
        });
    }
    let offers = runtime.offer_book.snapshot(horizon)?;
    let arrived = u64::try_from(patients.len())
        .map_err(|_| PathwayError::Runtime("patient count overflow".into()))?;
    let unfinished = arrived
        .saturating_sub(discharged)
        .saturating_sub(transferred)
        .saturating_sub(abandoned);
    Ok(crate::terminal::TerminalRunResult {
        version: 1,
        patients,
        offers,
        counts: crate::terminal::TerminalCounts {
            arrived,
            discharged,
            transferred,
            abandoned,
            unfinished,
        },
        resources: resource_usage,
    })
}

fn schedule_terminal_event(
    scheduler: &mut Scheduler,
    now: u64,
    duration: u64,
    horizon: u64,
    patient_index: usize,
    event_code: u32,
    event_count: &mut u64,
) -> Result<(), PathwayError> {
    let at = u128::from(now) + u128::from(duration);
    if at <= u128::from(horizon) {
        count_terminal_event(event_count)?;
        scheduler.schedule(ScheduleRequest {
            at: SimTime::from_ticks(at),
            priority: 0,
            entity: Some(EntityId::new(patient_index as u64, 0)),
            kind: EventKind::custom(event_code),
        });
    }
    Ok(())
}

fn count_terminal_event(count: &mut u64) -> Result<(), PathwayError> {
    *count = count
        .checked_add(1)
        .ok_or_else(|| PathwayError::InvalidInput("terminal event count overflow".into()))?;
    if *count > MAX_TERMINAL_SCHEDULED_EVENTS {
        return Err(PathwayError::InvalidInput(
            "terminal scheduler event limit exceeded".into(),
        ));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn grant_terminal_phase(
    now: u64,
    runtime: &mut TerminalRuntime,
    terminal: &crate::terminal::TerminalConfig,
    allocate_offers: bool,
    patients: &[PathwayPatient],
    pathways: &mut [PathwayResult],
    resources: &mut BTreeMap<String, ResourceRuntime>,
    scheduler: &mut Scheduler,
    horizon: u64,
    sampling: &SamplingContext<'_>,
    event_count: &mut u64,
) -> Result<(), PathwayError> {
    grant_waiting_spaces(
        now,
        &terminal.policy.ed_space_resource_id,
        &mut runtime.space_waiting,
        &mut runtime.patients,
        pathways,
        resources,
        sampling,
    )?;
    if allocate_offers {
        allocate_terminal_offers(now, runtime, terminal, resources, scheduler, horizon)?;
    }
    grant_terminal_work(
        u128::from(now),
        horizon,
        patients,
        pathways,
        resources,
        scheduler,
        runtime,
        terminal,
        event_count,
    )
}

fn cancel_waiting_patient(
    now: u64,
    patient_index: usize,
    patients: &[PathwayPatient],
    runtime: &mut TerminalRuntime,
    pathways: &mut [PathwayResult],
    resources: &mut BTreeMap<String, ResourceRuntime>,
    terminal: &crate::terminal::TerminalConfig,
) -> Result<(), PathwayError> {
    let state = &mut runtime.patients[patient_index];
    if state.terminal_tick.is_some() || state.abandonment_deadline != Some(now) {
        return Err(PathwayError::Runtime(
            "abandonment candidate is not waiting at its deadline".into(),
        ));
    }
    if let Some(queue_tick) = state.space_queue_tick {
        runtime.space_waiting.remove(&(queue_tick, patient_index));
    }
    if let Some(triage) = pathways[patient_index]
        .episodes
        .first()
        .filter(|episode| episode.start_tick.is_none())
    {
        let triage_resource = &patients[patient_index].stages[0].resource_id;
        let resource = resources
            .get_mut(triage_resource)
            .ok_or_else(|| PathwayError::Runtime("triage resource disappeared".into()))?;
        resource
            .waiting
            .remove(&(triage.queue_entry_tick, patient_index, 0, 0));
    }
    if state.space_start.is_some() {
        if state.space_end.is_some() {
            return Err(PathwayError::Runtime(
                "abandonment released ED space more than once".into(),
            ));
        }
        state.space_end = Some(now);
        let space = resources
            .get_mut(&terminal.policy.ed_space_resource_id)
            .ok_or_else(|| PathwayError::Runtime("space resource disappeared".into()))?;
        space.busy = space
            .busy
            .checked_sub(1)
            .ok_or_else(|| PathwayError::Runtime("space resource underflow".into()))?;
    }
    state.terminal_tick = Some(now);
    state.outcome = Some(crate::terminal::RealizedOutcome::Abandoned);
    state.abandonment_deadline = None;
    Ok(())
}

fn grant_waiting_spaces(
    now: u64,
    space_resource_id: &str,
    waiting: &mut BTreeSet<(u64, usize)>,
    terminal_patients: &mut [TerminalPatientRuntime],
    pathways: &mut [PathwayResult],
    resources: &mut BTreeMap<String, ResourceRuntime>,
    sampling: &SamplingContext<'_>,
) -> Result<(), PathwayError> {
    loop {
        let has_capacity = resources
            .get(space_resource_id)
            .is_some_and(|resource| resource.busy < resource.capacity);
        if !has_capacity || waiting.is_empty() {
            break;
        }
        let (_, patient_index) = waiting
            .pop_first()
            .ok_or_else(|| PathwayError::Runtime("space queue entry disappeared".into()))?;
        if terminal_patients[patient_index].space_start.is_some() {
            return Err(PathwayError::Runtime(
                "space acquired more than once".into(),
            ));
        }
        terminal_patients[patient_index].space_start = Some(now);
        resources
            .get_mut(space_resource_id)
            .ok_or_else(|| PathwayError::Runtime("space resource disappeared".into()))?
            .busy += 1;
        enqueue_stage(patient_index, 0, now, sampling, pathways, resources)?;
    }
    Ok(())
}

fn enqueue_boarder_care(
    patient_index: usize,
    now: u64,
    runtime: &mut TerminalRuntime,
    terminal: &crate::terminal::TerminalConfig,
    resources: &mut BTreeMap<String, ResourceRuntime>,
) -> Result<(), PathwayError> {
    let state = &mut runtime.patients[patient_index];
    if state.decision.as_ref().map(|item| item.provisional_intent)
        != Some(crate::disposition::DispositionIntent::Admission)
        || state.allocation.is_some()
        || state.care_active
    {
        return Ok(());
    }
    let Some(policy) = terminal.policy.boarder_care.as_ref() else {
        return Ok(());
    };
    let index = state.care.len();
    let task_id = format!(
        "{}:boarder-care:{index}",
        runtime.patient_ids[patient_index]
    );
    state.care.push(crate::terminal::TerminalWorkEpisode {
        task_id,
        resource_id: policy.resource_id.clone(),
        task_class_id: policy.task_class_id.clone(),
        queue_entry_tick: now,
        start_tick: None,
        completion_tick: None,
        cancelled_tick: None,
        required_work_ticks: policy.duration_ticks,
        observed_work_ticks: 0,
    });
    resources
        .get_mut(&policy.resource_id)
        .ok_or_else(|| PathwayError::Runtime("care resource disappeared".into()))?
        .waiting
        .insert((now, patient_index, 1, index));
    Ok(())
}

fn allocate_terminal_offers(
    now: u64,
    runtime: &mut TerminalRuntime,
    terminal: &crate::terminal::TerminalConfig,
    resources: &mut BTreeMap<String, ResourceRuntime>,
    scheduler: &mut Scheduler,
    horizon: u64,
) -> Result<(), PathwayError> {
    let mut waiting = Vec::new();
    for (index, state) in runtime.patients.iter().enumerate() {
        if state.boarding_start.is_some() && state.allocation.is_none() && !state.care_active {
            waiting.push(crate::offers::AdmissionRequest {
                patient_id: runtime.patient_ids[index].clone(),
                destination: state
                    .destination
                    .clone()
                    .or_else(|| {
                        runtime
                            .destination_by_patient
                            .get(&runtime.patient_ids[index])
                            .cloned()
                    })
                    .ok_or_else(|| {
                        PathwayError::Runtime("admission destination disappeared".into())
                    })?,
                ready_tick: state.boarding_start.unwrap_or(now),
            });
        }
    }
    let allocations = runtime.offer_book.allocate(now, &waiting)?;
    for allocation in allocations {
        let patient_index = runtime
            .patient_indices
            .get(&allocation.patient_id)
            .copied()
            .ok_or_else(|| PathwayError::Runtime("offer allocated to unknown patient".into()))?;
        let state = &mut runtime.patients[patient_index];
        state.allocation = Some(allocation.clone());
        state.destination = Some(allocation.destination.clone());
        for (care_index, care) in state.care.iter_mut().enumerate() {
            if care.start_tick.is_none() && care.completion_tick.is_none() {
                if let Some(resource) = resources.get_mut(&care.resource_id) {
                    resource
                        .waiting
                        .remove(&(care.queue_entry_tick, patient_index, 1, care_index));
                }
                care.cancelled_tick = Some(now);
            }
        }
        let transfer = crate::terminal::TerminalWorkEpisode {
            task_id: format!("{}:transfer", allocation.patient_id),
            resource_id: terminal.policy.transfer_resource_id.clone(),
            task_class_id: terminal.policy.transfer_task_class_id.clone(),
            queue_entry_tick: now,
            start_tick: None,
            completion_tick: None,
            cancelled_tick: None,
            required_work_ticks: allocation.transfer_duration_ticks,
            observed_work_ticks: 0,
        };
        state.transfer = Some(transfer);
        resources
            .get_mut(&terminal.policy.transfer_resource_id)
            .ok_or_else(|| PathwayError::Runtime("transfer resource disappeared".into()))?
            .waiting
            .insert((now, patient_index, 2, 0));
    }
    let _ = scheduler;
    let _ = horizon;
    Ok(())
}

#[allow(clippy::too_many_arguments)] // Explicit shared scheduler state is kept visible at the grant barrier.
fn grant_terminal_work(
    now: u128,
    horizon: u64,
    patients: &[PathwayPatient],
    pathways: &mut [PathwayResult],
    resources: &mut BTreeMap<String, ResourceRuntime>,
    scheduler: &mut Scheduler,
    runtime: &mut TerminalRuntime,
    terminal: &crate::terminal::TerminalConfig,
    event_count: &mut u64,
) -> Result<(), PathwayError> {
    for (resource_id, resource) in resources.iter_mut() {
        while resource.busy < resource.capacity && !resource.waiting.is_empty() {
            let (queue_tick, patient_index, work_kind, index) = resource
                .waiting
                .pop_first()
                .ok_or_else(|| PathwayError::Runtime("work queue entry disappeared".into()))?;
            let start = u64::try_from(now)
                .map_err(|_| PathwayError::Runtime("work start exceeds tick range".into()))?;
            let (duration, event_code) = match work_kind {
                0 => {
                    let stage = patients[patient_index]
                        .stages
                        .get(index)
                        .ok_or_else(|| PathwayError::Runtime("stage queue index invalid".into()))?;
                    let episode = pathways[patient_index].episodes.last_mut().ok_or_else(|| {
                        PathwayError::Runtime("stage queue has no episode".into())
                    })?;
                    if episode.kind != stage.kind || episode.start_tick.is_some() {
                        return Err(PathwayError::Runtime(
                            "clinical queue state mismatch".into(),
                        ));
                    }
                    episode.start_tick = Some(start);
                    pathways[patient_index].state = PathwayState::Active(stage.kind);
                    (
                        episode.sampled_duration_ticks,
                        COMPLETION_EVENT_BASE + index as u32,
                    )
                }
                1 => {
                    let duration = {
                        let episode = runtime.patients[patient_index]
                            .care
                            .get_mut(index)
                            .ok_or_else(|| {
                                PathwayError::Runtime("care queue index invalid".into())
                            })?;
                        if episode.start_tick.is_some() || episode.cancelled_tick.is_some() {
                            return Err(PathwayError::Runtime("care queue state mismatch".into()));
                        }
                        episode.start_tick = Some(start);
                        episode.required_work_ticks
                    };
                    runtime.patients[patient_index].care_active = true;
                    (duration, TERMINAL_CARE_COMPLETE_EVENT)
                }
                2 => {
                    let episode = runtime.patients[patient_index]
                        .transfer
                        .as_mut()
                        .ok_or_else(|| {
                            PathwayError::Runtime("transfer queue index invalid".into())
                        })?;
                    if episode.start_tick.is_some() {
                        return Err(PathwayError::Runtime(
                            "transfer queue state mismatch".into(),
                        ));
                    }
                    episode.start_tick = Some(start);
                    (
                        episode.required_work_ticks,
                        TERMINAL_TRANSFER_COMPLETE_EVENT,
                    )
                }
                _ => return Err(PathwayError::Runtime("unknown work kind".into())),
            };
            resource.busy += 1;
            if resource.busy > resource.capacity {
                return Err(PathwayError::Runtime("resource capacity exceeded".into()));
            }
            let completion = now + u128::from(duration);
            if completion <= u128::from(horizon) {
                count_terminal_event(event_count)?;
                scheduler.schedule(ScheduleRequest {
                    at: SimTime::from_ticks(completion),
                    priority: 0,
                    entity: Some(EntityId::new(patient_index as u64, 0)),
                    kind: EventKind::custom(event_code),
                });
            }
            let _ = (queue_tick, resource_id, terminal);
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)] // Completion updates the same scheduler, pool and lifecycle state.
fn complete_terminal_work(
    patient_index: usize,
    now: u64,
    is_care: bool,
    runtime: &mut TerminalRuntime,
    resources: &mut BTreeMap<String, ResourceRuntime>,
    scheduler: &mut Scheduler,
    horizon: u64,
    terminal: &crate::terminal::TerminalConfig,
    event_count: &mut u64,
) -> Result<(), PathwayError> {
    if is_care {
        let state = &mut runtime.patients[patient_index];
        let episode = state
            .care
            .iter_mut()
            .rev()
            .find(|episode| episode.start_tick.is_some() && episode.completion_tick.is_none())
            .ok_or_else(|| PathwayError::Runtime("care completion has no active episode".into()))?;
        episode.completion_tick = Some(now);
        episode.observed_work_ticks = episode.required_work_ticks;
        let resource = resources
            .get_mut(&episode.resource_id)
            .ok_or_else(|| PathwayError::Runtime("care resource disappeared".into()))?;
        resource.busy = resource
            .busy
            .checked_sub(1)
            .ok_or_else(|| PathwayError::Runtime("care resource underflow".into()))?;
        state.care_active = false;
        if state.allocation.is_none() && state.boarding_start.is_some() {
            if let Some(policy) = terminal.policy.boarder_care.as_ref() {
                schedule_terminal_event(
                    scheduler,
                    now,
                    policy.interval_ticks,
                    horizon,
                    patient_index,
                    TERMINAL_CARE_DUE_EVENT,
                    event_count,
                )?;
            }
        }
    } else {
        let state = &mut runtime.patients[patient_index];
        let episode = state
            .transfer
            .as_mut()
            .ok_or_else(|| PathwayError::Runtime("transfer completion has no episode".into()))?;
        episode.completion_tick = Some(now);
        episode.observed_work_ticks = episode.required_work_ticks;
        let resource_id = episode.resource_id.clone();
        let resource = resources
            .get_mut(&resource_id)
            .ok_or_else(|| PathwayError::Runtime("transfer resource disappeared".into()))?;
        resource.busy = resource
            .busy
            .checked_sub(1)
            .ok_or_else(|| PathwayError::Runtime("transfer resource underflow".into()))?;
        let allocation = state
            .allocation
            .as_ref()
            .ok_or_else(|| PathwayError::Runtime("transfer completed without allocation".into()))?;
        terminal_departure(
            patient_index,
            now,
            crate::terminal::RealizedOutcome::Transferred {
                destination: allocation.destination.clone(),
                offer_id: allocation.offer_id.clone(),
            },
            runtime,
            resources,
            terminal,
        )?;
    }
    Ok(())
}

fn terminal_departure(
    patient_index: usize,
    now: u64,
    outcome: crate::terminal::RealizedOutcome,
    runtime: &mut TerminalRuntime,
    resources: &mut BTreeMap<String, ResourceRuntime>,
    terminal: &crate::terminal::TerminalConfig,
) -> Result<(), PathwayError> {
    let state = &mut runtime.patients[patient_index];
    if state.terminal_tick.is_some() {
        return Err(PathwayError::Runtime(
            "patient departed more than once".into(),
        ));
    }
    state.terminal_tick = Some(now);
    state.outcome = Some(outcome);
    state.discharge_pending = false;
    if state.space_start.is_some() {
        state.space_end = Some(now);
        let space = resources
            .get_mut(&terminal.policy.ed_space_resource_id)
            .ok_or_else(|| PathwayError::Runtime("space resource disappeared".into()))?;
        space.busy = space
            .busy
            .checked_sub(1)
            .ok_or_else(|| PathwayError::Runtime("space resource underflow".into()))?;
    }
    Ok(())
}

fn enqueue_stage(
    patient_index: usize,
    stage_index: usize,
    tick: u64,
    sampling: &SamplingContext<'_>,
    results: &mut [PathwayResult],
    resources: &mut BTreeMap<String, ResourceRuntime>,
) -> Result<(), PathwayError> {
    let patient = &sampling.patients[patient_index];
    let stage = &patient.stages[stage_index];
    let service = &sampling.services[patient_index][stage_index];
    let sampled_duration_ticks =
        sample_episode_service(service, sampling.seed, patient_index, stage_index)?;
    let episode_index = u32::try_from(
        patient.stages[..stage_index]
            .iter()
            .filter(|previous| previous.kind == stage.kind)
            .count(),
    )
    .map_err(|_| PathwayError::Runtime("episode index exceeds u32".into()))?;
    let task_id = match stage.kind {
        StageKind::Triage => format!("{}:triage", patient.patient_id),
        StageKind::Assessment => format!("{}:assessment", patient.patient_id),
        StageKind::Diagnostics => format!("{}:diagnostics:{episode_index}", patient.patient_id),
        StageKind::Treatment => format!("{}:treatment", patient.patient_id),
    };
    let episode = StageEpisode {
        task_id,
        kind: stage.kind,
        episode_index,
        queue_entry_tick: tick,
        start_tick: None,
        completion_tick: None,
        observed_work_ticks: 0,
        sampled_duration_ticks,
        service: service.clone(),
    };
    results[patient_index].episodes.push(episode);
    results[patient_index].state = PathwayState::Queued(stage.kind);
    let resource = resources
        .get_mut(&stage.resource_id)
        .ok_or_else(|| PathwayError::Runtime("validated resource disappeared".into()))?;
    resource
        .waiting
        .insert((tick, patient_index, 0, stage_index));
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
            let (_, patient_index, work_kind, stage_index) =
                resource.waiting.pop_first().ok_or_else(|| {
                    PathwayError::Runtime("queued work disappeared while granting".into())
                })?;
            if work_kind != 0 {
                return Err(PathwayError::Runtime(
                    "legacy grant found terminal work".into(),
                ));
            }
            let patient = &patients[patient_index];
            let stage = &patient.stages[stage_index];
            let start = u64::try_from(now)
                .map_err(|_| PathwayError::Runtime("start tick overflow".into()))?;
            let sampled_duration = results[patient_index]
                .episodes
                .last()
                .ok_or_else(|| PathwayError::Runtime("grant has no queued episode".into()))?
                .sampled_duration_ticks;
            let completion = now + u128::from(sampled_duration);
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
                    kind: EventKind::custom(COMPLETION_EVENT_BASE + stage_index as u32),
                });
            }
        }
    }
    Ok(())
}

pub(crate) fn validate_config(
    config: &PathwayConfig,
    allow_discrete_sentinel: bool,
) -> Result<(), PathwayError> {
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
    let mut total_stages = 0usize;
    let mut total_events = 0u64;
    for patient in &config.patients {
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
        if !valid_stage_sequence(&patient.stages) {
            return invalid("stages must be triage, assessment, zero or more diagnostics, then treatment; or legacy two-stage");
        }
        total_stages = total_stages
            .checked_add(patient.stages.len())
            .ok_or_else(|| PathwayError::InvalidInput("stage count overflow".into()))?;
        total_events = total_events
            .checked_add(
                1 + u64::try_from(patient.stages.len()).map_err(|_| {
                    PathwayError::InvalidInput("stage count exceeds event range".into())
                })?,
            )
            .ok_or_else(|| PathwayError::InvalidInput("scheduler event count overflow".into()))?;
        if patient.stages.len() + 1 > 20 {
            return invalid("per-patient scheduler event limit exceeded");
        }
        for stage in &patient.stages {
            if !valid_identifier(&stage.resource_id) || !valid_identifier(&stage.task_class_id) {
                return invalid("stage resource and task references must be valid identifiers");
            }
            if stage.duration_ticks == 0 && !allow_discrete_sentinel {
                return invalid("stage duration must be positive");
            }
            if let Some(previous) = cross_kind.insert(&stage.resource_id, stage.kind) {
                if previous != stage.kind {
                    return invalid("a resource cannot serve both stage kinds");
                }
            }
        }
    }
    let _ = total_events;
    let mut validation_work = Vec::with_capacity(total_stages);
    for (index, patient) in config.patients.iter().enumerate() {
        for (stage_index, stage) in patient.stages.iter().enumerate() {
            validation_work.push(PatientWorkItem {
                patient_id: format!("validation-{index}-{stage_index}"),
                arrival_tick: patient.arrival_tick,
                work_ticks: 1,
                resource_id: stage.resource_id.clone(),
                task_class_id: stage.task_class_id.clone(),
            });
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

pub(crate) fn validate_service_bindings(
    config: &PathwayConfig,
    bindings: &[ServiceBinding],
) -> Result<HashMap<(String, usize), ServiceSpec>, PathwayError> {
    let expected = config
        .patients
        .iter()
        .try_fold(0usize, |total, patient| {
            total.checked_add(patient.stages.len())
        })
        .ok_or_else(|| PathwayError::InvalidInput("service binding count overflow".into()))?;
    if bindings.len() != expected {
        return Err(PathwayError::InvalidInput(
            "exactly one service binding is required per stage".into(),
        ));
    }
    let patients: HashMap<&str, &PathwayPatient> = config
        .patients
        .iter()
        .map(|patient| (patient.patient_id.as_str(), patient))
        .collect();
    let mut canonical = HashMap::with_capacity(expected);
    for binding in bindings {
        if !valid_identifier(&binding.patient_id) || binding.patient_id.contains(':') {
            return Err(PathwayError::InvalidInput(
                "service binding patient ID is invalid".into(),
            ));
        }
        let patient = patients.get(binding.patient_id.as_str()).ok_or_else(|| {
            PathwayError::InvalidInput("service binding refers to an unknown patient".into())
        })?;
        let stage = patient.stages.get(binding.stage_index).ok_or_else(|| {
            PathwayError::InvalidInput("service binding stage index is out of range".into())
        })?;
        let source = &binding.service;
        if source.version != 1
            || source.provenance.class != "synthetic"
            || source.provenance.notes.trim().is_empty()
            || source.provenance.notes.trim() != source.provenance.notes
        {
            return Err(PathwayError::InvalidInput(
                "service must use version 1 and trimmed synthetic provenance".into(),
            ));
        }
        match &source.law {
            ServiceLaw::Fixed { duration_ticks } => {
                if *duration_ticks == 0 || stage.duration_ticks != *duration_ticks {
                    return Err(PathwayError::InvalidInput("fixed service duration must be positive and equal StageWork.duration_ticks".into()));
                }
            }
            ServiceLaw::Discrete { outcomes } => {
                if stage.duration_ticks != 0 {
                    return Err(PathwayError::InvalidInput(
                        "discrete service requires zero StageWork duration sentinel".into(),
                    ));
                }
                if outcomes.is_empty() || outcomes.len() > 64 {
                    return Err(PathwayError::InvalidInput(
                        "discrete service support must contain 1..64 outcomes".into(),
                    ));
                }
                let mut durations = HashSet::with_capacity(outcomes.len());
                let mut total_weight = 0u64;
                for outcome in outcomes {
                    if outcome.duration_ticks == 0 || outcome.weight == 0 {
                        return Err(PathwayError::InvalidInput(
                            "discrete durations and weights must be positive".into(),
                        ));
                    }
                    if !durations.insert(outcome.duration_ticks) {
                        return Err(PathwayError::InvalidInput(
                            "discrete durations must be distinct".into(),
                        ));
                    }
                    total_weight = total_weight.checked_add(outcome.weight).ok_or_else(|| {
                        PathwayError::InvalidInput("discrete total weight overflow".into())
                    })?;
                }
                if total_weight == 0 {
                    return Err(PathwayError::InvalidInput(
                        "discrete total weight must be positive".into(),
                    ));
                }
            }
        }
        let mut service = source.clone();
        if let ServiceLaw::Discrete { outcomes } = &mut service.law {
            outcomes.sort_by_key(|outcome| outcome.duration_ticks);
        }
        let key = (binding.patient_id.clone(), binding.stage_index);
        if canonical.insert(key, service).is_some() {
            return Err(PathwayError::InvalidInput(
                "duplicate service binding".into(),
            ));
        }
    }
    if canonical.len() != expected {
        return Err(PathwayError::InvalidInput(
            "service bindings are missing configured stages".into(),
        ));
    }
    Ok(canonical)
}

fn service_domain_seed(root_seed: u64) -> u64 {
    derive_entity_seed(root_seed, EntityId::new(SERVICE_DOMAIN_INDEX, 0))
}

fn service_episode_seed(
    root_seed: u64,
    patient_ordinal: usize,
    stage_index: usize,
) -> Result<u64, PathwayError> {
    let slot = patient_ordinal
        .checked_mul(32)
        .and_then(|base| base.checked_add(stage_index))
        .and_then(|slot| u64::try_from(slot).ok())
        .ok_or_else(|| PathwayError::InvalidInput("service episode slot overflow".into()))?;
    Ok(derive_entity_seed(
        service_domain_seed(root_seed),
        EntityId::new(slot, 0),
    ))
}

fn sample_episode_service(
    service: &ServiceSpec,
    root_seed: u64,
    patient_ordinal: usize,
    stage_index: usize,
) -> Result<u64, PathwayError> {
    match service.law {
        ServiceLaw::Fixed { duration_ticks } => Ok(duration_ticks),
        ServiceLaw::Discrete { .. } => {
            let mut stream = DeterministicStream::from_seed(service_episode_seed(
                root_seed,
                patient_ordinal,
                stage_index,
            )?);
            sample_service_duration(service, &mut || stream.next_u64())
        }
    }
}

fn sample_service_duration(
    service: &ServiceSpec,
    draw: &mut impl FnMut() -> u64,
) -> Result<u64, PathwayError> {
    match &service.law {
        ServiceLaw::Fixed { duration_ticks } => Ok(*duration_ticks),
        ServiceLaw::Discrete { outcomes } => {
            let total = outcomes
                .iter()
                .try_fold(0u64, |sum, outcome| sum.checked_add(outcome.weight))
                .ok_or_else(|| {
                    PathwayError::InvalidInput("discrete total weight overflow".into())
                })?;
            select_discrete_duration(outcomes, total, draw)
        }
    }
}

fn select_discrete_duration(
    outcomes: &[ServiceOutcome],
    total_weight: u64,
    draw: &mut impl FnMut() -> u64,
) -> Result<u64, PathwayError> {
    if total_weight == 0 || outcomes.is_empty() {
        return Err(PathwayError::InvalidInput(
            "discrete support and total weight must be positive".into(),
        ));
    }
    if total_weight == 1 {
        return Ok(outcomes[0].duration_ticks);
    }
    let threshold = total_weight.wrapping_neg() % total_weight;
    for _ in 0..MAX_DRAWS {
        let raw = draw();
        if raw < threshold {
            continue;
        }
        let selected = raw % total_weight;
        let mut cumulative = 0u64;
        for outcome in outcomes {
            cumulative = cumulative.checked_add(outcome.weight).ok_or_else(|| {
                PathwayError::Runtime("discrete cumulative weight overflow".into())
            })?;
            if selected < cumulative {
                return Ok(outcome.duration_ticks);
            }
        }
        return Err(PathwayError::Runtime(
            "discrete selector exceeded validated cumulative weight".into(),
        ));
    }
    Err(PathwayError::Runtime(
        "discrete sampling exhausted 128 draws".into(),
    ))
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

fn valid_stage_sequence(stages: &[StageWork]) -> bool {
    if stages.len() == 2 {
        return stages[0].kind == StageKind::Triage && stages[1].kind == StageKind::Assessment;
    }
    if !(3..=MAX_STAGES).contains(&stages.len())
        || stages[0].kind != StageKind::Triage
        || stages[1].kind != StageKind::Assessment
        || stages.last().map(|stage| stage.kind) != Some(StageKind::Treatment)
    {
        return false;
    }
    stages[2..stages.len() - 1].len() <= MAX_DIAGNOSTICS
        && stages[2..stages.len() - 1]
            .iter()
            .all(|stage| stage.kind == StageKind::Diagnostics)
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

    fn terminal_fixture(
        intent: crate::disposition::DispositionIntent,
        offer: bool,
        horizon_ticks: u64,
    ) -> crate::terminal::TerminalConfig {
        let mut patient = patient("p1", 1, 3);
        patient
            .stages
            .push(stage(StageKind::Treatment, "resource.treatment", 5));
        let mut pathway = config(vec![patient], horizon_ticks);
        for id in ["resource.treatment", "resource.space", "resource.transfer"] {
            let mut resource = pathway.capacity_location.resource_buckets[0].clone();
            resource.resource_id = id.into();
            pathway.capacity_location.resource_buckets.push(resource);
        }
        let services = pathway.patients[0]
            .stages
            .iter()
            .enumerate()
            .map(|(stage_index, stage)| ServiceBinding {
                patient_id: "p1".into(),
                stage_index,
                service: ServiceSpec {
                    version: 1,
                    provenance: ServiceProvenance {
                        class: "synthetic".into(),
                        notes: "fixture service".into(),
                    },
                    law: ServiceLaw::Fixed {
                        duration_ticks: stage.duration_ticks,
                    },
                },
            })
            .collect();
        crate::terminal::TerminalConfig {
            version: 1,
            pathway,
            seed: 7,
            services,
            decision_policy: crate::disposition::DecisionPolicy {
                version: 1,
                provenance: ServiceProvenance {
                    class: "synthetic".into(),
                    notes: "fixture decision".into(),
                },
                reveal_at: crate::disposition::DecisionReveal::TreatmentCompletion,
            },
            decisions: vec![crate::disposition::DispositionBinding {
                patient_id: "p1".into(),
                latent_intent: intent,
            }],
            admissions: (intent == crate::disposition::DispositionIntent::Admission)
                .then(|| crate::terminal::AdmissionDestination {
                    patient_id: "p1".into(),
                    destination: "med".into(),
                })
                .into_iter()
                .collect(),
            offer_policy: crate::offers::OfferPolicy {
                version: 1,
                provenance: ServiceProvenance {
                    class: "synthetic".into(),
                    notes: "fixture offers".into(),
                },
            },
            offers: if offer {
                vec![crate::offers::BedOffer {
                    offer_id: "offer1".into(),
                    destination: "med".into(),
                    patient_id: None,
                    available_tick: 13,
                    withdrawn_tick: 21,
                    transfer_duration_ticks: 2,
                }]
            } else {
                Vec::new()
            },
            policy: crate::terminal::TerminalPolicy {
                version: 1,
                provenance: ServiceProvenance {
                    class: "synthetic".into(),
                    notes: "fixture terminal policy".into(),
                },
                ed_space_resource_id: "resource.space".into(),
                transfer_resource_id: "resource.transfer".into(),
                transfer_task_class_id: "task.synthetic".into(),
                discharge_delay_ticks: 0,
                abandonment: crate::terminal::AbandonmentPolicy::Disabled,
                boarder_care: None,
            },
        }
    }

    fn terminal_pair(horizon_ticks: u64) -> crate::terminal::TerminalConfig {
        use crate::disposition::{DispositionBinding, DispositionIntent};
        use crate::offers::BedOffer;
        let mut cfg = terminal_fixture(DispositionIntent::Admission, true, horizon_ticks);
        let mut second = cfg.pathway.patients[0].clone();
        second.patient_id = "p2".into();
        second.arrival_tick = 1;
        cfg.pathway.patients.push(second.clone());
        let mut second_services = cfg
            .services
            .iter()
            .map(|binding| {
                let mut binding = binding.clone();
                binding.patient_id = "p2".into();
                binding
            })
            .collect::<Vec<_>>();
        cfg.services.append(&mut second_services);
        cfg.decisions.push(DispositionBinding {
            patient_id: "p2".into(),
            latent_intent: DispositionIntent::Admission,
        });
        cfg.admissions.push(crate::terminal::AdmissionDestination {
            patient_id: "p2".into(),
            destination: "med".into(),
        });
        cfg.offers = vec![
            BedOffer {
                offer_id: "offer1".into(),
                destination: "med".into(),
                patient_id: None,
                available_tick: 13,
                withdrawn_tick: 30,
                transfer_duration_ticks: 2,
            },
            BedOffer {
                offer_id: "offer2".into(),
                destination: "med".into(),
                patient_id: None,
                available_tick: 13,
                withdrawn_tick: 30,
                transfer_duration_ticks: 2,
            },
        ];
        for resource in &mut cfg.pathway.capacity_location.resource_buckets {
            if resource.resource_id != "resource.transfer" {
                resource.physical_count = Some(2);
                resource.open_count = Some(2);
                resource.staffed_count = Some(2);
            }
        }
        cfg
    }

    fn append_terminal_patient(
        cfg: &mut crate::terminal::TerminalConfig,
        patient_id: &str,
        arrival_tick: u64,
        intent: crate::disposition::DispositionIntent,
    ) {
        let mut patient = cfg.pathway.patients[0].clone();
        patient.patient_id = patient_id.into();
        patient.arrival_tick = arrival_tick;
        cfg.pathway.patients.push(patient);
        let bindings = cfg
            .services
            .iter()
            .filter(|binding| binding.patient_id == "p1")
            .map(|binding| {
                let mut binding = binding.clone();
                binding.patient_id = patient_id.into();
                binding
            })
            .collect::<Vec<_>>();
        cfg.services.extend(bindings);
        cfg.decisions.push(crate::disposition::DispositionBinding {
            patient_id: patient_id.into(),
            latent_intent: intent,
        });
        if intent == crate::disposition::DispositionIntent::Admission {
            cfg.admissions.push(crate::terminal::AdmissionDestination {
                patient_id: patient_id.into(),
                destination: "med".into(),
            });
        }
    }

    fn set_terminal_resource_capacity(
        cfg: &mut crate::terminal::TerminalConfig,
        resource_id: &str,
        capacity: u64,
    ) {
        let resource = cfg
            .pathway
            .capacity_location
            .resource_buckets
            .iter_mut()
            .find(|resource| resource.resource_id == resource_id)
            .unwrap();
        resource.physical_count = Some(capacity);
        resource.open_count = Some(capacity);
        resource.staffed_count = Some(capacity);
    }

    fn make_abandonment_candidate(cfg: &mut crate::terminal::TerminalConfig, patience_ticks: u64) {
        cfg.offers.clear();
        cfg.policy.abandonment =
            crate::terminal::AbandonmentPolicy::BeforeFirstCare { patience_ticks };
    }

    #[test]
    fn abandonment_policy_is_explicit_and_disabled_stays_unchanged() {
        let positive = crate::terminal::AbandonmentPolicy::BeforeFirstCare { patience_ticks: 7 };
        let json = serde_json::to_string(&positive).unwrap();
        assert_eq!(json, r#"{"before_first_care":{"patience_ticks":7}}"#);
        assert_eq!(
            serde_json::from_str::<crate::terminal::AbandonmentPolicy>(&json).unwrap(),
            positive
        );
        assert!(serde_json::from_str::<crate::terminal::AbandonmentPolicy>(
            r#"{"before_first_care":{"patience_ticks":7,"unexpected":1}}"#
        )
        .is_err());
        let disabled = crate::terminal::run_terminal(&terminal_fixture(
            crate::disposition::DispositionIntent::Discharge,
            false,
            30,
        ))
        .unwrap();
        assert_eq!(disabled.counts.abandoned, 0);
        assert_eq!(disabled.patients[0].terminal_tick, Some(11));
        let mut zero =
            terminal_fixture(crate::disposition::DispositionIntent::Discharge, false, 30);
        zero.policy.abandonment =
            crate::terminal::AbandonmentPolicy::BeforeFirstCare { patience_ticks: 0 };
        assert!(matches!(
            crate::terminal::run_terminal(&zero),
            Err(PathwayError::InvalidInput(message)) if message.contains("patience must be positive")
        ));
    }

    #[test]
    fn abandonment_outside_space_removes_queue_without_acquiring_or_releasing() {
        let mut cfg = terminal_pair(20);
        cfg.decisions[1].latent_intent = crate::disposition::DispositionIntent::Discharge;
        cfg.admissions.retain(|item| item.patient_id == "p1");
        make_abandonment_candidate(&mut cfg, 1);
        set_terminal_resource_capacity(&mut cfg, "resource.space", 1);
        set_terminal_resource_capacity(&mut cfg, "resource.synthetic", 1);
        let result = crate::terminal::run_terminal(&cfg).unwrap();
        println!(
            "E13_ABANDON_OUTSIDE_TRACE {}",
            serde_json::to_string(&result).unwrap()
        );
        let patient = result
            .patients
            .iter()
            .find(|patient| patient.pathway.patient_id == "p2")
            .unwrap();
        assert_eq!(
            patient.realized_outcome,
            Some(crate::terminal::RealizedOutcome::Abandoned)
        );
        assert_eq!(patient.terminal_tick, Some(2));
        assert_eq!(patient.completed_los_ticks, Some(1));
        assert_eq!(patient.space.acquired_tick, None);
        assert_eq!(patient.space.released_tick, None);
        assert_eq!(patient.space.observed_occupancy_ticks, 0);
        assert!(patient.pathway.episodes.is_empty());
        assert_eq!(result.counts.abandoned, 1);
        let space = result
            .resources
            .iter()
            .find(|r| r.resource_id == "resource.space")
            .unwrap();
        assert_eq!(space.queued_at_horizon, 0);
        assert_eq!(space.busy_ticks, 19);
        assert_eq!(space.busy_at_horizon, 1);
    }

    #[test]
    fn abandonment_cancels_queued_triage_releases_space_and_regrants_at_same_tick() {
        let mut cfg = terminal_pair(3);
        cfg.decisions[1].latent_intent = crate::disposition::DispositionIntent::Discharge;
        cfg.admissions.retain(|item| item.patient_id == "p1");
        append_terminal_patient(
            &mut cfg,
            "p3",
            2,
            crate::disposition::DispositionIntent::Discharge,
        );
        make_abandonment_candidate(&mut cfg, 1);
        set_terminal_resource_capacity(&mut cfg, "resource.space", 2);
        set_terminal_resource_capacity(&mut cfg, "resource.synthetic", 1);
        let result = crate::terminal::run_terminal(&cfg).unwrap();
        println!(
            "E13_ABANDON_QUEUED_TRACE {}",
            serde_json::to_string(&result).unwrap()
        );
        let p2 = result
            .patients
            .iter()
            .find(|p| p.pathway.patient_id == "p2")
            .unwrap();
        let p3 = result
            .patients
            .iter()
            .find(|p| p.pathway.patient_id == "p3")
            .unwrap();
        let triage = &p2.pathway.episodes[0];
        assert_eq!(triage.queue_entry_tick, 1);
        assert_eq!(triage.start_tick, None);
        assert_eq!(triage.completion_tick, None);
        assert_eq!(triage.observed_work_ticks, 0);
        assert_eq!(p2.space.acquired_tick, Some(1));
        assert_eq!(p2.space.released_tick, Some(2));
        assert_eq!(p2.space.observed_occupancy_ticks, 1);
        assert_eq!(p2.terminal_tick, Some(2));
        assert_eq!(p3.space.acquired_tick, Some(2));
        assert_eq!(p3.pathway.episodes[0].queue_entry_tick, 2);
        assert_eq!(p3.pathway.episodes[0].start_tick, Some(3));
        assert_eq!(p3.pathway.episodes[0].observed_work_ticks, 0);
        let triage_pool = result
            .resources
            .iter()
            .find(|r| r.resource_id == "resource.synthetic")
            .unwrap();
        assert_eq!(triage_pool.queued_at_horizon, 0);
        let space = result
            .resources
            .iter()
            .find(|r| r.resource_id == "resource.space")
            .unwrap();
        assert_eq!(space.busy_ticks, 4);
        assert_eq!(space.busy_at_horizon, 2);
        assert_eq!(space.peak_busy, 2);
        assert_eq!(space.capacity_ticks, 6);
    }

    #[test]
    fn abandonment_removes_triage_from_its_actual_pool_after_input_permutation() {
        let mut cfg = terminal_pair(6);
        cfg.pathway.patients[0].stages[0].duration_ticks = 4;
        cfg.services
            .iter_mut()
            .find(|binding| binding.patient_id == "p1" && binding.stage_index == 0)
            .unwrap()
            .service
            .law = ServiceLaw::Fixed { duration_ticks: 4 };
        cfg.pathway.patients[1].arrival_tick = 2;
        append_terminal_patient(
            &mut cfg,
            "p3",
            4,
            crate::disposition::DispositionIntent::Discharge,
        );
        let mut triage_pool = cfg
            .pathway
            .capacity_location
            .resource_buckets
            .iter()
            .find(|resource| resource.resource_id == "resource.synthetic")
            .unwrap()
            .clone();
        triage_pool.resource_id = "resource.triage.alt".into();
        cfg.pathway
            .capacity_location
            .resource_buckets
            .push(triage_pool);
        cfg.pathway
            .patients
            .iter_mut()
            .find(|patient| patient.patient_id == "p3")
            .unwrap()
            .stages[0]
            .resource_id = "resource.triage.alt".into();
        cfg.decisions[1].latent_intent = crate::disposition::DispositionIntent::Discharge;
        cfg.admissions.retain(|item| item.patient_id == "p1");
        make_abandonment_candidate(&mut cfg, 1);
        set_terminal_resource_capacity(&mut cfg, "resource.synthetic", 1);
        set_terminal_resource_capacity(&mut cfg, "resource.triage.alt", 1);
        cfg.pathway.patients = vec![
            cfg.pathway.patients[0].clone(),
            cfg.pathway
                .patients
                .iter()
                .find(|patient| patient.patient_id == "p3")
                .unwrap()
                .clone(),
            cfg.pathway
                .patients
                .iter()
                .find(|patient| patient.patient_id == "p2")
                .unwrap()
                .clone(),
        ];
        let result = crate::terminal::run_terminal(&cfg).unwrap();
        let p2 = result
            .patients
            .iter()
            .find(|p| p.pathway.patient_id == "p2")
            .unwrap();
        assert_eq!(p2.terminal_tick, Some(3));
        assert_eq!(p2.pathway.episodes[0].queue_entry_tick, 2);
        assert_eq!(p2.pathway.episodes[0].start_tick, None);
        let synthetic = result
            .resources
            .iter()
            .find(|r| r.resource_id == "resource.synthetic")
            .unwrap();
        assert_eq!(synthetic.queued_at_horizon, 0);
        assert_eq!(synthetic.busy_at_horizon, 0);
    }

    #[test]
    fn triage_start_at_deadline_wins_even_when_it_is_the_horizon() {
        let mut cfg = terminal_pair(3);
        cfg.decisions[1].latent_intent = crate::disposition::DispositionIntent::Discharge;
        cfg.admissions.retain(|item| item.patient_id == "p1");
        make_abandonment_candidate(&mut cfg, 2);
        set_terminal_resource_capacity(&mut cfg, "resource.space", 2);
        set_terminal_resource_capacity(&mut cfg, "resource.synthetic", 1);
        let result = crate::terminal::run_terminal(&cfg).unwrap();
        let p2 = result
            .patients
            .iter()
            .find(|p| p.pathway.patient_id == "p2")
            .unwrap();
        assert_eq!(p2.pathway.episodes[0].start_tick, Some(3));
        assert_eq!(p2.pathway.episodes[0].observed_work_ticks, 0);
        assert_eq!(p2.realized_outcome, None);
        assert_eq!(p2.terminal_tick, None);
        assert_eq!(result.counts.abandoned, 0);
        assert_eq!(result.counts.unfinished, 2);
    }

    #[test]
    fn simultaneous_expired_waiters_use_pre_cancellation_snapshot_and_are_permutation_stable() {
        let mut cfg = terminal_pair(20);
        append_terminal_patient(
            &mut cfg,
            "p3",
            1,
            crate::disposition::DispositionIntent::Admission,
        );
        make_abandonment_candidate(&mut cfg, 1);
        set_terminal_resource_capacity(&mut cfg, "resource.space", 2);
        set_terminal_resource_capacity(&mut cfg, "resource.synthetic", 1);
        let result = crate::terminal::run_terminal(&cfg).unwrap();
        assert_eq!(result.counts.abandoned, 2);
        let p2 = result
            .patients
            .iter()
            .find(|p| p.pathway.patient_id == "p2")
            .unwrap();
        let p3 = result
            .patients
            .iter()
            .find(|p| p.pathway.patient_id == "p3")
            .unwrap();
        assert_eq!(p2.terminal_tick, Some(2));
        assert_eq!(p2.space.acquired_tick, Some(1));
        assert_eq!(p2.space.released_tick, Some(2));
        assert_eq!(p3.terminal_tick, Some(2));
        assert_eq!(p3.space.acquired_tick, None);
        let mut permuted = cfg;
        permuted.pathway.patients.reverse();
        permuted.services.reverse();
        permuted.decisions.reverse();
        permuted.admissions.reverse();
        permuted.offers.reverse();
        permuted
            .pathway
            .capacity_location
            .resource_buckets
            .reverse();
        assert_eq!(result, crate::terminal::run_terminal(&permuted).unwrap());
        assert_eq!(result.counts.arrived, 3);
        assert_eq!(
            result.counts.arrived,
            result.counts.discharged
                + result.counts.transferred
                + result.counts.abandoned
                + result.counts.unfinished
        );
    }

    #[test]
    fn first_triage_start_permanently_disables_abandonment_during_later_states() {
        let mut cfg = terminal_fixture(crate::disposition::DispositionIntent::Admission, false, 30);
        make_abandonment_candidate(&mut cfg, 12);
        let result = crate::terminal::run_terminal(&cfg).unwrap();
        let patient = &result.patients[0];
        assert_eq!(patient.pathway.episodes[0].start_tick, Some(1));
        assert_eq!(patient.decision.as_ref().unwrap().observed_tick, 11);
        assert_eq!(
            patient.endpoint_state,
            crate::terminal::EndpointState::Boarding
        );
        assert_eq!(patient.realized_outcome, None);
        assert_eq!(patient.terminal_tick, None);
        assert_eq!(result.counts.abandoned, 0);
    }

    #[test]
    fn abandonment_at_horizon_realizes_but_future_and_overflow_deadlines_are_censored() {
        let mut at_horizon = terminal_pair(2);
        at_horizon.decisions[1].latent_intent = crate::disposition::DispositionIntent::Discharge;
        at_horizon.admissions.retain(|item| item.patient_id == "p1");
        make_abandonment_candidate(&mut at_horizon, 1);
        set_terminal_resource_capacity(&mut at_horizon, "resource.space", 2);
        set_terminal_resource_capacity(&mut at_horizon, "resource.synthetic", 1);
        let result = crate::terminal::run_terminal(&at_horizon).unwrap();
        let p2 = result
            .patients
            .iter()
            .find(|p| p.pathway.patient_id == "p2")
            .unwrap();
        assert_eq!(p2.terminal_tick, Some(2));
        assert_eq!(p2.completed_los_ticks, Some(1));
        assert_eq!(p2.space.released_tick, Some(2));
        assert_eq!(result.counts.abandoned, 1);
        let mut future = at_horizon.clone();
        future.pathway.horizon_ticks = 2;
        future.policy.abandonment =
            crate::terminal::AbandonmentPolicy::BeforeFirstCare { patience_ticks: 2 };
        let future_result = crate::terminal::run_terminal(&future).unwrap();
        let future_p2 = future_result
            .patients
            .iter()
            .find(|p| p.pathway.patient_id == "p2")
            .unwrap();
        assert_eq!(future_p2.terminal_tick, None);
        assert_eq!(future_result.counts.abandoned, 0);
        assert_eq!(future_p2.space.released_tick, None);

        let mut overflow = terminal_pair(u64::MAX);
        overflow.decisions[1].latent_intent = crate::disposition::DispositionIntent::Discharge;
        overflow.admissions.retain(|item| item.patient_id == "p1");
        make_abandonment_candidate(&mut overflow, 2);
        for patient in &mut overflow.pathway.patients {
            patient.arrival_tick = u64::MAX - 1;
        }
        set_terminal_resource_capacity(&mut overflow, "resource.space", 2);
        set_terminal_resource_capacity(&mut overflow, "resource.synthetic", 1);
        let overflow_result = crate::terminal::run_terminal(&overflow).unwrap();
        assert_eq!(overflow_result.counts.abandoned, 0);
        assert!(overflow_result
            .patients
            .iter()
            .all(|p| p.terminal_tick.is_none()));
    }

    #[test]
    fn terminal_admission_uses_live_transfer_and_retains_space_until_departure() {
        let result = crate::terminal::run_terminal(&terminal_fixture(
            crate::disposition::DispositionIntent::Admission,
            true,
            30,
        ))
        .unwrap();
        let patient = &result.patients[0];
        assert_eq!(patient.decision.as_ref().unwrap().observed_tick, 11);
        assert_eq!(patient.boarding.as_ref().unwrap().end_tick, Some(13));
        assert_eq!(patient.boarding.as_ref().unwrap().observed_ticks, 2);
        assert_eq!(patient.transfer.as_ref().unwrap().start_tick, Some(13));
        assert_eq!(patient.transfer.as_ref().unwrap().completion_tick, Some(15));
        assert_eq!(patient.space.acquired_tick, Some(1));
        assert_eq!(patient.space.released_tick, Some(15));
        assert_eq!(patient.space.observed_occupancy_ticks, 14);
        assert_eq!(patient.boarding_time_ticks, Some(4));
        assert_eq!(patient.completed_los_ticks, Some(14));
        assert_eq!(
            result.counts,
            crate::terminal::TerminalCounts {
                arrived: 1,
                discharged: 0,
                transferred: 1,
                abandoned: 0,
                unfinished: 0,
            }
        );
    }

    #[test]
    fn terminal_discharge_and_no_offer_are_distinct_realized_and_censored_states() {
        let discharge = crate::terminal::run_terminal(&terminal_fixture(
            crate::disposition::DispositionIntent::Discharge,
            false,
            30,
        ))
        .unwrap();
        assert_eq!(discharge.patients[0].terminal_tick, Some(11));
        assert_eq!(discharge.patients[0].completed_los_ticks, Some(10));
        assert_eq!(discharge.counts.discharged, 1);

        let boarding = crate::terminal::run_terminal(&terminal_fixture(
            crate::disposition::DispositionIntent::Admission,
            false,
            30,
        ))
        .unwrap();
        let patient = &boarding.patients[0];
        assert_eq!(
            patient.endpoint_state,
            crate::terminal::EndpointState::Boarding
        );
        assert_eq!(
            patient.unfinished_reason,
            Some(crate::terminal::UnfinishedReason::AwaitingOfferAllocation)
        );
        assert_eq!(patient.boarding.as_ref().unwrap().observed_ticks, 19);
        assert_eq!(patient.space.observed_occupancy_ticks, 29);
        assert_eq!(patient.terminal_tick, None);
        assert_eq!(patient.completed_los_ticks, None);
        assert_eq!(boarding.counts.unfinished, 1);
    }

    #[test]
    fn terminal_exact_horizon_censors_transfer_and_releases_space_only_on_departure() {
        let result = crate::terminal::run_terminal(&terminal_fixture(
            crate::disposition::DispositionIntent::Admission,
            true,
            14,
        ))
        .unwrap();
        let patient = &result.patients[0];
        assert_eq!(patient.transfer.as_ref().unwrap().start_tick, Some(13));
        assert_eq!(patient.transfer.as_ref().unwrap().observed_work_ticks, 1);
        assert_eq!(patient.transfer.as_ref().unwrap().completion_tick, None);
        assert_eq!(patient.space.released_tick, None);
        assert_eq!(patient.space.observed_occupancy_ticks, 13);
        assert_eq!(patient.completed_los_ticks, None);
        let transfer = result
            .resources
            .iter()
            .find(|r| r.resource_id == "resource.transfer")
            .unwrap();
        assert_eq!(transfer.busy_ticks, 1);
        assert_eq!(transfer.busy_at_horizon, 1);
        assert_eq!(transfer.queued_at_horizon, 0);
        assert_eq!(transfer.capacity_ticks, 14);
        let space = result
            .resources
            .iter()
            .find(|r| r.resource_id == "resource.space")
            .unwrap();
        assert_eq!(space.busy_ticks, 13);
        assert_eq!(space.busy_at_horizon, 1);
        assert_eq!(space.capacity_ticks, 14);
    }

    #[test]
    fn terminal_due_care_is_cancelled_before_same_tick_offer_work_grants() {
        let mut cfg = terminal_fixture(crate::disposition::DispositionIntent::Admission, true, 20);
        cfg.policy.boarder_care = Some(crate::terminal::BoarderCarePolicy {
            resource_id: "resource.assessment".into(),
            task_class_id: "task.synthetic".into(),
            interval_ticks: 2,
            duration_ticks: 3,
        });
        let result = crate::terminal::run_terminal(&cfg).unwrap();
        let care = &result.patients[0].care[0];
        assert_eq!(care.queue_entry_tick, 13);
        assert_eq!(care.cancelled_tick, Some(13));
        assert_eq!(care.start_tick, None);
        assert_eq!(care.observed_work_ticks, 0);
        assert_eq!(
            result.patients[0].transfer.as_ref().unwrap().start_tick,
            Some(13)
        );
        assert_eq!(
            result
                .resources
                .iter()
                .find(|r| r.resource_id == "resource.assessment")
                .unwrap()
                .queued_at_horizon,
            0
        );
    }

    #[test]
    fn terminal_active_care_cannot_be_matched_and_offer_expires() {
        let mut cfg = terminal_fixture(crate::disposition::DispositionIntent::Admission, true, 20);
        cfg.offers[0].available_tick = 14;
        cfg.offers[0].withdrawn_tick = 15;
        cfg.policy.boarder_care = Some(crate::terminal::BoarderCarePolicy {
            resource_id: "resource.assessment".into(),
            task_class_id: "task.synthetic".into(),
            interval_ticks: 2,
            duration_ticks: 3,
        });
        let result = crate::terminal::run_terminal(&cfg).unwrap();
        assert_eq!(result.patients[0].care[0].start_tick, Some(13));
        assert_eq!(result.patients[0].care[0].completion_tick, Some(16));
        assert_eq!(result.patients[0].care.len(), 2);
        assert_eq!(result.patients[0].care[1].queue_entry_tick, 18);
        assert_eq!(result.patients[0].care[1].start_tick, Some(18));
        assert_eq!(result.patients[0].care[1].completion_tick, None);
        assert_eq!(result.patients[0].care[1].observed_work_ticks, 2);
        assert_eq!(result.patients[0].allocation, None);
        assert_eq!(result.offers[0].status, crate::offers::OfferStatus::Expired);
    }

    #[test]
    fn terminal_care_repeats_through_shared_clinical_resource_queue() {
        let mut cfg = terminal_pair(20);
        cfg.pathway.patients[1].arrival_tick = 11;
        cfg.decisions[1].latent_intent = crate::disposition::DispositionIntent::Discharge;
        cfg.admissions
            .retain(|admission| admission.patient_id == "p1");
        cfg.offers.clear();
        cfg.policy.boarder_care = Some(crate::terminal::BoarderCarePolicy {
            resource_id: "resource.assessment".into(),
            task_class_id: "task.synthetic".into(),
            interval_ticks: 2,
            duration_ticks: 3,
        });
        for resource in &mut cfg.pathway.capacity_location.resource_buckets {
            let capacity = if resource.resource_id == "resource.assessment" {
                1
            } else {
                2
            };
            resource.physical_count = Some(capacity);
            resource.open_count = Some(capacity);
            resource.staffed_count = Some(capacity);
        }
        let result = crate::terminal::run_terminal(&cfg).unwrap();
        let p1 = result
            .patients
            .iter()
            .find(|p| p.pathway.patient_id == "p1")
            .unwrap();
        let p2 = result
            .patients
            .iter()
            .find(|p| p.pathway.patient_id == "p2")
            .unwrap();
        assert_eq!(p1.care[0].start_tick, Some(13));
        assert_eq!(p1.care[0].completion_tick, Some(16));
        assert_eq!(p2.pathway.episodes[1].queue_entry_tick, 13);
        assert_eq!(p2.pathway.episodes[1].start_tick, Some(16));
        assert_eq!(p2.pathway.episodes[1].completion_tick, Some(19));
        assert_eq!(p1.care[1].queue_entry_tick, 18);
        assert_eq!(p1.care[1].start_tick, Some(19));
        assert_eq!(p1.care[1].observed_work_ticks, 1);
        assert_eq!(p1.care[1].completion_tick, None);
    }

    #[test]
    fn terminal_validation_rejects_care_budget_alias_and_missing_destination_before_run() {
        let valid = terminal_fixture(crate::disposition::DispositionIntent::Admission, true, 20);
        let mut over_budget = valid.clone();
        over_budget.pathway.horizon_ticks = 1_000_001;
        over_budget.policy.boarder_care = Some(crate::terminal::BoarderCarePolicy {
            resource_id: "resource.assessment".into(),
            task_class_id: "task.synthetic".into(),
            interval_ticks: 1,
            duration_ticks: 1,
        });
        assert!(matches!(
            crate::terminal::run_terminal(&over_budget),
            Err(PathwayError::InvalidInput(message)) if message.contains("care episode limit")
        ));
        let mut alias = valid.clone();
        alias.policy.ed_space_resource_id = alias.policy.transfer_resource_id.clone();
        assert!(matches!(
            crate::terminal::run_terminal(&alias),
            Err(PathwayError::InvalidInput(message)) if message.contains("invalid or aliased")
        ));
        let mut missing_destination = valid.clone();
        missing_destination.admissions.clear();
        assert!(matches!(
            crate::terminal::run_terminal(&missing_destination),
            Err(PathwayError::InvalidInput(message)) if message.contains("explicit destination")
        ));
        assert!(crate::terminal::run_terminal(&valid).is_ok());
    }

    #[test]
    fn terminal_transfer_pool_is_fcfs_and_input_permutations_are_stable() {
        let cfg = terminal_pair(25);
        let result = crate::terminal::run_terminal(&cfg).unwrap();
        let by_id: HashMap<_, _> = result
            .patients
            .iter()
            .map(|p| (p.pathway.patient_id.as_str(), p))
            .collect();
        assert_eq!(by_id["p1"].transfer.as_ref().unwrap().start_tick, Some(13));
        assert_eq!(
            by_id["p1"].transfer.as_ref().unwrap().completion_tick,
            Some(15)
        );
        assert_eq!(by_id["p2"].transfer.as_ref().unwrap().start_tick, Some(15));
        assert_eq!(
            by_id["p2"].transfer.as_ref().unwrap().completion_tick,
            Some(17)
        );
        let mut permuted = cfg;
        permuted.pathway.patients.reverse();
        permuted.services.reverse();
        permuted.decisions.reverse();
        permuted.admissions.reverse();
        permuted.offers.reverse();
        permuted
            .pathway
            .capacity_location
            .resource_buckets
            .reverse();
        assert_eq!(result, crate::terminal::run_terminal(&permuted).unwrap());
    }

    #[test]
    fn terminal_space_is_acquired_before_triage_and_held_through_departure() {
        let mut cfg = terminal_pair(30);
        for resource in &mut cfg.pathway.capacity_location.resource_buckets {
            let capacity = if resource.resource_id == "resource.space" {
                1
            } else {
                2
            };
            resource.physical_count = Some(capacity);
            resource.open_count = Some(capacity);
            resource.staffed_count = Some(capacity);
        }
        let result = crate::terminal::run_terminal(&cfg).unwrap();
        let by_id: HashMap<_, _> = result
            .patients
            .iter()
            .map(|p| (p.pathway.patient_id.as_str(), p))
            .collect();
        assert_eq!(by_id["p1"].space.acquired_tick, Some(1));
        assert_eq!(by_id["p1"].space.released_tick, Some(15));
        assert_eq!(by_id["p2"].space.queue_entry_tick, 1);
        assert_eq!(by_id["p2"].space.acquired_tick, Some(15));
        assert_eq!(by_id["p2"].pathway.episodes[0].start_tick, Some(15));
        assert_eq!(by_id["p2"].terminal_tick, Some(27));
    }

    #[test]
    fn terminal_departure_at_horizon_releases_space_and_grants_zero_work() {
        let mut cfg = terminal_pair(15);
        for resource in &mut cfg.pathway.capacity_location.resource_buckets {
            let capacity = if resource.resource_id == "resource.space" {
                1
            } else {
                2
            };
            resource.physical_count = Some(capacity);
            resource.open_count = Some(capacity);
            resource.staffed_count = Some(capacity);
        }
        let result = crate::terminal::run_terminal(&cfg).unwrap();
        let by_id: HashMap<_, _> = result
            .patients
            .iter()
            .map(|p| (p.pathway.patient_id.as_str(), p))
            .collect();
        assert_eq!(by_id["p1"].terminal_tick, Some(15));
        assert_eq!(by_id["p2"].space.acquired_tick, Some(15));
        assert_eq!(by_id["p2"].pathway.episodes[0].start_tick, Some(15));
        assert_eq!(by_id["p2"].pathway.episodes[0].observed_work_ticks, 0);
        assert_eq!(by_id["p2"].terminal_tick, None);
        let space = result
            .resources
            .iter()
            .find(|r| r.resource_id == "resource.space")
            .unwrap();
        assert_eq!(space.busy_at_horizon, 1);
        assert_eq!(space.busy_ticks, 14);
    }

    #[test]
    fn terminal_resource_metrics_match_independent_interval_sweep() {
        let result = crate::terminal::run_terminal(&terminal_pair(30)).unwrap();
        let mut intervals: HashMap<&str, Vec<(u64, u64)>> = HashMap::new();
        for patient in &result.patients {
            intervals.entry("resource.space").or_default().push((
                patient.space.acquired_tick.unwrap(),
                patient.space.released_tick.unwrap(),
            ));
            for episode in &patient.pathway.episodes {
                let resource = match episode.kind {
                    StageKind::Triage => "resource.synthetic",
                    StageKind::Assessment => "resource.assessment",
                    StageKind::Diagnostics => "resource.diagnostics",
                    StageKind::Treatment => "resource.treatment",
                };
                intervals.entry(resource).or_default().push((
                    episode.start_tick.unwrap(),
                    episode.completion_tick.unwrap(),
                ));
            }
            let transfer = patient.transfer.as_ref().unwrap();
            intervals.entry("resource.transfer").or_default().push((
                transfer.start_tick.unwrap(),
                transfer.completion_tick.unwrap(),
            ));
        }
        for usage in &result.resources {
            let spans = intervals
                .get(usage.resource_id.as_str())
                .cloned()
                .unwrap_or_default();
            let mut events = Vec::new();
            for (start, end) in &spans {
                events.push((*start, 1_i64));
                events.push((*end, -1_i64));
            }
            events.sort_by_key(|(tick, delta)| (*tick, *delta));
            let mut busy = 0_i64;
            let mut peak = 0_u64;
            let mut previous = 0_u64;
            let mut busy_ticks = 0_u128;
            for (tick, delta) in events {
                busy_ticks += u128::from(busy as u64) * u128::from(tick - previous);
                busy += delta;
                peak = peak.max(busy as u64);
                previous = tick;
            }
            let busy_at_horizon = spans.iter().filter(|(_, end)| *end > 30).count() as u64;
            assert_eq!(usage.busy_ticks, busy_ticks, "{}", usage.resource_id);
            assert_eq!(usage.peak_busy, peak, "{}", usage.resource_id);
            assert_eq!(
                usage.busy_at_horizon, busy_at_horizon,
                "{}",
                usage.resource_id
            );
            assert_eq!(usage.capacity_ticks, u128::from(usage.capacity) * 30);
            assert!(usage.busy_at_horizon <= usage.capacity);
            assert!(usage.peak_busy <= usage.capacity);
        }
    }

    #[test]
    fn terminal_u64_max_service_duration_uses_wide_completion_arithmetic() {
        let mut cfg = terminal_fixture(crate::disposition::DispositionIntent::Discharge, false, 20);
        cfg.pathway.patients[0].stages[0].duration_ticks = u64::MAX;
        cfg.services[0].service.law = ServiceLaw::Fixed {
            duration_ticks: u64::MAX,
        };
        let result = crate::terminal::run_terminal(&cfg).unwrap();
        assert_eq!(
            result.patients[0].pathway.episodes[0].observed_work_ticks,
            19
        );
        assert_eq!(result.patients[0].terminal_tick, None);
        assert_eq!(
            result.patients[0].endpoint_state,
            crate::terminal::EndpointState::Clinical
        );
    }

    #[test]
    fn terminal_u64_max_horizon_accumulates_metrics_without_overflow() {
        let mut cfg = terminal_fixture(
            crate::disposition::DispositionIntent::Discharge,
            false,
            u64::MAX,
        );
        cfg.pathway.patients[0].arrival_tick = 0;
        cfg.pathway.patients[0].stages[0].duration_ticks = u64::MAX;
        cfg.services[0].service.law = ServiceLaw::Fixed {
            duration_ticks: u64::MAX,
        };
        let result = crate::terminal::run_terminal(&cfg).unwrap();
        assert_eq!(
            result.patients[0].pathway.episodes[0].observed_work_ticks,
            u64::MAX
        );
        assert_eq!(result.patients[0].terminal_tick, None);
        let triage = result
            .resources
            .iter()
            .find(|r| r.resource_id == "resource.synthetic")
            .unwrap();
        assert_eq!(triage.busy_ticks, u128::from(u64::MAX));
        assert_eq!(triage.capacity_ticks, u128::from(u64::MAX));
        assert_eq!(triage.busy_at_horizon, 0);
    }

    #[test]
    fn terminal_scheduled_event_counter_rejects_first_event_past_bound() {
        let mut count = MAX_TERMINAL_SCHEDULED_EVENTS - 1;
        assert!(count_terminal_event(&mut count).is_ok());
        assert_eq!(count, MAX_TERMINAL_SCHEDULED_EVENTS);
        assert!(matches!(
            count_terminal_event(&mut count),
            Err(PathwayError::InvalidInput(message)) if message.contains("event limit")
        ));
        assert_eq!(count, MAX_TERMINAL_SCHEDULED_EVENTS + 1);
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
            match validate_config(&cfg, true) {
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
            match validate_config(&cfg, true) {
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

    fn extended_patient(id: &str, arrival_tick: u64, diagnostics: usize) -> PathwayPatient {
        let mut stages = vec![
            stage(StageKind::Triage, "resource.synthetic", 2),
            stage(StageKind::Assessment, "resource.assessment", 3),
        ];
        for _ in 0..diagnostics {
            stages.push(stage(StageKind::Diagnostics, "resource.diagnostics", 0));
        }
        stages.push(stage(StageKind::Treatment, "resource.treatment", 5));
        PathwayPatient {
            patient_id: id.into(),
            arrival_tick,
            mode: "alpha".into(),
            acuity: 3,
            stages,
        }
    }

    fn fixed_service(duration_ticks: u64) -> ServiceSpec {
        ServiceSpec {
            version: 1,
            provenance: ServiceProvenance {
                class: "synthetic".into(),
                notes: "synthetic service fixture".into(),
            },
            law: ServiceLaw::Fixed { duration_ticks },
        }
    }

    fn distribution_service() -> ServiceSpec {
        ServiceSpec {
            version: 1,
            provenance: ServiceProvenance {
                class: "synthetic".into(),
                notes: "synthetic discrete fixture".into(),
            },
            law: ServiceLaw::Discrete {
                outcomes: vec![
                    ServiceOutcome {
                        duration_ticks: 9,
                        weight: 4,
                    },
                    ServiceOutcome {
                        duration_ticks: 2,
                        weight: 1,
                    },
                    ServiceOutcome {
                        duration_ticks: 5,
                        weight: 2,
                    },
                ],
            },
        }
    }

    fn bindings_for(patient: &PathwayPatient, diagnostic: ServiceSpec) -> Vec<ServiceBinding> {
        patient
            .stages
            .iter()
            .enumerate()
            .map(|(stage_index, work)| ServiceBinding {
                patient_id: patient.patient_id.clone(),
                stage_index,
                service: if work.kind == StageKind::Diagnostics {
                    diagnostic.clone()
                } else {
                    fixed_service(work.duration_ticks)
                },
            })
            .collect()
    }

    fn add_extended_resources(profile: &mut CapacityLocationProfile) {
        for id in ["resource.diagnostics", "resource.treatment"] {
            let mut resource = profile.resource_buckets[0].clone();
            resource.resource_id = id.into();
            profile.resource_buckets.push(resource);
        }
    }

    #[test]
    fn fixed_pipeline_preserves_legacy_and_adds_diagnostics_treatment_continuation() {
        let legacy = config(vec![patient("p", 1, 3)], 30);
        let legacy_result = run_pathway(&legacy).unwrap();
        let bindings = legacy
            .patients
            .iter()
            .flat_map(|patient| {
                patient
                    .stages
                    .iter()
                    .enumerate()
                    .map(|(stage_index, stage)| ServiceBinding {
                        patient_id: patient.patient_id.clone(),
                        stage_index,
                        service: ServiceSpec {
                            version: 1,
                            provenance: ServiceProvenance {
                                class: "synthetic".into(),
                                notes: legacy.provenance.notes.clone(),
                            },
                            law: ServiceLaw::Fixed {
                                duration_ticks: stage.duration_ticks,
                            },
                        },
                    })
            })
            .collect::<Vec<_>>();
        assert_eq!(
            legacy_result,
            run_pathway_with_services(&legacy, 991, &bindings).unwrap()
        );

        let mut cfg = config(vec![extended_patient("p", 1, 0)], 30);
        add_extended_resources(&mut cfg.capacity_location);
        let patient = &cfg.patients[0];
        let bindings = bindings_for(patient, fixed_service(1));
        let result = run_pathway_with_services(&cfg, 11, &bindings).unwrap();
        assert_eq!(result[0].episodes[2].start_tick, Some(6));
        assert_eq!(result[0].episodes[2].completion_tick, Some(11));
        assert_eq!(result[0].state, PathwayState::TreatmentComplete);
        assert_eq!(result[0].episodes[2].task_id, "p:treatment");

        let mut cfg = config(vec![extended_patient("p", 1, 1)], 30);
        add_extended_resources(&mut cfg.capacity_location);
        cfg.patients[0].stages[2].duration_ticks = 4;
        let bindings = bindings_for(&cfg.patients[0], fixed_service(4));
        let result = run_pathway_with_services(&cfg, 11, &bindings).unwrap();
        assert_eq!(result[0].episodes[2].task_id, "p:diagnostics:0");
        assert_eq!(result[0].episodes[2].completion_tick, Some(10));
        assert_eq!(result[0].episodes[3].start_tick, Some(10));
        assert_eq!(result[0].episodes[3].completion_tick, Some(15));
        assert_eq!(result[0].state, PathwayState::TreatmentComplete);

        let mut cfg = config(vec![extended_patient("p", 1, 2)], 30);
        add_extended_resources(&mut cfg.capacity_location);
        for stage in &mut cfg.patients[0].stages[2..4] {
            stage.duration_ticks = 4;
        }
        let bindings = bindings_for(&cfg.patients[0], fixed_service(4));
        let result = run_pathway_with_services(&cfg, 11, &bindings).unwrap();
        assert_eq!(result[0].episodes[2].task_id, "p:diagnostics:0");
        assert_eq!(result[0].episodes[3].task_id, "p:diagnostics:1");
        assert_eq!(result[0].episodes[2].start_tick, Some(6));
        assert_eq!(result[0].episodes[3].start_tick, Some(10));
        assert_eq!(result[0].episodes[4].start_tick, Some(14));
        assert_eq!(
            result[0]
                .episodes
                .iter()
                .map(|e| e.observed_work_ticks)
                .sum::<u64>(),
            18
        );
    }

    #[test]
    fn root11_diagnostic_and_treatment_sampling_matches_independent_stream_vectors() {
        let mut cfg = config(vec![extended_patient("p", 1, 1)], 30);
        add_extended_resources(&mut cfg.capacity_location);
        cfg.patients[0].stages[2].duration_ticks = 0;
        cfg.patients[0].stages[3].duration_ticks = 0;
        let mut bindings = bindings_for(&cfg.patients[0], distribution_service());
        bindings[3].service = distribution_service();
        assert_eq!(service_domain_seed(11), 15_382_257_649_166_804_290);
        assert_eq!(
            service_episode_seed(11, 0, 2).unwrap(),
            11_268_637_777_555_707_280
        );
        assert_eq!(
            service_episode_seed(11, 0, 3).unwrap(),
            9_156_490_916_263_976_673
        );
        let mut diagnostic_stream =
            DeterministicStream::from_seed(service_episode_seed(11, 0, 2).unwrap());
        let mut treatment_stream =
            DeterministicStream::from_seed(service_episode_seed(11, 0, 3).unwrap());
        assert_eq!(diagnostic_stream.next_u64(), 11_357_213_296_724_872_813);
        assert_eq!(treatment_stream.next_u64(), 7_554_426_951_324_197_283);
        let result = run_pathway_with_services(&cfg, 11, &bindings).unwrap();
        assert_eq!(result[0].episodes[2].sampled_duration_ticks, 9);
        assert_eq!(result[0].episodes[3].sampled_duration_ticks, 9);
        assert_eq!(result[0].episodes[2].start_tick, Some(6));
        assert_eq!(result[0].episodes[2].completion_tick, Some(15));
        assert_eq!(result[0].episodes[3].start_tick, Some(15));
        assert_eq!(result[0].episodes[3].completion_tick, Some(24));
        assert_eq!(
            result[0].episodes[2].service.law,
            ServiceLaw::Discrete {
                outcomes: vec![
                    ServiceOutcome {
                        duration_ticks: 2,
                        weight: 1
                    },
                    ServiceOutcome {
                        duration_ticks: 5,
                        weight: 2
                    },
                    ServiceOutcome {
                        duration_ticks: 9,
                        weight: 4
                    },
                ]
            }
        );
    }

    #[test]
    fn patient_binding_and_support_permutations_preserve_full_results() {
        let mut cfg = config(
            vec![extended_patient("A", 1, 1), extended_patient("B", 1, 1)],
            40,
        );
        add_extended_resources(&mut cfg.capacity_location);
        for patient in &mut cfg.patients {
            patient.stages[2].duration_ticks = 0;
        }
        let mut bindings = cfg
            .patients
            .iter()
            .flat_map(|patient| bindings_for(patient, distribution_service()))
            .collect::<Vec<_>>();
        let ordered = run_pathway_with_services(&cfg, 11, &bindings).unwrap();
        cfg.patients.reverse();
        bindings.reverse();
        for binding in &mut bindings {
            if let ServiceLaw::Discrete { outcomes } = &mut binding.service.law {
                outcomes.reverse();
            }
        }
        let permuted = run_pathway_with_services(&cfg, 11, &bindings).unwrap();
        assert_eq!(ordered, permuted);
    }

    #[test]
    fn whole_path_sampling_reaches_all_support_buckets_from_independent_seeds() {
        let cases = [
            (
                12,
                16_912_471_765_267_108_955,
                14_613_286_938_028_199_468,
                14_598_664_838_165_031_071,
                2,
            ),
            (
                7,
                15_878_180_769_287_758_571,
                12_128_009_160_619_123_083,
                3_330_814_220_145_717_748,
                5,
            ),
            (
                0,
                5_632_354_988_204_184_364,
                12_360_294_385_068_959_209,
                14_846_839_984_075_645_146,
                9,
            ),
        ];
        for (seed, domain, episode, raw, expected) in cases {
            let mut cfg = config(vec![extended_patient("p", 1, 1)], 30);
            add_extended_resources(&mut cfg.capacity_location);
            let bindings = bindings_for(&cfg.patients[0], distribution_service());
            assert_eq!(service_domain_seed(seed), domain);
            assert_eq!(service_episode_seed(seed, 0, 2).unwrap(), episode);
            let mut stream = DeterministicStream::from_seed(episode);
            assert_eq!(stream.next_u64(), raw);
            let results = run_pathway_with_services(&cfg, seed, &bindings).unwrap();
            assert_eq!(results[0].episodes[2].sampled_duration_ticks, expected);
        }
    }

    #[test]
    fn discrete_selection_is_canonical_bounded_and_exact_at_bucket_boundaries() {
        let mut outcomes = vec![
            ServiceOutcome {
                duration_ticks: 9,
                weight: 4,
            },
            ServiceOutcome {
                duration_ticks: 5,
                weight: 2,
            },
            ServiceOutcome {
                duration_ticks: 2,
                weight: 1,
            },
        ];
        outcomes.sort_by_key(|item| item.duration_ticks);
        let expected = [(7, 2), (8, 5), (2, 5), (3, 9), (6, 9), (13, 9)];
        for (raw, duration) in expected {
            assert_eq!(
                select_discrete_duration(&outcomes, 7, &mut || raw).unwrap(),
                duration
            );
        }
        let mut draws = vec![0, 8].into_iter();
        let mut calls = 0;
        let sampled = select_discrete_duration(&outcomes, 7, &mut || {
            calls += 1;
            draws.next().unwrap()
        })
        .unwrap();
        assert_eq!(sampled, 5);
        assert_eq!(calls, 2);
        let mut no_draws = 0;
        assert_eq!(
            select_discrete_duration(
                &[ServiceOutcome {
                    duration_ticks: 7,
                    weight: 1
                }],
                1,
                &mut || {
                    no_draws += 1;
                    999
                }
            )
            .unwrap(),
            7
        );
        assert_eq!(no_draws, 0);
        let mut fixed_draws = 0;
        assert_eq!(
            sample_service_duration(&fixed_service(4), &mut || {
                fixed_draws += 1;
                0
            })
            .unwrap(),
            4
        );
        assert_eq!(fixed_draws, 0);
        let mut rejected_draws = 0;
        assert!(matches!(
            select_discrete_duration(&outcomes, 7, &mut || {
                rejected_draws += 1;
                0
            }),
            Err(PathwayError::Runtime(_))
        ));
        assert_eq!(rejected_draws, 128);
    }

    #[test]
    fn service_bindings_distributions_and_stage_bounds_reject_invalid_inputs() {
        let mut cfg = config(vec![extended_patient("p", 1, 1)], 30);
        add_extended_resources(&mut cfg.capacity_location);
        let valid = bindings_for(&cfg.patients[0], distribution_service());
        let mut invalid = valid.clone();
        invalid[2].service.version = 2;
        assert!(matches!(
            run_pathway_with_services(&cfg, 1, &invalid),
            Err(PathwayError::InvalidInput(_))
        ));
        let mut invalid = valid.clone();
        invalid[2].service.provenance.class = "empirical".into();
        assert!(matches!(
            run_pathway_with_services(&cfg, 1, &invalid),
            Err(PathwayError::InvalidInput(_))
        ));
        let mut invalid = valid.clone();
        invalid[2].service.law = ServiceLaw::Discrete { outcomes: vec![] };
        assert!(
            matches!(run_pathway_with_services(&cfg, 1, &invalid), Err(PathwayError::InvalidInput(message)) if message.contains("1..64"))
        );
        let mut invalid = valid.clone();
        invalid[2].service.law = ServiceLaw::Discrete {
            outcomes: (1..=65)
                .map(|duration_ticks| ServiceOutcome {
                    duration_ticks,
                    weight: 1,
                })
                .collect(),
        };
        assert!(
            matches!(run_pathway_with_services(&cfg, 1, &invalid), Err(PathwayError::InvalidInput(message)) if message.contains("1..64"))
        );
        let mut invalid = valid.clone();
        invalid[2].patient_id = "unknown".into();
        assert!(
            matches!(run_pathway_with_services(&cfg, 1, &invalid), Err(PathwayError::InvalidInput(message)) if message.contains("unknown patient"))
        );
        let mut invalid = valid.clone();
        invalid[2].stage_index = 99;
        assert!(
            matches!(run_pathway_with_services(&cfg, 1, &invalid), Err(PathwayError::InvalidInput(message)) if message.contains("out of range"))
        );
        let mut invalid = valid.clone();
        invalid[2] = invalid[1].clone();
        assert!(
            matches!(run_pathway_with_services(&cfg, 1, &invalid), Err(PathwayError::InvalidInput(message)) if message.contains("duplicate"))
        );
        let mut missing_and_extra = valid.clone();
        missing_and_extra.pop();
        missing_and_extra.push(ServiceBinding {
            patient_id: "unknown".into(),
            stage_index: 0,
            service: fixed_service(2),
        });
        assert!(matches!(
            run_pathway_with_services(&cfg, 1, &missing_and_extra),
            Err(PathwayError::InvalidInput(_))
        ));
        let mut invalid = valid.clone();
        invalid.pop();
        assert!(matches!(
            run_pathway_with_services(&cfg, 1, &invalid),
            Err(PathwayError::InvalidInput(_))
        ));
        let mut invalid = valid.clone();
        invalid[2].stage_index = 3;
        assert!(matches!(
            run_pathway_with_services(&cfg, 1, &invalid),
            Err(PathwayError::InvalidInput(_))
        ));
        let mut invalid = valid.clone();
        invalid[2].service.law = ServiceLaw::Fixed { duration_ticks: 4 };
        assert!(matches!(
            run_pathway_with_services(&cfg, 1, &invalid),
            Err(PathwayError::InvalidInput(_))
        ));
        let mut invalid = valid.clone();
        invalid[2].service.law = ServiceLaw::Discrete {
            outcomes: vec![ServiceOutcome {
                duration_ticks: 0,
                weight: 1,
            }],
        };
        assert!(matches!(
            run_pathway_with_services(&cfg, 1, &invalid),
            Err(PathwayError::InvalidInput(_))
        ));
        let mut invalid = valid.clone();
        invalid[2].service.law = ServiceLaw::Discrete {
            outcomes: vec![ServiceOutcome {
                duration_ticks: 2,
                weight: 0,
            }],
        };
        assert!(matches!(
            run_pathway_with_services(&cfg, 1, &invalid),
            Err(PathwayError::InvalidInput(_))
        ));
        let mut invalid = valid.clone();
        invalid[2].service.law = ServiceLaw::Discrete {
            outcomes: vec![
                ServiceOutcome {
                    duration_ticks: 2,
                    weight: 1,
                },
                ServiceOutcome {
                    duration_ticks: 2,
                    weight: 1,
                },
            ],
        };
        assert!(matches!(
            run_pathway_with_services(&cfg, 1, &invalid),
            Err(PathwayError::InvalidInput(_))
        ));
        let mut invalid = valid.clone();
        invalid[2].service.law = ServiceLaw::Discrete {
            outcomes: vec![
                ServiceOutcome {
                    duration_ticks: 2,
                    weight: u64::MAX,
                },
                ServiceOutcome {
                    duration_ticks: 3,
                    weight: 1,
                },
            ],
        };
        assert!(matches!(
            run_pathway_with_services(&cfg, 1, &invalid),
            Err(PathwayError::InvalidInput(_))
        ));
        let mut invalid = valid.clone();
        invalid[2].service.provenance.notes = " no trim".into();
        assert!(matches!(
            run_pathway_with_services(&cfg, 1, &invalid),
            Err(PathwayError::InvalidInput(_))
        ));
        cfg.patients[0].stages[0].duration_ticks = 0;
        assert!(matches!(
            run_pathway(&cfg),
            Err(PathwayError::InvalidInput(_))
        ));

        let mut sixteen = config(vec![extended_patient("p", 1, 16)], 200);
        add_extended_resources(&mut sixteen.capacity_location);
        let accepted = bindings_for(&sixteen.patients[0], distribution_service());
        assert_eq!(validate_config(&sixteen, true), Ok(()));
        assert_eq!(
            run_pathway_with_services(&sixteen, 1, &accepted).unwrap()[0]
                .episodes
                .len(),
            19
        );
        let mut seventeen = config(vec![extended_patient("p", 1, 17)], 100);
        add_extended_resources(&mut seventeen.capacity_location);
        assert!(matches!(
            validate_config(&seventeen, true),
            Err(PathwayError::InvalidInput(_))
        ));
    }

    #[test]
    fn queued_at_horizon_episode_is_sampled_once_and_capacity_delay_does_not_change_samples() {
        let mut cfg = config(
            vec![extended_patient("A", 1, 1), extended_patient("B", 1, 1)],
            4,
        );
        for item in &mut cfg.patients {
            item.stages[0].duration_ticks = 1;
            item.stages[1].duration_ticks = 1;
        }
        add_extended_resources(&mut cfg.capacity_location);
        let mut bindings = cfg
            .patients
            .iter()
            .flat_map(|patient| bindings_for(patient, distribution_service()))
            .collect::<Vec<_>>();
        let results = run_pathway_with_services(&cfg, 11, &bindings).unwrap();
        let queued = results
            .iter()
            .find(|result| result.patient_id == "B")
            .unwrap();
        let diag = queued
            .episodes
            .iter()
            .find(|episode| episode.kind == StageKind::Diagnostics)
            .unwrap();
        assert_eq!(diag.queue_entry_tick, 4);
        assert_eq!(diag.start_tick, None);
        assert_eq!(diag.observed_work_ticks, 0);
        assert!([2, 5, 9].contains(&diag.sampled_duration_ticks));
        assert!(matches!(
            queued.state,
            PathwayState::Queued(StageKind::Diagnostics)
        ));

        cfg.horizon_ticks = 50;
        cfg.patients = vec![patient("A", 1, 1), patient("B", 1, 2)];
        let mut common = cfg.patients[0].stages.clone();
        common[0] = stage(StageKind::Triage, "resource.synthetic", 0);
        for item in &mut cfg.patients {
            item.stages = common.clone();
        }
        bindings.clear();
        for item in &cfg.patients {
            bindings.push(ServiceBinding {
                patient_id: item.patient_id.clone(),
                stage_index: 0,
                service: distribution_service(),
            });
            bindings.push(ServiceBinding {
                patient_id: item.patient_id.clone(),
                stage_index: 1,
                service: fixed_service(3),
            });
        }
        cfg.capacity_location.resource_buckets[0].open_count = Some(1);
        cfg.capacity_location.resource_buckets[0].staffed_count = Some(1);
        let delayed = run_pathway_with_services(&cfg, 12, &bindings).unwrap();
        cfg.capacity_location.resource_buckets[0].open_count = Some(2);
        cfg.capacity_location.resource_buckets[0].staffed_count = Some(2);
        cfg.capacity_location.resource_buckets[0].physical_count = Some(2);
        let parallel = run_pathway_with_services(&cfg, 12, &bindings).unwrap();
        for (left, right) in delayed.iter().zip(&parallel) {
            assert_eq!(
                left.episodes[0].sampled_duration_ticks,
                right.episodes[0].sampled_duration_ticks
            );
        }
        assert!(delayed[1].episodes[0].start_tick > parallel[1].episodes[0].start_tick);
    }

    #[test]
    fn second_diagnostic_repeat_is_censored_when_active_or_queued_at_horizon() {
        let configure = |patients: Vec<PathwayPatient>, horizon| {
            let mut cfg = config(patients, horizon);
            add_extended_resources(&mut cfg.capacity_location);
            for resource in &mut cfg.capacity_location.resource_buckets {
                match resource.resource_id.as_str() {
                    "resource.synthetic" | "resource.assessment" => {
                        resource.open_count = Some(2);
                        resource.staffed_count = Some(2);
                        resource.physical_count = Some(2);
                    }
                    "resource.diagnostics" => {
                        resource.open_count = Some(1);
                        resource.staffed_count = Some(1);
                        resource.physical_count = Some(1);
                    }
                    _ => {}
                }
            }
            for patient in &mut cfg.patients {
                patient.stages[0].duration_ticks = 1;
                patient.stages[1].duration_ticks = 1;
                patient.stages[2].duration_ticks = 2;
                patient.stages[3].duration_ticks = 2;
            }
            cfg
        };
        let make_bindings = |cfg: &PathwayConfig| {
            cfg.patients
                .iter()
                .flat_map(|patient| bindings_for(patient, fixed_service(2)))
                .collect::<Vec<_>>()
        };

        let queued_cfg = configure(
            vec![extended_patient("A", 1, 2), extended_patient("B", 1, 2)],
            5,
        );
        let queued_results =
            run_pathway_with_services(&queued_cfg, 0, &make_bindings(&queued_cfg)).unwrap();
        let queued_a = &queued_results[0];
        let a_repeats = queued_a
            .episodes
            .iter()
            .filter(|episode| episode.kind == StageKind::Diagnostics)
            .collect::<Vec<_>>();
        assert_eq!(a_repeats.len(), 2);
        assert_eq!(a_repeats[0].episode_index, 0);
        assert_eq!(a_repeats[0].task_id, "A:diagnostics:0");
        assert_eq!(a_repeats[0].queue_entry_tick, 3);
        assert_eq!(a_repeats[0].start_tick, Some(3));
        assert_eq!(a_repeats[0].completion_tick, Some(5));
        assert_eq!(a_repeats[0].sampled_duration_ticks, 2);
        assert_eq!(a_repeats[0].observed_work_ticks, 2);
        assert_eq!(a_repeats[1].episode_index, 1);
        assert_eq!(a_repeats[1].task_id, "A:diagnostics:1");
        assert_eq!(a_repeats[1].queue_entry_tick, 5);
        assert_eq!(a_repeats[1].start_tick, None);
        assert_eq!(a_repeats[1].completion_tick, None);
        assert_eq!(a_repeats[1].sampled_duration_ticks, 2);
        assert_eq!(a_repeats[1].observed_work_ticks, 0);
        assert_eq!(queued_a.episodes.len(), 4);
        assert_eq!(queued_a.state, PathwayState::Queued(StageKind::Diagnostics));
        assert!(!queued_a
            .episodes
            .iter()
            .any(|episode| episode.kind == StageKind::Treatment));

        let queued_b = &queued_results[1];
        let b_first = queued_b
            .episodes
            .iter()
            .find(|episode| episode.kind == StageKind::Diagnostics)
            .unwrap();
        assert_eq!(b_first.episode_index, 0);
        assert_eq!(b_first.task_id, "B:diagnostics:0");
        assert_eq!(b_first.queue_entry_tick, 3);
        assert_eq!(b_first.start_tick, Some(5));
        assert_eq!(b_first.completion_tick, None);
        assert_eq!(b_first.sampled_duration_ticks, 2);
        assert_eq!(b_first.observed_work_ticks, 0);

        let active_cfg = configure(vec![extended_patient("A", 1, 2)], 6);
        let active_results =
            run_pathway_with_services(&active_cfg, 0, &make_bindings(&active_cfg)).unwrap();
        let active_a = &active_results[0];
        let active_repeats = active_a
            .episodes
            .iter()
            .filter(|episode| episode.kind == StageKind::Diagnostics)
            .collect::<Vec<_>>();
        assert_eq!(active_repeats.len(), 2);
        assert_eq!(active_repeats[0].task_id, "A:diagnostics:0");
        assert_eq!(active_repeats[0].episode_index, 0);
        assert_eq!(active_repeats[0].queue_entry_tick, 3);
        assert_eq!(active_repeats[0].start_tick, Some(3));
        assert_eq!(active_repeats[0].completion_tick, Some(5));
        assert_eq!(active_repeats[0].sampled_duration_ticks, 2);
        assert_eq!(active_repeats[0].observed_work_ticks, 2);
        assert_eq!(active_repeats[1].task_id, "A:diagnostics:1");
        assert_eq!(active_repeats[1].episode_index, 1);
        assert_eq!(active_repeats[1].queue_entry_tick, 5);
        assert_eq!(active_repeats[1].start_tick, Some(5));
        assert_eq!(active_repeats[1].completion_tick, None);
        assert_eq!(active_repeats[1].sampled_duration_ticks, 2);
        assert_eq!(active_repeats[1].observed_work_ticks, 1);
        assert_eq!(active_a.episodes.len(), 4);
        assert_eq!(active_a.state, PathwayState::Active(StageKind::Diagnostics));
        assert!(!active_a
            .episodes
            .iter()
            .any(|episode| episode.kind == StageKind::Treatment));
    }
}
