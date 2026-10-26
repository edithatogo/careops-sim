//! Error definitions for profile loading, schema validation, and precedence resolution.

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Categorical kinds of profile errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProfileErrorKind {
    MissingSource,
    IncompatibleVersion,
    MalformedFormat,
    JurisdictionMismatch,
    PrecedenceViolation,
    ValidationFailed,
}

impl Display for ProfileErrorKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::MissingSource => "MISSING_SOURCE",
            Self::IncompatibleVersion => "INCOMPATIBLE_VERSION",
            Self::MalformedFormat => "MALFORMED_FORMAT",
            Self::JurisdictionMismatch => "JURISDICTION_MISMATCH",
            Self::PrecedenceViolation => "PRECEDENCE_VIOLATION",
            Self::ValidationFailed => "VALIDATION_FAILED",
        };
        formatter.write_str(s)
    }
}

/// Structured profile error returned during loading and layered resolution.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileError {
    pub kind: ProfileErrorKind,
    pub message: String,
}

impl ProfileError {
    pub fn new(kind: ProfileErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl Display for ProfileError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "[{}] {}", self.kind, self.message)
    }
}

impl std::error::Error for ProfileError {}
