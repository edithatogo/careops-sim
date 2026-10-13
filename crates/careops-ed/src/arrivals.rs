//! Synthetic, count-conditioned arrivals for the experimental E1 tick fixture.
//!
//! This module is not a P4 loader or a clinical arrival model. Its integer-bin
//! placement is a reproducible fixture assumption, not an empirical dependence
//! or prevalence claim.

use std::collections::{BTreeSet, HashSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

use kairo_ecs_rng::{derive_entity_seed, DeterministicStream};
use kairo_ecs_types::EntityId;
use serde::{Deserialize, Serialize};

const ARRIVAL_DOMAIN_INDEX: u64 = 0x4341_5245_4F50_5341;
const MAX_BINS: usize = 4096;
const MAX_MODES_PER_BIN: usize = 32;
const MAX_DISTINCT_MODES: usize = 32;
const MAX_MODE_ID_BYTES: usize = 64;
const MAX_PATIENTS: u64 = 100_000;
const MAX_SAMPLING_DRAWS: usize = 128;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArrivalConfig {
    pub version: u32,
    pub seed: u64,
    pub horizon_ticks: u64,
    pub provenance: String,
    pub bins: Vec<ArrivalBin>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArrivalBin {
    pub start_tick: u64,
    pub end_tick: u64,
    pub modes: Vec<ArrivalModeCounts>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArrivalModeCounts {
    pub mode: String,
    pub total: u64,
    pub acuity_counts: [u64; 5],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct GeneratedArrival {
    pub patient_id: String,
    pub arrival_tick: u64,
    pub mode: String,
    pub acuity: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArrivalError {
    InvalidVersion,
    InvalidProvenance,
    InvalidHorizon,
    LimitExceeded,
    InvalidMode,
    DuplicateMode,
    InvalidBin,
    OverlappingBins,
    CountMismatch,
    ArithmeticOverflow,
    SamplingExhausted,
}

impl Display for ArrivalError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let description = match self {
            Self::InvalidVersion => "version must be 1",
            Self::InvalidProvenance => "provenance must be 'synthetic_tick_fixture'",
            Self::InvalidHorizon => "horizon_ticks must be positive",
            Self::LimitExceeded => "arrival configuration exceeds a bounded limit",
            Self::InvalidMode => "mode must be 1..64 ASCII identifier bytes",
            Self::DuplicateMode => "mode occurs more than once in a bin",
            Self::InvalidBin => "bin must satisfy 0 <= start < end <= horizon",
            Self::OverlappingBins => "arrival bins overlap",
            Self::CountMismatch => "mode total does not match acuity counts",
            Self::ArithmeticOverflow => "arrival count or tick arithmetic overflowed",
            Self::SamplingExhausted => "bounded unbiased sampling exhausted its draw budget",
        };
        formatter.write_str(description)
    }
}

impl Error for ArrivalError {}

/// Generate deterministic count-conditioned arrivals for synthetic tick fixtures.
///
/// This experimental API does not create a Poisson process or imply clinical
/// arrival prevalence, dependence, or time-unit conversion.
pub fn generate_arrivals(config: &ArrivalConfig) -> Result<Vec<GeneratedArrival>, ArrivalError> {
    let total = validate_config(config)?;
    let entity = EntityId::new(ARRIVAL_DOMAIN_INDEX, 0);
    let mut stream = DeterministicStream::from_seed(derive_entity_seed(config.seed, entity));
    generate_validated(config, total, &mut || stream.next_u64())
}

fn validate_config(config: &ArrivalConfig) -> Result<u64, ArrivalError> {
    if config.version != 1 {
        return Err(ArrivalError::InvalidVersion);
    }
    if config.provenance != "synthetic_tick_fixture" {
        return Err(ArrivalError::InvalidProvenance);
    }
    if config.horizon_ticks == 0 {
        return Err(ArrivalError::InvalidHorizon);
    }

    if config.bins.len() > MAX_BINS
        || config
            .bins
            .iter()
            .any(|bin| bin.modes.len() > MAX_MODES_PER_BIN)
    {
        return Err(ArrivalError::LimitExceeded);
    }
    let mut all_modes = HashSet::new();
    let mut total_arrivals = 0u64;
    for bin in &config.bins {
        for mode in &bin.modes {
            if !valid_mode(&mode.mode) {
                return Err(ArrivalError::InvalidMode);
            }
            all_modes.insert(mode.mode.as_str());
            if all_modes.len() > MAX_DISTINCT_MODES {
                return Err(ArrivalError::LimitExceeded);
            }
            total_arrivals = total_arrivals
                .checked_add(mode.total)
                .ok_or(ArrivalError::ArithmeticOverflow)?;
        }
    }
    if total_arrivals > MAX_PATIENTS {
        return Err(ArrivalError::LimitExceeded);
    }

    let mut intervals = Vec::with_capacity(config.bins.len());
    for bin in &config.bins {
        if bin.start_tick >= bin.end_tick || bin.end_tick > config.horizon_ticks {
            return Err(ArrivalError::InvalidBin);
        }
        intervals.push((bin.start_tick, bin.end_tick));

        let mut bin_modes = HashSet::with_capacity(bin.modes.len());
        for mode in &bin.modes {
            if !valid_mode(&mode.mode) {
                return Err(ArrivalError::InvalidMode);
            }
            if !bin_modes.insert(mode.mode.as_str()) {
                return Err(ArrivalError::DuplicateMode);
            }
            let count = mode
                .acuity_counts
                .iter()
                .try_fold(0u64, |sum, count| sum.checked_add(*count))
                .ok_or(ArrivalError::ArithmeticOverflow)?;
            if count != mode.total {
                return Err(ArrivalError::CountMismatch);
            }
        }
    }
    intervals.sort_unstable();
    if intervals.windows(2).any(|pair| pair[0].1 > pair[1].0) {
        return Err(ArrivalError::OverlappingBins);
    }
    Ok(total_arrivals)
}

fn valid_mode(mode: &str) -> bool {
    let bytes = mode.as_bytes();
    !bytes.is_empty()
        && bytes.len() <= MAX_MODE_ID_BYTES
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(*byte, b'_' | b'.' | b'-'))
}

fn generate_validated<F>(
    config: &ArrivalConfig,
    total: u64,
    draw: &mut F,
) -> Result<Vec<GeneratedArrival>, ArrivalError>
where
    F: FnMut() -> u64,
{
    let mut ordered_bins = config.bins.iter().collect::<Vec<_>>();
    ordered_bins.sort_unstable_by_key(|bin| (bin.start_tick, bin.end_tick));
    let modes = config
        .bins
        .iter()
        .flat_map(|bin| bin.modes.iter().map(|row| row.mode.as_str()))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let capacity = usize::try_from(total).map_err(|_| ArrivalError::ArithmeticOverflow)?;
    let mut arrivals = Vec::with_capacity(capacity);

    for (bin_index, bin) in ordered_bins.into_iter().enumerate() {
        let width = bin.end_tick - bin.start_tick;
        for (mode_index, mode_name) in modes.iter().enumerate() {
            let Some(row) = bin.modes.iter().find(|row| row.mode == *mode_name) else {
                continue;
            };
            for (acuity_index, count) in row.acuity_counts.iter().enumerate() {
                let acuity =
                    u8::try_from(acuity_index + 1).map_err(|_| ArrivalError::ArithmeticOverflow)?;
                for ordinal in 0..*count {
                    let patient_id = format!(
                        "arrival-v1-b{bin_index:04}-m{mode_index:02}-a{acuity}-n{ordinal:06}"
                    );
                    let offset = uniform_offset_with(width, &mut *draw)?;
                    let arrival_tick = bin
                        .start_tick
                        .checked_add(offset)
                        .ok_or(ArrivalError::ArithmeticOverflow)?;
                    arrivals.push(GeneratedArrival {
                        patient_id,
                        arrival_tick,
                        mode: (*mode_name).to_owned(),
                        acuity,
                    });
                }
            }
        }
    }
    arrivals.sort_by(|left, right| {
        left.arrival_tick
            .cmp(&right.arrival_tick)
            .then_with(|| left.patient_id.cmp(&right.patient_id))
    });
    Ok(arrivals)
}

fn uniform_offset_with<F>(width: u64, mut draw: F) -> Result<u64, ArrivalError>
where
    F: FnMut() -> u64,
{
    if width == 0 {
        return Err(ArrivalError::InvalidBin);
    }
    if width == 1 {
        return Ok(0);
    }
    let threshold = width.wrapping_neg() % width;
    for _ in 0..MAX_SAMPLING_DRAWS {
        let sample = draw();
        if sample >= threshold {
            return Ok(sample % width);
        }
    }
    Err(ArrivalError::SamplingExhausted)
}

#[cfg(test)]
fn generate_arrivals_with_draw<F>(
    config: &ArrivalConfig,
    mut draw: F,
) -> Result<Vec<GeneratedArrival>, ArrivalError>
where
    F: FnMut() -> u64,
{
    let total = validate_config(config)?;
    generate_validated(config, total, &mut draw)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kairo_ecs_types::EntityId;

    fn row(mode: &str, counts: [u64; 5]) -> ArrivalModeCounts {
        ArrivalModeCounts {
            mode: mode.to_owned(),
            total: counts.iter().sum(),
            acuity_counts: counts,
        }
    }

    fn config(seed: u64, bins: Vec<ArrivalBin>) -> ArrivalConfig {
        ArrivalConfig {
            version: 1,
            seed,
            horizon_ticks: 10,
            provenance: "synthetic_tick_fixture".to_owned(),
            bins,
        }
    }

    fn bin(start_tick: u64, end_tick: u64, modes: Vec<ArrivalModeCounts>) -> ArrivalBin {
        ArrivalBin {
            start_tick,
            end_tick,
            modes,
        }
    }

    fn golden_config() -> ArrivalConfig {
        config(
            11,
            vec![bin(
                0,
                10,
                vec![row("beta", [0, 1, 0, 2, 0]), row("alpha", [1, 0, 0, 0, 1])],
            )],
        )
    }

    fn reference_splitmix64(mut x: u64) -> u64 {
        x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = x;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn reference_derived_seed(run_seed: u64, entity: EntityId) -> u64 {
        let mut state = run_seed.wrapping_add(0xA8E5_1B2C_4D6F_9013);
        state ^= reference_splitmix64(entity.index.wrapping_add(0x9E37_79B9_7F4A_7C15));
        state = state.rotate_left(17);
        state = state.wrapping_add(
            reference_splitmix64(u64::from(entity.generation).wrapping_add(0xBF58_476D_1CE4_E5B9))
                .wrapping_mul(0x94D0_49BB_1331_11EB),
        );
        reference_splitmix64(state ^ 0xD6E8_FEB8_6659_FD93)
    }

    fn reference_next(state: &mut u64) -> u64 {
        *state = reference_splitmix64(*state);
        *state
    }

    #[test]
    fn independent_reference_and_generator_match_canonical_non_power_of_two_vector() {
        let domain = 0x4341_5245_4F50_5341;
        let reference_seed = reference_derived_seed(11, EntityId::new(domain, 0));
        assert_eq!(reference_seed, 11_256_203_924_989_987_342);
        let mut reference_state = reference_seed;
        assert_eq!(
            (0..5)
                .map(|_| reference_next(&mut reference_state))
                .collect::<Vec<_>>(),
            vec![
                17_821_187_095_488_769_459,
                13_413_196_433_402_241_925,
                8_498_566_457_648_754_569,
                10_697_322_012_517_942_665,
                3_759_853_232_367_487_096,
            ]
        );

        let arrivals = generate_arrivals(&golden_config()).unwrap();
        assert_eq!(
            arrivals,
            vec![
                GeneratedArrival {
                    patient_id: "arrival-v1-b0000-m00-a5-n000000".to_owned(),
                    arrival_tick: 5,
                    mode: "alpha".to_owned(),
                    acuity: 5,
                },
                GeneratedArrival {
                    patient_id: "arrival-v1-b0000-m01-a4-n000000".to_owned(),
                    arrival_tick: 5,
                    mode: "beta".to_owned(),
                    acuity: 4,
                },
                GeneratedArrival {
                    patient_id: "arrival-v1-b0000-m01-a4-n000001".to_owned(),
                    arrival_tick: 6,
                    mode: "beta".to_owned(),
                    acuity: 4,
                },
                GeneratedArrival {
                    patient_id: "arrival-v1-b0000-m00-a1-n000000".to_owned(),
                    arrival_tick: 9,
                    mode: "alpha".to_owned(),
                    acuity: 1,
                },
                GeneratedArrival {
                    patient_id: "arrival-v1-b0000-m01-a2-n000000".to_owned(),
                    arrival_tick: 9,
                    mode: "beta".to_owned(),
                    acuity: 2,
                },
            ]
        );
    }

    #[test]
    fn empty_width_one_adjacent_bins_and_excluded_endpoints() {
        assert!(generate_arrivals(&config(5, Vec::new()))
            .unwrap()
            .is_empty());

        let mut draws = 0;
        assert_eq!(
            uniform_offset_with(1, || {
                draws += 1;
                panic!("width one must not draw")
            }),
            Ok(0)
        );
        assert_eq!(draws, 0);

        let adjacent = config(
            5,
            vec![
                bin(1, 2, vec![row("m", [1, 0, 0, 0, 0])]),
                bin(0, 1, vec![row("m", [1, 0, 0, 0, 0])]),
            ],
        );
        let arrivals = generate_arrivals(&adjacent).unwrap();
        assert_eq!(arrivals.len(), 2);
        assert_eq!(
            arrivals
                .iter()
                .map(|item| item.arrival_tick)
                .collect::<Vec<_>>(),
            vec![0, 1]
        );
        assert!(arrivals.iter().all(|item| item.arrival_tick < 2));
    }

    #[test]
    fn exact_mode_acuity_counts_and_input_permutations_are_canonical() {
        let first = config(
            91,
            vec![
                bin(
                    5,
                    10,
                    vec![row("zeta", [0, 0, 0, 0, 1]), row("beta", [0, 2, 0, 0, 0])],
                ),
                bin(0, 5, vec![row("alpha", [1, 0, 1, 0, 0])]),
            ],
        );
        let mut permuted = first.clone();
        permuted.bins.reverse();
        for bin in &mut permuted.bins {
            bin.modes.reverse();
        }
        assert_eq!(
            generate_arrivals(&first).unwrap(),
            generate_arrivals(&permuted).unwrap()
        );
        let mut counts = std::collections::BTreeMap::new();
        for item in generate_arrivals(&first).unwrap() {
            *counts.entry((item.mode, item.acuity)).or_insert(0usize) += 1;
        }
        assert_eq!(
            counts,
            [
                (("alpha".to_owned(), 1), 1),
                (("alpha".to_owned(), 3), 1),
                (("beta".to_owned(), 2), 2),
                (("zeta".to_owned(), 5), 1),
            ]
            .into_iter()
            .collect()
        );

        let mut changed_seed = first.clone();
        changed_seed.seed = 92;
        let ids = |items: Vec<GeneratedArrival>| {
            let mut ids = items
                .into_iter()
                .map(|item| item.patient_id)
                .collect::<Vec<_>>();
            ids.sort();
            ids
        };
        assert_eq!(
            ids(generate_arrivals(&first).unwrap()),
            ids(generate_arrivals(&changed_seed).unwrap())
        );
    }

    #[test]
    fn accepts_limit_and_rejects_one_above_before_sampling() {
        let accepted = config(4, vec![bin(0, 1, vec![row("a", [100_000, 0, 0, 0, 0])])]);
        assert_eq!(generate_arrivals(&accepted).unwrap().len(), 100_000);

        let rejected = config(4, vec![bin(0, 1, vec![row("a", [100_001, 0, 0, 0, 0])])]);
        assert_eq!(
            generate_arrivals(&rejected),
            Err(ArrivalError::LimitExceeded)
        );
    }

    #[test]
    fn rejects_each_configuration_error_variant() {
        let mut value = golden_config();
        value.version = 2;
        assert_eq!(generate_arrivals(&value), Err(ArrivalError::InvalidVersion));

        let mut value = golden_config();
        value.provenance = "observed".to_owned();
        assert_eq!(
            generate_arrivals(&value),
            Err(ArrivalError::InvalidProvenance)
        );

        let mut value = golden_config();
        value.horizon_ticks = 0;
        assert_eq!(generate_arrivals(&value), Err(ArrivalError::InvalidHorizon));

        let mut value = config(1, vec![bin(0, 1, vec![]); MAX_BINS + 1]);
        assert_eq!(generate_arrivals(&value), Err(ArrivalError::LimitExceeded));

        let many_rows = (0..=MAX_MODES_PER_BIN)
            .map(|i| row(&format!("m{i}"), [0; 5]))
            .collect();
        value = config(1, vec![bin(0, 1, many_rows)]);
        assert_eq!(generate_arrivals(&value), Err(ArrivalError::LimitExceeded));

        let rows = (0..MAX_MODES_PER_BIN)
            .map(|i| row(&format!("m{i}"), [0; 5]))
            .collect::<Vec<_>>();
        value = config(
            1,
            vec![bin(0, 1, rows), bin(1, 2, vec![row("extra", [0; 5])])],
        );
        assert_eq!(generate_arrivals(&value), Err(ArrivalError::LimitExceeded));

        let exact_mode_limit = (0..MAX_DISTINCT_MODES)
            .map(|i| row(&format!("mode{i:02}"), [0; 5]))
            .collect();
        value = config(1, vec![bin(0, 1, exact_mode_limit)]);
        assert!(generate_arrivals(&value).unwrap().is_empty());

        value = ArrivalConfig {
            horizon_ticks: MAX_BINS as u64,
            bins: (0..MAX_BINS)
                .map(|i| bin(i as u64, i as u64 + 1, vec![]))
                .collect(),
            ..config(1, vec![])
        };
        assert!(generate_arrivals(&value).unwrap().is_empty());

        value = config(1, vec![bin(0, 1, vec![row("a", [100_001, 0, 0, 0, 0])])]);
        assert_eq!(generate_arrivals(&value), Err(ArrivalError::LimitExceeded));

        let mut value = golden_config();
        value.bins[0].modes[0].mode = "bad/name".to_owned();
        assert_eq!(generate_arrivals(&value), Err(ArrivalError::InvalidMode));
        let long_mode = "a".repeat(MAX_MODE_ID_BYTES + 1);
        for invalid_mode in ["", "mód", long_mode.as_str()] {
            let value = config(1, vec![bin(0, 1, vec![row(invalid_mode, [0; 5])])]);
            assert_eq!(generate_arrivals(&value), Err(ArrivalError::InvalidMode));
        }

        value = config(1, vec![bin(0, 1, vec![row("a", [0; 5]), row("a", [0; 5])])]);
        assert_eq!(generate_arrivals(&value), Err(ArrivalError::DuplicateMode));

        value = config(1, vec![bin(0, 11, vec![])]);
        assert_eq!(generate_arrivals(&value), Err(ArrivalError::InvalidBin));
        value = config(1, vec![bin(2, 2, vec![])]);
        assert_eq!(generate_arrivals(&value), Err(ArrivalError::InvalidBin));

        value = config(1, vec![bin(0, 4, vec![]), bin(3, 5, vec![])]);
        assert_eq!(
            generate_arrivals(&value),
            Err(ArrivalError::OverlappingBins)
        );
        value = config(1, vec![bin(0, 4, vec![]), bin(0, 4, vec![])]);
        assert_eq!(
            generate_arrivals(&value),
            Err(ArrivalError::OverlappingBins)
        );

        let mut mismatch = row("a", [1, 0, 0, 0, 0]);
        mismatch.total = 2;
        value = config(1, vec![bin(0, 1, vec![mismatch])]);
        assert_eq!(generate_arrivals(&value), Err(ArrivalError::CountMismatch));

        value = config(
            1,
            vec![bin(
                0,
                1,
                vec![ArrivalModeCounts {
                    mode: "a".into(),
                    total: 0,
                    acuity_counts: [u64::MAX, 1, 0, 0, 0],
                }],
            )],
        );
        assert_eq!(
            generate_arrivals(&value),
            Err(ArrivalError::ArithmeticOverflow)
        );

        value = config(
            1,
            vec![
                bin(0, 1, vec![row("a", [u64::MAX, 0, 0, 0, 0])]),
                bin(1, 2, vec![row("a", [1, 0, 0, 0, 0])]),
            ],
        );
        assert_eq!(
            generate_arrivals(&value),
            Err(ArrivalError::ArithmeticOverflow)
        );
    }

    #[test]
    fn serde_rejects_unknown_fields_and_wrong_acuity_array_length() {
        let source = serde_json::json!({
            "version": 1,
            "seed": 1,
            "horizon_ticks": 2,
            "provenance": "synthetic_tick_fixture",
            "bins": [{"start_tick": 0, "end_tick": 1, "modes": []}],
            "unexpected": true
        });
        assert!(serde_json::from_value::<ArrivalConfig>(source).is_err());

        let source = serde_json::json!({
            "version": 1,
            "seed": 1,
            "horizon_ticks": 2,
            "provenance": "synthetic_tick_fixture",
            "bins": [{"start_tick": 0, "end_tick": 1, "modes": [
                {"mode":"a", "total":1, "acuity_counts":[1,0,0,0]}
            ]}]
        });
        assert!(serde_json::from_value::<ArrivalConfig>(source).is_err());
    }

    #[test]
    fn unbiased_threshold_retry_and_bounded_exhaustion() {
        let mut draws = [5, 17].into_iter();
        assert_eq!(uniform_offset_with(10, || draws.next().unwrap()), Ok(7));
        assert!(matches!(
            uniform_offset_with(10, || 0),
            Err(ArrivalError::SamplingExhausted)
        ));
    }

    #[test]
    fn sampling_error_returns_no_partial_schedule_and_validation_draws_nothing() {
        let mut value = golden_config();
        let mut calls = 0usize;
        let result = generate_arrivals_with_draw(&value, || {
            calls += 1;
            if calls == 1 {
                7
            } else {
                0
            }
        });
        assert_eq!(result, Err(ArrivalError::SamplingExhausted));
        assert_eq!(calls, 129);

        value.bins[0].modes[0].total += 1;
        calls = 0;
        assert_eq!(
            generate_arrivals_with_draw(&value, || {
                calls += 1;
                7
            }),
            Err(ArrivalError::CountMismatch)
        );
        assert_eq!(calls, 0);
    }
}
