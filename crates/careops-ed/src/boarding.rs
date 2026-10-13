//! Synthetic boarding, discharge, and optional abandonment/transfer exits.
//!
//! Bed offers are finite patient-addressed boundary records. Accepting one does
//! not release ED occupancy before physical departure, and nothing in this
//! module infers a ward policy or a community-ambulance effect. A disabled
//! abandonment or facility-transfer policy cannot create that terminal reason.
//! Horizon truncation remains unfinished.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use serde::{Deserialize, Serialize};

use crate::disposition::DispositionIntent;
use crate::pathway::{PathwayError, PathwayPatient, PathwayResult, StageKind};

const MAX_PATIENTS: usize = 100_000;
const MAX_OFFERS: usize = 100_000;
const MAX_CARE_TASKS: usize = 10_000;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BedOffer {
    pub offer_id: String,
    pub patient_id: String,
    pub destination: String,
    pub available_tick: u64,
    pub withdrawn_tick: u64,
    pub transfer_duration_ticks: u64,
}

#[derive(Clone, Debug, Default)]
pub struct ExitOptions {
    pub abandonment: Option<AbandonmentPolicy>,
    pub transfer: Option<FacilityTransferPolicy>,
    pub transfer_requests: BTreeMap<String, String>,
    pub care: Option<BoarderCarePolicy>,
    pub compatible_destinations: BTreeMap<String, BTreeSet<String>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbandonmentPolicy {
    pub version: u32,
    pub patience_ticks: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FacilityTransferPolicy {
    pub version: u32,
    pub transport_delay_ticks: u64,
    pub eligible_destinations: BTreeSet<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoarderCarePolicy {
    pub version: u32,
    pub interval_ticks: u64,
    pub duration_ticks: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BoardingInterval {
    pub start_tick: u64,
    pub end_tick: u64,
    pub duration_ticks: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TransferInterval {
    pub offer_id: String,
    pub start_tick: u64,
    pub end_tick: u64,
    pub duration_ticks: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RealizedOutcome {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offer_id: Option<String>,
    pub terminal_tick: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StageInterval {
    pub stage: String,
    pub resource_id: String,
    pub start_tick: u64,
    pub end_tick: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PatientLatentState {
    pub acuity: u8,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PatientAgentKnowledge {
    pub triage_acuity: Option<u8>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EndpointPatientResult {
    pub patient_id: String,
    pub arrival_tick: u64,
    pub latent_state: PatientLatentState,
    pub agent_knowledge: PatientAgentKnowledge,
    pub provisional_disposition: Option<DispositionIntent>,
    pub endpoint_state: String,
    pub terminal_tick: Option<u64>,
    pub completed_los_ticks: Option<u64>,
    pub observed_elapsed_ticks: u64,
    pub active_work_ticks: u64,
    pub first_care_wait_ticks: u64,
    pub total_stage_queue_wait_ticks: u64,
    pub stage_intervals: Vec<StageInterval>,
    pub boarding_interval: Option<BoardingInterval>,
    pub transfer_interval: Option<TransferInterval>,
    pub realized_outcome: Option<RealizedOutcome>,
    pub unfinished_reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EndpointCounts {
    pub arrivals: u64,
    pub started: u64,
    pub terminal: u64,
    pub throughput_count: u64,
    pub unfinished: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EndpointMetrics {
    pub mean_active_work: Option<f64>,
    pub mean_completed_los: Option<f64>,
    pub mean_first_care_wait: Option<f64>,
    pub mean_observed_elapsed: Option<f64>,
    pub mean_total_stage_queue_wait: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EndpointDenominators {
    pub active_work: u64,
    pub completed_los: u64,
    pub first_care_wait: u64,
    pub observed_elapsed: u64,
    pub total_stage_queue_wait: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct BoarderCareTask {
    pub task_id: String,
    pub patient_id: String,
    pub start_tick: u64,
    pub completion_tick: Option<u64>,
    pub censored: bool,
    pub reserves_staff: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct BoundaryClaims {
    pub ward_policy: bool,
    pub community_ambulance: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct EndpointExecutionResult {
    pub counts: EndpointCounts,
    pub metrics_ticks: EndpointMetrics,
    pub denominators: EndpointDenominators,
    pub patients: Vec<EndpointPatientResult>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub boarder_care: Vec<BoarderCareTask>,
    #[serde(default, skip_serializing_if = "BoundaryClaims::unclaimed")]
    pub claims: BoundaryClaims,
}

impl BoundaryClaims {
    fn unclaimed(&self) -> bool {
        !self.ward_policy && !self.community_ambulance
    }
}

/// Endpoint projection used by the frozen E1 fixture. Optional policies stay off.
pub fn compute_endpoints(
    pathways: &[PathwayResult],
    inputs: &[PathwayPatient],
    provisional_dispositions: &BTreeMap<String, DispositionIntent>,
    bed_offers: &[BedOffer],
    horizon_tick: u64,
) -> Result<EndpointExecutionResult, PathwayError> {
    compute_endpoints_with_options(
        pathways,
        inputs,
        provisional_dispositions,
        bed_offers,
        horizon_tick,
        &ExitOptions::default(),
    )
}

/// Same projection with explicit abandonment, facility transfer, destination, and care policy.
pub fn compute_endpoints_with_options(
    pathways: &[PathwayResult],
    inputs: &[PathwayPatient],
    provisional_dispositions: &BTreeMap<String, DispositionIntent>,
    bed_offers: &[BedOffer],
    horizon_tick: u64,
    options: &ExitOptions,
) -> Result<EndpointExecutionResult, PathwayError> {
    if horizon_tick == 0 {
        return invalid("horizon_ticks must be positive");
    }
    if pathways.len() > MAX_PATIENTS || inputs.len() > MAX_PATIENTS {
        return invalid("endpoint patient limit exceeded");
    }
    validate_options(options)?;
    validate_bed_offers(bed_offers)?;
    let inputs = index_inputs(inputs)?;
    let mut offers_by_patient: BTreeMap<&str, Vec<&BedOffer>> = BTreeMap::new();
    for offer in bed_offers {
        offers_by_patient
            .entry(offer.patient_id.as_str())
            .or_default()
            .push(offer);
    }
    for offers in offers_by_patient.values_mut() {
        offers.sort_by(|left, right| {
            (left.available_tick, left.offer_id.as_str())
                .cmp(&(right.available_tick, right.offer_id.as_str()))
        });
    }

    let mut patients = Vec::with_capacity(pathways.len());
    let mut care_tasks = Vec::new();
    let mut consumed = HashSet::new();
    let mut totals = Totals::default();
    let mut seen_patients = HashSet::with_capacity(pathways.len());
    for pathway in pathways {
        if !seen_patients.insert(pathway.patient_id.as_str()) {
            return invalid("duplicate pathway patient");
        }
        if pathway.arrival_tick >= horizon_tick {
            return invalid("arrival must be before the horizon");
        }
        let input = inputs.get(pathway.patient_id.as_str()).ok_or_else(|| {
            PathwayError::InvalidInput(format!(
                "pathway result missing corresponding input patient: {}",
                pathway.patient_id
            ))
        })?;
        let provisional = provisional_dispositions
            .get(pathway.patient_id.as_str())
            .copied();
        let stages = stage_account(pathway, input)?;
        let resolved = resolve_patient(
            pathway,
            provisional,
            offers_by_patient
                .get(pathway.patient_id.as_str())
                .map(Vec::as_slice)
                .unwrap_or(&[]),
            horizon_tick,
            options,
            &mut consumed,
        )?;
        if let Some(care) = &options.care {
            if resolved.occupancy_until > stages.treatment_end.unwrap_or(pathway.arrival_tick)
                && resolved.boards
            {
                let start = stages.treatment_end.unwrap_or(pathway.arrival_tick);
                care_tasks.extend(boarder_care(
                    &pathway.patient_id,
                    start,
                    resolved.occupancy_until,
                    care,
                    horizon_tick,
                )?);
            }
        }
        let observed = if let Some(terminal) = resolved.terminal_tick {
            checked_sub(terminal, pathway.arrival_tick)?
        } else {
            checked_sub(horizon_tick, pathway.arrival_tick)?
        };
        let terminal = resolved.terminal_tick.is_some();
        totals.observe(&stages, observed, resolved.completed_los, terminal);
        patients.push(EndpointPatientResult {
            patient_id: pathway.patient_id.clone(),
            arrival_tick: pathway.arrival_tick,
            latent_state: PatientLatentState {
                acuity: pathway.acuity,
            },
            agent_knowledge: PatientAgentKnowledge {
                triage_acuity: pathway.known_acuity,
            },
            provisional_disposition: provisional,
            endpoint_state: if terminal {
                "terminal".into()
            } else {
                "unfinished_at_horizon".into()
            },
            terminal_tick: resolved.terminal_tick,
            completed_los_ticks: resolved.completed_los,
            observed_elapsed_ticks: observed,
            active_work_ticks: stages.active_work,
            first_care_wait_ticks: stages.first_care_wait,
            total_stage_queue_wait_ticks: stages.queue_wait,
            stage_intervals: stages.intervals,
            boarding_interval: resolved.boarding,
            transfer_interval: resolved.transfer,
            realized_outcome: resolved.outcome,
            unfinished_reason: resolved.unfinished_reason,
        });
    }

    Ok(EndpointExecutionResult {
        counts: EndpointCounts {
            arrivals: totals.arrivals,
            started: totals.started,
            terminal: totals.terminal,
            throughput_count: totals.terminal,
            unfinished: totals.unfinished,
        },
        metrics_ticks: totals.metrics(),
        denominators: EndpointDenominators {
            active_work: totals.arrivals,
            completed_los: totals.completed_los_count,
            first_care_wait: totals.arrivals,
            observed_elapsed: totals.arrivals,
            total_stage_queue_wait: totals.arrivals,
        },
        patients,
        boarder_care: care_tasks,
        claims: BoundaryClaims::default(),
    })
}

/// Validate bed offers against bounded specifications.
pub fn validate_bed_offers(offers: &[BedOffer]) -> Result<(), PathwayError> {
    if offers.len() > MAX_OFFERS {
        return invalid("bed offer limit exceeded");
    }
    let mut seen_ids = BTreeSet::new();
    for offer in offers {
        if !valid_identifier(&offer.offer_id) {
            return invalid("invalid bed offer ID syntax");
        }
        if !valid_identifier(&offer.patient_id) {
            return invalid("invalid bed offer patient ID syntax");
        }
        if !valid_identifier(&offer.destination) {
            return invalid("invalid bed offer destination syntax");
        }
        if !seen_ids.insert(offer.offer_id.as_str()) {
            return invalid("duplicate bed offer ID");
        }
        if offer.transfer_duration_ticks == 0 {
            return invalid("bed offer transfer duration must be positive");
        }
        if offer.available_tick >= offer.withdrawn_tick {
            return invalid("bed offer available tick must precede withdrawn tick");
        }
    }
    Ok(())
}

struct StageAccount {
    intervals: Vec<StageInterval>,
    active_work: u64,
    queue_wait: u64,
    first_care_wait: u64,
    started: bool,
    treatment_end: Option<u64>,
}

struct ResolvedExit {
    boarding: Option<BoardingInterval>,
    transfer: Option<TransferInterval>,
    outcome: Option<RealizedOutcome>,
    unfinished_reason: Option<String>,
    terminal_tick: Option<u64>,
    completed_los: Option<u64>,
    occupancy_until: u64,
    boards: bool,
}

#[derive(Default)]
struct Totals {
    arrivals: u64,
    started: u64,
    terminal: u64,
    unfinished: u64,
    active_work: u64,
    completed_los: u64,
    completed_los_count: u64,
    first_care_wait: u64,
    observed_elapsed: u64,
    queue_wait: u64,
}

impl Totals {
    fn observe(
        &mut self,
        stages: &StageAccount,
        observed: u64,
        completed_los: Option<u64>,
        terminal: bool,
    ) {
        self.arrivals += 1;
        if stages.started {
            self.started += 1;
        }
        self.active_work += stages.active_work;
        self.first_care_wait += stages.first_care_wait;
        self.queue_wait += stages.queue_wait;
        self.observed_elapsed += observed;
        if terminal {
            self.terminal += 1;
            if let Some(los) = completed_los {
                self.completed_los += los;
                self.completed_los_count += 1;
            }
        } else {
            self.unfinished += 1;
        }
    }

    fn metrics(&self) -> EndpointMetrics {
        EndpointMetrics {
            mean_active_work: mean(self.active_work, self.arrivals),
            mean_completed_los: mean(self.completed_los, self.completed_los_count),
            mean_first_care_wait: mean(self.first_care_wait, self.arrivals),
            mean_observed_elapsed: mean(self.observed_elapsed, self.arrivals),
            mean_total_stage_queue_wait: mean(self.queue_wait, self.arrivals),
        }
    }
}

fn resolve_patient(
    pathway: &PathwayResult,
    provisional: Option<DispositionIntent>,
    offers: &[&BedOffer],
    horizon_tick: u64,
    options: &ExitOptions,
    consumed: &mut HashSet<String>,
) -> Result<ResolvedExit, PathwayError> {
    let stages_end = treatment_end(pathway);
    if let Some(tick) = abandonment_tick(pathway, stages_end, horizon_tick, options)? {
        let los = checked_sub(tick, pathway.arrival_tick)?;
        return Ok(ResolvedExit {
            boarding: None,
            transfer: None,
            outcome: Some(RealizedOutcome {
                kind: "abandoned".into(),
                destination: None,
                offer_id: None,
                terminal_tick: tick,
            }),
            unfinished_reason: None,
            terminal_tick: Some(tick),
            completed_los: Some(los),
            occupancy_until: tick,
            boards: false,
        });
    }
    if let Some(end) = stages_end {
        if let Some(destination) = options.transfer_requests.get(&pathway.patient_id) {
            let policy = options
                .transfer
                .as_ref()
                .ok_or_else(|| PathwayError::InvalidInput("transfer policy is disabled".into()))?;
            return facility_transfer(pathway, end, destination, policy, horizon_tick);
        }
        match provisional {
            Some(DispositionIntent::Discharge) => {
                let los = checked_sub(end, pathway.arrival_tick)?;
                Ok(ResolvedExit {
                    boarding: None,
                    transfer: None,
                    outcome: Some(RealizedOutcome {
                        kind: "discharged".into(),
                        destination: None,
                        offer_id: None,
                        terminal_tick: end,
                    }),
                    unfinished_reason: None,
                    terminal_tick: Some(end),
                    completed_los: Some(los),
                    occupancy_until: end,
                    boards: false,
                })
            }
            Some(DispositionIntent::Admission) => {
                admit_or_board(pathway, end, offers, horizon_tick, options, consumed)
            }
            None => unfinished(horizon_tick, None, false),
        }
    } else {
        unfinished(horizon_tick, None, false)
    }
}

fn abandonment_tick(
    pathway: &PathwayResult,
    treatment_end: Option<u64>,
    horizon_tick: u64,
    options: &ExitOptions,
) -> Result<Option<u64>, PathwayError> {
    let Some(policy) = &options.abandonment else {
        return Ok(None);
    };
    if treatment_end.is_some() {
        return Ok(None);
    }
    let tick = pathway
        .arrival_tick
        .checked_add(policy.patience_ticks)
        .ok_or_else(|| PathwayError::InvalidInput("tick addition overflowed".into()))?;
    Ok((tick <= horizon_tick).then_some(tick))
}

fn facility_transfer(
    pathway: &PathwayResult,
    decision_tick: u64,
    destination: &str,
    policy: &FacilityTransferPolicy,
    horizon_tick: u64,
) -> Result<ResolvedExit, PathwayError> {
    if !policy.eligible_destinations.contains(destination) {
        return invalid("transfer destination is not eligible");
    }
    let departure = checked_add(decision_tick, policy.transport_delay_ticks)?;
    let transfer = TransferInterval {
        offer_id: String::new(),
        start_tick: decision_tick,
        end_tick: departure.min(horizon_tick),
        duration_ticks: departure.min(horizon_tick).saturating_sub(decision_tick),
    };
    if departure <= horizon_tick {
        let los = checked_sub(departure, pathway.arrival_tick)?;
        Ok(ResolvedExit {
            boarding: None,
            transfer: Some(TransferInterval {
                duration_ticks: policy.transport_delay_ticks,
                end_tick: departure,
                ..transfer
            }),
            outcome: Some(RealizedOutcome {
                kind: "transferred".into(),
                destination: Some(destination.to_owned()),
                offer_id: None,
                terminal_tick: departure,
            }),
            unfinished_reason: None,
            terminal_tick: Some(departure),
            completed_los: Some(los),
            occupancy_until: departure,
            boards: true,
        })
    } else {
        let mut censored = unfinished(horizon_tick, Some("censored_during_transfer".into()), true)?;
        censored.transfer = Some(transfer);
        censored.occupancy_until = horizon_tick;
        censored.boards = true;
        Ok(censored)
    }
}

fn admit_or_board(
    pathway: &PathwayResult,
    treatment_end: u64,
    offers: &[&BedOffer],
    horizon_tick: u64,
    options: &ExitOptions,
    consumed: &mut HashSet<String>,
) -> Result<ResolvedExit, PathwayError> {
    let allowed = options
        .compatible_destinations
        .get(pathway.patient_id.as_str());
    for offer in offers {
        if consumed.contains(offer.offer_id.as_str()) {
            continue;
        }
        if allowed.is_some_and(|destinations| !destinations.contains(&offer.destination)) {
            continue;
        }
        let Some(acceptance) = acceptance_tick(offer, treatment_end) else {
            continue;
        };
        let departure = checked_add(acceptance, offer.transfer_duration_ticks)?;
        let inside_window = departure <= offer.withdrawn_tick;
        if inside_window && departure <= horizon_tick {
            consumed.insert(offer.offer_id.clone());
            let los = checked_sub(departure, pathway.arrival_tick)?;
            return Ok(ResolvedExit {
                boarding: interval_between(treatment_end, acceptance)?,
                transfer: Some(TransferInterval {
                    offer_id: offer.offer_id.clone(),
                    start_tick: acceptance,
                    end_tick: departure,
                    duration_ticks: offer.transfer_duration_ticks,
                }),
                outcome: Some(RealizedOutcome {
                    kind: "transferred".into(),
                    destination: Some(offer.destination.clone()),
                    offer_id: Some(offer.offer_id.clone()),
                    terminal_tick: departure,
                }),
                unfinished_reason: None,
                terminal_tick: Some(departure),
                completed_los: Some(los),
                occupancy_until: departure,
                boards: true,
            });
        }
        if inside_window && acceptance <= horizon_tick {
            consumed.insert(offer.offer_id.clone());
            let mut censored =
                unfinished(horizon_tick, Some("censored_during_transfer".into()), true)?;
            censored.boarding = interval_between(treatment_end, acceptance)?;
            censored.transfer = Some(TransferInterval {
                offer_id: offer.offer_id.clone(),
                start_tick: acceptance,
                end_tick: horizon_tick,
                duration_ticks: checked_sub(horizon_tick, acceptance)?,
            });
            censored.occupancy_until = horizon_tick;
            return Ok(censored);
        }
    }
    let mut boarding = unfinished(
        horizon_tick,
        Some("boarding_without_compatible_offer".into()),
        true,
    )?;
    boarding.boarding = interval_between(treatment_end, horizon_tick)?;
    boarding.occupancy_until = horizon_tick;
    Ok(boarding)
}

fn acceptance_tick(offer: &BedOffer, treatment_end: u64) -> Option<u64> {
    let acceptance = treatment_end.max(offer.available_tick);
    (acceptance >= offer.available_tick && acceptance < offer.withdrawn_tick).then_some(acceptance)
}

fn unfinished(
    horizon_tick: u64,
    reason: Option<String>,
    boards: bool,
) -> Result<ResolvedExit, PathwayError> {
    Ok(ResolvedExit {
        boarding: None,
        transfer: None,
        outcome: None,
        unfinished_reason: reason,
        terminal_tick: None,
        completed_los: None,
        occupancy_until: horizon_tick,
        boards,
    })
}

fn interval_between(start: u64, end: u64) -> Result<Option<BoardingInterval>, PathwayError> {
    if end < start {
        return invalid("boarding interval ends before it starts");
    }
    Ok(Some(BoardingInterval {
        start_tick: start,
        end_tick: end,
        duration_ticks: checked_sub(end, start)?,
    }))
}

fn treatment_end(pathway: &PathwayResult) -> Option<u64> {
    pathway
        .episodes
        .iter()
        .rev()
        .find(|episode| episode.kind == StageKind::Treatment)
        .and_then(|episode| episode.completion_tick)
}

fn stage_account(
    pathway: &PathwayResult,
    input: &PathwayPatient,
) -> Result<StageAccount, PathwayError> {
    let mut intervals = Vec::new();
    let mut active_work = 0u64;
    let mut queue_wait = 0u64;
    for (index, episode) in pathway.episodes.iter().enumerate() {
        let (Some(start), Some(end)) = (episode.start_tick, episode.completion_tick) else {
            continue;
        };
        if end < start {
            return invalid("stage completion precedes its start");
        }
        active_work = checked_add(active_work, checked_sub(end, start)?)?;
        if start < episode.queue_entry_tick {
            return invalid("stage start precedes queue entry");
        }
        queue_wait = checked_add(queue_wait, checked_sub(start, episode.queue_entry_tick)?)?;
        let resource_id = input
            .stages
            .get(index)
            .map(|stage| stage.resource_id.clone())
            .ok_or_else(|| {
                PathwayError::InvalidInput("pathway episode has no input stage".into())
            })?;
        intervals.push(StageInterval {
            stage: stage_name(episode.kind).into(),
            resource_id,
            start_tick: start,
            end_tick: end,
        });
    }
    let first_start = pathway
        .episodes
        .iter()
        .find_map(|episode| episode.start_tick);
    Ok(StageAccount {
        intervals,
        active_work,
        queue_wait,
        first_care_wait: first_start
            .map(|start| checked_sub(start, pathway.arrival_tick))
            .transpose()?
            .unwrap_or(0),
        started: first_start.is_some(),
        treatment_end: treatment_end(pathway),
    })
}

fn boarder_care(
    patient_id: &str,
    start: u64,
    until_exclusive: u64,
    policy: &BoarderCarePolicy,
    horizon_tick: u64,
) -> Result<Vec<BoarderCareTask>, PathwayError> {
    let mut tasks = Vec::new();
    let mut tick = start;
    while tick < until_exclusive {
        if tasks.len() == MAX_CARE_TASKS {
            return invalid("boarder-care task limit exceeded");
        }
        let completion = checked_add(tick, policy.duration_ticks)?;
        let finishes = completion <= until_exclusive && completion <= horizon_tick;
        let ordinal = tasks.len();
        tasks.push(BoarderCareTask {
            task_id: format!("{patient_id}.boarder-care.{ordinal}"),
            patient_id: patient_id.to_owned(),
            start_tick: tick,
            completion_tick: finishes.then_some(completion),
            censored: !finishes,
            reserves_staff: false,
        });
        match tick.checked_add(policy.interval_ticks) {
            Some(next) => tick = next,
            None => break,
        }
    }
    Ok(tasks)
}

fn validate_options(options: &ExitOptions) -> Result<(), PathwayError> {
    if let Some(policy) = &options.abandonment {
        if policy.version != 1 {
            return invalid("abandonment policy version must be 1");
        }
    }
    if !options.transfer_requests.is_empty() && options.transfer.is_none() {
        return invalid("transfer policy is disabled");
    }
    if let Some(policy) = &options.transfer {
        if policy.version != 1 {
            return invalid("transfer policy version must be 1");
        }
        if policy.eligible_destinations.is_empty() {
            return invalid("transfer policy has no eligible destination");
        }
        for destination in &policy.eligible_destinations {
            if !valid_identifier(destination) {
                return invalid("transfer destination is invalid");
            }
        }
        for (patient_id, destination) in &options.transfer_requests {
            if !valid_identifier(patient_id) || !policy.eligible_destinations.contains(destination)
            {
                return invalid("transfer destination is not eligible");
            }
        }
    }
    if let Some(policy) = &options.care {
        if policy.version != 1 {
            return invalid("boarding policy version must be 1");
        }
        if policy.interval_ticks == 0 || policy.duration_ticks == 0 {
            return invalid("boarder-care interval and duration must be positive");
        }
    }
    Ok(())
}

fn index_inputs(
    inputs: &[PathwayPatient],
) -> Result<BTreeMap<&str, &PathwayPatient>, PathwayError> {
    let mut indexed = BTreeMap::new();
    for patient in inputs {
        if indexed
            .insert(patient.patient_id.as_str(), patient)
            .is_some()
        {
            return invalid("duplicate input patient");
        }
    }
    Ok(indexed)
}

fn stage_name(kind: StageKind) -> &'static str {
    match kind {
        StageKind::Triage => "triage",
        StageKind::Assessment => "assessment",
        StageKind::Diagnostics => "diagnostics",
        StageKind::Treatment => "treatment",
    }
}

fn mean(sum: u64, count: u64) -> Option<f64> {
    (count > 0).then_some(sum as f64 / count as f64)
}

fn checked_add(left: u64, right: u64) -> Result<u64, PathwayError> {
    left.checked_add(right)
        .ok_or_else(|| PathwayError::InvalidInput("tick addition overflowed".into()))
}

fn checked_sub(left: u64, right: u64) -> Result<u64, PathwayError> {
    left.checked_sub(right)
        .ok_or_else(|| PathwayError::InvalidInput("tick subtraction underflowed".into()))
}

fn valid_identifier(value: &str) -> bool {
    let bytes = value.as_bytes();
    !bytes.is_empty()
        && bytes.len() <= 64
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(*byte, b'_' | b'.' | b'-'))
}

fn invalid<T>(message: &str) -> Result<T, PathwayError> {
    Err(PathwayError::InvalidInput(message.to_owned()))
}
