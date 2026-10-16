#!/usr/bin/env python3
"""Unit tests for cross-platform simulation reproducibility verification (Task D4.3)."""

from __future__ import annotations

import copy
import unittest
from typing import Any

from tools.checkpoint_migration import (
    CLAIMED_PLATFORMS,
    UNSUPPORTED_PLATFORMS,
    UNSUPPORTED_FEATURES,
    PlatformReproducibilityError,
    generate_platform_reproducibility_report,
    verify_cross_platform_reproducibility,
)


def make_mock_platform_run(
    platform: str,
    drift: float = 0.0,
    int_drift: int = 0,
    features: list[str] | None = None,
) -> dict[str, Any]:
    """Constructs a deterministic mock simulation run document."""
    return {
        "platform": platform,
        "schema_version": 1,
        "batch_id": "batch-repro-20261016",
        "scenario_id": "nominal-ed-mvp",
        "status": "complete",
        "num_workers": 2,
        "total_replications": 4,
        "discrete_tick_totals": 4000,
        "patient_completions": 440,
        "event_counts": 1280,
        "seed_sequence": [42, 43, 44, 45],
        "rng_draws": [10293847, 56473829, 99887766, 12345678],
        "features": features if features is not None else ["deterministic_cpu_sim"],
        "replications": [
            {
                "replication_index": 0,
                "seed": 42,
                "arrivals": 120 + int_drift,
                "started": 115,
                "completed": 110,
                "unfinished": 5,
                "mean_wait_ticks": 45.200000000000 + drift,
                "horizon_ticks": 1000,
            },
            {
                "replication_index": 1,
                "seed": 43,
                "arrivals": 118,
                "started": 112,
                "completed": 108,
                "unfinished": 4,
                "mean_wait_ticks": 48.700000000000 + drift,
                "horizon_ticks": 1000,
            },
            {
                "replication_index": 2,
                "seed": 44,
                "arrivals": 125,
                "started": 120,
                "completed": 116,
                "unfinished": 4,
                "mean_wait_ticks": 42.100000000000 + drift,
                "horizon_ticks": 1000,
            },
            {
                "replication_index": 3,
                "seed": 45,
                "arrivals": 112,
                "started": 108,
                "completed": 106,
                "unfinished": 2,
                "mean_wait_ticks": 50.000000000000 + drift,
                "horizon_ticks": 1000,
            },
        ],
        "aggregate_mean_wait": 46.500000000000 + drift,
        "aggregate_std_error": 1.250000000000 + drift,
        "mean_completion_rate": 0.925000000000 + drift,
        "completed_at_epoch_ms": 1729080005000,
    }


class TestPlatformReproducibility(unittest.TestCase):
    """Test suite for cross-platform bit-for-bit integer & bounded float reproducibility."""

    def test_cross_platform_bit_for_bit_integer_reproducibility(self) -> None:
        """Verifies Linux x86_64 vs macOS aarch64 integer traces match identically."""
        ref_run = make_mock_platform_run("x86_64-unknown-linux-gnu")
        target_run = make_mock_platform_run("aarch64-apple-darwin")

        result = verify_cross_platform_reproducibility(ref_run, target_run, tolerance=1e-12)

        self.assertTrue(result["verified"])
        self.assertEqual(result["status"], "PASS")
        self.assertEqual(result["reference_platform"], "x86_64-unknown-linux-gnu")
        self.assertEqual(result["target_platform"], "aarch64-apple-darwin")
        self.assertEqual(result["replications_compared"], 4)
        self.assertEqual(result["max_observed_drift"], 0.0)

    def test_cross_platform_floating_point_tolerance(self) -> None:
        """Verifies float metrics within tolerance <= 1e-12 pass and report is generated."""
        tiny_drift = 5e-14  # Within 1e-12 tolerance
        ref_run = make_mock_platform_run("x86_64-unknown-linux-gnu")
        target_run = make_mock_platform_run("aarch64-apple-darwin", drift=tiny_drift)

        metrics = verify_cross_platform_reproducibility(ref_run, target_run, tolerance=1e-12)
        self.assertTrue(metrics["verified"])
        self.assertLessEqual(metrics["max_observed_drift"], 1e-12)

        report = generate_platform_reproducibility_report(
            ref_run["platform"], target_run["platform"], metrics
        )
        self.assertEqual(report["reproducibility_status"], "REPRODUCIBLE")
        self.assertEqual(report["integer_divergence_count"], 0)
        self.assertIn("report_id", report)
        self.assertIn("governance_decision", report)

    def test_rejection_of_platform_drift_exceeding_tolerance(self) -> None:
        """Asserts PlatformReproducibilityError on divergent float or integer metrics."""
        ref_run = make_mock_platform_run("x86_64-unknown-linux-gnu")

        # 1. Float drift exceeding tolerance (1e-8 > 1e-12)
        drift_run = make_mock_platform_run("aarch64-apple-darwin", drift=1e-8)
        with self.assertRaises(PlatformReproducibilityError) as ctx:
            verify_cross_platform_reproducibility(ref_run, drift_run, tolerance=1e-12)
        self.assertIn("drift", str(ctx.exception).lower())

        # 2. Integer discrepancy in replication arrivals
        int_mismatch_run = make_mock_platform_run("aarch64-apple-darwin", int_drift=1)
        with self.assertRaises(PlatformReproducibilityError) as ctx:
            verify_cross_platform_reproducibility(ref_run, int_mismatch_run, tolerance=1e-12)
        self.assertIn("integer mismatch", str(ctx.exception).lower())

        # 3. Top-level event count discrepancy
        top_int_mismatch_run = copy.deepcopy(ref_run)
        top_int_mismatch_run["platform"] = "aarch64-apple-darwin"
        top_int_mismatch_run["event_counts"] = 9999
        with self.assertRaises(PlatformReproducibilityError) as ctx:
            verify_cross_platform_reproducibility(ref_run, top_int_mismatch_run, tolerance=1e-12)
        self.assertIn("top-level integer mismatch", str(ctx.exception).lower())

    def test_unsupported_platform_or_feature_detection(self) -> None:
        """Asserts rejection when unverified platform or unverified feature is encountered."""
        ref_run = make_mock_platform_run("x86_64-unknown-linux-gnu")

        # 1. Unsupported platforms
        for unsupp_plat in UNSUPPORTED_PLATFORMS:
            unsupp_run = make_mock_platform_run(unsupp_plat)
            with self.assertRaises(PlatformReproducibilityError) as ctx:
                verify_cross_platform_reproducibility(ref_run, unsupp_run, tolerance=1e-12)
            self.assertIn("not among claimed platforms", str(ctx.exception))

        # 2. Unsupported features (e.g. GPU, Metal, distributed)
        for bad_feat in ("gpu_acceleration", "metal_acceleration", "distributed_execution"):
            bad_feat_run = make_mock_platform_run("aarch64-apple-darwin", features=[bad_feat])
            with self.assertRaises(PlatformReproducibilityError) as ctx:
                verify_cross_platform_reproducibility(ref_run, bad_feat_run, tolerance=1e-12)
            self.assertIn("unsupported feature detected", str(ctx.exception).lower())


if __name__ == "__main__":
    unittest.main()
