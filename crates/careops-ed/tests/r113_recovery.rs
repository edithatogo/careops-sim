#![cfg(feature = "native-data")]
use careops_ed::capture::{self, CaptureCheckpoint, PartStore};
use careops_ed::disposition::arrow_export::generate_arrow_export_bundle;
use careops_ed::disposition::batch_runner::{
    resume_batch_from_checkpoint, run_batch_with_workers, BatchConfig,
};
use careops_ed::parquet_capture::{ParquetPartStore, DEFAULT_LIMITS};
use careops_ed::ScenarioConfig;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

struct ObservedStore {
    inner: ParquetPartStore,
    root: PathBuf,
    stop: String,
    log: Mutex<()>,
}
impl ObservedStore {
    fn new(root: &Path, stop: &str) -> Self {
        let mut limits = DEFAULT_LIMITS;
        limits.max_rows = 1;
        Self {
            inner: ParquetPartStore::open(&root.join("capture"), limits).unwrap(),
            root: root.into(),
            stop: stop.into(),
            log: Mutex::new(()),
        }
    }
    fn pause(&self) {
        let mut ready = fs::File::create(self.root.join("ready")).unwrap();
        ready.write_all(self.stop.as_bytes()).unwrap();
        ready.sync_all().unwrap();
        loop {
            std::thread::park();
        }
    }
}
impl PartStore for ObservedStore {
    fn codec_id(&self) -> &str {
        self.inner.codec_id()
    }
    fn begin_replication(&self, index: usize) -> capture::Result<()> {
        let _guard = self.log.lock().unwrap();
        let mut log = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(self.root.join("starts"))
            .unwrap();
        writeln!(log, "{index}").unwrap();
        log.sync_all().unwrap();
        Ok(())
    }
    fn encode_part(
        &self,
        batch: &capture::CaptureBatch,
        writer: &mut dyn Write,
    ) -> capture::Result<()> {
        self.inner.encode_part(batch, writer)
    }
    fn write_part(
        &self,
        name: &str,
        batch: &capture::CaptureBatch,
        cap: usize,
    ) -> capture::Result<()> {
        self.inner.write_part(name, batch, cap)?;
        if (self.stop == "first_part" && batch.replication_index == 0 && batch.first_sequence == 0)
            || (self.stop == "mid_replication"
                && batch.replication_index == 0
                && batch.first_sequence == 2)
            || (self.stop == "part_after_ack"
                && batch.replication_index == 1
                && batch.first_sequence == 0)
        {
            self.pause();
        }
        Ok(())
    }
    fn read_part(&self, name: &str, cap: usize) -> capture::Result<Vec<u8>> {
        self.inner.read_part(name, cap)
    }
    fn decode_part(&self, bytes: &[u8]) -> capture::Result<capture::CaptureBatch> {
        self.inner.decode_part(bytes)
    }
    fn part_names(&self) -> capture::Result<Vec<String>> {
        self.inner.part_names()
    }
    fn is_complete(&self) -> capture::Result<bool> {
        self.inner.is_complete()
    }
    fn complete(&self, reps: &[capture::CapturedReplication]) -> capture::Result<()> {
        self.inner.complete(reps)?;
        if self.stop == "manifest" {
            self.pause();
        }
        Ok(())
    }
    fn commit_checkpoint(&self, path: &Path, bytes: &[u8]) -> capture::Result<()> {
        let checkpoint: CaptureCheckpoint = serde_json::from_slice(bytes).unwrap();
        let done = checkpoint.completed_replications.len();
        if self.stop == "before_ack" && done == 1 {
            self.pause();
        }
        if done == 1
            && matches!(
                self.stop.as_str(),
                "file_sync_failure" | "directory_sync_failure"
            )
        {
            return capture::durable_checkpoint_with_sync(path, bytes, &|file| {
                let directory = file.metadata()?.is_dir();
                if (self.stop == "file_sync_failure" && !directory)
                    || (self.stop == "directory_sync_failure" && directory)
                {
                    Err(std::io::Error::other(
                        "injected native checkpoint sync failure",
                    ))
                } else {
                    file.sync_all()
                }
            });
        }
        capture::durable_checkpoint(path, bytes)?;
        if (self.stop == "initial" && done == 0)
            || (self.stop == "ack" && done == 1)
            || (self.stop == "complete_checkpoint" && done == 4)
        {
            self.pause();
        }
        Ok(())
    }
    fn prepare_recovery(&self, pending: &[usize]) -> capture::Result<()> {
        self.inner.prepare_recovery(pending)
    }
    fn verify_complete(&self, reps: &[capture::CapturedReplication]) -> capture::Result<()> {
        self.inner.verify_complete(reps)
    }
}

// A native child fixture, invoked explicitly by the parent test below. It never
// runs a recovery scenario during ordinary discovery without its owned root.
#[test]
fn crash_fixture_child() {
    let Ok(root) = std::env::var("CAREOPS_R113_CHILD_ROOT") else {
        return;
    };
    let root = PathBuf::from(root);
    let mode = std::env::var("CAREOPS_R113_STOP").unwrap();
    let store = ObservedStore::new(&root, &mode);
    let mut limits = DEFAULT_LIMITS;
    limits.max_rows = 1;
    capture::run_captured_batch(
        &scenario(),
        &config(),
        limits,
        &root.join("capture"),
        &store,
    )
    .unwrap();
    panic!("child did not reach the requested publication boundary");
}

struct OwnedChild(std::process::Child);
impl Drop for OwnedChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn starts(root: &Path) -> Vec<usize> {
    fs::read_to_string(root.join("starts"))
        .unwrap_or_default()
        .lines()
        .map(|line| line.parse().unwrap())
        .collect()
}

#[test]
fn real_process_crashes_recover_preserved_parts_and_exact_rows_at_1_2_4_workers() {
    use std::collections::{BTreeMap, BTreeSet};
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};
    for mode in [
        "initial",
        "first_part",
        "mid_replication",
        "part_after_ack",
        "before_ack",
        "ack",
        "complete_checkpoint",
        "manifest",
    ] {
        for workers in [1, 2, 4] {
            let root = tempfile::tempdir().unwrap();
            fs::create_dir(root.path().join("capture")).unwrap();
            let log = fs::File::create(root.path().join("child.log")).unwrap();
            let mut child = OwnedChild(
                Command::new(std::env::current_exe().unwrap())
                    .args(["--exact", "crash_fixture_child", "--nocapture"])
                    .env("CAREOPS_R113_CHILD_ROOT", root.path())
                    .env("CAREOPS_R113_STOP", mode)
                    .env("POLARS_MAX_THREADS", "1")
                    .stdout(Stdio::from(log.try_clone().unwrap()))
                    .stderr(Stdio::from(log))
                    .spawn()
                    .unwrap(),
            );
            let deadline = Instant::now() + Duration::from_secs(20);
            while !root.path().join("ready").exists() {
                assert!(
                    child.0.try_wait().unwrap().is_none(),
                    "child failed: {}",
                    fs::read_to_string(root.path().join("child.log")).unwrap()
                );
                assert!(
                    Instant::now() < deadline,
                    "child publication boundary timed out: {mode}"
                );
                std::thread::sleep(Duration::from_millis(10));
            }
            child.0.kill().unwrap();
            assert!(!child.0.wait().unwrap().success());
            let dir = root.path().join("capture");
            let checkpoint = fs::read(dir.join("checkpoint.json")).unwrap();
            let cp: CaptureCheckpoint = serde_json::from_slice(&checkpoint).unwrap();
            let acknowledged: BTreeSet<_> = cp
                .completed_replications
                .iter()
                .map(|r| r.summary.replication_index)
                .collect();
            let retained: BTreeMap<_, _> = cp
                .completed_replications
                .iter()
                .flat_map(|r| &r.parts)
                .map(|p| (p.name.clone(), fs::read(dir.join(&p.name)).unwrap()))
                .collect();
            let original_starts = starts(root.path());
            let store = ObservedStore::new(root.path(), "none");
            let orphaned: BTreeMap<_, _> = store
                .part_names()
                .unwrap()
                .into_iter()
                .filter(|n| !retained.contains_key(n))
                .map(|n| {
                    let b = fs::read(dir.join(&n)).unwrap();
                    (n, b)
                })
                .collect();
            let mut cfg = config();
            cfg.num_workers = workers;
            let mut limits = DEFAULT_LIMITS;
            limits.max_rows = 1;
            // Invalid model binding must reject BEFORE archiving pending attempts
            // or dispatching a worker, even when genuine orphan parts exist.
            let mut wrong: serde_json::Value = serde_json::from_slice(&checkpoint).unwrap();
            wrong["model_contract_sha256"] = serde_json::json!("0".repeat(64));
            fs::write(
                dir.join("checkpoint.json"),
                serde_json::to_vec(&wrong).unwrap(),
            )
            .unwrap();
            assert!(
                capture::resume_captured_batch(&scenario(), &cfg, limits, &dir, &store).is_err()
            );
            assert_eq!(starts(root.path()), original_starts);
            assert!(!dir.join("recovery").exists());
            fs::write(dir.join("checkpoint.json"), &checkpoint).unwrap();
            let mut unknown_version: serde_json::Value =
                serde_json::from_slice(&checkpoint).unwrap();
            unknown_version["config"]["schema_version"] = serde_json::json!(99);
            fs::write(
                dir.join("checkpoint.json"),
                serde_json::to_vec(&unknown_version).unwrap(),
            )
            .unwrap();
            assert!(
                capture::resume_captured_batch(&scenario(), &cfg, limits, &dir, &store).is_err()
            );
            assert_eq!(starts(root.path()), original_starts);
            assert!(!dir.join("recovery").exists());
            fs::write(dir.join("checkpoint.json"), &checkpoint).unwrap();
            if mode == "first_part" && workers == 1 {
                fs::write(dir.join("unknown.part"), b"unrecognized namespace").unwrap();
                assert!(
                    capture::resume_captured_batch(&scenario(), &cfg, limits, &dir, &store)
                        .is_err()
                );
                assert!(!dir.join("recovery").exists());
                fs::remove_file(dir.join("unknown.part")).unwrap(); // Only our injected invalid fixture.
            }
            let recovered =
                capture::resume_captured_batch(&scenario(), &cfg, limits, &dir, &store).unwrap();
            assert_eq!(recovered.len(), 4);
            for (name, bytes) in &retained {
                assert_eq!(fs::read(dir.join(name)).unwrap(), *bytes);
            }
            for (name, bytes) in orphaned {
                let archive = fs::read_dir(dir.join("recovery"))
                    .unwrap()
                    .find_map(|a| {
                        let p = a.unwrap().path().join(&name);
                        p.is_file().then_some(p)
                    })
                    .unwrap();
                assert_eq!(fs::read(archive).unwrap(), bytes);
            }
            let calls = starts(root.path());
            for index in 0..4 {
                let expected = if original_starts.contains(&index) && !acknowledged.contains(&index)
                {
                    2
                } else {
                    1
                };
                assert_eq!(
                    calls.iter().filter(|&&i| i == index).count(),
                    expected,
                    "{mode}/{workers} index{index}"
                );
            }
            let repeated =
                capture::resume_captured_batch(&scenario(), &cfg, limits, &dir, &store).unwrap();
            assert_eq!(repeated, recovered);
            assert_eq!(starts(root.path()), calls);
            let final_doc = resume_batch_from_checkpoint(
                &dir.join("checkpoint.json"),
                &scenario(),
                workers,
                root.path(),
                None,
            )
            .unwrap();
            let rows = generate_arrow_export_bundle(&scenario(), &final_doc, None, None).unwrap();
            assert_eq!(rows.patient_records.len(), scenario().patients.len() * 4);
            for rep in &final_doc.replications {
                let mut input = scenario();
                input.seed = rep.seed;
                let oracle = careops_ed::run_scenario(&input).unwrap();
                assert_eq!(
                    (rep.arrivals, rep.started, rep.completed, rep.unfinished),
                    (
                        oracle.arrivals,
                        oracle.started,
                        oracle.completed,
                        oracle.unfinished
                    )
                );
                for patient in oracle.patients {
                    let row = rows
                        .patient_records
                        .iter()
                        .find(|r| {
                            r.replication_index == rep.replication_index
                                && r.patient_id == patient.patient_id
                        })
                        .unwrap();
                    assert_eq!(
                        (
                            row.arrival_tick,
                            row.start_tick,
                            row.completion_tick,
                            row.work_ticks,
                            row.wait_ticks,
                            row.elapsed_ticks
                        ),
                        (
                            patient.arrival_tick,
                            patient.start_tick,
                            patient.completion_tick,
                            patient.work_ticks,
                            patient.wait_ticks,
                            patient.elapsed_ticks
                        )
                    );
                }
            }
            assert!(resume_batch_from_checkpoint(
                &dir.join("checkpoint.json"),
                &scenario(),
                workers,
                root.path(),
                None
            )
            .is_err());
            assert_eq!(starts(root.path()), calls);
            println!("crash-window {mode}; resume workers{workers}: retained acknowledged parts, preserved attempts, exact coverage, no completed reruns");
        }
    }
}

#[test]
fn file_and_directory_sync_failures_never_publish_success_and_recover_explicitly() {
    for fault in ["file_sync_failure", "directory_sync_failure"] {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("capture")).unwrap();
        let store = ObservedStore::new(root.path(), fault);
        let mut limits = DEFAULT_LIMITS;
        limits.max_rows = 1;
        assert!(capture::run_captured_batch(
            &scenario(),
            &config(),
            limits,
            &root.path().join("capture"),
            &store
        )
        .is_err());
        assert!(!store.is_complete().unwrap());
        assert!(!root.path().join("batch_run.json").exists());
        let recovered = ObservedStore::new(root.path(), "none");
        let result = capture::resume_captured_batch(
            &scenario(),
            &config(),
            limits,
            &root.path().join("capture"),
            &recovered,
        )
        .unwrap();
        assert_eq!(result.len(), 4);
        assert!(recovered.is_complete().unwrap());
        println!("injected {fault}: failure propagated; no successful manifest; validated recovery passes");
    }
}

fn scenario() -> ScenarioConfig {
    serde_json::from_str(include_str!("../examples/nominal_mvp.json")).unwrap()
}
fn config() -> BatchConfig {
    BatchConfig {
        batch_id: "r113-durability".into(),
        scenario_id: scenario().scenario_id,
        base_seed: 2026,
        replications: 4,
        num_workers: 1,
        ..Default::default()
    }
}

#[test]
fn completed_manifest_without_batch_document_is_recoverable_without_republishing_parts() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("capture");
    fs::create_dir(&dir).unwrap();
    let store = ParquetPartStore::open(&dir, DEFAULT_LIMITS).unwrap();
    let reps =
        capture::run_captured_batch(&scenario(), &config(), DEFAULT_LIMITS, &dir, &store).unwrap();
    let original: Vec<_> = reps
        .iter()
        .flat_map(|r| &r.parts)
        .map(|p| (p.name.clone(), fs::read(dir.join(&p.name)).unwrap()))
        .collect();
    assert!(!root.path().join("batch_run.json").exists());
    let recovered = resume_batch_from_checkpoint(
        &dir.join("checkpoint.json"),
        &scenario(),
        4,
        root.path(),
        None,
    )
    .unwrap();
    assert_eq!(
        recovered.replications,
        reps.iter().map(|r| r.summary.clone()).collect::<Vec<_>>()
    );
    for (name, bytes) in original {
        assert_eq!(fs::read(dir.join(name)).unwrap(), bytes);
    }
    assert!(generate_arrow_export_bundle(&scenario(), &recovered, None, None).is_ok());
}

#[test]
fn export_rejects_every_checkpoint_envelope_drift() {
    for (key, value) in [
        ("schema_version", serde_json::json!(99)),
        ("batch_id", serde_json::json!("other")),
        ("base_seed", serde_json::json!(1)),
        ("total_replications", serde_json::json!(99)),
        ("status", serde_json::json!("in_progress")),
    ] {
        let root = tempfile::tempdir().unwrap();
        let doc =
            run_batch_with_workers(&scenario(), &config(), Some(root.path()), None, None).unwrap();
        let path = root.path().join("capture/checkpoint.json");
        let mut checkpoint: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        checkpoint[key] = value;
        fs::write(&path, serde_json::to_vec(&checkpoint).unwrap()).unwrap();
        assert!(
            generate_arrow_export_bundle(&scenario(), &doc, None, None).is_err(),
            "accepted {key} drift"
        );
    }
}

#[test]
fn compiled_model_is_bound_before_export_or_resume() {
    let root = tempfile::tempdir().unwrap();
    let doc =
        run_batch_with_workers(&scenario(), &config(), Some(root.path()), None, None).unwrap();
    let path = root.path().join("capture/checkpoint.json");
    let mut checkpoint: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(
        checkpoint["model_contract_sha256"].as_str().map(str::len),
        Some(64)
    );
    checkpoint["model_contract_sha256"] = serde_json::json!("0".repeat(64));
    fs::write(&path, serde_json::to_vec(&checkpoint).unwrap()).unwrap();
    assert!(generate_arrow_export_bundle(&scenario(), &doc, None, None).is_err());
    let parsed: CaptureCheckpoint = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let store = ParquetPartStore::open(path.parent().unwrap(), DEFAULT_LIMITS).unwrap();
    assert!(capture::resume_captured_batch(
        &scenario(),
        &parsed.metadata.config,
        DEFAULT_LIMITS,
        path.parent().unwrap(),
        &store
    )
    .is_err());
}

#[test]
fn internally_consistent_foreign_model_tag_is_readable_but_cannot_resume_execution() {
    // Synthetic provenance fixture, not a claim of physical cross-platform proof.
    let root = tempfile::tempdir().unwrap();
    let doc =
        run_batch_with_workers(&scenario(), &config(), Some(root.path()), None, None).unwrap();
    let dir = root.path().join("capture");
    let store = ParquetPartStore::open(&dir, DEFAULT_LIMITS).unwrap();
    let mut checkpoint: CaptureCheckpoint =
        serde_json::from_slice(&fs::read(dir.join("checkpoint.json")).unwrap()).unwrap();
    let foreign = "f".repeat(64);
    assert_ne!(foreign, capture::MODEL_CONTRACT_SHA256);
    checkpoint.metadata.model_contract_sha256 = foreign.clone();
    for replication in &mut checkpoint.completed_replications {
        replication.model_contract_sha256 = foreign.clone();
        for part in &mut replication.parts {
            let mut batch = store
                .decode_part(
                    &store
                        .read_part(&part.name, DEFAULT_LIMITS.max_bytes)
                        .unwrap(),
                )
                .unwrap();
            batch.model_contract_sha256 = foreign.clone();
            let mut bytes = vec![];
            store.encode_part(&batch, &mut bytes).unwrap();
            part.bytes = bytes.len();
            part.sha256 = careops_ed::disposition::api_runner::compute_sha256_hex(&bytes);
            fs::write(dir.join(&part.name), bytes).unwrap();
        }
    }
    fs::write(
        dir.join("checkpoint.json"),
        serde_json::to_vec(&checkpoint).unwrap(),
    )
    .unwrap();
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(dir.join("manifest.json")).unwrap()).unwrap();
    manifest["replications"] = serde_json::to_value(&checkpoint.completed_replications).unwrap();
    fs::write(
        dir.join("manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    assert!(generate_arrow_export_bundle(&scenario(), &doc, None, None).is_ok());
    assert!(
        capture::resume_captured_batch(&scenario(), &config(), DEFAULT_LIMITS, &dir, &store)
            .is_err()
    );
}

#[test]
fn unknown_nested_batch_version_rejects_export_resume_and_new_output_before_mutation() {
    let root = tempfile::tempdir().unwrap();
    let doc =
        run_batch_with_workers(&scenario(), &config(), Some(root.path()), None, None).unwrap();
    let dir = root.path().join("capture");
    let mut checkpoint: serde_json::Value =
        serde_json::from_slice(&fs::read(dir.join("checkpoint.json")).unwrap()).unwrap();
    checkpoint["config"]["schema_version"] = serde_json::json!(99);
    let bytes = serde_json::to_vec(&checkpoint).unwrap();
    fs::write(dir.join("checkpoint.json"), &bytes).unwrap();
    let store = ObservedStore::new(root.path(), "none");
    let preserved: Vec<_> = store
        .part_names()
        .unwrap()
        .into_iter()
        .map(|name| {
            let bytes = fs::read(dir.join(&name)).unwrap();
            (name, bytes)
        })
        .collect();
    assert!(generate_arrow_export_bundle(&scenario(), &doc, None, None).is_err());
    let mut unknown = config();
    unknown.schema_version = 99;
    assert!(
        capture::resume_captured_batch(&scenario(), &unknown, DEFAULT_LIMITS, &dir, &store)
            .is_err()
    );
    assert!(starts(root.path()).is_empty());
    assert!(!dir.join("recovery").exists());
    assert_eq!(fs::read(dir.join("checkpoint.json")).unwrap(), bytes);
    for (name, bytes) in preserved {
        assert_eq!(fs::read(dir.join(name)).unwrap(), bytes);
    }
    let output = root.path().join("new-output");
    assert!(run_batch_with_workers(&scenario(), &unknown, Some(&output), None, None).is_err());
    assert!(!output.exists());
}
