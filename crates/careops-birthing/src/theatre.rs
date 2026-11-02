//! Bounded, single-active theatre service; all times are synthetic UTC seconds.
//! Scheduled callback batches precede new boundary/policy events at the same
//! tick. The coordinator merges global event times and audits responsibility
//! after the complete global tick batch. Transfer never implies delivery.

use crate::{
    spaces::SpacePool,
    team::{RatioProfile, TeamAllocator, TeamRequest},
};
use careops_core::capacity::{CapabilitySet, UtcInterval};
use careops_core::pathway::{InterruptionEligibility, ObservedPolicyState};
use careops_core::staffing::{BookingId, DutyKind};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Targets {
    pub transfer: Option<u64>,
    pub incision: Option<u64>,
    pub birth: Option<u64>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueueRule {
    pub priority_rank: i32,
    pub targets: Targets,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueuePolicy {
    pub revision: String,
    pub jurisdiction_id: String,
    pub scheme_id: String,
    pub scheme_version: String,
    pub source_authority: String,
    pub citation: String,
    pub rules: BTreeMap<u8, QueueRule>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TheatreProfile {
    pub revision: String,
    pub preparation_ticks: u64,
    pub birth_after_incision_ticks: u64,
    pub completion_after_birth_ticks: u64,
    pub cleaning_ticks: u64,
    pub required_capabilities: Vec<String>,
    pub staffing: RatioProfile,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TheatreRequest {
    pub id: u64,
    pub episode: u64,
    pub booking: BookingId,
    pub decision_tick: u64,
    pub cohort_size: u32,
    /// Within this adapter, elapsed_wait_ticks uses the original theatre
    /// decision as its anchor; other domain wait clocks remain separate.
    pub observed: ObservedPolicyState,
    pub policy: QueuePolicy,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Assessment {
    pub observed: ObservedPolicyState,
    pub policy: QueuePolicy,
    pub rule: QueueRule,
    pub interruption: InterruptionEligibility,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetMetric {
    /// Immutable case-assessment foreign key; policy content is retained there.
    pub assessment_tick: u64,
    pub elapsed: u64,
    pub target: Option<u64>,
    pub breached: Option<bool>,
    pub policy_revision: String,
    pub scheme_version: String,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Metrics {
    pub transfer: Option<TargetMetric>,
    pub incision: Option<TargetMetric>,
    pub birth: Option<TargetMetric>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Timings {
    pub accepted: Option<u64>,
    pub departed: Option<u64>,
    pub arrived: Option<u64>,
    pub started: Option<u64>,
    pub incision: Option<u64>,
    pub birth: Option<u64>,
    pub completed: Option<u64>,
    pub cancelled: Option<u64>,
    pub cleaned: Option<u64>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceFailure {
    pub tick: u64,
    pub reason: String,
}
#[derive(Clone, Debug)]
pub struct TheatreCase {
    pub initial: TheatreRequest,
    pub assessments: Vec<Assessment>,
    pub timings: Timings,
    pub metrics: Metrics,
    pub resource_failures: Vec<ResourceFailure>,
    pub cancellation_reason: Option<String>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CallbackKind {
    Accepted,
    TransferDeparted,
    TransferArrived,
    ServiceStarted,
    ProcedureStarted,
    Birth,
    ServiceCompleted,
    RoomReady,
    Cancelled,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Callback {
    pub case_id: u64,
    pub episode: u64,
    pub tick: u64,
    pub kind: CallbackKind,
}
#[derive(Clone, Debug)]
struct Plan {
    id: u64,
    incision: u64,
    birth: u64,
    completion: u64,
    cleaned: u64,
}
#[derive(Clone, Debug)]
pub struct TheatreService {
    profile: TheatreProfile,
    capabilities: CapabilitySet,
    cases: BTreeMap<u64, TheatreCase>,
    active: Option<Plan>,
    cleanups: BTreeMap<u64, u64>,
    events: Vec<Callback>,
    now: u64,
}

impl TheatreService {
    pub fn new(profile: TheatreProfile) -> Result<Self, String> {
        let total = profile
            .preparation_ticks
            .checked_add(profile.birth_after_incision_ticks)
            .and_then(|n| n.checked_add(profile.completion_after_birth_ticks))
            .ok_or("service duration overflow")?;
        let horizon = total
            .checked_add(profile.cleaning_ticks)
            .ok_or("cleaning duration overflow")?;
        if total == 0
            || i64::try_from(horizon).is_err()
            || profile.revision.trim().is_empty()
            || profile.required_capabilities.is_empty()
            || profile.staffing.revision.trim().is_empty()
            || profile.staffing.cohort_unit.trim().is_empty()
            || profile.staffing.patients_per_staff.is_empty()
            || profile.staffing.patients_per_staff.len() > 64
            || profile
                .staffing
                .patients_per_staff
                .iter()
                .any(|(role, n)| role.trim().is_empty() || role.trim() != role || *n == 0)
        {
            return Err("invalid bounded theatre profile".into());
        }
        let capabilities =
            CapabilitySet::new(profile.required_capabilities.clone()).map_err(|e| e.to_string())?;
        Ok(Self {
            profile,
            capabilities,
            cases: BTreeMap::new(),
            active: None,
            cleanups: BTreeMap::new(),
            events: Vec::new(),
            now: 0,
        })
    }
    pub fn profile(&self) -> &TheatreProfile {
        &self.profile
    }
    pub fn case(&self, id: u64) -> Option<&TheatreCase> {
        self.cases.get(&id)
    }
    pub fn events(&self) -> &[Callback] {
        &self.events
    }
    fn policy_consistent(&self, policy: &QueuePolicy) -> bool {
        self.cases
            .values()
            .flat_map(|case| &case.assessments)
            .all(|assessment| {
                let old = &assessment.policy;
                (
                    old.jurisdiction_id.as_str(),
                    old.scheme_id.as_str(),
                    old.scheme_version.as_str(),
                    old.revision.as_str(),
                ) != (
                    policy.jurisdiction_id.as_str(),
                    policy.scheme_id.as_str(),
                    policy.scheme_version.as_str(),
                    policy.revision.as_str(),
                ) || old == policy
            })
    }
    pub fn next_event_tick(&self) -> Option<u64> {
        self.next_due().map(|e| e.0)
    }

    fn external_time(&self, tick: u64) -> Result<(), String> {
        calendar_tick(tick)?;
        if tick < self.now {
            return Err("late theatre event".into());
        }
        if self.next_event_tick().is_some_and(|due| due <= tick) {
            return Err("process due callbacks before external theatre events".into());
        }
        Ok(())
    }
    pub fn submit(&mut self, request: TheatreRequest) -> Result<(), String> {
        if let Some(prior) = self.cases.get(&request.id) {
            return if prior.initial == request {
                Ok(())
            } else {
                Err("theatre request payload conflict".into())
            };
        }
        self.external_time(request.decision_tick)?;
        if !self.policy_consistent(&request.policy) {
            return Err("policy revision content conflict".into());
        }
        let slots: u64 = self
            .profile
            .staffing
            .patients_per_staff
            .values()
            .map(|n| u64::from(request.cohort_size.div_ceil(*n)))
            .sum();
        if request.id == 0
            || request.episode == 0
            || request.booking == BookingId::new(0)
            || request.cohort_size == 0
            || slots > 64
            || self.cases.values().any(|c| {
                c.initial.episode == request.episode || c.initial.booking == request.booking
            })
        {
            return Err("invalid/reused theatre identity or unsupported team size".into());
        }
        let assessment = assess(
            request.observed.clone(),
            request.policy.clone(),
            request.decision_tick,
            request.decision_tick,
        )?;
        let tick = request.decision_tick;
        self.cases.insert(
            request.id,
            TheatreCase {
                initial: request,
                assessments: vec![assessment],
                timings: Timings::default(),
                metrics: Metrics::default(),
                resource_failures: Vec::new(),
                cancellation_reason: None,
            },
        );
        self.now = tick;
        Ok(())
    }
    pub fn accept(&mut self, id: u64, tick: u64) -> Result<(), String> {
        self.boundary(id, tick, CallbackKind::Accepted)
    }
    pub fn transfer_departed(&mut self, id: u64, tick: u64) -> Result<(), String> {
        self.boundary(id, tick, CallbackKind::TransferDeparted)
    }
    pub fn transfer_arrived(&mut self, id: u64, tick: u64) -> Result<(), String> {
        self.boundary(id, tick, CallbackKind::TransferArrived)
    }

    fn boundary(&mut self, id: u64, tick: u64, kind: CallbackKind) -> Result<(), String> {
        let case = self.cases.get(&id).ok_or("unknown theatre case")?;
        let (prior, predecessor) = match kind {
            CallbackKind::Accepted => (case.timings.accepted, Some(case.initial.decision_tick)),
            CallbackKind::TransferDeparted => (case.timings.departed, case.timings.accepted),
            CallbackKind::TransferArrived => (case.timings.arrived, case.timings.departed),
            _ => return Err("not a transfer boundary callback".into()),
        };
        if let Some(at) = prior {
            return if at == tick {
                Ok(())
            } else {
                Err("boundary callback payload conflict".into())
            };
        }
        self.external_time(tick)?;
        if case.timings.cancelled.is_some()
            || case.timings.completed.is_some()
            || predecessor.is_none_or(|at| tick < at)
        {
            return Err("invalid theatre boundary order".into());
        }
        let case = self.cases.get_mut(&id).expect("case exists");
        match kind {
            CallbackKind::Accepted => case.timings.accepted = Some(tick),
            CallbackKind::TransferDeparted => {
                case.timings.departed = Some(tick);
                case.metrics.transfer = Some(metric(case, tick, CallbackKind::TransferDeparted));
            }
            CallbackKind::TransferArrived => case.timings.arrived = Some(tick),
            _ => unreachable!(),
        }
        self.emit(id, tick, kind);
        self.now = tick;
        Ok(())
    }
    pub fn reassess(
        &mut self,
        id: u64,
        observed: ObservedPolicyState,
        policy: QueuePolicy,
    ) -> Result<(), String> {
        let case = self.cases.get(&id).ok_or("unknown theatre case")?;
        let tick = observed.observed_at_tick;
        if !self.policy_consistent(&policy) {
            return Err("policy revision content conflict".into());
        }
        let assessment = assess(observed, policy, case.initial.decision_tick, tick)?;
        if let Some(prior) = case
            .assessments
            .iter()
            .find(|a| a.observed.observed_at_tick == tick)
        {
            return if *prior == assessment {
                Ok(())
            } else {
                Err("conflicting same-tick policy input".into())
            };
        }
        self.external_time(tick)?;
        if case.timings.cancelled.is_some() || case.timings.completed.is_some() {
            return Err("case is closed".into());
        }
        self.cases
            .get_mut(&id)
            .expect("case exists")
            .assessments
            .push(assessment);
        self.now = tick;
        Ok(())
    }
    /// Cancels a queued procedure only. B6 owns transfer compensation/ownership.
    pub fn cancel(&mut self, id: u64, tick: u64, reason: String) -> Result<(), String> {
        let case = self.cases.get(&id).ok_or("unknown theatre case")?;
        if let Some(at) = case.timings.cancelled {
            return if at == tick && case.cancellation_reason.as_ref() == Some(&reason) {
                Ok(())
            } else {
                Err("cancellation payload conflict".into())
            };
        }
        self.external_time(tick)?;
        if reason.trim().is_empty()
            || case.timings.started.is_some()
            || case.timings.completed.is_some()
        {
            return Err("active/closed procedure is non-interruptible".into());
        }
        let case = self.cases.get_mut(&id).expect("case exists");
        case.timings.cancelled = Some(tick);
        case.cancellation_reason = Some(reason);
        self.emit(id, tick, CallbackKind::Cancelled);
        self.now = tick;
        Ok(())
    }
    pub fn start_next(
        &mut self,
        tick: u64,
        rooms: &mut SpacePool,
        team: &mut TeamAllocator,
    ) -> Result<Option<u64>, String> {
        self.external_time(tick)?;
        let mut spatial = rooms.clone();
        spatial.advance_to(calendar_tick(tick)?)?;
        let next = self
            .cases
            .iter()
            .filter(|(_, c)| {
                c.timings.arrived.is_some()
                    && c.timings.started.is_none()
                    && c.timings.cancelled.is_none()
            })
            .min_by_key(|(id, c)| {
                (
                    c.assessments
                        .last()
                        .expect("initial assessment")
                        .rule
                        .priority_rank,
                    c.initial.decision_tick,
                    **id,
                )
            })
            .map(|(id, _)| *id);
        let Some(id) = next else {
            *rooms = spatial;
            self.now = tick;
            return Ok(None);
        };
        if self.active.is_some() {
            self.failure(id, tick, "active non-interruptible procedure".into());
            *rooms = spatial;
            self.now = tick;
            return Ok(None);
        }
        let incision = tick
            .checked_add(self.profile.preparation_ticks)
            .ok_or("incision time overflow")?;
        let birth = incision
            .checked_add(self.profile.birth_after_incision_ticks)
            .ok_or("birth time overflow")?;
        let completion = birth
            .checked_add(self.profile.completion_after_birth_ticks)
            .ok_or("completion time overflow")?;
        let cleaned = completion
            .checked_add(self.profile.cleaning_ticks)
            .ok_or("cleaning time overflow")?;
        calendar_tick(cleaned)?;
        let case = &self.cases[&id];
        let request = TeamRequest {
            booking: case.initial.booking,
            patients: case.initial.cohort_size,
            profile: self.profile.staffing.clone(),
            interval: UtcInterval::new(calendar_tick(tick)?, calendar_tick(completion)?)
                .map_err(|e| e.to_string())?,
            kind: DutyKind::Procedure {
                interruptible: false,
            },
        };
        let mut people = team.clone();
        if let Err(reason) = people.admit(
            &mut spatial,
            case.initial.episode,
            self.capabilities.clone(),
            request,
        ) {
            self.failure(id, tick, reason);
            *rooms = spatial;
            self.now = tick;
            return Ok(None);
        }
        self.cases
            .get_mut(&id)
            .expect("case exists")
            .timings
            .started = Some(tick);
        self.active = Some(Plan {
            id,
            incision,
            birth,
            completion,
            cleaned,
        });
        *rooms = spatial;
        *team = people;
        self.emit(id, tick, CallbackKind::ServiceStarted);
        self.now = tick;
        Ok(Some(id))
    }
    /// Apply scheduled callbacks transactionally. The caller chooses global
    /// event frontiers, consumes Birth callbacks and then audits the full batch.
    pub fn advance_to(
        &mut self,
        tick: u64,
        rooms: &mut SpacePool,
    ) -> Result<Vec<Callback>, String> {
        calendar_tick(tick)?;
        if tick < self.now {
            return Err("late theatre horizon".into());
        }
        let mut candidate = self.clone();
        let mut spatial = rooms.clone();
        let before = candidate.events.len();
        while let Some((at, _, id, kind)) = candidate.next_due().filter(|event| event.0 <= tick) {
            spatial.advance_to(calendar_tick(at)?)?;
            let episode = candidate.cases[&id].initial.episode;
            match kind {
                CallbackKind::ProcedureStarted => {
                    let c = candidate.cases.get_mut(&id).expect("case exists");
                    c.timings.incision = Some(at);
                    c.metrics.incision = Some(metric(c, at, CallbackKind::ProcedureStarted));
                }
                CallbackKind::Birth => {
                    let c = candidate.cases.get_mut(&id).expect("case exists");
                    c.timings.birth = Some(at);
                    c.metrics.birth = Some(metric(c, at, CallbackKind::Birth));
                }
                CallbackKind::ServiceCompleted => {
                    spatial.depart(episode, calendar_tick(at)?)?;
                    candidate
                        .cases
                        .get_mut(&id)
                        .expect("case exists")
                        .timings
                        .completed = Some(at);
                    let plan = candidate.active.take().expect("active completion");
                    candidate.cleanups.insert(id, plan.cleaned);
                }
                CallbackKind::RoomReady => {
                    spatial.finish_cleaning(episode, calendar_tick(at)?)?;
                    candidate
                        .cases
                        .get_mut(&id)
                        .expect("case exists")
                        .timings
                        .cleaned = Some(at);
                    candidate.cleanups.remove(&id);
                }
                _ => unreachable!(),
            }
            candidate.emit(id, at, kind);
        }
        spatial.advance_to(calendar_tick(tick)?)?;
        candidate.now = tick;
        let events = candidate.events[before..].to_vec();
        *self = candidate;
        *rooms = spatial;
        Ok(events)
    }
    fn next_due(&self) -> Option<(u64, u8, u64, CallbackKind)> {
        let active = self.active.as_ref().map(|plan| {
            let c = &self.cases[&plan.id];
            if c.timings.incision.is_none() {
                (plan.incision, 0, plan.id, CallbackKind::ProcedureStarted)
            } else if c.timings.birth.is_none() {
                (plan.birth, 1, plan.id, CallbackKind::Birth)
            } else {
                (plan.completion, 2, plan.id, CallbackKind::ServiceCompleted)
            }
        });
        active
            .into_iter()
            .chain(
                self.cleanups
                    .iter()
                    .map(|(id, at)| (*at, 3, *id, CallbackKind::RoomReady)),
            )
            .min()
    }
    fn emit(&mut self, id: u64, tick: u64, kind: CallbackKind) {
        self.events.push(Callback {
            case_id: id,
            episode: self.cases[&id].initial.episode,
            tick,
            kind,
        });
    }
    fn failure(&mut self, id: u64, tick: u64, reason: String) {
        let list = &mut self
            .cases
            .get_mut(&id)
            .expect("case exists")
            .resource_failures;
        let event = ResourceFailure { tick, reason };
        if list.last() != Some(&event) {
            list.push(event);
        }
    }
}

fn calendar_tick(tick: u64) -> Result<i64, String> {
    i64::try_from(tick).map_err(|_| "tick exceeds calendar range".into())
}
fn assess(
    observed: ObservedPolicyState,
    policy: QueuePolicy,
    decision: u64,
    tick: u64,
) -> Result<Assessment, String> {
    let u = &observed.urgency;
    if tick < decision
        || observed.observed_at_tick != tick
        || observed.elapsed_wait_ticks != tick - decision
        || u.assessed_at_tick > tick
        || [
            &policy.revision,
            &policy.source_authority,
            &policy.citation,
            &u.effective_date,
            &u.source_authority,
            &u.citation,
            &u.jurisdiction_id,
            &u.scheme_id,
            &u.scheme_version,
        ]
        .iter()
        .any(|s| s.trim().is_empty())
        || (
            u.jurisdiction_id.as_str(),
            u.scheme_id.as_str(),
            u.scheme_version.as_str(),
        ) != (
            policy.jurisdiction_id.as_str(),
            policy.scheme_id.as_str(),
            policy.scheme_version.as_str(),
        )
        || u.source_sha256
            .as_ref()
            .is_some_and(|hash| hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()))
    {
        return Err("invalid/future observed policy input or provenance".into());
    }
    let rule = policy
        .rules
        .get(&u.category)
        .ok_or("undefined urgency category")?
        .clone();
    Ok(Assessment {
        observed,
        policy,
        rule,
        interruption: InterruptionEligibility::NotEligible,
    })
}
fn metric(case: &TheatreCase, tick: u64, kind: CallbackKind) -> TargetMetric {
    let assessment = case.assessments.last().expect("initial assessment");
    let target = match kind {
        CallbackKind::TransferDeparted => assessment.rule.targets.transfer,
        CallbackKind::ProcedureStarted => assessment.rule.targets.incision,
        CallbackKind::Birth => assessment.rule.targets.birth,
        _ => unreachable!("only measured callbacks produce target metrics"),
    };
    let elapsed = tick - case.initial.decision_tick;
    TargetMetric {
        assessment_tick: assessment.observed.observed_at_tick,
        elapsed,
        target,
        breached: target.map(|bound| elapsed > bound),
        policy_revision: assessment.policy.revision.clone(),
        scheme_version: assessment.policy.scheme_version.clone(),
    }
}
