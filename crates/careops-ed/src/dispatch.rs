//! Pure, validated selection of one ready request and one available staff member.
//!
//! This module does not advance time, create events, draw randomness, or mutate
//! allocation state. Callers rebuild the validated snapshot after a claim.

use std::collections::{BTreeMap, BTreeSet};

const MAX_REQUESTS: usize = 100_000;
const MAX_STAFF: usize = 1_024;
const MAX_SKILLS: usize = 32;
const MAX_CALENDAR_INTERVALS: usize = 1_024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalendarInterval {
    pub start_tick: u64,
    pub end_tick: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchRequestInput {
    pub request_id: String,
    pub ready_tick: u64,
    pub assigned_zone: String,
    pub required_skills: Vec<String>,
    pub urgency: i32,
    pub required_staff_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffSnapshotInput {
    pub staff_id: String,
    pub assigned_zone: String,
    pub skills: Vec<String>,
    pub shifts: Vec<CalendarInterval>,
    pub breaks: Vec<CalendarInterval>,
    pub idle: bool,
}

/// A snapshot that has passed the full dispatch input contract.
///
/// Its fields are private, so a caller cannot bypass validation:
///
/// ```compile_fail
/// use careops_ed::dispatch::ValidatedDispatchSnapshot;
/// let _unchecked = ValidatedDispatchSnapshot { requests: &[], staff: &[], now: 0, order: vec![], zone_masks: Default::default(), skill_masks: Default::default() };
/// ```
pub struct ValidatedDispatchSnapshot<'a> {
    requests: &'a [DispatchRequestInput],
    staff: &'a [StaffSnapshotInput],
    now: u64,
    order: Vec<usize>,
    zone_masks: BTreeMap<&'a str, Vec<u64>>,
    skill_masks: BTreeMap<&'a str, Vec<u64>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DispatchSelection<'a> {
    pub request_id: &'a str,
    pub staff_id: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DispatchValidationCategory {
    Cardinality,
    Horizon,
    CurrentTime,
    Identifier,
    DuplicateIdentifier,
    ReadyTime,
    Skills,
    UnsupportedStaffCount,
    CalendarInterval,
    CalendarOverlap,
    BreakOutsideShift,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchValidationError {
    pub category: DispatchValidationCategory,
    pub path: String,
}

fn error(category: DispatchValidationCategory, path: impl Into<String>) -> DispatchValidationError {
    DispatchValidationError {
        category,
        path: path.into(),
    }
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty() && value.len() <= 64 && value.is_ascii()
}

fn validate_skills(skills: &[String], path: &str) -> Result<(), DispatchValidationError> {
    if skills.len() > MAX_SKILLS {
        return Err(error(DispatchValidationCategory::Skills, path));
    }
    let mut previous: Option<&str> = None;
    for skill in skills {
        if !valid_identifier(skill) || previous.is_some_and(|value| value >= skill.as_str()) {
            return Err(error(DispatchValidationCategory::Skills, path));
        }
        previous = Some(skill);
    }
    Ok(())
}

fn normalized_calendar(
    intervals: &[CalendarInterval],
    staff_index: usize,
    field: &str,
) -> Result<Vec<CalendarInterval>, DispatchValidationError> {
    for (interval_index, interval) in intervals.iter().enumerate() {
        if interval.start_tick >= interval.end_tick {
            return Err(error(
                DispatchValidationCategory::CalendarInterval,
                format!("staff[{staff_index}].{field}[{interval_index}]"),
            ));
        }
    }
    let mut normalized = intervals.to_vec();
    normalized.sort_by_key(|interval| (interval.start_tick, interval.end_tick));
    if normalized
        .windows(2)
        .any(|pair| pair[1].start_tick < pair[0].end_tick)
    {
        return Err(error(
            DispatchValidationCategory::CalendarOverlap,
            format!("staff[{staff_index}].{field}"),
        ));
    }
    Ok(normalized)
}

/// Validate all rows and build deterministic indexes for selection at `now`.
pub fn validate_dispatch_snapshot<'a>(
    now: u64,
    horizon: u64,
    requests: &'a [DispatchRequestInput],
    staff: &'a [StaffSnapshotInput],
) -> Result<ValidatedDispatchSnapshot<'a>, DispatchValidationError> {
    if requests.len() > MAX_REQUESTS || staff.len() > MAX_STAFF {
        return Err(error(
            DispatchValidationCategory::Cardinality,
            if requests.len() > MAX_REQUESTS {
                "requests"
            } else {
                "staff"
            },
        ));
    }
    if horizon == 0 {
        return Err(error(DispatchValidationCategory::Horizon, "horizon"));
    }
    if now > horizon {
        return Err(error(DispatchValidationCategory::CurrentTime, "now"));
    }

    let mut request_ids = BTreeSet::new();
    for (index, request) in requests.iter().enumerate() {
        if !valid_identifier(&request.request_id) {
            return Err(error(
                DispatchValidationCategory::Identifier,
                format!("requests[{index}].request_id"),
            ));
        }
        if !request_ids.insert(request.request_id.as_str()) {
            return Err(error(
                DispatchValidationCategory::DuplicateIdentifier,
                format!("requests[{index}].request_id"),
            ));
        }
    }
    let mut staff_ids = BTreeSet::new();
    for (index, member) in staff.iter().enumerate() {
        if !valid_identifier(&member.staff_id) {
            return Err(error(
                DispatchValidationCategory::Identifier,
                format!("staff[{index}].staff_id"),
            ));
        }
        if !staff_ids.insert(member.staff_id.as_str()) {
            return Err(error(
                DispatchValidationCategory::DuplicateIdentifier,
                format!("staff[{index}].staff_id"),
            ));
        }
    }

    for (index, request) in requests.iter().enumerate() {
        if request.ready_tick >= horizon {
            return Err(error(
                DispatchValidationCategory::ReadyTime,
                format!("requests[{index}].ready_tick"),
            ));
        }
        if !valid_identifier(&request.assigned_zone) {
            return Err(error(
                DispatchValidationCategory::Identifier,
                format!("requests[{index}].assigned_zone"),
            ));
        }
        validate_skills(
            &request.required_skills,
            &format!("requests[{index}].required_skills"),
        )?;
        if request.required_staff_count != 1 {
            return Err(error(
                DispatchValidationCategory::UnsupportedStaffCount,
                format!("requests[{index}].required_staff_count"),
            ));
        }
    }

    let mut calendars = Vec::with_capacity(staff.len());
    for (index, member) in staff.iter().enumerate() {
        if !valid_identifier(&member.assigned_zone) {
            return Err(error(
                DispatchValidationCategory::Identifier,
                format!("staff[{index}].assigned_zone"),
            ));
        }
        validate_skills(&member.skills, &format!("staff[{index}].skills"))?;
        if member.shifts.len().saturating_add(member.breaks.len()) > MAX_CALENDAR_INTERVALS {
            return Err(error(
                DispatchValidationCategory::Cardinality,
                format!("staff[{index}]"),
            ));
        }
        let shifts = normalized_calendar(&member.shifts, index, "shifts")?;
        let breaks = normalized_calendar(&member.breaks, index, "breaks")?;
        for (break_index, rest) in member.breaks.iter().enumerate() {
            if !shifts
                .iter()
                .any(|shift| rest.start_tick >= shift.start_tick && rest.end_tick <= shift.end_tick)
            {
                return Err(error(
                    DispatchValidationCategory::BreakOutsideShift,
                    format!("staff[{index}].breaks[{break_index}]"),
                ));
            }
        }
        calendars.push((shifts, breaks));
    }

    let mut order: Vec<usize> = (0..staff.len()).collect();
    order.sort_by(|left, right| staff[*left].staff_id.cmp(&staff[*right].staff_id));
    let words = staff.len().div_ceil(64);
    let mut zone_masks: BTreeMap<&'a str, Vec<u64>> = BTreeMap::new();
    let mut skill_masks: BTreeMap<&'a str, Vec<u64>> = BTreeMap::new();
    for (sorted_index, &staff_index) in order.iter().enumerate() {
        let member = &staff[staff_index];
        let (shifts, breaks) = &calendars[staff_index];
        let on_shift = shifts
            .iter()
            .any(|interval| interval.start_tick <= now && now < interval.end_tick);
        let on_break = breaks
            .iter()
            .any(|interval| interval.start_tick <= now && now < interval.end_tick);
        if member.idle && on_shift && !on_break {
            let bit = 1_u64 << (sorted_index % 64);
            zone_masks
                .entry(member.assigned_zone.as_str())
                .or_insert_with(|| vec![0; words])[sorted_index / 64] |= bit;
            for skill in &member.skills {
                skill_masks
                    .entry(skill.as_str())
                    .or_insert_with(|| vec![0; words])[sorted_index / 64] |= bit;
            }
        }
    }
    Ok(ValidatedDispatchSnapshot {
        requests,
        staff,
        now,
        order,
        zone_masks,
        skill_masks,
    })
}

/// Select the highest-priority ready request with an eligible staff member.
pub fn select_dispatch<'a>(
    snapshot: &ValidatedDispatchSnapshot<'a>,
) -> Option<DispatchSelection<'a>> {
    let mut best: Option<(&DispatchRequestInput, usize)> = None;
    for request in snapshot
        .requests
        .iter()
        .filter(|request| request.ready_tick <= snapshot.now)
    {
        if request
            .required_skills
            .iter()
            .any(|skill| !snapshot.skill_masks.contains_key(skill.as_str()))
        {
            continue;
        }
        let Some(zone_mask) = snapshot.zone_masks.get(request.assigned_zone.as_str()) else {
            continue;
        };
        let mut selected_staff = None;
        'candidate: for word_index in 0..zone_mask.len() {
            let mut bits = zone_mask[word_index];
            for skill in &request.required_skills {
                let Some(skill_mask) = snapshot.skill_masks.get(skill.as_str()) else {
                    continue 'candidate;
                };
                bits &= skill_mask[word_index];
                if bits == 0 {
                    continue 'candidate;
                }
            }
            if bits != 0 {
                let index = word_index * 64 + bits.trailing_zeros() as usize;
                selected_staff = snapshot.order.get(index).copied();
                break;
            }
        }
        let Some(staff_index) = selected_staff else {
            continue;
        };
        let replace = best.is_none_or(|(current, _)| {
            request.urgency > current.urgency
                || (request.urgency == current.urgency && request.ready_tick < current.ready_tick)
                || (request.urgency == current.urgency
                    && request.ready_tick == current.ready_tick
                    && request.request_id < current.request_id)
        });
        if replace {
            best = Some((request, staff_index));
        }
    }
    best.map(|(request, staff_index)| DispatchSelection {
        request_id: request.request_id.as_str(),
        staff_id: snapshot.staff[staff_index].staff_id.as_str(),
    })
}
