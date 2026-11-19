//! Bounded stable-toolchain checks of the same oracle used by libFuzzer.
//! Synthetic fixtures only; this suite does not establish sanitizer coverage.

use careops_ed::parse_scenario_json;
use serde_json::{json, Value};

#[path = "../../../fuzz/scenario_oracle.rs"]
mod scenario_oracle;

const SEED: &str = include_str!("../examples/one_patient.json");

#[test]
fn synthetic_seed_is_admitted_before_mutation() {
    let parsed = parse_scenario_json(SEED).expect("synthetic seed must reach validation");
    assert_eq!(parsed.provenance.class, "synthetic");
    assert!(!parsed.patients.is_empty());
    scenario_oracle::check(SEED.as_bytes());
}

#[test]
fn every_prefix_and_single_byte_mutation_preserves_round_trip() {
    let seed = SEED.as_bytes();
    assert!(seed.len() <= scenario_oracle::MAX_INPUT_BYTES);
    // Exhaustive prefix truncation plus deterministic replacements at every byte.
    // The oracle permits rejection, but a panic or invalid accepted round trip fails.
    for end in 0..=seed.len() {
        scenario_oracle::check(&seed[..end]);
    }
    let mut mutated = seed.to_vec();
    for index in 0..seed.len() {
        for replacement in [0, 0xff, b'{', b'}', b'"', b'\\', b'\n'] {
            mutated[index] = replacement;
            scenario_oracle::check(&mutated);
        }
        mutated[index] = seed[index];
    }
}

#[test]
fn numeric_and_unicode_variants_preserve_admitted_values() {
    let original: Value = serde_json::from_str(SEED).unwrap();
    for seed in [0_u64, 1, u64::from(u32::MAX), u64::MAX] {
        for identifier in ["synthetic-a", "synthetic-\u{03bb}", "synthetic-\u{1f52c}"] {
            let mut value = original.clone();
            value["seed"] = json!(seed);
            value["scenario_id"] = json!(identifier);
            let source = value.to_string();
            let parsed = parse_scenario_json(&source).expect("generated valid variant");
            assert_eq!(parsed.seed, seed);
            assert_eq!(parsed.scenario_id, identifier);
            scenario_oracle::check(source.as_bytes());
        }
    }
}

#[test]
fn generated_invalid_envelopes_are_rejected() {
    let original: Value = serde_json::from_str(SEED).unwrap();
    for field in original.as_object().unwrap().keys() {
        let mut value = original.clone();
        value.as_object_mut().unwrap().remove(field);
        assert!(
            parse_scenario_json(&value.to_string()).is_err(),
            "missing {field}"
        );
    }
    for (field, invalid) in [
        ("schema_version", json!(0)),
        ("schema_version", json!(u64::MAX)),
        ("seed", json!(-1)),
        ("seed", json!("18446744073709551616")),
        ("horizon_ticks", json!(0)),
        ("horizon_ticks", json!(-1)),
        ("time_unit", json!("seconds")),
        ("patients", json!([])),
    ] {
        let mut value = original.clone();
        value[field] = invalid;
        assert!(
            parse_scenario_json(&value.to_string()).is_err(),
            "invalid {field}"
        );
    }
    for pointer in ["", "/provenance", "/capacity_location", "/patients/0"] {
        let mut value = original.clone();
        value
            .pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("unexpected_rollout_field".to_owned(), json!(true));
        assert!(
            parse_scenario_json(&value.to_string()).is_err(),
            "extra field at {pointer}"
        );
    }
}
