//! Policy clocks, administrative pauses, zero-target breach evaluation, and stale event detection (CC-06).

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

use super::error::{PathwayError, PathwayErrorKind};

/// Explicit lifecycle for a versioned administrative target clock.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyClockState {
    NotStarted,
    Running,
    Paused,
    Stopped,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyClockEventKind {
    Started,
    Advanced,
    Paused,
    Resumed,
    Stopped,
    Reassessed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyClockEvent {
    pub tick: u64,
    pub kind: PolicyClockEventKind,
    pub policy_version: String,
    pub target_duration_ticks: u64,
}

/// A versioned clock with explicit lifecycle and an append-only event history.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionedPolicyClock {
    clock_id: String,
    policy_version: String,
    target_duration_ticks: u64,
    elapsed_ticks: u64,
    pausable: bool,
    state: PolicyClockState,
    last_tick: u64,
    events: Vec<PolicyClockEvent>,
}

impl VersionedPolicyClock {
    pub fn new(
        clock_id: impl Into<String>,
        policy_version: impl Into<String>,
        target_duration_ticks: u64,
        pausable: bool,
    ) -> Self {
        Self {
            clock_id: clock_id.into(),
            policy_version: policy_version.into(),
            target_duration_ticks,
            elapsed_ticks: 0,
            pausable,
            state: PolicyClockState::NotStarted,
            last_tick: 0,
            events: Vec::new(),
        }
    }
    fn record(&mut self, tick: u64, kind: PolicyClockEventKind) {
        self.last_tick = tick;
        self.events.push(PolicyClockEvent {
            tick,
            kind,
            policy_version: self.policy_version.clone(),
            target_duration_ticks: self.target_duration_ticks,
        });
    }
    pub fn clock_id(&self) -> &str {
        &self.clock_id
    }
    pub fn policy_version(&self) -> &str {
        &self.policy_version
    }
    pub fn target_duration_ticks(&self) -> u64 {
        self.target_duration_ticks
    }
    pub fn elapsed_ticks(&self) -> u64 {
        self.elapsed_ticks
    }
    pub fn pausable(&self) -> bool {
        self.pausable
    }
    pub fn state(&self) -> PolicyClockState {
        self.state
    }
    pub fn last_tick(&self) -> u64 {
        self.last_tick
    }
    pub fn events(&self) -> &[PolicyClockEvent] {
        &self.events
    }
    fn check_tick(&self, tick: u64) -> Result<(), PathwayError> {
        if tick < self.last_tick {
            Err(PathwayError::new(
                PathwayErrorKind::InvalidTransition,
                "policy clock time cannot move backwards",
            ))
        } else {
            Ok(())
        }
    }
    pub fn start(&mut self, tick: u64) -> Result<(), PathwayError> {
        self.check_tick(tick)?;
        if self.state != PolicyClockState::NotStarted {
            return Err(PathwayError::new(
                PathwayErrorKind::InvalidTransition,
                "policy clock can only start once",
            ));
        }
        self.state = PolicyClockState::Running;
        self.record(tick, PolicyClockEventKind::Started);
        Ok(())
    }
    pub fn advance_to(&mut self, tick: u64) -> Result<(), PathwayError> {
        self.check_tick(tick)?;
        if !matches!(
            self.state,
            PolicyClockState::Running | PolicyClockState::Paused
        ) {
            return Err(PathwayError::new(
                PathwayErrorKind::InvalidTransition,
                "only an active policy clock can advance",
            ));
        }
        if self.state == PolicyClockState::Running {
            self.elapsed_ticks = self.elapsed_ticks.saturating_add(tick - self.last_tick);
        }
        self.record(tick, PolicyClockEventKind::Advanced);
        Ok(())
    }
    pub fn pause(&mut self, tick: u64) -> Result<(), PathwayError> {
        self.check_tick(tick)?;
        if !self.pausable || self.state != PolicyClockState::Running {
            return Err(PathwayError::new(
                PathwayErrorKind::InvalidTransition,
                "policy clock cannot be paused",
            ));
        }
        self.advance_to(tick)?;
        self.state = PolicyClockState::Paused;
        self.record(tick, PolicyClockEventKind::Paused);
        Ok(())
    }
    pub fn resume(&mut self, tick: u64) -> Result<(), PathwayError> {
        self.check_tick(tick)?;
        if self.state != PolicyClockState::Paused {
            return Err(PathwayError::new(
                PathwayErrorKind::InvalidTransition,
                "only a paused policy clock can resume",
            ));
        }
        self.state = PolicyClockState::Running;
        self.record(tick, PolicyClockEventKind::Resumed);
        Ok(())
    }
    pub fn stop(&mut self, tick: u64) -> Result<(), PathwayError> {
        self.advance_to(tick)?;
        self.state = PolicyClockState::Stopped;
        self.record(tick, PolicyClockEventKind::Stopped);
        Ok(())
    }
    pub fn reassess(
        &mut self,
        tick: u64,
        policy_version: impl Into<String>,
        target_duration_ticks: u64,
    ) -> Result<(), PathwayError> {
        self.advance_to(tick)?;
        if self.state == PolicyClockState::Stopped {
            return Err(PathwayError::new(
                PathwayErrorKind::InvalidTransition,
                "stopped policy clock cannot be reassessed",
            ));
        }
        self.policy_version = policy_version.into();
        self.target_duration_ticks = target_duration_ticks;
        self.record(tick, PolicyClockEventKind::Reassessed);
        Ok(())
    }
    pub fn is_breached(&self) -> bool {
        if self.target_duration_ticks == 0 {
            self.elapsed_ticks > 0
        } else {
            self.elapsed_ticks > self.target_duration_ticks
        }
    }
}

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
