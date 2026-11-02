//! Real capture orchestration with a filesystem FIXTURE codec/store.
//! JSON here is not a production trace format or crash-durability qualification.
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use careops_core::experiment::{ExperimentError, ExperimentErrorKind};
use careops_ed::capture::*;
use careops_ed::disposition::api_runner::compute_sha256_hex;
use careops_ed::disposition::batch_runner::{allocate_replication_seeds, BatchConfig};
use careops_ed::{parse_scenario_json, run_scenario, PatientResult, ScenarioConfig};

fn failure(e: impl std::fmt::Display) -> ExperimentError {
    ExperimentError::new(ExperimentErrorKind::OutputWriteFailed, e.to_string())
}
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "r111-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    BeforeWrite,
    PartialWrite,
    Flush,
    Sync,
    Rename,
    AfterCommit,
    Manifest,
}
#[derive(Default)]
struct Gate {
    state: Mutex<(bool, bool)>,
    changed: Condvar,
}
impl Gate {
    fn block(&self) {
        let mut state = self.state.lock().unwrap();
        state.0 = true;
        self.changed.notify_all();
        while !state.1 {
            state = self.changed.wait(state).unwrap();
        }
    }
    fn wait_entered(&self) {
        let (state, timeout) = self
            .changed
            .wait_timeout_while(self.state.lock().unwrap(), Duration::from_secs(10), |s| {
                !s.0
            })
            .unwrap();
        assert!(
            !timeout.timed_out() && state.0,
            "producer never reached real write boundary"
        );
    }
    fn release(&self) {
        self.state.lock().unwrap().1 = true;
        self.changed.notify_all();
    }
}
struct Release(Arc<Gate>);
impl Drop for Release {
    fn drop(&mut self) {
        self.0.release();
    }
}

struct FsFixture {
    root: PathBuf,
    fault: Mutex<Option<(Stage, usize)>>,
    starts: Mutex<BTreeMap<usize, usize>>,
    acknowledged: Mutex<Vec<(usize, usize)>>,
    gate: Option<Arc<Gate>>,
    padding: usize,
}
impl FsFixture {
    fn new(root: PathBuf) -> Self {
        fs::create_dir(&root).unwrap();
        Self {
            root,
            fault: Mutex::new(None),
            starts: Mutex::new(BTreeMap::new()),
            acknowledged: Mutex::new(vec![]),
            gate: None,
            padding: 0,
        }
    }
    fn path(&self, name: &str) -> Result<PathBuf> {
        let mut components = Path::new(name).components();
        if !matches!(components.next(), Some(Component::Normal(_))) || components.next().is_some() {
            return Err(failure("unsafe fixture path"));
        }
        let path = self.root.join(name);
        if fs::symlink_metadata(&path).is_ok_and(|m| m.file_type().is_symlink()) {
            return Err(failure("symlink"));
        }
        Ok(path)
    }
    fn inject(&self, stage: Stage, index: usize) {
        *self.fault.lock().unwrap() = Some((stage, index));
    }
    fn fault(&self, stage: Stage, index: usize) -> Result<()> {
        let mut current = self.fault.lock().unwrap();
        if *current == Some((stage, index)) {
            *current = None;
            return Err(failure(format!("injected {stage:?}")));
        }
        Ok(())
    }
    fn all_rows(&self) -> Vec<(usize, u64, PatientResult)> {
        let mut rows = vec![];
        for name in self.part_names().unwrap() {
            let batch = self
                .decode_part(&self.read_part(&name, 4096).unwrap())
                .unwrap();
            for row in batch.rows {
                rows.push((batch.replication_index, batch.seed, row));
            }
        }
        rows
    }
}
impl PartStore for FsFixture {
    fn codec_id(&self) -> &str {
        "r111.fixture-json.v1"
    }
    fn begin_replication(&self, index: usize) -> Result<()> {
        *self.starts.lock().unwrap().entry(index).or_default() += 1;
        Ok(())
    }
    fn encode_part(&self, batch: &CaptureBatch, writer: &mut dyn Write) -> Result<()> {
        for _ in 0..self.padding {
            writer.write_all(b" ").map_err(failure)?;
        }
        serde_json::to_writer(writer, batch).map_err(failure)
    }
    fn write_part(&self, name: &str, batch: &CaptureBatch, max_bytes: usize) -> Result<()> {
        let path = self.path(name)?;
        let staged = self.path(&format!("{name}.staged"))?;
        if path.exists() {
            return Err(failure("part collision"));
        }
        self.fault(Stage::BeforeWrite, batch.replication_index)?;
        if let Some(gate) = &self.gate {
            gate.block();
        }
        let result = (|| {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&staged)
                .map_err(failure)?;
            // A real prefix reaches the filesystem before the injected partial-write error.
            if *self.fault.lock().unwrap() == Some((Stage::PartialWrite, batch.replication_index)) {
                file.write_all(b"{\"schema\":").map_err(failure)?;
                self.fault(Stage::PartialWrite, batch.replication_index)?;
            }
            let mut bounded = BoundedPartWriter::new(&mut file, max_bytes);
            self.encode_part(batch, &mut bounded)?;
            let written = bounded.written();
            self.fault(Stage::Flush, batch.replication_index)?;
            bounded.flush().map_err(failure)?;
            self.fault(Stage::Sync, batch.replication_index)?;
            file.sync_all().map_err(failure)?;
            self.fault(Stage::Rename, batch.replication_index)?;
            fs::rename(&staged, &path).map_err(failure)?;
            self.fault(Stage::AfterCommit, batch.replication_index)?;
            self.acknowledged
                .lock()
                .unwrap()
                .push((batch.rows.len(), written));
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&staged);
        }
        result
    }
    fn read_part(&self, name: &str, max_bytes: usize) -> Result<Vec<u8>> {
        let mut bytes = vec![];
        File::open(self.path(name)?)
            .map_err(failure)?
            .take(max_bytes.saturating_add(1) as u64)
            .read_to_end(&mut bytes)
            .map_err(failure)?;
        if bytes.len() > max_bytes {
            return Err(failure("bounded fixture read exceeded"));
        }
        Ok(bytes)
    }
    fn decode_part(&self, bytes: &[u8]) -> Result<CaptureBatch> {
        serde_json::from_slice(bytes).map_err(failure)
    }
    fn part_names(&self) -> Result<Vec<String>> {
        let mut names = vec![];
        for entry in fs::read_dir(&self.root).map_err(failure)? {
            let entry = entry.map_err(failure)?;
            if entry.path().extension().is_some_and(|v| v == "part") {
                names.push(
                    entry
                        .file_name()
                        .into_string()
                        .map_err(|_| failure("nonutf8 name"))?,
                );
            }
        }
        names.sort();
        Ok(names)
    }
    fn is_complete(&self) -> Result<bool> {
        Ok(self.root.join("manifest.json").exists())
    }
    fn complete(&self, reps: &[CapturedReplication]) -> Result<()> {
        self.fault(Stage::Manifest, 0)?;
        let target = self.root.join("manifest.json");
        if target.exists() {
            return Err(failure("completed fixture cannot overwrite"));
        }
        let staged = self.root.join("manifest.staged");
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staged)
            .map_err(failure)?;
        serde_json::to_writer(&mut file, reps).map_err(failure)?;
        file.flush().map_err(failure)?;
        file.sync_all().map_err(failure)?;
        fs::rename(staged, target).map_err(failure)
    }
}

fn scenario() -> ScenarioConfig {
    parse_scenario_json(include_str!("../examples/nominal_mvp.json")).unwrap()
}
fn config(workers: usize) -> BatchConfig {
    BatchConfig {
        base_seed: 7,
        replications: 4,
        num_workers: workers,
        checkpoint_interval: Some(1),
        ..BatchConfig::default()
    }
}
fn limits() -> CaptureLimits {
    CaptureLimits {
        max_rows: 2,
        max_bytes: 4096,
        max_patient_id_bytes: 1024,
    }
}
fn reference(input: &ScenarioConfig, cfg: &BatchConfig) -> Vec<(usize, u64, PatientResult)> {
    let mut rows = vec![];
    for (index, seed) in allocate_replication_seeds(cfg.base_seed, cfg.replications)
        .into_iter()
        .enumerate()
    {
        let mut s = input.clone();
        s.seed = seed;
        if let Some(h) = cfg.horizon_override {
            s.horizon_ticks = h;
        }
        rows.extend(
            run_scenario(&s)
                .unwrap()
                .patients
                .into_iter()
                .map(|p| (index, seed, p)),
        );
    }
    rows
}
fn checkpoint(dir: &Path) -> CaptureCheckpoint {
    serde_json::from_slice(&fs::read(dir.join("checkpoint.json")).unwrap()).unwrap()
}
fn save_checkpoint(dir: &Path, checkpoint: &CaptureCheckpoint) {
    fs::write(
        dir.join("checkpoint.json"),
        serde_json::to_vec(checkpoint).unwrap(),
    )
    .unwrap();
}

#[test]
fn backpressure_waits_for_real_ack_and_preserves_rows() {
    let temp = Temp::new();
    let gate = Arc::new(Gate::default());
    let mut store = FsFixture::new(temp.0.join("parts"));
    store.gate = Some(gate.clone());
    let finished = AtomicBool::new(false);
    let input = scenario();
    let cfg = config(1);
    let dir = temp.0.join("checkpoints");
    std::thread::scope(|scope| {
        let thread = scope.spawn(|| {
            let r = run_captured_batch(&input, &cfg, limits(), &dir, &store);
            finished.store(true, Ordering::SeqCst);
            r
        });
        let _release = Release(gate.clone());
        gate.wait_entered();
        assert!(!finished.load(Ordering::SeqCst));
        assert_eq!(store.starts.lock().unwrap().len(), 1);
        assert!(store.acknowledged.lock().unwrap().is_empty());
        assert!(!store.is_complete().unwrap());
        gate.release();
        assert_eq!(thread.join().unwrap().unwrap().len(), 4);
    });
    assert_eq!(store.all_rows(), reference(&input, &cfg));
    for (rows, bytes) in store.acknowledged.lock().unwrap().iter() {
        assert!(*rows <= 2 && *bytes <= 4096);
    }
}

#[test]
fn byte_limit_applies_during_encoding_and_rejects_oversized_rows() {
    let mut bytes = vec![];
    let mut writer = BoundedPartWriter::new(&mut bytes, 3);
    writer.write_all(b"abc").unwrap();
    assert!(writer.write_all(b"d").is_err());
    assert_eq!(writer.written(), 3);
    assert_eq!(bytes, b"abc");
    let temp = Temp::new();
    let store = FsFixture::new(temp.0.join("parts"));
    let mut input = scenario();
    input.patients[0].patient_id = "x".repeat(512);
    let error = run_captured_batch(
        &input,
        &config(1),
        CaptureLimits {
            max_rows: 2,
            max_bytes: 128,
            max_patient_id_bytes: 1024,
        },
        &temp.0.join("checkpoints"),
        &store,
    )
    .unwrap_err();
    assert!(error.message.contains("byte limit"));
    assert!(store.part_names().unwrap().is_empty());
    assert!(!store.is_complete().unwrap());
}

#[test]
fn real_filesystem_faults_propagate_and_never_publish_completion() {
    for stage in [
        Stage::BeforeWrite,
        Stage::PartialWrite,
        Stage::Flush,
        Stage::Sync,
        Stage::Rename,
        Stage::AfterCommit,
        Stage::Manifest,
    ] {
        let temp = Temp::new();
        let store = FsFixture::new(temp.0.join("parts"));
        store.inject(stage, if stage == Stage::Manifest { 0 } else { 1 });
        let error = run_captured_batch(
            &scenario(),
            &config(1),
            limits(),
            &temp.0.join("checkpoints"),
            &store,
        )
        .unwrap_err();
        assert_eq!(
            error.kind,
            ExperimentErrorKind::OutputWriteFailed,
            "{stage:?}"
        );
        assert!(error.message.contains("injected"));
        assert!(!store.is_complete().unwrap(), "{stage:?}");
        // A later failing replication/publication cannot mutate already committed work.
        let saved = checkpoint(&temp.0.join("checkpoints"));
        for part in &saved.completed_replications[0].parts {
            assert_eq!(
                compute_sha256_hex(&store.read_part(&part.name, 4096).unwrap()),
                part.sha256
            );
        }
    }
}

#[test]
fn checkpoint_write_failure_cannot_be_reported_complete() {
    let temp = Temp::new();
    let store = FsFixture::new(temp.0.join("parts"));
    let dir = temp.0.join("checkpoints");
    fs::create_dir(&dir).unwrap();
    // Collision at the REAL inherited atomic-write staging path, not an invented trace path.
    fs::create_dir(dir.join(format!("checkpoint.tmp.{}", std::process::id()))).unwrap();
    assert!(run_captured_batch(&scenario(), &config(1), limits(), &dir, &store).is_err());
    assert!(!store.is_complete().unwrap());
}

fn interrupted(temp: &Temp) -> (FsFixture, PathBuf) {
    let store = FsFixture::new(temp.0.join("parts"));
    store.inject(Stage::BeforeWrite, 1);
    let dir = temp.0.join("checkpoints");
    assert!(run_captured_batch(&scenario(), &config(1), limits(), &dir, &store).is_err());
    let saved = checkpoint(&dir);
    assert_eq!(saved.completed_replications.len(), 1);
    assert_eq!(saved.completed_replications[0].summary.replication_index, 0);
    assert_eq!(saved.pending_indices, vec![1, 2, 3]);
    (store, dir)
}

#[test]
fn completed_replication_resume_is_gap_free_and_does_not_rerun_completed_work() {
    for workers in [1, 2, 4] {
        let temp = Temp::new();
        let (store, dir) = interrupted(&temp);
        let original = checkpoint(&dir).completed_replications[0].clone();
        let results =
            resume_captured_batch(&scenario(), &config(workers), limits(), &dir, &store).unwrap();
        assert_eq!(results[0], original);
        assert_eq!(store.starts.lock().unwrap()[&0], 1);
        assert_eq!(store.all_rows(), reference(&scenario(), &config(workers)));
        assert!(checkpoint(&dir).pending_indices.is_empty());
        assert!(store.is_complete().unwrap());
        let manifest = fs::read(store.root.join("manifest.json")).unwrap();
        assert!(
            resume_captured_batch(&scenario(), &config(workers), limits(), &dir, &store).is_err()
        );
        assert_eq!(
            fs::read(store.root.join("manifest.json")).unwrap(),
            manifest
        );
    }
}

#[test]
fn missing_corrupt_duplicate_and_mismatched_parts_reject_before_any_resumed_work() {
    for damage in [
        "missing",
        "corrupt",
        "duplicate",
        "gap",
        "seed",
        "path",
        "orphan",
        "lost_tail",
        "duplicate_row",
        "unknown_patient",
        "summary",
    ] {
        let temp = Temp::new();
        let (store, dir) = interrupted(&temp);
        let mut saved = checkpoint(&dir);
        let rep = &mut saved.completed_replications[0];
        let part = rep.parts[0].clone();
        match damage {
            "missing" => fs::remove_file(store.root.join(&part.name)).unwrap(),
            "corrupt" => fs::write(store.root.join(&part.name), b"broken").unwrap(),
            "duplicate" => rep.parts.push(part.clone()),
            "gap" => rep.parts[0].first_sequence += 1,
            "path" => rep.parts[0].name = "../outside.part".into(),
            "orphan" => fs::write(store.root.join("orphan.part"), b"orphan").unwrap(),
            "lost_tail" => {
                let last = rep.parts.pop().unwrap();
                fs::remove_file(store.root.join(last.name)).unwrap();
            }
            "duplicate_row" => {
                let bytes = fs::read(store.root.join(&part.name)).unwrap();
                let mut batch = store.decode_part(&bytes).unwrap();
                assert!(batch.rows.len() >= 2);
                batch.rows[1] = batch.rows[0].clone();
                let bytes = serde_json::to_vec(&batch).unwrap();
                fs::write(store.root.join(&part.name), &bytes).unwrap();
                rep.parts[0].bytes = bytes.len();
                rep.parts[0].sha256 = compute_sha256_hex(&bytes);
            }
            "summary" => {
                rep.summary.mean_wait_ticks += 1.0;
            }
            "unknown_patient" => {
                let mut batch = store
                    .decode_part(&fs::read(store.root.join(&part.name)).unwrap())
                    .unwrap();
                batch.rows[0].patient_id = "unknown".into();
                let bytes = serde_json::to_vec(&batch).unwrap();
                fs::write(store.root.join(&part.name), &bytes).unwrap();
                rep.parts[0].bytes = bytes.len();
                rep.parts[0].sha256 = compute_sha256_hex(&bytes);
            }
            "seed" => {
                let mut batch = store
                    .decode_part(&fs::read(store.root.join(&part.name)).unwrap())
                    .unwrap();
                batch.seed += 1;
                let bytes = serde_json::to_vec(&batch).unwrap();
                fs::write(store.root.join(&part.name), &bytes).unwrap();
                rep.parts[0].bytes = bytes.len();
                rep.parts[0].sha256 = compute_sha256_hex(&bytes);
            }
            _ => unreachable!(),
        }
        save_checkpoint(&dir, &saved);
        let before = store.starts.lock().unwrap().clone();
        assert!(
            resume_captured_batch(&scenario(), &config(2), limits(), &dir, &store).is_err(),
            "{damage}"
        );
        assert_eq!(*store.starts.lock().unwrap(), before, "{damage}");
        assert!(!store.is_complete().unwrap());
    }
}

#[test]
fn input_drift_and_unacknowledged_part_reject_without_implicit_replay() {
    let temp = Temp::new();
    let (store, dir) = interrupted(&temp);
    let mut input = scenario();
    input.horizon_ticks += 1;
    let before = store.starts.lock().unwrap().clone();
    assert!(resume_captured_batch(&input, &config(1), limits(), &dir, &store).is_err());
    assert_eq!(*store.starts.lock().unwrap(), before);
    let temp = Temp::new();
    let store = FsFixture::new(temp.0.join("parts"));
    store.inject(Stage::AfterCommit, 1);
    let dir = temp.0.join("checkpoints");
    assert!(run_captured_batch(&scenario(), &config(1), limits(), &dir, &store).is_err());
    let before = store.starts.lock().unwrap().clone();
    assert!(resume_captured_batch(&scenario(), &config(1), limits(), &dir, &store).is_err());
    assert_eq!(*store.starts.lock().unwrap(), before);
}

#[test]
fn manifest_failure_recovers_from_real_complete_checkpoint_without_reexecution() {
    let temp = Temp::new();
    let store = FsFixture::new(temp.0.join("parts"));
    store.inject(Stage::Manifest, 0);
    let dir = temp.0.join("checkpoints");
    assert!(run_captured_batch(&scenario(), &config(2), limits(), &dir, &store).is_err());
    assert!(checkpoint(&dir).pending_indices.is_empty());
    assert!(!store.is_complete().unwrap());
    let before = store.starts.lock().unwrap().clone();
    resume_captured_batch(&scenario(), &config(4), limits(), &dir, &store).unwrap();
    assert_eq!(*store.starts.lock().unwrap(), before);
    assert_eq!(store.all_rows(), reference(&scenario(), &config(4)));
    assert!(store.is_complete().unwrap());
}

#[test]
fn actual_codec_framing_controls_batch_split_and_byte_cap() {
    let temp = Temp::new();
    let mut store = FsFixture::new(temp.0.join("parts"));
    store.padding = 512;
    let rows = run_scenario(&scenario()).unwrap().patients;
    let sample = CaptureBatch {
        time_unit: "ticks".into(),
        schema: "careops.ed.capture-boundary.v1".into(),
        run_id: config(1).batch_id,
        input_sha256: "0".repeat(64),
        replication_index: 0,
        seed: u64::MAX,
        first_sequence: 999,
        rows: vec![rows[0].clone()],
    };
    let mut bytes = vec![];
    store.encode_part(&sample, &mut bytes).unwrap();
    let cap = CaptureLimits {
        max_rows: 10,
        max_bytes: bytes.len() + 64,
        max_patient_id_bytes: 1024,
    };
    run_captured_batch(
        &scenario(),
        &config(2),
        cap,
        &temp.0.join("checkpoints"),
        &store,
    )
    .unwrap();
    assert_eq!(store.all_rows(), reference(&scenario(), &config(2)));
    for (rows, size) in store.acknowledged.lock().unwrap().iter() {
        assert_eq!(*rows, 1);
        assert!(*size <= cap.max_bytes);
    }
}

#[test]
fn capture_wait_aggregate_overflow_is_a_typed_error_not_a_panic() {
    let temp = Temp::new();
    let store = FsFixture::new(temp.0.join("parts"));
    let mut input = scenario();
    input.horizon_ticks = u64::MAX - 1;
    for (index, patient) in input.patients.iter_mut().enumerate() {
        patient.arrival_tick = 0;
        patient.resource_id = "res.acute".into();
        patient.task_class_id = "task.acute_treatment".into();
        patient.work_ticks = if index < 2 {
            u64::MAX / 2
        } else if index < 4 {
            u64::MAX / 2 - 1
        } else {
            1
        };
    }
    input
        .validate()
        .expect("overflow fixture must pass scenario validation");
    let cfg = BatchConfig {
        replications: 1,
        ..config(1)
    };
    let error = run_captured_batch(&input, &cfg, limits(), &temp.0.join("checkpoints"), &store)
        .unwrap_err();
    assert_eq!(error.kind, ExperimentErrorKind::MalformedConfig);
    assert!(error.message.contains("wait sum overflow"));
    assert!(!store.is_complete().unwrap());
}
