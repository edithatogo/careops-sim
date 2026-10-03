#![no_main]

use careops_ed::parse_scenario_json;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };

    if let Ok(parsed) = parse_scenario_json(source) {
        let serialized = serde_json::to_string(&parsed).expect("ScenarioConfig serializes");
        let reparsed = parse_scenario_json(&serialized)
            .expect("a validated ScenarioConfig remains valid after serialization");
        assert_eq!(parsed, reparsed);
    }
});
