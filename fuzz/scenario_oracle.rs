//! Shared oracle for the stable mutation tests and the libFuzzer target.

use careops_ed::parse_scenario_json;

pub const MAX_INPUT_BYTES: usize = 65_536;

pub fn check(data: &[u8]) {
    if data.len() > MAX_INPUT_BYTES {
        return;
    }
    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };
    if let Ok(parsed) = parse_scenario_json(source) {
        let serialized = serde_json::to_string(&parsed).expect("ScenarioConfig serializes");
        let reparsed = parse_scenario_json(&serialized)
            .expect("a validated ScenarioConfig remains valid after serialization");
        assert_eq!(parsed, reparsed);
    }
}
