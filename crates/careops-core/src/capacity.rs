//! Neutral capacity accounting and UTC calendar boundaries.
//!
//! These snapshots validate and report aggregate capacity. They do not grant
//! allocation leases, resolve clinical eligibility, or expand local calendars.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CapacityError {
    InvalidCapacity(&'static str),
    InvalidTransition(&'static str),
    InvalidIdentifier,
    InvalidInterval,
    OverlappingIntervals,
    ConflictingPool(String),
    IncompatiblePoolAggregate,
    NonexistentLocalTime,
    ArithmeticOverflow,
}

impl Display for CapacityError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCapacity(reason) => write!(f, "invalid capacity snapshot: {reason}"),
            Self::InvalidTransition(reason) => write!(f, "invalid capacity transition: {reason}"),
            Self::InvalidIdentifier => write!(f, "identifier must not be blank"),
            Self::InvalidInterval => write!(f, "UTC interval must have start < end"),
            Self::OverlappingIntervals => write!(f, "calendar intervals must not overlap"),
            Self::ConflictingPool(pool) => {
                write!(f, "conflicting capacity declarations for pool {pool}")
            }
            Self::IncompatiblePoolAggregate => write!(
                f,
                "pool capacities must share a resource unit and snapshot time before aggregation"
            ),
            Self::NonexistentLocalTime => {
                write!(f, "local time was explicitly rejected as nonexistent")
            }
            Self::ArithmeticOverflow => write!(f, "capacity or time arithmetic overflow"),
        }
    }
}

impl Error for CapacityError {}

/// Counts at one instant, in one resource unit. `None` is unknown; `Some(0)`
/// is known zero. Open/staffed are gross ceilings before dispositions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapacitySnapshot {
    pub physical: Option<u64>,
    pub equipped: Option<u64>,
    pub open: Option<u64>,
    pub staffed: Option<u64>,
    pub reserved: u64,
    pub occupied: u64,
    pub cleaning: u64,
    pub unavailable: u64,
}

impl CapacitySnapshot {
    /// Validate inventory containment and exclusive disposition accounting.
    pub fn validate(&self) -> Result<(), CapacityError> {
        if matches!((self.equipped, self.physical), (Some(equipped), Some(physical)) if equipped > physical)
        {
            return Err(CapacityError::InvalidCapacity(
                "equipped units exceed physical units",
            ));
        }
        if matches!((self.open, self.physical), (Some(open), Some(physical)) if open > physical) {
            return Err(CapacityError::InvalidCapacity(
                "open ceiling exceeds physical units",
            ));
        }

        let dispositions = self.disposition_count()?;
        let inventory_ceiling = match (self.physical, self.equipped) {
            (Some(physical), Some(equipped)) => Some(physical.min(equipped)),
            (Some(physical), None) => Some(physical),
            (None, Some(equipped)) => Some(equipped),
            (None, None) => None,
        };
        if inventory_ceiling.is_some_and(|ceiling| dispositions > ceiling) {
            return Err(CapacityError::InvalidCapacity(
                "dispositions exceed known physical/equipped inventory",
            ));
        }
        Ok(())
    }

    /// Return `None` if any effective ceiling is unknown; known zero remains
    /// `Some(0)`. Existing dispositions are subtracted once from gross ceilings.
    pub fn assignable_capacity(&self) -> Result<Option<u64>, CapacityError> {
        self.validate()?;
        let ceilings = [self.physical, self.equipped, self.open, self.staffed];
        let known_ceilings: Vec<u64> = ceilings.into_iter().flatten().collect();
        if known_ceilings.contains(&0) {
            return Ok(Some(0));
        }
        if known_ceilings.len() != ceilings.len() {
            return Ok(None);
        }
        let ceiling = *known_ceilings
            .iter()
            .min()
            .expect("all capacity ceilings were checked as known");
        Ok(Some(ceiling.saturating_sub(self.disposition_count()?)))
    }

    /// Change gross open/staffed ceilings without evicting existing work.
    pub fn with_availability(
        &self,
        open: Option<u64>,
        staffed: Option<u64>,
    ) -> Result<Self, CapacityError> {
        let mut next = self.clone();
        next.open = open;
        next.staffed = staffed;
        next.validate()?;
        Ok(next)
    }

    /// Move accepted reservations into occupied work. This records a lifecycle
    /// transition; it does not arbitrate or grant a new reservation.
    pub fn start_reserved_work(&mut self, units: u64) -> Result<(), CapacityError> {
        let mut next = self.clone();
        next.reserved =
            next.reserved
                .checked_sub(units)
                .ok_or(CapacityError::InvalidTransition(
                    "started work exceeds reserved units",
                ))?;
        next.occupied = next
            .occupied
            .checked_add(units)
            .ok_or(CapacityError::ArithmeticOverflow)?;
        self.commit_transition(next)
    }

    /// Complete occupied work, optionally moving those units into cleaning.
    pub fn complete_occupied_work(
        &mut self,
        units: u64,
        requires_cleaning: bool,
    ) -> Result<(), CapacityError> {
        let mut next = self.clone();
        next.occupied =
            next.occupied
                .checked_sub(units)
                .ok_or(CapacityError::InvalidTransition(
                    "completed work exceeds occupied units",
                ))?;
        if requires_cleaning {
            next.cleaning = next
                .cleaning
                .checked_add(units)
                .ok_or(CapacityError::ArithmeticOverflow)?;
        }
        self.commit_transition(next)
    }

    /// Finish turnover and release cleaning units back to unassigned capacity.
    pub fn finish_cleaning(&mut self, units: u64) -> Result<(), CapacityError> {
        let mut next = self.clone();
        next.cleaning =
            next.cleaning
                .checked_sub(units)
                .ok_or(CapacityError::InvalidTransition(
                    "finished cleaning exceeds cleaning units",
                ))?;
        self.commit_transition(next)
    }

    /// Record units taken out of service for maintenance. This is a state
    /// update, not an allocation request; active dispositions are preserved.
    pub fn mark_unavailable(&mut self, units: u64) -> Result<(), CapacityError> {
        let mut next = self.clone();
        next.unavailable = next
            .unavailable
            .checked_add(units)
            .ok_or(CapacityError::ArithmeticOverflow)?;
        self.commit_transition(next)
    }

    /// Return maintained units to the unassigned set.
    pub fn restore_unavailable(&mut self, units: u64) -> Result<(), CapacityError> {
        let mut next = self.clone();
        next.unavailable =
            next.unavailable
                .checked_sub(units)
                .ok_or(CapacityError::InvalidTransition(
                    "restored units exceed unavailable units",
                ))?;
        self.commit_transition(next)
    }

    /// Positive demand against known-zero capacity is a known shortfall. Zero
    /// demand has zero shortfall even if the capacity snapshot is unknown.
    pub fn unmet_demand(&self, demand: u64) -> Result<Option<u64>, CapacityError> {
        self.validate()?;
        if demand == 0 {
            return Ok(Some(0));
        }
        Ok(self
            .assignable_capacity()?
            .map(|assignable| demand.saturating_sub(assignable)))
    }

    fn commit_transition(&mut self, next: Self) -> Result<(), CapacityError> {
        next.validate()?;
        *self = next;
        Ok(())
    }

    fn disposition_count(&self) -> Result<u64, CapacityError> {
        self.reserved
            .checked_add(self.occupied)
            .and_then(|count| count.checked_add(self.cleaning))
            .and_then(|count| count.checked_add(self.unavailable))
            .ok_or(CapacityError::ArithmeticOverflow)
    }
}

/// A resource member aliases this pool snapshot; it does not add capacity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapacityPoolDeclaration {
    pub pool_id: String,
    pub unit: String,
    pub revision: String,
    pub as_of_utc_seconds: i64,
    pub snapshot: CapacitySnapshot,
}

/// Sum assignable capacity by distinct pool identity. Conflicting aliases fail
/// closed; this function does not arbitrate concurrent allocation requests.
pub fn deduplicated_assignable_capacity(
    declarations: &[CapacityPoolDeclaration],
) -> Result<Option<u64>, CapacityError> {
    let mut pools = BTreeMap::<String, &CapacityPoolDeclaration>::new();
    for declaration in declarations {
        if declaration.pool_id.trim().is_empty()
            || declaration.unit.trim().is_empty()
            || declaration.revision.trim().is_empty()
        {
            return Err(CapacityError::InvalidIdentifier);
        }
        declaration.snapshot.validate()?;
        if let Some(previous) = pools.get(&declaration.pool_id) {
            if previous.unit != declaration.unit
                || previous.revision != declaration.revision
                || previous.as_of_utc_seconds != declaration.as_of_utc_seconds
                || previous.snapshot != declaration.snapshot
            {
                return Err(CapacityError::ConflictingPool(declaration.pool_id.clone()));
            }
        } else {
            pools.insert(declaration.pool_id.clone(), declaration);
        }
    }

    let mut known_total = 0_u64;
    let mut has_unknown = false;
    let mut aggregate_basis: Option<(&str, i64)> = None;
    for declaration in pools.values() {
        let basis = (declaration.unit.as_str(), declaration.as_of_utc_seconds);
        if aggregate_basis.is_some_and(|expected| expected != basis) {
            return Err(CapacityError::IncompatiblePoolAggregate);
        }
        aggregate_basis = Some(basis);
        match declaration.snapshot.assignable_capacity()? {
            Some(capacity) => {
                known_total = known_total
                    .checked_add(capacity)
                    .ok_or(CapacityError::ArithmeticOverflow)?;
            }
            None => has_unknown = true,
        }
    }
    Ok((!has_unknown).then_some(known_total))
}

/// Opaque capability identifiers with set semantics.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CapabilitySet(BTreeSet<String>);

impl CapabilitySet {
    pub fn new<I, S>(capabilities: I) -> Result<Self, CapacityError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut values = BTreeSet::new();
        for capability in capabilities {
            let capability = capability.into();
            if capability.trim().is_empty() {
                return Err(CapacityError::InvalidIdentifier);
            }
            values.insert(capability);
        }
        Ok(Self(values))
    }

    /// Matching is set inclusion only; no clinical or dispatch policy is implied.
    pub fn satisfies(&self, required: &Self) -> bool {
        required.0.is_subset(&self.0)
    }
}

/// A half-open interval of UTC Unix seconds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct UtcInterval {
    start: i64,
    end: i64,
}

impl UtcInterval {
    pub fn new(start: i64, end: i64) -> Result<Self, CapacityError> {
        if end <= start {
            return Err(CapacityError::InvalidInterval);
        }
        Ok(Self { start, end })
    }

    pub fn start(&self) -> i64 {
        self.start
    }

    pub fn end(&self) -> i64 {
        self.end
    }

    pub fn contains(&self, utc_seconds: i64) -> bool {
        self.start <= utc_seconds && utc_seconds < self.end
    }

    pub fn duration_seconds(&self) -> u128 {
        (i128::from(self.end) - i128::from(self.start)) as u128
    }

    pub fn from_resolved_local(
        start: ResolvedLocalInstant,
        end: ResolvedLocalInstant,
    ) -> Result<Self, CapacityError> {
        Self::new(start.to_utc_seconds()?, end.to_utc_seconds()?)
    }
}

/// Sorted, non-overlapping UTC intervals. Adjacent intervals are allowed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Calendar {
    intervals: Vec<UtcInterval>,
}

impl Calendar {
    pub fn new(mut intervals: Vec<UtcInterval>) -> Result<Self, CapacityError> {
        intervals.sort_unstable();
        if intervals.windows(2).any(|pair| pair[1].start < pair[0].end) {
            return Err(CapacityError::OverlappingIntervals);
        }
        Ok(Self { intervals })
    }

    pub fn contains(&self, utc_seconds: i64) -> bool {
        self.intervals
            .binary_search_by(|interval| {
                if utc_seconds < interval.start {
                    std::cmp::Ordering::Greater
                } else if utc_seconds >= interval.end {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            .is_ok()
    }

    pub fn intervals(&self) -> &[UtcInterval] {
        &self.intervals
    }
}

/// Local wall time supplied with an already-resolved UTC offset. Offset seconds
/// follow the conventional local-minus-UTC sign, e.g. UTC-05:00 is -18,000.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResolvedLocalInstant {
    pub local_epoch_seconds: i64,
    pub utc_offset_seconds: i32,
}

impl ResolvedLocalInstant {
    pub fn to_utc_seconds(self) -> Result<i64, CapacityError> {
        self.local_epoch_seconds
            .checked_sub(i64::from(self.utc_offset_seconds))
            .ok_or(CapacityError::ArithmeticOverflow)
    }
}

/// The caller must explicitly resolve or skip/reject nonexistent local time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocalTimeMapping {
    Resolved(ResolvedLocalInstant),
    SkippedNonexistent,
    RejectedNonexistent,
}

pub fn map_local_time(mapping: LocalTimeMapping) -> Result<Option<i64>, CapacityError> {
    match mapping {
        LocalTimeMapping::Resolved(instant) => Ok(Some(instant.to_utc_seconds()?)),
        LocalTimeMapping::SkippedNonexistent => Ok(None),
        LocalTimeMapping::RejectedNonexistent => Err(CapacityError::NonexistentLocalTime),
    }
}
