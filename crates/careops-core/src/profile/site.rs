//! Facility site profiles, capacity configurations, and site-level overrides (CC-05).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::error::{ProfileError, ProfileErrorKind};
use super::provenance::ProfileProvenance;

/// Typology of healthcare facilities.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SiteType {
    TertiaryHospital,
    RegionalHospital,
    UrgentCareCentre,
    CommunityClinic,
}

/// Baseline physical and staffed capacity profile for a facility.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SiteCapacityProfile {
    pub resuscitation_bays: usize,
    pub acute_treatment_bays: usize,
    pub subacute_consult_rooms: usize,
    pub short_stay_beds: usize,
    pub nominal_nurse_ratio: f64,
    pub nominal_physician_ratio: f64,
}

impl Default for SiteCapacityProfile {
    fn default() -> Self {
        Self {
            resuscitation_bays: 4,
            acute_treatment_bays: 24,
            subacute_consult_rooms: 12,
            short_stay_beds: 16,
            nominal_nurse_ratio: 0.25,      // 1 nurse per 4 acute patients
            nominal_physician_ratio: 0.125, // 1 doctor per 8 acute patients
        }
    }
}

/// Facility-specific site profile referencing a governing jurisdiction.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SiteProfile {
    pub schema_version: u32,
    pub site_id: String,
    pub jurisdiction_id: String,
    pub name: String,
    pub site_type: SiteType,
    pub capacity: SiteCapacityProfile,
    pub turnover_cleaning_ticks: Option<u64>,
    pub local_parameters: HashMap<String, String>,
    pub provenance: ProfileProvenance,
}

impl SiteProfile {
    /// Creates a standard metropolitan tertiary emergency facility profile.
    pub fn default_tertiary_hospital(jurisdiction_id: &str) -> Self {
        Self {
            schema_version: 1,
            site_id: "site-metropolitan-tertiary-01".to_string(),
            jurisdiction_id: jurisdiction_id.to_string(),
            name: "Metropolitan Tertiary Referral Centre".to_string(),
            site_type: SiteType::TertiaryHospital,
            capacity: SiteCapacityProfile {
                resuscitation_bays: 6,
                acute_treatment_bays: 36,
                subacute_consult_rooms: 16,
                short_stay_beds: 24,
                nominal_nurse_ratio: 0.33,
                nominal_physician_ratio: 0.16,
            },
            turnover_cleaning_ticks: Some(25), // Local site efficiency override
            local_parameters: HashMap::new(),
            provenance: ProfileProvenance {
                source_authority: "Metropolitan Health District Operational Plan".to_string(),
                citation: "MHD-ED-2026-CAP".to_string(),
                source_sha256: None,
                author: "careops-core".to_string(),
                notes: "Reference tertiary facility profile.".to_string(),
            },
        }
    }

    /// Creates a standard regional emergency facility profile.
    pub fn default_regional_hospital(jurisdiction_id: &str) -> Self {
        Self {
            schema_version: 1,
            site_id: "site-regional-base-01".to_string(),
            jurisdiction_id: jurisdiction_id.to_string(),
            name: "Regional Base Hospital".to_string(),
            site_type: SiteType::RegionalHospital,
            capacity: SiteCapacityProfile {
                resuscitation_bays: 2,
                acute_treatment_bays: 12,
                subacute_consult_rooms: 6,
                short_stay_beds: 8,
                nominal_nurse_ratio: 0.25,
                nominal_physician_ratio: 0.10,
            },
            turnover_cleaning_ticks: None, // Uses jurisdiction default
            local_parameters: HashMap::new(),
            provenance: ProfileProvenance {
                source_authority: "Regional Health Service Bed Census".to_string(),
                citation: "RHS-2026-EST".to_string(),
                source_sha256: None,
                author: "careops-core".to_string(),
                notes: "Reference regional base hospital profile.".to_string(),
            },
        }
    }

    /// Validates internal consistency of the site profile.
    pub fn validate(&self) -> Result<(), ProfileError> {
        if self.schema_version != 1 {
            return Err(ProfileError::new(
                ProfileErrorKind::IncompatibleVersion,
                format!("unsupported schema version: {}", self.schema_version),
            ));
        }

        if self.site_id.trim().is_empty() {
            return Err(ProfileError::new(
                ProfileErrorKind::ValidationFailed,
                "site_id cannot be empty",
            ));
        }

        if self.jurisdiction_id.trim().is_empty() {
            return Err(ProfileError::new(
                ProfileErrorKind::ValidationFailed,
                "jurisdiction_id reference cannot be empty",
            ));
        }

        if self.capacity.acute_treatment_bays == 0 {
            return Err(ProfileError::new(
                ProfileErrorKind::ValidationFailed,
                "site capacity must include at least one acute treatment bay",
            ));
        }

        Ok(())
    }
}
