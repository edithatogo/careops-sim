//! R11.1 red oracle against the real batch/export path. No capture implementation.

use std::collections::{BTreeSet, HashMap};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};

use crate::disposition::arrow_export::generate_arrow_export_bundle;
use crate::disposition::batch_runner::{
    allocate_replication_seeds, run_batch_with_workers, BatchConfig,
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
    let bundle = generate_arrow_export_bundle(&scenario, &batch, None, None).unwrap();
    assert_eq!(
        bundle.patient_records.len(),
        reference.iter().map(|r| r.patients.len()).sum::<usize>()
    );
    let mut identities = BTreeSet::new();
    for row in &bundle.patient_records {
        assert!(
            identities.insert((row.replication_index, row.patient_id.clone())),
            "duplicate exported row"
        );
        assert_eq!(row.seed, seeds[row.replication_index]);
        let expected = reference[row.replication_index]
            .patients
            .iter()
            .find(|p| p.patient_id == row.patient_id)
            .unwrap();
        assert_eq!(row.arrival_tick, expected.arrival_tick);
        assert_eq!(row.start_tick, expected.start_tick);
        assert_eq!(row.completion_tick, expected.completion_tick);
        assert_eq!(row.wait_ticks, expected.wait_ticks);
        assert_eq!(row.work_ticks, expected.work_ticks);
        assert_eq!(row.elapsed_ticks, expected.elapsed_ticks);
    }
    // Deliberate behavioral red: current exporter adds four real simulation calls.
    assert_eq!(
        observation.seeds(),
        sorted_seeds,
        "export must consume original capture without invoking any replication again"
    );
}

#[test]
fn export_invokes_each_replication_once_one_worker() {
    assert_original_run_only(1);
}

#[test]
fn export_invokes_each_replication_once_two_workers() {
    assert_original_run_only(2);
}

#[test]
fn export_invokes_each_replication_once_four_workers() {
    assert_original_run_only(4);
}
