//! Jurisdiction, facility site profiles, and layered configuration loader (CC-05, CC-06).

pub mod error;
pub mod jurisdiction;
pub mod layered;
pub mod provenance;
pub mod site;

pub use error::{ProfileError, ProfileErrorKind};
pub use jurisdiction::{
    AcuityTarget, JurisdictionPolicies, JurisdictionProfile, TargetClockConfig, UrgencyScheme,
};
pub use layered::{LayeredProfile, ProfileLoader, RuntimeOverrides};
pub use provenance::ProfileProvenance;
pub use site::{SiteCapacityProfile, SiteProfile, SiteType};
