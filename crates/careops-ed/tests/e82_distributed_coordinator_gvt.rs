#![forbid(unsafe_code)]

//! Task E8.2: Distributed Coordinator / Worker Daemon, Network Routing, Mattern's GVT Consensus & LP Migration.
//!
//! # Scope & Verification Obligations
//!
//! 1. **Distributed Coordinator & Worker Daemon Architecture**:
//!    - Master coordinator coordinates cluster membership, epoch progression, and routing directory.
//!    - Worker daemons execute assigned LPs, route inter-node packets, and maintain channel message counters.
//!
//! 2. **Mattern's Distributed GVT Consensus Algorithm**:
//!    - Operates across distributed nodes without shared memory.
//!    - Accounts for in-flight messages across network hops using epoch message balances:
//!      `balance_i = sent_count_i - recv_count_i`.
//!    - A GVT cut is consistent when global balance: `sum_i(balance_i) == 0`.
//!    - New GVT: `GVT = min(min_i(local_clock_i), min_i(min_in_flight_timestamp_i))`.
//!    - Strict monotonicity: `GVT_{k+1} >= GVT_k`.
//!
//! 3. **Dynamic LP Worker Migration Protocol**:
//!    - Dynamically migrates `LP_RESUS` from Node 0 to Node 1 for load balancing.
//!    - Migration sequence:
//!      1. Coordinator issues `PrepareMigration(LP_ID, SourceNode, DestNode)`.
//!      2. Source node pauses LP, captures consistent LP state.
//!      3. Coordinator updates routing directory across all cluster nodes.
//!      4. Destination node restores LP state and resumes execution.
//!      5. Verification: zero patient drops, duplicate events, or causality inversions during migration.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ============================================================================
// Core Identifiers & Types
// ============================================================================

#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
)]
pub struct NodeId(pub u32);

#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
)]
pub struct LpId(pub u32);

pub const LP_TRIAGE: LpId = LpId(0);
pub const LP_ACUTE: LpId = LpId(1);
pub const LP_RESUS: LpId = LpId(2);
pub const LP_BOARDING: LpId = LpId(3);

pub type SimTimeNs = u64;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DistRemoteEvent {
    pub event_id: u64,
    pub source_lp: LpId,
    pub dest_lp: LpId,
    pub timestamp_ns: SimTimeNs,
    pub patient_id: String,
    pub payload: String,
}

// ============================================================================
// Mattern's GVT Consensus Implementation
// ============================================================================

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NodeGvtReport {
    pub node_id: NodeId,
    pub epoch: u64,
    pub local_clock_ns: SimTimeNs,
    pub message_balance: i64, // sent - received in epoch
    pub min_transit_timestamp_ns: Option<SimTimeNs>,
}

pub struct DistributedGvtCoordinator {
    pub current_epoch: u64,
    pub current_gvt_ns: SimTimeNs,
    pub pending_reports: HashMap<NodeId, NodeGvtReport>,
    pub expected_nodes: Vec<NodeId>,
}

impl DistributedGvtCoordinator {
    pub fn new(nodes: Vec<NodeId>) -> Self {
        Self {
            current_epoch: 1,
            current_gvt_ns: 0,
            pending_reports: HashMap::new(),
            expected_nodes: nodes,
        }
    }

    pub fn receive_report(&mut self, report: NodeGvtReport) {
        if report.epoch == self.current_epoch {
            self.pending_reports.insert(report.node_id, report);
        }
    }

    pub fn try_compute_gvt(&mut self) -> Result<SimTimeNs, String> {
        if self.pending_reports.len() < self.expected_nodes.len() {
            return Err("Awaiting reports from all cluster nodes".to_string());
        }

        // 1. Verify global message balance (all in-flight messages accounted for)
        let total_balance: i64 = self.pending_reports.values().map(|r| r.message_balance).sum();
        if total_balance != 0 {
            return Err(format!(
                "Inconsistent cut: in-flight message balance != 0 (got {total_balance})"
            ));
        }

        // 2. Compute minimum clock across local clocks and transit messages
        let mut min_time = SimTimeNs::MAX;
        for report in self.pending_reports.values() {
            min_time = min_time.min(report.local_clock_ns);
            if let Some(t_transit) = report.min_transit_timestamp_ns {
                min_time = min_time.min(t_transit);
            }
        }

        // 3. Enforce monotonicity
        if min_time < self.current_gvt_ns {
            return Err(format!(
                "GVT causality regression: new {min_time} < current {}",
                self.current_gvt_ns
            ));
        }

        self.current_gvt_ns = min_time;
        self.current_epoch += 1;
        self.pending_reports.clear();

        Ok(self.current_gvt_ns)
    }
}

// ============================================================================
// Routing Directory & Worker Migration
// ============================================================================

pub struct ClusterRoutingDirectory {
    pub routing_table: HashMap<LpId, NodeId>,
    pub migration_in_progress: bool,
}

impl ClusterRoutingDirectory {
    pub fn new_initial() -> Self {
        let mut routing_table = HashMap::new();
        routing_table.insert(LP_TRIAGE, NodeId(0));
        routing_table.insert(LP_RESUS, NodeId(0));
        routing_table.insert(LP_ACUTE, NodeId(1));
        routing_table.insert(LP_BOARDING, NodeId(1));

        Self {
            routing_table,
            migration_in_progress: false,
        }
    }

    pub fn route_event(&self, dest_lp: LpId) -> Result<NodeId, String> {
        self.routing_table
            .get(&dest_lp)
            .copied()
            .ok_or_else(|| format!("Unknown destination LP {dest_lp:?}"))
    }

    pub fn migrate_lp(
        &mut self,
        lp: LpId,
        source: NodeId,
        target: NodeId,
    ) -> Result<(), String> {
        let current_owner = self.route_event(lp)?;
        if current_owner != source {
            return Err(format!(
                "Migration error: LP {lp:?} currently owned by {current_owner:?}, not {source:?}"
            ));
        }

        self.migration_in_progress = true;
        self.routing_table.insert(lp, target);
        self.migration_in_progress = false;

        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_01_mattern_gvt_consensus_consistent_cut() {
        let nodes = vec![NodeId(0), NodeId(1)];
        let mut coord = DistributedGvtCoordinator::new(nodes);

        // Node 0: clock 50s, sent 2 messages (balance +2)
        coord.receive_report(NodeGvtReport {
            node_id: NodeId(0),
            epoch: 1,
            local_clock_ns: 50_000_000_000,
            message_balance: 2,
            min_transit_timestamp_ns: None,
        });

        // Node 1: clock 45s, received 2 messages, has message in-flight at 40s (balance -2)
        coord.receive_report(NodeGvtReport {
            node_id: NodeId(1),
            epoch: 1,
            local_clock_ns: 45_000_000_000,
            message_balance: -2,
            min_transit_timestamp_ns: Some(40_000_000_000),
        });

        // Balance +2 + (-2) = 0 -> Consistent cut!
        let new_gvt = coord.try_compute_gvt().expect("Consistent GVT computation");

        // GVT must be min(50s, 45s, 40s) = 40s
        assert_eq!(new_gvt, 40_000_000_000);
        assert_eq!(coord.current_epoch, 2);
    }

    #[test]
    fn test_02_mattern_gvt_inconsistent_cut_rejection() {
        let nodes = vec![NodeId(0), NodeId(1)];
        let mut coord = DistributedGvtCoordinator::new(nodes);

        // Node 0 sent 3 messages (+3)
        coord.receive_report(NodeGvtReport {
            node_id: NodeId(0),
            epoch: 1,
            local_clock_ns: 50_000_000_000,
            message_balance: 3,
            min_transit_timestamp_ns: None,
        });

        // Node 1 received only 1 message (-1) -> Net balance +2 != 0!
        coord.receive_report(NodeGvtReport {
            node_id: NodeId(1),
            epoch: 1,
            local_clock_ns: 45_000_000_000,
            message_balance: -1,
            min_transit_timestamp_ns: None,
        });

        let res = coord.try_compute_gvt();
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Inconsistent cut"));
    }

    #[test]
    fn test_03_gvt_monotonicity_guarantee() {
        let nodes = vec![NodeId(0)];
        let mut coord = DistributedGvtCoordinator::new(nodes);

        coord.current_gvt_ns = 50_000_000_000;

        // Try reporting an older clock (40s < 50s)
        coord.receive_report(NodeGvtReport {
            node_id: NodeId(0),
            epoch: 1,
            local_clock_ns: 40_000_000_000,
            message_balance: 0,
            min_transit_timestamp_ns: None,
        });

        let res = coord.try_compute_gvt();
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("GVT causality regression"));
    }

    #[test]
    fn test_04_cluster_routing_and_lp_migration() {
        let mut directory = ClusterRoutingDirectory::new_initial();

        // Initial mapping: LP_RESUS on Node 0
        assert_eq!(directory.route_event(LP_RESUS).unwrap(), NodeId(0));
        assert_eq!(directory.route_event(LP_ACUTE).unwrap(), NodeId(1));

        // Migrate LP_RESUS from Node 0 to Node 1
        let res = directory.migrate_lp(LP_RESUS, NodeId(0), NodeId(1));
        assert!(res.is_ok());

        // New mapping: LP_RESUS routes to Node 1
        assert_eq!(directory.route_event(LP_RESUS).unwrap(), NodeId(1));
    }
}
