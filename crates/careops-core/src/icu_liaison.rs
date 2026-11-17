//! Illustrative ICU liaison cost-consequence probabilistic simulation.
//!
//! This model does not simulate clinical decision-making or event-time queues.
//! Its results are hypothetical until site-specific inputs are independently qualified.
//! Input provenance is explicit: public benchmarks are not local estimates.

use kairo_ecs_rng::DeterministicStream;
use serde::{Deserialize, Serialize};

const MAX_ITERATIONS: usize = 100_000;
const MAX_ANNUAL_DISCHARGES: u64 = 10_000_000;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub schema_version: u32,
    pub scenario_id: String,
    pub data_class: String,
    pub seed: u64,
    pub iterations: usize,
    pub annual_eligible_discharges: u64,
    pub inputs: Inputs,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Inputs {
    pub delayed_discharge_ge12h_rate: Parameter,
    pub delayed_cases_addressable_fraction: Parameter,
    pub addressable_cases_improved_fraction: Parameter,
    pub hours_released_per_improved_case: Parameter,
    pub unplanned_72h_readmission_risk: Parameter,
    pub relative_readmission_reduction: Parameter,
    pub additional_cost_per_readmission_aud: Parameter,
    pub net_avoidable_cost_per_icu_day_aud: Parameter,
    pub annual_incremental_service_cost_aud: Parameter,
}

/// Triangular uncertainty, with mode equal to value. Bounds equal to value
/// produce a fixed parameter. Every field has an explicit provenance status.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Parameter {
    pub value: f64,
    pub low: f64,
    pub high: f64,
    pub status: String,
    pub source: String,
    pub url: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Interval {
    pub mean: f64,
    pub p05: f64,
    pub p50: f64,
    pub p95: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct Report {
    pub schema_version: u32,
    pub scenario_id: String,
    pub interpretation: &'static str,
    pub seed: u64,
    pub iterations: usize,
    pub icu_bed_days_released: Interval,
    pub readmissions_avoided: Interval,
    pub gross_resource_benefit_aud: Interval,
    pub net_resource_benefit_aud: Interval,
    pub probability_net_benefit_nonnegative: f64,
}

impl Parameter {
    fn validate(&self, name: &str, minimum: f64, maximum: f64) -> Result<(), String> {
        if !self.value.is_finite() || !self.low.is_finite() || !self.high.is_finite() {
            return Err(format!("{name}: input values must be finite"));
        }
        if self.low < minimum
            || self.high > maximum
            || self.low > self.value
            || self.value > self.high
        {
            return Err(format!("{name}: invalid triangular bounds or supported range"));
        }
        if !matches!(
            self.status.as_str(),
            "public_benchmark" | "illustrative_assumption" | "local_input_unvalidated"
        ) {
            return Err(format!("{name}: unsupported provenance status"));
        }
        if self.source.trim().is_empty()
            || (self.status == "public_benchmark"
                && self.url.as_deref().is_none_or(|url| url.trim().is_empty()))
        {
            return Err(format!("{name}: missing provenance"));
        }
        Ok(())
    }

    fn sample(&self, stream: &mut DeterministicStream) -> f64 {
        if self.low == self.high {
            return self.value;
        }
        // The upstream deterministic RNG supplies the bits; only the triangular
        // input-distribution inverse CDF is model-specific to this pilot.
        let u = ((stream.next_u64() >> 11) as f64) / ((1u64 << 53) as f64);
        let width = self.high - self.low;
        let midpoint = (self.value - self.low) / width;
        if u < midpoint {
            self.low + (u * width * (self.value - self.low)).sqrt()
        } else {
            self.high - ((1.0 - u) * width * (self.high - self.value)).sqrt()
        }
    }
}

impl Scenario {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1
            || self.scenario_id.trim().is_empty()
            || self.data_class != "synthetic"
            || !(1..=MAX_ITERATIONS).contains(&self.iterations)
            || !(1..=MAX_ANNUAL_DISCHARGES).contains(&self.annual_eligible_discharges)
        {
            return Err("invalid scenario metadata, bounds or data_class".to_string());
        }
        for (name, parameter, minimum, maximum) in [
            ("delayed_discharge_ge12h_rate", &self.inputs.delayed_discharge_ge12h_rate, 0.0, 1.0),
            ("delayed_cases_addressable_fraction", &self.inputs.delayed_cases_addressable_fraction, 0.0, 1.0),
            ("addressable_cases_improved_fraction", &self.inputs.addressable_cases_improved_fraction, 0.0, 1.0),
            ("hours_released_per_improved_case", &self.inputs.hours_released_per_improved_case, 0.0, f64::MAX),
            ("unplanned_72h_readmission_risk", &self.inputs.unplanned_72h_readmission_risk, 0.0, 1.0),
            ("relative_readmission_reduction", &self.inputs.relative_readmission_reduction, -1.0, 1.0),
            ("additional_cost_per_readmission_aud", &self.inputs.additional_cost_per_readmission_aud, 0.0, f64::MAX),
            ("net_avoidable_cost_per_icu_day_aud", &self.inputs.net_avoidable_cost_per_icu_day_aud, 0.0, f64::MAX),
            ("annual_incremental_service_cost_aud", &self.inputs.annual_incremental_service_cost_aud, 0.0, f64::MAX),
        ] {
            parameter.validate(name, minimum, maximum)?;
        }
        Ok(())
    }
}

fn interval(mut values: Vec<f64>) -> Interval {
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    values.sort_by(f64::total_cmp);
    let quantile = |fraction: f64| {
        let index = ((values.len() - 1) as f64 * fraction).round() as usize;
        values[index]
    };
    Interval {
        mean,
        p05: quantile(0.05),
        p50: quantile(0.50),
        p95: quantile(0.95),
    }
}

/// Propagate uncertain parameters through an annual cohort cost-consequence model.
///
/// This is a probabilistic sensitivity analysis, not an ICU discrete-event model.
/// Cohort expected events are used within each replication; sampling uncertainty
/// in observed patient counts and time-of-day bed demand is not modelled.
/// The discharge-delay value applies only to index discharge-ready hours;
/// readmission costs apply only to subsequent return episodes, so they do not overlap.
pub fn simulate(scenario: &Scenario) -> Result<Report, String> {
    scenario.validate()?;
    let n = scenario.annual_eligible_discharges as f64;
    let mut stream = DeterministicStream::from_seed(scenario.seed);
    let mut days = Vec::with_capacity(scenario.iterations);
    let mut readmissions = Vec::with_capacity(scenario.iterations);
    let mut gross = Vec::with_capacity(scenario.iterations);
    let mut net = Vec::with_capacity(scenario.iterations);
    let mut nonnegative = 0usize;

    for _ in 0..scenario.iterations {
        let i = &scenario.inputs;
        let delayed_rate = i.delayed_discharge_ge12h_rate.sample(&mut stream);
        let addressable = i.delayed_cases_addressable_fraction.sample(&mut stream);
        let improved = i.addressable_cases_improved_fraction.sample(&mut stream);
        let hours = i.hours_released_per_improved_case.sample(&mut stream);
        let baseline_readmission = i.unplanned_72h_readmission_risk.sample(&mut stream);
        let relative_effect = i.relative_readmission_reduction.sample(&mut stream);
        let return_cost = i.additional_cost_per_readmission_aud.sample(&mut stream);
        let daily_cost = i.net_avoidable_cost_per_icu_day_aud.sample(&mut stream);
        let service_cost = i.annual_incremental_service_cost_aud.sample(&mut stream);

        let released_days = n * delayed_rate * addressable * improved * hours / 24.0;
        let treated_risk = (baseline_readmission * (1.0 - relative_effect)).clamp(0.0, 1.0);
        let returns_avoided = n * (baseline_readmission - treated_risk);
        let benefit = released_days * daily_cost + returns_avoided * return_cost;
        let net_benefit = benefit - service_cost;

        days.push(released_days);
        readmissions.push(returns_avoided);
        gross.push(benefit);
        net.push(net_benefit);
        if net_benefit >= 0.0 {
            nonnegative += 1;
        }
    }

    Ok(Report {
        schema_version: 1,
        scenario_id: scenario.scenario_id.clone(),
        interpretation: "illustrative_probabilistic_scenario_not_a_chhhs_effect_estimate",
        seed: scenario.seed,
        iterations: scenario.iterations,
        icu_bed_days_released: interval(days),
        readmissions_avoided: interval(readmissions),
        gross_resource_benefit_aud: interval(gross),
        net_resource_benefit_aud: interval(net),
        probability_net_benefit_nonnegative: nonnegative as f64 / scenario.iterations as f64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Scenario {
        serde_json::from_str(include_str!(
            "../../../model-inputs/icu-liaison/illustrative.json"
        ))
        .expect("committed illustrative fixture must parse")
    }

    fn fixed(parameter: &mut Parameter, value: f64) {
        parameter.value = value;
        parameter.low = value;
        parameter.high = value;
    }

    #[test]
    fn same_seed_and_inputs_produce_identical_report() {
        let scenario = fixture();
        let first = serde_json::to_value(simulate(&scenario).unwrap()).unwrap();
        let second = serde_json::to_value(simulate(&scenario).unwrap()).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn no_service_effect_releases_no_days_and_avoids_no_returns() {
        let mut scenario = fixture();
        fixed(&mut scenario.inputs.addressable_cases_improved_fraction, 0.0);
        fixed(&mut scenario.inputs.relative_readmission_reduction, 0.0);
        fixed(&mut scenario.inputs.annual_incremental_service_cost_aud, 250_000.0);
        let result = simulate(&scenario).unwrap();
        assert_eq!(result.icu_bed_days_released.mean, 0.0);
        assert_eq!(result.readmissions_avoided.mean, 0.0);
        assert_eq!(result.gross_resource_benefit_aud.mean, 0.0);
        assert!((result.net_resource_benefit_aud.mean + 250_000.0).abs() < 1e-7);
    }

    #[test]
    fn scenario_can_model_higher_readmission_after_surveillance() {
        let mut scenario = fixture();
        fixed(&mut scenario.inputs.relative_readmission_reduction, -0.20);
        let result = simulate(&scenario).unwrap();
        assert!(result.readmissions_avoided.mean < 0.0);
    }

    #[test]
    fn fixed_inputs_match_independent_accounting_identity() {
        let mut scenario = fixture();
        for p in [
            &mut scenario.inputs.delayed_discharge_ge12h_rate,
            &mut scenario.inputs.delayed_cases_addressable_fraction,
            &mut scenario.inputs.addressable_cases_improved_fraction,
            &mut scenario.inputs.hours_released_per_improved_case,
            &mut scenario.inputs.unplanned_72h_readmission_risk,
            &mut scenario.inputs.relative_readmission_reduction,
            &mut scenario.inputs.additional_cost_per_readmission_aud,
            &mut scenario.inputs.net_avoidable_cost_per_icu_day_aud,
            &mut scenario.inputs.annual_incremental_service_cost_aud,
        ] {
            p.low = p.value;
            p.high = p.value;
        }
        let out = simulate(&scenario).unwrap();
        let expected_days = 1000.0 * 0.204 * 0.45 * 0.5 * 4.0 / 24.0;
        let expected_returns = 1000.0 * 0.03 * 0.15;
        let expected_gross = expected_days * 1800.0 + expected_returns * 15000.0;
        assert!((out.icu_bed_days_released.mean - expected_days).abs() < 1e-9);
        assert!((out.readmissions_avoided.mean - expected_returns).abs() < 1e-9);
        assert!((out.net_resource_benefit_aud.mean - (expected_gross - 250000.0)).abs() < 1e-6);
    }

    #[test]
    fn rejects_missing_public_source_or_invalid_probabilities() {
        let mut scenario = fixture();
        scenario.inputs.delayed_discharge_ge12h_rate.url = None;
        assert!(simulate(&scenario).is_err());
        let mut scenario = fixture();
        scenario.inputs.unplanned_72h_readmission_risk.high = 1.01;
        assert!(simulate(&scenario).is_err());
    }

    #[test]
    fn rejects_unknown_scenario_version_and_non_synthetic_import() {
        let mut scenario = fixture();
        scenario.schema_version = 2;
        assert!(simulate(&scenario).is_err());
        let mut scenario = fixture();
        scenario.data_class = "patient_records".to_string();
        assert!(simulate(&scenario).is_err());
    }
}
