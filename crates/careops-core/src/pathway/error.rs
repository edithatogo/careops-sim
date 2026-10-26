//! Error definitions for clinical pathways, progression, and policy clocks (CC-04, CC-06).

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Categorical kinds of pathway and progression errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PathwayErrorKind {
    InvalidTransition,
    LoopBoundExceeded,
    StaleEvent,
    ZeroTargetError,
    CensoredAtHorizon,
    InvalidCovariate,
    DrawOwnershipViolation,
}

impl Display for PathwayErrorKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::InvalidTransition => "INVALID_TRANSITION",
            Self::LoopBoundExceeded => "LOOP_BOUND_EXCEEDED",
            Self::StaleEvent => "STALE_EVENT",
            Self::ZeroTargetError => "ZERO_TARGET_ERROR",
            Self::CensoredAtHorizon => "CENSORED_AT_HORIZON",
            Self::InvalidCovariate => "INVALID_COVARIATE",
            Self::DrawOwnershipViolation => "DRAW_OWNERSHIP_VIOLATION",
        };
        formatter.write_str(s)
    }
}

/// Structured error returned during pathway progression and clock evaluations.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathwayError {
    pub kind: PathwayErrorKind,
    pub message: String,
}

impl PathwayError {
    pub fn new(kind: PathwayErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl Display for PathwayError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "[{}] {}", self.kind, self.message)
    }
}

impl std::error::Error for PathwayError {}
