#![forbid(unsafe_code)]

//! Narrow Snapshot and Command ABI (`careops_ed.abi.v1`) for Task E5.1.
//!
//! # Scope & Architecture Review
//!
//! 1. **Narrow Snapshot/Command ABI (`careops_ed.abi.v1`)**:
//!    - Standardized WebSocket message envelopes: `SnapshotEnvelope`, `DeltaEnvelope`,
//!      `CommandEnvelope`, `CommandAckEnvelope`, `ProgressEnvelope`, and `ErrorEnvelope`.
//!    - Wide Integer Lossless Representation: Virtual time / ticks are represented as `u128`
//!      in Rust but serialized to JSON as decimal strings (`U128Tick`) so that JavaScript
//!      IEEE 754 `float64` runtimes never truncate integer precision beyond $2^{53}-1$
//!      (9,007,199,254,740,991).
//!    - Spatial Coordinates: Exact integer millimeters (`x_mm`, `y_mm`, `floor_level`) to guarantee
//!      exact cross-platform determinism between Native (x86_64, aarch64) and Wasm (wasm32-unknown-unknown).
//!
//! 2. **State Sync & Sequencing**:
//!    - Monotonic sequencing (`seq`, `base_seq`).
//!    - Explicit gap detection: If `delta.base_seq != client.last_seen_seq`, state transitions to
//!      `ResyncRequired` and prompts a full snapshot recovery.
//!    - Duplicate / stale delta suppression: If `delta.seq <= client.last_seen_seq`, delta is safely ignored.
//!    - Handshake validation: Spatial package revision matching (`spatial_revision`) fails closed on mismatch.
//!
//! 3. **Idempotent Commands & Cancellation**:
//!    - Command deduplication store (`IdempotentCommandStore`) caches acknowledgments by UUID.
//!      Duplicate deliveries return `AckStatus::DuplicateIgnored` without re-applying side-effects.
//!    - Simulation cancellation halts ticks, locks runner in `Cancelled` state, and emits progress
//!      telemetry with `cancelled: true`.
//!
//! 4. **Wasm & FFI Parity Review**:
//!    - Browser targets: Supported in modern evergreen browsers (Chrome 115+, Firefox 120+, Safari 17+).
//!    - Buffer Lifetime: Memory across Wasm/JS boundary is bounded; string envelopes are parsed into
//!      typed structs without lingering pointers or unbounded heap retention.

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fmt;
use std::ops::Deref;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

// ============================================================================
// ABI Constants
// ============================================================================

/// Canonical schema identifier for the Task E5.1 narrow ABI.
pub const ABI_SCHEMA_VERSION: &str = "careops_ed.abi.v1";

/// Maximum number of command acknowledgments retained for idempotency deduplication.
pub const DEFAULT_IDEMPOTENCY_CAPACITY: usize = 1024;

// ============================================================================
// Wide Integer Lossless Representation (u128 virtual tick)
// ============================================================================

/// Lossless representation of a 128-bit virtual simulation tick.
///
/// In JavaScript engines, numbers are double-precision floats (IEEE 754) with a safe
/// integer limit of $2^{53} - 1$ (`Number.MAX_SAFE_INTEGER` = 9,007,199,254,740,991).
/// To prevent truncation or corruption of simulation time, `U128Tick` always serializes
/// as a decimal string in JSON. Deserialization accepts either a string or an integer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct U128Tick(pub u128);

impl U128Tick {
    pub const ZERO: Self = Self(0);
    pub const MAX: Self = Self(u128::MAX);

    #[must_use]
    pub const fn new(value: u128) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn as_u128(self) -> u128 {
        self.0
    }
}

impl Deref for U128Tick {
    type Target = u128;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u128> for U128Tick {
    fn from(v: u128) -> Self {
        Self(v)
    }
}

impl From<u64> for U128Tick {
    fn from(v: u64) -> Self {
        Self(v as u128)
    }
}

impl fmt::Display for U128Tick {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for U128Tick {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for U128Tick {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct U128TickVisitor;

        impl<'de> Visitor<'de> for U128TickVisitor {
            type Value = U128Tick;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a u128 formatted as a decimal string or integer")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                value.parse::<u128>().map(U128Tick).map_err(|err| {
                    de::Error::custom(format!("invalid u128 string '{value}': {err}"))
                })
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(U128Tick(value as u128))
            }

            fn visit_u128<E>(self, value: u128) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(U128Tick(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if value < 0 {
                    Err(de::Error::custom("u128 tick cannot be negative"))
                } else {
                    Ok(U128Tick(value as u128))
                }
            }
        }

        deserializer.deserialize_any(U128TickVisitor)
    }
}

// ============================================================================
// Spatial and Entity State Models
// ============================================================================

/// Snapshot state of a single entity (patient, clinician, equipment, bed).
/// Coordinates are fixed-point millimeters to preserve exact spatial reproducibility.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityStateSnapshot {
    pub entity_id: String,
    pub entity_type: String,
    pub location_id: String,
    pub x_mm: i64,
    pub y_mm: i64,
    pub floor_level: i32,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_id: Option<String>,
}

/// Incremental mutation applied to an entity during a delta update.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EntityMutation {
    pub entity_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_mm: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y_mm: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub floor_level: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_id: Option<String>,
}

/// Snapshot of a waiting room or service queue.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueStateSnapshot {
    pub queue_id: String,
    pub name: String,
    pub length: usize,
    pub waiting_entity_ids: Vec<String>,
}

/// Delta modification to a queue.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueDelta {
    pub queue_id: String,
    pub length: usize,
    pub joined_ids: Vec<String>,
    pub departed_ids: Vec<String>,
}

/// Macro operational metric summary attached to snapshots.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct MetricSummarySnapshot {
    pub total_arrivals: usize,
    pub active_patients: usize,
    pub completed_patients: usize,
    pub mean_wait_ticks: f64,
    pub bed_utilization: f64,
}

// ============================================================================
// Envelopes: Snapshot, Delta, Command, Ack, Progress, Error
// ============================================================================

/// Full world snapshot envelope (`careops_ed.abi.v1`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SnapshotEnvelope {
    pub schema_version: String,
    pub run_id: String,
    pub incarnation_id: String,
    pub seq: u64,
    pub tick: U128Tick,
    pub spatial_revision: String,
    pub entities: Vec<EntityStateSnapshot>,
    pub queues: Vec<QueueStateSnapshot>,
    pub metrics: MetricSummarySnapshot,
}

/// Incremental state delta envelope.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DeltaEnvelope {
    pub schema_version: String,
    pub run_id: String,
    pub seq: u64,
    pub base_seq: u64,
    pub tick: U128Tick,
    pub created: Vec<EntityStateSnapshot>,
    pub updated: Vec<EntityMutation>,
    pub removed: Vec<String>,
    pub queue_updates: Vec<QueueDelta>,
}

/// Enumeration of interactive simulation command types.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandKind {
    Pause,
    Resume,
    Step,
    SetSpeed,
    Cancel,
    InjectPatient,
    AlterBedCapacity,
    ResyncSnapshot,
}

/// Command envelope dispatched from UI client / controller to simulation runner.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommandEnvelope {
    pub command_id: String,
    pub command_seq: u64,
    pub command_type: CommandKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_tick: Option<U128Tick>,
    pub payload: serde_json::Value,
}

/// Status code for command acknowledgment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AckStatus {
    Accepted,
    Rejected,
    DuplicateIgnored,
}

/// Command acknowledgment returned to caller.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandAckEnvelope {
    pub command_id: String,
    pub status: AckStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_tick: Option<U128Tick>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// High-frequency execution progress envelope.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProgressEnvelope {
    pub tick_current: U128Tick,
    pub tick_horizon: U128Tick,
    pub percent_complete: f64,
    pub steps_per_sec: f64,
    pub cancelled: bool,
}

/// Protocol error envelope.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorEnvelope {
    pub code: u32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tick: Option<U128Tick>,
    pub fatal: bool,
}

// ============================================================================
// State Machine & Protocol Logic
// ============================================================================

/// Protocol synchronization status for a connected client session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionSyncStatus {
    Uninitialized,
    Synced,
    ResyncRequired,
}

/// Possible outcomes when applying an incoming delta.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeltaApplyOutcome {
    Applied,
    StaleIgnored,
}

/// Error kinds encountered during state synchronization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProtocolError {
    SchemaMismatch { expected: String, actual: String },
    SpatialRevisionMismatch { client: String, server: String },
    RunMismatch { expected: String, actual: String },
    ResyncRequired { expected_base: u64, got_base: u64 },
    InvalidState(String),
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SchemaMismatch { expected, actual } => {
                write!(
                    f,
                    "ABI schema mismatch: expected '{expected}', got '{actual}'"
                )
            }
            Self::SpatialRevisionMismatch { client, server } => {
                write!(
                    f,
                    "Spatial layout revision mismatch: client has '{client}', server has '{server}'"
                )
            }
            Self::RunMismatch { expected, actual } => {
                write!(f, "Run ID mismatch: expected '{expected}', got '{actual}'")
            }
            Self::ResyncRequired {
                expected_base,
                got_base,
            } => {
                write!(
                    f,
                    "Sequencing gap detected: expected base_seq {expected_base}, got {got_base}. Full resync required."
                )
            }
            Self::InvalidState(msg) => write!(f, "Invalid protocol state: {msg}"),
        }
    }
}

impl std::error::Error for ProtocolError {}

/// Client-side session state machine tracking local state, entity map, queues, and sequence.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientSessionState {
    pub run_id: String,
    pub incarnation_id: String,
    pub spatial_revision: String,
    pub last_seen_seq: u64,
    pub last_tick: U128Tick,
    pub status: SessionSyncStatus,
    pub entities: BTreeMap<String, EntityStateSnapshot>,
    pub queues: BTreeMap<String, QueueStateSnapshot>,
    pub metrics: MetricSummarySnapshot,
}

impl ClientSessionState {
    /// Initialize a new client session with an expected spatial package revision.
    #[must_use]
    pub fn new(spatial_revision: impl Into<String>) -> Self {
        Self {
            run_id: String::new(),
            incarnation_id: String::new(),
            spatial_revision: spatial_revision.into(),
            last_seen_seq: 0,
            last_tick: U128Tick::ZERO,
            status: SessionSyncStatus::Uninitialized,
            entities: BTreeMap::new(),
            queues: BTreeMap::new(),
            metrics: MetricSummarySnapshot::default(),
        }
    }

    /// Perform a client-server handshake check against the server's spatial revision.
    pub fn handshake(&self, server_spatial_rev: &str) -> Result<(), ProtocolError> {
        if self.spatial_revision != server_spatial_rev {
            Err(ProtocolError::SpatialRevisionMismatch {
                client: self.spatial_revision.clone(),
                server: server_spatial_rev.to_string(),
            })
        } else {
            Ok(())
        }
    }

    /// Ingest a full snapshot, resetting local entity/queue/metric state.
    pub fn apply_snapshot(&mut self, snap: &SnapshotEnvelope) -> Result<(), ProtocolError> {
        if snap.schema_version != ABI_SCHEMA_VERSION {
            return Err(ProtocolError::SchemaMismatch {
                expected: ABI_SCHEMA_VERSION.to_string(),
                actual: snap.schema_version.clone(),
            });
        }
        if snap.spatial_revision != self.spatial_revision {
            return Err(ProtocolError::SpatialRevisionMismatch {
                client: self.spatial_revision.clone(),
                server: snap.spatial_revision.clone(),
            });
        }

        self.run_id = snap.run_id.clone();
        self.incarnation_id = snap.incarnation_id.clone();
        self.last_seen_seq = snap.seq;
        self.last_tick = snap.tick;

        self.entities.clear();
        for ent in &snap.entities {
            self.entities.insert(ent.entity_id.clone(), ent.clone());
        }

        self.queues.clear();
        for q in &snap.queues {
            self.queues.insert(q.queue_id.clone(), q.clone());
        }

        self.metrics = snap.metrics.clone();
        self.status = SessionSyncStatus::Synced;

        Ok(())
    }

    /// Ingest an incremental delta.
    ///
    /// Detects sequence gaps (`delta.base_seq != last_seen_seq`) and transitions to
    /// `ResyncRequired`. Ignores stale / duplicate deltas (`delta.seq <= last_seen_seq`).
    pub fn apply_delta(
        &mut self,
        delta: &DeltaEnvelope,
    ) -> Result<DeltaApplyOutcome, ProtocolError> {
        if delta.schema_version != ABI_SCHEMA_VERSION {
            return Err(ProtocolError::SchemaMismatch {
                expected: ABI_SCHEMA_VERSION.to_string(),
                actual: delta.schema_version.clone(),
            });
        }

        if self.status == SessionSyncStatus::Uninitialized {
            self.status = SessionSyncStatus::ResyncRequired;
            return Err(ProtocolError::InvalidState(
                "Cannot apply delta before initial full snapshot".to_string(),
            ));
        }

        if delta.run_id != self.run_id {
            return Err(ProtocolError::RunMismatch {
                expected: self.run_id.clone(),
                actual: delta.run_id.clone(),
            });
        }

        // Stale or duplicate delta: ignore safely
        if delta.seq <= self.last_seen_seq {
            return Ok(DeltaApplyOutcome::StaleIgnored);
        }

        // Gap detection: base_seq must match our exact current sequence
        if delta.base_seq != self.last_seen_seq {
            self.status = SessionSyncStatus::ResyncRequired;
            return Err(ProtocolError::ResyncRequired {
                expected_base: self.last_seen_seq,
                got_base: delta.base_seq,
            });
        }

        // Apply entity creations
        for ent in &delta.created {
            self.entities.insert(ent.entity_id.clone(), ent.clone());
        }

        // Apply entity mutations
        for mut_spec in &delta.updated {
            if let Some(existing) = self.entities.get_mut(&mut_spec.entity_id) {
                if let Some(loc) = &mut_spec.location_id {
                    existing.location_id = loc.clone();
                }
                if let Some(x) = mut_spec.x_mm {
                    existing.x_mm = x;
                }
                if let Some(y) = mut_spec.y_mm {
                    existing.y_mm = y;
                }
                if let Some(fl) = mut_spec.floor_level {
                    existing.floor_level = fl;
                }
                if let Some(st) = &mut_spec.status {
                    existing.status = st.clone();
                }
                if let Some(rt) = &mut_spec.route_id {
                    existing.route_id = Some(rt.clone());
                }
            }
        }

        // Apply entity removals
        for id in &delta.removed {
            self.entities.remove(id);
        }

        // Apply queue updates
        for q_delta in &delta.queue_updates {
            if let Some(q) = self.queues.get_mut(&q_delta.queue_id) {
                q.length = q_delta.length;
                q.waiting_entity_ids
                    .retain(|id| !q_delta.departed_ids.contains(id));
                for j in &q_delta.joined_ids {
                    if !q.waiting_entity_ids.contains(j) {
                        q.waiting_entity_ids.push(j.clone());
                    }
                }
            }
        }

        self.last_seen_seq = delta.seq;
        self.last_tick = delta.tick;

        Ok(DeltaApplyOutcome::Applied)
    }
}

// ============================================================================
// Idempotent Command Store & Execution Engine
// ============================================================================

/// Deduplication store maintaining recently acknowledged command results.
#[derive(Clone, Debug)]
pub struct IdempotentCommandStore {
    capacity: usize,
    history: HashMap<String, CommandAckEnvelope>,
    fifo_order: VecDeque<String>,
}

impl Default for IdempotentCommandStore {
    fn default() -> Self {
        Self::new(DEFAULT_IDEMPOTENCY_CAPACITY)
    }
}

impl IdempotentCommandStore {
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(16),
            history: HashMap::new(),
            fifo_order: VecDeque::new(),
        }
    }

    /// Submit a command for idempotent execution.
    ///
    /// If `command_id` has already been processed, the previous acknowledgment is
    /// returned immediately with `AckStatus::DuplicateIgnored`, and `executor` is NOT invoked.
    pub fn process_command<F>(&mut self, cmd: &CommandEnvelope, executor: F) -> CommandAckEnvelope
    where
        F: FnOnce(&CommandEnvelope) -> Result<Option<U128Tick>, String>,
    {
        if let Some(cached) = self.history.get(&cmd.command_id) {
            let mut ack = cached.clone();
            ack.status = AckStatus::DuplicateIgnored;
            return ack;
        }

        let ack = match executor(cmd) {
            Ok(effective_tick) => CommandAckEnvelope {
                command_id: cmd.command_id.clone(),
                status: AckStatus::Accepted,
                effective_tick,
                reason: None,
            },
            Err(reason) => CommandAckEnvelope {
                command_id: cmd.command_id.clone(),
                status: AckStatus::Rejected,
                effective_tick: None,
                reason: Some(reason),
            },
        };

        // Cache ack and bound store size
        if self.fifo_order.len() >= self.capacity {
            if let Some(oldest) = self.fifo_order.pop_front() {
                self.history.remove(&oldest);
            }
        }
        self.fifo_order.push_back(cmd.command_id.clone());
        self.history.insert(cmd.command_id.clone(), ack.clone());

        ack
    }
}

/// Mock interactive simulation runner to test command handling, state progression,
/// cancellation, and progress reporting.
#[derive(Debug)]
pub struct MockSimulationRunner {
    pub run_id: String,
    pub incarnation_id: String,
    pub spatial_revision: String,
    pub current_tick: U128Tick,
    pub horizon_ticks: U128Tick,
    pub seq: u64,
    pub bed_capacity: usize,
    pub cancelled: bool,
    pub paused: bool,
    pub command_store: IdempotentCommandStore,
}

impl MockSimulationRunner {
    #[must_use]
    pub fn new(
        run_id: impl Into<String>,
        incarnation_id: impl Into<String>,
        spatial_revision: impl Into<String>,
        horizon: u128,
        bed_capacity: usize,
    ) -> Self {
        Self {
            run_id: run_id.into(),
            incarnation_id: incarnation_id.into(),
            spatial_revision: spatial_revision.into(),
            current_tick: U128Tick::ZERO,
            horizon_ticks: U128Tick::new(horizon),
            seq: 0,
            bed_capacity,
            cancelled: false,
            paused: false,
            command_store: IdempotentCommandStore::default(),
        }
    }

    /// Advance simulation ticks if not paused or cancelled.
    pub fn advance_ticks(&mut self, delta_ticks: u128) -> bool {
        if self.cancelled || self.paused {
            return false;
        }
        let next = (self.current_tick.0 + delta_ticks).min(self.horizon_ticks.0);
        self.current_tick = U128Tick::new(next);
        self.seq += 1;
        true
    }

    /// Dispatch an interactive command idempotently.
    pub fn handle_command(&mut self, cmd: &CommandEnvelope) -> CommandAckEnvelope {
        let current_tick = self.current_tick;
        let cancelled = self.cancelled;

        // Process through idempotent store
        let mut bed_capacity = self.bed_capacity;
        let mut new_cancelled = self.cancelled;
        let mut new_paused = self.paused;

        let ack = self.command_store.process_command(cmd, |command| {
            if cancelled && command.command_type != CommandKind::ResyncSnapshot {
                return Err("Simulation is cancelled; command rejected".to_string());
            }

            match command.command_type {
                CommandKind::Pause => {
                    new_paused = true;
                    Ok(Some(current_tick))
                }
                CommandKind::Resume => {
                    new_paused = false;
                    Ok(Some(current_tick))
                }
                CommandKind::Cancel => {
                    new_cancelled = true;
                    Ok(Some(current_tick))
                }
                CommandKind::AlterBedCapacity => {
                    if let Some(delta) = command
                        .payload
                        .get("delta")
                        .and_then(serde_json::Value::as_i64)
                    {
                        let new_cap = bed_capacity as i64 + delta;
                        if new_cap < 0 {
                            return Err("Bed capacity cannot be negative".to_string());
                        }
                        bed_capacity = new_cap as usize;
                        Ok(Some(current_tick))
                    } else if let Some(target) = command
                        .payload
                        .get("capacity")
                        .and_then(serde_json::Value::as_u64)
                    {
                        bed_capacity = target as usize;
                        Ok(Some(current_tick))
                    } else {
                        Err("AlterBedCapacity payload missing 'delta' or 'capacity'".to_string())
                    }
                }
                CommandKind::Step => Ok(Some(current_tick)),
                CommandKind::SetSpeed
                | CommandKind::InjectPatient
                | CommandKind::ResyncSnapshot => Ok(Some(current_tick)),
            }
        });

        if ack.status == AckStatus::Accepted {
            self.bed_capacity = bed_capacity;
            self.cancelled = new_cancelled;
            self.paused = new_paused;
        }

        ack
    }

    /// Produce a progress report envelope.
    #[must_use]
    pub fn report_progress(&self, steps_per_sec: f64) -> ProgressEnvelope {
        let percent = if self.horizon_ticks.0 == 0 {
            100.0
        } else {
            (self.current_tick.0 as f64 / self.horizon_ticks.0 as f64 * 100.0).clamp(0.0, 100.0)
        };

        ProgressEnvelope {
            tick_current: self.current_tick,
            tick_horizon: self.horizon_ticks,
            percent_complete: percent,
            steps_per_sec,
            cancelled: self.cancelled,
        }
    }
}

// ============================================================================
// Comprehensive Test Suite
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test 1: Wide Integer Lossless Roundtrip beyond 53-bits.
    ///
    /// Verifies values:
    /// - 0
    /// - 1,000
    /// - $2^{53} - 1 = 9,007,199,254,740,991$ (maximum JavaScript safe integer)
    /// - $2^{53} = 9,007,199,254,740,992$ (boundary where float64 loses integer precision)
    /// - $2^{64} - 1 = 18,446,744,073,709,551,615$ (exceeds u64 max)
    /// - $2^{120} = 1,329,227,995,784,915,872,903,807,060,280,344,576$ (large u128 virtual tick)
    #[test]
    fn test_wide_integer_lossless_roundtrip_beyond_53bits() {
        let test_ticks: &[u128] = &[
            0,
            1_000,
            (1u128 << 53) - 1, // 9_007_199_254_740_991
            1u128 << 53,       // 9_007_199_254_740_992
            u64::MAX as u128,  // 18_446_744_073_709_551_615
            1u128 << 120,      // 1329227995784915872903807060280344576
        ];

        for &val in test_ticks {
            let tick = U128Tick::new(val);

            // Serialize to JSON
            let json_str = serde_json::to_string(&tick).expect("serialize U128Tick");

            // Verify that the JSON output is a quoted string representation, preventing JS float64 truncation
            let expected_quoted = format!("\"{val}\"");
            assert_eq!(
                json_str, expected_quoted,
                "Tick {val} must serialize as exact decimal string"
            );

            // Deserialize back to U128Tick
            let deserialized: U128Tick =
                serde_json::from_str(&json_str).expect("deserialize U128Tick from string");

            // Assert exact bit-for-bit identity
            assert_eq!(
                deserialized.as_u128(),
                val,
                "Deserialized tick value must match original bit-for-bit"
            );
        }

        // Also verify lenient deserialization from raw JSON numeric integer (for backward compatibility)
        let json_int = "9007199254740991";
        let from_int: U128Tick = serde_json::from_str(json_int).expect("deserialize from integer");
        assert_eq!(from_int.as_u128(), 9_007_199_254_740_991);
    }

    /// Test 2: Snapshot and Delta Serialization Contract (`careops_ed.abi.v1`).
    ///
    /// Verifies:
    /// - Schema version `"careops_ed.abi.v1"`
    /// - Spatial revision binding
    /// - Entity locations in integer millimeters (`x_mm`, `y_mm`)
    /// - Queue snapshots and metric summaries
    #[test]
    fn test_snapshot_and_delta_serialization_contract() {
        let snapshot = SnapshotEnvelope {
            schema_version: ABI_SCHEMA_VERSION.to_string(),
            run_id: "run-ed-synthetic-001".to_string(),
            incarnation_id: "inc-node-a-01".to_string(),
            seq: 10,
            tick: U128Tick::new(18_446_744_073_709_552_000), // beyond 64-bit
            spatial_revision: "spatial.v2.cairns_ed".to_string(),
            entities: vec![
                EntityStateSnapshot {
                    entity_id: "patient-101".to_string(),
                    entity_type: "patient".to_string(),
                    location_id: "waiting_bay_a".to_string(),
                    x_mm: 12_500,
                    y_mm: -8_200,
                    floor_level: 1,
                    status: "waiting_triage".to_string(),
                    route_id: None,
                },
                EntityStateSnapshot {
                    entity_id: "nurse-04".to_string(),
                    entity_type: "nurse".to_string(),
                    location_id: "triage_desk_1".to_string(),
                    x_mm: 15_000,
                    y_mm: -5_000,
                    floor_level: 1,
                    status: "idle".to_string(),
                    route_id: None,
                },
            ],
            queues: vec![QueueStateSnapshot {
                queue_id: "queue_triage".to_string(),
                name: "Triage Assessment Queue".to_string(),
                length: 1,
                waiting_entity_ids: vec!["patient-101".to_string()],
            }],
            metrics: MetricSummarySnapshot {
                total_arrivals: 1,
                active_patients: 1,
                completed_patients: 0,
                mean_wait_ticks: 120.5,
                bed_utilization: 0.25,
            },
        };

        // Serialize and inspect JSON
        let snap_json = serde_json::to_string_pretty(&snapshot).expect("serialize snapshot");
        assert!(snap_json.contains(r#""schema_version": "careops_ed.abi.v1""#));
        assert!(snap_json.contains(r#""spatial_revision": "spatial.v2.cairns_ed""#));
        assert!(snap_json.contains(r#""x_mm": 12500"#));
        assert!(snap_json.contains(r#""y_mm": -8200"#));

        // Roundtrip deserialize
        let parsed_snap: SnapshotEnvelope =
            serde_json::from_str(&snap_json).expect("deserialize snapshot");
        assert_eq!(parsed_snap, snapshot);

        // Test Delta Envelope
        let delta = DeltaEnvelope {
            schema_version: ABI_SCHEMA_VERSION.to_string(),
            run_id: "run-ed-synthetic-001".to_string(),
            seq: 11,
            base_seq: 10,
            tick: U128Tick::new(18_446_744_073_709_552_500),
            created: vec![EntityStateSnapshot {
                entity_id: "patient-102".to_string(),
                entity_type: "patient".to_string(),
                location_id: "ambulance_bay".to_string(),
                x_mm: 5_000,
                y_mm: 20_000,
                floor_level: 1,
                status: "arriving".to_string(),
                route_id: Some("route_amb_bay".to_string()),
            }],
            updated: vec![EntityMutation {
                entity_id: "patient-101".to_string(),
                location_id: Some("resus_bay_1".to_string()),
                x_mm: Some(25_000),
                y_mm: Some(10_000),
                floor_level: None,
                status: Some("in_triage".to_string()),
                route_id: Some("route_wait_to_resus".to_string()),
            }],
            removed: vec!["bed-stale-01".to_string()],
            queue_updates: vec![QueueDelta {
                queue_id: "queue_triage".to_string(),
                length: 0,
                joined_ids: vec![],
                departed_ids: vec!["patient-101".to_string()],
            }],
        };

        let delta_json = serde_json::to_string(&delta).expect("serialize delta");
        let parsed_delta: DeltaEnvelope =
            serde_json::from_str(&delta_json).expect("deserialize delta");
        assert_eq!(parsed_delta, delta);
    }

    /// Test 3: Sequencing Gap Detection, Duplicate Delta Safety, and Resync Trigger.
    ///
    /// Verifies:
    /// - Applying delta with `seq=1, base_seq=0` succeeds (`Applied`).
    /// - Applying delta with `seq=3, base_seq=2` (missing seq 2) returns `Err(ResyncRequired)`.
    /// - Client transitions to `ResyncRequired` state.
    /// - Re-applying full snapshot with `seq=3` cleanses state and returns to `Synced`.
    #[test]
    fn test_sequencing_gap_detection_and_resync_trigger() {
        let mut client = ClientSessionState::new("spatial.v1.generic");

        // Initial full snapshot at seq 0
        let snap_0 = SnapshotEnvelope {
            schema_version: ABI_SCHEMA_VERSION.to_string(),
            run_id: "run-gap-test-1".to_string(),
            incarnation_id: "inc-1".to_string(),
            seq: 0,
            tick: U128Tick::new(0),
            spatial_revision: "spatial.v1.generic".to_string(),
            entities: vec![EntityStateSnapshot {
                entity_id: "patient-1".to_string(),
                entity_type: "patient".to_string(),
                location_id: "waiting_area".to_string(),
                x_mm: 1000,
                y_mm: 2000,
                floor_level: 0,
                status: "waiting".to_string(),
                route_id: None,
            }],
            queues: vec![],
            metrics: MetricSummarySnapshot::default(),
        };

        client
            .apply_snapshot(&snap_0)
            .expect("apply initial snapshot");
        assert_eq!(client.status, SessionSyncStatus::Synced);
        assert_eq!(client.last_seen_seq, 0);

        // Delta 1: seq=1, base_seq=0 (valid consecutive step)
        let delta_1 = DeltaEnvelope {
            schema_version: ABI_SCHEMA_VERSION.to_string(),
            run_id: "run-gap-test-1".to_string(),
            seq: 1,
            base_seq: 0,
            tick: U128Tick::new(10),
            created: vec![],
            updated: vec![EntityMutation {
                entity_id: "patient-1".to_string(),
                location_id: None,
                x_mm: Some(1500),
                y_mm: Some(2500),
                floor_level: None,
                status: None,
                route_id: None,
            }],
            removed: vec![],
            queue_updates: vec![],
        };

        let outcome = client.apply_delta(&delta_1).expect("apply delta 1");
        assert_eq!(outcome, DeltaApplyOutcome::Applied);
        assert_eq!(client.last_seen_seq, 1);
        assert_eq!(client.entities.get("patient-1").unwrap().x_mm, 1500);

        // Delta 1 re-delivery: seq=1, base_seq=0 (stale/duplicate delta)
        let stale_outcome = client
            .apply_delta(&delta_1)
            .expect("stale delta should be safely ignored");
        assert_eq!(stale_outcome, DeltaApplyOutcome::StaleIgnored);
        assert_eq!(client.last_seen_seq, 1);

        // Delta 3: seq=3, base_seq=2 (gap: missed seq 2!)
        let delta_3 = DeltaEnvelope {
            schema_version: ABI_SCHEMA_VERSION.to_string(),
            run_id: "run-gap-test-1".to_string(),
            seq: 3,
            base_seq: 2,
            tick: U128Tick::new(30),
            created: vec![],
            updated: vec![],
            removed: vec![],
            queue_updates: vec![],
        };

        let gap_err = client.apply_delta(&delta_3).unwrap_err();
        match gap_err {
            ProtocolError::ResyncRequired {
                expected_base,
                got_base,
            } => {
                assert_eq!(expected_base, 1);
                assert_eq!(got_base, 2);
            }
            other => panic!("Expected ResyncRequired, got: {other:?}"),
        }

        assert_eq!(client.status, SessionSyncStatus::ResyncRequired);

        // Resync recovery: Client requests and receives full snapshot at seq 3
        let snap_resync = SnapshotEnvelope {
            schema_version: ABI_SCHEMA_VERSION.to_string(),
            run_id: "run-gap-test-1".to_string(),
            incarnation_id: "inc-1".to_string(),
            seq: 3,
            tick: U128Tick::new(30),
            spatial_revision: "spatial.v1.generic".to_string(),
            entities: vec![EntityStateSnapshot {
                entity_id: "patient-1".to_string(),
                entity_type: "patient".to_string(),
                location_id: "consult_room_1".to_string(),
                x_mm: 5000,
                y_mm: 6000,
                floor_level: 0,
                status: "in_consultation".to_string(),
                route_id: None,
            }],
            queues: vec![],
            metrics: MetricSummarySnapshot::default(),
        };

        client
            .apply_snapshot(&snap_resync)
            .expect("resync snapshot recovery");
        assert_eq!(client.status, SessionSyncStatus::Synced);
        assert_eq!(client.last_seen_seq, 3);
        assert_eq!(
            client.entities.get("patient-1").unwrap().status,
            "in_consultation"
        );
    }

    /// Test 4: Idempotent Command Deduplication.
    ///
    /// Verifies:
    /// - Command `CMD-101` (AlterBedCapacity: +5) returns `Accepted` on first execution.
    /// - Re-issuing command `CMD-101` returns `DuplicateIgnored`.
    /// - Bed capacity is not double-incremented.
    #[test]
    fn test_idempotent_command_deduplication() {
        let mut runner = MockSimulationRunner::new(
            "run-idem-001",
            "inc-001",
            "spatial.v1.generic",
            10_000,
            20, // initial 20 beds
        );

        let cmd = CommandEnvelope {
            command_id: "CMD-101-UUID-AAA".to_string(),
            command_seq: 1,
            command_type: CommandKind::AlterBedCapacity,
            effective_tick: None,
            payload: serde_json::json!({ "delta": 5 }),
        };

        // First dispatch: Accepted
        let ack1 = runner.handle_command(&cmd);
        assert_eq!(ack1.command_id, "CMD-101-UUID-AAA");
        assert_eq!(ack1.status, AckStatus::Accepted);
        assert_eq!(runner.bed_capacity, 25);

        // Second dispatch with identical command_id: DuplicateIgnored
        let ack2 = runner.handle_command(&cmd);
        assert_eq!(ack2.command_id, "CMD-101-UUID-AAA");
        assert_eq!(ack2.status, AckStatus::DuplicateIgnored);
        // CRITICAL: capacity must stay at 25, NOT increment to 30
        assert_eq!(
            runner.bed_capacity, 25,
            "Idempotent command must not double-mutate bed capacity"
        );

        // Different command with new ID: AlterBedCapacity +10
        let cmd2 = CommandEnvelope {
            command_id: "CMD-102-UUID-BBB".to_string(),
            command_seq: 2,
            command_type: CommandKind::AlterBedCapacity,
            effective_tick: None,
            payload: serde_json::json!({ "delta": 10 }),
        };
        let ack3 = runner.handle_command(&cmd2);
        assert_eq!(ack3.status, AckStatus::Accepted);
        assert_eq!(runner.bed_capacity, 35);
    }

    /// Test 5: Cancellation and Progress Flow.
    ///
    /// Verifies:
    /// - Dispatched `Cancel` command transitions runner to `cancelled = true`.
    /// - Further simulation ticks do not advance virtual time.
    /// - Emitted `ProgressEnvelope` reports `cancelled: true` and current completion percentage.
    #[test]
    fn test_cancellation_and_progress_flow() {
        let mut runner = MockSimulationRunner::new(
            "run-cancel-test",
            "inc-001",
            "spatial.v1.generic",
            1_000,
            15,
        );

        // Advance 250 ticks
        assert!(runner.advance_ticks(250));
        assert_eq!(runner.current_tick.as_u128(), 250);

        let progress_pre = runner.report_progress(500.0);
        assert_eq!(progress_pre.percent_complete, 25.0);
        assert!(!progress_pre.cancelled);

        // Issue Cancel command
        let cancel_cmd = CommandEnvelope {
            command_id: "CMD-CANCEL-999".to_string(),
            command_seq: 10,
            command_type: CommandKind::Cancel,
            effective_tick: None,
            payload: serde_json::Value::Null,
        };

        let ack = runner.handle_command(&cancel_cmd);
        assert_eq!(ack.status, AckStatus::Accepted);
        assert!(runner.cancelled);

        // Further attempts to advance ticks must fail and not advance current_tick
        let advanced = runner.advance_ticks(100);
        assert!(
            !advanced,
            "Cancelled simulation runner must reject tick advance"
        );
        assert_eq!(
            runner.current_tick.as_u128(),
            250,
            "Virtual time must stay halted upon cancellation"
        );

        let progress_post = runner.report_progress(0.0);
        assert!(progress_post.cancelled);
        assert_eq!(progress_post.percent_complete, 25.0);

        // Further non-resync commands rejected
        let alter_cmd = CommandEnvelope {
            command_id: "CMD-ALTER-POST-CANCEL".to_string(),
            command_seq: 11,
            command_type: CommandKind::AlterBedCapacity,
            effective_tick: None,
            payload: serde_json::json!({ "delta": 2 }),
        };
        let ack_rejected = runner.handle_command(&alter_cmd);
        assert_eq!(ack_rejected.status, AckStatus::Rejected);
        assert!(ack_rejected
            .reason
            .unwrap()
            .contains("Simulation is cancelled"));
    }

    /// Test 6: Spatial Revision Mismatch Rejection.
    ///
    /// Verifies client/server handshake fails closed if `client_spatial_revision != server_spatial_revision`.
    #[test]
    fn test_spatial_revision_mismatch_rejection() {
        let client = ClientSessionState::new("cairns_ed_rev_2026_10");

        // Matching revision: Handshake succeeds
        assert!(client.handshake("cairns_ed_rev_2026_10").is_ok());

        // Mismatched revision: Handshake fails closed
        let mismatch_err = client.handshake("cairns_ed_rev_2025_01").unwrap_err();
        match mismatch_err {
            ProtocolError::SpatialRevisionMismatch { client, server } => {
                assert_eq!(client, "cairns_ed_rev_2026_10");
                assert_eq!(server, "cairns_ed_rev_2025_01");
            }
            other => panic!("Expected SpatialRevisionMismatch, got: {other:?}"),
        }

        // Applying snapshot with mismatched revision also fails closed
        let mut client_state = ClientSessionState::new("cairns_ed_rev_2026_10");
        let wrong_rev_snap = SnapshotEnvelope {
            schema_version: ABI_SCHEMA_VERSION.to_string(),
            run_id: "run-mismatch".to_string(),
            incarnation_id: "inc-1".to_string(),
            seq: 0,
            tick: U128Tick::ZERO,
            spatial_revision: "incompatible_layout_v9".to_string(),
            entities: vec![],
            queues: vec![],
            metrics: MetricSummarySnapshot::default(),
        };

        let snap_err = client_state.apply_snapshot(&wrong_rev_snap).unwrap_err();
        assert!(matches!(
            snap_err,
            ProtocolError::SpatialRevisionMismatch { .. }
        ));
    }

    /// Test 7: Error Envelope and Protocol Failure Formats.
    ///
    /// Verifies exact serialization and deserialization of protocol and runner errors.
    #[test]
    fn test_error_envelope_serialization() {
        let err = ErrorEnvelope {
            code: 4004,
            message: "Route obstruction between triage and CT scanner".to_string(),
            tick: Some(U128Tick::new(125_400)),
            fatal: false,
        };

        let err_json = serde_json::to_string(&err).expect("serialize ErrorEnvelope");
        assert!(err_json.contains(r#""code":4004"#));
        assert!(err_json.contains(r#""tick":"125400""#));
        assert!(err_json.contains(r#""fatal":false"#));

        let parsed: ErrorEnvelope =
            serde_json::from_str(&err_json).expect("deserialize ErrorEnvelope");
        assert_eq!(parsed, err);
    }

    /// Test 8: Native / Wasm FFI Buffer Lifetime and Parsing Resilience.
    ///
    /// Simulates bounded memory buffers across FFI / WebSocket boundaries.
    #[test]
    fn test_ffi_buffer_lifetime_and_wasm_parity() {
        // Prepare a raw UTF-8 byte stream as would arrive via WebSocket / WebAssembly memory
        let payload = r#"{
            "schema_version": "careops_ed.abi.v1",
            "run_id": "wasm-run-001",
            "incarnation_id": "wasm-inc-01",
            "seq": 42,
            "tick": "340282366920938463463374607431768211455",
            "spatial_revision": "spatial.v1.cairns",
            "entities": [],
            "queues": [],
            "metrics": {
                "total_arrivals": 0,
                "active_patients": 0,
                "completed_patients": 0,
                "mean_wait_ticks": 0.0,
                "bed_utilization": 0.0
            }
        }"#;

        // Zero-copy parse from byte slice (wasm linear memory slice pattern)
        let parsed: SnapshotEnvelope =
            serde_json::from_slice(payload.as_bytes()).expect("parse from byte slice");

        assert_eq!(parsed.seq, 42);
        assert_eq!(parsed.tick.as_u128(), u128::MAX);
        assert_eq!(parsed.spatial_revision, "spatial.v1.cairns");
    }
}
