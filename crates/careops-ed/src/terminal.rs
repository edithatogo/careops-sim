//! Experimental shared clinical and terminal ED runtime.

use std::collections::{BTreeMap, HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::disposition::{
    DecisionObservation, DecisionPolicy, DispositionBinding, DispositionIntent,
};
use crate::offers::{BedOffer, OfferAllocation, OfferObservation, OfferPolicy};
use crate::pathway::{
    PathwayConfig, PathwayError, PathwayResult, ServiceBinding, ServiceProvenance, StageKind,
};

const MAX_PATIENTS: usize = 100_000;
const MAX_OFFERS: usize = 100_000;
const MAX_STAGES_PER_PATIENT: usize = 19;
const MAX_SERVICE_BINDINGS: usize = MAX_PATIENTS * MAX_STAGES_PER_PATIENT;
const MAX_CARE_EPISODES: u128 = 1_000_000;
const MAX_SCHEDULED_EVENTS: u128 = 4_400_000;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerminalConfig {
    pub version: u32,
    pub pathway: PathwayConfig,
    pub seed: u64,
    pub services: Vec<ServiceBinding>,
    pub decision_policy: DecisionPolicy,
    pub decisions: Vec<DispositionBinding>,
    pub admissions: Vec<AdmissionDestination>,
    pub offer_policy: OfferPolicy,
    pub offers: Vec<BedOffer>,
    pub policy: TerminalPolicy,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdmissionDestination {
    pub patient_id: String,
    pub destination: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerminalPolicy {
    pub version: u32,
    pub provenance: ServiceProvenance,
    pub ed_space_resource_id: String,
    pub transfer_resource_id: String,
    pub transfer_task_class_id: String,
    pub discharge_delay_ticks: u64,
    pub abandonment: AbandonmentPolicy,
    pub boarder_care: Option<BoarderCarePolicy>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AbandonmentPolicy {
    Disabled,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoarderCarePolicy {
    pub resource_id: String,
    pub task_class_id: String,
    pub interval_ticks: u64,
    pub duration_ticks: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TerminalRunResult {
    pub version: u32,
    pub patients: Vec<TerminalPatientResult>,
    pub offers: Vec<OfferObservation>,
    pub counts: TerminalCounts,
    pub resources: Vec<ResourceUsage>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TerminalPatientResult {
    pub pathway: PathwayResult,
    pub decision: Option<DecisionObservation>,
    pub space: SpaceEpisode,
    pub boarding: Option<ObservedInterval>,
    pub allocation: Option<OfferAllocation>,
    pub pending_transfer: Option<ObservedInterval>,
    pub transfer: Option<TerminalWorkEpisode>,
    pub care: Vec<TerminalWorkEpisode>,
    pub endpoint_state: EndpointState,
    pub unfinished_reason: Option<UnfinishedReason>,
    pub realized_outcome: Option<RealizedOutcome>,
    pub terminal_tick: Option<u64>,
    pub completed_los_ticks: Option<u64>,
    pub observed_elapsed_ticks: u64,
    pub boarding_time_ticks: Option<u64>,
    pub observed_boarding_ticks: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SpaceEpisode {
    pub resource_id: String,
    pub queue_entry_tick: u64,
    pub acquired_tick: Option<u64>,
    pub released_tick: Option<u64>,
    pub observed_occupancy_ticks: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ObservedInterval {
    pub start_tick: u64,
    pub end_tick: Option<u64>,
    pub observed_ticks: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TerminalWorkEpisode {
    pub task_id: String,
    pub resource_id: String,
    pub task_class_id: String,
    pub queue_entry_tick: u64,
    pub start_tick: Option<u64>,
    pub completion_tick: Option<u64>,
    pub cancelled_tick: Option<u64>,
    pub required_work_ticks: u64,
    pub observed_work_ticks: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EndpointState {
    WaitingForSpace,
    Clinical,
    Boarding,
    PendingTransfer,
    InTransfer,
    DischargePending,
    Terminal,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RealizedOutcome {
    Discharged,
    Transferred {
        destination: String,
        offer_id: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnfinishedReason {
    WaitingForSpace,
    ClinicalQueued { stage: StageKind },
    ClinicalActive { stage: StageKind },
    AwaitingOfferAllocation,
    BoarderCareQueued,
    BoarderCareActive,
    TransferQueued,
    TransferActive,
    DischargePending,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TerminalCounts {
    pub arrived: u64,
    pub discharged: u64,
    pub transferred: u64,
    pub unfinished: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ResourceUsage {
    pub resource_id: String,
    pub capacity: u64,
    pub busy_ticks: u128,
    pub capacity_ticks: u128,
    pub busy_at_horizon: u64,
    pub queued_at_horizon: u64,
    pub peak_busy: u64,
}

/// Run the live terminal pathway. Clinical, care, space and transfer work share
/// the same event scheduler and resource accounting.
pub fn run_terminal(config: &TerminalConfig) -> Result<TerminalRunResult, PathwayError> {
    validate(config)?;
    crate::pathway::run_terminal_live(config)
}

fn validate(config: &TerminalConfig) -> Result<(), PathwayError> {
    let invalid = |message: &str| Err(PathwayError::InvalidInput(message.to_owned()));
    if config.version != 1 || config.policy.version != 1 {
        return invalid("terminal config and policy versions must be 1");
    }
    if config.pathway.patients.len() > MAX_PATIENTS
        || config.offers.len() > MAX_OFFERS
        || config.decisions.len() > MAX_PATIENTS
        || config.admissions.len() > MAX_PATIENTS
        || config.services.len() > MAX_SERVICE_BINDINGS
    {
        return invalid("terminal patient or offer limit exceeded");
    }
    if config.decisions.len() != config.pathway.patients.len() {
        return invalid("exactly one decision is required per patient");
    }
    if config.policy.provenance.class != "synthetic"
        || config.policy.provenance.notes.trim().is_empty()
        || config.policy.provenance.notes.trim() != config.policy.provenance.notes
    {
        return invalid("terminal policy requires trimmed synthetic provenance");
    }
    if config.decision_policy.version != 1
        || config.decision_policy.provenance.class != "synthetic"
        || config.decision_policy.provenance.notes.trim().is_empty()
        || config.decision_policy.provenance.notes.trim() != config.decision_policy.provenance.notes
    {
        return invalid("decision policy requires version 1 and trimmed synthetic provenance");
    }
    let mut patient_ids = HashSet::with_capacity(config.pathway.patients.len());
    let mut intent_map = HashMap::with_capacity(config.decisions.len());
    for patient in &config.pathway.patients {
        patient_ids.insert(patient.patient_id.as_str());
        let stages = &patient.stages;
        if stages.len() < 3
            || stages.len() > MAX_STAGES_PER_PATIENT
            || stages.first().map(|stage| stage.kind) != Some(StageKind::Triage)
            || stages.get(1).map(|stage| stage.kind) != Some(StageKind::Assessment)
            || stages.last().map(|stage| stage.kind) != Some(StageKind::Treatment)
            || stages[2..stages.len() - 1]
                .iter()
                .any(|stage| stage.kind != StageKind::Diagnostics)
        {
            return invalid("terminal runtime requires treatment-ending clinical paths");
        }
    }
    for decision in &config.decisions {
        if !patient_ids.contains(decision.patient_id.as_str())
            || intent_map
                .insert(decision.patient_id.as_str(), decision.latent_intent)
                .is_some()
        {
            return invalid("decision bindings must be unique and name configured patients");
        }
    }
    let mut destinations = BTreeMap::new();
    for admission in &config.admissions {
        if !patient_ids.contains(admission.patient_id.as_str())
            || !valid_id(&admission.destination)
            || destinations
                .insert(
                    admission.patient_id.as_str(),
                    admission.destination.as_str(),
                )
                .is_some()
        {
            return invalid("admission destinations must be unique and valid");
        }
    }
    for (patient_id, intent) in &intent_map {
        match intent {
            DispositionIntent::Admission if !destinations.contains_key(patient_id) => {
                return invalid("admission decision requires an explicit destination");
            }
            DispositionIntent::Discharge if destinations.contains_key(patient_id) => {
                return invalid("discharge decision cannot have an admission destination");
            }
            _ => {}
        }
    }
    if destinations.len() != intent_map.len() {
        for (patient_id, intent) in &intent_map {
            if *intent == DispositionIntent::Admission && !destinations.contains_key(patient_id) {
                return invalid("admission destination is missing");
            }
        }
        if destinations
            .keys()
            .any(|patient_id| intent_map.get(patient_id) != Some(&DispositionIntent::Admission))
        {
            return invalid("admission destination is extra");
        }
    }
    if !valid_id(&config.policy.ed_space_resource_id)
        || !valid_id(&config.policy.transfer_resource_id)
        || !valid_id(&config.policy.transfer_task_class_id)
        || config.policy.ed_space_resource_id == config.policy.transfer_resource_id
    {
        return invalid("terminal resource identifiers are invalid or aliased");
    }
    let care = config.policy.boarder_care.as_ref();
    if let Some(care) = care {
        if !valid_id(&care.resource_id)
            || !valid_id(&care.task_class_id)
            || care.interval_ticks == 0
            || care.duration_ticks == 0
        {
            return invalid("boarder care requires valid resources and positive interval/duration");
        }
    }
    let profile = &config.pathway.capacity_location;
    let space = profile
        .resource_buckets
        .iter()
        .find(|resource| resource.resource_id == config.policy.ed_space_resource_id)
        .ok_or_else(|| PathwayError::InvalidInput("unknown ED space resource".into()))?;
    let transfer = profile
        .resource_buckets
        .iter()
        .find(|resource| resource.resource_id == config.policy.transfer_resource_id)
        .ok_or_else(|| PathwayError::InvalidInput("unknown transfer resource".into()))?;
    let capacity = |resource: &crate::ResourceBucket| {
        resource
            .open_count
            .unwrap_or(0)
            .min(resource.staffed_count.unwrap_or(0))
    };
    if capacity(space) == 0 || capacity(transfer) == 0 {
        return invalid("terminal space and transfer resources require positive capacity");
    }
    let clinical_resources: HashSet<&str> = config
        .pathway
        .patients
        .iter()
        .flat_map(|patient| {
            patient
                .stages
                .iter()
                .map(|stage| stage.resource_id.as_str())
        })
        .collect();
    if clinical_resources.contains(config.policy.ed_space_resource_id.as_str())
        || clinical_resources.contains(config.policy.transfer_resource_id.as_str())
        || care.is_some_and(|care| {
            care.resource_id == config.policy.ed_space_resource_id
                || care.resource_id == config.policy.transfer_resource_id
        })
    {
        return invalid("space and transfer resources must be separate from work pools");
    }
    let transfer_task = profile
        .task_classes
        .iter()
        .find(|task| task.task_class_id == config.policy.transfer_task_class_id)
        .ok_or_else(|| PathwayError::InvalidInput("unknown transfer task class".into()))?;
    if !transfer_task.required_resource_classes.is_empty()
        && !transfer_task
            .required_resource_classes
            .contains(&transfer.resource_class)
    {
        return invalid("transfer task class is incompatible with transfer resource");
    }
    if !has_task_eligibility(profile, transfer, &config.policy.transfer_task_class_id) {
        return invalid("transfer task has no matching role/zone/location eligibility");
    }
    if let Some(care) = care {
        let resource = profile
            .resource_buckets
            .iter()
            .find(|resource| resource.resource_id == care.resource_id)
            .ok_or_else(|| PathwayError::InvalidInput("unknown boarder care resource".into()))?;
        let task = profile
            .task_classes
            .iter()
            .find(|task| task.task_class_id == care.task_class_id)
            .ok_or_else(|| PathwayError::InvalidInput("unknown boarder care task class".into()))?;
        if !task.required_resource_classes.is_empty()
            && !task
                .required_resource_classes
                .contains(&resource.resource_class)
        {
            return invalid("care task class is incompatible with care resource");
        }
        if !has_task_eligibility(profile, resource, &care.task_class_id) {
            return invalid("care task has no matching role/zone/location eligibility");
        }
    }
    let care_due_bound = if let Some(care) = care {
        intent_map
            .values()
            .filter(|intent| **intent == DispositionIntent::Admission)
            .try_fold(0u128, |sum, _| {
                sum.checked_add(u128::from(
                    config.pathway.horizon_ticks / care.interval_ticks,
                ))
            })
            .ok_or_else(|| PathwayError::InvalidInput("care event bound overflow".into()))?
    } else {
        0
    };
    if care_due_bound > MAX_CARE_EPISODES {
        return invalid("boarder care episode limit exceeded");
    }
    let clinical_count = config
        .pathway
        .patients
        .iter()
        .try_fold(0u128, |sum, patient| {
            sum.checked_add(patient.stages.len() as u128)
        })
        .ok_or_else(|| PathwayError::InvalidInput("clinical event bound overflow".into()))?;
    let admission_count = intent_map
        .values()
        .filter(|intent| **intent == DispositionIntent::Admission)
        .count() as u128;
    let discharge_count = intent_map.len() as u128 - admission_count;
    let event_bound = (config.pathway.patients.len() as u128)
        .checked_add(clinical_count)
        .and_then(|n| n.checked_add(care_due_bound.checked_mul(2)?))
        .and_then(|n| n.checked_add(admission_count))
        .and_then(|n| n.checked_add(discharge_count))
        .and_then(|n| n.checked_add((config.offers.len() as u128).checked_mul(2)?))
        .ok_or_else(|| {
            PathwayError::InvalidInput("terminal scheduler event bound overflow".into())
        })?;
    if event_bound > MAX_SCHEDULED_EVENTS {
        return invalid("terminal scheduler event limit exceeded");
    }
    for offer in &config.offers {
        if offer.patient_id.as_ref().is_some_and(|patient_id| {
            !intent_map.contains_key(patient_id.as_str())
                || intent_map.get(patient_id.as_str()) != Some(&DispositionIntent::Admission)
        }) {
            return invalid("restricted offer must name an admission patient");
        }
    }
    crate::offers::OfferBook::new(&config.offer_policy, &config.offers)?;
    Ok(())
}

fn has_task_eligibility(
    profile: &crate::CapacityLocationProfile,
    resource: &crate::ResourceBucket,
    task_class_id: &str,
) -> bool {
    profile.eligibility.iter().any(|entry| {
        entry.task_class_id == task_class_id
            && entry.zone_id == resource.zone_id
            && entry
                .location_id
                .as_ref()
                .is_none_or(|location| location == &resource.location_id)
    })
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-'))
}
