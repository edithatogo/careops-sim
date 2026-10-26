//! Provenance records for jurisdiction and site profiles.

use serde::{Deserialize, Serialize};

/// Provenance metadata documenting the authority and derivation of a profile.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileProvenance {
    pub source_authority: String,
    pub citation: String,
    pub source_sha256: Option<String>,
    pub author: String,
    pub notes: String,
}

impl Default for ProfileProvenance {
    fn default() -> Self {
        Self {
            source_authority: "CareOps Standard Clinical Defaults".to_string(),
            citation: "Australasian College for Emergency Medicine (ACEM) Policy P06".to_string(),
            source_sha256: None,
            author: "careops-core".to_string(),
            notes: "Synthetic baseline parameters for software verification.".to_string(),
        }
    }
}
