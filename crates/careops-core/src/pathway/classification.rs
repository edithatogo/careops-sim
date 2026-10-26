//! Versioned urgency assessments and observed-only policy decisions (CC-06).

use serde::{Deserialize, Serialize};

use crate::profile::JurisdictionProfile;

use super::error::{PathwayError, PathwayErrorKind};

/// A recorded urgency classification bound to the policy version and source used.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionedUrgencyAssessment {
    pub jurisdiction_id: String,
    pub scheme_id: String,
    pub scheme_version: String,
    pub category: u8,
    pub assessed_at_tick: u64,
    pub effective_date: String,
    pub source_authority: String,
    pub citation: String,
    pub source_sha256: Option<String>,
}

impl VersionedUrgencyAssessment {
    pub fn from_profile(
        profile: &JurisdictionProfile,
        category: u8,
        tick: u64,
    ) -> Result<Self, PathwayError> {
        profile
            .validate()
            .map_err(|e| PathwayError::new(PathwayErrorKind::InvalidCovariate, e.to_string()))?;
        if profile
            .urgency_scheme
            .targets
            .iter()
            .filter(|t| t.category == category)
            .count()
            != 1
        {
            return Err(PathwayError::new(
                PathwayErrorKind::InvalidCovariate,
                format!("category {category} is not defined exactly once"),
            ));
        }
        Ok(Self {
            jurisdiction_id: profile.jurisdiction_id.clone(),
            scheme_id: profile.urgency_scheme.scheme_id.clone(),
            scheme_version: profile.urgency_scheme.version.clone(),
            category,
            assessed_at_tick: tick,
            effective_date: profile.effective_date.clone(),
            source_authority: profile.provenance.source_authority.clone(),
            citation: profile.provenance.citation.clone(),
            source_sha256: profile.provenance.source_sha256.clone(),
        })
    }
}

/// Append-only record of observed assessments; each reassessment retains its own version.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassificationHistory {
    assessments: Vec<VersionedUrgencyAssessment>,
}

impl ClassificationHistory {
    pub fn assessments(&self) -> &[VersionedUrgencyAssessment] {
        &self.assessments
    }
    pub fn reassess(&mut self, assessment: VersionedUrgencyAssessment) -> Result<(), PathwayError> {
        if self
            .assessments
            .last()
            .is_some_and(|last| assessment.assessed_at_tick < last.assessed_at_tick)
        {
            return Err(PathwayError::new(
                PathwayErrorKind::InvalidTransition,
                "assessment time cannot move backwards",
            ));
        }
        self.assessments.push(assessment);
        Ok(())
    }
}

/// Inputs available at the decision point. This type intentionally has no outcome or future-event fields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedPolicyState {
    pub observed_at_tick: u64,
    pub urgency: VersionedUrgencyAssessment,
    pub elapsed_wait_ticks: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UrgencyCategory(pub u8);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceTargetStatus {
    WithinTarget,
    Breached,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DispatchPriority(pub String);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InterruptionEligibility {
    NotEligible,
    EligibleAtSafePoint,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyDecision {
    pub urgency: UrgencyCategory,
    pub service_target: ServiceTargetStatus,
    pub dispatch_priority: Option<DispatchPriority>,
    pub interruption: InterruptionEligibility,
}

/// A decision policy can inspect only observed state and returns independent policy dimensions.
pub trait DecisionPolicy {
    fn decide(&self, observed: &ObservedPolicyState) -> PolicyDecision;
}
