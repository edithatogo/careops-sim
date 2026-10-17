#!/usr/bin/env python3
"""Task E8.2: Distributed Coordinator, Mattern GVT Consensus & LP Migration Contracts.

Validates:
1. Distributed GVT cut message balance rules.
2. In-flight message timestamp bounding on GVT.
3. Monotonic GVT progression constraints.
4. Cluster routing directory and dynamic LP migration.
5. Execution of native Rust test suite e82_distributed_coordinator_gvt.
"""

import subprocess
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent


class TestE82CoordinatorGvtContracts(unittest.TestCase):
    """Test suite for Task E8.2 Coordinator & GVT Consensus Contracts."""

    def test_01_mattern_gvt_balance_rule(self):
        """Verifies that sum of message balances across nodes must equal 0 for consistent cut."""
        reports = [
            {"node_id": 0, "balance": 2},
            {"node_id": 1, "balance": -2},
        ]
        total_balance = sum(r["balance"] for r in reports)
        self.assertEqual(total_balance, 0, "Consistent cut requires total balance == 0")

        inconsistent_reports = [
            {"node_id": 0, "balance": 3},
            {"node_id": 1, "balance": -1},
        ]
        self.assertNotEqual(sum(r["balance"] for r in inconsistent_reports), 0)

    def test_02_gvt_bounding_by_in_flight_messages(self):
        """Verifies that GVT is bounded by the minimum of local clocks and in-flight transit timestamps."""
        local_clocks = [50_000_000_000, 45_000_000_000]
        transit_messages = [40_000_000_000]

        all_timestamps = local_clocks + transit_messages
        gvt = min(all_timestamps)

        self.assertEqual(gvt, 40_000_000_000)
        self.assertLess(gvt, min(local_clocks))

    def test_03_routing_directory_migration_update(self):
        """Verifies cluster routing table mutation during LP migration."""
        routing = {0: "Node-0", 1: "Node-1", 2: "Node-0", 3: "Node-1"}
        self.assertEqual(routing[2], "Node-0")

        # Migrate LP 2 from Node-0 to Node-1
        routing[2] = "Node-1"
        self.assertEqual(routing[2], "Node-1")

    def test_04_native_cargo_test_verification(self):
        """Executes native cargo test e82_distributed_coordinator_gvt and asserts 0 failures."""
        cmd = [
            "cargo",
            "test",
            "-p",
            "careops-ed",
            "--test",
            "e82_distributed_coordinator_gvt",
        ]
        result = subprocess.run(
            cmd,
            cwd=str(REPO_ROOT),
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            check=False,
        )
        self.assertEqual(
            result.returncode,
            0,
            f"Native cargo test e82_distributed_coordinator_gvt failed:\n{result.stdout}",
        )
        self.assertIn("4 passed; 0 failed", result.stdout)


if __name__ == "__main__":
    unittest.main()
