//! Jurisdiction-level policy profiles and clinical targets (CC-06).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::error::{ProfileError, ProfileErrorKind};
use super::provenance::ProfileProvenance;

/// Triage acuity category with maximum waiting time target in minutes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AcuityTarget {
    pub category: u8,
    pub name: String,
    pub max_wait_minutes: u32,
    pub immediate_service: bool,
}

/// Standardized clinical urgency scheme (e.g., Australasian Triage Scale ATS 1–5).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UrgencyScheme {
    pub scheme_id: String,
    pub version: String,
    pub targets: Vec<AcuityTarget>,
}

impl UrgencyScheme {
    /// Returns default Australasian Triage Scale (ATS) clinical definition.
    pub fn default_ats() -> Self {
        Self {
            scheme_id: "ATS".to_string(),
            version: "2026.1".to_string(),
            targets: vec![
                AcuityTarget {
                    category: 1,
                    name: "Resuscitation".to_string(),
                    max_wait_minutes: 0,
                    immediate_service: true,
                },
                AcuityTarget {
                    category: 2,
                    name: "Emergency".to_string(),
                    max_wait_minutes: 10,
                    immediate_service: false,
                },
                AcuityTarget {
                    category: 3,
                    name: "Urgent".to_string(),
                    max_wait_minutes: 30,
                    immediate_service: false,
                },
                AcuityTarget {
                    category: 4,
                    name: "Semi-urgent".to_string(),
                    max_wait_minutes: 60,
                    immediate_service: false,
                },
                AcuityTarget {
                    category: 5,
                    name: "Non-urgent".to_string(),
                    max_wait_minutes: 120,
                    immediate_service: false,
                },
            ],
        }
    }
}

/// Administrative target clock configuration.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TargetClockConfig {
    pub clock_id: String,
    pub target_duration_ticks: u64,
    pub pausable: bool,
    pub description: String,
}

/// Operational policies governed at the jurisdiction tier.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JurisdictionPolicies {
    pub default_cleaning_ticks: u64,
    pub max_boarding_ticks_before_escalation: u64,
    pub mandatory_clinical_escort_required: bool,
    pub reassessment_interval_ticks: u64,
}

impl Default for JurisdictionPolicies {
    fn default() -> Self {
        Self {
            default_cleaning_ticks: 30,
            max_boarding_ticks_before_escalation: 240,
            mandatory_clinical_escort_required: true,
            reassessment_interval_ticks: 60,
        }
    }
}

/// Complete jurisdiction-level profile specifying clinical policy defaults outside Kairos.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JurisdictionProfile {
    pub schema_version: u32,
    pub jurisdiction_id: String,
    pub name: String,
    pub effective_date: String,
    pub time_zone: String,
    pub urgency_scheme: UrgencyScheme,
    pub target_clocks: HashMap<String, TargetClockConfig>,
    pub policies: JurisdictionPolicies,
    pub provenance: ProfileProvenance,
}

impl Default for JurisdictionProfile {
    fn default() -> Self {
        Self::default_australian_jurisdiction()
    }
}

impl JurisdictionProfile {
    /// Creates the default Australian clinical emergency profile.
    pub fn default_australian_jurisdiction() -> Self {
        let mut clocks = HashMap::new();
        clocks.insert(
            "nehrt_4hour_target".to_string(),
            TargetClockConfig {
                clock_id: "nehrt_4hour_target".to_string(),
                target_duration_ticks: 240,
                pausable: false,
                description: "National Emergency Access Target (NEAT/NEHRT) 4-hour departure clock"
                    .to_string(),
            },
        );

        Self {
            schema_version: 1,
            jurisdiction_id: "au-national".to_string(),
            name: "Australian National Emergency Care Baseline".to_string(),
            effective_date: "2026-01-01".to_string(),
            time_zone: "Australia/Sydney".to_string(),
            urgency_scheme: UrgencyScheme::default_ats(),
            target_clocks: clocks,
            policies: JurisdictionPolicies::default(),
            provenance: ProfileProvenance::default(),
        }
    }

    /// Validates internal consistency of the jurisdiction profile.
    pub fn validate(&self) -> Result<(), ProfileError> {
        if self.schema_version != 1 {
            return Err(ProfileError::new(
                ProfileErrorKind::IncompatibleVersion,
                format!("unsupported schema version: {}", self.schema_version),
            ));
        }

        if self.jurisdiction_id.trim().is_empty() {
            return Err(ProfileError::new(
                ProfileErrorKind::ValidationFailed,
                "jurisdiction_id cannot be empty",
            ));
        }

        for (field, value) in [
            ("effective_date", self.effective_date.as_str()),
            (
                "urgency_scheme.scheme_id",
                self.urgency_scheme.scheme_id.as_str(),
            ),
            (
                "urgency_scheme.version",
                self.urgency_scheme.version.as_str(),
            ),
            (
                "provenance.source_authority",
                self.provenance.source_authority.as_str(),
            ),
            ("provenance.citation", self.provenance.citation.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(ProfileError::new(
                    ProfileErrorKind::ValidationFailed,
                    format!("{field} cannot be empty"),
                ));
            }
        }

        if self.urgency_scheme.targets.is_empty() {
            return Err(ProfileError::new(
                ProfileErrorKind::ValidationFailed,
                "urgency scheme must define at least one category",
            ));
        }

        // Verify ATS monotonicity: higher category numbers must have non-decreasing wait targets
        let mut prev_target = 0u32;
        for target in &self.urgency_scheme.targets {
            if target.max_wait_minutes < prev_target {
                return Err(ProfileError::new(
                    ProfileErrorKind::ValidationFailed,
                    format!(
                        "non-monotonic wait target for category {}: {} min < {} min",
                        target.category, target.max_wait_minutes, prev_target
                    ),
                ));
            }
            prev_target = target.max_wait_minutes;
        }

        Ok(())
    }
}
