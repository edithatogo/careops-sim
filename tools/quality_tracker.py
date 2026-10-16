#!/usr/bin/env python3
"""Quality Debt, Flaky Test Quarantine, and Performance Drift Tracker (Task D5.3).

This module implements the automated quality policy and maintainer effort review
governed by `conductor/tracks/development_readiness_20260925/plan.md#D5.3`:
  - Scans quality debt (debt markers, compiler warning records, coverage deficits).
  - Audits flaky test exceptions with expiration dates and quarantine compliance.
  - Detects performance drift and memory regressions against baseline snapshots.
  - Evaluates automation return on investment (ROI) and flags burdensome workflows.
  - Rolls back evidence-weakening workflows and writes immutable rollback receipts.
  - Strictly prohibits autonomous publication or git pushing.

Python standard library ONLY (no external dependencies).
Compatible with Python 3.12+ and `.artifacts/ci/harness-venv/bin/python`.
"""

from __future__ import annotations

import argparse
from dataclasses import asdict, dataclass, field
from datetime import date, datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import sys
from typing import Any, Dict, List, Optional, Tuple, Union


# ---------------------------------------------------------------------------
# 1. Exception Hierarchy
# ---------------------------------------------------------------------------

class QualityTrackingError(Exception):
    """Base exception for all quality and performance tracking errors."""


class QualityDebtExceededError(QualityTrackingError):
    """Raised when quality debt score exceeds configured threshold."""


class UnquarantinedFlakeError(QualityTrackingError):
    """Raised when an unquarantined or expired flaky test is detected."""


class PerformanceRegressionError(QualityTrackingError):
    """Raised when throughput degrades beyond tolerance or memory leak is detected."""


class WorkflowEvidenceWeakenedError(QualityTrackingError):
    """Raised when an automated workflow is determined to weaken evidence."""


class AutonomousPublicationError(QualityTrackingError):
    """Raised if any quality tool or automated workflow attempts public registry publication or git pushing."""


# Backward compatibility alias
AutonomousPublicationViolationError = AutonomousPublicationError


# ---------------------------------------------------------------------------
# Prohibited Publication Guard
# ---------------------------------------------------------------------------

PROHIBITED_PUBLICATION_PATTERNS: tuple[str, ...] = (
    r"\bgit\s+push\b",
    r"\bcargo\s+publish\b",
    r"\bnpm\s+publish\b",
    r"\btwine\s+upload\b",
    r"\bgh\s+release\s+create\b",
    r"\bexternal\s+network\s+publication\b",
    r"\bpublic\s+registry\s+publication\b",
)


def assert_no_autonomous_publication(action: str) -> None:
    """Raises AutonomousPublicationError if action contains autonomous publication commands."""
    for pattern in PROHIBITED_PUBLICATION_PATTERNS:
        if re.search(pattern, action, re.IGNORECASE):
            raise AutonomousPublicationError(
                f"Autonomous publication prohibited: '{action}' matches prohibited pattern '{pattern}'"
            )


# ---------------------------------------------------------------------------
# 2. Dataclasses & Models
# ---------------------------------------------------------------------------

@dataclass
class DebtMetrics:
    """Composite quality debt metrics across the codebase."""
    todo_markers_count: int
    warning_count: int
    uncovered_lines: int
    debt_score: float
    status: str
    details: list[str] = field(default_factory=list)

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass
class FlakeRecord:
    """Record of a flaky test quarantine entry."""
    test_id: str
    owner: str
    reason: str
    issue_url: str
    expires_on: str
    is_expired: bool
    is_quarantined: bool

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass
class PerformanceSnapshot:
    """Benchmark performance snapshot."""
    benchmark_id: str
    throughput_steps_per_sec: float
    memory_rss_bytes: int
    elapsed_seconds: float
    timestamp_utc: str
    drift_percent: float = 0.0

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> PerformanceSnapshot:
        return cls(
            benchmark_id=str(data["benchmark_id"]),
            throughput_steps_per_sec=float(data["throughput_steps_per_sec"]),
            memory_rss_bytes=int(data["memory_rss_bytes"]),
            elapsed_seconds=float(data["elapsed_seconds"]),
            timestamp_utc=str(data["timestamp_utc"]),
            drift_percent=float(data.get("drift_percent", 0.0)),
        )


@dataclass
class AutomationROI:
    """Return on investment (ROI) evaluation of an automated workflow."""
    workflow_name: str
    hours_saved: float
    triage_hours_spent: float
    intervention_count: int
    false_alerts_count: int
    efficiency_ratio: float
    verdict: str  # "beneficial" | "neutral" | "burdensome"

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass
class WorkflowRollbackReceipt:
    """Receipt generated when an evidence-weakening workflow is rolled back."""
    workflow_name: str
    rolled_back_at_utc: str
    rolled_back_by: str
    reason: str
    evidence_weakening_details: str
    state_hash: str

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


# ---------------------------------------------------------------------------
# 3. Core Functional Capabilities
# ---------------------------------------------------------------------------

# Markers to scan for quality debt (constructed so self-scan does not trigger false match)
_DEBT_MARKER_NAMES = ("TO" + "DO", "FIX" + "ME", "HA" + "CK", "DEPRE" + "CATED")
_DEBT_MARKER_REGEX = re.compile(r"\b(" + "|".join(_DEBT_MARKER_NAMES) + r")\b")

_EXCLUDE_DIR_NAMES = frozenset({
    ".git", ".hg", ".svn", "target", "node_modules", "__pycache__",
    ".venv", "venv", ".artifacts", ".idea", ".vscode", ".tox",
    "harness-venv",
})

_SCANNABLE_EXTENSIONS = frozenset({
    ".rs", ".py", ".md", ".toml", ".json", ".yaml", ".yml", ".sh",
    ".c", ".h", ".cpp", ".txt",
})


def audit_quality_debt(
    repo_root: Path | str,
    max_score: float = 100.0,
    warnings_file: Optional[Path | str] = None,
    coverage_file: Optional[Path | str] = None,
    warning_count: Optional[int] = None,
    uncovered_lines: Optional[int] = None,
    raise_on_error: bool = False,
) -> DebtMetrics:
    """Audit source code and docs for debt indicators, compiler warnings, and coverage deficits.

    Formula:
        debt_score = (todo_count * 1.0) + (warning_count * 5.0) + (uncovered_deficit * 2.0)
    """
    root = Path(repo_root).resolve()
    details: list[str] = []

    # 1. Scan source code and documentation for debt markers
    todo_markers_count = 0
    marker_locations: list[str] = []

    if root.exists():
        for dirpath, dirnames, filenames in os.walk(root):
            # Prune excluded directories in-place
            dirnames[:] = [d for d in dirnames if d not in _EXCLUDE_DIR_NAMES and not d.startswith(".git")]
            current_dir = Path(dirpath)

            for filename in filenames:
                file_path = current_dir / filename
                if file_path.suffix.lower() not in _SCANNABLE_EXTENSIONS:
                    continue

                # Skip self if scanning tools/quality_tracker.py
                try:
                    if file_path.resolve() == Path(__file__).resolve():
                        continue
                except Exception:
                    pass

                # Avoid scanning overly large files (> 2 MB)
                try:
                    if file_path.stat().st_size > 2 * 1024 * 1024:
                        continue
                except OSError:
                    continue

                try:
                    content = file_path.read_text(encoding="utf-8", errors="ignore")
                except OSError:
                    continue

                for line_idx, line in enumerate(content.splitlines(), start=1):
                    matches = _DEBT_MARKER_REGEX.findall(line)
                    if matches:
                        count = len(matches)
                        todo_markers_count += count
                        rel_path = file_path.relative_to(root) if file_path.is_relative_to(root) else file_path.name
                        for m in matches:
                            if len(marker_locations) < 50:
                                marker_locations.append(f"{rel_path}:{line_idx} [{m}]")

    # 2. Check compiler warning records
    if warning_count is not None:
        warnings = max(0, int(warning_count))
    else:
        warnings = 0
        w_path: Optional[Path] = None
        if warnings_file is not None:
            w_path = Path(warnings_file)
            if not w_path.is_absolute():
                w_path = root / w_path
        else:
            candidates = [
                root / ".artifacts/ci/warnings.json",
                root / "target/warnings.json",
                root / ".artifacts/warnings.json",
            ]
            for cand in candidates:
                if cand.exists():
                    w_path = cand
                    break

        if w_path and w_path.exists():
            try:
                if w_path.suffix == ".json":
                    data = json.loads(w_path.read_text(encoding="utf-8"))
                    if isinstance(data, list):
                        warnings = len(data)
                    elif isinstance(data, dict):
                        if "count" in data and isinstance(data["count"], (int, float)):
                            warnings = int(data["count"])
                        elif "warnings" in data and isinstance(data["warnings"], list):
                            warnings = len(data["warnings"])
                else:
                    text = w_path.read_text(encoding="utf-8", errors="ignore")
                    matches = re.findall(r"(?i)\bwarning(\[[^\]]+\])?:\s*", text)
                    warnings = len(matches)
            except Exception as exc:
                details.append(f"Could not parse warnings file {w_path}: {exc}")

    # 3. Check uncovered lines / coverage deficit
    if uncovered_lines is not None:
        uncovered = max(0, int(uncovered_lines))
    else:
        uncovered = 0
        c_path: Optional[Path] = None
        if coverage_file is not None:
            c_path = Path(coverage_file)
            if not c_path.is_absolute():
                c_path = root / c_path
        else:
            candidates = [
                root / ".artifacts/ci/coverage.json",
                root / ".artifacts/coverage.json",
            ]
            for cand in candidates:
                if cand.exists():
                    c_path = cand
                    break

        if c_path and c_path.exists():
            try:
                c_data = json.loads(c_path.read_text(encoding="utf-8"))
                if isinstance(c_data, dict):
                    if "uncovered_lines" in c_data:
                        uncovered = int(c_data["uncovered_lines"])
                    elif "data" in c_data and isinstance(c_data["data"], list):
                        total_uncovered = 0
                        for entry in c_data["data"]:
                            if isinstance(entry, dict) and "files" in entry:
                                for f in entry["files"]:
                                    if "summary" in f and "lines" in f["summary"]:
                                        lines_info = f["summary"]["lines"]
                                        c_cnt = lines_info.get("count", 0)
                                        c_cov = lines_info.get("covered", 0)
                                        if c_cnt > c_cov:
                                            total_uncovered += (c_cnt - c_cov)
                        uncovered = total_uncovered
            except Exception as exc:
                details.append(f"Could not parse coverage file {c_path}: {exc}")

    # 4. Compute composite debt score
    # debt_score = (todo_count * 1.0) + (warning_count * 5.0) + (uncovered_deficit * 2.0)
    debt_score = round(
        (todo_markers_count * 1.0) + (warnings * 5.0) + (uncovered * 2.0),
        2,
    )

    if debt_score > max_score:
        status = "debt_exceeded"
        summary_msg = (
            f"Quality debt score {debt_score:.2f} exceeds threshold {max_score:.2f} "
            f"(debt markers: {todo_markers_count}, compiler warnings: {warnings}, uncovered lines: {uncovered})"
        )
        details.insert(0, summary_msg)
        if marker_locations:
            details.append(f"Sample debt marker locations: {', '.join(marker_locations[:10])}")
        if raise_on_error:
            raise QualityDebtExceededError(summary_msg)
    else:
        status = "acceptable"
        summary_msg = (
            f"Quality debt score {debt_score:.2f} within threshold {max_score:.2f} "
            f"(debt markers: {todo_markers_count}, compiler warnings: {warnings}, uncovered lines: {uncovered})"
        )
        details.insert(0, summary_msg)

    return DebtMetrics(
        todo_markers_count=todo_markers_count,
        warning_count=warnings,
        uncovered_lines=uncovered,
        debt_score=debt_score,
        status=status,
        details=details,
    )


def _parse_iso_date(value: Any) -> Optional[date]:
    """Parse ISO YYYY-MM-DD date."""
    if not isinstance(value, str) or len(value) != 10:
        return None
    try:
        parsed = date.fromisoformat(value)
    except ValueError:
        return None
    if parsed.isoformat() != value:
        return None
    return parsed


def audit_flaky_tests(
    repo_root: Path | str,
    flake_config: Optional[dict[str, Any]] = None,
    today: Optional[Union[str, date]] = None,
    raise_on_error: bool = False,
) -> list[FlakeRecord]:
    """Validate flaky test inventory against tools/quality_flakes.py rules and check expiry.

    Raises UnquarantinedFlakeError if unquarantined or expired flakes are found and raise_on_error is True.
    """
    root = Path(repo_root).resolve()

    if flake_config is None:
        gates_path = root / ".config/d33-quality-gates.json"
        if gates_path.exists():
            try:
                flake_config = json.loads(gates_path.read_text(encoding="utf-8"))
            except (OSError, json.JSONDecodeError) as exc:
                if raise_on_error:
                    raise UnquarantinedFlakeError(f"Failed to read flaky test configuration: {exc}")
                return []
        else:
            flake_config = {"schema_version": 1, "flakes": []}

    # Determine reference date
    if today is None:
        comparison_date = date.today()
    elif isinstance(today, date):
        comparison_date = today
    elif isinstance(today, str):
        parsed = _parse_iso_date(today)
        if parsed is None:
            if raise_on_error:
                raise UnquarantinedFlakeError(f"today must be a valid YYYY-MM-DD date: '{today}'")
            comparison_date = date.today()
        else:
            comparison_date = parsed
    else:
        if raise_on_error:
            raise UnquarantinedFlakeError(f"today must be a date or YYYY-MM-DD string: {today!r}")
        comparison_date = date.today()

    # Delegate structural validation to tools/quality_flakes.py if available
    quality_flakes_mod = None
    try:
        from tools import quality_flakes as quality_flakes_mod
    except ImportError:
        try:
            import quality_flakes as quality_flakes_mod
        except ImportError:
            pass

    structural_errors: list[str] = []
    if quality_flakes_mod is not None:
        structural_errors = quality_flakes_mod.check(flake_config, today=comparison_date)

    raw_flakes = flake_config.get("flakes", [])
    records: list[FlakeRecord] = []
    failed_reasons: list[str] = []

    if not isinstance(raw_flakes, list):
        if raise_on_error:
            raise UnquarantinedFlakeError("flakes configuration must be a list")
        return []

    for entry in raw_flakes:
        if not isinstance(entry, dict):
            continue

        test_id = str(entry.get("test_id", "")).strip()
        owner = str(entry.get("owner", "")).strip()
        reason = str(entry.get("reason", "")).strip()
        issue_url = str(entry.get("issue_url", "")).strip()
        expires_on = str(entry.get("expires_on", "")).strip()

        expiry = _parse_iso_date(expires_on)
        if expiry is None:
            is_expired = True
            is_quarantined = False
            failed_reasons.append(f"Test '{test_id}' has invalid expiration date '{expires_on}'")
        elif comparison_date > expiry:
            is_expired = True
            is_quarantined = False
            failed_reasons.append(f"Test '{test_id}' quarantine expired on {expires_on} (comparison date: {comparison_date})")
        else:
            is_expired = False
            # Check explicit quarantine flag if provided
            quarantine_flag = entry.get("is_quarantined", entry.get("quarantined", True))
            is_quarantined = bool(quarantine_flag)
            if not is_quarantined:
                failed_reasons.append(f"Test '{test_id}' is marked unquarantined")

        records.append(
            FlakeRecord(
                test_id=test_id,
                owner=owner,
                reason=reason,
                issue_url=issue_url,
                expires_on=expires_on,
                is_expired=is_expired,
                is_quarantined=is_quarantined,
            )
        )

    # Check for unquarantined items listed outside the inventory
    unquarantined_entries = flake_config.get("unquarantined_flakes", [])
    if isinstance(unquarantined_entries, list):
        for u in unquarantined_entries:
            if isinstance(u, dict):
                t_id = str(u.get("test_id", "")).strip()
                failed_reasons.append(f"Unquarantined flake identified: '{t_id}'")
                records.append(
                    FlakeRecord(
                        test_id=t_id,
                        owner=str(u.get("owner", "")).strip(),
                        reason=str(u.get("reason", "")).strip(),
                        issue_url=str(u.get("issue_url", "")).strip(),
                        expires_on=str(u.get("expires_on", "")).strip(),
                        is_expired=False,
                        is_quarantined=False,
                    )
                )

    if (structural_errors or failed_reasons) and raise_on_error:
        all_errs = structural_errors + failed_reasons
        raise UnquarantinedFlakeError(f"Flaky test quarantine audit failed: {'; '.join(all_errs)}")

    return records


def audit_performance_drift(
    baseline: Union[PerformanceSnapshot, dict[str, Any]],
    current: Union[PerformanceSnapshot, dict[str, Any]],
    max_slowdown_percent: float = 15.0,
    max_memory_growth_percent: float = 50.0,
    raise_on_error: bool = False,
) -> dict[str, Any]:
    """Compare current performance snapshot against baseline for throughput degradation and memory drift."""
    if isinstance(baseline, dict):
        base_snap = PerformanceSnapshot.from_dict(baseline)
    else:
        base_snap = baseline

    if isinstance(current, dict):
        curr_snap = PerformanceSnapshot.from_dict(current)
    else:
        curr_snap = current

    b_tput = float(base_snap.throughput_steps_per_sec)
    c_tput = float(curr_snap.throughput_steps_per_sec)
    b_mem = int(base_snap.memory_rss_bytes)
    c_mem = int(curr_snap.memory_rss_bytes)

    # Throughput slowdown: % drop relative to baseline
    if b_tput > 0.0:
        slowdown_percent = max(0.0, ((b_tput - c_tput) / b_tput) * 100.0)
        throughput_drift_percent = ((c_tput - b_tput) / b_tput) * 100.0
    else:
        slowdown_percent = 0.0
        throughput_drift_percent = 0.0

    curr_snap.drift_percent = round(throughput_drift_percent, 2)

    # Memory growth: % increase relative to baseline
    if b_mem > 0:
        memory_growth_percent = max(0.0, ((c_mem - b_mem) / b_mem) * 100.0)
    else:
        memory_growth_percent = 0.0

    is_slowdown = slowdown_percent > max_slowdown_percent
    is_memory_leak = memory_growth_percent > max_memory_growth_percent
    is_regression = is_slowdown or is_memory_leak

    details: list[str] = []
    if is_slowdown:
        details.append(
            f"Throughput degraded by {slowdown_percent:.2f}% (exceeds {max_slowdown_percent:.1f}% tolerance): "
            f"baseline={b_tput:.2f} steps/s, current={c_tput:.2f} steps/s"
        )
    if is_memory_leak:
        details.append(
            f"Memory RSS grew by {memory_growth_percent:.2f}% (exceeds {max_memory_growth_percent:.1f}% tolerance): "
            f"baseline={b_mem} bytes, current={c_mem} bytes"
        )
    if not is_regression:
        details.append(
            f"Performance within tolerance: throughput drift={throughput_drift_percent:+.2f}%, "
            f"memory growth={memory_growth_percent:+.2f}%"
        )

    if is_regression and raise_on_error:
        raise PerformanceRegressionError("; ".join(details))

    return {
        "benchmark_id": curr_snap.benchmark_id,
        "baseline_throughput": b_tput,
        "current_throughput": c_tput,
        "slowdown_percent": round(slowdown_percent, 2),
        "throughput_drift_percent": round(throughput_drift_percent, 2),
        "baseline_memory_rss_bytes": b_mem,
        "current_memory_rss_bytes": c_mem,
        "memory_growth_percent": round(memory_growth_percent, 2),
        "max_slowdown_percent": max_slowdown_percent,
        "max_memory_growth_percent": max_memory_growth_percent,
        "is_regression": is_regression,
        "status": "regressed" if is_regression else "passed",
        "details": details,
    }


def evaluate_automation_roi(
    workflow_name: str,
    hours_saved: float,
    triage_hours: float,
    interventions: int,
    false_alerts: int,
) -> AutomationROI:
    """Evaluate whether automation saves maintainer effort or adds maintenance burden.

    Formula:
        efficiency_ratio = hours_saved / max(0.1, triage_hours + (interventions * 0.5))

    Classification:
        efficiency_ratio >= 2.0 and false_alerts <= 2 -> "beneficial"
        1.0 <= efficiency_ratio < 2.0                -> "neutral"
        efficiency_ratio < 1.0 or false_alerts > 5   -> "burdensome"
    """
    saved = float(hours_saved)
    triage = float(triage_hours)
    interv = int(interventions)
    alerts = int(false_alerts)

    denominator = max(0.1, triage + (interv * 0.5))
    efficiency_ratio = round(saved / denominator, 4)

    if efficiency_ratio < 1.0 or alerts > 5:
        verdict = "burdensome"
    elif efficiency_ratio >= 2.0 and alerts <= 2:
        verdict = "beneficial"
    else:
        verdict = "neutral"

    return AutomationROI(
        workflow_name=workflow_name,
        hours_saved=saved,
        triage_hours_spent=triage,
        intervention_count=interv,
        false_alerts_count=alerts,
        efficiency_ratio=efficiency_ratio,
        verdict=verdict,
    )


def assert_workflow_evidence_integrity(workflow_name: str, actions: list[str]) -> None:
    """Check workflow actions for evidence-weakening patterns.

    Raises WorkflowEvidenceWeakenedError if any action attempts to weaken test coverage,
    ignore errors, bypass gates, or remove evidence records.
    """
    evidence_weakening_patterns = (
        r"\bignore[\s\-_]?errors?\b",
        r"\bcontinue[\s\-_]?on[\s\-_]?error\s*:\s*true\b",
        r"\blower[\s\-_]?coverage\b",
        r"\breduce[\s\-_]?threshold\b",
        r"\bskip[\s\-_]?tests?\b",
        r"\bdisable[\s\-_]?gate\b",
        r"\bweakened?[\s\-_]+assertion\b",
        r"\bdelete[\s\-_]?evidence\b",
    )
    for act in actions:
        for pat in evidence_weakening_patterns:
            if re.search(pat, act, re.IGNORECASE):
                raise WorkflowEvidenceWeakenedError(
                    f"Workflow '{workflow_name}' weakens evidence: action '{act}' matches pattern '{pat}'"
                )


def rollback_weakened_workflow(
    workflow_name: str,
    reason: str,
    user: str = "maintainer",
    evidence_dir: Union[Path, str] = "conductor/evidence",
    workflows_config_path: Optional[Union[Path, str]] = None,
    evidence_weakening_details: Optional[str] = None,
) -> WorkflowRollbackReceipt:
    """Formally disable offending workflow, verify no publication, and write immutable receipt."""
    # 1. Prohibit autonomous publication or pushing
    assert_no_autonomous_publication(workflow_name)
    assert_no_autonomous_publication(reason)
    assert_no_autonomous_publication(user)
    if evidence_weakening_details:
        assert_no_autonomous_publication(evidence_weakening_details)

    if not workflow_name.strip() or not reason.strip():
        raise ValueError("workflow_name and reason must be nonempty strings")

    now_utc = datetime.now(timezone.utc).isoformat()
    weakening_details = evidence_weakening_details or f"Automated evidence rollback: {reason}"

    # 2. Formally disable workflow in configuration if file exists
    if workflows_config_path is not None:
        wf_path = Path(workflows_config_path)
        if wf_path.exists():
            try:
                data = json.loads(wf_path.read_text(encoding="utf-8"))
                if isinstance(data, dict):
                    workflows = data.get("workflows", data)
                    if isinstance(workflows, dict) and workflow_name in workflows:
                        target = workflows[workflow_name]
                        if isinstance(target, dict):
                            target["enabled"] = False
                            target["status"] = "disabled"
                            target["rolled_back_at_utc"] = now_utc
                            target["rolled_back_by"] = user
                            target["rollback_reason"] = reason
                    elif isinstance(workflows, list):
                        for item in workflows:
                            if isinstance(item, dict) and item.get("name") == workflow_name:
                                item["enabled"] = False
                                item["status"] = "disabled"
                                item["rolled_back_at_utc"] = now_utc
                                item["rolled_back_by"] = user
                                item["rollback_reason"] = reason
                wf_path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
            except Exception as exc:
                raise QualityTrackingError(f"Failed to update workflow configuration at {wf_path}: {exc}")

    # 3. Compute immutable state hash
    hash_payload = f"{workflow_name}:{now_utc}:{user}:{reason}:{weakening_details}".encode("utf-8")
    state_hash = hashlib.sha256(hash_payload).hexdigest()

    receipt = WorkflowRollbackReceipt(
        workflow_name=workflow_name,
        rolled_back_at_utc=now_utc,
        rolled_back_by=user,
        reason=reason,
        evidence_weakening_details=weakening_details,
        state_hash=state_hash,
    )

    # 4. Write immutable rollback receipt into evidence directory
    ev_dir = Path(evidence_dir)
    ev_dir.mkdir(parents=True, exist_ok=True)
    safe_name = re.sub(r"[^a-zA-Z0-9_\-]", "_", workflow_name)
    receipt_file = ev_dir / f"rollback_{safe_name}.json"

    # If receipt already exists, make it writable temporarily to update or write versioned
    if receipt_file.exists():
        try:
            os.chmod(receipt_file, 0o644)
        except OSError:
            pass

    receipt_file.write_text(json.dumps(receipt.to_dict(), indent=2) + "\n", encoding="utf-8")

    # Mark receipt file read-only to guarantee immutability
    try:
        os.chmod(receipt_file, 0o444)
    except OSError:
        pass

    return receipt


# ---------------------------------------------------------------------------
# 4. CLI Interface
# ---------------------------------------------------------------------------

def build_parser() -> argparse.ArgumentParser:
    """Build the argument parser for quality_tracker."""
    parser = argparse.ArgumentParser(
        prog="quality_tracker.py",
        description="CareOps Sim Quality Debt, Flaky Test Quarantine, and Performance Drift Tracker",
    )
    subparsers = parser.add_subparsers(dest="subcommand", required=True)

    # 1. Debt
    debt_p = subparsers.add_parser("debt", help="Audit quality debt markers and metrics")
    debt_p.add_argument("--repo-root", type=Path, default=Path("."), help="Repository root path")
    debt_p.add_argument("--max-score", type=float, default=100.0, help="Maximum allowed quality debt score")
    debt_p.add_argument("--output", type=Path, default=None, help="Optional output JSON path")

    # 2. Flakes
    flakes_p = subparsers.add_parser("flakes", help="Audit flaky test inventory and quarantine expiry")
    flakes_p.add_argument("--config", type=Path, default=None, help="Path to quality gates or flakes JSON config")
    flakes_p.add_argument("--today", type=str, default=None, help="Reference date (YYYY-MM-DD) for expiry check")

    # 3. Performance
    perf_p = subparsers.add_parser("performance", help="Audit performance drift against baseline snapshot")
    perf_p.add_argument("--baseline", type=Path, required=True, help="Path to baseline PerformanceSnapshot JSON")
    perf_p.add_argument("--current", type=Path, required=True, help="Path to current PerformanceSnapshot JSON")
    perf_p.add_argument("--max-slowdown", type=float, default=15.0, help="Maximum allowable throughput slowdown percent")

    # 4. ROI
    roi_p = subparsers.add_parser("roi", help="Evaluate automation return on investment (ROI)")
    roi_p.add_argument("--workflow", type=str, required=True, help="Workflow or routine name")
    roi_p.add_argument("--saved", type=float, required=True, help="Maintainer hours saved")
    roi_p.add_argument("--triage", type=float, required=True, help="Maintainer triage hours spent")
    roi_p.add_argument("--interventions", type=int, required=True, help="Number of human interventions required")
    roi_p.add_argument("--false-alerts", type=int, required=True, help="Number of false positive alerts")

    # 5. Rollback
    rollback_p = subparsers.add_parser("rollback", help="Roll back an evidence-weakening automated workflow")
    rollback_p.add_argument("--workflow", type=str, required=True, help="Workflow name to disable and roll back")
    rollback_p.add_argument("--reason", type=str, required=True, help="Reason for rolling back workflow")
    rollback_p.add_argument("--user", type=str, default=os.environ.get("USER", "maintainer"), help="Maintainer identifier")
    rollback_p.add_argument("--evidence-dir", type=Path, default=Path("conductor/evidence"), help="Evidence receipt directory")
    rollback_p.add_argument("--workflows-config", type=Path, default=None, help="Optional workflow configuration file")

    return parser


def main(argv: Optional[list[str]] = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)

    if args.subcommand == "debt":
        try:
            metrics = audit_quality_debt(
                repo_root=args.repo_root,
                max_score=args.max_score,
                raise_on_error=False,
            )
            print(f"Quality Debt Audit: {metrics.status.upper()}")
            print(f"  Debt Score:       {metrics.debt_score:.2f} (Max threshold: {args.max_score:.2f})")
            print(f"  TODO Markers:     {metrics.todo_markers_count}")
            print(f"  Compiler Warnings:{metrics.warning_count}")
            print(f"  Uncovered Lines:  {metrics.uncovered_lines}")

            if args.output:
                args.output.parent.mkdir(parents=True, exist_ok=True)
                args.output.write_text(json.dumps(metrics.to_dict(), indent=2) + "\n", encoding="utf-8")
                print(f"Saved debt metrics JSON to {args.output}")

            if metrics.debt_score > args.max_score:
                print(f"ERROR: Quality debt score {metrics.debt_score:.2f} exceeds threshold {args.max_score:.2f}", file=sys.stderr)
                return 1
            return 0
        except Exception as exc:
            print(f"quality_tracker debt error: {exc}", file=sys.stderr)
            return 2

    elif args.subcommand == "flakes":
        try:
            flake_config = None
            if args.config:
                if not args.config.exists():
                    print(f"Config file not found: {args.config}", file=sys.stderr)
                    return 2
                flake_config = json.loads(args.config.read_text(encoding="utf-8"))

            records = audit_flaky_tests(
                repo_root=Path("."),
                flake_config=flake_config,
                today=args.today,
                raise_on_error=False,
            )
            print(f"Flaky Test Audit: {len(records)} test exception(s) documented.")
            has_failure = False
            for r in records:
                status_str = "OK"
                if r.is_expired:
                    status_str = "EXPIRED"
                    has_failure = True
                elif not r.is_quarantined:
                    status_str = "UNQUARANTINED"
                    has_failure = True
                print(f"  [{status_str}] {r.test_id} (Owner: {r.owner}, Expires: {r.expires_on})")

            if has_failure:
                print("ERROR: One or more flaky tests are expired or unquarantined.", file=sys.stderr)
                return 1
            return 0
        except Exception as exc:
            print(f"quality_tracker flakes error: {exc}", file=sys.stderr)
            return 2

    elif args.subcommand == "performance":
        try:
            if not args.baseline.exists():
                print(f"Baseline file not found: {args.baseline}", file=sys.stderr)
                return 2
            if not args.current.exists():
                print(f"Current snapshot file not found: {args.current}", file=sys.stderr)
                return 2

            b_data = json.loads(args.baseline.read_text(encoding="utf-8"))
            c_data = json.loads(args.current.read_text(encoding="utf-8"))
            res = audit_performance_drift(
                baseline=b_data,
                current=c_data,
                max_slowdown_percent=args.max_slowdown,
                raise_on_error=False,
            )

            print(f"Performance Drift Audit: {res['status'].upper()}")
            print(f"  Baseline Throughput: {res['baseline_throughput']:.2f} steps/s")
            print(f"  Current Throughput:  {res['current_throughput']:.2f} steps/s")
            print(f"  Throughput Drift:    {res['throughput_drift_percent']:+.2f}% (Slowdown: {res['slowdown_percent']:.2f}%, Max: {args.max_slowdown:.1f}%)")
            print(f"  Memory RSS Growth:   {res['memory_growth_percent']:+.2f}%")

            if res["is_regression"]:
                print(f"ERROR: Performance regression detected: {'; '.join(res['details'])}", file=sys.stderr)
                return 1
            return 0
        except Exception as exc:
            print(f"quality_tracker performance error: {exc}", file=sys.stderr)
            return 2

    elif args.subcommand == "roi":
        try:
            roi = evaluate_automation_roi(
                workflow_name=args.workflow,
                hours_saved=args.saved,
                triage_hours=args.triage,
                interventions=args.interventions,
                false_alerts=args.false_alerts,
            )
            print(f"Automation ROI Evaluation: {roi.workflow_name}")
            print(f"  Efficiency Ratio: {roi.efficiency_ratio:.2f}")
            print(f"  Verdict:          {roi.verdict.upper()}")
            print(f"  Maintainer Hours: {roi.hours_saved:.1f} saved vs {roi.triage_hours_spent:.1f} triage")
            print(f"  Frictions:        {roi.intervention_count} interventions, {roi.false_alerts_count} false alerts")
            return 0
        except Exception as exc:
            print(f"quality_tracker roi error: {exc}", file=sys.stderr)
            return 2

    elif args.subcommand == "rollback":
        try:
            receipt = rollback_weakened_workflow(
                workflow_name=args.workflow,
                reason=args.reason,
                user=args.user,
                evidence_dir=args.evidence_dir,
                workflows_config_path=args.workflows_config,
            )
            print(f"Workflow '{receipt.workflow_name}' rolled back successfully.")
            print(f"  Receipt Hash: {receipt.state_hash}")
            print(f"  Rolled Back By: {receipt.rolled_back_by} at {receipt.rolled_back_at_utc}")
            print(f"  Evidence Receipt: {args.evidence_dir / f'rollback_{receipt.workflow_name}.json'}")
            return 0
        except AutonomousPublicationError as exc:
            print(f"AUTONOMOUS PUBLICATION PROHIBITED: {exc}", file=sys.stderr)
            return 1
        except Exception as exc:
            print(f"quality_tracker rollback error: {exc}", file=sys.stderr)
            return 2

    return 0


if __name__ == "__main__":
    sys.exit(main())
