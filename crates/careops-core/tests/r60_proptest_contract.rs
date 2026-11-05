//! Bounded current-core contracts; no simulation RNG or production dependency.
use careops_core::capacity::CapacitySnapshot;
use careops_core::identity::{
    BirthEventId, EpisodeEventId, EpisodeId, EpisodeStatus, IdentityCommand, IdentityError,
    IdentityLedger, IdentityOutcome, LifecycleEventId, PersonId, PersonOrigin, PersonStatus,
    SpawnEventId,
};
use careops_core::transfer::ServiceId;
use proptest::prelude::*;
use proptest::test_runner::{
    Config, FileFailurePersistence, RngAlgorithm, RngSeed, TestCaseResult, TestError, TestRunner,
};
use sha2::{Digest, Sha256};
use std::cell::{Cell, RefCell};
use std::collections::BTreeSet;

const SEED: u64 = 0x595920261105;
fn runner(persistence: Option<Box<dyn proptest::test_runner::FailurePersistence>>) -> TestRunner {
    runner_cases(persistence, 128)
}
fn runner_cases(
    persistence: Option<Box<dyn proptest::test_runner::FailurePersistence>>,
    cases: u32,
) -> TestRunner {
    TestRunner::new(Config {
        cases,
        max_shrink_iters: 1000,
        max_shrink_time: 0,
        max_local_rejects: 0,
        max_global_rejects: 0,
        rng_algorithm: RngAlgorithm::ChaCha,
        rng_seed: RngSeed::Fixed(SEED),
        failure_persistence: persistence,
        ..Config::default()
    })
}
fn domain_runner() -> TestRunner {
    runner(Some(Box::new(FileFailurePersistence::Direct(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/r60-proptest-regressions.txt"
    )))))
}
fn uint() -> impl Strategy<Value = u64> {
    prop_oneof![
        Just(0),
        Just(1),
        Just(u64::MAX - 1),
        Just(u64::MAX),
        any::<u64>()
    ]
}
// Widen arithmetic independently: production validates with checked u64 addition.
fn capacity_oracle(s: &CapacitySnapshot) -> Result<Option<u64>, ()> {
    let total = [s.reserved, s.occupied, s.cleaning, s.unavailable]
        .into_iter()
        .map(u128::from)
        .sum::<u128>();
    if total > u128::from(u64::MAX)
        || matches!((s.equipped,s.physical),(Some(e),Some(p)) if e>p)
        || matches!((s.open,s.physical),(Some(o),Some(p)) if o>p)
        || [s.physical, s.equipped]
            .into_iter()
            .flatten()
            .any(|x| total > u128::from(x))
    {
        return Err(());
    }
    let ceilings = [s.physical, s.equipped, s.open, s.staffed];
    if ceilings.contains(&Some(0)) {
        return Ok(Some(0));
    }
    if ceilings.contains(&None) {
        return Ok(None);
    }
    let min = ceilings.into_iter().flatten().min().unwrap();
    Ok(Some(u128::from(min).saturating_sub(total) as u64))
}
fn compare_capacity(
    s: &CapacitySnapshot,
    valid: bool,
    observed: Result<Option<u64>, ()>,
) -> TestCaseResult {
    let expected = capacity_oracle(s);
    prop_assert_eq!(valid, expected.is_ok());
    prop_assert_eq!(observed, expected);
    Ok(())
}
fn capacity_contract(s: CapacitySnapshot) -> TestCaseResult {
    compare_capacity(
        &s,
        s.validate().is_ok(),
        s.assignable_capacity().map_err(|_| ()),
    )
}
#[test]
fn generated_capacity_and_unknown_zero_overflow_boundaries() {
    let optional = || proptest::option::of(uint());
    let strategy = (
        optional(),
        optional(),
        optional(),
        optional(),
        uint(),
        uint(),
        uint(),
        uint(),
    )
        .prop_map(
            |(physical, equipped, open, staffed, reserved, occupied, cleaning, unavailable)| {
                CapacitySnapshot {
                    physical,
                    equipped,
                    open,
                    staffed,
                    reserved,
                    occupied,
                    cleaning,
                    unavailable,
                }
            },
        );
    domain_runner().run(&strategy, capacity_contract).unwrap();
    for (ceilings, dispositions, expected) in [
        ([None; 4], [0; 4], None),
        ([None, Some(0), None, None], [0; 4], Some(0)),
        ([Some(u64::MAX); 4], [u64::MAX - 1, 0, 0, 0], Some(1)),
        ([Some(u64::MAX); 4], [u64::MAX, 0, 0, 0], Some(0)),
    ] {
        let [physical, equipped, open, staffed] = ceilings;
        let [reserved, occupied, cleaning, unavailable] = dispositions;
        let s = CapacitySnapshot {
            physical,
            equipped,
            open,
            staffed,
            reserved,
            occupied,
            cleaning,
            unavailable,
        };
        assert_eq!(s.assignable_capacity().unwrap(), expected);
        capacity_contract(s).unwrap();
    }
    for n in [0, 1, u64::MAX - 1, u64::MAX] {
        for known in [None, Some(0), Some(n)] {
            capacity_contract(CapacitySnapshot {
                physical: known,
                equipped: Some(n),
                open: known,
                staffed: Some(n),
                reserved: n,
                occupied: 1,
                cleaning: 0,
                unavailable: 0,
            })
            .unwrap();
        }
    }
}
fn transition_contract((n, ops): (u64, Vec<(u8, u64)>)) -> TestCaseResult {
    let mut s = CapacitySnapshot {
        physical: Some(n),
        equipped: Some(n),
        open: Some(n),
        staffed: Some(n),
        reserved: n,
        occupied: 0,
        cleaning: 0,
        unavailable: 0,
    };
    let mut model = [n, 0, 0, 0];
    for (op, units) in ops {
        let before = s.clone();
        let mut next = model.map(u128::from);
        let (from, to) = match op {
            0 => (Some(0), Some(1)),
            1 => (Some(1), Some(2)),
            2 => (Some(2), None),
            3 => (None, Some(3)),
            _ => (Some(3), None),
        };
        let mut valid = true;
        if let Some(i) = from {
            if next[i] < u128::from(units) {
                valid = false;
            } else {
                next[i] -= u128::from(units);
            }
        }
        if let Some(i) = to {
            next[i] += u128::from(units);
        }
        valid &= next.iter().sum::<u128>() <= u128::from(n);
        let result = match op {
            0 => s.start_reserved_work(units),
            1 => s.complete_occupied_work(units, true),
            2 => s.finish_cleaning(units),
            3 => s.mark_unavailable(units),
            _ => s.restore_unavailable(units),
        };
        prop_assert_eq!(result.is_ok(), valid);
        if valid {
            model = next.map(|x| x as u64);
        } else {
            prop_assert_eq!(&s, &before);
        }
        prop_assert_eq!([s.reserved, s.occupied, s.cleaning, s.unavailable], model);
        capacity_contract(s.clone())?;
    }
    Ok(())
}
#[test]
fn generated_capacity_transitions_are_conservative_and_atomic() {
    domain_runner()
        .run(
            &(uint(), proptest::collection::vec((0u8..5, uint()), 1..=16)),
            transition_contract,
        )
        .unwrap();
    transition_contract((
        u64::MAX,
        vec![
            (0, u64::MAX),
            (1, u64::MAX),
            (2, u64::MAX),
            (3, u64::MAX),
            (4, u64::MAX),
        ],
    ))
    .unwrap();
}
// Public A5.2 framing reference, separately checked against Python hashlib vectors.
fn reference_id(kind: &[u8], namespace: u64, event: u64) -> u64 {
    let mut bytes = b"careops.identity.id.v1\0".to_vec();
    bytes.extend_from_slice(kind);
    bytes.push(0);
    bytes.extend_from_slice(&namespace.to_be_bytes());
    bytes.extend_from_slice(&event.to_be_bytes());
    u64::from_be_bytes(Sha256::digest(bytes)[..8].try_into().unwrap())
}
fn spawn(event: u64) -> IdentityCommand {
    IdentityCommand::Spawn {
        event_id: SpawnEventId::new(event),
        origin: PersonOrigin::ExternalEntry,
    }
}
fn enter(ns: u64, event: u64) -> IdentityCommand {
    IdentityCommand::EnterService {
        event_id: EpisodeEventId::new(event),
        person_id: PersonId::new(reference_id(b"person", ns, event)),
        service_id: ServiceId::new(1),
    }
}
fn ids_collide(kind: &[u8], ns: u64, keys: &[u64]) -> bool {
    let ids: BTreeSet<_> = keys.iter().map(|k| reference_id(kind, ns, *k)).collect();
    ids.contains(&0) || ids.len() != keys.len()
}
fn check_order(actual: &[IdentityOutcome], expected: &[IdentityOutcome]) -> TestCaseResult {
    prop_assert_eq!(actual, expected);
    Ok(())
}
fn identity_contract((ns, raw, reverse): (u64, Vec<u64>, bool)) -> TestCaseResult {
    // Nonzero logical event keys are a set, not filtered random rejection.
    let keys: Vec<_> = raw
        .into_iter()
        .map(|x| x.max(1))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let mut commands: Vec<_> = keys
        .iter()
        .flat_map(|k| [enter(ns, *k), spawn(*k)])
        .collect();
    if reverse {
        commands.reverse();
    }
    let mut ledger = IdentityLedger::new(ns);
    let before = ledger.checkpoint().unwrap();
    let result = ledger.apply_tick(0, commands.clone());
    if ids_collide(b"person", ns, &keys) || ids_collide(b"episode", ns, &keys) {
        prop_assert_eq!(result, Err(IdentityError::IdCollision));
        prop_assert_eq!(ledger.checkpoint().unwrap(), before);
        return Ok(());
    }
    let expected: Vec<_> = keys
        .iter()
        .map(|k| IdentityOutcome::PersonCreated(PersonId::new(reference_id(b"person", ns, *k))))
        .chain(keys.iter().map(|k| {
            IdentityOutcome::EpisodeEntered(EpisodeId::new(reference_id(b"episode", ns, *k)))
        }))
        .collect();
    check_order(&result.unwrap(), &expected)?;
    let n = keys.len();
    prop_assert_eq!(ledger.people_count(), n);
    prop_assert_eq!(ledger.episodes_count(), n);
    prop_assert_eq!(ledger.counts().people_created, n as u64);
    prop_assert_eq!(ledger.counts().service_entries, n as u64);
    let bytes = ledger.checkpoint().unwrap();
    prop_assert_eq!(
        ledger.apply_tick(0, commands).unwrap(),
        vec![IdentityOutcome::Duplicate; 2 * n]
    );
    prop_assert_eq!(&ledger.checkpoint().unwrap(), &bytes);
    let mut restored = IdentityLedger::restore(&bytes).unwrap();
    // Complete prefix tick0; suffix tick1 contains every exit and death.
    let suffix: Vec<_> = keys
        .iter()
        .map(|k| IdentityCommand::ExitService {
            event_id: LifecycleEventId::new(*k),
            episode_id: EpisodeId::new(reference_id(b"episode", ns, *k)),
        })
        .collect();
    prop_assert_eq!(
        ledger.apply_tick(1, suffix.clone()),
        restored.apply_tick(1, suffix)
    );
    // Exits and deaths share lifecycle keys; allocate unused keys independently.
    let used: BTreeSet<_> = keys.iter().copied().collect();
    let deaths: Vec<_> = keys
        .iter()
        .zip((1..).filter(|k| !used.contains(k)))
        .map(|(k, id)| IdentityCommand::Death {
            event_id: LifecycleEventId::new(id),
            person_id: PersonId::new(reference_id(b"person", ns, *k)),
        })
        .collect();
    prop_assert_eq!(
        ledger.apply_tick(2, deaths.clone()),
        restored.apply_tick(2, deaths)
    );
    prop_assert_eq!(ledger.checkpoint().unwrap(), restored.checkpoint().unwrap());
    prop_assert_eq!(ledger.counts().service_exits, n as u64);
    prop_assert_eq!(ledger.counts().deaths, n as u64);
    for k in keys {
        prop_assert_eq!(
            ledger
                .person(PersonId::new(reference_id(b"person", ns, k)))
                .unwrap()
                .status,
            PersonStatus::Deceased
        );
        prop_assert_eq!(
            ledger
                .episode(EpisodeId::new(reference_id(b"episode", ns, k)))
                .unwrap()
                .status,
            EpisodeStatus::Exited
        );
    }
    Ok(())
}
#[test]
fn generated_identity_order_duplicates_lifecycle_and_checkpoint_recovery() {
    domain_runner()
        .run(
            &(
                uint(),
                proptest::collection::vec(uint(), 1..=16),
                any::<bool>(),
            ),
            identity_contract,
        )
        .unwrap();
    identity_contract((u64::MAX, vec![0, 1, u64::MAX - 1, u64::MAX], true)).unwrap();
}
fn unchanged(before: &[u8], after: &[u8]) -> TestCaseResult {
    prop_assert_eq!(before, after);
    Ok(())
}
fn rollback_contract(ns: u64) -> TestCaseResult {
    let mut ledger = IdentityLedger::new(ns);
    let keys = [11, 12];
    if ids_collide(b"person", ns, &keys) {
        return Ok(());
    } // Exclude this predicted degenerate case; no claim to have sampled a hash collision.
    ledger.apply_tick(0, vec![spawn(12)]).unwrap();
    let before = ledger.checkpoint().unwrap();
    let result = ledger.apply_tick(
        1,
        vec![
            IdentityCommand::Spawn {
                event_id: SpawnEventId::new(12),
                origin: PersonOrigin::Birth {
                    birth_event_id: BirthEventId::new(1),
                    infant_ordinal: 0,
                },
            },
            spawn(11),
        ],
    );
    prop_assert_eq!(result, Err(IdentityError::EventConflict));
    unchanged(&before, &ledger.checkpoint().unwrap())?;
    let mut reference = IdentityLedger::restore(&before).unwrap();
    prop_assert_eq!(
        ledger.apply_tick(1, vec![spawn(11)]),
        reference.apply_tick(1, vec![spawn(11)])
    );
    prop_assert_eq!(
        ledger.checkpoint().unwrap(),
        reference.checkpoint().unwrap()
    );
    prop_assert_eq!(ledger.people_count(), 2);
    Ok(())
}
#[test]
fn generated_invalid_batches_rollback_after_valid_canonical_prefix() {
    domain_runner().run(&uint(), rollback_contract).unwrap();
    for ns in [0, 1, u64::MAX - 1, u64::MAX] {
        rollback_contract(ns).unwrap();
    }
}
#[test]
fn same_seed_replays_cases_and_full_width_ids_round_trip() {
    let record = || {
        let values = RefCell::new(Vec::new());
        runner(None)
            .run(&uint(), |x| {
                values.borrow_mut().push(x);
                prop_assert_eq!(
                    serde_json::from_str::<PersonId>(
                        &serde_json::to_string(&PersonId::new(x)).unwrap()
                    )
                    .unwrap()
                    .value(),
                    x
                );
                Ok(())
            })
            .unwrap();
        values.into_inner()
    };
    let first = record();
    assert_eq!(first.len(), 128);
    assert_eq!(first, record());
    for x in [0, 1, u64::MAX - 1, u64::MAX] {
        assert_eq!(PersonId::new(x).value(), x);
        assert_eq!(SpawnEventId::new(x).value(), x);
    }
    assert_eq!(reference_id(b"person", 1, 1), 5_152_514_178_808_668_989);
    assert_eq!(reference_id(b"episode", 1, 1), 4_708_837_883_263_603_961);
}
#[test]
fn negative_controls_reject_conservation_order_and_partial_commit() {
    let result = runner(None).run(&(1u64..=1024), |n| {
        let s = CapacitySnapshot {
            physical: Some(n),
            equipped: Some(n),
            open: Some(n),
            staffed: Some(n),
            reserved: n,
            occupied: 0,
            cleaning: 0,
            unavailable: 0,
        };
        compare_capacity(&s, true, Ok(Some(1)))?;
        Ok(())
    });
    assert!(matches!(result, Err(TestError::Fail(_, 1))));
    let a = IdentityOutcome::PersonCreated(PersonId::new(1));
    let b = IdentityOutcome::PersonCreated(PersonId::new(2));
    let result = runner(None).run(&Just(()), |()| check_order(&[b, a], &[a, b]));
    assert!(matches!(result, Err(TestError::Fail(_, ()))));
    let mut prefix = IdentityLedger::new(1);
    prefix.apply_tick(0, vec![spawn(12)]).unwrap();
    let before = prefix.checkpoint().unwrap();
    prefix.apply_tick(1, vec![spawn(11)]).unwrap();
    let after = prefix.checkpoint().unwrap();
    let result = runner(None).run(&Just(()), |()| unchanged(&before, &after));
    assert!(matches!(result, Err(TestError::Fail(_, ()))));
}
#[test]
fn shrinking_and_saved_failure_replay_execute_without_random_cases() {
    let tmp = tempfile::tempdir().unwrap();
    // Upstream Direct requires &'static str; one bounded test-only allocation.
    let path: &'static str = Box::leak(
        tmp.path()
            .join("saved.txt")
            .to_str()
            .unwrap()
            .to_owned()
            .into_boxed_str(),
    );
    let persistence = || {
        Some(Box::new(FileFailurePersistence::Direct(path))
            as Box<dyn proptest::test_runner::FailurePersistence>)
    };
    let fail = |x| {
        prop_assert!(x < 10);
        Ok(())
    };
    assert!(matches!(
        runner(persistence()).run(&(10u64..=1024), fail),
        Err(TestError::Fail(_, 10))
    ));
    assert!(std::fs::metadata(path).unwrap().len() > 0);
    let mut replay = runner_cases(persistence(), 0);
    let executed = Cell::new(0);
    let result = replay.run(&(10u64..=1024), |x| {
        executed.set(executed.get() + 1);
        fail(x)
    });
    assert!(matches!(result, Err(TestError::Fail(_, 10))));
    assert!(executed.get() > 0);
}
