//! Native CLI execution API. Binary cutover is a separate coherent consumer gate.
#![forbid(unsafe_code)]
use careops_ed::capture::{BoundedPartWriter, MODEL_CONTRACT_SHA256};
use careops_ed::disposition::api_runner::{
    check_safe_cli_path, compute_sha256_hex, validate_scenario_source, CancellationToken,
    ExitCodeDef, ValidationError, MAX_SCENARIO_PAYLOAD_BYTES,
};
use careops_ed::disposition::batch_runner::{
    resume_batch_from_checkpoint, run_batch_with_workers, BatchConfig,
};
use careops_ed::disposition::native_output::{
    convert_native_dataset, export_native_batch, read_native_dataset, stream_patient_csv,
    ExportLimits, NativeFormat,
};
use careops_ed::{parse_scenario_json, ScenarioConfig};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub const MAX_CONTROL_BYTES: usize = 128 * 1024;
pub const MAX_STDOUT_BYTES: usize = 8 * 1024 * 1024;
pub const USAGE:&str="Usage: careops-ed capabilities [--require MODE]\n  careops-ed validate INPUT.json\n  careops-ed run INPUT.json --output-dir FRESH [--seed N] [--horizon T] [--format parquet|ipc|csv] [--stdout] [--backend cpu_serial]\n  careops-ed batch INPUT.json --output-dir FRESH [--replications N] [--workers N] [--checkpoint N] [--seed N] [--horizon T] [--format parquet|ipc|csv] [--stdout] [--backend cpu_replications]\n  careops-ed batch INPUT.json --output-dir EXISTING --resume EXISTING/capture/checkpoint.json [--workers N]\n  careops-ed export DATASET --output-dir FRESH [--format parquet|ipc|csv]\n  careops-ed compare LEFT_DATASET RIGHT_DATASET\nDefault format is real Parquet/Zstd. --stdout requires CSV and streams patients only. Fresh output roots are never overwritten. Resume is capture-only; incomplete export cannot be retried in place.\nSoftware verification of simulation mechanics does NOT constitute clinical domain validation for any healthcare site.\n";
fn error(code: ExitCodeDef, message: impl std::fmt::Display) -> ValidationError {
    ValidationError::new(code, message.to_string())
}
fn usage(message: impl std::fmt::Display) -> ValidationError {
    error(ExitCodeDef::UsageError, message)
}
fn invalid(message: impl std::fmt::Display) -> ValidationError {
    error(ExitCodeDef::MalformedConfig, message)
}
fn output(message: impl std::fmt::Display) -> ValidationError {
    error(ExitCodeDef::OutputWriteFailed, message)
}
#[derive(Default)]
struct Options {
    input: Option<PathBuf>,
    destination: Option<PathBuf>,
    backend: Option<String>,
    seed: Option<u64>,
    horizon: Option<u64>,
    replications: Option<usize>,
    workers: Option<usize>,
    checkpoint: Option<usize>,
    format: Option<NativeFormat>,
    resume: Option<PathBuf>,
    stream: bool,
    help: bool,
}
fn once<T>(slot: &mut Option<T>, value: T) -> Result<(), ValidationError> {
    if slot.is_some() {
        return Err(usage("option specified more than once"));
    }
    *slot = Some(value);
    Ok(())
}
fn argument<'a>(args: &'a [String], index: &mut usize) -> Result<&'a str, ValidationError> {
    *index += 1;
    args.get(*index)
        .filter(|s| !s.starts_with('-'))
        .map(String::as_str)
        .ok_or_else(|| usage("option requires one value"))
}
fn parse(args: &[String]) -> Result<Options, ValidationError> {
    let batch = args[0] == "batch";
    let execution = matches!(args[0].as_str(), "run" | "batch");
    let mut options = Options::default();
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" => options.help = true,
            "--output-dir" | "-o" => once(
                &mut options.destination,
                PathBuf::from(argument(args, &mut index)?),
            )?,
            "--backend" if execution => {
                once(&mut options.backend, argument(args, &mut index)?.to_owned())?
            }
            "--seed" if execution => once(
                &mut options.seed,
                argument(args, &mut index)?
                    .parse()
                    .map_err(|_| usage("seed requires a u64"))?,
            )?,
            "--horizon" if execution => once(
                &mut options.horizon,
                argument(args, &mut index)?
                    .parse()
                    .map_err(|_| usage("horizon requires a u64"))?,
            )?,
            "--replications" | "-r" if batch => once(
                &mut options.replications,
                argument(args, &mut index)?
                    .parse()
                    .map_err(|_| usage("replications requires an integer"))?,
            )?,
            "--workers" | "-w" if batch => once(
                &mut options.workers,
                argument(args, &mut index)?
                    .parse()
                    .map_err(|_| usage("workers requires an integer"))?,
            )?,
            "--checkpoint" if batch => once(
                &mut options.checkpoint,
                argument(args, &mut index)?
                    .parse()
                    .map_err(|_| usage("checkpoint requires an integer"))?,
            )?,
            "--resume" if batch => once(
                &mut options.resume,
                PathBuf::from(argument(args, &mut index)?),
            )?,
            "--format" => {
                let format = match argument(args, &mut index)? {
                    "parquet" => NativeFormat::Parquet,
                    "ipc" => NativeFormat::Ipc,
                    "csv" => NativeFormat::Csv,
                    _ => return Err(usage("format must be parquet, ipc or csv")),
                };
                once(&mut options.format, format)?;
            }
            "--stdout" if execution => {
                if options.stream {
                    return Err(usage("--stdout specified more than once"));
                }
                options.stream = true;
            }
            other if other.starts_with('-') => {
                return Err(usage(format!("unknown native option {other}")))
            }
            path => once(&mut options.input, PathBuf::from(path))?,
        }
        index += 1;
    }
    for value in [options.replications, options.workers, options.checkpoint]
        .into_iter()
        .flatten()
    {
        if value == 0 || value > 32 {
            return Err(usage(
                "replications, workers and checkpoint must be in1..=32",
            ));
        }
    }
    if options.horizon == Some(0) {
        return Err(invalid("horizon must be positive"));
    }
    Ok(options)
}
fn execution_modes() -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1,
        "scope": "careops-ed executable modes; no hardware probe",
        "modes": [
            {"id": "cpu_serial", "available": true, "parallelism": "none"},
            {"id": "cpu_replications", "available": true, "parallelism": "independent_replications"},
            {"id": "metal", "available": false, "reason": "ED Metal workload is not integrated or device/performance qualified"},
            {"id": "pdes", "available": false, "reason": "ED within-run PDES is not integrated or serial-oracle qualified"},
            {"id": "grpc", "available": false, "reason": "ED gRPC is not integrated or protocol/failure qualified"},
            {"id": "mpi", "available": false, "reason": "ED MPI is not integrated or runtime/failure qualified"},
            {"id": "distributed", "available": false, "reason": "ED multi-host execution is not integrated or recovery/parity qualified"}
        ]
    })
}

fn require_mode(selected: &str) -> Result<(), ValidationError> {
    match selected {
        "cpu_serial" | "cpu_replications" => Ok(()),
        "metal" | "pdes" | "grpc" | "mpi" | "distributed" => Err(error(
            ExitCodeDef::SimulationFailed,
            format!("careops-ed: backend '{selected}' is not integrated and qualified; inspect 'capabilities'"),
        )),
        _ => Err(usage(format!("careops-ed: unknown backend '{selected}'; inspect 'capabilities'"))),
    }
}
fn capability(args: &[String], stdout: &mut dyn Write) -> Result<(), ValidationError> {
    if args.len() != 1 {
        if args.len() != 3 || args[1] != "--require" {
            return Err(usage("careops-ed: capabilities [--require <mode>]"));
        }
        require_mode(&args[2])?;
    }
    stdout
        .write_all(&control(&execution_modes())?)
        .map_err(output)?;
    stdout.write_all(b"\n").map_err(output)?;
    stdout.flush().map_err(output)
}
fn mode(command: &str, selected: Option<&str>) -> Result<(), ValidationError> {
    let expected = if command == "run" {
        "cpu_serial"
    } else {
        "cpu_replications"
    };
    let selected = selected.unwrap_or(expected);
    if ["cpu_serial", "cpu_replications"].contains(&selected) && selected != expected {
        return Err(usage("backend is not available for this command"));
    }
    require_mode(selected)
}
fn bounded_read(path: &Path, limit: usize) -> Result<Vec<u8>, ValidationError> {
    let metadata = fs::symlink_metadata(path).map_err(|e| error(ExitCodeDef::FileNotFound, e))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > limit as u64 {
        return Err(invalid(
            "input/control must be a bounded regular non-symlink file",
        ));
    }
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|e| error(ExitCodeDef::FileNotFound, e))?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| error(ExitCodeDef::FileNotFound, e))?;
    if bytes.len() > limit {
        return Err(invalid("input/control byte budget exceeded"));
    }
    Ok(bytes)
}
fn control<T: Serialize>(value: &T) -> Result<Vec<u8>, ValidationError> {
    let mut writer = BoundedPartWriter::new(Vec::new(), MAX_CONTROL_BYTES);
    serde_json::to_writer_pretty(&mut writer, value).map_err(output)?;
    Ok(writer.into_inner())
}
fn publish_control(path: &Path, bytes: &[u8]) -> Result<(), ValidationError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(output)?;
    file.write_all(bytes).map_err(output)?;
    file.flush().map_err(output)?;
    file.sync_all().map_err(output)?;
    File::open(
        path.parent()
            .ok_or_else(|| output("control has no parent"))?,
    )
    .and_then(|f| f.sync_all())
    .map_err(output)
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Invocation {
    pub schema: String,
    pub command: String,
    pub raw_input_sha256: String,
    pub parsed_input_sha256: String,
    pub model_contract_sha256: String,
    pub base_seed: u64,
    pub replications: usize,
    pub initial_workers: usize,
    pub horizon_override: Option<u64>,
    pub checkpoint_interval: usize,
    pub format: NativeFormat,
}
impl Invocation {
    fn config(&self, scenario: &ScenarioConfig, workers: usize) -> BatchConfig {
        BatchConfig {
            batch_id: format!("native-{}-seed{}", scenario.scenario_id, self.base_seed),
            scenario_id: scenario.scenario_id.clone(),
            base_seed: self.base_seed,
            replications: self.replications,
            num_workers: workers,
            horizon_override: self.horizon_override,
            checkpoint_interval: Some(self.checkpoint_interval),
            ..Default::default()
        }
    }
    fn validate(&self, scenario: &ScenarioConfig, raw: &[u8]) -> Result<(), ValidationError> {
        if self.schema != "careops.ed.cli-invocation.v1"
            || !matches!(self.command.as_str(), "run" | "batch")
            || self.raw_input_sha256 != compute_sha256_hex(raw)
            || self.parsed_input_sha256
                != compute_sha256_hex(&serde_json::to_vec(scenario).map_err(invalid)?)
            || self.model_contract_sha256 != MODEL_CONTRACT_SHA256
            || self.replications == 0
            || self.replications > 32
            || self.initial_workers == 0
            || self.initial_workers > 32
            || self.checkpoint_interval == 0
            || self.checkpoint_interval > 32
            || self.horizon_override == Some(0)
            || (self.command == "run" && self.replications != 1)
        {
            return Err(invalid("invocation input/model/config binding mismatch"));
        }
        let rows = scenario
            .patients
            .len()
            .checked_mul(self.replications)
            .and_then(|n| n.checked_add(self.replications))
            .and_then(|n| n.checked_add(5))
            .ok_or_else(|| invalid("native CLI row count overflow"))?;
        if rows > ExportLimits::default().max_total_rows {
            return Err(invalid("native CLI total export row budget exceeded"));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Completion {
    pub schema: String,
    pub status: String,
    pub invocation_file: String,
    pub invocation_sha256: String,
    pub batch_file: String,
    pub batch_sha256: String,
    pub dataset_manifest: String,
    pub dataset_manifest_sha256: String,
}
fn validate_identifiers(scenario: &ScenarioConfig) -> Result<(), ValidationError> {
    if scenario.scenario_id.len() > 1024
        || scenario.time_unit.len() > 128
        || scenario.patients.iter().any(|p| {
            [&p.patient_id, &p.resource_id, &p.task_class_id]
                .iter()
                .any(|s| s.len() > 4096)
        })
        || scenario
            .capacity_location
            .resource_buckets
            .iter()
            .any(|r| r.resource_id.len() > 4096 || r.location_id.len() > 4096)
    {
        return Err(invalid("native CLI identifier budget exceeded"));
    }
    Ok(())
}
fn create_private_root(root: &Path) -> Result<(), ValidationError> {
    let builder = fs::DirBuilder::new();
    #[cfg(unix)]
    let builder = {
        use std::os::unix::fs::DirBuilderExt;
        let mut builder = builder;
        builder.mode(0o700);
        builder
    };
    builder.create(root).map_err(|e| {
        if e.kind() == std::io::ErrorKind::AlreadyExists {
            error(
                ExitCodeDef::FileAlreadyExists,
                "native output root already exists; no overwrite",
            )
        } else {
            output(e)
        }
    })
}
fn export(args: &[String], stdout: &mut dyn Write) -> Result<(), ValidationError> {
    let options = parse(args)?;
    if options.backend.is_some()
        || options.seed.is_some()
        || options.horizon.is_some()
        || options.resume.is_some()
        || options.stream
        || options.help
    {
        return Err(usage(
            "export accepts DATASET --output-dir FRESH [--format parquet|ipc|csv]",
        ));
    }
    let source = options
        .input
        .ok_or_else(|| usage("export requires one dataset"))?;
    let destination = options
        .destination
        .ok_or_else(|| usage("export requires --output-dir"))?;
    check_safe_cli_path(&source)?;
    check_safe_cli_path(&destination)?;
    let dataset = read_native_dataset(&source)?;
    create_private_root(&destination)?;
    let manifest = convert_native_dataset(
        &dataset,
        &destination,
        options.format.unwrap_or(NativeFormat::Parquet),
        ExportLimits::default(),
    )?;
    let report = serde_json::json!({"schema":"careops.ed.cli-export.v1","status":"complete",
        "source_manifest_sha256":dataset.manifest_sha256(),"format":manifest.format,
        "dataset_manifest":"manifest.json", "dataset_manifest_sha256":compute_sha256_hex(&bounded_read(&destination.join("manifest.json"), ExportLimits::default().max_manifest_bytes)?),
        "conversion_receipt":"conversion.json", "conversion_receipt_sha256":compute_sha256_hex(&bounded_read(&destination.join("conversion.json"), MAX_CONTROL_BYTES)?)});
    stdout.write_all(&control(&report)?).map_err(output)?;
    stdout.flush().map_err(output)
}

fn compare(args: &[String], stdout: &mut dyn Write) -> Result<(), ValidationError> {
    if args.len() != 3 || args[1..].iter().any(|a| a.starts_with('-')) {
        return Err(usage("compare requires exactly two native datasets"));
    }
    for path in &args[1..] {
        check_safe_cli_path(Path::new(path))?;
    }
    let left = read_native_dataset(Path::new(&args[1]))?;
    let right = read_native_dataset(Path::new(&args[2]))?;
    let a = left.manifest();
    let b = right.manifest();
    if a.time_unit != b.time_unit
        || a.runtime_revision != b.runtime_revision
        || a.data_revision != b.data_revision
        || a.polars_revision != b.polars_revision
        || a.model_contract_sha256 != b.model_contract_sha256
    {
        return Err(error(
            ExitCodeDef::IncompatibleScenario,
            "comparison requires compatible units and model/runtime contracts",
        ));
    }
    let a = left.metrics();
    let b = right.metrics();
    if a.height() != 5 || b.height() != 5 {
        return Err(invalid(
            "comparison requires the five qualified native metrics",
        ));
    }
    let names_a = a.column("name").map_err(invalid)?.str().map_err(invalid)?;
    let names_b = b.column("name").map_err(invalid)?.str().map_err(invalid)?;
    let mut index = std::collections::BTreeMap::new();
    for i in 0..b.height() {
        let name = names_b.get(i);
        if index
            .insert(name.ok_or_else(|| invalid("null metric name"))?, i)
            .is_some()
        {
            return Err(invalid("duplicate metric name"));
        }
    }
    let mut seen = std::collections::BTreeSet::new();
    let values_a = a.column("value").map_err(invalid)?.f64().map_err(invalid)?;
    let values_b = b.column("value").map_err(invalid)?.f64().map_err(invalid)?;
    let mut metrics = Vec::with_capacity(5);
    for i in 0..a.height() {
        let name = names_a.get(i);
        let name = name.ok_or_else(|| invalid("null metric name"))?;
        if !seen.insert(name) {
            return Err(invalid("duplicate metric name"));
        }
        let j = *index
            .get(name)
            .ok_or_else(|| invalid("metric key mismatch"))?;
        let expected = match name {
            "mean_wait_ticks" => [
                left.manifest().time_unit.as_str(),
                "included_replications",
                "mean_of_replication_wait_means",
                "replication",
                "equal_replication_zero_for_empty",
            ],
            "mean_elapsed_ticks" => [
                left.manifest().time_unit.as_str(),
                "completed_patients",
                "pooled_completed_patient_mean",
                "completed_patient",
                "equal_completed_patient",
            ],
            "completion_rate" => [
                "ratio",
                "total_arrivals",
                "pooled_completed_over_arrivals",
                "arrival",
                "pooled_counts",
            ],
            "throughput_per_horizon_tick" => [
                "patients_per_tick",
                "per_replication_horizon_ticks",
                "mean_of_replication_throughputs",
                "replication",
                "equal_replication",
            ],
            "censored_presentation_rate" => [
                "ratio",
                "included_replications",
                "mean_of_replication_unfinished_rates",
                "replication",
                "equal_replication_zero_for_empty",
            ],
            _ => return Err(invalid("unqualified comparison metric")),
        };
        for (field, expected) in [
            "unit",
            "denominator_name",
            "estimand",
            "sample_unit",
            "weighting",
        ]
        .into_iter()
        .zip(expected)
        {
            if a.column(field)
                .map_err(invalid)?
                .str()
                .map_err(invalid)?
                .get(i)
                != Some(expected)
                || b.column(field)
                    .map_err(invalid)?
                    .str()
                    .map_err(invalid)?
                    .get(j)
                    != Some(expected)
            {
                return Err(error(
                    ExitCodeDef::IncompatibleScenario,
                    "comparison unit or estimand mismatch",
                ));
            }
        }
        let av = values_a.get(i);
        let bv = values_b.get(j);
        let delta = av.zip(bv).map(|(av, bv)| bv - av);
        if delta.is_some_and(|n| !n.is_finite()) {
            return Err(invalid("comparison delta overflow"));
        }
        metrics.push(
            serde_json::json!({"name":name,"left":av,"right":bv,"delta":delta,
            "unit":a.column("unit").map_err(invalid)?.str().map_err(invalid)?.get(i),
            "estimand":a.column("estimand").map_err(invalid)?.str().map_err(invalid)?.get(i)}),
        );
    }
    let report = serde_json::json!({"schema":"careops.ed.native-comparison.v1",
        "left_manifest_sha256":left.manifest_sha256(),"right_manifest_sha256":right.manifest_sha256(),
        "uncertainty_method":"not_estimated","uncertainty_reason":"paired_difference_method_unqualified",
        "metrics":metrics});
    stdout.write_all(&control(&report)?).map_err(output)?;
    stdout.flush().map_err(output)
}

/// Execute native run/batch into a real writer. `main` adopts this API only at
/// the coherent benchmark/consumer cutover. Cancellation retains original R11
/// capture semantics; incomplete export is never retried in place.
pub fn execute(
    args: &[String],
    stdout: &mut dyn Write,
    cancel: Option<&CancellationToken>,
) -> Result<(), ValidationError> {
    if args.is_empty() || args.len() > 32 || args.iter().any(|s| s.len() > 16384) {
        return Err(usage(USAGE));
    }
    match args[0].as_str() {
        "capabilities" => return capability(args, stdout),
        "export" => return export(args, stdout),
        "compare" => return compare(args, stdout),
        "--help" | "-h" if args.len() == 1 => {
            stdout.write_all(USAGE.as_bytes()).map_err(output)?;
            return stdout.flush().map_err(output);
        }
        "validate" => {
            if args.len() != 2 {
                return Err(usage("validate requires one input"));
            }
            let path = Path::new(&args[1]);
            check_safe_cli_path(path)?;
            let bytes = bounded_read(path, MAX_SCENARIO_PAYLOAD_BYTES)?;
            let source = std::str::from_utf8(&bytes).map_err(invalid)?;
            let report = validate_scenario_source(source)?;
            stdout.write_all(&control(&report)?).map_err(output)?;
            return stdout.flush().map_err(output);
        }
        "run" | "batch" => {}
        _ => return Err(usage("unknown native command")),
    }
    let options = parse(args)?;
    mode(&args[0], options.backend.as_deref())?;
    if options.help {
        stdout.write_all(USAGE.as_bytes()).map_err(output)?;
        return stdout.flush().map_err(output);
    }
    if cancel.is_some_and(|token| token.is_cancelled()) {
        return Err(error(
            ExitCodeDef::Interrupted,
            "native execution cancelled before input/output",
        ));
    }
    if options.resume.is_none() && options.stream && options.format != Some(NativeFormat::Csv) {
        return Err(usage("--stdout requires explicit CSV format"));
    }
    let input = options
        .input
        .as_ref()
        .ok_or_else(|| usage("native execution requires one input"))?;
    let root = options
        .destination
        .as_ref()
        .ok_or_else(|| usage("native execution requires --output-dir"))?;
    check_safe_cli_path(input)?;
    check_safe_cli_path(root)?;
    if let Some(path) = &options.resume {
        check_safe_cli_path(path)?;
    }
    let raw = bounded_read(input, MAX_SCENARIO_PAYLOAD_BYTES)?;
    let source = std::str::from_utf8(&raw).map_err(invalid)?;
    let scenario = parse_scenario_json(source).map_err(invalid)?;
    validate_identifiers(&scenario)?;
    let (invocation, invocation_bytes, workers) = if let Some(checkpoint) = &options.resume {
        let metadata = fs::symlink_metadata(root).map_err(output)?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(invalid("resume root must be a real directory"));
        }
        if root.join("manifest.json").exists() {
            return Err(error(
                ExitCodeDef::FileAlreadyExists,
                "native artifacts are already complete",
            ));
        }
        if root.join("data").exists() || root.join("batch_run.json").exists() {
            return Err(invalid("resume is capture-only; completed capture or incomplete export needs a fresh export destination"));
        }
        let invocation_bytes = bounded_read(&root.join("invocation.json"), MAX_CONTROL_BYTES)?;
        let invocation_digest = format!("{}\n", compute_sha256_hex(&invocation_bytes));
        if bounded_read(&root.join("invocation.sha256"), 65)? != invocation_digest.as_bytes() {
            return Err(invalid("immutable invocation checksum mismatch"));
        }
        let invocation: Invocation = serde_json::from_slice(&invocation_bytes).map_err(invalid)?;
        invocation.validate(&scenario, &raw)?;
        if bounded_read(&root.join("input.json"), MAX_SCENARIO_PAYLOAD_BYTES)? != raw {
            return Err(invalid("resume raw input differs from immutable original"));
        }
        if options.seed.is_some_and(|v| v != invocation.base_seed)
            || options
                .horizon
                .is_some_and(|v| Some(v) != invocation.horizon_override)
            || options
                .replications
                .is_some_and(|v| v != invocation.replications)
            || options
                .checkpoint
                .is_some_and(|v| v != invocation.checkpoint_interval)
            || options.format.is_some_and(|v| v != invocation.format)
        {
            return Err(invalid("resume cannot change immutable invocation"));
        }
        let capture = root.join("capture");
        let expected = capture.join("checkpoint.json");
        for path in [&capture, &expected] {
            if fs::symlink_metadata(path)
                .map_err(invalid)?
                .file_type()
                .is_symlink()
            {
                return Err(invalid("resume checkpoint path cannot be a symlink"));
            }
        }
        if checkpoint.canonicalize().map_err(invalid)?
            != expected.canonicalize().map_err(invalid)?
        {
            return Err(invalid(
                "resume checkpoint must belong to this exact output root",
            ));
        }
        let checkpoint_state = careops_ed::capture::read_checkpoint(checkpoint).map_err(invalid)?;
        if checkpoint_state.metadata.config
            != invocation.config(&scenario, checkpoint_state.metadata.config.num_workers)
            || checkpoint_state.metadata.config.num_workers == 0
            || checkpoint_state.metadata.config.num_workers > 32
            || checkpoint_state.metadata.limits != careops_ed::parquet_capture::DEFAULT_LIMITS
            || checkpoint_state.metadata.codec_id != careops_ed::parquet_capture::CODEC
            || checkpoint_state.metadata.model_contract_sha256 != invocation.model_contract_sha256
        {
            return Err(invalid(
                "native checkpoint differs from immutable invocation or qualified capture limits",
            ));
        }
        let workers = options.workers.unwrap_or(invocation.initial_workers);
        (invocation, invocation_bytes, workers)
    } else {
        let invocation = Invocation {
            schema: "careops.ed.cli-invocation.v1".into(),
            command: args[0].clone(),
            raw_input_sha256: compute_sha256_hex(&raw),
            parsed_input_sha256: compute_sha256_hex(
                &serde_json::to_vec(&scenario).map_err(invalid)?,
            ),
            model_contract_sha256: MODEL_CONTRACT_SHA256.into(),
            base_seed: options.seed.unwrap_or(scenario.seed),
            replications: options
                .replications
                .unwrap_or(if args[0] == "run" { 1 } else { 4 }),
            initial_workers: options
                .workers
                .unwrap_or(if args[0] == "run" { 1 } else { 2 }),
            horizon_override: options.horizon,
            checkpoint_interval: options
                .checkpoint
                .unwrap_or(if args[0] == "run" { 1 } else { 2 }),
            format: options.format.unwrap_or_default(),
        };
        invocation.validate(&scenario, &raw)?;
        let invocation_bytes = control(&invocation)?;
        let workers = invocation.initial_workers;
        (invocation, invocation_bytes, workers)
    };
    if options.stream && invocation.format != NativeFormat::Csv {
        return Err(usage("--stdout requires explicit CSV format"));
    }
    let document = if let Some(path) = &options.resume {
        resume_batch_from_checkpoint(path, &scenario, workers, root, cancel)?
    } else {
        create_private_root(root)?;
        publish_control(&root.join("input.json"), &raw)?;
        publish_control(&root.join("invocation.json"), &invocation_bytes)?;
        publish_control(
            &root.join("invocation.sha256"),
            format!("{}\n", compute_sha256_hex(&invocation_bytes)).as_bytes(),
        )?;
        run_batch_with_workers(
            &scenario,
            &invocation.config(&scenario, workers),
            Some(root),
            cancel,
            None,
        )?
    };
    let destination = root.join("data");
    create_private_root(&destination)?;
    let manifest = export_native_batch(
        &scenario,
        &document,
        &destination,
        invocation.format,
        ExportLimits::default(),
    )?;
    if manifest.parsed_input_sha256 != invocation.parsed_input_sha256
        || manifest.model_contract_sha256 != invocation.model_contract_sha256
    {
        return Err(invalid("native dataset differs from immutable invocation"));
    }
    let completion = Completion {
        schema: "careops.ed.cli-artifacts.v1".into(),
        status: "complete".into(),
        invocation_file: "invocation.json".into(),
        invocation_sha256: compute_sha256_hex(&invocation_bytes),
        batch_file: "batch_run.json".into(),
        batch_sha256: compute_sha256_hex(&bounded_read(
            &root.join("batch_run.json"),
            MAX_CONTROL_BYTES,
        )?),
        dataset_manifest: "data/manifest.json".into(),
        dataset_manifest_sha256: compute_sha256_hex(&bounded_read(
            &destination.join("manifest.json"),
            ExportLimits::default().max_manifest_bytes,
        )?),
    };
    let completion_bytes = control(&completion)?;
    publish_control(&root.join("manifest.json"), &completion_bytes)?;
    if options.stream {
        stream_patient_csv(&scenario, &document, stdout, MAX_STDOUT_BYTES)?;
    } else {
        stdout.write_all(&completion_bytes).map_err(output)?;
        stdout.write_all(b"\n").map_err(output)?;
    }
    stdout.flush().map_err(output)
}
