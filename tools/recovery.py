"""Pure, read-only recovery decisions for supervised worker attempts."""

from __future__ import annotations

import argparse
import datetime as _datetime
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any

import tasks

_SHA1 = re.compile(r"^[a-f0-9]{40}$")
_SHA256 = re.compile(r"^[a-f0-9]{64}$")
_ID = re.compile(r"^[A-Za-z0-9_.-]{1,128}$")
_DRIVE = re.compile(r"^[A-Za-z]:")
_SENSITIVE = re.compile(
    r"(?:api[_-]?key|access[_-]?token|token|password|passwd|secret|authorization)\s*[:=]|"
    r"\bbearer\s+[A-Za-z0-9._~+/=-]{8,}|-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----|"
    r"\bAKIA[0-9A-Z]{16}\b|\bgh[pousr]_[A-Za-z0-9]{20,}\b|\b\d{3}-\d{2}-\d{4}\b|"
    r"\bMRN\s*[:#=]", re.IGNORECASE
)


def _canonical(path: Any) -> bool:
    if not isinstance(path, str) or not path or len(path) > 512 or "\x00" in path or "\\" in path:
        return False
    if path == ".":
        return True
    if path.startswith("/") or path.endswith("/") or "//" in path or _DRIVE.match(path):
        return False
    return all(part not in ("", ".", "..") for part in path.split("/"))


def _overlap(left: str, right: str) -> bool:
    return left == right or left.startswith(right.rstrip("/") + "/") or right.startswith(left.rstrip("/") + "/")


def _within(path: str, reservations: list[str]) -> bool:
    return any(path == item or path.startswith(item.rstrip("/") + "/") for item in reservations)


def _inside(path: Path, root: Path) -> bool:
    return path == root or root in path.parents


def _resolve(root: Path, relative: str, *, strict: bool = False) -> Path | None:
    if not _canonical(relative):
        return None
    try:
        base = root.resolve(strict=True)
        found = (base / relative).resolve(strict=strict)
    except (OSError, RuntimeError, ValueError):
        return None
    return found if _inside(found, base) else None


def _valid_time(value: Any) -> bool:
    if not isinstance(value, str) or not re.fullmatch(r"\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d(?:\.\d+)?Z", value):
        return False
    try:
        _datetime.datetime.fromisoformat(value[:-1] + "+00:00")
        return True
    except ValueError:
        return False


def _attestation(value: Any, packet_id: str, attempt_id: str) -> bool:
    if not isinstance(value, dict):
        return False
    if value.get("packet_id") != packet_id or value.get("attempt_id") != attempt_id:
        return False
    confirmer = value.get("confirmer")
    basis = value.get("basis")
    return (isinstance(confirmer, str) and bool(confirmer.strip()) and len(confirmer) <= 128
            and isinstance(basis, str) and bool(basis.strip()) and len(basis) <= 512
            and not _SENSITIVE.search(confirmer + " " + basis)
            and not any(ord(char) < 32 for char in confirmer + basis)
            and _valid_time(value.get("timestamp_utc")))


def _issue_resolution(value: Any, packet_id: str, attempt_id: str) -> bool:
    if not isinstance(value, dict):
        return False
    if value.get("packet_id") != packet_id or value.get("attempt_id") != attempt_id:
        return False
    resolver = value.get("resolver")
    summary = value.get("summary")
    return (isinstance(resolver, str) and bool(resolver.strip()) and len(resolver) <= 128
            and isinstance(summary, str) and bool(summary.strip()) and len(summary) <= 512
            and not _SENSITIVE.search(resolver + " " + summary)
            and not any(ord(char) < 32 for char in resolver + summary)
            and _valid_time(value.get("timestamp_utc")))


def _decision(name: str, reason: str, **extra: Any) -> dict[str, Any]:
    return {"decision": name, "reason": reason, **extra}


def _packet_paths_safe(packet: dict[str, Any], root: Path) -> bool:
    """Contain all packet paths before the legacy checker follows symlinks."""
    target_repo = packet.get("target_repo")
    target = _resolve(root, target_repo, strict=True) if _canonical(target_repo) else None
    if target is None or not target.is_dir():
        return False

    def resolved_paths(values: Any, anchor: Path, *, strict: bool, require_file: bool = False) -> bool:
        if not isinstance(values, list):
            return False
        for value in values:
            if not _canonical(value):
                return False
            found = _resolve(anchor, value, strict=strict)
            if found is None or (require_file and not found.is_file()):
                return False
        return True

    contexts = packet.get("context_paths")
    inputs = packet.get("input_hashes")
    sources = packet.get("source_hashes", {})
    writes = packet.get("write_paths")
    protected = packet.get("protected_paths")
    if not resolved_paths(contexts, target, strict=True, require_file=True):
        return False
    if not isinstance(inputs, dict) or not isinstance(sources, dict):
        return False
    if not resolved_paths(list(inputs), target, strict=True, require_file=True):
        return False
    if not resolved_paths(list(sources), root, strict=True, require_file=True):
        return False
    if not resolved_paths(writes, target, strict=False) or not resolved_paths(protected, target, strict=False):
        return False
    result_path = packet.get("result_path")
    binding = packet.get("coordinator_binding")
    if not _canonical(result_path) or not isinstance(binding, dict):
        return False
    if _resolve(target, result_path) is None or not _canonical(binding.get("receipt_path")):
        return False
    if _resolve(target, binding["receipt_path"]) is None:
        return False
    verification = packet.get("verification")
    if not isinstance(verification, list) or not verification:
        return False
    for check in verification:
        if not isinstance(check, dict) or not _canonical(check.get("cwd")):
            return False
        cwd = _resolve(target, check["cwd"], strict=True)
        if cwd is None or not cwd.is_dir():
            return False

    dependencies = packet.get("leaf_dependencies")
    prerequisites = binding.get("prerequisite_receipts")
    if (not isinstance(dependencies, list) or not isinstance(prerequisites, dict)
            or not all(isinstance(item, str) and item for item in dependencies)
            or len(dependencies) != len(set(dependencies))
            or set(prerequisites) != set(dependencies)):
        return False
    for dependency in dependencies:
        item = prerequisites.get(dependency)
        if not isinstance(item, dict) or item.get("status") != "accepted" or item.get("all_instances_accepted") is not True:
            return False
        expected, accepted = item.get("expected_instances"), item.get("accepted_instances")
        if (not isinstance(expected, list) or not expected or expected != accepted
                or not all(isinstance(instance, str) and instance for instance in expected)
                or len(expected) != len(set(expected))
                or not isinstance(item.get("reviewer"), str) or not item["reviewer"].strip()):
            return False
        artifacts = item.get("artifacts")
        if not isinstance(artifacts, dict) or not artifacts:
            return False
        for artifact, digest in artifacts.items():
            if not _canonical(artifact) or not isinstance(digest, str) or not _SHA256.fullmatch(digest):
                return False
            found = _resolve(root, artifact, strict=True)
            if found is None or not found.is_file() or hashlib.sha256(found.read_bytes()).hexdigest() != digest:
                return False
    return True


def _packet_ready(packet: dict[str, Any], root: Path) -> bool:
    try:
        return _packet_paths_safe(packet, root) and not tasks.packet_errors(root, packet)
    except (KeyError, TypeError, ValueError, OSError, subprocess.SubprocessError):
        return False


def _validate_identity(ledger: dict[str, Any], packet: dict[str, Any], checkpoint: dict[str, Any],
                       root: Path) -> tuple[str | None, list[str] | None, list[dict[str, Any]] | None]:
    packet_id = packet.get("packet_id")
    attempt_id = ledger.get("attempt_id")
    if (not isinstance(packet_id, str) or not _ID.fullmatch(packet_id)
            or ledger.get("packet_id") != packet_id or not isinstance(attempt_id, str)
            or not _ID.fullmatch(attempt_id) or checkpoint.get("packet_id") != packet_id
            or checkpoint.get("attempt_id") != attempt_id):
        return None, None, None
    if ledger.get("target_repo") != packet.get("target_repo") or not _canonical(packet.get("target_repo")):
        return None, None, None
    if ledger.get("base_commit") != packet.get("base_commit") or not _SHA1.fullmatch(str(packet.get("base_commit", ""))):
        return None, None, None
    if checkpoint.get("schema_version") != 1:
        return None, None, None
    target = _resolve(root, packet["target_repo"], strict=True)
    if target is None or not target.is_dir():
        return None, None, None
    binding = packet.get("coordinator_binding")
    if not isinstance(binding, dict):
        return None, None, None
    receipt = binding.get("receipt_path")
    result = packet.get("result_path")
    if (not _canonical(receipt) or not _canonical(result) or ledger.get("receipt_path") != receipt
            or ledger.get("result_path") != result):
        return None, None, None
    write_paths = packet.get("write_paths")
    reserved = ledger.get("reserved_paths")
    if (not isinstance(write_paths, list) or not write_paths
            or not all(_canonical(item) for item in write_paths)
            or len(set(write_paths)) != len(write_paths)
            or not isinstance(reserved, list) or not all(_canonical(item) for item in reserved)
            or len(set(reserved)) != len(reserved)
            or any(path not in reserved for path in write_paths)):
        return None, None, None
    steps = packet.get("steps")
    if (not isinstance(steps, list) or not steps
            or not all(isinstance(label, str) and label.strip() for label in steps)):
        return None, None, None
    entries = checkpoint.get("steps")
    if not isinstance(entries, list) or len(entries) != len(steps):
        return None, None, None
    return attempt_id, write_paths, entries


def _checkpoint_prefix(ledger: dict[str, Any], packet: dict[str, Any], checkpoint: dict[str, Any],
                       root: Path, write_paths: list[str], entries: list[dict[str, Any]]) -> tuple[
                           list[dict[str, Any]] | None, dict[str, Any] | None, str | None
                       ]:
    target = _resolve(root, packet["target_repo"], strict=True)
    assert target is not None
    labels = packet["steps"]
    prefix_open = True
    accepted: list[dict[str, Any]] = []
    accepted_paths: list[tuple[str, Path]] = []
    first_not_started: dict[str, Any] | None = None
    unknown = False
    protected = packet.get("protected_paths")
    inputs = packet.get("input_hashes")
    source_hashes = packet.get("source_hashes", {})
    if (not isinstance(protected, list) or not all(_canonical(item) for item in protected)
            or not isinstance(inputs, dict) or not isinstance(source_hashes, dict)
            or not all(_canonical(p) for p in inputs) or not all(_canonical(p) for p in source_hashes)):
        return None, None, "checkpoint_or_packet_invalid"
    binding = packet["coordinator_binding"]
    sidecars = [packet["result_path"], binding["receipt_path"]]

    for index, (step, label) in enumerate(zip(entries, labels)):
        if not isinstance(step, dict) or isinstance(step.get("index"), bool) or step.get("index") != index or step.get("label") != label:
            return None, None, "checkpoint_or_packet_invalid"
        state = step.get("state")
        artifacts = step.get("artifacts_sha256")
        if state not in ("accepted", "started_unknown", "not_started") or not isinstance(artifacts, dict):
            return None, None, "checkpoint_or_packet_invalid"
        if state == "accepted":
            if not prefix_open or unknown or not artifacts:
                return None, None, "checkpoint_or_packet_invalid"
            checked: dict[str, str] = {}
            for path, digest in artifacts.items():
                if not _canonical(path) or not isinstance(digest, str) or not _SHA256.fullmatch(digest):
                    return None, None, "checkpoint_or_packet_invalid"
                if path in sidecars or not _within(path, write_paths):
                    return None, None, "artifact_path_rejected"
                resolved = _resolve(target, path, strict=True)
                if resolved is None or not resolved.is_file():
                    return None, None, "artifact_path_rejected"
                # Reject both lexical and resolved aliases/overlaps with inputs,
                # protected paths, and coordinator control sidecars.
                rel_resolved = resolved.relative_to(target.resolve()).as_posix()
                for other in sidecars + list(inputs) + list(source_hashes) + protected:
                    if _overlap(path, other):
                        return None, None, "artifact_path_rejected"
                    other_root = root if other in source_hashes else target
                    other_path = _resolve(other_root, other)
                    if other_path is not None:
                        try:
                            other_rel = other_path.relative_to(target.resolve()).as_posix()
                        except ValueError:
                            other_rel = None
                        if other_rel is not None and _overlap(rel_resolved, other_rel):
                            return None, None, "artifact_path_rejected"
                if hashlib.sha256(resolved.read_bytes()).hexdigest() != digest:
                    return None, None, "artifact_hash_drift"
                for prior_path, prior_resolved in accepted_paths:
                    if _overlap(path, prior_path) or resolved == prior_resolved:
                        return None, None, "artifact_path_rejected"
                accepted_paths.append((path, resolved))
                checked[path] = digest
            accepted.append({"index": index, "label": label, "artifacts_sha256": checked})
        elif state == "started_unknown":
            if artifacts:
                return None, None, "checkpoint_or_packet_invalid"
            unknown = True
            prefix_open = False
        else:
            if artifacts:
                return None, None, "checkpoint_or_packet_invalid"
            prefix_open = False
            if first_not_started is None:
                first_not_started = {"index": index, "label": label}
    if unknown:
        return accepted, None, "started_unknown"
    return accepted, first_not_started, None


def inspect_attempt(ledger, packet, checkpoint, packet_sha256, repo_root, *,
                    worker_stopped_attestation=None, new_attempt_id=None,
                    prior_attempt_ids=(), issue_resolution=None) -> dict[str, object]:
    """Return a bounded recovery decision without executing or writing anything."""
    if not isinstance(ledger, dict) or not isinstance(packet, dict) or not isinstance(checkpoint, dict):
        return _decision("blocked", "malformed_input")
    status = ledger.get("state")
    if status not in ("prepared", "running", "ready_for_review", "blocked", "failed", "cancelled", "integrated"):
        return _decision("blocked", "ledger_status_invalid")
    if status == "integrated":
        return _decision("terminal", "attempt_integrated")
    if status == "ready_for_review":
        return _decision("route_to_acceptance", "review_receipt_and_diff")
    if not isinstance(packet_sha256, str) or not _SHA256.fullmatch(packet_sha256):
        return _decision("rebind_required", "packet_hash_mismatch")
    if ledger.get("packet_sha256") != packet_sha256:
        return _decision("rebind_required", "packet_hash_mismatch")
    try:
        root = Path(repo_root).resolve(strict=True)
    except (OSError, RuntimeError, TypeError, ValueError):
        return _decision("blocked", "repository_root_invalid")
    if packet.get("status") != "prepared" or not _packet_ready(packet, root):
        return _decision("rebind_required", "prepared_packet_stale")
    attempt_id, write_paths, entries = _validate_identity(ledger, packet, checkpoint, root)
    if attempt_id is None or write_paths is None or entries is None:
        return _decision("blocked", "ledger_or_checkpoint_invalid")
    accepted, resume, checkpoint_error = _checkpoint_prefix(ledger, packet, checkpoint, root, write_paths, entries)
    if checkpoint_error == "started_unknown":
        return _decision("manual_reconciliation", "started_unknown_step", accepted_steps=accepted or [])
    if checkpoint_error:
        return _decision("blocked", checkpoint_error)
    if resume is None:
        return _decision("route_to_acceptance", "all_steps_accepted", accepted_steps=accepted or [])
    if status == "prepared":
        if accepted:
            return _decision("blocked", "prepared_attempt_has_accepted_side_effects")
        return _decision("dispatch_existing_attempt", "packet_revalidated")
    if status == "running":
        if worker_stopped_attestation is None:
            return _decision("suspected_stale", "stop_attestation_required", accepted_steps=accepted or [])
        if not _attestation(worker_stopped_attestation, packet["packet_id"], attempt_id):
            return _decision("blocked", "stop_attestation_invalid")
    if status in ("blocked", "failed", "cancelled"):
        if not _attestation(worker_stopped_attestation, packet["packet_id"], attempt_id):
            return _decision("blocked", "stop_attestation_required")
        if not _issue_resolution(issue_resolution, packet["packet_id"], attempt_id):
            return _decision("blocked", "issue_resolution_required")
    if (not isinstance(new_attempt_id, str) or not _ID.fullmatch(new_attempt_id)
            or new_attempt_id == attempt_id or not isinstance(prior_attempt_ids, (list, tuple, set))
            or any(not isinstance(item, str) for item in prior_attempt_ids)
            or new_attempt_id in prior_attempt_ids):
        return _decision("blocked", "new_attempt_identity_invalid")
    return _decision("retry_proposed", "accepted_prefix_verified",
                     retry_plan={"packet_id": packet["packet_id"], "packet_sha256": packet_sha256,
                                 "attempt_id": new_attempt_id, "supersedes_attempt_id": attempt_id,
                                 "accepted_steps": accepted or [], "resume_step": resume})


def _read_json(path: str) -> Any:
    with Path(path).open(encoding="utf-8") as stream:
        return json.load(stream)


def _main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    inspect = sub.add_parser("inspect")
    inspect.add_argument("ledger_json")
    inspect.add_argument("packet_json")
    inspect.add_argument("checkpoint_json")
    inspect.add_argument("--repo", required=True)
    try:
        args = parser.parse_args(argv[1:])
        ledger = _read_json(args.ledger_json)
        packet_bytes = Path(args.packet_json).read_bytes()
        packet = json.loads(packet_bytes.decode("utf-8"))
        checkpoint = _read_json(args.checkpoint_json)
    except (OSError, UnicodeError, json.JSONDecodeError, SystemExit):
        print(json.dumps(_decision("blocked", "input_unavailable")))
        return 1
    decision = inspect_attempt(ledger, packet, checkpoint, hashlib.sha256(packet_bytes).hexdigest(), args.repo)
    print(json.dumps(decision, sort_keys=True))
    if decision["decision"] in ("blocked", "suspected_stale", "manual_reconciliation", "rebind_required"):
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(_main(sys.argv))
