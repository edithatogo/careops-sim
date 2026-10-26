//! Layered JurisdictionProfile and SiteProfile schema and loader tests.
//!
//! Validates:
//! - Precedence: Base Defaults < Jurisdiction < Site < Runtime Override.
//! - Error handling: Missing source file, incompatible schema version, jurisdiction mismatch, malformed JSON.
//! - Profile validation: Monotonic ATS targets, non-empty IDs.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use careops_core::profile::{
    JurisdictionProfile, ProfileErrorKind, ProfileLoader, RuntimeOverrides, SiteProfile, SiteType,
};

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(name: &str) -> Self {
        let unique = format!("careops_profile_test_{}_{}", name, std::process::id());
        let path = std::env::temp_dir().join(unique);
        fs::create_dir_all(&path).expect("failed to create temp dir");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn test_default_profiles_pass_validation() {
    let jur = JurisdictionProfile::default_australian_jurisdiction();
    jur.validate()
        .expect("default jurisdiction profile must be valid");

    let tertiary = SiteProfile::default_tertiary_hospital(&jur.jurisdiction_id);
    tertiary
        .validate()
        .expect("default tertiary site profile must be valid");

    let regional = SiteProfile::default_regional_hospital(&jur.jurisdiction_id);
    regional
        .validate()
        .expect("default regional site profile must be valid");
}

#[test]
fn test_layered_precedence_resolution() {
    let jur = JurisdictionProfile::default_australian_jurisdiction();
    let mut site_with_override = SiteProfile::default_tertiary_hospital(&jur.jurisdiction_id);
    site_with_override.turnover_cleaning_ticks = Some(20);
    site_with_override
        .local_parameters
        .insert("param_a".to_string(), "site_val_a".to_string());
    site_with_override
        .local_parameters
        .insert("param_b".to_string(), "site_val_b".to_string());

    // 1. Site overrides Jurisdiction default (20 ticks vs 30 ticks)
    let layered_site = ProfileLoader::resolve_layered_profile(&jur, &site_with_override, None)
        .expect("layering must succeed");
    assert_eq!(layered_site.effective_cleaning_ticks, 20);
    assert_eq!(layered_site.effective_max_boarding_ticks, 240); // from jurisdiction
    assert_eq!(
        layered_site.resolved_parameters.get("param_a").unwrap(),
        "site_val_a"
    );

    // 2. Site without override inherits Jurisdiction default (30 ticks)
    let mut site_without_override = SiteProfile::default_regional_hospital(&jur.jurisdiction_id);
    site_without_override.turnover_cleaning_ticks = None;
    let layered_jur = ProfileLoader::resolve_layered_profile(&jur, &site_without_override, None)
        .expect("layering must succeed");
    assert_eq!(layered_jur.effective_cleaning_ticks, 30); // from jurisdiction default

    // 3. Runtime override overrides both Site and Jurisdiction (12 ticks)
    let run_overrides = RuntimeOverrides {
        cleaning_ticks: Some(12),
        max_boarding_ticks: Some(180),
        parameters: HashMap::from([("param_a".to_string(), "runtime_override_a".to_string())]),
        ..RuntimeOverrides::default()
    };

    let layered_run =
        ProfileLoader::resolve_layered_profile(&jur, &site_with_override, Some(&run_overrides))
            .expect("layering with runtime overrides must succeed");
    assert_eq!(layered_run.effective_cleaning_ticks, 12);
    assert_eq!(layered_run.effective_max_boarding_ticks, 180);
    assert_eq!(
        layered_run.resolved_parameters.get("param_a").unwrap(),
        "runtime_override_a"
    );
    assert_eq!(
        layered_run.resolved_parameters.get("param_b").unwrap(),
        "site_val_b"
    );
}

#[test]
fn test_missing_source_file_fails_closed() {
    let non_existent = Path::new("/path/to/missing_jurisdiction_9999.json");
    let res = ProfileLoader::load_jurisdiction_from_file(non_existent);
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.kind, ProfileErrorKind::MissingSource);

    let non_existent_site = Path::new("/path/to/missing_site_9999.json");
    let res_site = ProfileLoader::load_site_from_file(non_existent_site);
    assert!(res_site.is_err());
    assert_eq!(res_site.unwrap_err().kind, ProfileErrorKind::MissingSource);
}

#[test]
fn test_incompatible_schema_version_fails_closed() {
    let mut jur = JurisdictionProfile::default_australian_jurisdiction();
    jur.schema_version = 2; // Unsupported version
    let json_str = serde_json::to_string(&jur).unwrap();

    let res = ProfileLoader::load_jurisdiction_from_json(&json_str);
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.kind, ProfileErrorKind::IncompatibleVersion);
    assert!(err.message.contains("unsupported schema version: 2"));

    let mut site = SiteProfile::default_tertiary_hospital("au-national");
    site.schema_version = 0; // Unsupported version
    let site_json = serde_json::to_string(&site).unwrap();
    let res_site = ProfileLoader::load_site_from_json(&site_json);
    assert!(res_site.is_err());
    assert_eq!(
        res_site.unwrap_err().kind,
        ProfileErrorKind::IncompatibleVersion
    );
}

#[test]
fn test_jurisdiction_mismatch_fails_closed() {
    let jur = JurisdictionProfile::default_australian_jurisdiction(); // "au-national"
    let site = SiteProfile::default_tertiary_hospital("au-qld-metro"); // mismatched reference

    let res = ProfileLoader::resolve_layered_profile(&jur, &site, None);
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.kind, ProfileErrorKind::JurisdictionMismatch);
    assert!(
        err.message
            .contains("references jurisdiction 'au-qld-metro'")
    );
}

#[test]
fn test_malformed_json_fails_closed() {
    let malformed = "{ this is not valid json }";
    let res = ProfileLoader::load_jurisdiction_from_json(malformed);
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().kind, ProfileErrorKind::MalformedFormat);

    let res_site = ProfileLoader::load_site_from_json(malformed);
    assert!(res_site.is_err());
    assert_eq!(
        res_site.unwrap_err().kind,
        ProfileErrorKind::MalformedFormat
    );
}

#[test]
fn test_non_monotonic_acuity_targets_rejected() {
    let mut jur = JurisdictionProfile::default_australian_jurisdiction();
    // Invert categories 2 and 3: category 2 gets 40 min, category 3 gets 20 min -> non-monotonic!
    jur.urgency_scheme.targets[1].max_wait_minutes = 40;
    jur.urgency_scheme.targets[2].max_wait_minutes = 20;

    let res = jur.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.kind, ProfileErrorKind::ValidationFailed);
    assert!(err.message.contains("non-monotonic wait target"));
}

#[test]
fn test_file_loader_roundtrip() {
    let temp = TempDir::new("file_roundtrip");
    let jur_path = temp.path().join("jurisdiction.json");
    let site_path = temp.path().join("site.json");

    let jur = JurisdictionProfile::default_australian_jurisdiction();
    let site = SiteProfile::default_regional_hospital(&jur.jurisdiction_id);

    fs::write(&jur_path, serde_json::to_string_pretty(&jur).unwrap()).unwrap();
    fs::write(&site_path, serde_json::to_string_pretty(&site).unwrap()).unwrap();

    let loaded_jur = ProfileLoader::load_jurisdiction_from_file(&jur_path)
        .expect("file loading jurisdiction must succeed");
    let loaded_site =
        ProfileLoader::load_site_from_file(&site_path).expect("file loading site must succeed");

    assert_eq!(loaded_jur, jur);
    assert_eq!(loaded_site, site);

    let layered = ProfileLoader::resolve_layered_profile(&loaded_jur, &loaded_site, None)
        .expect("layering from loaded files must succeed");
    assert_eq!(layered.facility_name, site.name);
    assert_eq!(layered.site_type, SiteType::RegionalHospital);
}
