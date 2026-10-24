//! Independent clinical progression, proportional hazards, and competing risks (CC-04).

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use super::error::{PathwayError, PathwayErrorKind};

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
