//! Public API, experiment runner, CLI dispatcher, atomic outputs, and deterministic error codes for E3.1.
//!
//! Exposes:
//! - Deterministic error / exit codes.
//! - Scenario validation (`validate`).
//! - Experiment execution with atomic writes, overwrite protection, and cancellation (`run`).
//! - Metric comparison with units, denominators, uncertainty, and incompatible scenario diagnostics (`compare`).
//! - Structured exports in JSON, CSV, and Arrow schema metadata (`export`).
//! - Bounded CLI dispatcher with fresh-user walkthrough and reference scenario selection.

use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::disposition::composite_fixture::run_summary_to_csv;
use crate::{parse_scenario_json, run_scenario, RunSummary, ScenarioConfig};

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

/// Run manifest recording complete provenance and input hash.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunManifest {
    pub schema_version: u32,
    pub scenario_id: String,
    pub input_file: String,
    pub input_sha256: String,
    pub seed: u64,
    pub time_unit: String,
    pub horizon_ticks: u64,
}

/// Complete run artifact document combining manifest and summary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunDocument {
    pub manifest: RunManifest,
    pub summary: RunSummary,
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

/// Validates scenario configuration from JSON text.
pub fn validate_scenario_source(source: &str) -> Result<ValidationSuccess, ValidationError> {
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

    // Verify patient arrival bounds
    for p in &scenario.patients {
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

/// Export output formats.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    #[default]
    Json,
    Csv,
    ArrowSchema,
}

/// Options controlling simulation execution and output behavior.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunOptions {
    pub seed_override: Option<u64>,
    pub horizon_override: Option<u64>,
    pub output_path: Option<PathBuf>,
    pub overwrite: bool,
    pub format: ExportFormat,
    pub simulate_disk_full: bool,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            seed_override: None,
            horizon_override: None,
            output_path: None,
            overwrite: false,
            format: ExportFormat::Json,
            simulate_disk_full: false,
        }
    }
}

/// Cooperative cancellation token.
#[derive(Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

/// Artifact marker written when a run is aborted or interrupted.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartialRunMarker {
    pub status: String,
    pub scenario_id: String,
    pub interrupted: bool,
    pub reason: String,
}

/// Artifact inspection status for recovery.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactStatus {
    Complete,
    IncompletePartial(String),
    Corrupted(String),
    NotFound,
}

/// Inspects a run artifact to prevent partial or corrupted runs from being mistaken for complete.
pub fn inspect_artifact(path: &Path) -> ArtifactStatus {
    if !path.exists() {
        // Also check if a .partial marker exists for this path
        let partial_path = path.with_extension("partial");
        if partial_path.exists() {
            if let Ok(content) = fs::read_to_string(&partial_path) {
                return ArtifactStatus::IncompletePartial(content);
            }
        }
        return ArtifactStatus::NotFound;
    }

    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) => return ArtifactStatus::Corrupted(e.to_string()),
    };

    if let Ok(marker) = serde_json::from_slice::<PartialRunMarker>(&bytes) {
        if marker.status == "incomplete" || marker.interrupted {
            return ArtifactStatus::IncompletePartial(marker.reason);
        }
    }

    match serde_json::from_slice::<RunDocument>(&bytes) {
        Ok(_) => ArtifactStatus::Complete,
        Err(e) => ArtifactStatus::Corrupted(e.to_string()),
    }
}

/// Executes an experiment with atomic output writing, overwrite protection, and cancellation checks.
pub fn run_experiment(
    scenario: &ScenarioConfig,
    options: &RunOptions,
    input_file_label: &str,
    input_bytes: &[u8],
    cancel_token: Option<&CancellationToken>,
) -> Result<RunDocument, ValidationError> {
    // Check cancellation early
    if let Some(token) = cancel_token {
        if token.is_cancelled() {
            return Err(ValidationError::new(
                ExitCodeDef::Interrupted,
                "simulation cancelled before start",
            ));
        }
    }

    // Check output overwrite protection before executing
    if let Some(ref out_path) = options.output_path {
        if out_path.exists() && !options.overwrite {
            return Err(ValidationError::new(
                ExitCodeDef::FileAlreadyExists,
                format!(
                    "output file '{}' already exists; use --overwrite to replace it",
                    out_path.display()
                ),
            ));
        }
    }

    // Apply overrides
    let mut effective_scenario = scenario.clone();
    if let Some(seed) = options.seed_override {
        effective_scenario.seed = seed;
    }
    if let Some(horizon) = options.horizon_override {
        if horizon == 0 {
            return Err(ValidationError::new(
                ExitCodeDef::MalformedConfig,
                "horizon_ticks override must be greater than zero",
            ));
        }
        effective_scenario.horizon_ticks = horizon;
    }

    // Run simulation
    let summary = run_scenario(&effective_scenario)
        .map_err(|e| ValidationError::new(ExitCodeDef::SimulationFailed, e.to_string()))?;

    // Check cancellation mid-flight or immediately post-run
    if let Some(token) = cancel_token {
        if token.is_cancelled() {
            // Write partial marker if output_path is specified
            if let Some(ref out_path) = options.output_path {
                let partial_path = out_path.with_extension("partial");
                let marker = PartialRunMarker {
                    status: "incomplete".to_string(),
                    scenario_id: summary.scenario_id.clone(),
                    interrupted: true,
                    reason: "cancelled by user or cancellation token during execution".to_string(),
                };
                if let Ok(json) = serde_json::to_string_pretty(&marker) {
                    let _ = fs::write(&partial_path, json);
                }
            }
            return Err(ValidationError::new(
                ExitCodeDef::Interrupted,
                "simulation interrupted by cancellation",
            ));
        }
    }

    let input_sha256 = compute_sha256_hex(input_bytes);
    let manifest = RunManifest {
        schema_version: 1,
        scenario_id: summary.scenario_id.clone(),
        input_file: input_file_label.to_string(),
        input_sha256,
        seed: summary.seed,
        time_unit: summary.time_unit.clone(),
        horizon_ticks: summary.horizon_ticks,
    };

    let document = RunDocument { manifest, summary };

    // Atomic output writing if output_path requested
    if let Some(ref out_path) = options.output_path {
        if options.simulate_disk_full {
            return Err(ValidationError::new(
                ExitCodeDef::OutputWriteFailed,
                "simulated disk full / no space left on device",
            ));
        }

        let serialized = export_document(&document, options.format)?;

        // Ensure parent directory exists
        if let Some(parent) = out_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|e| {
                    ValidationError::new(
                        ExitCodeDef::OutputWriteFailed,
                        format!("failed to create output directory: {e}"),
                    )
                })?;
            }
        }

        // Write to temporary atomic staging file
        let tmp_path = out_path.with_extension(format!("tmp.{}", std::process::id()));
        fs::write(&tmp_path, serialized.as_bytes()).map_err(|e| {
            ValidationError::new(
                ExitCodeDef::OutputWriteFailed,
                format!("failed to write temporary file: {e}"),
            )
        })?;

        // Atomic rename
        fs::rename(&tmp_path, out_path).map_err(|e| {
            let _ = fs::remove_file(&tmp_path);
            ValidationError::new(
                ExitCodeDef::OutputWriteFailed,
                format!("failed to atomically commit output file: {e}"),
            )
        })?;

        // Remove any lingering .partial marker on successful commit
        let partial_path = out_path.with_extension("partial");
        if partial_path.exists() {
            let _ = fs::remove_file(partial_path);
        }
    }

    Ok(document)
}

/// Seed and replication allocation using deterministic 64-bit stride.
pub fn allocate_replication_seeds(base_seed: u64, replications: usize) -> Vec<u64> {
    let mut seeds = Vec::with_capacity(replications);
    let mut seen = HashSet::new();
    for r in 0..replications {
        let seed = base_seed.wrapping_add((r as u64).wrapping_mul(0x9E3779B97F4A7C15));
        assert!(seen.insert(seed), "replication seeds must be unique");
        seeds.push(seed);
    }
    seeds
}

/// Metric comparison with standardized units, denominators, and uncertainty.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MetricComparison {
    pub metric_name: String,
    pub unit: String,
    pub denominator_description: String,
    pub base_value: f64,
    pub candidate_value: f64,
    pub delta: f64,
    pub percent_change: Option<f64>,
    pub base_se: Option<f64>,
    pub candidate_se: Option<f64>,
}

/// Comparison report across two scenario runs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScenarioComparisonReport {
    pub base_scenario_id: String,
    pub candidate_scenario_id: String,
    pub time_unit: String,
    pub metrics: Vec<MetricComparison>,
    pub diagnostics: Vec<String>,
}

/// Compares matching metrics across two scenario runs, checking scenario compatibility.
pub fn compare_run_documents(
    base: &RunDocument,
    candidate: &RunDocument,
) -> Result<ScenarioComparisonReport, ValidationError> {
    // 1. Verify schema version compatibility
    if base.manifest.schema_version != candidate.manifest.schema_version {
        return Err(ValidationError::new(
            ExitCodeDef::IncompatibleScenario,
            format!(
                "incompatible schema versions: base has {}, candidate has {}",
                base.manifest.schema_version, candidate.manifest.schema_version
            ),
        ));
    }

    // 2. Verify time unit compatibility
    if base.manifest.time_unit != candidate.manifest.time_unit {
        return Err(ValidationError::new(
            ExitCodeDef::IncompatibleScenario,
            format!(
                "incompatible time units: base uses '{}', candidate uses '{}'",
                base.manifest.time_unit, candidate.manifest.time_unit
            ),
        ));
    }

    let b_sum = &base.summary;
    let c_sum = &candidate.summary;

    let mut metrics = Vec::new();
    let mut diagnostics = Vec::new();

    // Metric 1: Arrivals
    let arr_delta = c_sum.arrivals as f64 - b_sum.arrivals as f64;
    metrics.push(MetricComparison {
        metric_name: "arrivals".to_string(),
        unit: "patient".to_string(),
        denominator_description: "total presentation count".to_string(),
        base_value: b_sum.arrivals as f64,
        candidate_value: c_sum.arrivals as f64,
        delta: arr_delta,
        percent_change: if b_sum.arrivals > 0 {
            Some((arr_delta / b_sum.arrivals as f64) * 100.0)
        } else {
            None
        },
        base_se: None,
        candidate_se: None,
    });

    // Metric 2: Completed
    let comp_delta = c_sum.completed as f64 - b_sum.completed as f64;
    metrics.push(MetricComparison {
        metric_name: "completed".to_string(),
        unit: "patient".to_string(),
        denominator_description: "arrivals".to_string(),
        base_value: b_sum.completed as f64,
        candidate_value: c_sum.completed as f64,
        delta: comp_delta,
        percent_change: if b_sum.completed > 0 {
            Some((comp_delta / b_sum.completed as f64) * 100.0)
        } else {
            None
        },
        base_se: None,
        candidate_se: None,
    });

    // Metric 3: Unfinished
    let unf_delta = c_sum.unfinished as f64 - b_sum.unfinished as f64;
    metrics.push(MetricComparison {
        metric_name: "unfinished".to_string(),
        unit: "patient".to_string(),
        denominator_description: "arrivals".to_string(),
        base_value: b_sum.unfinished as f64,
        candidate_value: c_sum.unfinished as f64,
        delta: unf_delta,
        percent_change: if b_sum.unfinished > 0 {
            Some((unf_delta / b_sum.unfinished as f64) * 100.0)
        } else {
            None
        },
        base_se: None,
        candidate_se: None,
    });

    // Metric 4: Mean wait ticks
    let b_waits: Vec<u64> = b_sum.patients.iter().filter_map(|p| p.wait_ticks).collect();
    let c_waits: Vec<u64> = c_sum.patients.iter().filter_map(|p| p.wait_ticks).collect();

    let b_mean_wait = if !b_waits.is_empty() {
        b_waits.iter().sum::<u64>() as f64 / b_waits.len() as f64
    } else {
        0.0
    };
    let c_mean_wait = if !c_waits.is_empty() {
        c_waits.iter().sum::<u64>() as f64 / c_waits.len() as f64
    } else {
        0.0
    };
    let wait_delta = c_mean_wait - b_mean_wait;

    // Standard error calculation for wait ticks
    let b_se = if b_waits.len() > 1 {
        let var = b_waits
            .iter()
            .map(|&w| (w as f64 - b_mean_wait).powi(2))
            .sum::<f64>()
            / (b_waits.len() - 1) as f64;
        Some((var / b_waits.len() as f64).sqrt())
    } else {
        None
    };

    let c_se = if c_waits.len() > 1 {
        let var = c_waits
            .iter()
            .map(|&w| (w as f64 - c_mean_wait).powi(2))
            .sum::<f64>()
            / (c_waits.len() - 1) as f64;
        Some((var / c_waits.len() as f64).sqrt())
    } else {
        None
    };

    metrics.push(MetricComparison {
        metric_name: "mean_wait".to_string(),
        unit: base.manifest.time_unit.clone(),
        denominator_description: "completed patients with wait intervals".to_string(),
        base_value: b_mean_wait,
        candidate_value: c_mean_wait,
        delta: wait_delta,
        percent_change: if b_mean_wait > 0.0 {
            Some((wait_delta / b_mean_wait) * 100.0)
        } else {
            None
        },
        base_se: b_se,
        candidate_se: c_se,
    });

    if wait_delta > 0.0 {
        diagnostics.push(format!(
            "Constrained capacity or demand increased mean wait by {:.2} {}",
            wait_delta, base.manifest.time_unit
        ));
    } else if wait_delta < 0.0 {
        diagnostics.push(format!(
            "Expanded capacity or reduced contention decreased mean wait by {:.2} {}",
            wait_delta.abs(),
            base.manifest.time_unit
        ));
    } else {
        diagnostics.push("Mean wait times are identical between scenarios".to_string());
    }

    if c_sum.unfinished > b_sum.unfinished {
        diagnostics.push(format!(
            "Unfinished / censored presentations increased by {}",
            c_sum.unfinished - b_sum.unfinished
        ));
    }

    Ok(ScenarioComparisonReport {
        base_scenario_id: base.manifest.scenario_id.clone(),
        candidate_scenario_id: candidate.manifest.scenario_id.clone(),
        time_unit: base.manifest.time_unit.clone(),
        metrics,
        diagnostics,
    })
}

/// Exports a run document into the requested serialization format.
pub fn export_document(doc: &RunDocument, format: ExportFormat) -> Result<String, ValidationError> {
    match format {
        ExportFormat::Json => serde_json::to_string_pretty(doc)
            .map_err(|e| ValidationError::new(ExitCodeDef::MalformedConfig, e.to_string())),
        ExportFormat::Csv => Ok(run_summary_to_csv(&doc.summary)),
        ExportFormat::ArrowSchema => {
            // Emits Arrow-compatible tabular schema metadata
            let schema_doc = serde_json::json!({
                "arrow_schema": {
                    "version": "1.0",
                    "fields": [
                        {"name": "patient_id", "type": "utf8", "nullable": false, "unit": "id"},
                        {"name": "arrival_tick", "type": "uint64", "nullable": false, "unit": doc.manifest.time_unit},
                        {"name": "start_tick", "type": "uint64", "nullable": true, "unit": doc.manifest.time_unit},
                        {"name": "completion_tick", "type": "uint64", "nullable": true, "unit": doc.manifest.time_unit},
                        {"name": "work_ticks", "type": "uint64", "nullable": false, "unit": doc.manifest.time_unit},
                        {"name": "wait_ticks", "type": "uint64", "nullable": true, "unit": doc.manifest.time_unit, "denominator": "started_patients"},
                        {"name": "elapsed_ticks", "type": "uint64", "nullable": true, "unit": doc.manifest.time_unit, "denominator": "completed_patients"},
                        {"name": "status", "type": "utf8", "nullable": false, "unit": "categorical"}
                    ],
                    "metadata": {
                        "scenario_id": doc.manifest.scenario_id,
                        "seed": doc.manifest.seed.to_string(),
                        "time_unit": doc.manifest.time_unit,
                        "horizon_ticks": doc.manifest.horizon_ticks.to_string(),
                        "input_sha256": doc.manifest.input_sha256
                    }
                }
            });
            serde_json::to_string_pretty(&schema_doc)
                .map_err(|e| ValidationError::new(ExitCodeDef::MalformedConfig, e.to_string()))
        }
    }
}

/// CLI command execution output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CliOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: ExitCodeDef,
}

impl CliOutput {
    pub fn success(stdout: impl Into<String>) -> Self {
        Self {
            stdout: stdout.into(),
            stderr: String::new(),
            exit_code: ExitCodeDef::Success,
        }
    }

    pub fn error(code: ExitCodeDef, message: impl Into<String>) -> Self {
        Self {
            stdout: String::new(),
            stderr: message.into(),
            exit_code: code,
        }
    }
}

/// Friendly, comprehensive usage text for fresh users.
pub fn cli_usage_text() -> &'static str {
    "Usage: careops-ed <command> [options]\n\
\n\
Headless Emergency Department Simulation & Experiment Tool.\n\
\n\
Commands:\n\
  validate <scenario.json>\n\
      Validate scenario schema, parameters, and consistency.\n\
\n\
  run <scenario.json> [options]\n\
      Execute a simulation scenario.\n\
      Options:\n\
        --seed <u64>          Override pseudo-random generator seed\n\
        --horizon <u64>       Override simulation horizon in ticks\n\
        --output <path>       Destination path for results\n\
        --overwrite           Allow overwriting existing output file\n\
        --format <json|csv>   Output format (default: json)\n\
        --reference <name>    Use checked-in reference scenario (nominal | constrained)\n\
\n\
  batch <scenario.json> [options]\n\
      Execute multi-replication experiment across independent CPU workers with checkpointing.\n\
      Options:\n\
        --replications <usize> Number of replications (default: 4)\n\
        --workers <usize>      Number of parallel worker threads (default: 2)\n\
        --seed <u64>           Base seed for deterministic seed sequence\n\
        --horizon <u64>        Override simulation horizon in ticks\n\
        --output-dir <path>    Output directory for checkpoint and final artifact\n\
        --checkpoint <usize>   Checkpoint interval in replications (default: 2)\n\
        --resume <path>        Resume previously interrupted batch from checkpoint\n\
\n\
  compare <base_run.json> <candidate_run.json>\n\
      Compare metrics between two simulation runs with units and denominators.\n\
\n\
  export <run.json> [options]\n\
      Export run artifact to CSV or Arrow-compatible format.\n\
      Options:\n\
        --format <json|csv|arrow> Output format (default: csv)\n\
        --output <path>           Destination file\n\
        --overwrite               Allow overwriting existing output\n\
\n\
Exit Codes:\n\
  0  Success\n\
  2  Usage error / invalid arguments\n\
  3  File not found\n\
  4  Malformed scenario configuration\n\
  5  Incompatible scenarios for comparison\n\
  6  Output file already exists (overwrite protection)\n\
  7  Output write failed (I/O or disk full)\n\
  8  Simulation failed (runtime or invariant error)\n\
  9  Interrupted / cancelled execution"
}

/// Executes CLI arguments and returns structured output and exit code.
pub fn execute_cli_args(args: &[String]) -> CliOutput {
    if args.is_empty() {
        return CliOutput::error(
            ExitCodeDef::UsageError,
            format!("careops-ed: missing command\n\n{}", cli_usage_text()),
        );
    }

    let cmd = &args[0];
    if cmd == "--help" || cmd == "-h" || cmd == "help" {
        return CliOutput::success(cli_usage_text());
    }

    match cmd.as_str() {
        "validate" => {
            if args.len() < 2 {
                return CliOutput::error(
                    ExitCodeDef::UsageError,
                    "careops-ed: validate requires a scenario file path",
                );
            }
            let path = PathBuf::from(&args[1]);
            if !path.exists() {
                return CliOutput::error(
                    ExitCodeDef::FileNotFound,
                    format!("careops-ed: scenario file not found: '{}'", path.display()),
                );
            }
            let bytes = match fs::read(&path) {
                Ok(b) => b,
                Err(e) => {
                    return CliOutput::error(
                        ExitCodeDef::FileNotFound,
                        format!("careops-ed: failed to read '{}': {e}", path.display()),
                    )
                }
            };
            let source = match std::str::from_utf8(&bytes) {
                Ok(s) => s,
                Err(_) => {
                    return CliOutput::error(
                        ExitCodeDef::MalformedConfig,
                        "careops-ed: file contains invalid UTF-8",
                    )
                }
            };
            match validate_scenario_source(source) {
                Ok(report) => {
                    let beds_desc = match (report.physical_count, report.open_count) {
                        (Some(p), Some(o)) => format!("{p} physical / {o} open beds"),
                        (Some(p), None) => format!("{p} physical beds"),
                        (None, Some(o)) => format!("{o} open beds"),
                        (None, None) => "unspecified beds".to_string(),
                    };
                    let msg = format!(
                        "valid scenario '{}' (schema v{}, {} ticks {}, {}, {} patients)",
                        report.scenario_id,
                        report.schema_version,
                        report.horizon_ticks,
                        report.time_unit,
                        beds_desc,
                        report.patient_count
                    );
                    CliOutput::success(msg)
                }
                Err(err) => CliOutput::error(err.code, format!("careops-ed: {err}")),
            }
        }
        "run" => {
            let mut scenario_arg: Option<String> = None;
            let mut seed_override: Option<u64> = None;
            let mut horizon_override: Option<u64> = None;
            let mut output_path: Option<PathBuf> = None;
            let mut overwrite = false;
            let mut format = ExportFormat::Json;

            let mut idx = 1;
            while idx < args.len() {
                match args[idx].as_str() {
                    "--help" | "-h" => return CliOutput::success(cli_usage_text()),
                    "--seed" => {
                        idx += 1;
                        if idx >= args.len() {
                            return CliOutput::error(
                                ExitCodeDef::UsageError,
                                "careops-ed: --seed requires a numeric argument",
                            );
                        }
                        match args[idx].parse::<u64>() {
                            Ok(s) => seed_override = Some(s),
                            Err(_) => {
                                return CliOutput::error(
                                    ExitCodeDef::UsageError,
                                    "careops-ed: invalid seed integer",
                                )
                            }
                        }
                    }
                    "--horizon" => {
                        idx += 1;
                        if idx >= args.len() {
                            return CliOutput::error(
                                ExitCodeDef::UsageError,
                                "careops-ed: --horizon requires a numeric argument",
                            );
                        }
                        match args[idx].parse::<u64>() {
                            Ok(h) => horizon_override = Some(h),
                            Err(_) => {
                                return CliOutput::error(
                                    ExitCodeDef::UsageError,
                                    "careops-ed: invalid horizon integer",
                                )
                            }
                        }
                    }
                    "--output" | "-o" => {
                        idx += 1;
                        if idx >= args.len() {
                            return CliOutput::error(
                                ExitCodeDef::UsageError,
                                "careops-ed: --output requires a file path",
                            );
                        }
                        output_path = Some(PathBuf::from(&args[idx]));
                    }
                    "--overwrite" => {
                        overwrite = true;
                    }
                    "--format" => {
                        idx += 1;
                        if idx >= args.len() {
                            return CliOutput::error(
                                ExitCodeDef::UsageError,
                                "careops-ed: --format requires 'json' or 'csv'",
                            );
                        }
                        match args[idx].to_lowercase().as_str() {
                            "json" => format = ExportFormat::Json,
                            "csv" => format = ExportFormat::Csv,
                            other => {
                                return CliOutput::error(
                                    ExitCodeDef::UsageError,
                                    format!("careops-ed: unsupported format '{other}'"),
                                )
                            }
                        }
                    }
                    flag if flag.starts_with('-') => {
                        return CliOutput::error(
                            ExitCodeDef::UsageError,
                            format!("careops-ed: unknown option '{flag}'"),
                        );
                    }
                    pos_arg => {
                        if scenario_arg.is_none() {
                            scenario_arg = Some(pos_arg.to_string());
                        } else {
                            return CliOutput::error(
                                ExitCodeDef::UsageError,
                                format!("careops-ed: unexpected extra argument '{pos_arg}'"),
                            );
                        }
                    }
                }
                idx += 1;
            }

            let Some(input_file) = scenario_arg else {
                return CliOutput::error(
                    ExitCodeDef::UsageError,
                    "careops-ed: run requires a scenario file path",
                );
            };

            let input_path = PathBuf::from(&input_file);
            if !input_path.exists() {
                return CliOutput::error(
                    ExitCodeDef::FileNotFound,
                    format!(
                        "careops-ed: scenario file not found: '{}'",
                        input_path.display()
                    ),
                );
            }

            let bytes = match fs::read(&input_path) {
                Ok(b) => b,
                Err(e) => {
                    return CliOutput::error(
                        ExitCodeDef::FileNotFound,
                        format!("careops-ed: failed to read '{}': {e}", input_path.display()),
                    )
                }
            };

            let source = match std::str::from_utf8(&bytes) {
                Ok(s) => s,
                Err(_) => {
                    return CliOutput::error(
                        ExitCodeDef::MalformedConfig,
                        "careops-ed: file contains invalid UTF-8",
                    )
                }
            };

            let scenario = match parse_scenario_json(source) {
                Ok(s) => s,
                Err(e) => {
                    return CliOutput::error(
                        ExitCodeDef::MalformedConfig,
                        format!("careops-ed: invalid scenario: {e}"),
                    )
                }
            };

            let options = RunOptions {
                seed_override,
                horizon_override,
                output_path,
                overwrite,
                format,
                simulate_disk_full: false,
            };

            match run_experiment(&scenario, &options, &input_file, &bytes, None) {
                Ok(doc) => {
                    if let Some(ref out_p) = options.output_path {
                        CliOutput::success(format!(
                            "run completed successfully; wrote output to '{}'",
                            out_p.display()
                        ))
                    } else {
                        match export_document(&doc, options.format) {
                            Ok(text) => CliOutput::success(text),
                            Err(e) => CliOutput::error(e.code, format!("careops-ed: {e}")),
                        }
                    }
                }
                Err(err) => CliOutput::error(err.code, format!("careops-ed: {err}")),
            }
        }
        "compare" => {
            if args.len() < 3 {
                return CliOutput::error(
                    ExitCodeDef::UsageError,
                    "careops-ed: compare requires two run artifact JSON files (base and candidate)",
                );
            }
            let base_path = PathBuf::from(&args[1]);
            let cand_path = PathBuf::from(&args[2]);

            if !base_path.exists() {
                return CliOutput::error(
                    ExitCodeDef::FileNotFound,
                    format!("careops-ed: base file not found: '{}'", base_path.display()),
                );
            }
            if !cand_path.exists() {
                return CliOutput::error(
                    ExitCodeDef::FileNotFound,
                    format!(
                        "careops-ed: candidate file not found: '{}'",
                        cand_path.display()
                    ),
                );
            }

            let b_bytes = match fs::read(&base_path) {
                Ok(b) => b,
                Err(e) => {
                    return CliOutput::error(
                        ExitCodeDef::FileNotFound,
                        format!(
                            "careops-ed: failed to read base '{}': {e}",
                            base_path.display()
                        ),
                    )
                }
            };
            let c_bytes = match fs::read(&cand_path) {
                Ok(b) => b,
                Err(e) => {
                    return CliOutput::error(
                        ExitCodeDef::FileNotFound,
                        format!(
                            "careops-ed: failed to read candidate '{}': {e}",
                            cand_path.display()
                        ),
                    )
                }
            };

            let base_doc: RunDocument = match serde_json::from_slice(&b_bytes) {
                Ok(d) => d,
                Err(e) => {
                    return CliOutput::error(
                        ExitCodeDef::MalformedConfig,
                        format!("careops-ed: invalid base run JSON: {e}"),
                    )
                }
            };
            let cand_doc: RunDocument = match serde_json::from_slice(&c_bytes) {
                Ok(d) => d,
                Err(e) => {
                    return CliOutput::error(
                        ExitCodeDef::MalformedConfig,
                        format!("careops-ed: invalid candidate run JSON: {e}"),
                    )
                }
            };

            match compare_run_documents(&base_doc, &cand_doc) {
                Ok(rep) => {
                    let out = serde_json::to_string_pretty(&rep).unwrap_or_default();
                    CliOutput::success(out)
                }
                Err(e) => CliOutput::error(e.code, format!("careops-ed: {e}")),
            }
        }
        "export" => {
            if args.len() < 2 {
                return CliOutput::error(
                    ExitCodeDef::UsageError,
                    "careops-ed: export requires a run document JSON file",
                );
            }
            let mut run_path: Option<PathBuf> = None;
            let mut format = ExportFormat::Csv;
            let mut output_path: Option<PathBuf> = None;
            let mut overwrite = false;

            let mut idx = 1;
            while idx < args.len() {
                match args[idx].as_str() {
                    "--format" => {
                        idx += 1;
                        if idx >= args.len() {
                            return CliOutput::error(
                                ExitCodeDef::UsageError,
                                "careops-ed: --format requires 'csv', 'json', or 'arrow'",
                            );
                        }
                        match args[idx].to_lowercase().as_str() {
                            "csv" => format = ExportFormat::Csv,
                            "json" => format = ExportFormat::Json,
                            "arrow" => format = ExportFormat::ArrowSchema,
                            other => {
                                return CliOutput::error(
                                    ExitCodeDef::UsageError,
                                    format!("careops-ed: unsupported export format '{other}'"),
                                )
                            }
                        }
                    }
                    "--output" | "-o" => {
                        idx += 1;
                        if idx >= args.len() {
                            return CliOutput::error(
                                ExitCodeDef::UsageError,
                                "careops-ed: --output requires a file path",
                            );
                        }
                        output_path = Some(PathBuf::from(&args[idx]));
                    }
                    "--overwrite" => {
                        overwrite = true;
                    }
                    pos if !pos.starts_with('-') => {
                        if run_path.is_none() {
                            run_path = Some(PathBuf::from(pos));
                        } else {
                            return CliOutput::error(
                                ExitCodeDef::UsageError,
                                format!("careops-ed: unexpected extra argument '{pos}'"),
                            );
                        }
                    }
                    other => {
                        return CliOutput::error(
                            ExitCodeDef::UsageError,
                            format!("careops-ed: unrecognized option '{other}'"),
                        );
                    }
                }
                idx += 1;
            }

            let Some(path) = run_path else {
                return CliOutput::error(
                    ExitCodeDef::UsageError,
                    "careops-ed: export requires a run document JSON file",
                );
            };

            if !path.exists() {
                return CliOutput::error(
                    ExitCodeDef::FileNotFound,
                    format!("careops-ed: file not found: '{}'", path.display()),
                );
            }

            let bytes = match fs::read(&path) {
                Ok(b) => b,
                Err(e) => {
                    return CliOutput::error(
                        ExitCodeDef::FileNotFound,
                        format!("careops-ed: failed to read '{}': {e}", path.display()),
                    )
                }
            };

            let doc: RunDocument = match serde_json::from_slice(&bytes) {
                Ok(d) => d,
                Err(e) => {
                    return CliOutput::error(
                        ExitCodeDef::MalformedConfig,
                        format!("careops-ed: invalid run JSON: {e}"),
                    )
                }
            };

            let exported = match export_document(&doc, format) {
                Ok(s) => s,
                Err(e) => return CliOutput::error(e.code, format!("careops-ed: {e}")),
            };

            if let Some(out_p) = output_path {
                if out_p.exists() && !overwrite {
                    return CliOutput::error(
                        ExitCodeDef::FileAlreadyExists,
                        format!(
                            "careops-ed: output file '{}' already exists",
                            out_p.display()
                        ),
                    );
                }
                if let Err(e) = fs::write(&out_p, &exported) {
                    return CliOutput::error(
                        ExitCodeDef::OutputWriteFailed,
                        format!("careops-ed: failed to write to '{}': {e}", out_p.display()),
                    );
                }
                CliOutput::success(format!("exported to '{}'", out_p.display()))
            } else {
                CliOutput::success(exported)
            }
        }
        "batch" => {
            let mut scenario_arg: Option<String> = None;
            let mut replications: usize = 4;
            let mut num_workers: usize = 2;
            let mut seed_override: Option<u64> = None;
            let mut horizon_override: Option<u64> = None;
            let mut output_dir: Option<PathBuf> = None;
            let mut checkpoint_interval: Option<usize> = Some(2);
            let mut resume_path: Option<PathBuf> = None;

            let mut idx = 1;
            while idx < args.len() {
                match args[idx].as_str() {
                    "--help" | "-h" => return CliOutput::success(cli_usage_text()),
                    "--replications" | "-r" => {
                        idx += 1;
                        if idx >= args.len() {
                            return CliOutput::error(
                                ExitCodeDef::UsageError,
                                "careops-ed: --replications requires an integer",
                            );
                        }
                        match args[idx].parse::<usize>() {
                            Ok(r) if r > 0 => replications = r,
                            _ => {
                                return CliOutput::error(
                                    ExitCodeDef::UsageError,
                                    "careops-ed: invalid replications integer",
                                )
                            }
                        }
                    }
                    "--workers" | "-w" => {
                        idx += 1;
                        if idx >= args.len() {
                            return CliOutput::error(
                                ExitCodeDef::UsageError,
                                "careops-ed: --workers requires an integer",
                            );
                        }
                        match args[idx].parse::<usize>() {
                            Ok(w) if w > 0 => num_workers = w,
                            _ => {
                                return CliOutput::error(
                                    ExitCodeDef::UsageError,
                                    "careops-ed: invalid workers integer",
                                )
                            }
                        }
                    }
                    "--seed" => {
                        idx += 1;
                        if idx >= args.len() {
                            return CliOutput::error(
                                ExitCodeDef::UsageError,
                                "careops-ed: --seed requires a numeric argument",
                            );
                        }
                        match args[idx].parse::<u64>() {
                            Ok(s) => seed_override = Some(s),
                            _ => {
                                return CliOutput::error(
                                    ExitCodeDef::UsageError,
                                    "careops-ed: invalid seed integer",
                                )
                            }
                        }
                    }
                    "--horizon" => {
                        idx += 1;
                        if idx >= args.len() {
                            return CliOutput::error(
                                ExitCodeDef::UsageError,
                                "careops-ed: --horizon requires a numeric argument",
                            );
                        }
                        match args[idx].parse::<u64>() {
                            Ok(h) => horizon_override = Some(h),
                            _ => {
                                return CliOutput::error(
                                    ExitCodeDef::UsageError,
                                    "careops-ed: invalid horizon integer",
                                )
                            }
                        }
                    }
                    "--output-dir" | "-o" => {
                        idx += 1;
                        if idx >= args.len() {
                            return CliOutput::error(
                                ExitCodeDef::UsageError,
                                "careops-ed: --output-dir requires a directory path",
                            );
                        }
                        output_dir = Some(PathBuf::from(&args[idx]));
                    }
                    "--checkpoint" => {
                        idx += 1;
                        if idx >= args.len() {
                            return CliOutput::error(
                                ExitCodeDef::UsageError,
                                "careops-ed: --checkpoint requires an integer",
                            );
                        }
                        match args[idx].parse::<usize>() {
                            Ok(c) if c > 0 => checkpoint_interval = Some(c),
                            _ => {
                                return CliOutput::error(
                                    ExitCodeDef::UsageError,
                                    "careops-ed: invalid checkpoint integer",
                                )
                            }
                        }
                    }
                    "--resume" => {
                        idx += 1;
                        if idx >= args.len() {
                            return CliOutput::error(
                                ExitCodeDef::UsageError,
                                "careops-ed: --resume requires a checkpoint file path",
                            );
                        }
                        resume_path = Some(PathBuf::from(&args[idx]));
                    }
                    pos if !pos.starts_with('-') && scenario_arg.is_none() => {
                        scenario_arg = Some(pos.to_string());
                    }
                    other => {
                        return CliOutput::error(
                            ExitCodeDef::UsageError,
                            format!("careops-ed: unrecognized batch option '{other}'"),
                        );
                    }
                }
                idx += 1;
            }

            let Some(input_file) = scenario_arg else {
                return CliOutput::error(
                    ExitCodeDef::UsageError,
                    "careops-ed: batch requires a scenario file path",
                );
            };

            let input_path = PathBuf::from(&input_file);
            if !input_path.exists() {
                return CliOutput::error(
                    ExitCodeDef::FileNotFound,
                    format!(
                        "careops-ed: scenario file not found: '{}'",
                        input_path.display()
                    ),
                );
            }

            let bytes = match fs::read(&input_path) {
                Ok(b) => b,
                Err(e) => {
                    return CliOutput::error(
                        ExitCodeDef::FileNotFound,
                        format!("careops-ed: failed to read '{}': {e}", input_path.display()),
                    )
                }
            };

            let source = match std::str::from_utf8(&bytes) {
                Ok(s) => s,
                Err(_) => {
                    return CliOutput::error(
                        ExitCodeDef::MalformedConfig,
                        "careops-ed: file contains invalid UTF-8",
                    )
                }
            };

            let scenario = match crate::parse_scenario_json(source) {
                Ok(s) => s,
                Err(e) => {
                    return CliOutput::error(
                        ExitCodeDef::MalformedConfig,
                        format!("careops-ed: failed to parse scenario JSON: {e}"),
                    );
                }
            };

            if let Some(r_path) = resume_path {
                let out_dir = match output_dir {
                    Some(d) => d,
                    None => r_path
                        .parent()
                        .map(PathBuf::from)
                        .unwrap_or_else(|| PathBuf::from(".")),
                };
                match crate::disposition::batch_runner::resume_batch_from_checkpoint(
                    &r_path,
                    &scenario,
                    num_workers,
                    &out_dir,
                    None,
                ) {
                    Ok(doc) => {
                        let json = serde_json::to_string_pretty(&doc).unwrap_or_default();
                        CliOutput::success(json)
                    }
                    Err(e) => CliOutput::error(e.code, format!("careops-ed: {e}")),
                }
            } else {
                let batch_cfg = crate::disposition::batch_runner::BatchConfig {
                    schema_version: 1,
                    batch_id: format!("batch-{}", scenario.scenario_id),
                    scenario_id: scenario.scenario_id.clone(),
                    base_seed: seed_override.unwrap_or(scenario.seed),
                    replications,
                    num_workers,
                    horizon_override,
                    checkpoint_interval,
                };

                match crate::disposition::batch_runner::run_batch_with_workers(
                    &scenario,
                    &batch_cfg,
                    output_dir.as_deref(),
                    None,
                    None,
                ) {
                    Ok(doc) => {
                        let json = serde_json::to_string_pretty(&doc).unwrap_or_default();
                        CliOutput::success(json)
                    }
                    Err(e) => CliOutput::error(e.code, format!("careops-ed: {e}")),
                }
            }
        }
        unknown => CliOutput::error(
            ExitCodeDef::UsageError,
            format!(
                "careops-ed: unknown command '{unknown}'\n\n{}",
                cli_usage_text()
            ),
        ),
    }
}
