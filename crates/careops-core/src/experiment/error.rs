//! Structured error types for experiment lifecycle and execution.

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Categorical kinds of experiment errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExperimentErrorKind {
    UsageError,
    FileNotFound,
    MalformedConfig,
    IncompatibleScenario,
    FileAlreadyExists,
    OutputWriteFailed,
    ExecutionFailed,
    Interrupted,
}

impl Display for ExperimentErrorKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::UsageError => "USAGE_ERROR",
            Self::FileNotFound => "FILE_NOT_FOUND",
            Self::MalformedConfig => "MALFORMED_CONFIG",
            Self::IncompatibleScenario => "INCOMPATIBLE_SCENARIO",
            Self::FileAlreadyExists => "FILE_ALREADY_EXISTS",
            Self::OutputWriteFailed => "OUTPUT_WRITE_FAILED",
            Self::ExecutionFailed => "EXECUTION_FAILED",
            Self::Interrupted => "INTERRUPTED",
        };
        formatter.write_str(name)
    }
}

/// Structured error returned by experiment runner operations.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExperimentError {
    pub kind: ExperimentErrorKind,
    pub message: String,
}

impl ExperimentError {
    pub fn new(kind: ExperimentErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl Display for ExperimentError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "[{}] {}", self.kind, self.message)
    }
}

impl std::error::Error for ExperimentError {}
