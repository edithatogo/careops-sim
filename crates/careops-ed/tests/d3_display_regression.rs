//! Public error formatting regression for the remaining D3 Display mutation.
use careops_ed::{parse_scenario_json, run_scenario};

#[test]
fn simulation_error_display_preserves_validation_context() {
    let mut scenario = parse_scenario_json(include_str!("../examples/one_patient.json")).unwrap();
    scenario.patients[0].work_ticks = 0;
    let error = run_scenario(&scenario).unwrap_err();
    assert_eq!(
        error.to_string(),
        "simulation failed: invalid scenario: patient 'synthetic-patient-1' work_ticks must be positive"
    );
}
