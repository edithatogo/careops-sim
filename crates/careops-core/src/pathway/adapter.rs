//! Bounded configurable pathway adapter with independently scheduled clinical progression (CC-04, CC-06, CC-07).

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

use super::error::PathwayError;
use super::progression::CompetingRiskModel;
use super::stage::{StageGraph, StageId, StageNode};
use super::streams::PathwayRngStreams;

/// Specification for a patient entering a configurable clinical pathway.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PathwayPatientSpec {
    pub patient_id: String,
    pub arrival_tick: u64,
    pub initial_stage: StageId,
    pub acuity: u8,
    pub covariates: HashMap<String, f64>,
    pub planned_stage_durations: HashMap<StageId, u64>,
    /// Explicit branch choices keyed by the stage being completed.
    #[serde(default)]
    pub stage_route_choices: HashMap<StageId, StageId>,
    pub base_seed: u64,
}

/// Recorded outcome of a single stage episode for a patient.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageExecutionRecord {
    pub stage_id: StageId,
    pub episode_index: usize,
    pub queue_entry_tick: u64,
    pub start_tick: Option<u64>,
    pub completion_tick: Option<u64>,
    pub observed_duration_ticks: u64,
    pub is_censored: bool,
}

/// Clinical progression event logged during waiting or active care.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClinicalProgressionLog {
    pub tick: u64,
    pub patient_id: String,
    pub cause: String,
    pub previous_acuity: u8,
    pub new_acuity: u8,
}

/// Final outcome of a patient traversing a configurable pathway.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PatientPathwayOutcome {
    pub patient_id: String,
    pub arrival_tick: u64,
    pub final_stage: StageId,
    pub final_acuity: u8,
    pub is_completed: bool,
    pub is_censored: bool,
    pub stage_records: Vec<StageExecutionRecord>,
    pub progression_logs: Vec<ClinicalProgressionLog>,
}

/// Internal active patient tracker within the simulation adapter.
struct ActivePatientState {
    spec: PathwayPatientSpec,
    current_stage: StageId,
    current_acuity: u8,
    stage_repetitions: HashMap<StageId, usize>,
    stage_records: Vec<StageExecutionRecord>,
    progression_logs: Vec<ClinicalProgressionLog>,
    queue_entry_tick: u64,
    service_start_tick: Option<u64>,
    remaining_service_ticks: u64,
    streams: PathwayRngStreams,
    completed: bool,
    censored: bool,
}

/// Configurable pathway adapter managing stage transitions, capacity, and clinical progression.
pub struct ConfigurablePathwayAdapter {
    pub graph: StageGraph,
    pub horizon_ticks: u64,
    pub stage_capacities: HashMap<StageId, usize>,
    pub progression_model: Option<CompetingRiskModel>,
}

impl ConfigurablePathwayAdapter {
    pub fn new(graph: StageGraph, horizon_ticks: u64) -> Self {
        Self {
            graph,
            horizon_ticks,
            stage_capacities: HashMap::new(),
            progression_model: None,
        }
    }

    /// Sets physical/staffed server capacity for a stage.
    pub fn set_stage_capacity(&mut self, stage_id: StageId, capacity: usize) {
        self.stage_capacities.insert(stage_id, capacity);
    }

    /// Attaches an independent clinical progression model.
    pub fn set_progression_model(&mut self, model: CompetingRiskModel) {
        self.progression_model = Some(model);
    }

    /// Constructs the canonical 4-stage ED pathway graph (Triage -> Assessment -> Diagnostics -> Treatment -> Discharge).
    pub fn build_ed_canonical_graph() -> StageGraph {
        let mut graph = StageGraph::new();

        let s_triage = StageId::new("triage");
        let s_assessment = StageId::new("assessment");
        let s_diagnostics = StageId::new("diagnostics");
        let s_treatment = StageId::new("treatment");
        let s_discharge = StageId::new("discharge");

        graph.add_node(StageNode {
            id: s_triage.clone(),
            name: "Triage".to_string(),
            max_repetitions: 1,
            is_terminal: false,
        });
        graph.add_node(StageNode {
            id: s_assessment.clone(),
            name: "Assessment".to_string(),
            max_repetitions: 1,
            is_terminal: false,
        });
        graph.add_node(StageNode {
            id: s_diagnostics.clone(),
            name: "Diagnostics".to_string(),
            max_repetitions: 16, // Maximum 16 diagnostic repeats per ED specification
            is_terminal: false,
        });
        graph.add_node(StageNode {
            id: s_treatment.clone(),
            name: "Treatment".to_string(),
            max_repetitions: 1,
            is_terminal: false,
        });
        graph.add_node(StageNode {
            id: s_discharge.clone(),
            name: "Discharge".to_string(),
            max_repetitions: 1,
            is_terminal: true,
        });

        // Register valid ED transitions
        graph.add_transition(&s_triage, &s_assessment);
        graph.add_transition(&s_assessment, &s_diagnostics);
        graph.add_transition(&s_assessment, &s_treatment); // Direct assessment to treatment
        graph.add_transition(&s_diagnostics, &s_diagnostics); // Repeated diagnostic cycle
        graph.add_transition(&s_diagnostics, &s_treatment);
        graph.add_transition(&s_treatment, &s_discharge);

        graph
    }

    /// Executes the configurable pathway simulation for a cohort of patients up to `horizon_ticks`.
    pub fn run_simulation(
        &self,
        patients: &[PathwayPatientSpec],
    ) -> Result<Vec<PatientPathwayOutcome>, PathwayError> {
        let mut arrivals_by_tick: HashMap<u64, Vec<PathwayPatientSpec>> = HashMap::new();
        for p in patients {
            arrivals_by_tick
                .entry(p.arrival_tick)
                .or_default()
                .push(p.clone());
        }

        // Queues of patient indices per stage waiting for capacity
        let mut stage_queues: HashMap<StageId, VecDeque<usize>> = HashMap::new();
        // Currently active patients per stage: Vec<patient_idx>
        let mut stage_active: HashMap<StageId, Vec<usize>> = HashMap::new();

        let mut patient_states: Vec<ActivePatientState> = Vec::with_capacity(patients.len());

        for current_tick in 0..=self.horizon_ticks {
            // 1. Process new arrivals at current_tick
            if let Some(new_arrivals) = arrivals_by_tick.remove(&current_tick) {
                for arrival in new_arrivals {
                    let idx = patient_states.len();
                    let initial_stage = arrival.initial_stage.clone();
                    self.graph.is_terminal(&initial_stage)?;
                    let initial_acuity = arrival.acuity;
                    let base_seed = arrival.base_seed;

                    let state = ActivePatientState {
                        spec: arrival,
                        current_stage: initial_stage.clone(),
                        current_acuity: initial_acuity,
                        stage_repetitions: HashMap::new(),
                        stage_records: Vec::new(),
                        progression_logs: Vec::new(),
                        queue_entry_tick: current_tick,
                        service_start_tick: None,
                        remaining_service_ticks: 0,
                        streams: PathwayRngStreams::new(base_seed),
                        completed: false,
                        censored: false,
                    };
                    patient_states.push(state);
                    stage_queues
                        .entry(initial_stage)
                        .or_default()
                        .push_back(idx);
                }
            }

            // 2. Evaluate independent clinical progression for ALL incomplete patients (CC-04)
            // (Progression occurs whether patient is queued waiting or receiving active service)
            if let Some(ref model) = self.progression_model {
                for state in &mut patient_states {
                    if state.completed || state.censored {
                        continue;
                    }
                    if state.current_acuity > 1 {
                        if let Some(cause) = model.sample_event(
                            &state.spec.covariates,
                            &mut state.streams.progression_stream,
                        )? {
                            // A sampled cause is preserved for audit; acuity is the bounded
                            // observable progression dimension in this adapter.
                            let prev = state.current_acuity;
                            state.current_acuity -= 1;
                            state.progression_logs.push(ClinicalProgressionLog {
                                tick: current_tick,
                                patient_id: state.spec.patient_id.clone(),
                                cause,
                                previous_acuity: prev,
                                new_acuity: state.current_acuity,
                            });
                        }
                    }
                }
            }

            // 3. Complete active work finishing at current_tick
            for (_stage_id, active_indices) in &mut stage_active {
                let mut finished_indices = Vec::new();
                for &idx in active_indices.iter() {
                    let p = &mut patient_states[idx];
                    if p.remaining_service_ticks > 0 {
                        p.remaining_service_ticks -= 1;
                    }
                    if p.remaining_service_ticks == 0 {
                        finished_indices.push(idx);
                    }
                }

                for finished_idx in finished_indices {
                    active_indices.retain(|&i| i != finished_idx);
                    let p = &mut patient_states[finished_idx];
                    let episode_idx = p.stage_records.len();

                    let record = StageExecutionRecord {
                        stage_id: p.current_stage.clone(),
                        episode_index: episode_idx,
                        queue_entry_tick: p.queue_entry_tick,
                        start_tick: p.service_start_tick,
                        completion_tick: Some(current_tick),
                        observed_duration_ticks: current_tick
                            - p.service_start_tick.unwrap_or(current_tick),
                        is_censored: false,
                    };
                    p.stage_records.push(record);

                    // Determine next stage transition
                    let next_stage_opt = self.determine_next_stage(p)?;
                    match next_stage_opt {
                        Some(next_stage) => {
                            let reps = p
                                .stage_repetitions
                                .entry(p.current_stage.clone())
                                .or_insert(0);
                            self.graph
                                .validate_transition(&p.current_stage, &next_stage, *reps)?;
                            if p.current_stage == next_stage {
                                *reps += 1;
                            }
                            p.current_stage = next_stage.clone();
                            p.queue_entry_tick = current_tick;
                            p.service_start_tick = None;
                            stage_queues
                                .entry(next_stage)
                                .or_default()
                                .push_back(finished_idx);
                        }
                        None => {
                            // Completed full pathway
                            p.completed = true;
                        }
                    }
                }
            }

            // 4. Grant capacity to queued patients (FCFS per stage)
            for (stage_id, queue) in &mut stage_queues {
                let capacity = self
                    .stage_capacities
                    .get(stage_id)
                    .copied()
                    .unwrap_or(usize::MAX);
                let active = stage_active.entry(stage_id.clone()).or_default();

                while active.len() < capacity && !queue.is_empty() {
                    let next_patient_idx = queue.pop_front().unwrap();
                    let p = &mut patient_states[next_patient_idx];

                    // Sample or retrieve stage duration using service stream
                    let duration = p
                        .spec
                        .planned_stage_durations
                        .get(stage_id)
                        .copied()
                        .unwrap_or(10);

                    p.service_start_tick = Some(current_tick);
                    p.remaining_service_ticks = duration;
                    active.push(next_patient_idx);
                }
            }

            // 5. Right-censoring at horizon
            if current_tick == self.horizon_ticks {
                for p in &mut patient_states {
                    if !p.completed {
                        p.censored = true;
                        // Record censored in-flight or queued episode
                        let record = StageExecutionRecord {
                            stage_id: p.current_stage.clone(),
                            episode_index: p.stage_records.len(),
                            queue_entry_tick: p.queue_entry_tick,
                            start_tick: p.service_start_tick,
                            completion_tick: None,
                            observed_duration_ticks: current_tick
                                .saturating_sub(p.service_start_tick.unwrap_or(p.queue_entry_tick)),
                            is_censored: true,
                        };
                        p.stage_records.push(record);
                    }
                }
            }
        }

        // Assemble final outcome objects
        let outcomes = patient_states
            .into_iter()
            .map(|s| PatientPathwayOutcome {
                patient_id: s.spec.patient_id,
                arrival_tick: s.spec.arrival_tick,
                final_stage: s.current_stage,
                final_acuity: s.current_acuity,
                is_completed: s.completed,
                is_censored: s.censored,
                stage_records: s.stage_records,
                progression_logs: s.progression_logs,
            })
            .collect();

        Ok(outcomes)
    }

    /// Determines the next stage for a patient finishing their current stage.
    fn determine_next_stage(
        &self,
        patient: &ActivePatientState,
    ) -> Result<Option<StageId>, PathwayError> {
        let current = &patient.current_stage;
        if self.graph.is_terminal(current)? {
            return Ok(None);
        }
        let transitions = self.graph.outgoing_transitions(current)?;
        if transitions.is_empty() {
            return Err(PathwayError::new(
                super::error::PathwayErrorKind::InvalidTransition,
                format!(
                    "non-terminal stage '{}' has no outgoing transition",
                    current.0
                ),
            ));
        }
        if let Some(choice) = patient.spec.stage_route_choices.get(current) {
            self.graph.validate_transition(
                current,
                choice,
                patient.stage_repetitions.get(current).copied().unwrap_or(0),
            )?;
            return Ok(Some(choice.clone()));
        }
        if transitions.len() == 1 {
            return Ok(transitions.into_iter().next());
        }

        // Planned durations act as explicit route hints for compatibility with the
        // original ED adapter. Ambiguous branches must be supplied in
        // stage_route_choices instead of inferred from domain-specific stage names.
        let hinted: Vec<_> = transitions
            .into_iter()
            .filter(|stage| patient.spec.planned_stage_durations.contains_key(stage))
            .collect();
        match hinted.as_slice() {
            [choice] => Ok(Some(choice.clone())),
            [] => Err(PathwayError::new(
                super::error::PathwayErrorKind::InvalidTransition,
                format!(
                    "branch at '{}' requires an explicit route choice",
                    current.0
                ),
            )),
            _ => Err(PathwayError::new(
                super::error::PathwayErrorKind::InvalidTransition,
                format!(
                    "branch at '{}' has multiple planned routes; choose one explicitly",
                    current.0
                ),
            )),
        }
    }
}
