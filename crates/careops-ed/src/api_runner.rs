//! Shared scenario validation, bounded path checks and deterministic errors.
//!
//! Native execution, comparison and file export belong to careops-ed-cli and
//! native_output. This module retains the scalar validation and integrity controls
//! used by both native and minimal CPU consumers.

use std::fmt::{Display, Formatter};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::parse_scenario_json;

/// Deterministic exit codes for API and CLI operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum ExitCodeDef {
    Success = 0,
    UsageError = 2,
    FileNotFound = 3,
    MalformedConfig = 4,
    IncompatibleScenario = 5,
    FileAlreadyExists = 6,
    OutputWriteFailed = 7,
    SimulationFailed = 8,
    Interrupted = 9,
}

impl Display for ExitCodeDef {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Success => "SUCCESS (0)",
            Self::UsageError => "USAGE_ERROR (2)",
            Self::FileNotFound => "FILE_NOT_FOUND (3)",
            Self::MalformedConfig => "MALFORMED_CONFIG (4)",
            Self::IncompatibleScenario => "INCOMPATIBLE_SCENARIO (5)",
            Self::FileAlreadyExists => "FILE_ALREADY_EXISTS (6)",
            Self::OutputWriteFailed => "OUTPUT_WRITE_FAILED (7)",
            Self::SimulationFailed => "SIMULATION_FAILED (8)",
            Self::Interrupted => "INTERRUPTED (9)",
        };
        formatter.write_str(name)
    }
}

/// Scenario validation report.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationSuccess {
    pub scenario_id: String,
    pub schema_version: u32,
    pub horizon_ticks: u64,
    pub time_unit: String,
    pub patient_count: usize,
    pub physical_count: Option<u64>,
    pub open_count: Option<u64>,
    pub zones_count: usize,
    pub locations_count: usize,
    pub warnings: Vec<String>,
}

/// Structured error returned by runner and validation functions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationError {
    pub code: ExitCodeDef,
    pub message: String,
}

impl Display for ValidationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for ValidationError {}

impl ValidationError {
    pub fn new(code: ExitCodeDef, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

/// Standard SHA-256 using the workspace-qualified implementation.
pub fn compute_sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(data)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

pub const MAX_SCENARIO_PAYLOAD_BYTES: usize = 10 * 1024 * 1024; // 10 MiB limit

/// Verifies that a path does not contain path traversal components (e.g. `..`).
pub fn check_safe_cli_path(path: &Path) -> Result<(), ValidationError> {
    for component in path.components() {
        if matches!(component, std::path::Component::ParentDir) {
            return Err(ValidationError::new(
                ExitCodeDef::UsageError,
                format!("path traversal rejected in '{}'", path.display()),
            ));
        }
    }
    Ok(())
}

/// Validates scenario configuration from JSON text.
pub fn validate_scenario_source(source: &str) -> Result<ValidationSuccess, ValidationError> {
    if source.len() > MAX_SCENARIO_PAYLOAD_BYTES {
        return Err(ValidationError::new(
            ExitCodeDef::MalformedConfig,
            format!(
                "scenario payload exceeds maximum allowed size ({} bytes)",
                MAX_SCENARIO_PAYLOAD_BYTES
            ),
        ));
    }

    let scenario = parse_scenario_json(source)
        .map_err(|e| ValidationError::new(ExitCodeDef::MalformedConfig, e.to_string()))?;

    if scenario.schema_version != 1 {
        return Err(ValidationError::new(
            ExitCodeDef::MalformedConfig,
            format!(
                "unsupported schema_version {}; only version 1 is supported",
                scenario.schema_version
            ),
        ));
    }
    if scenario.scenario_id.trim().is_empty() {
        return Err(ValidationError::new(
            ExitCodeDef::MalformedConfig,
            "scenario_id must not be empty",
        ));
    }
    if scenario.horizon_ticks == 0 {
        return Err(ValidationError::new(
            ExitCodeDef::MalformedConfig,
            "horizon_ticks must be greater than zero",
        ));
    }
    if scenario.time_unit.trim().is_empty() {
        return Err(ValidationError::new(
            ExitCodeDef::MalformedConfig,
            "time_unit must not be empty",
        ));
    }

    let mut warnings = Vec::new();
    let cap = &scenario.capacity_location.capacity;
    if cap.physical_count == Some(0) && cap.open_count == Some(0) {
        return Err(ValidationError::new(
            ExitCodeDef::MalformedConfig,
            "capacity physical and open counts cannot both be zero",
        ));
    }

    for route in &scenario.capacity_location.routes {
        if !route.distance.value.is_finite() || route.distance.value <= 0.0 {
            return Err(ValidationError::new(
                ExitCodeDef::MalformedConfig,
                format!(
                    "route from '{}' to '{}' has invalid non-positive distance {}",
                    route.from_location_id, route.to_location_id, route.distance.value
                ),
            ));
        }
    }

    // Verify patient arrival and duration bounds
    for p in &scenario.patients {
        if p.work_ticks == 0 {
            return Err(ValidationError::new(
                ExitCodeDef::MalformedConfig,
                format!("patient '{}' has non-positive work_ticks", p.patient_id),
            ));
        }
        if p.arrival_tick > scenario.horizon_ticks {
            warnings.push(format!(
                "patient {} arrives at tick {} which exceeds horizon_ticks {}",
                p.patient_id, p.arrival_tick, scenario.horizon_ticks
            ));
        }
    }

    Ok(ValidationSuccess {
        scenario_id: scenario.scenario_id,
        schema_version: scenario.schema_version,
        horizon_ticks: scenario.horizon_ticks,
        time_unit: scenario.time_unit,
        patient_count: scenario.patients.len(),
        physical_count: cap.physical_count,
        open_count: cap.open_count,
        zones_count: scenario.capacity_location.zones.len(),
        locations_count: scenario.capacity_location.locations.len(),
        warnings,
    })
}

pub use careops_core::experiment::allocate_replication_seeds;
pub use careops_core::experiment::CancellationToken;
