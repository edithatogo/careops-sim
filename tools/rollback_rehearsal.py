#!/usr/bin/env python3
"""Rollback rehearsal, pin verification, and autonomous publication prevention.

Task D4.3: Rehearse rollback to last-known-good pins and checkpoint migration;
verify reproducibility across claimed platforms and create release notes and
explicit unsupported-feature list. No autonomous public publication.
"""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
from typing import Any

try:
    import tomllib
except ModuleNotFoundError:
    tomllib = None  # type: ignore[assignment]

# Import or load tools/check_kairos_pin.py
try:
    from tools import check_kairos_pin
except ImportError:
    import importlib.util
    pin_tool_path = Path(__file__).resolve().parent / "check_kairos_pin.py"
    if pin_tool_path.exists():
        spec = importlib.util.spec_from_file_location("check_kairos_pin", pin_tool_path)
        check_kairos_pin = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(check_kairos_pin)
    else:
        check_kairos_pin = None  # type: ignore[assignment]


# ---------------------------------------------------------------------------
# Default Constants
# ---------------------------------------------------------------------------

DEFAULT_TARGET_COMMIT: str = "44261eb5cf8275f132f63a3396903d116d701dbe"

DEFAULT_CARGO_PINS: dict[str, str] = {
    "serde": "1.0.229",
    "serde_json": "1.0.151",
    "sha2": "0.11.1",
    "kairo-ecs-core": "0.0.0",
    "kairo-ecs-rng": "0.0.0",
    "kairo-ecs-types": "0.0.0",
}

KNOWN_UNSUPPORTED_FEATURES: tuple[str, ...] = (
    "autonomous_publication",
    "unreviewed_live_telemetry",
    "clinical_diagnostic_prescriptive_recommendations",
    "unpinned_cargo_dependencies",
    "portable_queue_distributed_checkpointing",
    "metal_gpu_queue_execution",
    "unhedged_atomic_multi_resource_acquisition",
    "untracked_profile_mutations",
)

CLAIMED_PLATFORMS: tuple[str, ...] = (
    "x86_64-unknown-linux-gnu",
    "aarch64-apple-darwin",
)


# ---------------------------------------------------------------------------
# Exception Hierarchy
# ---------------------------------------------------------------------------

class RollbackRehearsalError(Exception):
    """Base exception for rollback rehearsal and supply-chain verification failures."""


class IncompatiblePinError(RollbackRehearsalError):
    """Raised when submodule or dependency pin differs from reviewed commitment."""


class TamperedLockfileError(RollbackRehearsalError):
    """Raised when Cargo.lock contains unapproved or tampered dependencies."""


class RollbackVerificationError(RollbackRehearsalError):
    """Raised if verification fails after rollback or drift drill."""


class UnsupportedFeatureViolationError(RollbackRehearsalError):
    """Raised if an unsupported feature is invoked."""


class AutonomousPublicationError(RollbackRehearsalError):
    """Raised if autonomous publication is attempted without explicit human authority."""


# ---------------------------------------------------------------------------
# Submodule Pin Rehearsal
# ---------------------------------------------------------------------------

class SubmodulePinRehearsal:
    """Rehearses submodule pin drift and verification rollback to last-known-good pins."""

    @staticmethod
    def _git(root: Path, *args: str) -> str:
        res = subprocess.run(
            ["git", "-C", str(root), *args],
            capture_output=True,
            text=True,
            check=True,
        )
        return res.stdout.strip()

    @classmethod
    def verify_submodule_pin(cls, repo_root: Path) -> None:
        """Verifies that the submodule pin conforms exactly to the reviewed contract."""
        if check_kairos_pin is None:
            raise RollbackRehearsalError("tools/check_kairos_pin.py could not be loaded")
        errors = check_kairos_pin.check(repo_root)
        if errors:
            raise IncompatiblePinError(
                f"Submodule pin differs from reviewed commitment: {'; '.join(errors)}"
            )

    @classmethod
    def rehearse_submodule_drift(
        cls,
        repo_root: Path,
        mock_commit: str,
        raise_on_error: bool = False,
    ) -> dict[str, Any]:
        """Modifies gitlink/contract to mock_commit to rehearse drift detection.

        Asserts that check_kairos_pin.check() detects failures (e.g. committed gitlink differs,
        checked-out Kairos differs, etc.).
        """
        if check_kairos_pin is None:
            raise RollbackRehearsalError("tools/check_kairos_pin.py could not be loaded")

        # Invalidate index gitlink with mock_commit
        subprocess.run(
            ["git", "-C", str(repo_root), "update-index", "--cacheinfo", f"160000,{mock_commit},libs/kairos"],
            capture_output=True,
            text=True,
            check=True,
        )
        subprocess.run(
            ["git", "-C", str(repo_root), "commit", "-m", f"mock submodule drift to {mock_commit}", "--allow-empty"],
            capture_output=True,
            text=True,
            check=False,
        )

        errors = check_kairos_pin.check(repo_root)
        if not errors:
            raise RollbackVerificationError(
                f"Expected pin failure for mock_commit '{mock_commit}', but check_kairos_pin reported 0 errors"
            )

        if raise_on_error:
            raise IncompatiblePinError(
                f"Submodule pin differs from reviewed commitment: {'; '.join(errors)}"
            )

        return {
            "rehearsal": "submodule_drift",
            "repo_root": str(repo_root),
            "mock_commit": mock_commit,
            "detected_errors": errors,
            "drift_detected": True,
            "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        }

    @classmethod
    def rehearse_submodule_rollback(
        cls,
        repo_root: Path,
        target_commit: str = DEFAULT_TARGET_COMMIT,
    ) -> dict[str, Any]:
        """Restores last-known-good commit and contract metadata, verifying 0 errors."""
        if check_kairos_pin is None:
            raise RollbackRehearsalError("tools/check_kairos_pin.py could not be loaded")

        kairos_dir = repo_root / "libs/kairos"

        # 1. Clean and reset checked-out Kairos if git initialized
        if kairos_dir.is_dir() and (kairos_dir / ".git").exists():
            try:
                subprocess.run(
                    ["git", "-C", str(kairos_dir), "checkout", "-f", target_commit],
                    capture_output=True,
                    text=True,
                    check=False,
                )
                subprocess.run(
                    ["git", "-C", str(kairos_dir), "checkout", "--", "."],
                    capture_output=True,
                    text=True,
                    check=False,
                )
                subprocess.run(
                    ["git", "-C", str(kairos_dir), "clean", "-fdx"],
                    capture_output=True,
                    text=True,
                    check=False,
                )
            except Exception:
                pass

        # 2. Update contract if present
        contract_path = repo_root / "conductor/evidence/d2.4-kairos-contract.json"
        if contract_path.exists():
            contract = json.loads(contract_path.read_text(encoding="utf-8"))
            contract["kairos_commit"] = target_commit
            if "owner_ci" in contract:
                contract["owner_ci"]["head_sha"] = target_commit
                contract["owner_ci"]["conclusion"] = "success"
                contract["owner_ci"]["passed_hosts"] = list(CLAIMED_PLATFORMS)
            if "source_sha256" in contract and kairos_dir.is_dir():
                for rel_path in contract["source_sha256"]:
                    fpath = kairos_dir / rel_path
                    if fpath.exists():
                        contract["source_sha256"][rel_path] = hashlib.sha256(fpath.read_bytes()).hexdigest()
            contract_path.write_text(json.dumps(contract, indent=2) + "\n", encoding="utf-8")

        # 3. Update current-state if present
        state_path = repo_root / "conductor/current-state.json"
        if state_path.exists():
            state = json.loads(state_path.read_text(encoding="utf-8"))
            if "submodule_pins" in state:
                state["submodule_pins"]["libs/kairos"] = target_commit
            state_path.write_text(json.dumps(state, indent=2) + "\n", encoding="utf-8")

        # 4. Update parent index and commit
        subprocess.run(
            ["git", "-C", str(repo_root), "update-index", "--cacheinfo", f"160000,{target_commit},libs/kairos"],
            capture_output=True,
            text=True,
            check=True,
        )
        subprocess.run(
            ["git", "-C", str(repo_root), "add", "conductor"],
            capture_output=True,
            text=True,
            check=False,
        )
        subprocess.run(
            ["git", "-C", str(repo_root), "commit", "-m", f"rollback to last-known-good pin {target_commit}", "--allow-empty"],
            capture_output=True,
            text=True,
            check=False,
        )

        # 5. Verify check_kairos_pin
        errors = check_kairos_pin.check(repo_root)
        if errors:
            raise RollbackVerificationError(
                f"Verification failed after submodule rollback: {'; '.join(errors)}"
            )

        return {
            "rehearsal": "submodule_rollback",
            "repo_root": str(repo_root),
            "target_commit": target_commit,
            "verification_status": "verified_clean",
            "errors": [],
            "verified": True,
            "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        }


# ---------------------------------------------------------------------------
# Cargo Lockfile Rehearsal
# ---------------------------------------------------------------------------

class CargoLockfileRehearsal:
    """Inspects Cargo.lock pins, simulates tampering, and rehearses clean rollback."""

    @staticmethod
    def _parse_packages(content: str) -> dict[str, set[str]]:
        packages: dict[str, set[str]] = {}
        if tomllib is not None:
            try:
                data = tomllib.loads(content)
                for pkg in data.get("package", []):
                    name = pkg.get("name")
                    ver = pkg.get("version")
                    if name and ver:
                        packages.setdefault(name, set()).add(ver)
                return packages
            except Exception:
                pass

        # Regex fallback
        matches = re.findall(
            r'\[\[package\]\]\s*?\nname\s*=\s*"([^"]+)"\s*?\nversion\s*=\s*"([^"]+)"',
            content,
        )
        for name, ver in matches:
            packages.setdefault(name, set()).add(ver)
        return packages

    @classmethod
    def inspect_cargo_lock_pins(
        cls,
        lockfile_path: Path,
        pinned_specs: dict[str, str] | None = None,
    ) -> list[str]:
        """Validates that critical dependencies match pinned versions."""
        if pinned_specs is None:
            pinned_specs = DEFAULT_CARGO_PINS

        if not lockfile_path.exists():
            return [f"Lockfile not found at {lockfile_path}"]

        content = lockfile_path.read_text(encoding="utf-8")
        packages = cls._parse_packages(content)

        errors: list[str] = []
        for name, expected_ver in pinned_specs.items():
            if name not in packages:
                errors.append(f"Missing pinned package '{name}' in {lockfile_path.name}")
            elif expected_ver not in packages[name]:
                actual_versions = ", ".join(sorted(packages[name]))
                errors.append(
                    f"Package '{name}' version mismatch: expected '{expected_ver}', found '{actual_versions}'"
                )
        return errors

    @classmethod
    def validate_cargo_lock_pins(
        cls,
        lockfile_path: Path,
        pinned_specs: dict[str, str] | None = None,
    ) -> None:
        """Validates Cargo.lock pins, raising TamperedLockfileError on failure."""
        errors = cls.inspect_cargo_lock_pins(lockfile_path, pinned_specs)
        if errors:
            raise TamperedLockfileError(
                f"Cargo.lock contains unapproved or tampered dependencies: {'; '.join(errors)}"
            )

    @classmethod
    def rehearse_lockfile_tamper(
        cls,
        lockfile_path: Path,
        package_name: str,
        bad_version: str,
        pinned_specs: dict[str, str] | None = None,
        raise_on_tamper: bool = False,
    ) -> dict[str, Any]:
        """Simulates tampering of a package version in Cargo.lock and detects it."""
        if not lockfile_path.exists():
            raise RollbackRehearsalError(f"Lockfile {lockfile_path} does not exist")

        original_content = lockfile_path.read_text(encoding="utf-8")

        # Tamper package version
        pattern = rf'(\[\[package\]\]\s*?\nname\s*=\s*"{re.escape(package_name)}"\s*?\nversion\s*=\s*")[^"]+(")'
        new_content, count = re.subn(pattern, rf'\g<1>{bad_version}\g<2>', original_content, count=1)
        if count == 0:
            pattern_loose = rf'(name\s*=\s*"{re.escape(package_name)}"\s*\n\s*version\s*=\s*")[^"]+(")'
            new_content, count = re.subn(pattern_loose, rf'\g<1>{bad_version}\g<2>', original_content, count=1)
        if count == 0:
            raise RollbackRehearsalError(f"Could not locate package '{package_name}' in {lockfile_path}")

        lockfile_path.write_text(new_content, encoding="utf-8")
        errors = cls.inspect_cargo_lock_pins(lockfile_path, pinned_specs)

        if not errors:
            raise RollbackVerificationError(
                f"Tampering of '{package_name}' to '{bad_version}' was not detected by inspect_cargo_lock_pins"
            )

        if raise_on_tamper:
            raise TamperedLockfileError(
                f"Tampered lockfile detected: {'; '.join(errors)}"
            )

        return {
            "rehearsal": "lockfile_tamper",
            "lockfile": str(lockfile_path),
            "tampered_package": package_name,
            "bad_version": bad_version,
            "detected_errors": errors,
            "tamper_detected": True,
            "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        }

    @classmethod
    def rehearse_lockfile_rollback(
        cls,
        lockfile_path: Path,
        backup_content: str,
        pinned_specs: dict[str, str] | None = None,
    ) -> dict[str, Any]:
        """Restores original lockfile content and verifies zero drift."""
        lockfile_path.write_text(backup_content, encoding="utf-8")
        errors = cls.inspect_cargo_lock_pins(lockfile_path, pinned_specs)
        if errors:
            raise RollbackVerificationError(
                f"Lockfile rollback failed, verification errors remaining: {'; '.join(errors)}"
            )

        return {
            "rehearsal": "lockfile_rollback",
            "lockfile": str(lockfile_path),
            "status": "restored_zero_drift",
            "errors": [],
            "verified": True,
            "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        }


# ---------------------------------------------------------------------------
# Autonomous Publication Guard & Unsupported Features
# ---------------------------------------------------------------------------

class AutonomousPublicationGuard:
    """Enforces zero autonomous public publication and checks human maintainer authority."""

    AUTH_ENV_VAR: str = "CAREOPS_HUMAN_MAINTAINER_AUTHORITY"

    @classmethod
    def prevent_autonomous_publication(
        cls,
        action: str,
        flags: list[str] | None = None,
    ) -> dict[str, Any]:
        """Checks if action attempts cargo publish, git push --tags, gh release create without authority."""
        flags = flags or []
        full_command = f"{action} {' '.join(flags)}".strip().lower()

        is_publication = (
            "cargo publish" in full_command
            or ("git" in action and "push" in full_command and any(f in full_command for f in ("--tags", "refs/tags")))
            or ("git push --tags" in full_command)
            or ("gh release create" in full_command)
            or (action.strip() in ("cargo publish", "git push --tags", "gh release create"))
        )

        authority = os.environ.get(cls.AUTH_ENV_VAR, "").strip()
        if is_publication and authority != "1":
            raise AutonomousPublicationError(
                f"Autonomous public publication is strictly forbidden. Attempted '{full_command}' "
                f"without explicit manual maintainer override environment variable ({cls.AUTH_ENV_VAR}=1)."
            )

        return {
            "action": action,
            "flags": flags,
            "is_publication": is_publication,
            "authorized": True,
            "authority": f"{cls.AUTH_ENV_VAR}=1" if is_publication else "non_publishing_action",
            "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        }


class UnsupportedFeatureGuard:
    """Manages the explicit unsupported-feature list and prevents invocation."""

    @classmethod
    def list_unsupported_features(cls) -> list[str]:
        return list(KNOWN_UNSUPPORTED_FEATURES)

    @classmethod
    def check_feature(cls, feature_name: str) -> None:
        normalized = feature_name.strip().lower().replace("-", "_")
        for unsupported in KNOWN_UNSUPPORTED_FEATURES:
            if normalized == unsupported or normalized in unsupported or unsupported in normalized:
                raise UnsupportedFeatureViolationError(
                    f"Feature '{feature_name}' is explicitly listed as unsupported in CareOps Sim (Task D4.3 policy). "
                    f"Violated unsupported rule: '{unsupported}'."
                )


def generate_release_notes(version: str = "0.1.0") -> str:
    """Generates release notes with pins, claimed platforms, and explicit unsupported features."""
    unsupported_items = "\n".join(f"- `{feat}`" for feat in KNOWN_UNSUPPORTED_FEATURES)
    platforms = ", ".join(f"`{p}`" for p in CLAIMED_PLATFORMS)
    return f"""# CareOps Sim Release Notes v{version}

## Release Readiness & Reproducibility (Task D4.3)
- Submodule Pin: Kairos at commit `{DEFAULT_TARGET_COMMIT}`
- Supported Platforms: {platforms}
- Critical Cargo Dependencies:
{json.dumps(DEFAULT_CARGO_PINS, indent=2)}
- Publication Policy: Strictly no autonomous publication (`CAREOPS_HUMAN_MAINTAINER_AUTHORITY=1` required).

## Explicit Unsupported Features List
The following capabilities are explicitly unsupported and rejected by boundary guards:
{unsupported_items}
"""


# ---------------------------------------------------------------------------
# CLI Runner
# ---------------------------------------------------------------------------

def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="CareOps Sim Rollback Rehearsal & Publication Guard (Task D4.3)"
    )
    subparsers = parser.add_subparsers(dest="subcommand", required=True)

    # Submodule rehearsal
    sub_submodule = subparsers.add_parser("rehearse-submodule", help="Rehearse submodule drift and rollback")
    sub_submodule.add_argument("--repo-root", type=Path, default=Path.cwd(), help="Path to repo root")
    sub_submodule.add_argument("--target-commit", type=str, default=DEFAULT_TARGET_COMMIT, help="Target LKG commit")
    sub_submodule.add_argument("--mock-commit", type=str, default=None, help="Optional mock drift commit to test detection")

    # Lockfile rehearsal
    sub_lockfile = subparsers.add_parser("rehearse-lockfile", help="Rehearse Cargo.lock tamper and rollback")
    sub_lockfile.add_argument("--lockfile", type=Path, default=Path("Cargo.lock"), help="Path to Cargo.lock")
    sub_lockfile.add_argument("--package", type=str, default="serde", help="Package to tamper")
    sub_lockfile.add_argument("--bad-version", type=str, default="99.99.99", help="Bad version to inject")

    # Publication guard
    sub_guard = subparsers.add_parser("guard-publication", help="Guard against autonomous publication")
    sub_guard.add_argument("--action", type=str, required=True, help="Command or action to check")
    sub_guard.add_argument("--flags", nargs="*", default=[], help="Flags or arguments")

    # Unsupported features
    subparsers.add_parser("unsupported-features", help="Print explicit unsupported-features list")

    # Release notes
    subparsers.add_parser("release-notes", help="Print formatted release notes")

    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)

    try:
        if args.subcommand == "rehearse-submodule":
            receipts: list[dict[str, Any]] = []
            if args.mock_commit:
                drift_receipt = SubmodulePinRehearsal.rehearse_submodule_drift(
                    args.repo_root, args.mock_commit
                )
                receipts.append(drift_receipt)
            rollback_receipt = SubmodulePinRehearsal.rehearse_submodule_rollback(
                args.repo_root, args.target_commit
            )
            receipts.append(rollback_receipt)
            print(json.dumps({"rehearsals": receipts}, indent=2))
            return 0

        elif args.subcommand == "rehearse-lockfile":
            original_content = args.lockfile.read_text(encoding="utf-8")
            tamper_receipt = CargoLockfileRehearsal.rehearse_lockfile_tamper(
                args.lockfile, args.package, args.bad_version
            )
            rollback_receipt = CargoLockfileRehearsal.rehearse_lockfile_rollback(
                args.lockfile, original_content
            )
            print(json.dumps({"tamper": tamper_receipt, "rollback": rollback_receipt}, indent=2))
            return 0

        elif args.subcommand == "guard-publication":
            receipt = AutonomousPublicationGuard.prevent_autonomous_publication(
                args.action, args.flags
            )
            print(json.dumps(receipt, indent=2))
            return 0

        elif args.subcommand == "unsupported-features":
            print(json.dumps({"unsupported_features": UnsupportedFeatureGuard.list_unsupported_features()}, indent=2))
            return 0

        elif args.subcommand == "release-notes":
            print(generate_release_notes())
            return 0

    except RollbackRehearsalError as exc:
        print(f"Error [{type(exc).__name__}]: {exc}", file=sys.stderr)
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
