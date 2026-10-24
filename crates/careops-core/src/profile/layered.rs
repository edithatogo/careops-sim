//! Layered profile resolution enforcing strict precedence (Base -> Jurisdiction -> Site -> Run Override).

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::error::{ProfileError, ProfileErrorKind};
use super::jurisdiction::JurisdictionProfile;
use super::site::{SiteCapacityProfile, SiteProfile, SiteType};

/// Runtime scenario parameter overrides that take highest precedence.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RuntimeOverrides {
    pub cleaning_ticks: Option<u64>,
    pub max_boarding_ticks: Option<u64>,
    pub capacity_multipliers: Option<HashMap<String, f64>>,
    pub parameters: HashMap<String, String>,
}

/// Fully resolved operational and clinical profile ready for simulation execution.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayeredProfile {
    pub schema_version: u32,
    pub jurisdiction_id: String,
    pub site_id: String,
    pub facility_name: String,
    pub site_type: SiteType,
    pub time_zone: String,
    pub effective_capacity: SiteCapacityProfile,
    pub effective_cleaning_ticks: u64,
    pub effective_max_boarding_ticks: u64,
    pub resolved_parameters: HashMap<String, String>,
    pub precedence_audit: Vec<String>,
}

/// Loader and resolver for jurisdiction and site configuration profiles.
pub struct ProfileLoader;

impl ProfileLoader {
    /// Loads and parses a `JurisdictionProfile` from JSON text.
    pub fn load_jurisdiction_from_json(json_str: &str) -> Result<JurisdictionProfile, ProfileError> {
        let profile: JurisdictionProfile = serde_json::from_str(json_str).map_err(|e| {
            ProfileError::new(
                ProfileErrorKind::MalformedFormat,
                format!("failed to parse jurisdiction profile JSON: {e}"),
            )
        })?;
        profile.validate()?;
        Ok(profile)
    }

    /// Loads and parses a `JurisdictionProfile` from a file path.
    pub fn load_jurisdiction_from_file(path: &Path) -> Result<JurisdictionProfile, ProfileError> {
        if !path.exists() {
            return Err(ProfileError::new(
                ProfileErrorKind::MissingSource,
                format!("jurisdiction profile file not found: '{}'", path.display()),
            ));
        }
        let content = fs::read_to_string(path).map_err(|e| {
            ProfileError::new(
                ProfileErrorKind::MissingSource,
                format!("failed to read jurisdiction file '{}': {e}", path.display()),
            )
        })?;
        Self::load_jurisdiction_from_json(&content)
    }

    /// Loads and parses a `SiteProfile` from JSON text.
    pub fn load_site_from_json(json_str: &str) -> Result<SiteProfile, ProfileError> {
        let profile: SiteProfile = serde_json::from_str(json_str).map_err(|e| {
            ProfileError::new(
                ProfileErrorKind::MalformedFormat,
                format!("failed to parse site profile JSON: {e}"),
            )
        })?;
        profile.validate()?;
        Ok(profile)
    }

    /// Loads and parses a `SiteProfile` from a file path.
    pub fn load_site_from_file(path: &Path) -> Result<SiteProfile, ProfileError> {
        if !path.exists() {
            return Err(ProfileError::new(
                ProfileErrorKind::MissingSource,
                format!("site profile file not found: '{}'", path.display()),
            ));
        }
        let content = fs::read_to_string(path).map_err(|e| {
            ProfileError::new(
                ProfileErrorKind::MissingSource,
                format!("failed to read site file '{}': {e}", path.display()),
            )
        })?;
        Self::load_site_from_json(&content)
    }

    /// Resolves a complete operational profile with strict precedence:
    /// Base Defaults < Jurisdiction < Site < Runtime Override.
    pub fn resolve_layered_profile(
        jurisdiction: &JurisdictionProfile,
        site: &SiteProfile,
        run_overrides: Option<&RuntimeOverrides>,
    ) -> Result<LayeredProfile, ProfileError> {
        // Enforce version compatibility
        jurisdiction.validate()?;
        site.validate()?;

        // Enforce cross-profile jurisdiction referential integrity
        if site.jurisdiction_id != jurisdiction.jurisdiction_id {
            return Err(ProfileError::new(
                ProfileErrorKind::JurisdictionMismatch,
                format!(
                    "site '{}' references jurisdiction '{}', but jurisdiction profile is '{}'",
                    site.site_id, site.jurisdiction_id, jurisdiction.jurisdiction_id
                ),
            ));
        }

        let mut audit = Vec::new();
        audit.push(format!("Base: jurisdiction '{}'", jurisdiction.jurisdiction_id));
        audit.push(format!("Site: '{}' ({})", site.site_id, site.name));

        // 1. Cleaning ticks precedence: Runtime Override > Site > Jurisdiction Default
        let effective_cleaning = if let Some(override_val) = run_overrides.and_then(|r| r.cleaning_ticks) {
            audit.push(format!("cleaning_ticks: overridden by runtime ({override_val} ticks)"));
            override_val
        } else if let Some(site_val) = site.turnover_cleaning_ticks {
            audit.push(format!("cleaning_ticks: set by site profile ({site_val} ticks)"));
            site_val
        } else {
            let jur_val = jurisdiction.policies.default_cleaning_ticks;
            audit.push(format!("cleaning_ticks: set by jurisdiction default ({jur_val} ticks)"));
            jur_val
        };

        // 2. Max boarding ticks before escalation: Runtime Override > Jurisdiction Default
        let effective_boarding = if let Some(override_val) = run_overrides.and_then(|r| r.max_boarding_ticks) {
            audit.push(format!("max_boarding_ticks: overridden by runtime ({override_val} ticks)"));
            override_val
        } else {
            let jur_val = jurisdiction.policies.max_boarding_ticks_before_escalation;
            audit.push(format!("max_boarding_ticks: set by jurisdiction policy ({jur_val} ticks)"));
            jur_val
        };

        // 3. Parameters merging with layered precedence: Site params overlay Jurisdiction, Runtime overlays both
        let mut resolved_params = HashMap::new();
        for (k, v) in &site.local_parameters {
            resolved_params.insert(k.clone(), v.clone());
        }
        if let Some(r_over) = run_overrides {
            for (k, v) in &r_over.parameters {
                resolved_params.insert(k.clone(), v.clone());
                audit.push(format!("param '{k}': overridden by runtime"));
            }
        }

        Ok(LayeredProfile {
            schema_version: 1,
            jurisdiction_id: jurisdiction.jurisdiction_id.clone(),
            site_id: site.site_id.clone(),
            facility_name: site.name.clone(),
            site_type: site.site_type,
            time_zone: jurisdiction.time_zone.clone(),
            effective_capacity: site.capacity.clone(),
            effective_cleaning_ticks: effective_cleaning,
            effective_max_boarding_ticks: effective_boarding,
            resolved_parameters: resolved_params,
            precedence_audit: audit,
        })
    }
}
