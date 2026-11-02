//! Configured synthetic staffing over qualified core roster/assignment contracts.
//! StaticTeamReservation is a bounded single-process approximation: clone,
//! validate and commit a finite known duty interval. It is not a queued atomic
//! resource grant, dynamic clinical staffing model or durable checkpoint.

use crate::spaces::SpacePool;
use careops_core::capacity::{Calendar, CapabilitySet, UtcInterval};
use careops_core::staffing::{AssignmentId, BookingId, DutyKind, StaffId, StaffingRegistry};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallWindow {
    pub requested_at: i64,
    pub duty: UtcInterval,
}

#[derive(Clone, Debug)]
pub struct MemberProfile {
    pub staff: StaffId,
    pub revision: String,
    pub roles: CapabilitySet,
    pub roster: Calendar,
    /// Declared breaks and absences, within the effective duty windows.
    pub unavailable: Calendar,
    pub overtime_per_shift_seconds: u64,
    pub call_in_latency_seconds: u64,
    /// None means on-site; Some(empty) means no call-in has been configured.
    pub call_windows: Option<Vec<CallWindow>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RatioProfile {
    pub revision: String,
    pub cohort_unit: String,
    pub patients_per_staff: BTreeMap<String, u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TeamRequest {
    pub booking: BookingId,
    pub patients: u32,
    pub profile: RatioProfile,
    pub interval: UtcInterval,
    pub kind: DutyKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoleAssignment {
    pub role: String,
    pub staff: StaffId,
    pub assignment: AssignmentId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TeamReceipt {
    pub assignments: Vec<RoleAssignment>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdmissionReceipt {
    pub room: u64,
    pub team: TeamReceipt,
}

#[derive(Clone, Debug)]
struct Admission {
    request: TeamRequest,
    requirements: CapabilitySet,
    receipt: AdmissionReceipt,
}

#[derive(Clone, Debug)]
struct Handover {
    incoming: StaffId,
    overlap: UtcInterval,
    next: UtcInterval,
    result: (AssignmentId, AssignmentId),
}

#[derive(Clone, Debug, Default)]
pub struct TeamAllocator {
    registry: StaffingRegistry,
    members: BTreeMap<StaffId, MemberProfile>,
    bookings: BTreeMap<BookingId, (TeamRequest, TeamReceipt)>,
    roles: BTreeMap<AssignmentId, String>,
    admissions: BTreeMap<u64, Admission>,
    handovers: BTreeMap<AssignmentId, Handover>,
    escorts: BTreeMap<AssignmentId, (UtcInterval, AssignmentId)>,
}

impl TeamAllocator {
    pub fn new(profiles: Vec<MemberProfile>) -> Result<Self, String> {
        let mut result = Self::default();
        for profile in profiles {
            if profile.staff.value() == 0
                || profile.revision.trim().is_empty()
                || result.members.contains_key(&profile.staff)
            {
                return Err("duplicate/invalid staff or missing profile revision".into());
            }
            let overtime = i64::try_from(profile.overtime_per_shift_seconds)
                .map_err(|_| "overtime overflow")?;
            let mut windows = Vec::new();
            for interval in profile.roster.intervals() {
                let end = interval
                    .end()
                    .checked_add(overtime)
                    .ok_or("overtime overflow")?;
                windows.push(
                    UtcInterval::new(interval.start(), end)
                        .map_err(|error| format!("{error:?}"))?,
                );
            }
            let effective = Calendar::new(windows).map_err(|error| format!("{error:?}"))?;
            if profile
                .call_windows
                .as_ref()
                .is_some_and(|calls| calls.iter().any(|call| !covers(&effective, call.duty)))
            {
                return Err("call-in duty lies outside effective roster".into());
            }
            result
                .registry
                .register_staff(
                    profile.staff,
                    effective,
                    profile.unavailable.clone(),
                    profile.call_in_latency_seconds,
                )
                .map_err(|error| format!("{error:?}"))?;
            if let Some(calls) = &profile.call_windows {
                for call in calls {
                    result
                        .registry
                        .call_in_available_at(profile.staff, call.requested_at)
                        .map_err(|error| format!("{error:?}"))?;
                }
            }
            result.members.insert(profile.staff, profile);
        }
        Ok(result)
    }

    pub fn registry(&self) -> &StaffingRegistry {
        &self.registry
    }

    /// Verify a retained service admission against this registry's full live
    /// duty claims. A historical receipt alone cannot authorize clinical effects.
    pub fn validate_admission(
        &self,
        episode: u64,
        request: &TeamRequest,
        receipt: &AdmissionReceipt,
    ) -> Result<(), String> {
        let prior = self
            .admissions
            .get(&episode)
            .ok_or("unknown paired admission")?;
        if &prior.request != request
            || &prior.receipt != receipt
            || !self.complete_claims(request, &receipt.team)
        {
            return Err("admission does not own the complete paired duty claims".into());
        }
        Ok(())
    }

    fn complete_claims(&self, request: &TeamRequest, receipt: &TeamReceipt) -> bool {
        receipt.assignments.iter().all(|role| {
            self.registry
                .assignment(role.assignment)
                .is_some_and(|claim| {
                    claim.staff == role.staff
                        && claim.booking == request.booking
                        && claim.interval == request.interval
                        && claim.kind == request.kind
                        && self.roles.get(&role.assignment) == Some(&role.role)
                })
        })
    }

    fn ready(&self, staff: StaffId, interval: UtcInterval) -> bool {
        self.members.get(&staff).is_some_and(|member| {
            member.call_windows.as_ref().is_none_or(|calls| {
                calls.iter().any(|call| {
                    call.duty.start() <= interval.start()
                        && interval.end() <= call.duty.end()
                        && self
                            .registry
                            .call_in_available_at(staff, call.requested_at)
                            .is_ok_and(|ready| ready <= interval.start())
                })
            })
        })
    }

    pub fn allocate(&mut self, request: TeamRequest) -> Result<TeamReceipt, String> {
        if let Some((prior, result)) = self.bookings.get(&request.booking) {
            return if prior == &request {
                Ok(result.clone())
            } else {
                Err("team booking payload conflict".into())
            };
        }
        if request.booking == BookingId::new(0)
            || request.patients == 0
            || request.interval.start() < 0
            || request.profile.revision.trim().is_empty()
            || request.profile.cohort_unit.trim().is_empty()
            || request.profile.patients_per_staff.is_empty()
        {
            return Err("invalid team request or profile identity".into());
        }
        let mut slots = Vec::new();
        for (role, denominator) in &request.profile.patients_per_staff {
            if role.trim().is_empty() || role.trim() != role || *denominator == 0 {
                return Err("invalid configured staffing ratio".into());
            }
            let count = request.patients.div_ceil(*denominator) as usize;
            if count > 64 || slots.len() + count > 64 {
                return Err("static reservation supports at most64 role slots".into());
            }
            slots.extend(std::iter::repeat_n(role.clone(), count));
        }
        let candidates: Vec<Vec<StaffId>> = slots
            .iter()
            .map(|role| {
                let required = CapabilitySet::new([role.clone()]).expect("role was validated");
                self.members
                    .iter()
                    .filter_map(|(id, member)| {
                        if !member.roles.satisfies(&required) || !self.ready(*id, request.interval)
                        {
                            return None;
                        }
                        let mut probe = self.registry.clone();
                        probe
                            .assign(*id, request.booking, request.interval, request.kind)
                            .ok()
                            .map(|_| *id)
                    })
                    .collect()
            })
            .collect();
        let mut matching = BTreeMap::new();
        for slot in 0..slots.len() {
            if !augment(slot, &candidates, &mut BTreeSet::new(), &mut matching) {
                return Err("no complete qualified available team".into());
            }
        }
        let by_slot: BTreeMap<usize, StaffId> = matching
            .into_iter()
            .map(|(staff, slot)| (slot, staff))
            .collect();
        let mut registry = self.registry.clone();
        let mut assignments = Vec::new();
        for (slot, role) in slots.into_iter().enumerate() {
            let staff = by_slot[&slot];
            let assignment = registry
                .assign(staff, request.booking, request.interval, request.kind)
                .map_err(|error| format!("{error:?}"))?;
            assignments.push(RoleAssignment {
                role,
                staff,
                assignment,
            });
        }
        let receipt = TeamReceipt { assignments };
        self.registry = registry;
        for assignment in &receipt.assignments {
            self.roles
                .insert(assignment.assignment, assignment.role.clone());
        }
        self.bookings
            .insert(request.booking, (request, receipt.clone()));
        Ok(receipt)
    }

    /// StaticTeamReservation: start a new space/team admission at the known duty
    /// interval's start. Earlier waiting reservations are outside this helper.
    /// Failed admission leaves both registries unchanged. One booking serves one
    /// episode; unpaired historical room receipts cannot create a new admission.
    pub fn admit(
        &mut self,
        spaces: &mut SpacePool,
        episode: u64,
        requirements: CapabilitySet,
        request: TeamRequest,
    ) -> Result<AdmissionReceipt, String> {
        if let Some(prior) = self.admissions.get(&episode) {
            return if prior.request == request
                && prior.requirements == requirements
                && spaces.matches_started_reservation(
                    episode,
                    &requirements,
                    request.interval.start(),
                    prior.receipt.room,
                ) {
                Ok(prior.receipt.clone())
            } else {
                Err("admission payload conflict".into())
            };
        }
        if self
            .admissions
            .values()
            .any(|prior| prior.request.booking == request.booking)
        {
            return Err("team booking already bound to another episode".into());
        }
        let mut candidate = self.clone();
        let mut rooms = spaces.clone();
        let team = candidate.allocate(request.clone())?;
        // A historical booking receipt is not a new live staffing grant.
        // Every role must still own the whole declared duty before admission.
        if !candidate.complete_claims(&request, &team) {
            return Err("historical team receipt no longer owns complete duty claims".into());
        }
        let room = rooms.reserve(episode, requirements.clone(), request.interval.start())?;
        if !rooms.holds_reservation(episode) {
            return Err("historical room receipt is not a current reservation".into());
        }
        rooms.start(episode, request.interval.start())?;
        let receipt = AdmissionReceipt { room, team };
        candidate.admissions.insert(
            episode,
            Admission {
                request,
                requirements,
                receipt: receipt.clone(),
            },
        );
        *self = candidate;
        *spaces = rooms;
        Ok(receipt)
    }

    pub fn handover(
        &mut self,
        prior: AssignmentId,
        incoming: StaffId,
        overlap: UtcInterval,
        next: UtcInterval,
    ) -> Result<(AssignmentId, AssignmentId), String> {
        if let Some(previous) = self.handovers.get(&prior) {
            return if (previous.incoming, previous.overlap, previous.next)
                == (incoming, overlap, next)
            {
                Ok(previous.result)
            } else {
                Err("handover payload conflict".into())
            };
        }
        let role = self
            .roles
            .get(&prior)
            .ok_or("unknown owned assignment")?
            .clone();
        if matches!(
            self.registry
                .assignment(prior)
                .ok_or("unknown assignment")?
                .kind,
            DutyKind::Escort { .. }
        ) {
            return Err("an escort chain retains its worker until completion".into());
        }
        let required = CapabilitySet::new([role.clone()]).expect("stored role is valid");
        if !self
            .members
            .get(&incoming)
            .is_some_and(|member| member.roles.satisfies(&required))
            || !self.ready(incoming, overlap)
            || !self.ready(incoming, next)
        {
            return Err("incoming worker is not qualified/ready".into());
        }
        let result = self
            .registry
            .handover(prior, incoming, overlap, next)
            .map_err(|error| format!("{error:?}"))?;
        self.roles.insert(result.0, role.clone());
        self.roles.insert(result.1, role);
        self.handovers.insert(
            prior,
            Handover {
                incoming,
                overlap,
                next,
                result,
            },
        );
        Ok(result)
    }

    pub fn continue_escort(
        &mut self,
        prior: AssignmentId,
        next: UtcInterval,
    ) -> Result<AssignmentId, String> {
        if let Some((interval, result)) = self.escorts.get(&prior) {
            return if *interval == next {
                Ok(*result)
            } else {
                Err("escort payload conflict".into())
            };
        }
        let role = self
            .roles
            .get(&prior)
            .ok_or("unknown owned assignment")?
            .clone();
        let staff = self
            .registry
            .assignment(prior)
            .ok_or("unknown assignment")?
            .staff;
        if !self.ready(staff, next) {
            return Err("escort worker is not ready".into());
        }
        let result = self
            .registry
            .continue_escort(prior, next)
            .map_err(|error| format!("{error:?}"))?;
        self.roles.insert(result, role);
        self.escorts.insert(prior, (next, result));
        Ok(result)
    }

    pub fn release_at(&mut self, assignment: AssignmentId, tick: i64) -> Result<(), String> {
        if !self.roles.contains_key(&assignment) {
            return Err("unknown owned assignment".into());
        }
        let current = self
            .registry
            .assignment(assignment)
            .ok_or("unknown assignment")?;
        if self.admissions.values().any(|admission| {
            admission
                .receipt
                .team
                .assignments
                .iter()
                .any(|role| role.assignment == assignment)
                && tick < admission.request.interval.end()
        }) {
            return Err("static admission duty commitment has not completed".into());
        }
        if matches!(current.kind, DutyKind::Escort { .. }) && tick < current.interval.end() {
            return Err("escort commitment has not completed".into());
        }
        if self.handovers.iter().any(|(prior, handover)| {
            ((*prior == assignment || handover.result.0 == assignment)
                && tick < handover.overlap.end())
                || (handover.result.1 == assignment && tick < current.interval.end())
        }) {
            return Err("handover responsibility commitment has not completed".into());
        }
        self.registry
            .release_at(assignment, tick)
            .map_err(|error| format!("{error:?}"))
    }

    /// Verify responsibility at the room pool's actual observation tick. A duty
    /// window expiring while its episode is still occupying a room is a visible
    /// shortfall, never an inferred extension of staff availability. Call after
    /// each event; departure at a boundary precedes this post-event audit.
    pub fn audit_responsibility(&self, spaces: &SpacePool, tick: i64) -> Result<(), String> {
        if spaces.current_tick() != tick {
            return Err("room/team observation ticks differ".into());
        }
        let assignments = self.registry.assignments();
        for (episode, admission) in &self.admissions {
            if !spaces.matches_started_reservation(
                *episode,
                &admission.requirements,
                admission.request.interval.start(),
                admission.receipt.room,
            ) {
                return Err("admission is not paired with this room history".into());
            }
            if !spaces.occupies(*episode) {
                continue;
            }
            for (role, denominator) in &admission.request.profile.patients_per_staff {
                let required = admission.request.patients.div_ceil(*denominator) as usize;
                let active = assignments
                    .iter()
                    .filter(|claim| {
                        claim.booking == admission.request.booking
                            && claim.interval.contains(tick)
                            && self.roles.get(&claim.id) == Some(role)
                    })
                    .count();
                if active < required {
                    return Err("occupied episode has insufficient qualified duty coverage".into());
                }
            }
        }
        Ok(())
    }
}

fn augment(
    slot: usize,
    candidates: &[Vec<StaffId>],
    seen: &mut BTreeSet<StaffId>,
    matching: &mut BTreeMap<StaffId, usize>,
) -> bool {
    for staff in &candidates[slot] {
        if !seen.insert(*staff) {
            continue;
        }
        if matching
            .get(staff)
            .copied()
            .is_none_or(|old| augment(old, candidates, seen, matching))
        {
            matching.insert(*staff, slot);
            return true;
        }
    }
    false
}

fn covers(calendar: &Calendar, target: UtcInterval) -> bool {
    let mut cursor = target.start();
    for interval in calendar.intervals() {
        if interval.end() <= cursor {
            continue;
        }
        if interval.start() > cursor {
            return false;
        }
        cursor = interval.end();
        if cursor >= target.end() {
            return true;
        }
    }
    false
}
