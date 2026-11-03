//! Native CLI execution API. Binary cutover is a separate coherent consumer gate.
#![forbid(unsafe_code)]
use careops_ed::capture::{BoundedPartWriter, MODEL_CONTRACT_SHA256};
use careops_ed::disposition::api_runner::{
    check_safe_cli_path, compute_sha256_hex, execute_cli_args, validate_scenario_source,
    CancellationToken, ExitCodeDef, ValidationError, MAX_SCENARIO_PAYLOAD_BYTES,
};
use careops_ed::disposition::batch_runner::{
    resume_batch_from_checkpoint, run_batch_with_workers, BatchConfig,
};
use careops_ed::disposition::native_output::{
    export_native_batch, stream_patient_csv, ExportLimits, NativeFormat,
};
use careops_ed::{parse_scenario_json, ScenarioConfig};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub const MAX_CONTROL_BYTES: usize = 128 * 1024;
pub const MAX_STDOUT_BYTES: usize = 8 * 1024 * 1024;
pub const USAGE:&str="Usage: careops-ed capabilities [--require MODE]\n  careops-ed validate INPUT.json\n  careops-ed run INPUT.json --output-dir FRESH [--seed N] [--horizon T] [--format parquet|ipc|csv] [--stdout] [--backend cpu_serial]\n  careops-ed batch INPUT.json --output-dir FRESH [--replications N] [--workers N] [--checkpoint N] [--seed N] [--horizon T] [--format parquet|ipc|csv] [--stdout] [--backend cpu_replications]\n  careops-ed batch INPUT.json --output-dir EXISTING --resume EXISTING/capture/checkpoint.json [--workers N]\nDefault format is real Parquet/Zstd. --stdout requires CSV and streams patients only. Fresh output roots are never overwritten. Resume is capture-only; incomplete export cannot be retried in place.\n";
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
    let mut options = Options::default();
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" => options.help = true,
            "--output-dir" | "-o" => once(
                &mut options.destination,
                PathBuf::from(argument(args, &mut index)?),
            )?,
            "--backend" => once(&mut options.backend, argument(args, &mut index)?.to_owned())?,
            "--seed" => once(
                &mut options.seed,
                argument(args, &mut index)?
                    .parse()
                    .map_err(|_| usage("seed requires a u64"))?,
            )?,
            "--horizon" => once(
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
            "--stdout" => {
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
fn capability(args: &[String], stdout: &mut dyn Write) -> Result<(), ValidationError> {
    let result = execute_cli_args(args);
    if result.exit_code != ExitCodeDef::Success {
        return Err(error(result.exit_code, result.stderr));
    }
    stdout.write_all(result.stdout.as_bytes()).map_err(output)?;
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
    let result = execute_cli_args(&["capabilities".into(), "--require".into(), selected.into()]);
    if result.exit_code != ExitCodeDef::Success {
        return Err(error(result.exit_code, result.stderr));
    }
    Ok(())
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
