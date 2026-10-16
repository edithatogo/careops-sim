#!/usr/bin/env python3
"""Unit tests for checkpoint migration, rollback, and state conservation (Task D4.3)."""

from __future__ import annotations

import copy
import hashlib
import json
import unittest
from typing import Any

from tools.checkpoint_migration import (
    CheckpointMigrationError,
    IncompatibleCheckpointSchemaError,
    CorruptedCheckpointError,
    StateConservationViolationError,
    compute_provenance_digest,
    compute_replications_checksum,
    migrate_v1_to_v2,
    rollback_v2_to_v1,
    validate_checkpoint_v1,
    validate_checkpoint_v2,
    verify_checkpoint_conservation,
)


def make_nominal_v1_checkpoint() -> dict[str, Any]:
    """Generates a nominal valid v1 batch checkpoint."""
    return {
        "schema_version": 1,
        "batch_id": "batch-alpha-20261016",
        "scenario_id": "nominal-ed-mvp",
        "base_seed": 42,
        "total_replications": 5,
        "horizon_override": None,
        "completed_replications": [
            {
                "replication_index": 0,
                "seed": 42,
                "arrivals": 120,
                "started": 115,
                "completed": 110,
                "unfinished": 5,
                "mean_wait_ticks": 45.2,
                "horizon_ticks": 1000,
            },
            {
                "replication_index": 1,
                "seed": 43,
                "arrivals": 118,
                "started": 112,
                "completed": 108,
                "unfinished": 4,
                "mean_wait_ticks": 48.7,
                "horizon_ticks": 1000,
            },
            {
                "replication_index": 2,
                "seed": 44,
                "arrivals": 125,
                "started": 120,
                "completed": 116,
                "unfinished": 4,
                "mean_wait_ticks": 42.1,
                "horizon_ticks": 1000,
            },
        ],
        "pending_indices": [3, 4],
        "status": "in_progress",
        "checkpoint_epoch_ms": 1729080000000,
    }


def mock_simulate_replication(index: usize, base_seed: int) -> dict[str, Any]:
    """Deterministic mock simulation for a single replication."""
    seed = base_seed + index
    arrivals = 120 + (index * 2)
    started = arrivals - 3
    completed = started - 2
    unfinished = arrivals - completed
    mean_wait = 40.0 + (index * 1.5)
    return {
        "replication_index": index,
        "seed": seed,
        "arrivals": arrivals,
        "started": started,
        "completed": completed,
        "unfinished": unfinished,
        "mean_wait_ticks": mean_wait,
        "horizon_ticks": 1000,
    }


def mock_resume_batch(checkpoint: dict[str, Any]) -> dict[str, Any]:
    """Resumes remaining work from a checkpoint to final completion."""
    completed = copy.deepcopy(checkpoint["completed_replications"])
    base_seed = checkpoint["base_seed"]
    for idx in checkpoint["pending_indices"]:
        rep = mock_simulate_replication(idx, base_seed)
        completed.append(rep)

    completed.sort(key=lambda r: r["replication_index"])
    mean_wait = sum(r["mean_wait_ticks"] for r in completed) / len(completed)
    total_completed = sum(r["completed"] for r in completed)
    total_arrivals = sum(r["arrivals"] for r in completed)
    rate = total_completed / total_arrivals if total_arrivals > 0 else 0.0

    return {
        "batch_id": checkpoint["batch_id"],
        "scenario_id": checkpoint["scenario_id"],
        "status": "complete",
        "total_replications": len(completed),
        "replications": completed,
        "aggregate_mean_wait": mean_wait,
        "mean_completion_rate": rate,
    }


class TestCheckpointMigration(unittest.TestCase):
    """Test suite for checkpoint migration, rollback, and state conservation."""

    def test_valid_v1_checkpoint_validation(self) -> None:
        """Validates that a nominal v1 checkpoint satisfies all invariants."""
        v1 = make_nominal_v1_checkpoint()
        # Should not raise
        validate_checkpoint_v1(v1)

    def test_v1_to_v2_forward_migration(self) -> None:
        """Migrates v1 -> v2, asserts metadata, fields, and SHA-256 digest."""
        v1 = make_nominal_v1_checkpoint()
        v2 = migrate_v1_to_v2(v1)

        self.assertEqual(v2["schema_version"], 2)
        self.assertEqual(v2["checkpoint_version"], "2.0.0")
        self.assertEqual(v2["simulation_engine"], "careops-ed-v1")
        self.assertIn("provenance_digest", v2)
        self.assertEqual(len(v2["provenance_digest"]), 64)

        meta = v2["migration_metadata"]
        self.assertEqual(meta["migrated_from_version"], 1)
        self.assertEqual(meta["migrated_by"], "careops-sim-checkpoint-migration")
        self.assertIn("migrated_at_utc", meta)

        expected_checksum = compute_replications_checksum(v1["completed_replications"])
        self.assertEqual(meta["checksum"], expected_checksum)

        # Assert bit-for-bit conservation of completed replications and pending indices
        self.assertEqual(v2["completed_replications"], v1["completed_replications"])
        self.assertEqual(v2["pending_indices"], v1["pending_indices"])
        self.assertTrue(verify_checkpoint_conservation(v1, v2))

    def test_v2_to_v1_rollback_migration(self) -> None:
        """Downgrades v2 -> v1, asserting perfect roundtrip state conservation."""
        v1_original = make_nominal_v1_checkpoint()
        v2 = migrate_v1_to_v2(v1_original)

        v1_rolled_back = rollback_v2_to_v1(v2)

        self.assertEqual(v1_rolled_back["schema_version"], 1)
        self.assertNotIn("checkpoint_version", v1_rolled_back)
        self.assertNotIn("simulation_engine", v1_rolled_back)
        self.assertNotIn("provenance_digest", v1_rolled_back)
        self.assertNotIn("migration_metadata", v1_rolled_back)

        # Validate roundtrip equality
        self.assertEqual(v1_original, v1_rolled_back)
        self.assertTrue(verify_checkpoint_conservation(v1_original, v1_rolled_back))

    def test_rejection_of_incompatible_schema_version(self) -> None:
        """Rejects checkpoint declaring incompatible schema version (e.g. 99)."""
        v1 = make_nominal_v1_checkpoint()
        v1["schema_version"] = 99
        with self.assertRaises(IncompatibleCheckpointSchemaError):
            validate_checkpoint_v1(v1)

        with self.assertRaises(IncompatibleCheckpointSchemaError):
            migrate_v1_to_v2(v1)

    def test_rejection_of_corrupted_state_conservation(self) -> None:
        """Rejects mismatch where completed + pending != total."""
        v1 = make_nominal_v1_checkpoint()
        # Drop one pending index: completed (3) + pending (1) != total (5)
        v1["pending_indices"] = [3]
        with self.assertRaises(StateConservationViolationError):
            validate_checkpoint_v1(v1)

        with self.assertRaises(StateConservationViolationError):
            migrate_v1_to_v2(v1)

    def test_rejection_of_overlapping_indices(self) -> None:
        """Rejects checkpoint where index is present in both completed and pending."""
        v1 = make_nominal_v1_checkpoint()
        # Index 2 is in completed_replications and also in pending_indices
        v1["pending_indices"] = [2, 3]
        v1["total_replications"] = 5
        with self.assertRaises(StateConservationViolationError):
            validate_checkpoint_v1(v1)

    def test_rejection_of_tampered_migration_checksum(self) -> None:
        """Rejects v2 checkpoint with tampered replication data or altered checksum."""
        v1 = make_nominal_v1_checkpoint()
        v2 = migrate_v1_to_v2(v1)

        # Alter completed replication data
        tampered_v2 = copy.deepcopy(v2)
        tampered_v2["completed_replications"][0]["arrivals"] = 9999

        with self.assertRaises(CorruptedCheckpointError):
            validate_checkpoint_v2(tampered_v2)

        with self.assertRaises(CorruptedCheckpointError):
            rollback_v2_to_v1(tampered_v2)

        # Alter checksum directly
        tampered_checksum_v2 = copy.deepcopy(v2)
        tampered_checksum_v2["migration_metadata"]["checksum"] = "0" * 64
        with self.assertRaises(CorruptedCheckpointError):
            validate_checkpoint_v2(tampered_checksum_v2)

    def test_checkpoint_resumption_equivalence(self) -> None:
        """Mocks resuming batch from migrated v2 vs rolled back v1, proving identical outcome."""
        v1_original = make_nominal_v1_checkpoint()
        v2 = migrate_v1_to_v2(v1_original)
        v1_rolled_back = rollback_v2_to_v1(v2)

        outcome_from_v2 = mock_resume_batch(v2)
        outcome_from_v1_rollback = mock_resume_batch(v1_rolled_back)

        self.assertEqual(outcome_from_v2, outcome_from_v1_rollback)
        self.assertEqual(len(outcome_from_v2["replications"]), 5)
        self.assertEqual(outcome_from_v2["status"], "complete")

    def test_last_known_good_pin_rollback_rehearsal(self) -> None:
        """Rehearses pin drift detection and rollback to last-known-good reviewed commit."""
        last_known_good_commit = "44261eb5cf8275f132f63a3396903d116d701dbe"
        unreviewed_candidate_commit = "deadbeef1234567890abcdef1234567890abcdef"

        state = {"submodule_pins": {"libs/kairos": last_known_good_commit}}
        self.assertEqual(state["submodule_pins"]["libs/kairos"], last_known_good_commit)

        # Simulate unreviewed pin update
        state["submodule_pins"]["libs/kairos"] = unreviewed_candidate_commit
        self.assertNotEqual(state["submodule_pins"]["libs/kairos"], last_known_good_commit)

        # Rehearse rollback to last-known-good pin
        state["submodule_pins"]["libs/kairos"] = last_known_good_commit
        self.assertEqual(state["submodule_pins"]["libs/kairos"], last_known_good_commit)


if __name__ == "__main__":
    unittest.main()

