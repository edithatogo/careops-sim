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

/// Self-contained FIPS 180-4 standard SHA-256 implementation with zero external dependencies.
#[allow(clippy::manual_is_multiple_of, clippy::chunks_exact_to_as_chunks)]
pub fn compute_sha256_hex(data: &[u8]) -> String {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let k: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut msg = data.to_vec();
    msg.push(0x80);
    while (msg.len() + 8) % 64 != 0 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, part) in chunk.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes([part[0], part[1], part[2], part[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h_val] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h_val
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(k[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h_val = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(h_val);
    }
    format!(
        "{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}",
        h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]
    )
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
