"""Read-only D1.5 packet and receipt drift checks."""

from __future__ import annotations

import hashlib
import json
import os
import re
import subprocess
import sys
from pathlib import Path, PurePosixPath
from typing import Any

from command_receipts import validate_receipt


_SHA256 = re.compile(r"^[a-f0-9]{64}$")
_SHA1 = re.compile(r"^[a-f0-9]{40}$")
_IDENTIFIER = re.compile(r"^[A-Za-z0-9_.-]+$")
_DRIVE = re.compile(r"^[A-Za-z]:")


def _canonical_path(value: Any) -> bool:
    if not isinstance(value, str) or not value or "\x00" in value or "\\" in value:
        return False
    if value == ".":
        return True
    if value.startswith("/") or value.endswith("/") or "//" in value or _DRIVE.match(value):
        return False
    parts = value.split("/")
    return all(part not in ("", ".", "..") for part in parts)


def _overlap(left: str, right: str) -> bool:
    return left == right or left.startswith(right + "/") or right.startswith(left + "/")


def _within_any(path: str, reservations: list[str]) -> bool:
    return any(path == reservation or path.startswith(reservation.rstrip("/") + "/") for reservation in reservations)


def _inside(path: Path, root: Path) -> bool:
    return path == root or root in path.parents


def _resolve_under(root: Path, relative: str) -> Path | None:
    if not _canonical_path(relative):
        return None
    try:
        resolved_root = root.resolve(strict=True)
        resolved = (resolved_root / relative).resolve(strict=False)
    except (OSError, RuntimeError):
        return None
    return resolved if _inside(resolved, resolved_root) else None


def _relative_resolved(path: Path, root: Path) -> str | None:
    try:
        return path.resolve(strict=False).relative_to(root.resolve(strict=True)).as_posix() or "."
    except (OSError, RuntimeError, ValueError):
        return None


def _valid_hash_map(value: Any, *, nonempty: bool = False) -> bool:
    if not isinstance(value, dict) or (nonempty and not value):
        return False
    return all(_canonical_path(path) and isinstance(digest, str) and bool(_SHA256.fullmatch(digest))
               for path, digest in value.items())


def _unique_paths(value: Any, *, nonempty: bool = False) -> bool:
    return (isinstance(value, list) and (not nonempty or bool(value))
            and all(_canonical_path(item) for item in value)
            and len(value) == len(set(value)))


def _packet_configuration_errors(packet: Any, receipt_path: Any) -> list[str]:
    errors: list[str] = []
    if not isinstance(packet, dict):
        return ["Packet shape is invalid"]
    binding = packet.get("coordinator_binding")
    if not isinstance(binding, dict):
        return ["Coordinator binding is invalid"]
    if packet.get("status") != "prepared":
        errors.append("Packet status is invalid")
    if not isinstance(packet.get("packet_id"), str) or not _IDENTIFIER.fullmatch(packet["packet_id"]):
        errors.append("Packet identity is invalid")
    if packet.get("target_repo") != ".":
        errors.append("Packet target repository is invalid")
    if not isinstance(packet.get("base_commit"), str) or not _SHA1.fullmatch(packet["base_commit"]):
        errors.append("Packet base commit is invalid")

    write_paths = packet.get("write_paths")
    if not _unique_paths(write_paths, nonempty=True):
        errors.append("Packet write reservations are invalid")
        write_paths = []
    protected = packet.get("protected_paths")
    if not _unique_paths(protected):
        errors.append("Packet protected paths are invalid")
        protected = []
    result_path = packet.get("result_path")
    bound_receipt_path = binding.get("receipt_path")
    if not _canonical_path(result_path) or result_path not in write_paths:
        errors.append("Packet result path is invalid")
    if not _canonical_path(bound_receipt_path) or bound_receipt_path not in write_paths:
        errors.append("Bound receipt path is invalid")
    if not isinstance(receipt_path, str) or not _canonical_path(receipt_path) or receipt_path != bound_receipt_path:
        errors.append("Receipt path does not match the coordinator binding")
    # Malformed packet values must produce sanitized validation errors, not
    # trigger an unhashable-type exception while assembling control paths.
    sidecars = {path for path in (result_path, bound_receipt_path)
                if _canonical_path(path)}

    required = binding.get("required_outputs")
    if not _unique_paths(required, nonempty=True):
        errors.append("Required output list is invalid")
        required = []
    for path in required:
        if path not in write_paths:
            errors.append("Required output is outside the write reservation")
        if path in sidecars:
            errors.append("Control sidecar cannot be a required output")

    if not isinstance(packet.get("input_hashes"), dict) or not _valid_hash_map(packet.get("input_hashes")):
        errors.append("Packet input hash map is invalid")
    if not isinstance(packet.get("source_hashes"), dict) or not _valid_hash_map(packet.get("source_hashes")):
        errors.append("Packet source hash map is invalid")
    if not isinstance(packet.get("verification"), list) or not packet.get("verification"):
        errors.append("Packet verification list is invalid")
    else:
        for check in packet["verification"]:
            if not isinstance(check, dict):
                errors.append("Packet verification entry is invalid")
                continue
            argv = check.get("argv")
            if (not isinstance(argv, list) or not argv
                    or not all(isinstance(arg, str) and bool(arg) for arg in argv)
                    or not _canonical_path(check.get("cwd"))
                    or isinstance(check.get("expected_exit"), bool)
                    or not isinstance(check.get("expected_exit"), int)):
                errors.append("Packet verification entry is invalid")

    dependencies = packet.get("leaf_dependencies")
    prereqs = binding.get("prerequisite_receipts")
    if (not isinstance(dependencies, list)
            or not all(isinstance(item, str) and bool(_IDENTIFIER.fullmatch(item)) for item in dependencies)
            or len(dependencies) != len(set(dependencies))):
        errors.append("Packet prerequisite list is invalid")
    elif not isinstance(prereqs, dict) or set(prereqs) != set(dependencies):
        errors.append("Prerequisite receipt coverage is invalid")
    return errors


def _git_head(target: Path) -> str | None:
    result = subprocess.run(
        ["git", "-C", str(target), "rev-parse", "--verify", "HEAD"],
        stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False,
    )
    if result.returncode != 0:
        return None
    try:
        return result.stdout.decode("ascii").strip()
    except UnicodeDecodeError:
        return None


def _git_changed_paths(target: Path) -> tuple[set[str] | None, list[str]]:
    result = subprocess.run(
        ["git", "-C", str(target), "status", "--porcelain=v1", "-z", "--untracked-files=all"],
        stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False,
    )
    if result.returncode != 0:
        return None, ["Git working-tree status could not be read"]
    entries = result.stdout.split(b"\0")
    changed: set[str] = set()
    errors: list[str] = []
    for entry in entries:
        if not entry:
            continue
        if len(entry) < 4 or entry[2:3] != b" ":
            errors.append("Git working-tree status format is invalid")
            continue
        try:
            state = entry[:2].decode("ascii")
            path = entry[3:].decode("utf-8")
        except UnicodeDecodeError:
            errors.append("Git working-tree path encoding is unsupported")
            continue
        if "R" in state or "C" in state:
            errors.append("Git renames and copies are not accepted")
            continue
        if "D" in state:
            errors.append("Git deletions are not accepted")
            continue
        if not _canonical_path(path):
            errors.append("Git working-tree path is noncanonical")
            continue
        changed.add(path)
    return changed, errors


def _hash_matches(path: Path, expected: str) -> bool:
    try:
        return path.is_file() and hashlib.sha256(path.read_bytes()).hexdigest() == expected
    except OSError:
        return False


def _prerequisite_errors(packet: dict[str, Any], repo_root: Path) -> list[str]:
    errors: list[str] = []
    binding = packet["coordinator_binding"]
    dependencies = packet["leaf_dependencies"]
    prereqs = binding["prerequisite_receipts"]
    for dependency in dependencies:
        item = prereqs.get(dependency)
        if not isinstance(item, dict):
            errors.append("Prerequisite receipt is invalid")
            continue
        if item.get("status") != "accepted":
            errors.append("Prerequisite status is not accepted")
        if not isinstance(item.get("reviewer"), str) or not item["reviewer"].strip():
            errors.append("Prerequisite reviewer is missing")
        if item.get("all_instances_accepted") is not True:
            errors.append("Prerequisite instance acceptance is incomplete")
        expected = item.get("expected_instances")
        accepted = item.get("accepted_instances")
        if (not _unique_paths(expected, nonempty=True) or not _unique_paths(accepted, nonempty=True)
                or expected != accepted):
            errors.append("Prerequisite instance sets are invalid")
        artifacts = item.get("artifacts")
        if not _valid_hash_map(artifacts, nonempty=True):
            errors.append("Prerequisite artifact map is invalid")
            continue
        for path, digest in artifacts.items():
            resolved = _resolve_under(repo_root, path)
            if resolved is None or not _hash_matches(resolved, digest):
                errors.append("Prerequisite artifact hash drift detected")
    return errors


def verify_acceptance(receipt: Any, receipt_path: Any, packet: Any, repo_root: Any) -> list[str]:
    """Check a worker receipt against its bound packet and current pre-commit tree."""
    errors = _packet_configuration_errors(packet, receipt_path)
    if errors or not isinstance(packet, dict) or not isinstance(receipt, dict):
        if not isinstance(receipt, dict):
            errors.append("Receipt shape is invalid")
        return errors
    try:
        root = Path(repo_root).resolve(strict=True)
        target = _resolve_under(root, packet["target_repo"])
        if target is None or not target.is_dir():
            return ["Target repository path is invalid"]
    except (OSError, RuntimeError, TypeError):
        return ["Repository root is invalid"]

    receipt_errors = validate_receipt(receipt)
    if receipt_errors:
        # The receipt schema guarantees the shapes used below (notably hash
        # maps and changed-path lists). Fail closed before operating on an
        # untrusted malformed structure.
        return ["Receipt schema or semantic validation failed"]
    if receipt.get("status") != "ready_for_review":
        errors.append("Receipt is not ready for review")
    if receipt.get("packet_id") != packet.get("packet_id"):
        errors.append("Receipt packet identity does not match")
    if receipt.get("target_repo") != packet.get("target_repo"):
        errors.append("Receipt target repository does not match")
    if receipt.get("base_commit") != packet.get("base_commit"):
        errors.append("Receipt base commit does not match")
    if receipt.get("inputs_sha256") != packet.get("input_hashes"):
        errors.append("Receipt input hash map does not match")

    if _git_head(target) != packet.get("base_commit"):
        errors.append("Current Git base has drifted")
    for path, digest in packet["input_hashes"].items():
        resolved = _resolve_under(target, path)
        if resolved is None or not _hash_matches(resolved, digest):
            errors.append("Packet input hash drift detected")
    for path, digest in packet["source_hashes"].items():
        resolved = _resolve_under(root, path)
        if resolved is None or not _hash_matches(resolved, digest):
            errors.append("Packet source hash drift detected")

    commands = receipt.get("commands")
    checks = packet.get("verification")
    if isinstance(commands, list) and isinstance(checks, list):
        if len(commands) != len(checks):
            errors.append("Receipt verification command count does not match")
        for index, (command, check) in enumerate(zip(commands, checks)):
            if not isinstance(command, dict) or not isinstance(check, dict):
                errors.append("Receipt verification command is invalid")
                continue
            if (command.get("argv") != check.get("argv")
                    or command.get("cwd") != check.get("cwd")
                    or command.get("expected_exit") != check.get("expected_exit")):
                errors.append("Receipt verification command does not match")

    binding = packet["coordinator_binding"]
    required = binding["required_outputs"]
    sidecars = {packet["result_path"], binding["receipt_path"]}
    output_hashes = receipt.get("outputs_sha256")
    changed_paths = receipt.get("changed_paths")
    if not isinstance(output_hashes, dict):
        output_hashes = {}
    if not isinstance(changed_paths, list):
        changed_paths = []
    if any(path in output_hashes or path in changed_paths for path in sidecars):
        errors.append("Control sidecars must be excluded from receipt outputs and changes")

    inputs = list(packet["input_hashes"]) + list(packet["source_hashes"])
    protected = packet["protected_paths"]
    sidecar_resolved = []
    for sidecar in sidecars:
        resolved_sidecar = _resolve_under(target, sidecar)
        if resolved_sidecar is not None:
            normalized_sidecar = _relative_resolved(resolved_sidecar, target)
            if normalized_sidecar is not None:
                sidecar_resolved.append(normalized_sidecar)
    output_paths = set(output_hashes) | set(changed_paths) | set(required)
    for path in output_paths:
        if not _canonical_path(path):
            errors.append("Receipt output path is noncanonical")
            continue
        if not _within_any(path, packet["write_paths"]):
            errors.append("Receipt output path is outside the write reservation")
        if path in sidecars:
            continue
        output_resolved = _resolve_under(target, path)
        if output_resolved is None:
            errors.append("Receipt output path escapes the target repository")
            continue
        output_rel = _relative_resolved(output_resolved, target)
        if output_rel is None:
            errors.append("Receipt output path cannot be normalized")
            continue
        if any(_overlap(output_rel, item) for item in sidecar_resolved):
            errors.append("Receipt output resolves into a control sidecar")
        protected_resolved = []
        for protected_path in protected:
            resolved_protected = _resolve_under(target, protected_path)
            if resolved_protected is not None:
                protected_rel = _relative_resolved(resolved_protected, target)
                if protected_rel is not None:
                    protected_resolved.append(protected_rel)
        if any(_overlap(output_rel, item) for item in protected_resolved):
            errors.append("Receipt output resolves into a protected path")
        for input_path in inputs:
            resolved_input = _resolve_under(target, input_path)
            if resolved_input is not None:
                input_rel = _relative_resolved(resolved_input, target)
                if input_rel is not None and _overlap(output_rel, input_rel):
                    errors.append("Receipt output aliases a packet input")
                    break

    for path in required:
        if path not in output_hashes:
            errors.append("Required output hash is missing")
        if any(_overlap(path, item) for item in inputs):
            errors.append("Required output overlaps a packet input")
    for path, digest in output_hashes.items():
        if path in sidecars:
            continue
        resolved = _resolve_under(target, path) if _canonical_path(path) else None
        if resolved is None or not _hash_matches(resolved, digest):
            errors.append("Receipt output hash drift detected")

    errors.extend(_prerequisite_errors(packet, root))
    actual, git_errors = _git_changed_paths(target)
    errors.extend(git_errors)
    if actual is not None:
        actual.difference_update(sidecars)
        if set(changed_paths) != actual:
            errors.append("Receipt changed paths do not match Git working-tree paths")
    return list(dict.fromkeys(errors))


def _main(argv: list[str]) -> int:
    if len(argv) != 6 or argv[1] != "verify" or argv[4] != "--repo":
        print("usage: python3 tools/receipt_drift.py verify RECEIPT_JSON PACKET_JSON --repo ROOT", file=sys.stderr)
        return 2
    try:
        with Path(argv[3]).open(encoding="utf-8") as stream:
            packet = json.load(stream)
    except (OSError, UnicodeError, json.JSONDecodeError):
        print("packet JSON could not be read", file=sys.stderr)
        return 1
    receipt_path = argv[2]
    static_errors = _packet_configuration_errors(packet, receipt_path)
    if static_errors:
        print("; ".join(static_errors), file=sys.stderr)
        return 1
    try:
        root = Path(argv[5]).resolve(strict=True)
        receipt_file = _resolve_under(root, receipt_path)
    except (OSError, RuntimeError):
        receipt_file = None
        root = Path(".")
    if receipt_file is None or not receipt_file.is_file():
        print("Bound receipt file could not be read", file=sys.stderr)
        return 1
    try:
        with receipt_file.open(encoding="utf-8") as stream:
            receipt = json.load(stream)
    except (OSError, UnicodeError, json.JSONDecodeError):
        print("Bound receipt JSON is invalid", file=sys.stderr)
        return 1
    errors = verify_acceptance(receipt, receipt_path, packet, root)
    if errors:
        print("; ".join(errors), file=sys.stderr)
        return 1
    print("Receipt and packet acceptance checks passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(_main(sys.argv))
