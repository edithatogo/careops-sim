use std::collections::{BTreeMap, BTreeSet};

use careops_ed::disposition::{
    DecisionPolicy, DecisionReveal, DispositionBinding, DispositionIntent,
};
use careops_ed::offers::{BedOffer, OfferPolicy};
use careops_ed::pathway::{
    PathwayConfig, PathwayPatient, ServiceBinding, ServiceLaw, ServiceProvenance, ServiceSpec,
    StageKind, StageWork,
};
use careops_ed::terminal::{
    AbandonmentPolicy, AdmissionDestination, BoarderCarePolicy, TerminalConfig, TerminalPolicy,
    TerminalRunResult,
};
use careops_ed::{
    AggregateCapacity, CapacityLocationProfile, Eligibility, KnowledgeStatus, Location,
    ResourceBucket, StaffCount, StaffRole, TaskClass, Zone,
};

#[derive(Clone)]
pub struct PatientSpec {
    pub id: &'static str,
    pub arrival: u64,
    pub intent: DispositionIntent,
    pub stages: [u64; 3],
    pub resources: [&'static str; 3],
}

#[derive(Clone)]
pub struct OfferSpec {
    pub id: &'static str,
    pub destination: &'static str,
    pub patient: Option<&'static str>,
    pub available: u64,
    pub withdrawn: u64,
    pub duration: u64,
}

pub struct BuildOptions {
    pub horizon: u64,
    pub patients: Vec<PatientSpec>,
    pub offers: Vec<OfferSpec>,
    pub discharge_delay: u64,
    pub care: Option<(String, String, u64, u64)>,
    pub capacities: BTreeMap<String, u64>,
}

pub fn standard(horizon: u64, patients: Vec<PatientSpec>) -> TerminalConfig {
    build(BuildOptions {
        horizon,
        patients,
        offers: vec![],
        discharge_delay: 0,
        care: None,
        capacities: BTreeMap::new(),
    })
}

pub fn build(options: BuildOptions) -> TerminalConfig {
    let mut capacities = BTreeMap::from([
        ("res.assessment".to_owned(), 1),
        ("res.space".to_owned(), 4),
        ("res.transfer".to_owned(), 1),
        ("res.treatment".to_owned(), 1),
        ("res.triage".to_owned(), 1),
    ]);
    capacities.extend(options.capacities);

    let mut all_resources = capacities.clone();
    let (care, care_resource_id, care_task_id) = match options.care {
        Some((resource, task, interval, duration)) => {
            all_resources.entry(resource.clone()).or_insert(1);
            (
                Some(BoarderCarePolicy {
                    resource_id: resource.clone(),
                    task_class_id: task.clone(),
                    interval_ticks: interval,
                    duration_ticks: duration,
                }),
                resource,
                task,
            )
        }
        None => (None, String::new(), String::new()),
    };
    let mut zones = BTreeMap::new();
    let mut locations = BTreeMap::new();
    let mut resources = BTreeMap::new();
    let mut tasks = BTreeMap::new();
    let mut eligibility = BTreeSet::new();
    for (resource_id, capacity) in &all_resources {
        let (resource_class, task_id, zone_id) = if resource_id == "res.space" {
            ("space", "task.space", "zone.space")
        } else if resource_id == "res.transfer" {
            ("transfer", "task.transfer", "zone.transfer")
        } else if resource_id == "res.triage" {
            ("nurse", "task.triage", "zone.triage")
        } else if resource_id == "res.assessment" {
            ("nurse", "task.assessment", "zone.assessment")
        } else if resource_id == "res.treatment" {
            ("nurse", "task.treatment", "zone.treatment")
        } else {
            ("nurse", resource_id.as_str(), "zone.shared")
        };
        let location_id = format!("location.{zone_id}");
        zones.insert(
            zone_id.to_owned(),
            Zone {
                zone_id: zone_id.to_owned(),
                name: zone_id.to_owned(),
            },
        );
        locations.insert(
            location_id.clone(),
            Location {
                location_id: location_id.clone(),
                name: location_id.clone(),
                zone_id: zone_id.to_owned(),
            },
        );
        resources.insert(
            resource_id.clone(),
            ResourceBucket {
                resource_id: resource_id.clone(),
                resource_class: resource_class.to_owned(),
                location_id,
                zone_id: zone_id.to_owned(),
                pool_group_id: None,
                physical_status: KnowledgeStatus::Known,
                physical_count: Some(*capacity),
                availability_status: KnowledgeStatus::Known,
                open_count: Some(*capacity),
                staffed_count: Some(*capacity),
                availability_reason: None,
                capabilities: vec![],
            },
        );
        tasks
            .entry(task_id.to_owned())
            .or_insert_with(|| TaskClass {
                task_class_id: task_id.to_owned(),
                name: task_id.to_owned(),
                required_resource_classes: vec![resource_class.to_owned()],
            });
        eligibility.insert((task_id.to_owned(), zone_id.to_owned()));
        if !care_resource_id.is_empty() && *resource_id == care_resource_id {
            tasks
                .entry(care_task_id.clone())
                .or_insert_with(|| TaskClass {
                    task_class_id: care_task_id.clone(),
                    name: care_task_id.clone(),
                    required_resource_classes: vec![resource_class.to_owned()],
                });
            eligibility.insert((care_task_id.clone(), zone_id.to_owned()));
        }
    }
    let profile = CapacityLocationProfile {
        schema_version: 3,
        status: "proposed".into(),
        capacity: AggregateCapacity {
            unit: "treatment_space".into(),
            bucket_completeness: "complete".into(),
            physical_status: KnowledgeStatus::Known,
            physical_count: Some(4),
            availability_status: KnowledgeStatus::Known,
            open_count: Some(4),
            staffed_count: Some(4),
            availability_reason: None,
            equipment: vec![],
        },
        zones: zones.into_values().collect(),
        locations: locations.into_values().collect(),
        resource_buckets: resources.into_values().collect(),
        task_classes: tasks.into_values().collect(),
        staff_roles: vec![StaffRole {
            staff_role_id: "staff.synthetic".into(),
            name: "synthetic".into(),
            counts: vec![StaffCount {
                basis: "present".into(),
                status: KnowledgeStatus::Known,
                count: Some(8),
                reference: "synthetic.fixture".into(),
                reason: None,
            }],
        }],
        eligibility: eligibility
            .into_iter()
            .map(|(task_class_id, zone_id)| Eligibility {
                staff_role_id: "staff.synthetic".into(),
                task_class_id,
                zone_id,
                location_id: None,
            })
            .collect(),
        routes: vec![],
    };

    let mut patients = Vec::new();
    let mut decisions = Vec::new();
    let mut admissions = Vec::new();
    let mut services = Vec::new();
    for p in options.patients {
        let stage_rows = [
            (
                StageKind::Triage,
                p.resources[0],
                "task.triage",
                p.stages[0],
            ),
            (
                StageKind::Assessment,
                p.resources[1],
                "task.assessment",
                p.stages[1],
            ),
            (
                StageKind::Treatment,
                p.resources[2],
                "task.treatment",
                p.stages[2],
            ),
        ];
        let stages = stage_rows
            .iter()
            .map(|(kind, resource, task, duration)| StageWork {
                kind: *kind,
                resource_id: (*resource).into(),
                task_class_id: (*task).into(),
                duration_ticks: *duration,
            })
            .collect();
        patients.push(PathwayPatient {
            patient_id: p.id.into(),
            arrival_tick: p.arrival,
            mode: "synthetic".into(),
            acuity: 3,
            stages,
        });
        decisions.push(DispositionBinding {
            patient_id: p.id.into(),
            latent_intent: p.intent,
        });
        if p.intent == DispositionIntent::Admission {
            admissions.push(AdmissionDestination {
                patient_id: p.id.into(),
                destination: "dest.med".into(),
            });
        }
        for (stage_index, (_, _, _, duration)) in stage_rows.iter().enumerate() {
            services.push(ServiceBinding {
                patient_id: p.id.into(),
                stage_index,
                service: ServiceSpec {
                    version: 1,
                    provenance: ServiceProvenance {
                        class: "synthetic".into(),
                        notes: "fixed test service".into(),
                    },
                    law: ServiceLaw::Fixed {
                        duration_ticks: *duration,
                    },
                },
            });
        }
    }
    TerminalConfig {
        version: 1,
        pathway: PathwayConfig {
            version: 1,
            horizon_ticks: options.horizon,
            provenance: careops_ed::Provenance {
                class: "synthetic".into(),
                notes: "terminal lifecycle test".into(),
            },
            capacity_location: profile,
            patients,
        },
        seed: 0,
        services,
        decision_policy: DecisionPolicy {
            version: 1,
            provenance: ServiceProvenance {
                class: "synthetic".into(),
                notes: "latent decision fixture".into(),
            },
            reveal_at: DecisionReveal::TreatmentCompletion,
        },
        decisions,
        admissions,
        offer_policy: OfferPolicy {
            version: 1,
            provenance: ServiceProvenance {
                class: "synthetic".into(),
                notes: "finite offer fixture".into(),
            },
        },
        offers: options
            .offers
            .into_iter()
            .map(|o| BedOffer {
                offer_id: o.id.into(),
                destination: o.destination.into(),
                patient_id: o.patient.map(str::to_owned),
                available_tick: o.available,
                withdrawn_tick: o.withdrawn,
                transfer_duration_ticks: o.duration,
            })
            .collect(),
        policy: TerminalPolicy {
            version: 1,
            provenance: ServiceProvenance {
                class: "synthetic".into(),
                notes: "terminal policy fixture".into(),
            },
            ed_space_resource_id: "res.space".into(),
            transfer_resource_id: "res.transfer".into(),
            transfer_task_class_id: "task.transfer".into(),
            discharge_delay_ticks: options.discharge_delay,
            abandonment: AbandonmentPolicy::Disabled,
            boarder_care: care,
        },
    }
}

pub fn patient<'a>(
    result: &'a TerminalRunResult,
    id: &str,
) -> &'a careops_ed::terminal::TerminalPatientResult {
    result
        .patients
        .iter()
        .find(|p| p.pathway.patient_id == id)
        .expect("patient result exists")
}

pub fn assert_conservation(result: &TerminalRunResult) {
    assert_eq!(
        result.counts.arrived,
        result.counts.discharged + result.counts.transferred + result.counts.unfinished
    );
    assert_eq!(result.counts.arrived as usize, result.patients.len());
    let mut discharged = 0u64;
    let mut transferred = 0u64;
    let mut unfinished = 0u64;
    for patient in &result.patients {
        let terminal = patient.endpoint_state == careops_ed::terminal::EndpointState::Terminal;
        assert_eq!(terminal, patient.terminal_tick.is_some());
        assert_eq!(terminal, patient.unfinished_reason.is_none());
        assert_eq!(
            patient.terminal_tick.is_some(),
            patient.realized_outcome.is_some()
        );
        assert_eq!(
            patient.terminal_tick.is_some(),
            patient.completed_los_ticks.is_some()
        );
        assert_eq!(
            patient
                .terminal_tick
                .map(|tick| tick - patient.pathway.arrival_tick),
            patient.completed_los_ticks
        );
        match (&patient.realized_outcome, terminal) {
            (Some(careops_ed::terminal::RealizedOutcome::Discharged), true) => discharged += 1,
            (Some(careops_ed::terminal::RealizedOutcome::Transferred { .. }), true) => {
                transferred += 1
            }
            (None, false) => unfinished += 1,
            _ => panic!(
                "terminal status and realized outcome disagree for {}",
                patient.pathway.patient_id
            ),
        }
    }
    assert_eq!(result.counts.discharged, discharged);
    assert_eq!(result.counts.transferred, transferred);
    assert_eq!(result.counts.unfinished, unfinished);
}

/// Independently reconstruct resource occupancy from returned row intervals.
pub fn assert_interval_accounting(
    config: &TerminalConfig,
    result: &TerminalRunResult,
    horizon: u64,
) {
    assert_eq!(
        horizon, config.pathway.horizon_ticks,
        "sweep horizon matches config"
    );
    let expected_patient_ids: BTreeSet<_> = config
        .pathway
        .patients
        .iter()
        .map(|p| p.patient_id.as_str())
        .collect();
    let result_patient_ids: Vec<_> = result
        .patients
        .iter()
        .map(|p| p.pathway.patient_id.as_str())
        .collect();
    assert_eq!(
        result_patient_ids.len(),
        result_patient_ids
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .len(),
        "result patient IDs are unique"
    );
    assert_eq!(
        result_patient_ids.iter().copied().collect::<BTreeSet<_>>(),
        expected_patient_ids,
        "all configured patients are returned"
    );
    let expected_resource_ids: BTreeSet<_> = config
        .pathway
        .capacity_location
        .resource_buckets
        .iter()
        .map(|r| r.resource_id.as_str())
        .collect();
    let result_resource_ids: Vec<_> = result
        .resources
        .iter()
        .map(|r| r.resource_id.as_str())
        .collect();
    assert_eq!(
        result_resource_ids.len(),
        result_resource_ids
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .len(),
        "result resource IDs are unique"
    );
    assert_eq!(
        result_resource_ids.iter().copied().collect::<BTreeSet<_>>(),
        expected_resource_ids,
        "all configured resources are accounted for"
    );
    for usage in &result.resources {
        let resource_id = usage.resource_id.as_str();
        let profile_resource = config
            .pathway
            .capacity_location
            .resource_buckets
            .iter()
            .find(|resource| resource.resource_id == resource_id)
            .expect("reported resource is configured");
        let configured_capacity = profile_resource
            .open_count
            .unwrap_or(0)
            .min(profile_resource.staffed_count.unwrap_or(0));
        assert_eq!(
            usage.capacity, configured_capacity,
            "configured capacity for {resource_id}"
        );
        assert_eq!(
            usage.capacity_ticks,
            u128::from(configured_capacity) * u128::from(horizon)
        );
        let mut intervals = Vec::<(u64, u64, bool)>::new();
        let mut queued_at_horizon = 0u64;
        for patient in &result.patients {
            assert!(
                patient.pathway.arrival_tick <= horizon,
                "arrival after horizon reported"
            );
            assert!(
                patient.terminal_tick.is_none_or(|tick| tick <= horizon),
                "terminal after horizon reported"
            );
            let elapsed_end = patient.terminal_tick.unwrap_or(horizon).min(horizon);
            assert_eq!(
                patient.observed_elapsed_ticks,
                elapsed_end - patient.pathway.arrival_tick
            );
            if let Some(decision) = &patient.decision {
                assert!(
                    decision.observed_tick <= horizon,
                    "decision after horizon reported"
                );
                if decision.provisional_intent == DispositionIntent::Admission {
                    let boarding_end = patient.terminal_tick.unwrap_or(horizon).min(horizon);
                    assert_eq!(
                        patient.observed_boarding_ticks,
                        Some(boarding_end - decision.observed_tick)
                    );
                }
            }
            assert!(
                patient
                    .allocation
                    .as_ref()
                    .is_none_or(|allocation| allocation.allocated_tick <= horizon),
                "allocation after horizon reported"
            );
            assert!(
                patient
                    .boarding
                    .as_ref()
                    .is_none_or(|interval| interval.end_tick.is_none_or(|tick| tick <= horizon)),
                "boarding end after horizon reported"
            );
            assert!(
                patient
                    .pending_transfer
                    .as_ref()
                    .is_none_or(|interval| interval.end_tick.is_none_or(|tick| tick <= horizon)),
                "pending transfer end after horizon reported"
            );
            if let Some(input) = config
                .pathway
                .patients
                .iter()
                .find(|p| p.patient_id == patient.pathway.patient_id)
            {
                for (index, stage) in patient.pathway.episodes.iter().enumerate() {
                    assert!(
                        stage.start_tick.is_none_or(|tick| tick <= horizon),
                        "future clinical start reported"
                    );
                    assert!(
                        stage.completion_tick.is_none_or(|tick| tick <= horizon),
                        "future clinical completion reported"
                    );
                    if stage.start_tick.is_none()
                        && stage.queue_entry_tick <= horizon
                        && input.stages[index].resource_id == resource_id
                    {
                        queued_at_horizon += 1;
                    }
                    if let Some(start) = stage.start_tick {
                        if input.stages[index].resource_id == resource_id {
                            let actual_end = stage.completion_tick;
                            assert!(
                                actual_end.is_none_or(|tick| tick <= horizon),
                                "future clinical completion reported"
                            );
                            let end = actual_end.unwrap_or(horizon).min(horizon);
                            intervals.push((
                                start.min(horizon),
                                end,
                                actual_end.is_none() || actual_end.is_some_and(|t| t > horizon),
                            ));
                        }
                    }
                }
            }
            for work in patient.care.iter().chain(patient.transfer.iter()) {
                assert!(
                    work.start_tick.is_none_or(|tick| tick <= horizon),
                    "future work start reported"
                );
                assert!(
                    work.completion_tick.is_none_or(|tick| tick <= horizon),
                    "future work completion reported"
                );
                if work.resource_id == resource_id {
                    if work.start_tick.is_none()
                        && work.cancelled_tick.is_none()
                        && work.queue_entry_tick <= horizon
                    {
                        queued_at_horizon += 1;
                    }
                    if let Some(start) = work.start_tick {
                        let actual_end = work.completion_tick;
                        assert!(
                            actual_end.is_none_or(|tick| tick <= horizon),
                            "future work completion reported"
                        );
                        let end = actual_end.unwrap_or(horizon).min(horizon);
                        intervals.push((
                            start.min(horizon),
                            end,
                            actual_end.is_none() || actual_end.is_some_and(|t| t > horizon),
                        ));
                    }
                }
            }
            if patient.space.resource_id == resource_id {
                assert!(
                    patient
                        .space
                        .acquired_tick
                        .is_none_or(|tick| tick <= horizon),
                    "future space grant reported"
                );
                assert!(
                    patient
                        .space
                        .released_tick
                        .is_none_or(|tick| tick <= horizon),
                    "future space release reported"
                );
                if patient.space.acquired_tick.is_none()
                    && patient.space.queue_entry_tick <= horizon
                {
                    queued_at_horizon += 1;
                }
                if let Some(start) = patient.space.acquired_tick {
                    let actual_end = patient.space.released_tick;
                    assert!(
                        actual_end.is_none_or(|tick| tick <= horizon),
                        "future space release reported"
                    );
                    let end = actual_end.unwrap_or(horizon).min(horizon);
                    intervals.push((
                        start.min(horizon),
                        end,
                        actual_end.is_none() || actual_end.is_some_and(|t| t > horizon),
                    ));
                }
            }
        }
        let mut points = BTreeMap::<u64, i64>::new();
        let mut actual_at_h = 0u64;
        for (start, end, open_at_h) in &intervals {
            if end > start {
                *points.entry(*start).or_default() += 1;
                *points.entry(*end).or_default() -= 1;
            }
            if *start <= horizon && *open_at_h {
                actual_at_h += 1;
            }
        }
        let mut busy = 0i64;
        let mut area = 0u128;
        let mut peak = 0u64;
        let mut prev = 0u64;
        for (tick, delta) in points {
            if tick > prev {
                area += (busy as u128) * u128::from(tick - prev);
            }
            busy += delta;
            assert!(
                busy >= 0 && busy <= usage.capacity as i64,
                "capacity exceeded for {}",
                usage.resource_id
            );
            peak = peak.max(busy as u64);
            prev = tick;
        }
        if prev < horizon {
            area += (busy as u128) * u128::from(horizon - prev);
        }
        peak = peak.max(actual_at_h);
        assert_eq!(
            area, usage.busy_ticks,
            "interval area for {}",
            usage.resource_id
        );
        assert_eq!(
            usage.queued_at_horizon, queued_at_horizon,
            "queued-at-H for {}",
            usage.resource_id
        );
        assert_eq!(
            actual_at_h, usage.busy_at_horizon,
            "actual busy at H for {}",
            usage.resource_id
        );
        assert_eq!(peak, usage.peak_busy, "peak busy for {}", usage.resource_id);
    }
}
