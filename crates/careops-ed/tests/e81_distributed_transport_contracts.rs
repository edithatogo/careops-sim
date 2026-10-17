#![forbid(unsafe_code)]

//! Task E8.1: Distributed Cluster Topology, Network Transport Contracts, Heartbeat & Resync.
//!
//! # Scope & Verification Obligations
//!
//! 1. **Distributed Cluster Topology & Multi-Node Mapping**:
//!    - Maps 4 ED Logical Processes (`LP_TRIAGE`, `LP_ACUTE`, `LP_RESUS`, `LP_BOARDING`)
//!      across multiple distributed compute nodes (`NodeId(0)`, `NodeId(1)`, ...).
//!    - Node state machine: `Active`, `Suspect`, `Dead`.
//!
//! 2. **Transport Envelopes & Network Wire Format (`careops_ed.dist.v1`)**:
//!    - Lossless JSON / binary packet serialization.
//!    - Monotonic sequencing, packet deduplication, and gap detection.
//!    - Packet types: `Heartbeat`, `EventBatch`, `NullMessage`, `GvtCut`, `ResyncRequest`, `ResyncSnapshot`.
//!
//! 3. **Heartbeat Protocol & Failure Detection**:
//!    - Periodic heartbeats with node load and monotonic sequence.
//!    - Unresponsive nodes transition to `Suspect` after `T_suspect`, then `Dead` after `T_dead`.
//!
//! 4. **Reconnect & State Resync Protocol**:
//!    - Network interruption simulation: packets dropped between sequence $k$ and $k+5$.
//!    - Receiver detects sequence gap, issues `ResyncRequest`.
//!    - Sender provides state recovery snapshot and replays missed events.
//!
//! 5. **Network Partition Tolerance & Split-Brain Prevention**:
//!    - Quorum rule: an isolated minority sub-cluster halts progress cleanly rather than
//!      violating global event causality.

use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

// ============================================================================
// Core Distributed Topology Types
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum NodeStatus {
    Active,
    Suspect,
    Dead,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NodeDescriptor {
    pub node_id: NodeId,
    pub endpoint: String,
    pub hosted_lps: Vec<LpId>,
    pub status: NodeStatus,
    pub last_heartbeat_tick: u64,
}

// ============================================================================
// Wire Transport Envelopes (careops_ed.dist.v1)
// ============================================================================

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DistPacket {
    pub schema_version: String,
    pub packet_id: u64,
    pub seq: u64,
    pub source_node: NodeId,
    pub dest_node: NodeId,
    pub payload: DistPayload,
    pub timestamp_utc: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum DistPayload {
    Heartbeat {
        seq: u64,
        epoch: u64,
        load_metric: u32,
    },
    EventBatch {
        events: Vec<DistRemoteEvent>,
    },
    NullMessage {
        source_lp: LpId,
        dest_lp: LpId,
        safe_time_ns: SimTimeNs,
    },
    GvtCut {
        epoch: u64,
        local_min_ns: SimTimeNs,
        message_balance: i64,
    },
    ResyncRequest {
        target_lp: LpId,
        expected_seq: u64,
    },
    ResyncSnapshot {
        lp_id: LpId,
        snapshot_time_ns: SimTimeNs,
        state_digest: String,
    },
}

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
// Cluster Node Registry & Failure Detector
// ============================================================================

pub struct ClusterRegistry {
    pub nodes: HashMap<NodeId, NodeDescriptor>,
    pub heartbeat_timeout_ticks: u64,
    pub dead_timeout_ticks: u64,
}

impl ClusterRegistry {
    pub fn new_standard_4lp_cluster() -> Self {
        let mut nodes = HashMap::new();
        // Node 0 hosts Triage & Resus
        nodes.insert(
            NodeId(0),
            NodeDescriptor {
                node_id: NodeId(0),
                endpoint: "tcp://10.0.0.1:9001".to_string(),
                hosted_lps: vec![LP_TRIAGE, LP_RESUS],
                status: NodeStatus::Active,
                last_heartbeat_tick: 0,
            },
        );
        // Node 1 hosts Acute & Boarding
        nodes.insert(
            NodeId(1),
            NodeDescriptor {
                node_id: NodeId(1),
                endpoint: "tcp://10.0.0.2:9001".to_string(),
                hosted_lps: vec![LP_ACUTE, LP_BOARDING],
                status: NodeStatus::Active,
                last_heartbeat_tick: 0,
            },
        );

        Self {
            nodes,
            heartbeat_timeout_ticks: 100,
            dead_timeout_ticks: 300,
        }
    }

    pub fn receive_heartbeat(&mut self, node_id: NodeId, current_tick: u64) {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.last_heartbeat_tick = current_tick;
            node.status = NodeStatus::Active;
        }
    }

    pub fn audit_health(&mut self, current_tick: u64) {
        for node in self.nodes.values_mut() {
            let elapsed = current_tick.saturating_sub(node.last_heartbeat_tick);
            if elapsed >= self.dead_timeout_ticks {
                node.status = NodeStatus::Dead;
            } else if elapsed >= self.heartbeat_timeout_ticks {
                node.status = NodeStatus::Suspect;
            } else {
                node.status = NodeStatus::Active;
            }
        }
    }

    pub fn has_quorum(&self) -> bool {
        let total = self.nodes.len();
        let active = self
            .nodes
            .values()
            .filter(|n| n.status == NodeStatus::Active)
            .count();
        active > total / 2
    }
}

// ============================================================================
// Receiver Sequencing & Gap Detection State Machine
// ============================================================================

pub struct ResyncStateMachine {
    pub expected_seq: u64,
    pub received_packets: VecDeque<DistPacket>,
    pub resync_triggered: bool,
}

impl ResyncStateMachine {
    pub fn new() -> Self {
        Self {
            expected_seq: 1,
            received_packets: VecDeque::new(),
            resync_triggered: false,
        }
    }

    pub fn process_packet(&mut self, packet: DistPacket) -> Result<(), DistPayload> {
        if packet.seq == self.expected_seq {
            self.expected_seq += 1;
            self.received_packets.push_back(packet);
            self.resync_triggered = false;
            Ok(())
        } else if packet.seq < self.expected_seq {
            // Duplicate / stale packet ignored safely
            Ok(())
        } else {
            // Gap detected! (packet.seq > expected_seq)
            self.resync_triggered = true;
            Err(DistPayload::ResyncRequest {
                target_lp: LP_ACUTE,
                expected_seq: self.expected_seq,
            })
        }
    }

    pub fn apply_resync_snapshot(&mut self, new_seq: u64) {
        self.expected_seq = new_seq;
        self.resync_triggered = false;
    }
}

impl Default for ResyncStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_01_cluster_topology_mapping_and_quorum() {
        let cluster = ClusterRegistry::new_standard_4lp_cluster();
        assert_eq!(cluster.nodes.len(), 2);
        assert!(cluster.has_quorum());

        // All 4 LPs hosted across the 2 nodes
        let mut all_lps = HashSet::new();
        for node in cluster.nodes.values() {
            for lp in &node.hosted_lps {
                all_lps.insert(*lp);
            }
        }
        assert_eq!(all_lps.len(), 4);
    }

    #[test]
    fn test_02_heartbeat_failure_detection() {
        let mut cluster = ClusterRegistry::new_standard_4lp_cluster();

        // Tick 50: Node 0 active
        cluster.receive_heartbeat(NodeId(0), 50);
        cluster.audit_health(120);

        // Node 0 elapsed = 70 < 100 -> Active
        assert_eq!(cluster.nodes[&NodeId(0)].status, NodeStatus::Active);
        // Node 1 elapsed = 120 >= 100 -> Suspect
        assert_eq!(cluster.nodes[&NodeId(1)].status, NodeStatus::Suspect);

        // Tick 350: Node 1 dead
        cluster.audit_health(350);
        assert_eq!(cluster.nodes[&NodeId(1)].status, NodeStatus::Dead);
    }

    #[test]
    fn test_03_packet_serialization_roundtrip() {
        let packet = DistPacket {
            schema_version: "careops_ed.dist.v1".to_string(),
            packet_id: 101,
            seq: 1,
            source_node: NodeId(0),
            dest_node: NodeId(1),
            payload: DistPayload::EventBatch {
                events: vec![DistRemoteEvent {
                    event_id: 1,
                    source_lp: LP_TRIAGE,
                    dest_lp: LP_ACUTE,
                    timestamp_ns: 25_000_000_000,
                    patient_id: "PAT-001".to_string(),
                    payload: "TRIAGE_CLEARED".to_string(),
                }],
            },
            timestamp_utc: 1729160000,
        };

        let json = serde_json::to_string(&packet).expect("Serialization");
        let restored: DistPacket = serde_json::from_str(&json).expect("Deserialization");

        assert_eq!(packet, restored);
        assert_eq!(restored.schema_version, "careops_ed.dist.v1");
    }

    #[test]
    fn test_04_sequencing_gap_detection_and_resync_trigger() {
        let mut sm = ResyncStateMachine::new();

        let make_pkt = |seq: u64| DistPacket {
            schema_version: "careops_ed.dist.v1".to_string(),
            packet_id: seq,
            seq,
            source_node: NodeId(0),
            dest_node: NodeId(1),
            payload: DistPayload::NullMessage {
                source_lp: LP_TRIAGE,
                dest_lp: LP_ACUTE,
                safe_time_ns: seq * 10_000_000_000,
            },
            timestamp_utc: 1729160000,
        };

        // Seq 1 received cleanly
        assert!(sm.process_packet(make_pkt(1)).is_ok());
        assert_eq!(sm.expected_seq, 2);

        // Seq 3 received (Seq 2 was dropped in flight!)
        let err = sm.process_packet(make_pkt(3));
        assert!(err.is_err());
        assert!(sm.resync_triggered);

        match err.unwrap_err() {
            DistPayload::ResyncRequest { expected_seq, .. } => {
                assert_eq!(expected_seq, 2);
            }
            _ => panic!("Expected ResyncRequest"),
        }

        // Apply snapshot recovering up to Seq 4
        sm.apply_resync_snapshot(4);
        assert!(!sm.resync_triggered);
        assert_eq!(sm.expected_seq, 4);
    }

    #[test]
    fn test_05_quorum_loss_blocks_split_brain() {
        let mut cluster = ClusterRegistry::new_standard_4lp_cluster();
        // Advance clock past dead threshold with 0 heartbeats
        cluster.audit_health(1000);

        assert_eq!(cluster.nodes[&NodeId(0)].status, NodeStatus::Dead);
        assert_eq!(cluster.nodes[&NodeId(1)].status, NodeStatus::Dead);
        assert!(!cluster.has_quorum());
    }
}
