//! R11.1 original-run invocation and typed capture/export oracle.

#[cfg(feature = "native-data")]
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};

#[cfg(feature = "native-data")]
use crate::disposition::batch_runner::{
    allocate_replication_seeds, run_batch_with_workers, BatchConfig,
};
#[cfg(feature = "native-data")]
use crate::disposition::native_output::{
    export_native_batch, read_native_dataset, ExportLimits, NativeFormat,
};

static CALLS: OnceLock<Mutex<HashMap<String, Vec<u64>>>> = OnceLock::new();
static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

fn registry() -> &'static Mutex<HashMap<String, Vec<u64>>> {
    CALLS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Observe only registered test scenarios, including calls on scoped workers.
/// Compiled out of normal library builds; no RNG/event/output mutation.
pub(super) fn observe_invocation(scenario_id: &str, seed: u64) {
    let mut calls = registry().lock().unwrap_or_else(|e| e.into_inner());
    if let Some(seeds) = calls.get_mut(scenario_id) {
        seeds.push(seed);
    }
}

struct Observation(String);

impl Observation {
    fn new() -> Self {
        let id = format!("r111-observed-{}", NEXT_ID.fetch_add(1, Ordering::Relaxed));
        assert!(registry()
            .lock()
            .unwrap()
            .insert(id.clone(), vec![])
            .is_none());
        Self(id)
    }

    fn seeds(&self) -> Vec<u64> {
        let mut seeds = registry().lock().unwrap()[&self.0].clone();
        seeds.sort_unstable();
        seeds
    }
}

impl Drop for Observation {
    fn drop(&mut self) {
        registry()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&self.0);
    }
}

#[test]
fn observer_isolates_concurrent_scenarios_and_cleans_up() {
    let a = Observation::new();
    let b = Observation::new();
    std::thread::scope(|scope| {
        for seed in 0..8 {
            let id = if seed % 2 == 0 { &a.0 } else { &b.0 };
            scope.spawn(move || observe_invocation(id, seed));
        }
    });
    assert_eq!(a.seeds(), vec![0, 2, 4, 6]);
    assert_eq!(b.seeds(), vec![1, 3, 5, 7]);
    let id = a.0.clone();
    drop(a);
    observe_invocation(&id, 99);
    assert!(!registry().lock().unwrap().contains_key(&id));
    assert_eq!(b.seeds(), vec![1, 3, 5, 7]);
}

#[cfg(feature = "native-data")]
fn assert_original_run_only(workers: usize) {
    let mut scenario = crate::parse_scenario_json(include_str!("../examples/nominal_mvp.json"))
        .expect("valid existing synthetic fixture");
    let config = BatchConfig {
        base_seed: 7,
        replications: 4,
        num_workers: workers,
        checkpoint_interval: None,
        ..BatchConfig::default()
    };
    let seeds = allocate_replication_seeds(config.base_seed, config.replications);
    // Independent expected rows are produced before registering the observed run.
    let reference: Vec<_> = seeds
        .iter()
        .map(|seed| {
            let mut input = scenario.clone();
            input.seed = *seed;
            crate::run_scenario(&input).unwrap()
        })
        .collect();
    let observation = Observation::new();
    scenario.scenario_id = observation.0.clone();
    let config = BatchConfig {
        scenario_id: scenario.scenario_id.clone(),
        ..config
    };
    let mut sorted_seeds = seeds.clone();
    sorted_seeds.sort_unstable();
    let batch = run_batch_with_workers(&scenario, &config, None, None, None).unwrap();
    assert_eq!(
        observation.seeds(),
        sorted_seeds,
        "original batch must run each seed once"
    );
    let destination = tempfile::tempdir().unwrap();
    export_native_batch(
        &scenario,
        &batch,
        destination.path(),
        NativeFormat::Parquet,
        ExportLimits::default(),
    )
    .unwrap();
    let data = read_native_dataset(destination.path()).unwrap();
    let patients = data.patients();
    assert_eq!(
        patients.height(),
        reference.iter().map(|r| r.patients.len()).sum::<usize>()
    );
    let mut identities = BTreeSet::new();
    for row in 0..patients.height() {
        let replication = patients
            .column("replication_index")
            .unwrap()
            .u64()
            .unwrap()
            .get(row)
            .unwrap() as usize;
        let patient_id = patients
            .column("patient_id")
            .unwrap()
            .str()
            .unwrap()
            .get(row)
            .unwrap();
        assert!(
            identities.insert((replication, patient_id)),
            "duplicate exported row"
        );
        assert_eq!(
            patients.column("seed").unwrap().u64().unwrap().get(row),
            Some(seeds[replication])
        );
        let expected = reference[replication]
            .patients
            .iter()
            .find(|p| p.patient_id == patient_id)
            .unwrap();
        for (name, expected) in [
            ("arrival_tick", Some(expected.arrival_tick)),
            ("start_tick", expected.start_tick),
            ("completion_tick", expected.completion_tick),
            ("wait_ticks", expected.wait_ticks),
            ("work_ticks", Some(expected.work_ticks)),
            ("elapsed_ticks", expected.elapsed_ticks),
        ] {
            assert_eq!(
                patients
                    .column(&format!("{name}_hi"))
                    .unwrap()
                    .u64()
                    .unwrap()
                    .get(row),
                expected.map(|_| 0)
            );
            assert_eq!(
                patients
                    .column(&format!("{name}_lo"))
                    .unwrap()
                    .u64()
                    .unwrap()
                    .get(row),
                expected
            );
        }
    }
    // Publishing and rereading production native files must not run any replication again.
    assert_eq!(
        observation.seeds(),
        sorted_seeds,
        "export must consume original capture without invoking any replication again"
    );
}

#[cfg(feature = "native-data")]
#[test]
fn export_invokes_each_replication_once_one_worker() {
    assert_original_run_only(1);
}

#[cfg(feature = "native-data")]
#[test]
fn export_invokes_each_replication_once_two_workers() {
    assert_original_run_only(2);
}

#[cfg(feature = "native-data")]
#[test]
fn export_invokes_each_replication_once_four_workers() {
    assert_original_run_only(4);
}
