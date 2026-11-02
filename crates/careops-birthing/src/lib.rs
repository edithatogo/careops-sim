//! Domain-owned birthing pathway state and progression runtime.
//!
//! Time values are discrete simulation ticks. This crate defines mechanics only;
//! it supplies no clinical rates, thresholds, targets, or site defaults.

use careops_core::pathway::{
    CompetingRiskModel, ObservedPolicyState, PathwayError, PathwayErrorKind, PathwayRngStreams,
    ScheduledEvent, VersionedUrgencyAssessment,
};
use std::collections::{HashMap, HashSet};

pub mod delivery;
pub mod spaces;
pub mod team;
pub mod theatre;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PatientState {
    NotInLabour,
    LatentLabour,
    ActiveLabour,
    LaterLabourStage,
    InductionStarted,
    SlowProgressionObserved,
    DeteriorationObserved,
    OperativeDeliveryPlanned,
    EscalationPlanned,
    DeliveryComplete,
}

impl PatientState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotInLabour => "not_in_labour",
            Self::LatentLabour => "latent_labour",
            Self::ActiveLabour => "active_labour",
            Self::LaterLabourStage => "later_labour_stage",
            Self::InductionStarted => "induction_started",
            Self::SlowProgressionObserved => "slow_progression_observed",
            Self::DeteriorationObserved => "deterioration_observed",
            Self::OperativeDeliveryPlanned => "operative_delivery_planned",
            Self::EscalationPlanned => "escalation_planned",
            Self::DeliveryComplete => "delivery_complete",
        }
    }
}

impl std::str::FromStr for PatientState {
    type Err = PathwayError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "not_in_labour" => Ok(Self::NotInLabour),
            "latent_labour" => Ok(Self::LatentLabour),
            "active_labour" => Ok(Self::ActiveLabour),
            "later_labour_stage" => Ok(Self::LaterLabourStage),
            "induction_started" => Ok(Self::InductionStarted),
            "slow_progression_observed" => Ok(Self::SlowProgressionObserved),
            "deterioration_observed" => Ok(Self::DeteriorationObserved),
            "operative_delivery_planned" => Ok(Self::OperativeDeliveryPlanned),
            "escalation_planned" => Ok(Self::EscalationPlanned),
            "delivery_complete" => Ok(Self::DeliveryComplete),
            _ => Err(invalid(format!("unknown patient state '{value}'"))),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceState {
    WaitingForSuite,
    SuiteReserved,
    InTransit,
    InSuite,
    WaitingForAssessment,
    WaitingForTheatre,
    InTheatre,
    WaitingForReview,
    WaitingForTransfer,
    DestinationArrived,
}

impl ServiceState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WaitingForSuite => "waiting_for_suite",
            Self::SuiteReserved => "suite_reserved",
            Self::InTransit => "in_transit",
            Self::InSuite => "in_suite",
            Self::WaitingForAssessment => "waiting_for_assessment",
            Self::WaitingForTheatre => "waiting_for_theatre",
            Self::InTheatre => "in_theatre",
            Self::WaitingForReview => "waiting_for_review",
            Self::WaitingForTransfer => "waiting_for_transfer",
            Self::DestinationArrived => "destination_arrived",
        }
    }
}

impl std::str::FromStr for ServiceState {
    type Err = PathwayError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "waiting_for_suite" => Ok(Self::WaitingForSuite),
            "suite_reserved" => Ok(Self::SuiteReserved),
            "in_transit" => Ok(Self::InTransit),
            "in_suite" => Ok(Self::InSuite),
            "waiting_for_assessment" => Ok(Self::WaitingForAssessment),
            "waiting_for_theatre" => Ok(Self::WaitingForTheatre),
            "in_theatre" => Ok(Self::InTheatre),
            "waiting_for_review" => Ok(Self::WaitingForReview),
            "waiting_for_transfer" => Ok(Self::WaitingForTransfer),
            "destination_arrived" => Ok(Self::DestinationArrived),
            _ => Err(invalid(format!("unknown service state '{value}'"))),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventCategory {
    Routine,
    Progression,
    Intervention,
    Deterioration,
    Service,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EventEffect {
    pub patient_state: Option<PatientState>,
    pub service_state: Option<ServiceState>,
    pub category: EventCategory,
}

impl EventEffect {
    pub fn combined(
        patient_state: Option<PatientState>,
        service_state: Option<ServiceState>,
        category: EventCategory,
    ) -> Self {
        Self {
            patient_state,
            service_state,
            category,
        }
    }

    pub fn progression(patient_state: PatientState) -> Self {
        Self::combined(Some(patient_state), None, EventCategory::Progression)
    }

    pub fn intervention(patient_state: PatientState, service_state: Option<ServiceState>) -> Self {
        Self::combined(
            Some(patient_state),
            service_state,
            EventCategory::Intervention,
        )
    }

    pub fn deterioration(patient_state: PatientState, service_state: Option<ServiceState>) -> Self {
        Self::combined(
            Some(patient_state),
            service_state,
            EventCategory::Deterioration,
        )
    }

    pub fn service(service_state: ServiceState) -> Self {
        Self::combined(None, Some(service_state), EventCategory::Service)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateTraceEntry {
    pub event_id: String,
    pub tick: u64,
    pub category: EventCategory,
    pub patient_state: PatientState,
    pub service_state: ServiceState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EpisodeObservation {
    pub tick: u64,
    pub patient_state: PatientState,
    pub service_state: ServiceState,
    pub censored: bool,
    pub censor_tick: Option<u64>,
}

#[derive(Clone, Debug)]
struct QueuedEvent {
    scheduled: ScheduledEvent,
    label: String,
    effect: EventEffect,
}

/// One maternal episode. It owns its named random streams and ordered event queue.
#[derive(Clone, Debug)]
pub struct BirthingEpisode {
    patient_id: u64,
    tick: u64,
    patient_state: PatientState,
    service_state: ServiceState,
    generation: u32,
    events: Vec<QueuedEvent>,
    known_event_ids: HashSet<u64>,
    trace: Vec<StateTraceEntry>,
    stale_event_ids: Vec<String>,
    rng: PathwayRngStreams,
    censor_tick: Option<u64>,
}

impl BirthingEpisode {
    pub fn new(
        patient_id: u64,
        patient_state: PatientState,
        service_state: ServiceState,
        seed: u64,
    ) -> Self {
        Self {
            patient_id,
            tick: 0,
            patient_state,
            service_state,
            generation: 0,
            events: Vec::new(),
            known_event_ids: HashSet::new(),
            trace: Vec::new(),
            stale_event_ids: Vec::new(),
            rng: PathwayRngStreams::new(seed),
            censor_tick: None,
        }
    }

    pub fn schedule(
        &mut self,
        event_id: u64,
        label: impl Into<String>,
        tick: u64,
        priority: u8,
        effect: EventEffect,
    ) -> Result<(), PathwayError> {
        if self.censor_tick.is_some() {
            return Err(invalid("cannot schedule an event after horizon censoring"));
        }
        if tick < self.tick {
            return Err(invalid("event tick precedes the current simulation tick"));
        }
        if !self.known_event_ids.insert(event_id) {
            return Err(invalid(format!("duplicate event id {event_id}")));
        }
        let label = label.into();
        self.events.push(QueuedEvent {
            scheduled: ScheduledEvent {
                event_id,
                tick,
                priority,
                patient_id: self.patient_id,
                generation: self.generation,
                description: label.clone(),
            },
            label,
            effect,
        });
        Ok(())
    }

    /// Replaces the outstanding progression schedule with a sample for one
    /// discrete synthetic simulation tick. Any queued progression is invalidated
    /// before sampling; the caller supplies the current model/covariates and
    /// cause-to-state mapping. Service and routing schedules are not redrawn.
    pub fn sample_progression_tick(
        &mut self,
        event_id: u64,
        label: impl Into<String>,
        priority: u8,
        model: &CompetingRiskModel,
        covariates: &HashMap<String, f64>,
        cause_states: &HashMap<String, PatientState>,
    ) -> Result<bool, PathwayError> {
        if self.censor_tick.is_some() {
            return Err(invalid("cannot sample progression after horizon censoring"));
        }
        if self
            .events
            .iter()
            .any(|event| event.effect.category == EventCategory::Progression)
        {
            self.generation = self
                .generation
                .checked_add(1)
                .ok_or_else(|| invalid("patient event generation overflow"))?;
        }
        let Some(cause) = model.sample_event(covariates, &mut self.rng.progression_stream)? else {
            return Ok(false);
        };
        let next_state = cause_states.get(&cause).copied().ok_or_else(|| {
            PathwayError::new(
                PathwayErrorKind::InvalidCovariate,
                format!("no patient-state mapping for progression cause '{cause}'"),
            )
        })?;
        let tick = self
            .tick
            .checked_add(1)
            .ok_or_else(|| invalid("simulation tick overflow"))?;
        self.schedule(
            event_id,
            label,
            tick,
            priority,
            EventEffect::progression(next_state),
        )?;
        Ok(true)
    }

    /// Executes all events through `inclusive_tick` in the core deterministic order.
    pub fn advance_through(&mut self, inclusive_tick: u64) -> Result<(), PathwayError> {
        if self.censor_tick.is_some() {
            return Err(invalid("episode is already censored at its horizon"));
        }
        if inclusive_tick < self.tick {
            return Err(invalid("cannot move simulation time backwards"));
        }
        loop {
            let next_index = self
                .events
                .iter()
                .enumerate()
                .min_by(|(_, left), (_, right)| left.scheduled.cmp(&right.scheduled))
                .map(|(index, _)| index);
            let Some(index) = next_index else { break };
            if self.events[index].scheduled.tick > inclusive_tick {
                break;
            }
            let event = self.events.remove(index);
            self.tick = event.scheduled.tick;
            if event.effect.category == EventCategory::Progression
                && event.scheduled.validate_execution(self.generation).is_err()
            {
                self.stale_event_ids.push(event.label);
                continue;
            }

            if let Some(next_patient_state) = event.effect.patient_state {
                if next_patient_state != self.patient_state {
                    self.generation = self
                        .generation
                        .checked_add(1)
                        .ok_or_else(|| invalid("patient event generation overflow"))?;
                    self.patient_state = next_patient_state;
                }
            }
            if let Some(next_service_state) = event.effect.service_state {
                self.service_state = next_service_state;
            }
            self.trace.push(StateTraceEntry {
                event_id: event.label,
                tick: event.scheduled.tick,
                category: event.effect.category,
                patient_state: self.patient_state,
                service_state: self.service_state,
            });
        }
        self.tick = inclusive_tick;
        Ok(())
    }

    /// Executes through the inclusive horizon and right-censors unfinished episodes.
    pub fn censor_at(&mut self, horizon: u64) -> Result<EpisodeObservation, PathwayError> {
        self.advance_through(horizon)?;
        if self.patient_state != PatientState::DeliveryComplete {
            self.censor_tick = Some(horizon);
        }
        Ok(self.observation())
    }

    /// Builds the core policy input from observed time and caller-supplied
    /// versioned assessment data. Patient/service latent state and queued events
    /// are intentionally not part of `ObservedPolicyState`.
    pub fn policy_input(
        &self,
        urgency: VersionedUrgencyAssessment,
        elapsed_wait_ticks: u64,
    ) -> Result<ObservedPolicyState, PathwayError> {
        if urgency.assessed_at_tick > self.tick {
            return Err(invalid(
                "urgency assessment is later than the observed episode tick",
            ));
        }
        Ok(ObservedPolicyState {
            observed_at_tick: self.tick,
            urgency,
            elapsed_wait_ticks,
        })
    }

    pub fn observation(&self) -> EpisodeObservation {
        EpisodeObservation {
            tick: self.tick,
            patient_state: self.patient_state,
            service_state: self.service_state,
            censored: self.censor_tick.is_some(),
            censor_tick: self.censor_tick,
        }
    }

    pub fn trace(&self) -> &[StateTraceEntry] {
        &self.trace
    }

    pub fn stale_event_ids(&self) -> &[String] {
        &self.stale_event_ids
    }

    pub fn generation(&self) -> u32 {
        self.generation
    }

    pub fn draw_counts(&self) -> (usize, usize, usize) {
        (
            self.rng.progression_stream.draw_count,
            self.rng.service_stream.draw_count,
            self.rng.routing_stream.draw_count,
        )
    }
}

fn invalid(message: impl Into<String>) -> PathwayError {
    PathwayError::new(PathwayErrorKind::InvalidTransition, message)
}
