//! Configurable directed stage graph with strict transition validation and bounded cycles (CC-04).

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

use super::error::{PathwayError, PathwayErrorKind};

/// Unique identifier for a clinical pathway stage.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StageId(pub String);

impl StageId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

/// Node in the configurable stage graph.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StageNode {
    pub id: StageId,
    pub name: String,
    pub max_repetitions: usize,
    pub is_terminal: bool,
}

/// Directed stage graph defining valid transitions and cycle limits.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StageGraph {
    nodes: HashMap<StageId, StageNode>,
    allowed_transitions: HashMap<StageId, HashSet<StageId>>,
}

impl StageGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a stage node to the graph.
    pub fn add_node(&mut self, node: StageNode) {
        self.allowed_transitions.entry(node.id.clone()).or_default();
        self.nodes.insert(node.id.clone(), node);
    }

    /// Registers a permitted directed transition from one stage to another.
    pub fn add_transition(&mut self, from: &StageId, to: &StageId) {
        self.allowed_transitions
            .entry(from.clone())
            .or_default()
            .insert(to.clone());
    }

    /// Validates whether a transition from `from` to `to` is permitted.
    pub fn validate_transition(
        &self,
        from: &StageId,
        to: &StageId,
        current_repetition_count: usize,
    ) -> Result<(), PathwayError> {
        if !self.nodes.contains_key(from) {
            return Err(PathwayError::new(
                PathwayErrorKind::InvalidTransition,
                format!("origin stage '{}' is not registered in stage graph", from.0),
            ));
        }

        if !self.nodes.contains_key(to) {
            return Err(PathwayError::new(
                PathwayErrorKind::InvalidTransition,
                format!("destination stage '{}' is not registered in stage graph", to.0),
            ));
        }

        let allowed = self
            .allowed_transitions
            .get(from)
            .map(|set| set.contains(to))
            .unwrap_or(false);

        if !allowed {
            return Err(PathwayError::new(
                PathwayErrorKind::InvalidTransition,
                format!("transition from '{}' to '{}' is not permitted", from.0, to.0),
            ));
        }

        // Loop / repetition bound check
        if from == to {
            let node = self.nodes.get(from).unwrap();
            if current_repetition_count >= node.max_repetitions {
                return Err(PathwayError::new(
                    PathwayErrorKind::LoopBoundExceeded,
                    format!(
                        "stage '{}' repetition limit of {} exceeded (attempted {})",
                        from.0, node.max_repetitions, current_repetition_count + 1
                    ),
                ));
            }
        }

        Ok(())
    }
}
