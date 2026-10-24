//! Policy clocks, administrative pauses, zero-target breach evaluation, and stale event detection (CC-06).

use std::cmp::Ordering;
use serde::{Deserialize, Serialize};

use super::error::{PathwayError, PathwayErrorKind};

/// Administrative policy clock tracking operational target compliance.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PolicyClock {
    pub clock_id: String,
    pub target_duration_ticks: u64,
    pub elapsed_ticks: u64,
    pub is_paused: bool,
}

impl PolicyClock {
    pub fn new(clock_id: impl Into<String>, target_duration_ticks: u64) -> Self {
        Self {
            clock_id: clock_id.into(),
            target_duration_ticks,
            elapsed_ticks: 0,
            is_paused: false,
        }
    }

    /// Advances the clock by `ticks` if not currently paused.
    pub fn advance(&mut self, ticks: u64) {
        if !self.is_paused {
            self.elapsed_ticks += ticks;
        }
    }

    pub fn pause(&mut self) {
        self.is_paused = true;
    }

    pub fn resume(&mut self) {
        self.is_paused = false;
    }

    /// Evaluates target breach ratio without division by zero.
    ///
    /// - For target > 0: `elapsed / target`.
    /// - For target == 0 (immediate service):
    ///   - If elapsed == 0: 0.0 (no breach).
    ///   - If elapsed > 0: INFINITY (instantaneous breach).
    pub fn breach_ratio(&self) -> f64 {
        if self.target_duration_ticks == 0 {
            if self.elapsed_ticks == 0 {
                0.0
            } else {
                f64::INFINITY
            }
        } else {
            self.elapsed_ticks as f64 / self.target_duration_ticks as f64
        }
    }

    /// Returns `true` if the target threshold has been exceeded.
    pub fn is_breached(&self) -> bool {
        if self.target_duration_ticks == 0 {
            self.elapsed_ticks > 0
        } else {
            self.elapsed_ticks > self.target_duration_ticks
        }
    }
}

/// Scheduled simulation event with deterministic tie-breaking and stale event invalidation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduledEvent {
    pub event_id: u64,
    pub tick: u64,
    pub priority: u8, // lower number = higher priority (0 is highest)
    pub patient_id: u64,
    pub generation: u32,
    pub description: String,
}

impl ScheduledEvent {
    /// Checks whether this event is stale because patient state advanced to a new generation.
    pub fn is_stale(&self, current_generation: u32) -> bool {
        self.generation < current_generation
    }

    /// Validates and executes an event, failing closed if stale.
    pub fn validate_execution(&self, current_generation: u32) -> Result<(), PathwayError> {
        if self.is_stale(current_generation) {
            return Err(PathwayError::new(
                PathwayErrorKind::StaleEvent,
                format!(
                    "event {} (gen {}) is stale; current generation is {}",
                    self.event_id, self.generation, current_generation
                ),
            ));
        }
        Ok(())
    }
}

impl Ord for ScheduledEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        // Deterministic multi-key tie breaking: tick -> priority -> event_id -> patient_id
        self.tick
            .cmp(&other.tick)
            .then_with(|| self.priority.cmp(&other.priority))
            .then_with(|| self.event_id.cmp(&other.event_id))
            .then_with(|| self.patient_id.cmp(&other.patient_id))
    }
}

impl PartialOrd for ScheduledEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
