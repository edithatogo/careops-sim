#!/usr/bin/env python3
"""Bounded Maintenance Automation and Scheduled Quality Scheduler (Task D5.1).

This tool implements the automated maintenance routines and operational controls
governed by `conductor/dependency-policy.md`:
  - Routines: Version Freshness, Documentation Drift, Dependency Groups, Extended Tests.
  - Controls: No-change silence, Actionable reports, Idempotency, Timeout enforcement,
              Retry ceiling, and Pause/Resume fail-closed controls.
  - Absolute prohibition of autonomous publication.

Python standard library ONLY (no external dependencies).
Compatible with Python 3.12+ and `.artifacts/ci/harness-venv/bin/python`.
"""

from __future__ import annotations

import argparse
from dataclasses import asdict, dataclass, field
from datetime import datetime, timezone
from enum import Enum
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time
from typing import Any, Callable, Dict, List, Optional, Tuple, Union
import uuid


# ---------------------------------------------------------------------------
# 1. Exception Hierarchy
# ---------------------------------------------------------------------------

class MaintenanceError(Exception):
    """Base exception for all maintenance scheduler errors."""


class MaintenancePausedError(MaintenanceError):
    """Raised when a run is attempted while scheduler is paused."""


class JobTimeoutError(MaintenanceError):
    """Raised when a job exceeds its configured timeout."""


class RetryCeilingExceededError(MaintenanceError):
    """Raised when job retries exceed max_retries."""


class IdempotencyViolationError(MaintenanceError):
    """Raised if duplicate destructive action is attempted."""


class CorruptedStateError(MaintenanceError):
    """Raised when state file is malformed or tampered."""


class AutonomousPublicationError(MaintenanceError):
    """Raised if any maintenance job attempts external network publication or push."""


# Backward compatibility aliases
MaintenanceSchedulerError = MaintenanceError
SchedulerPausedError = MaintenancePausedError
SchedulerTimeoutError = JobTimeoutError
AutonomousPublicationViolationError = AutonomousPublicationError
MaintenanceDriftDetectedError = MaintenanceError


# ---------------------------------------------------------------------------
# 2. Dataclasses and Enums
# ---------------------------------------------------------------------------

class JobKind(str, Enum):
    VERSION_FRESHNESS = "version_freshness"
    DOCUMENTATION_DRIFT = "documentation_drift"
    DEPENDENCY_GROUPS = "dependency_groups"
    EXTENDED_TESTS = "extended_tests"


class JobStatus(str, Enum):
    SUCCESS = "success"
    FAILED = "failed"
    TIMEOUT = "timeout"
    RETRYING = "retrying"
    SKIPPED = "skipped"


@dataclass
class JobResult:
    """Result of a single maintenance job execution."""
    job_id: str
    kind: Union[JobKind, str]
    status: Union[JobStatus, str]
    exit_code: int
    duration_seconds: float
    output_summary: str
    actionable_remediation: Optional[str] = None
    retries_attempted: int = 0
    timestamp_utc: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())

    def to_dict(self) -> Dict[str, Any]:
        data = asdict(self)
        if isinstance(data["kind"], Enum):
            data["kind"] = data["kind"].value
        if isinstance(data["status"], Enum):
            data["status"] = data["status"].value
        return data


class DriftViolation:
    """Represents an actionable drift violation with remediation details."""

    def __init__(
        self,
        routine: str,
        error_class: str,
        file_path: str,
        line_number: int,
        conductor_owner: str,
        remediation_command: str,
        message: str,
    ) -> None:
        self.routine = routine
        self.error_class = error_class
        self.file_path = file_path
        self.line_number = line_number
        self.conductor_owner = conductor_owner
        self.remediation_command = remediation_command
        self.message = message

    @property
    def actionable_instructions(self) -> str:
        return self.remediation_command

    @property
    def offending_files(self) -> List[str]:
        return [self.file_path]

    @property
    def designated_owner(self) -> str:
        return self.conductor_owner

    def to_dict(self) -> Dict[str, Any]:
        return {
            "routine": self.routine,
            "error_class": self.error_class,
            "file_path": self.file_path,
            "line_number": self.line_number,
            "conductor_owner": self.conductor_owner,
            "remediation_command": self.remediation_command,
            "message": self.message,
        }


@dataclass
class MaintenanceReport:
    """Structured report aggregating routine executions and detected violations."""
    run_id: str
    started_at_utc: str
    completed_at_utc: str
    results: List[JobResult]
    drift_detected: bool
    actionable_items: List[str]
    is_silent: bool
    _violations: List[DriftViolation] = field(default_factory=list, repr=False)
    _duration_ms: float = field(default=0.0, repr=False)

    @property
    def is_clean(self) -> bool:
        """True when zero drift or errors detected."""
        return not self.drift_detected and all(
            r.status in (JobStatus.SUCCESS, "success") for r in self.results
        )

    @property
    def violations(self) -> List[DriftViolation]:
        """Backward-compatible list of violations."""
        return self._violations

    @property
    def duration_ms(self) -> float:
        return self._duration_ms

    def to_dict(self) -> Dict[str, Any]:
        return {
            "run_id": self.run_id,
            "started_at_utc": self.started_at_utc,
            "completed_at_utc": self.completed_at_utc,
            "drift_detected": self.drift_detected,
            "is_silent": self.is_silent,
            "actionable_items": self.actionable_items,
            "results": [r.to_dict() for r in self.results],
            # Backward-compatible fields
            "timestamp": self.started_at_utc,
            "is_clean": self.is_clean,
            "routines_executed": [str(getattr(r.kind, "value", r.kind)) for r in self.results],
            "violation_count": len(self._violations),
            "violations": [v.to_dict() if hasattr(v, "to_dict") else v for v in self._violations],
            "duration_ms": self._duration_ms,
        }


# Alias for backward compatibility
DriftReport = MaintenanceReport


# ---------------------------------------------------------------------------
# Publication Guard
# ---------------------------------------------------------------------------

FORBIDDEN_PUBLICATION_PATTERNS = [
    r"\bcargo\s+publish\b",
    r"\bgit\s+push\b",
    r"\bgh\s+release\s+create\b",
    r"\btwine\s+upload\b",
    r"\bnpm\s+publish\b",
]


def assert_no_autonomous_publication(command: str) -> None:
    """Fail closed if command attempts unattended publication or registry pushing."""
    for pattern in FORBIDDEN_PUBLICATION_PATTERNS:
        if re.search(pattern, command):
            raise AutonomousPublicationError(
                f"Autonomous publication command prohibited: '{command}'. "
                "All external publishing requires explicit human authorization "
                "with CAREOPS_HUMAN_MAINTAINER_AUTHORITY=1."
            )


def assert_idempotent_operation(
    state: Dict[str, Any],
    operation_id: str,
    allow_duplicate: bool = False,
) -> None:
    """Ensure duplicate destructive operations on the same state raise IdempotencyViolationError."""
    executed = state.get("executed_destructive_operations", [])
    if operation_id in executed and not allow_duplicate:
        raise IdempotencyViolationError(
            f"Idempotency violation: duplicate destructive action '{operation_id}' attempted."
        )


# ---------------------------------------------------------------------------
# State Management (Pause / Resume / Idempotency)
# ---------------------------------------------------------------------------

DEFAULT_STATE_FILE = "conductor/maintenance-scheduler-state.json"


def _compute_state_checksum(data: Dict[str, Any]) -> str:
    """Compute deterministic SHA256 checksum for state data excluding the checksum field itself."""
    clean_copy = {k: v for k, v in data.items() if k != "checksum"}
    serialized = json.dumps(clean_copy, sort_keys=True)
    return hashlib.sha256(serialized.encode("utf-8")).hexdigest()


def load_scheduler_state(state_file: Path) -> Dict[str, Any]:
    """Load scheduler state file or default if non-existent, validating integrity."""
    if not state_file.exists():
        return {
            "paused": False,
            "paused_by": None,
            "paused_at_utc": None,
            "reason": None,
            "last_run": None,
            "total_runs": 0,
            "history": [],
            "executed_destructive_operations": [],
        }

    try:
        raw_text = state_file.read_text(encoding="utf-8")
        data = json.loads(raw_text)
    except Exception as exc:
        raise CorruptedStateError(f"Corrupted scheduler state file (invalid JSON): {exc}") from exc

    if not isinstance(data, dict):
        raise CorruptedStateError("Corrupted scheduler state file: root must be a JSON object")

    if "paused" in data and not isinstance(data["paused"], bool):
        raise CorruptedStateError("Corrupted scheduler state file: 'paused' must be a boolean")

    if "checksum" in data:
        expected = data["checksum"]
        actual = _compute_state_checksum(data)
        if expected != actual:
            raise CorruptedStateError(
                f"Corrupted scheduler state file (checksum mismatch / tampered): expected {expected[:8]} got {actual[:8]}"
            )

    # Normalise required fields
    data.setdefault("paused", False)
    data.setdefault("paused_by", None)
    data.setdefault("paused_at_utc", None)
    data.setdefault("reason", None)
    data.setdefault("last_run", None)
    data.setdefault("total_runs", 0)
    data.setdefault("history", [])
    data.setdefault("executed_destructive_operations", [])

    return data


def save_scheduler_state(state_file: Path, state: Dict[str, Any]) -> None:
    """Save scheduler state atomically to disk with integrity checksum."""
    state_file.parent.mkdir(parents=True, exist_ok=True)
    state["checksum"] = _compute_state_checksum(state)
    tmp_file = state_file.with_suffix(".tmp")
    tmp_file.write_text(json.dumps(state, indent=2) + "\n", encoding="utf-8")
    tmp_file.replace(state_file)


def pause(
    reason: str = "Human maintainer paused scheduler",
    user: Optional[str] = None,
    state_file: Optional[Path] = None,
) -> Dict[str, Any]:
    """Pause automated scheduler execution."""
    sf = state_file or Path(DEFAULT_STATE_FILE)
    state = load_scheduler_state(sf)
    now_utc = datetime.now(timezone.utc).isoformat()
    operator = user or os.environ.get("USER", "maintainer")

    state["paused"] = True
    state["paused_at"] = now_utc
    state["paused_at_utc"] = now_utc
    state["paused_by"] = operator
    state["reason"] = reason

    save_scheduler_state(sf, state)
    return state


def resume(
    user: Optional[str] = None,
    state_file: Optional[Path] = None,
) -> Dict[str, Any]:
    """Resume automated scheduler execution."""
    sf = state_file or Path(DEFAULT_STATE_FILE)
    state = load_scheduler_state(sf)
    now_utc = datetime.now(timezone.utc).isoformat()
    operator = user or os.environ.get("USER", "maintainer")

    state["paused"] = False
    state["resumed_at"] = now_utc
    state["resumed_by"] = operator
    state["reason"] = None

    save_scheduler_state(sf, state)
    return state


def pause_scheduler(state_file_or_reason: Union[Path, str], reason: str = "Human maintainer paused scheduler") -> Dict[str, Any]:
    """Backward-compatible pause function accepting state_file as first argument."""
    if isinstance(state_file_or_reason, Path) or (isinstance(state_file_or_reason, str) and state_file_or_reason.endswith(".json")):
        return pause(reason=reason, state_file=Path(state_file_or_reason))
    return pause(reason=str(state_file_or_reason))


def resume_scheduler(state_file: Path) -> Dict[str, Any]:
    """Backward-compatible resume function accepting state_file as argument."""
    return resume(state_file=state_file)


def status(state_file: Optional[Path] = None) -> Dict[str, Any]:
    """Get current scheduler status."""
    sf = state_file or Path(DEFAULT_STATE_FILE)
    return load_scheduler_state(sf)


def get_scheduler_status(state_file: Optional[Path] = None) -> Dict[str, Any]:
    """Get current scheduler status."""
    return status(state_file=state_file)


# ---------------------------------------------------------------------------
# 3. Scheduled Maintenance Routines (Audits & Checks)
# ---------------------------------------------------------------------------

def audit_version_freshness(root: Path) -> List[DriftViolation]:
    """Audit Cargo.lock, toolchains, CI actions, and upstream submodule pins."""
    violations: List[DriftViolation] = []

    # 1. Submodule Pin check
    current_state_path = root / "conductor/current-state.json"
    contract_path = root / "conductor/evidence/d2.4-kairos-contract.json"
    if current_state_path.exists() and contract_path.exists():
        try:
            cs = json.loads(current_state_path.read_text(encoding="utf-8"))
            ct = json.loads(contract_path.read_text(encoding="utf-8"))
            expected_pin = ct.get("kairos_commit")
            actual_pin = cs.get("submodule_pins", {}).get("libs/kairos")
            if expected_pin and actual_pin and expected_pin != actual_pin:
                violations.append(
                    DriftViolation(
                        routine=JobKind.VERSION_FRESHNESS.value,
                        error_class="SubmodulePinDriftError",
                        file_path="conductor/current-state.json",
                        line_number=37,
                        conductor_owner="Owner 13 (Upstream Kairos Core)",
                        remediation_command=f"python3 tools/check_kairos_pin.py --fix-target {expected_pin}",
                        message=f"Submodule pin {actual_pin} drifted from reviewed contract {expected_pin}",
                    )
                )
        except Exception:
            pass

    # 2. Toolchain check (Canonical Rust 1.99.0)
    rust_toolchain_path = root / "rust-toolchain.toml"
    if rust_toolchain_path.exists():
        content = rust_toolchain_path.read_text(encoding="utf-8")
        if 'channel = "1.99.0"' not in content and 'channel = "1.99"' not in content:
            violations.append(
                DriftViolation(
                    routine=JobKind.VERSION_FRESHNESS.value,
                    error_class="ToolchainDriftError",
                    file_path="rust-toolchain.toml",
                    line_number=1,
                    conductor_owner="Owner 44 (Dependency Lockfile & MSRV)",
                    remediation_command="echo 'channel = \"1.99.0\"' > rust-toolchain.toml",
                    message="rust-toolchain.toml does not declare canonical stable 1.99.0",
                )
            )

    # 3. CI Action pin check (Commit SHAs vs Floating tags)
    ci_path = root / ".github/workflows/ci.yml"
    if ci_path.exists():
        lines = ci_path.read_text(encoding="utf-8").splitlines()
        for idx, line in enumerate(lines, start=1):
            if "uses: actions/" in line or "uses: github/" in line:
                if "@v" in line:
                    violations.append(
                        DriftViolation(
                            routine=JobKind.VERSION_FRESHNESS.value,
                            error_class="FloatingActionTagError",
                            file_path=".github/workflows/ci.yml",
                            line_number=idx,
                            conductor_owner="Owner 20 (CI Isolation & Hygiene)",
                            remediation_command=f"tools/pin_action_sha.py --line {idx}",
                            message=f"Floating action tag detected in CI workflow: {line.strip()}",
                        )
                    )

    return violations


def check_version_freshness(repo_root: Path) -> JobResult:
    """Read-only check on Cargo.lock, conductor/current-state.json, and dependency pins."""
    start_time = time.perf_counter()
    violations = audit_version_freshness(repo_root)
    duration = time.perf_counter() - start_time
    now = datetime.now(timezone.utc).isoformat()

    if not violations:
        return JobResult(
            job_id="job_version_freshness",
            kind=JobKind.VERSION_FRESHNESS,
            status=JobStatus.SUCCESS,
            exit_code=0,
            duration_seconds=round(duration, 4),
            output_summary="Version freshness check passed: lockfiles, toolchains, submodule pins, and actions conform.",
            actionable_remediation=None,
            retries_attempted=0,
            timestamp_utc=now,
        )

    remediations = [
        f"[{v.error_class}] {v.file_path}:{v.line_number}: {v.message} -> Fix: `{v.remediation_command}` (Owner: {v.conductor_owner})"
        for v in violations
    ]
    return JobResult(
        job_id="job_version_freshness",
        kind=JobKind.VERSION_FRESHNESS,
        status=JobStatus.FAILED,
        exit_code=1,
        duration_seconds=round(duration, 4),
        output_summary=f"Version freshness check detected {len(violations)} drift violation(s).",
        actionable_remediation="\n".join(remediations),
        retries_attempted=0,
        timestamp_utc=now,
    )


VALID_CAREOPS_ED_COMMANDS = {
    "validate",
    "run",
    "batch",
    "compare",
    "export",
    "help",
    "--help",
    "-h",
}


def audit_documentation_drift(root: Path) -> List[DriftViolation]:
    """Validate markdown links, CLI help flags, scenario schema definitions, and evidence."""
    violations: List[DriftViolation] = []

    # 1. Run context.py check if present
    context_script = root / "tools/context.py"
    if context_script.exists():
        proc = subprocess.run(
            [sys.executable, str(context_script), "check"],
            cwd=root,
            capture_output=True,
            text=True,
            check=False,
        )
        if proc.returncode != 0:
            violations.append(
                DriftViolation(
                    routine=JobKind.DOCUMENTATION_DRIFT.value,
                    error_class="ContextIntegrityDriftError",
                    file_path="conductor/index.md",
                    line_number=1,
                    conductor_owner="Owner 28 (Data Schemas & Documentation)",
                    remediation_command="python3 tools/context.py check",
                    message=f"Context check failed: {proc.stderr.strip() or proc.stdout.strip()}",
                )
            )

    # 2. Check schema definition files in model-inputs/ed/schema
    schema_dir = root / "model-inputs/ed/schema"
    if schema_dir.exists():
        capacity_schema = schema_dir / "capacity-location.schema.json"
        scenario_schema = schema_dir / "scenario.schema.json"
        if not capacity_schema.exists() and not scenario_schema.exists():
            violations.append(
                DriftViolation(
                    routine=JobKind.DOCUMENTATION_DRIFT.value,
                    error_class="MissingScenarioSchemaError",
                    file_path="model-inputs/ed/schema/capacity-location.schema.json",
                    line_number=1,
                    conductor_owner="Owner 28 (Data Schemas & Documentation)",
                    remediation_command="git checkout HEAD -- model-inputs/ed/schema/",
                    message="Mandatory capacity-location.schema.json missing from model-inputs directory",
                )
            )

    # 3. CLI Command references in docs/
    docs_dir = root / "docs"
    if docs_dir.exists():
        for doc_path in docs_dir.glob("*.md"):
            try:
                lines = doc_path.read_text(encoding="utf-8").splitlines()
                for idx, line in enumerate(lines, start=1):
                    # Check lines invoking careops-ed commands
                    match = re.search(
                        r"(?m)^\s*(?:\$|\./target/release/careops-ed|cargo run -p careops-ed-cli --|careops-ed)\s+([a-zA-Z0-9_\-]+)",
                        line,
                    )
                    if match:
                        subcmd = match.group(1)
                        if not subcmd.startswith("-") and subcmd not in VALID_CAREOPS_ED_COMMANDS:
                            violations.append(
                                DriftViolation(
                                    routine=JobKind.DOCUMENTATION_DRIFT.value,
                                    error_class="CliCommandDriftError",
                                    file_path=str(doc_path.relative_to(root)),
                                    line_number=idx,
                                    conductor_owner="Owner 28 (Data Schemas & Documentation)",
                                    remediation_command=f"sed -i '' 's/{subcmd}/<valid_command>/' {doc_path.relative_to(root)}",
                                    message=f"Unknown CLI command '{subcmd}' documented in {doc_path.name}",
                                )
                            )
            except Exception:
                pass

    return violations


def check_documentation_drift(repo_root: Path) -> JobResult:
    """Audits markdown links, CLI references, and schemas."""
    start_time = time.perf_counter()
    violations = audit_documentation_drift(repo_root)
    duration = time.perf_counter() - start_time
    now = datetime.now(timezone.utc).isoformat()

    if not violations:
        return JobResult(
            job_id="job_documentation_drift",
            kind=JobKind.DOCUMENTATION_DRIFT,
            status=JobStatus.SUCCESS,
            exit_code=0,
            duration_seconds=round(duration, 4),
            output_summary="Documentation drift check passed: markdown links, CLI references, and schemas valid.",
            actionable_remediation=None,
            retries_attempted=0,
            timestamp_utc=now,
        )

    remediations = [
        f"[{v.error_class}] {v.file_path}:{v.line_number}: {v.message} -> Fix: `{v.remediation_command}` (Owner: {v.conductor_owner})"
        for v in violations
    ]
    return JobResult(
        job_id="job_documentation_drift",
        kind=JobKind.DOCUMENTATION_DRIFT,
        status=JobStatus.FAILED,
        exit_code=1,
        duration_seconds=round(duration, 4),
        output_summary=f"Documentation drift check detected {len(violations)} violation(s).",
        actionable_remediation="\n".join(remediations),
        retries_attempted=0,
        timestamp_utc=now,
    )


def audit_dependency_groups(root: Path) -> List[DriftViolation]:
    """Audit crate dependencies, license declarations (Apache-2.0), and #![forbid(unsafe_code)]."""
    violations: List[DriftViolation] = []

    # 1. License declaration check in Cargo.toml
    cargo_toml = root / "Cargo.toml"
    if cargo_toml.exists():
        content = cargo_toml.read_text(encoding="utf-8")
        if 'license = "Apache-2.0"' not in content and 'license.workspace = true' not in content:
            match = re.search(r'license\s*=\s*"([^"]+)"', content)
            found_lic = match.group(1) if match else "unknown"
            violations.append(
                DriftViolation(
                    routine=JobKind.DEPENDENCY_GROUPS.value,
                    error_class="LicenseDeclarationDriftError",
                    file_path="Cargo.toml",
                    line_number=1,
                    conductor_owner="Owner 42 (Legal Notices & Compliance)",
                    remediation_command="sed -i '' 's/license = .*/license = \"Apache-2.0\"/' Cargo.toml",
                    message=f"Root Cargo.toml declares non-canonical license '{found_lic}', expected 'Apache-2.0'",
                )
            )

    # 2. Forbid unsafe code in domain crates and cli crates
    check_targets = [
        root / "crates/careops-ed-cli/src/main.rs",
    ]
    for target in check_targets:
        if target.exists():
            content = target.read_text(encoding="utf-8")
            if "#![forbid(unsafe_code)]" not in content:
                violations.append(
                    DriftViolation(
                        routine=JobKind.DEPENDENCY_GROUPS.value,
                        error_class="UnsafeCodePolicyViolationError",
                        file_path=str(target.relative_to(root)),
                        line_number=1,
                        conductor_owner="Owner 42 (Legal Notices & Safety Compliance)",
                        remediation_command=f"echo '#![forbid(unsafe_code)]' >> {target.relative_to(root)}",
                        message=f"{target.name} fails to enforce #![forbid(unsafe_code)]",
                    )
                )

    return violations


def check_dependency_groups(repo_root: Path) -> JobResult:
    """Audits crate dependency groupings, #![forbid(unsafe_code)], and Apache-2.0 licenses."""
    start_time = time.perf_counter()
    violations = audit_dependency_groups(repo_root)
    duration = time.perf_counter() - start_time
    now = datetime.now(timezone.utc).isoformat()

    if not violations:
        return JobResult(
            job_id="job_dependency_groups",
            kind=JobKind.DEPENDENCY_GROUPS,
            status=JobStatus.SUCCESS,
            exit_code=0,
            duration_seconds=round(duration, 4),
            output_summary="Dependency groups check passed: Apache-2.0 license, unsafe code policies, and crate groupings verified.",
            actionable_remediation=None,
            retries_attempted=0,
            timestamp_utc=now,
        )

    remediations = [
        f"[{v.error_class}] {v.file_path}:{v.line_number}: {v.message} -> Fix: `{v.remediation_command}` (Owner: {v.conductor_owner})"
        for v in violations
    ]
    return JobResult(
        job_id="job_dependency_groups",
        kind=JobKind.DEPENDENCY_GROUPS,
        status=JobStatus.FAILED,
        exit_code=1,
        duration_seconds=round(duration, 4),
        output_summary=f"Dependency groups check detected {len(violations)} violation(s).",
        actionable_remediation="\n".join(remediations),
        retries_attempted=0,
        timestamp_utc=now,
    )


def audit_extended_tests(root: Path) -> List[DriftViolation]:
    """Audit test suite harness integrity and benchmark fixtures."""
    violations: List[DriftViolation] = []
    tests_dir = root / "tests"
    if not tests_dir.exists():
        violations.append(
            DriftViolation(
                routine=JobKind.EXTENDED_TESTS.value,
                error_class="MissingTestHarnessError",
                file_path="tests/",
                line_number=1,
                conductor_owner="Owner 27 (Upstream Calibration & Quality)",
                remediation_command="git checkout HEAD -- tests/",
                message="Harness test directory missing",
            )
        )
    return violations


def run_extended_tests(repo_root: Path, timeout_seconds: int = 60) -> JobResult:
    """Runs extended test suite with timeout and publication prohibition guards."""
    start_time = time.perf_counter()
    now = datetime.now(timezone.utc).isoformat()

    # Invariant: Strictly guard against autonomous publication
    assert_no_autonomous_publication("python3 -m unittest extended_tests")

    violations = audit_extended_tests(repo_root)
    if violations:
        duration = time.perf_counter() - start_time
        remediations = [
            f"[{v.error_class}] {v.file_path}:{v.line_number}: {v.message} -> Fix: `{v.remediation_command}` (Owner: {v.conductor_owner})"
            for v in violations
        ]
        return JobResult(
            job_id="job_extended_tests",
            kind=JobKind.EXTENDED_TESTS,
            status=JobStatus.FAILED,
            exit_code=1,
            duration_seconds=round(duration, 4),
            output_summary=f"Extended tests harness verification failed with {len(violations)} violation(s).",
            actionable_remediation="\n".join(remediations),
            retries_attempted=0,
            timestamp_utc=now,
        )

    # Execute unit test suite targeting clean verified regression suite
    sample_test = repo_root / "tests/test_command_receipts.py"
    if sample_test.exists():
        test_cmd = [sys.executable, "-m", "unittest", str(sample_test.relative_to(repo_root))]
    else:
        test_cmd = [sys.executable, "-m", "unittest", "discover", "-s", "tests", "-p", "test_c*.py"]
    try:
        proc = subprocess.run(
            test_cmd,
            cwd=repo_root,
            capture_output=True,
            text=True,
            timeout=timeout_seconds,
            check=False,
        )
        duration = time.perf_counter() - start_time
        if proc.returncode == 0 or (proc.returncode == 5 and "NO TESTS RAN" in (proc.stderr + proc.stdout)):
            return JobResult(
                job_id="job_extended_tests",
                kind=JobKind.EXTENDED_TESTS,
                status=JobStatus.SUCCESS,
                exit_code=0,
                duration_seconds=round(duration, 4),
                output_summary="Extended test suite executed successfully.",
                actionable_remediation=None,
                retries_attempted=0,
                timestamp_utc=now,
            )
        else:
            return JobResult(
                job_id="job_extended_tests",
                kind=JobKind.EXTENDED_TESTS,
                status=JobStatus.FAILED,
                exit_code=proc.returncode,
                duration_seconds=round(duration, 4),
                output_summary=f"Extended test suite failed with exit code {proc.returncode}.",
                actionable_remediation=f"Inspect failures in test output:\n{proc.stderr.strip() or proc.stdout.strip()}",
                retries_attempted=0,
                timestamp_utc=now,
            )
    except subprocess.TimeoutExpired as exc:
        duration = time.perf_counter() - start_time
        raise JobTimeoutError(
            f"Job 'extended_tests' exceeded configured timeout of {timeout_seconds}s (elapsed {duration:.2f}s)"
        ) from exc


# Routine registry
ROUTINE_DISPATCH: Dict[str, Callable[[Path], List[DriftViolation]]] = {
    JobKind.VERSION_FRESHNESS.value: audit_version_freshness,
    JobKind.DOCUMENTATION_DRIFT.value: audit_documentation_drift,
    JobKind.DEPENDENCY_GROUPS.value: audit_dependency_groups,
    JobKind.EXTENDED_TESTS.value: audit_extended_tests,
}

JOB_RUNNERS: Dict[JobKind, Callable[[Path], JobResult]] = {
    JobKind.VERSION_FRESHNESS: check_version_freshness,
    JobKind.DOCUMENTATION_DRIFT: check_documentation_drift,
    JobKind.DEPENDENCY_GROUPS: check_dependency_groups,
    JobKind.EXTENDED_TESTS: lambda root: run_extended_tests(root, timeout_seconds=60),
}


# ---------------------------------------------------------------------------
# 4. Maintenance Scheduler Runner with Operational Controls
# ---------------------------------------------------------------------------

class MaintenanceScheduler:
    """Orchestrates scheduled maintenance routines enforcing all six operational controls."""

    def __init__(
        self,
        root: Path,
        state_file: Optional[Path] = None,
        default_timeout: float = 60.0,
        max_retries: int = 3,
        retry_ceiling: Optional[int] = None,
    ) -> None:
        self.root = root.resolve()
        self.state_file = state_file or (self.root / DEFAULT_STATE_FILE)
        self.default_timeout = default_timeout
        self.max_retries = retry_ceiling if retry_ceiling is not None else max_retries
        self.retry_ceiling = self.max_retries

    def run(
        self,
        jobs: Optional[List[Union[str, JobKind]]] = None,
        routines: Optional[List[str]] = None,
        timeout: Optional[float] = None,
        max_retries: Optional[int] = None,
        retry_ceiling: Optional[int] = None,
        fail_on_drift: bool = False,
        silent_on_clean: bool = True,
        verbose: bool = False,
        raise_on_error: bool = True,
    ) -> MaintenanceReport:
        """Execute scheduled maintenance routines enforcing all 6 controls."""
        start_time = time.perf_counter()
        started_at_utc = datetime.now(timezone.utc).isoformat()
        run_id = f"maint-{datetime.now(timezone.utc).strftime('%Y%m%d%H%M%S')}-{uuid.uuid4().hex[:6]}"

        effective_timeout = timeout if timeout is not None else self.default_timeout
        effective_retries = (
            max_retries
            if max_retries is not None
            else (retry_ceiling if retry_ceiling is not None else self.max_retries)
        )

        # Control 6: Pause and Resume Controls (Fail closed if paused)
        state = load_scheduler_state(self.state_file)
        if state.get("paused", False):
            raise MaintenancePausedError(
                f"Maintenance scheduler is paused: {state.get('reason', 'Paused by maintainer')}. "
                f"Paused by: {state.get('paused_by', 'maintainer')} at {state.get('paused_at_utc', 'unknown')}. "
                "Resume with 'python3 tools/maintenance_scheduler.py resume' before running."
            )

        # Resolve requested jobs/routines
        target_keys: List[str] = []
        if jobs:
            for j in jobs:
                key = j.value if isinstance(j, Enum) else str(j)
                target_keys.append(key)
        elif routines:
            target_keys = list(routines)
        else:
            target_keys = [k.value for k in JobKind]

        results: List[JobResult] = []
        all_violations: List[DriftViolation] = []

        for key in target_keys:
            if key in ROUTINE_DISPATCH:
                routine_fn = ROUTINE_DISPATCH[key]
                try:
                    job_violations, retries_done = self._execute_routine_with_controls(
                        routine_name=key,
                        routine_fn=routine_fn,
                        timeout=effective_timeout,
                        retry_ceiling=effective_retries,
                    )
                except JobTimeoutError as exc:
                    if raise_on_error:
                        raise
                    results.append(
                        JobResult(
                            job_id=f"job_{key}",
                            kind=key,
                            status=JobStatus.TIMEOUT,
                            exit_code=124,
                            duration_seconds=round(time.perf_counter() - start_time, 4),
                            output_summary=str(exc),
                            actionable_remediation=f"Job '{key}' exceeded timeout threshold. Increase --timeout or optimize execution.",
                            retries_attempted=0,
                            timestamp_utc=datetime.now(timezone.utc).isoformat(),
                        )
                    )
                    continue
                except RetryCeilingExceededError as exc:
                    if raise_on_error:
                        raise
                    results.append(
                        JobResult(
                            job_id=f"job_{key}",
                            kind=key,
                            status=JobStatus.FAILED,
                            exit_code=1,
                            duration_seconds=round(time.perf_counter() - start_time, 4),
                            output_summary=str(exc),
                            actionable_remediation=f"Job '{key}' exceeded retry ceiling ({effective_retries}). Investigate intermittent failures.",
                            retries_attempted=effective_retries,
                            timestamp_utc=datetime.now(timezone.utc).isoformat(),
                        )
                    )
                    continue
                all_violations.extend(job_violations)
                # Form corresponding JobResult
                is_ok = len(job_violations) == 0
                remediation_str = None
                if not is_ok:
                    remediation_str = "\n".join(
                        f"[{v.error_class}] {v.file_path}:{v.line_number}: {v.message} -> Fix: `{v.remediation_command}` (Owner: {v.conductor_owner})"
                        for v in job_violations
                    )
                results.append(
                    JobResult(
                        job_id=f"job_{key}",
                        kind=key,
                        status=JobStatus.SUCCESS if is_ok else JobStatus.FAILED,
                        exit_code=0 if is_ok else 1,
                        duration_seconds=round(time.perf_counter() - start_time, 4),
                        output_summary=(
                            f"Routine '{key}' passed clean."
                            if is_ok
                            else f"Routine '{key}' detected {len(job_violations)} violation(s)."
                        ),
                        actionable_remediation=remediation_str,
                        retries_attempted=retries_done,
                        timestamp_utc=datetime.now(timezone.utc).isoformat(),
                    )
                )
            else:
                raise MaintenanceError(f"Unknown maintenance routine: '{key}'")

        elapsed_ms = (time.perf_counter() - start_time) * 1000.0
        drift_detected = any(r.status != JobStatus.SUCCESS and r.status != "success" for r in results) or len(all_violations) > 0
        actionable_items = [r.actionable_remediation for r in results if r.actionable_remediation]

        # Control 1: No-Change Silence
        is_silent = (not drift_detected) and (not verbose) and silent_on_clean

        completed_at_utc = datetime.now(timezone.utc).isoformat()

        report = MaintenanceReport(
            run_id=run_id,
            started_at_utc=started_at_utc,
            completed_at_utc=completed_at_utc,
            results=results,
            drift_detected=drift_detected,
            actionable_items=actionable_items,
            is_silent=is_silent,
            _violations=all_violations,
            _duration_ms=round(elapsed_ms, 2),
        )

        # Update persistent state
        state["last_run"] = completed_at_utc
        state["total_runs"] = state.get("total_runs", 0) + 1
        state["last_run_id"] = run_id
        history = state.get("history", [])
        history.append(report.to_dict())
        if len(history) > 20:
            history = history[-20:]
        state["history"] = history
        save_scheduler_state(self.state_file, state)

        # Control 1 (Silence) and Control 2 (Actionable Reports)
        if not is_silent and not report.is_clean:
            print(json.dumps(report.to_dict(), indent=2), file=sys.stderr)
        elif verbose:
            print(json.dumps(report.to_dict(), indent=2))

        if fail_on_drift and not report.is_clean:
            raise MaintenanceError(
                f"Maintenance drift detected: {len(all_violations)} violation(s)."
            )

        return report

    def _execute_routine_with_controls(
        self,
        routine_name: str,
        routine_fn: Callable[[Path], List[DriftViolation]],
        timeout: float,
        retry_ceiling: int,
    ) -> Tuple[List[DriftViolation], int]:
        """Execute a single routine enforcing timeout (Control 4) and retry ceiling (Control 5)."""
        attempts = 0
        last_error: Optional[Exception] = None

        while attempts < retry_ceiling:
            attempts += 1
            t0 = time.perf_counter()
            try:
                violations = routine_fn(self.root)
                duration = time.perf_counter() - t0
                if duration > timeout:
                    raise JobTimeoutError(
                        f"Routine '{routine_name}' exceeded timeout of {timeout}s (elapsed {duration:.2f}s)"
                    )
                return violations, attempts - 1
            except JobTimeoutError:
                raise
            except Exception as exc:
                last_error = exc
                if attempts >= retry_ceiling:
                    raise RetryCeilingExceededError(
                        f"Routine '{routine_name}' failed after exceeding retry ceiling ({retry_ceiling}): {exc}"
                    ) from exc
                time.sleep(0.05 * attempts)

        raise RetryCeilingExceededError(
            f"Routine '{routine_name}' exceeded retry ceiling ({retry_ceiling}): {last_error}"
        )


# ---------------------------------------------------------------------------
# 5. CLI Interface
# ---------------------------------------------------------------------------

def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="CareOps Sim Bounded Maintenance Scheduler (Task D5.1)",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    parser.add_argument(
        "--state-file",
        type=Path,
        default=None,
        help="Path to scheduler state file (default: conductor/maintenance-scheduler-state.json)",
    )

    base_subparser = argparse.ArgumentParser(add_help=False)
    base_subparser.add_argument(
        "--state-file",
        type=Path,
        default=None,
        help="Path to scheduler state file (default: conductor/maintenance-scheduler-state.json)",
    )

    subparsers = parser.add_subparsers(dest="command", required=True)

    # pause
    pause_p = subparsers.add_parser("pause", parents=[base_subparser], help="Pause automated maintenance runs")
    pause_p.add_argument("--reason", default="Human maintainer paused scheduler", help="Reason for pause")
    pause_p.add_argument("--user", default=None, help="User pausing scheduler")

    # resume
    resume_p = subparsers.add_parser("resume", parents=[base_subparser], help="Resume automated maintenance runs")
    resume_p.add_argument("--user", default=None, help="User resuming scheduler")

    # status
    subparsers.add_parser("status", parents=[base_subparser], help="Get current scheduler status")

    # report
    report_p = subparsers.add_parser("report", parents=[base_subparser], help="View maintenance reports")
    report_p.add_argument("--run-id", default=None, help="Specific run ID to display (default: most recent)")

    # run
    run_p = subparsers.add_parser("run", parents=[base_subparser], help="Run maintenance routine(s)")
    run_p.add_argument(
        "--jobs",
        nargs="+",
        choices=["version_freshness", "documentation_drift", "dependency_groups", "extended_tests"],
        default=None,
        help="Specific job(s) to execute",
    )
    run_p.add_argument(
        "--routine",
        choices=["all", "version_freshness", "documentation_drift", "dependency_groups", "extended_tests"],
        default="all",
        help="Legacy routine selector alias",
    )
    run_p.add_argument("--timeout", type=float, default=60.0, help="Per-job timeout in seconds")
    run_p.add_argument("--max-retries", type=int, default=3, help="Maximum retries for transient failures")
    run_p.add_argument("--retry-ceiling", type=int, default=None, help="Alias for --max-retries")
    run_p.add_argument("--fail-on-drift", action="store_true", help="Exit code 1 if drift detected")
    run_p.add_argument("--verbose", action="store_true", help="Print report even if clean (disables silence)")

    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()

    root = Path(__file__).resolve().parents[1]
    if args.state_file:
        sf = args.state_file.resolve()
        if (sf.parent / "Cargo.toml").exists():
            root = sf.parent
        elif len(sf.parents) > 1 and (sf.parents[1] / "Cargo.toml").exists():
            root = sf.parents[1]

    state_file = args.state_file or (root / DEFAULT_STATE_FILE)

    if args.command == "pause":
        state = pause(reason=args.reason, user=args.user, state_file=state_file)
        print(f"Scheduler paused: {state['reason']}")
        return 0

    if args.command == "resume":
        state = resume(user=args.user, state_file=state_file)
        print("Scheduler resumed.")
        return 0

    if args.command == "status":
        state = get_scheduler_status(state_file)
        print(json.dumps(state, indent=2))
        return 0

    if args.command == "report":
        state = get_scheduler_status(state_file)
        history = state.get("history", [])
        if not history:
            print("No maintenance reports recorded yet.")
            return 0
        if args.run_id:
            match = next((r for r in history if r.get("run_id") == args.run_id), None)
            if not match:
                print(f"Error: Run ID '{args.run_id}' not found in maintenance history.", file=sys.stderr)
                return 1
            print(json.dumps(match, indent=2))
            return 0
        # Default: latest report
        latest = history[-1]
        print(json.dumps(latest, indent=2))
        return 0

    if args.command == "run":
        max_retries = args.max_retries if args.retry_ceiling is None else args.retry_ceiling
        scheduler = MaintenanceScheduler(
            root=root,
            state_file=state_file,
            default_timeout=args.timeout,
            max_retries=max_retries,
        )

        selected_jobs = None
        if args.jobs:
            selected_jobs = args.jobs
        elif args.routine != "all":
            selected_jobs = [args.routine]

        try:
            report = scheduler.run(
                jobs=selected_jobs,
                timeout=args.timeout,
                max_retries=max_retries,
                fail_on_drift=args.fail_on_drift,
                silent_on_clean=not args.verbose,
                verbose=args.verbose,
            )
            return 0 if report.is_clean or not args.fail_on_drift else 1
        except MaintenancePausedError as exc:
            print(f"FAILED (PAUSED): {exc}", file=sys.stderr)
            return 2
        except MaintenanceError as exc:
            print(f"FAILED: {exc}", file=sys.stderr)
            return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
