#!/usr/bin/env python3
"""Checkpoint migration and platform reproducibility verification engine.

Task D4.3: Rehearse rollback to last-known-good pins and checkpoint migration;
verify reproducibility across claimed platforms and create release notes and
explicit unsupported-feature list. No autonomous public publication.
"""

from __future__ import annotations

import argparse
import copy
from datetime import datetime, timezone
import hashlib
import json
import math
import os
from pathlib import Path
import sys
from typing import Any
import uuid


# ---------------------------------------------------------------------------
# Constants & Configuration
# ---------------------------------------------------------------------------

CLAIMED_PLATFORMS: tuple[str, ...] = (
    "x86_64-unknown-linux-gnu",
    "aarch64-apple-darwin",
)

UNSUPPORTED_PLATFORMS: tuple[str, ...] = (
    "x86_64-pc-windows-msvc",
    "aarch64-unknown-linux-gnu",
    "wasm32-unknown-unknown",
    "wasm32-wasi",
    "i686-unknown-linux-gnu",
)

UNSUPPORTED_FEATURES: tuple[str, ...] = (
    "gpu_acceleration",
    "metal_acceleration",
    "cuda_acceleration",
    "mlx_acceleration",
    "distributed_execution",
    "multi_node_cluster",
    "within_run_parallel_backend",
    "live_arrow_ipc",
    "gui_cad_renderer",
    "real_time_web_service",
    "cairns_calibrated_profile",
)

REQUIRED_V1_FIELDS: tuple[str, ...] = (
    "schema_version",
    "batch_id",
    "scenario_id",
    "base_seed",
    "total_replications",
    "completed_replications",
    "pending_indices",
    "status",
    "checkpoint_epoch_ms",
)

REQUIRED_V2_FIELDS: tuple[str, ...] = (
    *REQUIRED_V1_FIELDS,
    "checkpoint_version",
    "simulation_engine",
    "provenance_digest",
    "migration_metadata",
)


# ---------------------------------------------------------------------------
# Exception Hierarchy
# ---------------------------------------------------------------------------

class CheckpointMigrationError(Exception):
    """Base exception for all checkpoint migration and reproducibility errors."""


class IncompatibleCheckpointSchemaError(CheckpointMigrationError):
    """Raised when checkpoint schema version or structure is incompatible."""


class CorruptedCheckpointError(CheckpointMigrationError):
    """Raised when checkpoint data is corrupted or fails integrity checks."""


class StateConservationViolationError(CorruptedCheckpointError):
    """Raised when state conservation invariants are violated during checkpoint migration."""


class PlatformReproducibilityError(CheckpointMigrationError):
    """Raised when cross-platform reproducibility checks fail or exceed tolerance."""


# ---------------------------------------------------------------------------
# Checksum and Digest Utilities
# ---------------------------------------------------------------------------

def compute_replications_checksum(completed_replications: list[dict[str, Any]]) -> str:
    """Computes deterministic canonical SHA-256 hash across completed replications."""
    canonical_json = json.dumps(completed_replications, sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(canonical_json.encode("utf-8")).hexdigest()


def compute_provenance_digest(data: dict[str, Any], checksum: str) -> str:
    """Computes canonical provenance digest from simulation metadata and replications checksum."""
    engine = data.get("simulation_engine", "careops-ed-v1")
    batch_id = data.get("batch_id", "")
    scenario_id = data.get("scenario_id", "")
    base_seed = data.get("base_seed", 0)
    total_reps = data.get("total_replications", 0)
    raw = f"{engine}:{batch_id}:{scenario_id}:{base_seed}:{total_reps}:{checksum}"
    return hashlib.sha256(raw.encode("utf-8")).hexdigest()


# ---------------------------------------------------------------------------
# Checkpoint Validation & Migration Engine
# ---------------------------------------------------------------------------

def validate_checkpoint_v1(data: dict[str, Any]) -> None:
    """Validates schema v1 BatchCheckpoint invariants.

    Raises:
        IncompatibleCheckpointSchemaError: If schema version or required fields are missing.
        CorruptedCheckpointError: If fields are malformed or state conservation is violated.
        StateConservationViolationError: If indices overlap or total counts do not conserve.
    """
    if not isinstance(data, dict):
        raise IncompatibleCheckpointSchemaError("Checkpoint must be a JSON dictionary object.")

    if "schema_version" not in data:
        raise IncompatibleCheckpointSchemaError("Missing required field 'schema_version'.")

    if data["schema_version"] != 1:
        raise IncompatibleCheckpointSchemaError(
            f"Expected schema_version 1 for v1 checkpoint, found {data['schema_version']}."
        )

    for field in REQUIRED_V1_FIELDS:
        if field not in data:
            raise IncompatibleCheckpointSchemaError(f"Missing required v1 field '{field}'.")

    # Type validation
    if not isinstance(data["batch_id"], str) or not data["batch_id"]:
        raise CorruptedCheckpointError("Field 'batch_id' must be a non-empty string.")
    if not isinstance(data["scenario_id"], str) or not data["scenario_id"]:
        raise CorruptedCheckpointError("Field 'scenario_id' must be a non-empty string.")
    if not isinstance(data["base_seed"], int) or isinstance(data["base_seed"], bool):
        raise CorruptedCheckpointError("Field 'base_seed' must be an integer.")
    if not isinstance(data["total_replications"], int) or isinstance(data["total_replications"], bool):
        raise CorruptedCheckpointError("Field 'total_replications' must be an integer.")
    if data["total_replications"] < 0:
        raise CorruptedCheckpointError("Field 'total_replications' cannot be negative.")
    if not isinstance(data["completed_replications"], list):
        raise CorruptedCheckpointError("Field 'completed_replications' must be a list.")
    if not isinstance(data["pending_indices"], list):
        raise CorruptedCheckpointError("Field 'pending_indices' must be a list.")
    if not isinstance(data["status"], str):
        raise CorruptedCheckpointError("Field 'status' must be a string.")
    if not isinstance(data["checkpoint_epoch_ms"], int) or isinstance(data["checkpoint_epoch_ms"], bool):
        raise CorruptedCheckpointError("Field 'checkpoint_epoch_ms' must be an integer.")

    completed = data["completed_replications"]
    pending = data["pending_indices"]
    total = data["total_replications"]

    # State conservation check: completed + pending == total
    if len(completed) + len(pending) != total:
        raise StateConservationViolationError(
            f"State conservation violation: completed ({len(completed)}) + "
            f"pending ({len(pending)}) != total ({total})."
        )

    # Validate replication indices
    completed_indices: list[int] = []
    for rep in completed:
        if not isinstance(rep, dict) or "replication_index" not in rep:
            raise CorruptedCheckpointError("Each completed replication must contain 'replication_index'.")
        idx = rep["replication_index"]
        if not isinstance(idx, int) or isinstance(idx, bool):
            raise CorruptedCheckpointError(f"Invalid replication_index: {idx}")
        if idx < 0 or idx >= total:
            raise CorruptedCheckpointError(
                f"Replication index {idx} out of bounds for total_replications {total}."
            )
        completed_indices.append(idx)

    # Check for duplicate completed indices
    if len(completed_indices) != len(set(completed_indices)):
        raise CorruptedCheckpointError("Duplicate replication index found in completed_replications.")

    # Validate pending indices
    for idx in pending:
        if not isinstance(idx, int) or isinstance(idx, bool):
            raise CorruptedCheckpointError(f"Invalid pending index: {idx}")
        if idx < 0 or idx >= total:
            raise CorruptedCheckpointError(
                f"Pending index {idx} out of bounds for total_replications {total}."
            )

    if len(pending) != len(set(pending)):
        raise CorruptedCheckpointError("Duplicate pending index found in pending_indices.")

    # Overlapping indices check
    overlap = set(completed_indices).intersection(set(pending))
    if overlap:
        raise StateConservationViolationError(
            f"Corrupted state: indices {sorted(overlap)} appear in both completed and pending sets."
        )


def validate_checkpoint_v2(data: dict[str, Any]) -> None:
    """Validates schema v2 BatchCheckpoint invariants including provenance and checksum."""
    if not isinstance(data, dict):
        raise IncompatibleCheckpointSchemaError("Checkpoint must be a JSON dictionary object.")

    if data.get("schema_version") != 2:
        raise IncompatibleCheckpointSchemaError(
            f"Expected schema_version 2 for v2 checkpoint, found {data.get('schema_version')}."
        )

    for field in REQUIRED_V2_FIELDS:
        if field not in data:
            raise IncompatibleCheckpointSchemaError(f"Missing required v2 field '{field}'.")

    if data.get("checkpoint_version") != "2.0.0":
        raise IncompatibleCheckpointSchemaError(
            f"Expected checkpoint_version '2.0.0', found '{data.get('checkpoint_version')}'."
        )

    if data.get("simulation_engine") != "careops-ed-v1":
        raise IncompatibleCheckpointSchemaError(
            f"Expected simulation_engine 'careops-ed-v1', found '{data.get('simulation_engine')}'."
        )

    meta = data["migration_metadata"]
    if not isinstance(meta, dict):
        raise CorruptedCheckpointError("Field 'migration_metadata' must be a dictionary.")

    for m_field in ("migrated_from_version", "migrated_at_utc", "migrated_by", "checksum"):
        if m_field not in meta:
            raise IncompatibleCheckpointSchemaError(f"Missing migration_metadata field '{m_field}'.")

    # Verify state conservation invariants
    total = data["total_replications"]
    completed = data["completed_replications"]
    pending = data["pending_indices"]

    if len(completed) + len(pending) != total:
        raise StateConservationViolationError(
            f"State conservation violation in v2: completed ({len(completed)}) + "
            f"pending ({len(pending)}) != total ({total})."
        )

    completed_indices = [r["replication_index"] for r in completed if isinstance(r, dict) and "replication_index" in r]
    if len(completed_indices) != len(completed):
        raise CorruptedCheckpointError("Invalid replication records in v2 completed_replications.")

    overlap = set(completed_indices).intersection(set(pending))
    if overlap:
        raise StateConservationViolationError(f"Overlapping completed and pending indices in v2: {sorted(overlap)}.")

    # Cryptographic integrity check: verify checksum matches completed replications
    actual_checksum = compute_replications_checksum(completed)
    expected_checksum = meta["checksum"]
    if actual_checksum != expected_checksum:
        raise CorruptedCheckpointError(
            f"Tampered migration checksum: expected '{expected_checksum}', computed '{actual_checksum}'."
        )

    # Provenance digest check
    actual_digest = compute_provenance_digest(data, expected_checksum)
    if data["provenance_digest"] != actual_digest:
        raise CorruptedCheckpointError(
            f"Tampered provenance digest: expected '{data['provenance_digest']}', computed '{actual_digest}'."
        )


def migrate_v1_to_v2(v1_data: dict[str, Any]) -> dict[str, Any]:
    """Upgrades a v1 checkpoint to v2 format with provenance digest and metadata.

    Preserves 100% of replication results bit-for-bit without data loss or reordering.
    """
    validate_checkpoint_v1(v1_data)

    v2 = copy.deepcopy(v1_data)
    v2["schema_version"] = 2
    v2["checkpoint_version"] = "2.0.0"
    v2["simulation_engine"] = "careops-ed-v1"

    checksum = compute_replications_checksum(v2["completed_replications"])
    v2["migration_metadata"] = {
        "migrated_from_version": 1,
        "migrated_at_utc": datetime.now(timezone.utc).isoformat(),
        "migrated_by": "careops-sim-checkpoint-migration",
        "checksum": checksum,
    }
    v2["provenance_digest"] = compute_provenance_digest(v2, checksum)

    # Validate constructed v2 artifact
    validate_checkpoint_v2(v2)
    return v2


def rollback_v2_to_v1(v2_data: dict[str, Any]) -> dict[str, Any]:
    """Downgrades a v2 checkpoint back to schema v1.

    Validates v2 integrity before rolling back, stripping v2 metadata while
    strictly preserving all progress and execution state.
    """
    validate_checkpoint_v2(v2_data)

    v1 = {
        "schema_version": 1,
        "batch_id": v2_data["batch_id"],
        "scenario_id": v2_data["scenario_id"],
        "base_seed": v2_data["base_seed"],
        "total_replications": v2_data["total_replications"],
        "horizon_override": v2_data.get("horizon_override"),
        "completed_replications": copy.deepcopy(v2_data["completed_replications"]),
        "pending_indices": list(v2_data["pending_indices"]),
        "status": v2_data["status"],
        "checkpoint_epoch_ms": v2_data["checkpoint_epoch_ms"],
    }

    validate_checkpoint_v1(v1)
    return v1


def verify_checkpoint_conservation(before: dict[str, Any], after: dict[str, Any]) -> bool:
    """Asserts 100% preservation of completed count, pending indices, base seed, batch ID, scenario ID.

    Raises:
        StateConservationViolationError: On any deviation between checkpoints.
    """
    for field in ("batch_id", "scenario_id", "base_seed", "total_replications"):
        val_before = before.get(field)
        val_after = after.get(field)
        if val_before != val_after:
            raise StateConservationViolationError(
                f"Field '{field}' altered during transition: before={val_before}, after={val_after}"
            )

    before_completed = before.get("completed_replications", [])
    after_completed = after.get("completed_replications", [])
    if len(before_completed) != len(after_completed):
        raise StateConservationViolationError(
            f"Completed count mismatch: before={len(before_completed)}, after={len(after_completed)}"
        )

    # Check completed replications bit-for-bit
    if before_completed != after_completed:
        raise StateConservationViolationError("Completed replications content altered during transition.")

    before_pending = before.get("pending_indices", [])
    after_pending = after.get("pending_indices", [])
    if before_pending != after_pending:
        raise StateConservationViolationError(
            f"Pending indices mismatch: before={before_pending}, after={after_pending}"
        )

    return True


# ---------------------------------------------------------------------------
# Platform Reproducibility Verification Engine
# ---------------------------------------------------------------------------

def _extract_platform(run: dict[str, Any]) -> str:
    """Extracts platform/host identifier from run document."""
    platform = run.get("platform") or run.get("host") or run.get("target_platform")
    if not platform or not isinstance(platform, str):
        raise PlatformReproducibilityError("Missing or invalid platform identifier in run document.")
    return platform


def _check_unsupported_features(run: dict[str, Any]) -> None:
    """Audits run document for unsupported or prohibited features."""
    features = run.get("features") or run.get("enabled_features") or []
    if isinstance(features, (list, tuple)):
        for feat in features:
            if feat in UNSUPPORTED_FEATURES or any(
                bad in feat.lower() for bad in ("gpu", "metal", "cuda", "mlx", "cluster", "distributed")
            ):
                raise PlatformReproducibilityError(
                    f"Unsupported feature detected: '{feat}'. Outside native-v1 CPU execution scope."
                )

    if run.get("unsupported_features_claimed"):
        raise PlatformReproducibilityError(
            f"Run document asserts unsupported feature claims: {run.get('unsupported_features_claimed')}"
        )


def verify_cross_platform_reproducibility(
    reference_run: dict[str, Any],
    target_run: dict[str, Any],
    tolerance: float = 1e-12,
) -> dict[str, Any]:
    """Compares simulation runs across claimed platforms for bit-for-bit and float tolerance compliance.

    Verifies:
      - Claimed platforms (x86_64-unknown-linux-gnu and aarch64-apple-darwin).
      - Rejection of unverified platforms or unsupported features.
      - Integer fields (event counts, patient completions, discrete tick totals, RNG draws) match 100% bit-for-bit.
      - Floating point metrics match within tolerance <= 1e-12.

    Raises:
        PlatformReproducibilityError: If platform is unsupported or drift exceeds tolerance.
    """
    ref_host = _extract_platform(reference_run)
    tgt_host = _extract_platform(target_run)

    # Platform claim verification
    if ref_host not in CLAIMED_PLATFORMS:
        raise PlatformReproducibilityError(
            f"Reference platform '{ref_host}' is not among claimed platforms: {CLAIMED_PLATFORMS}"
        )
    if tgt_host not in CLAIMED_PLATFORMS:
        raise PlatformReproducibilityError(
            f"Target platform '{tgt_host}' is not among claimed platforms: {CLAIMED_PLATFORMS}"
        )

    # Feature audit
    _check_unsupported_features(reference_run)
    _check_unsupported_features(target_run)

    # Top-level integer checks
    int_fields_to_check = (
        "total_replications",
        "num_workers",
        "discrete_tick_totals",
        "patient_completions",
        "event_counts",
    )
    for field in int_fields_to_check:
        if field in reference_run and field in target_run:
            if reference_run[field] != target_run[field]:
                raise PlatformReproducibilityError(
                    f"Top-level integer mismatch for '{field}': ref={reference_run[field]}, tgt={target_run[field]}"
                )

    # Deterministic RNG / seed sequence checks
    for rng_field in ("rng_draws", "seed_sequence", "seeds"):
        if rng_field in reference_run and rng_field in target_run:
            if reference_run[rng_field] != target_run[rng_field]:
                raise PlatformReproducibilityError(
                    f"RNG/seed sequence mismatch for '{rng_field}': bit-for-bit equivalence failed."
                )

    # Replication results check
    ref_reps = reference_run.get("replications") or reference_run.get("completed_replications", [])
    tgt_reps = target_run.get("replications") or target_run.get("completed_replications", [])

    if len(ref_reps) != len(tgt_reps):
        raise PlatformReproducibilityError(
            f"Replication count mismatch: reference has {len(ref_reps)}, target has {len(tgt_reps)}"
        )

    max_drift = 0.0
    int_fields = (
        "replication_index",
        "seed",
        "arrivals",
        "started",
        "completed",
        "unfinished",
        "horizon_ticks",
        "event_count",
    )
    float_fields = (
        "mean_wait_ticks",
        "utilization",
        "aggregate_mean_wait",
        "aggregate_std_error",
        "mean_completion_rate",
    )

    for i, (r_ref, r_tgt) in enumerate(zip(ref_reps, tgt_reps)):
        # Integer bit-for-bit check
        for f in int_fields:
            if f in r_ref and f in r_tgt:
                if r_ref[f] != r_tgt[f]:
                    raise PlatformReproducibilityError(
                        f"Replication {i} integer mismatch for '{f}': ref={r_ref[f]}, tgt={r_tgt[f]}"
                    )

        # Float tolerance check
        for f in float_fields:
            if f in r_ref and f in r_tgt:
                v_ref = float(r_ref[f])
                v_tgt = float(r_tgt[f])
                diff = abs(v_ref - v_tgt)
                if diff > max_drift:
                    max_drift = diff
                if diff > tolerance:
                    raise PlatformReproducibilityError(
                        f"Replication {i} floating-point drift for '{f}' ({diff:.2e}) exceeds tolerance {tolerance:.2e}: "
                        f"ref={v_ref}, tgt={v_tgt}"
                    )

    # Top-level aggregate float checks
    for f in float_fields:
        if f in reference_run and f in target_run:
            v_ref = float(reference_run[f])
            v_tgt = float(target_run[f])
            diff = abs(v_ref - v_tgt)
            if diff > max_drift:
                max_drift = diff
            if diff > tolerance:
                raise PlatformReproducibilityError(
                    f"Aggregate float drift for '{f}' ({diff:.2e}) exceeds tolerance {tolerance:.2e}: "
                    f"ref={v_ref}, tgt={v_tgt}"
                )

    return {
        "status": "PASS",
        "verified": True,
        "reference_platform": ref_host,
        "target_platform": tgt_host,
        "replications_compared": len(ref_reps),
        "tolerance": tolerance,
        "max_observed_drift": max_drift,
        "verified_at_utc": datetime.now(timezone.utc).isoformat(),
    }


def generate_platform_reproducibility_report(
    reference_host: str,
    target_host: str,
    metrics: dict[str, Any],
) -> dict[str, Any]:
    """Generates a structured reproducibility verification audit report."""
    return {
        "report_id": f"PRR-{uuid.uuid4().hex[:12].upper()}",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "reference_platform": reference_host,
        "target_platform": target_host,
        "claimed_platforms": list(CLAIMED_PLATFORMS),
        "reproducibility_status": "REPRODUCIBLE",
        "integer_divergence_count": 0,
        "max_floating_point_drift": metrics.get("max_observed_drift", 0.0),
        "tolerance_threshold": metrics.get("tolerance", 1e-12),
        "metrics_summary": metrics,
        "unsupported_features_audited": list(UNSUPPORTED_FEATURES),
        "governance_decision": "PASS: Bit-for-bit integer & bounded float reproducibility confirmed across claimed platforms.",
    }


# ---------------------------------------------------------------------------
# CLI Implementation
# ---------------------------------------------------------------------------

def build_parser() -> argparse.ArgumentParser:
    """Builds CLI argument parser for checkpoint migration and reproducibility."""
    parser = argparse.ArgumentParser(
        prog="checkpoint_migration.py",
        description="CareOps Sim Checkpoint Migration and Platform Reproducibility Engine",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    # Subcommand: migrate
    p_mig = subparsers.add_parser("migrate", help="Migrate v1 checkpoint to v2 format")
    p_mig.add_argument("--input", "-i", required=True, type=Path, help="Path to v1 checkpoint JSON")
    p_mig.add_argument("--output", "-o", required=True, type=Path, help="Path to write v2 checkpoint JSON")

    # Subcommand: rollback
    p_roll = subparsers.add_parser("rollback", help="Roll back v2 checkpoint to v1 format")
    p_roll.add_argument("--input", "-i", required=True, type=Path, help="Path to v2 checkpoint JSON")
    p_roll.add_argument("--output", "-o", required=True, type=Path, help="Path to write v1 checkpoint JSON")

    # Subcommand: verify-conservation
    p_cons = subparsers.add_parser("verify-conservation", help="Verify state conservation between two checkpoints")
    p_cons.add_argument("--before", "-b", required=True, type=Path, help="Path to baseline checkpoint JSON")
    p_cons.add_argument("--after", "-a", required=True, type=Path, help="Path to migrated checkpoint JSON")

    # Subcommand: verify-reproducibility
    p_rep = subparsers.add_parser("verify-reproducibility", help="Verify cross-platform simulation reproducibility")
    p_rep.add_argument("--reference", "-r", required=True, type=Path, help="Reference run output JSON")
    p_rep.add_argument("--target", "-t", required=True, type=Path, help="Target platform run output JSON")
    p_rep.add_argument("--tolerance", type=float, default=1e-12, help="Floating point tolerance (default: 1e-12)")

    return parser


def main(argv: list[str] | None = None) -> int:
    """CLI entrypoint."""
    parser = build_parser()
    args = parser.parse_args(argv)

    try:
        if args.command == "migrate":
            with open(args.input, "r", encoding="utf-8") as f:
                data = json.load(f)
            v2 = migrate_v1_to_v2(data)
            args.output.parent.mkdir(parents=True, exist_ok=True)
            with open(args.output, "w", encoding="utf-8") as f:
                json.dump(v2, f, indent=2)
            print(f"Successfully migrated checkpoint to v2: {args.output}")

        elif args.command == "rollback":
            with open(args.input, "r", encoding="utf-8") as f:
                data = json.load(f)
            v1 = rollback_v2_to_v1(data)
            args.output.parent.mkdir(parents=True, exist_ok=True)
            with open(args.output, "w", encoding="utf-8") as f:
                json.dump(v1, f, indent=2)
            print(f"Successfully rolled back checkpoint to v1: {args.output}")

        elif args.command == "verify-conservation":
            with open(args.before, "r", encoding="utf-8") as f:
                before = json.load(f)
            with open(args.after, "r", encoding="utf-8") as f:
                after = json.load(f)
            verify_checkpoint_conservation(before, after)
            print("Checkpoint conservation verified: 100% state preserved.")

        elif args.command == "verify-reproducibility":
            with open(args.reference, "r", encoding="utf-8") as f:
                ref = json.load(f)
            with open(args.target, "r", encoding="utf-8") as f:
                tgt = json.load(f)
            metrics = verify_cross_platform_reproducibility(ref, tgt, tolerance=args.tolerance)
            ref_host = _extract_platform(ref)
            tgt_host = _extract_platform(tgt)
            report = generate_platform_reproducibility_report(ref_host, tgt_host, metrics)
            print(json.dumps(report, indent=2))

        return 0

    except CheckpointMigrationError as err:
        print(f"ERROR: {err}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
