use super::*;
use kairo_ecs_abm::spatial::{RoutePlan, TransitGraphV1};
use kairo_ecs_abm::{
    observe_transit_arrival_handoff, pending_transit_arrival_handoff, register_transit_context,
    schedule_transit_control, schedule_transit_start, TransitArrivalHandoff, TransitContext,
    TransitContextCheckpointLimitsV1, TransitContextCheckpointV2, TransitPhase,
};
use kairo_ecs_des::FlowDomainControl;
use std::sync::Arc;

const TRANSIT_REGISTRATION: &str = "careops-ed-transit-handoff-v1";
const TRANSIT_KIND: EventKind = EventKind::custom(4101);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransitBounds {
    pub transits: u64,
    pub total_route_segments: u64,
    pub controls: u64,
    pub native_snapshot_limits: TransitContextCheckpointLimitsV1,
}

pub struct TransitInput {
    pub route_id: String,
    pub graph: Arc<TransitGraphV1>,
    pub route: RoutePlan,
    pub start_tick: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransitHandle {
    runtime: FlowRuntimeIdentity,
    service: WorkId,
    carrier: WorkId,
}

struct TransitEntry {
    identity: ServiceIdentity,
    _route_id: String,
    graph: Arc<TransitGraphV1>,
    carrier: WorkId,
    start_event: EventId,
    controls: BTreeSet<EventId>,
    native_events: BTreeSet<EventId>,
    observed_event: Option<EventId>,
}

pub(super) struct TransitAdapterState {
    bounds: TransitBounds,
    entries: BTreeMap<WorkId, TransitEntry>,
    route_graphs: BTreeMap<String, Vec<u8>>,
    segments_used: u64,
    controls_used: u64,
}

impl EdFlowAdapter {
    pub fn new_with_transit(
        plan: ValidatedAdapterPlan,
        bounds: TransitBounds,
    ) -> Result<Self, AdapterError> {
        Self::build(plan, Some(bounds))
    }

    pub fn begin_transit(
        &mut self,
        handle: &ServiceHandle,
        input: TransitInput,
    ) -> Result<TransitHandle, AdapterError> {
        let state = self.transit.as_ref().ok_or(AdapterError::WrongHandleRole)?;
        if handle.runtime != self.runtime_identity {
            return Err(AdapterError::ForeignHandle);
        }
        let frozen = self
            .services
            .get(&handle.work)
            .ok_or(AdapterError::ForeignHandle)?;
        if frozen.identity != handle.identity || self.used_services.contains(&handle.work) {
            return Err(AdapterError::WrongHandleRole);
        }
        if state.entries.contains_key(&handle.work) {
            return Err(AdapterError::WrongHandleRole);
        }
        let spec = self.flow.work(handle.work)?;
        let progress = self.flow.work_progress(handle.work)?;
        if spec.request.is_some() || progress.state != WorkState::Pending {
            return Err(AdapterError::WrongNativeState);
        }
        check_id(&input.route_id)?;
        if input.graph.canonical_bytes_ref() != input.route.graph_canonical_bytes() {
            return Err(AdapterError::InvalidInput("route graph mismatch"));
        }
        let segment_count = u64::try_from(input.route.segments().len())
            .map_err(|_| AdapterError::BudgetExceeded("route segments"))?;
        if state.entries.len() as u64 >= state.bounds.transits {
            return Err(AdapterError::BudgetExceeded("transits"));
        }
        if state
            .segments_used
            .checked_add(segment_count)
            .filter(|n| *n <= state.bounds.total_route_segments)
            .is_none()
        {
            return Err(AdapterError::BudgetExceeded("route segments"));
        }
        if let Some(previous) = state.route_graphs.get(&input.route_id) {
            if previous.as_slice() != input.graph.canonical_bytes_ref() {
                return Err(AdapterError::InvalidInput("route id graph collision"));
            }
        }
        if (input.start_tick as u128) < self.now().ticks() || input.start_tick > self.horizon_tick {
            return Err(AdapterError::InvalidInput("transit start outside horizon"));
        }
        let transit_context = TransitContext::new_handoff(
            &self.flow,
            input.route.clone(),
            handle.work,
            SimTime::from_ticks(input.start_tick as u128),
            ParentPhase::Ready.priority(),
        )
        .map_err(|_| AdapterError::InvalidInput("invalid native transit route"))?;
        let preview = transit_context
            .checkpoint_v2(&self.runtime_identity, state.bounds.native_snapshot_limits)
            .map_err(|_| AdapterError::InvalidInput("transit snapshot limits"))?;
        drop(preview);
        self.reserve_entity(2)?;
        self.reserve_event()?;
        let actor = self.flow.spawn_actor()?;
        let carrier = self.flow.create_actor_domain_context(
            actor,
            TRANSIT_REGISTRATION,
            TRANSIT_KIND,
            transit_context,
        )?;
        let start_event = schedule_transit_start(
            &mut self.flow,
            carrier,
            TRANSIT_KIND,
            SimTime::from_ticks(input.start_tick as u128),
            ParentPhase::Ready.priority(),
        )?;
        let state = self.transit.as_mut().expect("enabled above");
        state.segments_used += segment_count;
        state
            .route_graphs
            .entry(input.route_id.clone())
            .or_insert_with(|| input.graph.canonical_bytes());
        state.entries.insert(
            handle.work,
            TransitEntry {
                identity: frozen.identity.clone(),
                _route_id: input.route_id,
                graph: input.graph,
                carrier,
                start_event,
                controls: BTreeSet::new(),
                native_events: BTreeSet::new(),
                observed_event: None,
            },
        );
        self.entity_used += 2;
        self.counters.scheduled_adapter_events += 1;
        Ok(TransitHandle {
            runtime: self.runtime_identity.clone(),
            service: handle.work,
            carrier,
        })
    }

    pub fn schedule_transit_control(
        &mut self,
        handle: &TransitHandle,
        action: FlowDomainControl,
        at_tick: u64,
    ) -> Result<EventId, AdapterError> {
        let state = self.transit.as_ref().ok_or(AdapterError::WrongHandleRole)?;
        let entry = state
            .entries
            .get(&handle.service)
            .ok_or(AdapterError::ForeignHandle)?;
        if handle.runtime != self.runtime_identity || entry.carrier != handle.carrier {
            return Err(AdapterError::ForeignHandle);
        }
        if state.controls_used >= state.bounds.controls {
            return Err(AdapterError::BudgetExceeded("transit controls"));
        }
        if (at_tick as u128) < self.now().ticks() || at_tick > self.horizon_tick {
            return Err(AdapterError::InvalidInput(
                "transit control outside horizon",
            ));
        }
        self.reserve_event()?;
        let id = schedule_transit_control(
            &mut self.flow,
            handle.carrier,
            TRANSIT_KIND,
            action,
            SimTime::from_ticks(at_tick as u128),
            ParentPhase::Decision.priority(),
        )?;
        let state = self.transit.as_mut().expect("enabled above");
        state.controls_used += 1;
        state
            .entries
            .get_mut(&handle.service)
            .expect("entry checked")
            .controls
            .insert(id);
        self.counters.scheduled_adapter_events += 1;
        Ok(id)
    }

    pub fn transit_snapshot(
        &self,
        handle: &TransitHandle,
    ) -> Result<TransitContextCheckpointV2, AdapterError> {
        let state = self.transit.as_ref().ok_or(AdapterError::WrongHandleRole)?;
        let entry = state
            .entries
            .get(&handle.service)
            .ok_or(AdapterError::ForeignHandle)?;
        if handle.runtime != self.runtime_identity || entry.carrier != handle.carrier {
            return Err(AdapterError::ForeignHandle);
        }
        self.flow
            .work_context::<TransitContext>(handle.carrier)?
            .checkpoint_v2(&self.runtime_identity, state.bounds.native_snapshot_limits)
            .map_err(|_| AdapterError::InvalidInput("transit snapshot invalid"))
    }

    pub fn transit_arrival(
        &self,
        handle: &TransitHandle,
    ) -> Result<Option<TransitArrivalHandoff>, AdapterError> {
        let state = self.transit.as_ref().ok_or(AdapterError::WrongHandleRole)?;
        let entry = state
            .entries
            .get(&handle.service)
            .ok_or(AdapterError::ForeignHandle)?;
        if handle.runtime != self.runtime_identity || entry.carrier != handle.carrier {
            return Err(AdapterError::ForeignHandle);
        }
        Ok(pending_transit_arrival_handoff(&self.flow, handle.carrier)?)
    }
}

impl TransitAdapterState {
    pub(super) fn new(bounds: TransitBounds) -> Self {
        Self {
            bounds,
            entries: BTreeMap::new(),
            route_graphs: BTreeMap::new(),
            segments_used: 0,
            controls_used: 0,
        }
    }

    pub(super) fn register(flow: &mut FlowRuntime) -> Result<(), AdapterError> {
        register_transit_context(flow, TRANSIT_REGISTRATION, TRANSIT_KIND)?;
        Ok(())
    }

    pub(super) fn owned_carriers(&self, flow: &FlowRuntime, event: EventId) -> Vec<WorkId> {
        self.entries
            .values()
            .filter_map(|entry| {
                let expected = flow
                    .work_context::<TransitContext>(entry.carrier)
                    .ok()
                    .and_then(|ctx| {
                        ctx.checkpoint_v2(&flow.identity(), self.bounds.native_snapshot_limits)
                            .ok()
                    })
                    .and_then(|snap| snap.expected_event);
                (entry.start_event == event
                    || entry.controls.contains(&event)
                    || entry.native_events.contains(&event)
                    || expected == Some((event.index, event.generation)))
                .then_some(entry.carrier)
            })
            .collect()
    }

    pub(super) fn observe(
        &mut self,
        flow: &FlowRuntime,
        dispatch: &FlowDispatch,
        owned_carriers: &[WorkId],
    ) -> Result<Vec<FoundationObservation>, AdapterError> {
        let mut out = Vec::new();
        for entry in self.entries.values_mut() {
            if let Some(arrival) = observe_transit_arrival_handoff(flow, entry.carrier, dispatch)? {
                entry.observed_event = Some(dispatch.event);
                out.push(FoundationObservation::Transit {
                    identity: entry.identity.clone(),
                    carrier: entry.carrier,
                    event: dispatch.event,
                    at: dispatch.at,
                    phase: TransitPhase::Arrived,
                    arrival: Some(arrival),
                });
            } else if owned_carriers.contains(&entry.carrier) {
                let phase = flow
                    .work_context::<TransitContext>(entry.carrier)?
                    .checkpoint_v2(&flow.identity(), self.bounds.native_snapshot_limits)
                    .map_err(|_| AdapterError::Native(FlowError::InvalidWork))?
                    .phase;
                let _graph_identity = entry.graph.canonical_bytes_ref();
                out.push(FoundationObservation::Transit {
                    identity: entry.identity.clone(),
                    carrier: entry.carrier,
                    event: dispatch.event,
                    at: dispatch.at,
                    phase,
                    arrival: None,
                });
            }
            if owned_carriers.contains(&entry.carrier) {
                for batch in &dispatch.callback_batches {
                    if let kairo_ecs_des::FlowBatchReceipt::Accepted(rows) = batch {
                        entry.native_events.extend(rows.iter().map(|row| row.event));
                    }
                }
            }
            entry.native_events.remove(&dispatch.event);
        }
        Ok(out)
    }

    pub(super) fn blocks_admission(&self, work: WorkId) -> bool {
        self.entries
            .get(&work)
            .is_some_and(|entry| entry.observed_event.is_none())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kairo_ecs_abm::spatial::{EdgeId, MovementModeId, MovementProfile, NodeId, TransitEdge};
    use kairo_ecs_abm::{
        register_transit_context_checkpoint_codec_v2, register_transit_context_checkpoint_domain,
    };
    use kairo_ecs_des::{
        FlowCallbackCodeV1, FlowCheckpointCodecError, FlowCheckpointCodecs, FlowCheckpointLimits,
        FlowCheckpointRebindV1,
    };

    fn encode_marker(_: &MarkerContext, _: usize) -> Result<Vec<u8>, FlowCheckpointCodecError> {
        Ok(Vec::new())
    }

    fn decode_marker(
        bytes: &[u8],
        _: &FlowCheckpointRebindV1,
    ) -> Result<MarkerContext, FlowCheckpointCodecError> {
        if bytes.is_empty() {
            Ok(MarkerContext)
        } else {
            Err(FlowCheckpointCodecError("invalid marker bytes".into()))
        }
    }

    fn encode_service(_: &FrozenService, _: usize) -> Result<Vec<u8>, FlowCheckpointCodecError> {
        Ok(Vec::new())
    }

    fn decode_service(
        bytes: &[u8],
        _: &FlowCheckpointRebindV1,
    ) -> Result<FrozenService, FlowCheckpointCodecError> {
        if !bytes.is_empty() {
            return Err(FlowCheckpointCodecError("invalid service bytes".into()));
        }
        Ok(FrozenService {
            identity: ServiceIdentity {
                patient_id: "checkpoint".into(),
                episode_id: "checkpoint-episode".into(),
                stage_index: 0,
                segment_index: 0,
            },
            original_ticks: 1,
            useful_before_ticks: 0,
            remaining_ticks: 1,
            origin: SampleOrigin::Fixed,
        })
    }

    fn transit_codecs(adapter: &EdFlowAdapter, graph: Arc<TransitGraphV1>) -> FlowCheckpointCodecs {
        let mut codecs = FlowCheckpointCodecs::new();
        codecs
            .register_context::<MarkerContext>(MARKER_CONTEXT, 1, encode_marker, decode_marker)
            .unwrap();
        codecs
            .register_domain_plan_hook_with_receipt::<MarkerContext>(
                MARKER_CONTEXT,
                MARKER_KIND,
                no_command_marker_plan,
                no_command_marker_receipt,
                FlowCallbackCodeV1 {
                    stable_id: "careops-ed.marker.plan".into(),
                    version: 1,
                },
                FlowCallbackCodeV1 {
                    stable_id: "careops-ed.marker.receipt".into(),
                    version: 1,
                },
            )
            .unwrap();
        codecs
            .register_context::<FrozenService>(SERVICE_CONTEXT, 1, encode_service, decode_service)
            .unwrap();
        let limits = TransitContextCheckpointLimitsV1::new(16, 4096, 16, 8192);
        register_transit_context_checkpoint_codec_v2(
            &mut codecs,
            TRANSIT_REGISTRATION,
            adapter.runtime_identity.clone(),
            graph,
            limits,
        )
        .unwrap();
        register_transit_context_checkpoint_domain(
            &mut codecs,
            TRANSIT_REGISTRATION,
            TRANSIT_KIND,
            FlowCallbackCodeV1 {
                stable_id: "careops-ed.transit.plan".into(),
                version: 1,
            },
            FlowCallbackCodeV1 {
                stable_id: "careops-ed.transit.receipt".into(),
                version: 1,
            },
        )
        .unwrap();
        codecs
    }

    #[test]
    fn same_runtime_wrong_carrier_cannot_rebind_service_owner() {
        let bounds = FoundationBounds {
            patient_owners: 2,
            services: 2,
            requests: 2,
            preempt_capable_requests: 0,
            releases_or_cancels: 0,
            parent_markers: 0,
        };
        let plan = AdapterPlan::validate(vec![], vec![], 4, bounds).unwrap();
        let mut adapter = EdFlowAdapter::new_with_transit(
            plan,
            TransitBounds {
                transits: 2,
                total_route_segments: 0,
                controls: 1,
                native_snapshot_limits: TransitContextCheckpointLimitsV1::new(0, 4096, 16, 8192),
            },
        )
        .unwrap();
        let node = NodeId::new(1);
        let graph = Arc::new(TransitGraphV1::new(1, vec![node], vec![]).unwrap());
        let profile = MovementProfile::new("walk", 1).unwrap();
        let route = graph.route(node, node, &profile, 1).unwrap();
        let make_service = |patient: &str| FrozenService {
            identity: ServiceIdentity {
                patient_id: patient.into(),
                episode_id: format!("{patient}-ep"),
                stage_index: 0,
                segment_index: 0,
            },
            original_ticks: 1,
            useful_before_ticks: 0,
            remaining_ticks: 1,
            origin: SampleOrigin::Fixed,
        };
        let first_service = adapter.create_service(make_service("owner-a")).unwrap();
        let first = adapter
            .begin_transit(
                &first_service,
                TransitInput {
                    route_id: "zero".into(),
                    graph: Arc::clone(&graph),
                    route: route.clone(),
                    start_tick: 0,
                },
            )
            .unwrap();
        let second_service = adapter.create_service(make_service("owner-b")).unwrap();
        let second = adapter
            .begin_transit(
                &second_service,
                TransitInput {
                    route_id: "zero".into(),
                    graph,
                    route,
                    start_tick: 0,
                },
            )
            .unwrap();
        let forged = TransitHandle {
            runtime: first.runtime.clone(),
            service: first.service,
            carrier: second.carrier,
        };
        let snapshot_before = adapter.snapshot();
        let transit_before = adapter.transit_snapshot(&first).unwrap();
        assert!(matches!(
            adapter.transit_snapshot(&forged),
            Err(AdapterError::ForeignHandle)
        ));
        assert!(matches!(
            adapter.transit_arrival(&forged),
            Err(AdapterError::ForeignHandle)
        ));
        assert!(matches!(
            adapter.schedule_transit_control(&forged, FlowDomainControl::Pause, 1),
            Err(AdapterError::ForeignHandle)
        ));
        assert_eq!(adapter.snapshot(), snapshot_before);
        assert_eq!(adapter.transit_snapshot(&first).unwrap(), transit_before);
    }

    #[test]
    fn arrived_stale_domain_changes_only_native_dispatch_accounting() {
        let bounds = FoundationBounds {
            patient_owners: 1,
            services: 1,
            requests: 1,
            preempt_capable_requests: 0,
            releases_or_cancels: 0,
            parent_markers: 1,
        };
        let plan = AdapterPlan::validate(
            vec![ResourceSpec {
                id: "room".into(),
                capacity: 1,
            }],
            vec![],
            10,
            bounds,
        )
        .unwrap();
        let mut adapter = EdFlowAdapter::new_with_transit(
            plan,
            TransitBounds {
                transits: 1,
                total_route_segments: 2,
                controls: 2,
                native_snapshot_limits: TransitContextCheckpointLimitsV1::new(16, 4096, 16, 8192),
            },
        )
        .unwrap();
        let nodes = vec![NodeId::new(1), NodeId::new(2), NodeId::new(3)];
        let graph = Arc::new(
            TransitGraphV1::new(
                1,
                nodes.clone(),
                vec![
                    TransitEdge {
                        id: EdgeId::new(1),
                        from: nodes[0],
                        to: nodes[1],
                        length_mm: 2_000,
                        allowed_modes: vec![MovementModeId::new("walk").unwrap()],
                    },
                    TransitEdge {
                        id: EdgeId::new(2),
                        from: nodes[1],
                        to: nodes[2],
                        length_mm: 2_000,
                        allowed_modes: vec![MovementModeId::new("walk").unwrap()],
                    },
                ],
            )
            .unwrap(),
        );
        let route = graph
            .route(
                nodes[0],
                nodes[2],
                &MovementProfile::new("walk", 1_000).unwrap(),
                1,
            )
            .unwrap();
        assert_eq!(route.duration().ticks(), 4);
        let service = adapter
            .create_service(FrozenService {
                identity: ServiceIdentity {
                    patient_id: "checkpoint-patient".into(),
                    episode_id: "checkpoint-episode".into(),
                    stage_index: 0,
                    segment_index: 0,
                },
                original_ticks: 1,
                useful_before_ticks: 0,
                remaining_ticks: 1,
                origin: SampleOrigin::Fixed,
            })
            .unwrap();
        let transit = adapter
            .begin_transit(
                &service,
                TransitInput {
                    route_id: "checkpoint-route".into(),
                    graph: Arc::clone(&graph),
                    route,
                    start_tick: 0,
                },
            )
            .unwrap();
        let start = adapter.step().unwrap().unwrap();
        assert_eq!(start.native.at, SimTime::ZERO);
        let progress = adapter.step().unwrap().unwrap();
        assert_eq!(progress.native.at, SimTime::from_ticks(2));
        let old_domain = adapter.peek_next().unwrap().id;
        assert_eq!(adapter.peek_next().unwrap().at, SimTime::from_ticks(4));
        adapter
            .schedule_transit_control(&transit, FlowDomainControl::Pause, 4)
            .unwrap();
        adapter
            .schedule_transit_control(&transit, FlowDomainControl::Resume, 4)
            .unwrap();
        let mut arrival_dispatch = None;
        while adapter.transit_arrival(&transit).unwrap().is_none() {
            let dispatch = adapter.step().unwrap().unwrap();
            if adapter.transit_arrival(&transit).unwrap().is_some() {
                arrival_dispatch = Some(dispatch);
            }
        }
        let arrival_dispatch = arrival_dispatch.unwrap();
        assert_eq!(arrival_dispatch.native.error, None);
        assert!(arrival_dispatch.native.records.is_empty());
        assert!(arrival_dispatch
            .native
            .callback_batches
            .iter()
            .any(|batch| matches!(
                batch,
                kairo_ecs_des::FlowBatchReceipt::Accepted(rows) if rows.is_empty()
            )));
        assert!(
            pending_transit_arrival_handoff(&adapter.flow, transit.carrier)
                .unwrap()
                .is_some()
        );
        assert!(observe_transit_arrival_handoff(
            &adapter.flow,
            transit.carrier,
            &arrival_dispatch.native
        )
        .unwrap()
        .is_some());

        let unrelated_marker = adapter
            .schedule_marker("private-unrelated", 6, ParentPhase::Decision)
            .unwrap();
        let codecs = transit_codecs(&adapter, Arc::clone(&graph));
        let before = adapter
            .flow
            .capture_checkpoint_v4(&codecs, FlowCheckpointLimits::default())
            .unwrap();
        assert!(before
            .scheduler
            .entries
            .iter()
            .any(|entry| entry.id == old_domain));
        assert!(before.commands.iter().any(|(event, command)| {
            *event == old_domain
                && matches!(command, kairo_ecs_des::FlowCommandV3::Domain(work, kind)
                    if *work == transit.carrier && *kind == TRANSIT_KIND)
        }));
        let stale = adapter.step().unwrap().unwrap();
        assert_eq!(stale.native.event, old_domain);
        assert!(stale.native.error.is_none());
        assert!(stale.native.records.is_empty());
        let after = adapter
            .flow
            .capture_checkpoint_v4(&codecs, FlowCheckpointLimits::default())
            .unwrap();

        let mut expected = before.clone();
        expected.scheduler.now = stale.native.at;
        expected.scheduler.dispatched_events += 1;
        expected
            .scheduler
            .entries
            .retain(|entry| entry.id != old_domain);
        expected.commands.retain(|(event, _)| *event != old_domain);
        expected.next_batch_identity += 1;
        expected.budget_tick = Some(stale.native.at);
        expected.budget_consumed = if before.budget_tick == Some(stale.native.at) {
            before.budget_consumed
        } else {
            0
        } + 1;
        assert!(!after.commands.iter().any(|(event, _)| *event == old_domain));
        assert_eq!(after, expected);

        let unrelated_dispatch = adapter.step().unwrap().unwrap();
        assert_eq!(unrelated_dispatch.native.event, unrelated_marker);
        assert_eq!(unrelated_dispatch.native.at, SimTime::from_ticks(6));
        assert!(observe_transit_arrival_handoff(
            &adapter.flow,
            transit.carrier,
            &unrelated_dispatch.native
        )
        .unwrap()
        .is_none());

        let mut private_state = TransitAdapterState::new(TransitBounds {
            transits: 1,
            total_route_segments: 2,
            controls: 2,
            native_snapshot_limits: TransitContextCheckpointLimitsV1::new(16, 4096, 16, 8192),
        });
        private_state.entries.insert(
            service.work_id(),
            TransitEntry {
                identity: service.identity().clone(),
                _route_id: "checkpoint-route".into(),
                graph,
                carrier: transit.carrier,
                start_event: start.native.event,
                controls: BTreeSet::new(),
                native_events: BTreeSet::new(),
                observed_event: None,
            },
        );
        let mut error_marked = arrival_dispatch.native.clone();
        error_marked.error = Some(FlowError::InvalidWork);
        assert!(private_state
            .observe(&adapter.flow, &error_marked, &[])
            .is_err());
        assert_eq!(
            private_state.entries[&service.work_id()].observed_event,
            None
        );
        let unrelated = private_state
            .observe(&adapter.flow, &unrelated_dispatch.native, &[])
            .unwrap();
        assert!(unrelated.is_empty());
        assert_eq!(
            private_state.entries[&service.work_id()].observed_event,
            None
        );
        let actual = private_state
            .observe(&adapter.flow, &arrival_dispatch.native, &[])
            .unwrap();
        assert_eq!(actual.len(), 1);
        assert!(matches!(
            actual[0],
            FoundationObservation::Transit {
                arrival: Some(_),
                ..
            }
        ));
        assert_eq!(
            private_state.entries[&service.work_id()].observed_event,
            Some(arrival_dispatch.native.event)
        );

        let request = adapter
            .admit_service(
                &service,
                Admission {
                    resource_id: "room".into(),
                    key: QueueKey::from_request(4, "checkpoint-patient".into()).unwrap(),
                    urgency: 1,
                    can_preempt: false,
                    preemptible: false,
                },
            )
            .unwrap();
        assert!(
            pending_transit_arrival_handoff(&adapter.flow, transit.carrier)
                .unwrap()
                .is_none()
        );
        assert!(observe_transit_arrival_handoff(
            &adapter.flow,
            transit.carrier,
            &arrival_dispatch.native
        )
        .unwrap()
        .is_none());
        adapter.flush_dirty().unwrap();
        let mut post_admission = Vec::new();
        while let Some(dispatch) = adapter.step().unwrap() {
            post_admission.push(dispatch);
        }
        assert!(post_admission
            .iter()
            .any(|dispatch| dispatch.native.records.iter().any(|record| {
                record.request == request.request_id()
                    && record.snapshot.work == Some(service.work_id())
                    && record.transition == LifecycleTransition::Granted
            })));
        assert!(post_admission
            .iter()
            .any(|dispatch| dispatch.native.records.iter().any(|record| {
                record.request == request.request_id()
                    && record.snapshot.work == Some(service.work_id())
                    && record.transition == LifecycleTransition::Completed
            })));
    }
}
