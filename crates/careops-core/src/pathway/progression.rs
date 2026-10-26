//! Independent clinical progression, proportional hazards, and competing risks (CC-04).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::error::{PathwayError, PathwayErrorKind};
use super::streams::SubsystemStream;

/// Competing hazard rate definition.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HazardRate {
    pub cause_id: String,
    pub base_rate: f64,
    pub covariate_coefficients: HashMap<String, f64>,
}

impl HazardRate {
    pub fn new(cause_id: impl Into<String>, base_rate: f64) -> Self {
        Self {
            cause_id: cause_id.into(),
            base_rate,
            covariate_coefficients: HashMap::new(),
        }
    }

    /// Evaluates effective hazard given current covariate values.
    pub fn evaluate(&self, covariates: &HashMap<String, f64>) -> f64 {
        if self.base_rate <= 0.0 {
            return 0.0;
        }
        let mut linear_predictor = 0.0;
        for (cov_name, &beta) in &self.covariate_coefficients {
            let val = covariates.get(cov_name).copied().unwrap_or(0.0);
            linear_predictor += beta * val;
        }
        self.base_rate * linear_predictor.exp()
    }
}

/// Competing risk model evaluating multiple simultaneous clinical progression causes.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CompetingRiskModel {
    hazards: Vec<HazardRate>,
}

impl CompetingRiskModel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_hazard(&mut self, hazard: HazardRate) {
        self.hazards.push(hazard);
    }

    /// Total hazard rate under given covariates: Lambda = sum(lambda_i).
    pub fn total_hazard(&self, covariates: &HashMap<String, f64>) -> f64 {
        self.hazards.iter().map(|h| h.evaluate(covariates)).sum()
    }

    /// Samples at most one event over a unit interval and then its competing cause.
    /// The first progression-stream draw determines whether any event occurs using
    /// `1 - exp(-total_hazard)`; a second draw selects the cause proportionally.
    pub fn sample_event(
        &self,
        covariates: &HashMap<String, f64>,
        stream: &mut SubsystemStream,
    ) -> Result<Option<String>, PathwayError> {
        let rates = self.effective_rates(covariates)?;
        let total: f64 = rates.iter().map(|(_, rate)| rate).sum();
        if total == 0.0 {
            return Ok(None);
        }
        if total.is_nan() {
            return Err(PathwayError::new(
                PathwayErrorKind::InvalidCovariate,
                "total progression hazard is not finite",
            ));
        }
        let event_probability = -(-total).exp_m1();
        if stream.next_uniform() >= event_probability {
            return Ok(None);
        }
        let max_rate = rates.iter().map(|(_, rate)| *rate).fold(0.0, f64::max);
        let scaled_total: f64 = rates.iter().map(|(_, rate)| rate / max_rate).sum();
        let threshold = stream.next_uniform() * scaled_total;
        let mut cumulative = 0.0;
        for (cause, rate) in &rates {
            cumulative += rate / max_rate;
            if threshold < cumulative {
                return Ok(Some(cause.clone()));
            }
        }
        // Floating-point rounding at the upper boundary resolves to the last
        // positive cause and cannot create an event without a cause.
        Ok(rates
            .iter()
            .rev()
            .find(|(_, rate)| *rate > 0.0)
            .map(|(cause, _)| cause.clone()))
    }

    fn effective_rates(
        &self,
        covariates: &HashMap<String, f64>,
    ) -> Result<Vec<(String, f64)>, PathwayError> {
        self.hazards
            .iter()
            .map(|hazard| {
                let rate = hazard.evaluate(covariates);
                if !rate.is_finite() || rate < 0.0 {
                    return Err(PathwayError::new(
                        PathwayErrorKind::InvalidCovariate,
                        format!(
                            "hazard for cause '{}' must be finite and nonnegative",
                            hazard.cause_id
                        ),
                    ));
                }
                Ok((hazard.cause_id.clone(), rate))
            })
            .collect()
    }

    /// Analytic cause probabilities: P(cause_i wins) = lambda_i / sum(lambda_j).
    pub fn analytic_cause_probabilities(
        &self,
        covariates: &HashMap<String, f64>,
    ) -> Result<HashMap<String, f64>, PathwayError> {
        let total = self.total_hazard(covariates);
        if total <= 0.0 {
            return Err(PathwayError::new(
                PathwayErrorKind::ZeroTargetError,
                "total hazard is zero; no event occurs (zero-hazard control)",
            ));
        }
        let mut probs = HashMap::new();
        for h in &self.hazards {
            let lambda_i = h.evaluate(covariates);
            probs.insert(h.cause_id.clone(), lambda_i / total);
        }
        Ok(probs)
    }

    /// Adjusts residual time when covariates change, conserving cumulative integrated hazard.
    ///
    /// If remaining residual time under old covariates is `t_rem_old`, then under new covariates:
    /// `t_rem_new = t_rem_old * (lambda_old / lambda_new)`.
    pub fn adjust_residual_time_on_covariate_change(
        &self,
        t_rem_old: f64,
        old_covariates: &HashMap<String, f64>,
        new_covariates: &HashMap<String, f64>,
    ) -> Result<f64, PathwayError> {
        let lambda_old = self.total_hazard(old_covariates);
        let lambda_new = self.total_hazard(new_covariates);

        if lambda_new <= 0.0 {
            // New hazard is zero: progression stops indefinitely
            return Ok(f64::INFINITY);
        }

        if lambda_old <= 0.0 {
            return Err(PathwayError::new(
                PathwayErrorKind::InvalidCovariate,
                "cannot scale residual time from zero base hazard",
            ));
        }

        // Proportional hazard time scaling
        Ok(t_rem_old * (lambda_old / lambda_new))
    }
}
