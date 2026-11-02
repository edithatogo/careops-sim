//! Native batch integration. All exported rows originate in the worker execution.
use crate::capture;
use crate::disposition::api_runner::{CancellationToken, ExitCodeDef, ValidationError};
use crate::disposition::batch_runner::{
    atomic_write_file, compute_batch_aggregates, current_epoch_ms, BatchConfig, BatchProgress,
    BatchRunDocument,
};
use crate::parquet_capture::{CaptureSource, ParquetPartStore, DEFAULT_LIMITS};
use crate::ScenarioConfig;
use std::io::Write;
use std::path::Path;

pub(crate) fn error(e: careops_core::experiment::ExperimentError) -> ValidationError {
    use careops_core::experiment::ExperimentErrorKind as K;
    ValidationError::new(
        match e.kind {
            K::Interrupted => ExitCodeDef::Interrupted,
            K::OutputWriteFailed => ExitCodeDef::OutputWriteFailed,
            K::FileNotFound => ExitCodeDef::FileNotFound,
            K::MalformedConfig => ExitCodeDef::MalformedConfig,
            _ => ExitCodeDef::SimulationFailed,
        },
        e.message,
    )
}
fn finish(
    config: &BatchConfig,
    source: CaptureSource,
    results: Vec<capture::CapturedReplication>,
    output: Option<&Path>,
) -> Result<BatchRunDocument, ValidationError> {
    let summaries: Vec<_> = results.into_iter().map(|r| r.summary).collect();
    let (mean, se, completion) = compute_batch_aggregates(&summaries);
    let doc = BatchRunDocument {
        schema_version: 2,
        batch_id: config.batch_id.clone(),
        scenario_id: config.scenario_id.clone(),
        status: "complete".into(),
        num_workers: config.num_workers.max(1),
        total_replications: summaries.len(),
        replications: summaries,
        aggregate_mean_wait: mean,
        aggregate_std_error: se,
        mean_completion_rate: completion,
        completed_at_epoch_ms: current_epoch_ms(),
        capture: Some(source),
    };
    if let Some(root) = output {
        let bytes = serde_json::to_vec_pretty(&doc)
            .map_err(|e| ValidationError::new(ExitCodeDef::MalformedConfig, e.to_string()))?;
        ParquetPartStore::open(root, DEFAULT_LIMITS)
            .map_err(error)?
            .publish("batch_run.json", |f| {
                f.write_all(&bytes).map_err(|e| {
                    careops_core::experiment::ExperimentError::new(
                        careops_core::experiment::ExperimentErrorKind::OutputWriteFailed,
                        e.to_string(),
                    )
                })
            })
            .map_err(error)?;
        let partial = root.join("batch_run.partial");
        if partial.exists() {
            std::fs::remove_file(partial)
                .map_err(|e| ValidationError::new(ExitCodeDef::OutputWriteFailed, e.to_string()))?;
        }
    }
    Ok(doc)
}
pub(crate) fn run(
    scenario: &ScenarioConfig,
    config: &BatchConfig,
    output: Option<&Path>,
    cancel: Option<&CancellationToken>,
    progress: Option<&(dyn Fn(BatchProgress) + Send + Sync)>,
) -> Result<BatchRunDocument, ValidationError> {
    if cancel.is_some_and(|token| token.is_cancelled()) {
        return Err(ValidationError::new(
            ExitCodeDef::Interrupted,
            "batch cancelled before dispatch",
        ));
    }
    let source = if let Some(root) = output {
        if root.join("batch_run.json").exists() {
            return Err(ValidationError::new(
                ExitCodeDef::OutputWriteFailed,
                "completed batch cannot be overwritten",
            ));
        }

        std::fs::create_dir_all(root)
            .map_err(|e| ValidationError::new(ExitCodeDef::OutputWriteFailed, e.to_string()))?;
        let dir = root.join("capture");
        std::fs::create_dir(&dir)
            .map_err(|e| ValidationError::new(ExitCodeDef::OutputWriteFailed, e.to_string()))?;
        CaptureSource::persistent(
            dir.canonicalize()
                .map_err(|e| ValidationError::new(ExitCodeDef::OutputWriteFailed, e.to_string()))?,
            DEFAULT_LIMITS,
        )
    } else {
        CaptureSource::temporary().map_err(error)?
    };
    let store = ParquetPartStore::open(&source.directory, source.limits).map_err(error)?;
    let adapter = |p: careops_core::experiment::BatchProgress| {
        if let Some(cb) = progress {
            cb(BatchProgress {
                batch_id: p.batch_id,
                total_replications: p.total_replications,
                completed_replications: p.completed_replications,
                percent_complete: p.percent_complete,
                running_mean_wait: p.running_metric,
            });
        }
    };
    let results = capture::run_captured_batch_controlled(
        scenario,
        config,
        source.limits,
        &source.directory,
        &store,
        cancel,
        Some(&adapter),
    )
    .map_err(error);
    if let Err(e) = &results {
        if let Some(root) = output {
            let marker = crate::disposition::batch_runner::PartialBatchMarker {
                status: "incomplete".into(),
                batch_id: config.batch_id.clone(),
                completed_replications: 0,
                total_replications: config.replications,
                error: e.message.clone(),
            };
            atomic_write_file(
                &root.join("batch_run.partial"),
                &serde_json::to_vec(&marker).map_err(|e| {
                    ValidationError::new(ExitCodeDef::MalformedConfig, e.to_string())
                })?,
            )?;
        }
    }
    finish(config, source, results?, output)
}
pub(crate) fn resume(
    path: &Path,
    scenario: &ScenarioConfig,
    workers: usize,
    output: &Path,
    cancel: Option<&CancellationToken>,
) -> Result<BatchRunDocument, ValidationError> {
    if output.join("batch_run.json").exists() {
        return Err(ValidationError::new(
            ExitCodeDef::OutputWriteFailed,
            "completed batch cannot be overwritten",
        ));
    }
    let checkpoint = capture::read_checkpoint(path).map_err(error)?;
    let mut config = checkpoint.metadata.config;
    config.num_workers = workers;
    let source = CaptureSource::persistent(
        path.parent()
            .ok_or_else(|| {
                ValidationError::new(ExitCodeDef::MalformedConfig, "checkpoint has no parent")
            })?
            .canonicalize()
            .map_err(|e| ValidationError::new(ExitCodeDef::FileNotFound, e.to_string()))?,
        checkpoint.metadata.limits,
    );
    let store = ParquetPartStore::open(&source.directory, source.limits).map_err(error)?;
    let results = capture::resume_captured_batch_controlled(
        scenario,
        &config,
        source.limits,
        &source.directory,
        &store,
        cancel,
    )
    .map_err(error)?;
    finish(&config, source, results, Some(output))
}
/// Validates persisted source before returning immutable replication partitions.
pub(crate) fn export_source(
    scenario: &ScenarioConfig,
    doc: &BatchRunDocument,
) -> Result<
    (
        ParquetPartStore,
        Vec<capture::CapturedReplication>,
        capture::CaptureMetadata,
    ),
    ValidationError,
> {
    let source = doc.capture.as_ref().ok_or_else(|| {
        ValidationError::new(
            ExitCodeDef::MalformedConfig,
            "batch has no original capture; export cannot rerun simulations",
        )
    })?;
    let store = ParquetPartStore::open(&source.directory, source.limits).map_err(error)?;
    let checkpoint =
        capture::read_checkpoint(&source.directory.join("checkpoint.json")).map_err(error)?;
    let meta = checkpoint.metadata;
    let expected =
        capture::metadata(scenario, &meta.config, source.limits, &store).map_err(error)?;
    let reps = store.completed_replications().map_err(error)?;
    if meta != expected
        || doc.status != "complete"
        || doc.batch_id != meta.config.batch_id
        || doc.scenario_id != meta.config.scenario_id
        || doc.total_replications != meta.config.replications
        || reps.len() != doc.total_replications
        || reps.iter().map(|r| r.summary.clone()).collect::<Vec<_>>() != doc.replications
        || checkpoint.completed_replications != reps
        || !checkpoint.pending_indices.is_empty()
    {
        return Err(ValidationError::new(
            ExitCodeDef::MalformedConfig,
            "capture/document binding mismatch",
        ));
    }
    let (mean, se, rate) = compute_batch_aggregates(&doc.replications);
    if (
        doc.aggregate_mean_wait,
        doc.aggregate_std_error,
        doc.mean_completion_rate,
    ) != (mean, se, rate)
    {
        return Err(ValidationError::new(
            ExitCodeDef::MalformedConfig,
            "batch aggregate mismatch",
        ));
    }
    capture::validate_parts(&reps, &meta, scenario, &store).map_err(error)?;
    Ok((store, reps, meta))
}
