#!/usr/bin/env python3
"""Bounded synthetic ED CLI benchmark runner (development evidence only)."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import select
import signal
import subprocess
import sys
import time
from pathlib import Path

from ed_benchmark_process import (
    BenchmarkError, CONSUMER_QUANTUM, MAX_OUTPUT, MAX_STDERR, MAX_RSS_BYTES,
    MAX_WALL_SECONDS, run_bounded, _managed_child, _kill_group,
    _rss_bytes, _run_to_full,
)


MAX_INPUT = 8 * 1024 * 1024
SIZES = (100, 1_000, 10_000)
REPEATS = 2
SOAK_RUNS = 20
SEED = 7
WORK_TICKS = 2




def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"),
                      allow_nan=False).encode("utf-8")


def make_input(template: dict, count: int) -> bytes:
    scenario = json.loads(json.dumps(template))
    scenario["scenario_id"] = f"d34-fifo-{count}"
    scenario["seed"] = SEED
    scenario["horizon_ticks"] = 2 * count + 2
    scenario["patients"] = [
        {"patient_id": f"synthetic-patient-{i + 1:05d}",
         "arrival_tick": 1, "work_ticks": WORK_TICKS,
         "resource_id": "resource.synthetic", "task_class_id": "task.synthetic"}
        for i in range(count)
    ]
    return json.dumps(scenario, indent=2, allow_nan=False).encode("utf-8") + b"\n"


def preflight_input(data: bytes) -> None:
    if len(data) > MAX_INPUT:
        raise BenchmarkError(f"input exceeds {MAX_INPUT} byte harness limit")










def validate_document(document: dict, expected: dict, input_hash: str,
                      input_filename: str) -> dict:
    manifest = document.get("manifest")
    summary = document.get("summary")
    if not isinstance(manifest, dict) or not isinstance(summary, dict):
        raise BenchmarkError("CLI output lacks manifest or summary object")
    if manifest.get("input_sha256") != input_hash:
        raise BenchmarkError("manifest input hash mismatch")
    if manifest.get("input_file") != input_filename:
        raise BenchmarkError("manifest input filename mismatch")
    count = expected["count"]
    manifest_wanted = {"schema_version": 1, "scenario_id": expected["scenario_id"],
                       "seed": SEED, "time_unit": "tick", "horizon_ticks": 2 * count + 2}
    _strict_fields(manifest, manifest_wanted, "manifest")
    if set(manifest) != set(manifest_wanted) | {"input_sha256", "input_file"}:
        raise BenchmarkError("manifest field set mismatch")
    summary_wanted = {"schema_version": 1, "scenario_id": expected["scenario_id"],
                      "seed": SEED, "time_unit": "tick", "horizon_ticks": 2 * count + 2}
    _strict_fields(summary, summary_wanted, "summary")
    wanted_counts = {"arrivals": count, "started": count,
                     "completed": count, "unfinished": 0}
    for key, value in wanted_counts.items():
        if key not in summary or not _same_typed(summary[key], value):
            raise BenchmarkError(f"summary {key} mismatch")
    if set(summary) != set(summary_wanted) | set(wanted_counts) | {"patients"}:
        raise BenchmarkError("summary field set mismatch")
    if summary.get("scenario_id") != expected["scenario_id"] or summary.get("seed") != SEED:
        raise BenchmarkError("summary scenario identity or seed mismatch")
    rows = summary.get("patients")
    if not isinstance(rows, list) or len(rows) != count:
        raise BenchmarkError("patient row count mismatch")
    for k, row in enumerate(rows):
        want = {"patient_id": f"synthetic-patient-{k + 1:05d}",
                "arrival_tick": 1, "start_tick": 1 + 2 * k,
                "completion_tick": 3 + 2 * k, "work_ticks": 2,
                "wait_ticks": 2 * k, "elapsed_ticks": 2 + 2 * k,
                "status": "completed"}
        if not _same_typed(row, want):
            raise BenchmarkError(f"patient row {k} differs from FIFO oracle")
    repeat_summary = dict(summary)
    repeat_summary.pop("input_file", None)
    return repeat_summary


def _same_typed(actual, wanted) -> bool:
    if type(actual) is not type(wanted):
        return False
    if isinstance(wanted, dict):
        return actual.keys() == wanted.keys() and all(
            _same_typed(actual[key], value) for key, value in wanted.items())
    if isinstance(wanted, list):
        return len(actual) == len(wanted) and all(
            _same_typed(a, b) for a, b in zip(actual, wanted))
    return actual == wanted


def _strict_fields(actual, wanted, label):
    for key, value in wanted.items():
        if key not in actual or not _same_typed(actual[key], value):
            raise BenchmarkError(f"{label} {key} mismatch")


def _one_run(binary: Path, run_dir: Path, count: int, template: dict) -> dict:
    raw = make_input(template, count)
    preflight_input(raw)
    tag = f"n{count:05d}-{time.time_ns()}"
    run_dir.mkdir(parents=True, exist_ok=True)
    input_path = run_dir / f"{tag}.input.json"
    output_path = run_dir / f"{tag}.output.json"
    input_path.write_bytes(raw)
    result = run_bounded([str(binary), "run", str(input_path)])
    stdout = result.pop("stdout")
    stderr = result.pop("stderr")
    if result["timed_out"]:
        raise BenchmarkError("CLI exceeded wall-time budget")
    if result["exit_code"] != 0:
        raise BenchmarkError(f"CLI failed ({result['exit_code']}): {stderr.decode(errors='replace')}")
    if result["stdout_overflow"]:
        raise BenchmarkError("CLI stdout exceeded the captured-output budget")
    output_path.write_bytes(stdout)
    if len(stdout) > MAX_OUTPUT:
        raise BenchmarkError("CLI stdout exceeded the captured-output budget")
    try:
        doc = json.loads(stdout, parse_constant=lambda value: (_ for _ in ()).throw(
            BenchmarkError(f"non-finite JSON number {value}")))
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise BenchmarkError(f"invalid CLI JSON: {exc}") from exc
    expected = {"count": count, "scenario_id": f"d34-fifo-{count}"}
    repeat_summary = validate_document(doc, expected, sha256(raw), str(input_path))
    result.update({"input_sha256": sha256(raw), "output_sha256": sha256(stdout),
                   "input_bytes": len(raw), "output_bytes": len(stdout),
                   "repeat_summary_sha256": sha256(canonical(repeat_summary)),
                   "input_path": str(input_path), "output_path": str(output_path)})
    if result["elapsed_seconds"] > MAX_WALL_SECONDS:
        raise BenchmarkError("child exceeded wall-time budget")
    if result["peak_rss_bytes"] > MAX_RSS_BYTES:
        raise BenchmarkError("child exceeded peak-RSS budget")
    return result


def _preflight_checks(binary: Path, run_dir: Path, template: dict) -> dict:
    zero = json.loads(make_input(template, 1))
    zero["patients"][0]["work_ticks"] = 0
    zero_bytes = json.dumps(zero).encode()
    zero_path = run_dir / "zero-work.input.json"
    zero_path.write_bytes(zero_bytes)
    zero_result = run_bounded([str(binary), "run", str(zero_path)])
    zero_error = zero_result["stderr"].decode(errors="replace").lower()
    if (zero_result["timed_out"] or zero_result["exit_code"] == 0
            or "work_ticks" not in zero_error or "json" in zero_error):
        raise BenchmarkError("zero-work scenario was accepted")
    valid_path = run_dir / "storage-probe.input.json"
    valid_path.write_bytes(make_input(template, 1))
    oversized_refused = False
    try:
        preflight_input(b"x" * (MAX_INPUT + 1))
    except BenchmarkError:
        oversized_refused = True
    if not oversized_refused:
        raise BenchmarkError("oversized input preflight did not refuse")
    full_status = "unverified"
    full_path = Path("/dev/full")
    if full_path.exists():
        full_result = _run_to_full([str(binary), "run", str(valid_path)])
        full_error = full_result["stderr"].lower()
        full_status = "pass" if (not full_result["timed_out"]
                      and full_result["exit_code"] != 0
                      and full_result["elapsed_seconds"] <= MAX_WALL_SECONDS
                      and 0 < full_result["peak_rss_bytes"] <= MAX_RSS_BYTES
                      and (b"no space" in full_error
                           or b"enospc" in full_error or b"os error 28" in full_error)) else "fail"
        if full_status != "pass":
            raise BenchmarkError("/dev/full did not produce a CLI output failure")
    elif platform.system() == "Linux":
        raise BenchmarkError("Linux /dev/full is unavailable; exhausted-storage check cannot pass")
    return {"zero_work": "pass", "zero_work_error": zero_error.strip(),
            "oversized_input_preflight": "pass", "dev_full": full_status}




def _slow_consumer_probe(binary: Path, run_dir: Path, template: dict) -> dict:
    raw = make_input(template, 10_000)
    preflight_input(raw)
    path = run_dir / "slow-consumer.input.json"
    path.write_bytes(raw)
    probe = run_bounded([str(binary), "run", str(path)], timeout=MAX_WALL_SECONDS,
                        stdout_limit=MAX_OUTPUT, slow_consumer_delay=0.001)
    if (probe["timed_out"] or probe["exit_code"] != 0 or probe["stdout_overflow"]
            or probe["elapsed_seconds"] > MAX_WALL_SECONDS
            or probe["peak_rss_bytes"] > MAX_RSS_BYTES):
        raise BenchmarkError(
            "slow stdout-consumer probe failed: "
            f"exit={probe['exit_code']} timeout={probe['timed_out']} "
            f"overflow={probe['stdout_overflow']} bytes={len(probe['stdout'])} "
            f"elapsed={probe['elapsed_seconds']:.3f}s rss={probe['peak_rss_bytes']} "
            f"reads={probe['stdout_read_count']} max_gap={probe['max_read_gap_seconds']:.3f}s "
            f"stderr={probe['stderr'][:256].decode(errors='replace')!r}")
    try:
        doc = json.loads(probe["stdout"], parse_constant=lambda value: (_ for _ in ()).throw(
            BenchmarkError(f"non-finite JSON number {value}")))
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise BenchmarkError(f"slow-consumer CLI output invalid: {exc}") from exc
    validate_document(doc, {"count": 10_000, "scenario_id": "d34-fifo-10000"},
                      sha256(raw), str(path))
    return {"status": "pass", "bytes": len(probe["stdout"]),
            "consumer_quantum_bytes": CONSUMER_QUANTUM, "delay_seconds_per_quantum": 0.001,
            "output_sha256": sha256(probe["stdout"]),
            "elapsed_seconds": probe["elapsed_seconds"],
            "peak_rss_bytes": probe["peak_rss_bytes"], "input_sha256": sha256(raw)}


def _blocked_child_termination_probe(binary: Path, run_dir: Path, template: dict) -> dict:
    raw = make_input(template, 10_000)
    preflight_input(raw)
    path = run_dir / "blocked-output.input.json"
    path.write_bytes(raw)
    started = time.monotonic()
    with _managed_child([str(binary), "run", str(path)], stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE, start_new_session=True) as proc:
        try:
            import select
            readable, _, _ = select.select([proc.stdout], [], [], 5.0)
            if not readable or proc.poll() is not None:
                raise BenchmarkError("ED CLI did not produce blocked-pipe output before termination")
            time.sleep(0.1)  # Keep unread output pending and allow the pipe to fill.
            if proc.poll() is not None:
                raise BenchmarkError("ED CLI completed while output remained unread")
            _kill_group(proc.pid)
            deadline = time.monotonic() + 0.5
            while True:
                waited, status, usage = os.wait4(proc.pid, os.WNOHANG)
                if waited:
                    break
                if time.monotonic() >= deadline:
                    _kill_group(proc.pid, signal.SIGKILL)
                    _, status, usage = os.wait4(proc.pid, 0)
                    break
                time.sleep(0.01)
            proc.returncode = os.waitstatus_to_exitcode(status)
            if proc.returncode == 0:
                raise BenchmarkError("blocked ED CLI unexpectedly completed successfully")
            rss = _rss_bytes(usage)
            elapsed = time.monotonic() - started
            if elapsed > MAX_WALL_SECONDS or rss <= 0 or rss > MAX_RSS_BYTES:
                raise BenchmarkError("blocked-output termination exceeded its time or RSS budget")
            return {"status": "pass",
                    "exit_code": proc.returncode, "peak_rss_bytes": rss,
                    "stdout_read": False, "observed_elapsed_seconds": elapsed,
                    "input_sha256": sha256(raw)}
        finally:
            if proc.returncode is None:
                _kill_group(proc.pid)
                deadline = time.monotonic() + 0.5
                while True:
                    try:
                        waited, status, _ = os.wait4(proc.pid, os.WNOHANG)
                    except ChildProcessError:
                        proc.returncode = 255
                        break
                    if waited:
                        proc.returncode = os.waitstatus_to_exitcode(status)
                        break
                    if time.monotonic() >= deadline:
                        _kill_group(proc.pid, signal.SIGKILL)
                        try:
                            _, status, _ = os.wait4(proc.pid, 0)
                            proc.returncode = os.waitstatus_to_exitcode(status)
                        except ChildProcessError:
                            proc.returncode = 255
                        break
                    time.sleep(0.01)
            proc.stdout.close()
            proc.stderr.close()


def run_suite(binary: Path, output_file: Path) -> dict:
    binary = binary.resolve(strict=True)
    if not binary.is_file() or not os.access(binary, os.X_OK):
        raise BenchmarkError("--binary must identify an executable file")
    template_path = Path("crates/careops-ed/examples/one_patient.json")
    template_bytes = template_path.read_bytes()
    template = json.loads(template_bytes)
    output_file.parent.mkdir(parents=True, exist_ok=True)
    output_dir = output_file.parent / (output_file.stem + "-artifacts")
    output_dir.mkdir(parents=True, exist_ok=True)
    budgets = {"contract": "D3.4 development fixture; not release SLA",
               "sizes": list(SIZES), "repeats_per_size": REPEATS,
               "soak_runs_n1000": SOAK_RUNS, "seed": SEED,
               "arrival_tick": 1, "work_ticks": WORK_TICKS,
               "wall_seconds_per_child": MAX_WALL_SECONDS,
               "peak_rss_bytes_per_child": MAX_RSS_BYTES,
               "input_bytes_per_child": MAX_INPUT, "stdout_bytes_per_child": MAX_OUTPUT}
    (output_dir / "budgets.json").write_bytes(canonical(budgets) + b"\n")
    checks = _preflight_checks(binary, output_dir, template)
    checks["slow_stdout_consumer"] = _slow_consumer_probe(binary, output_dir, template)
    checks["blocked_child_termination"] = _blocked_child_termination_probe(
        binary, output_dir, template)
    measurements = []
    summaries = {}
    for count in SIZES:
        for repeat in range(REPEATS):
            record = _one_run(binary, output_dir, count, template)
            record.update({"case": count, "repeat": repeat + 1, "kind": "representative"})
            measurements.append(record)
            summaries.setdefault(count, []).append(record["repeat_summary_sha256"])
        if len(set(summaries[count])) != 1:
            raise BenchmarkError(f"repeat summaries differ for n={count}")
    for repeat in range(SOAK_RUNS):
        record = _one_run(binary, output_dir, 1_000, template)
        record.update({"case": 1_000, "repeat": repeat + 1, "kind": "soak"})
        measurements.append(record)
        if record["repeat_summary_sha256"] != summaries[1_000][0]:
            raise BenchmarkError(f"soak summary differs at repetition {repeat + 1}")
    rustc = subprocess.run(["rustup", "run", "1.99.0", "rustc", "--version", "--verbose"], check=True,
                           stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=5)
    cargo = subprocess.run(["rustup", "run", "1.99.0", "cargo", "--version"], check=True,
                           stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=5)
    if not rustc.stdout.startswith(b"rustc 1.99.0 ") or not cargo.stdout.startswith(b"cargo 1.99.0 "):
        raise BenchmarkError("canonical compiler/Cargo identity mismatch")
    result = {"schema_version": 1, "source_commit": _git_head(),
              "harness_command": [sys.executable, str(Path(__file__).resolve()),
                                  "--binary", str(binary), "--output", str(output_file)],
              "binary_sha256": sha256(binary.read_bytes()),
              "input_template_sha256": sha256(template_bytes),
              "harness_sha256": sha256(Path(__file__).read_bytes()),
              "process_harness_sha256": sha256(Path(__file__).with_name("ed_benchmark_process.py").read_bytes()),
              "cwd": str(Path.cwd()),
              "kairos_pin": subprocess.check_output(["git", "-C", "libs/kairos", "rev-parse", "HEAD"], text=True).strip(),
              "host": {"system": platform.system(), "release": platform.release(),
                       "machine": platform.machine(), "python": platform.python_version()},
              "toolchain": rustc.stdout.decode().strip(),
              "cargo": cargo.stdout.decode().strip(),
              "compiler_attribution": "canonical identity query; binary build is bound by the external locked build receipt",
              "budgets": budgets, "adversarial_checks": checks,
              "measurements": measurements}
    output_file.write_bytes(json.dumps(result, indent=2,
                                       allow_nan=False).encode() + b"\n")
    return result


def _git_head() -> str:
    result = subprocess.run(["git", "rev-parse", "HEAD"], check=True,
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    return result.stdout.decode().strip()


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path,
                        help="prebuilt locked Rust CLI executable")
    parser.add_argument("--output", required=True, type=Path,
                        help="JSON result report path; per-run files use a sibling directory")
    args = parser.parse_args(argv)
    try:
        result = run_suite(args.binary, args.output)
    except (BenchmarkError, OSError, subprocess.SubprocessError, ValueError) as exc:
        print(f"benchmark failed: {exc}", file=sys.stderr)
        return 1
    print(json.dumps({"result": str(args.output),
                      "measurements": len(result["measurements"]),
                      "status": "complete"}, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
