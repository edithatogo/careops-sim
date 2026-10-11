"""Immutable bounded gates for E2.2 checkpoint preview."""
import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[2]
TOOLCHAIN = Path("/Users/doughnut/.rustup/toolchains/1.99.0-aarch64-apple-darwin/bin")
SOURCES = [
    "crates/kairo-ecs-calibration/src/shadow_native.rs",
    "crates/kairo-ecs-calibration/src/c5_native.rs",
]
PROTECTED = [
    "crates/kairo-ecs-des/src/flow/checkpoint.rs",
    "crates/kairo-ecs-des/src/flow/checkpoint/v2.rs",
    "crates/kairo-ecs-des/src/flow.rs",
]
FROZEN_PROTECTED = {
    PROTECTED[0]: "9fcc232fb14403e91c43fc7cd7640210ce2d33b61468527b307d573e4906af81",
    PROTECTED[1]: "f6622ccf102a122bc6869617f2e9bd60a464892d7a99b0ab9cf74af5e4dc5b94",
    PROTECTED[2]: "08fc5f9bd7fe77bd4f0b27cd0b6d6f8d5de9a9f20925ff1794b04cc9d03995c4",
}
ORACLES = [
    "native_preview_handles_v2_pending_disable_and_horizon_without_advancing",
    "native_preview_disabled_queue_has_no_next_event_or_completion",
    "c5_native_preview_preserves_pending_disable_and_completion",
    "c5_native_preview_limit_error_propagates_without_v1_fallback",
]
GATES = {
    "tests": [str(TOOLCHAIN / "cargo"), "test", "--locked", "-p", "kairo-ecs-calibration", "--features", "flow"],
    "clippy": [str(TOOLCHAIN / "cargo"), "clippy", "--locked", "-p", "kairo-ecs-calibration", "--features", "flow", "--all-targets", "--", "-D", "warnings"],
    "format": [str(TOOLCHAIN / "rustfmt"), "--edition", "2021", "--check", "--config", "skip_children=true", *SOURCES],
}

def file_hashes(paths):
    return {path: hashlib.sha256((ROOT / path).read_bytes()).hexdigest()
            if (ROOT / path).is_file() else None for path in paths}

def write_new(path, value):
    with path.open("x", encoding="utf-8") as stream:
        json.dump(value, stream, indent=2)
        stream.write("\n")

def main():
    if len(sys.argv) != 2 or sys.argv[1] not in GATES:
        raise SystemExit("choose tests, clippy, or format")
    gate = sys.argv[1]
    out = ROOT / ".artifacts/mvp/E2.2.engine.checkpoint-preview/logs"
    out.mkdir(parents=True, exist_ok=True)
    stem = f"{gate}-{time.time_ns()}-{os.getpid()}"
    log = out / (stem + ".log")
    before = file_hashes(SOURCES)
    protected_before = file_hashes(PROTECTED)
    record = {"gate": gate, "argv": GATES[gate], "cwd": str(ROOT),
              "head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
              "python": sys.version, "start": datetime.datetime.now(datetime.timezone.utc).isoformat(),
              "source_hashes": before, "protected_hashes": protected_before,
              "log": str(log.relative_to(ROOT)), "status": "started"}
    write_new(out / (stem + "-started.json"), record)
    versions = {name: subprocess.check_output([str(TOOLCHAIN / name), "--version"], text=True).strip()
                for name in ("rustc", "cargo")}
    if not all(value.startswith(name + " 1.99.0 ") for name, value in versions.items()):
        raise SystemExit("Required Rust/Cargo 1.99.0 unavailable")
    record["toolchain"] = versions
    env = dict(os.environ, PYTHONDONTWRITEBYTECODE="1",
               PATH=str(TOOLCHAIN) + os.pathsep + os.environ.get("PATH", ""),
               CARGO_HOME="/tmp/c2-route-receipt-cargo-home",
               CARGO_TARGET_DIR=str(out / "target"), CARGO_INCREMENTAL="0",
               CARGO_PROFILE_DEV_DEBUG="0", CARGO_PROFILE_TEST_DEBUG="0", CARGO_BUILD_JOBS="2")
    command_exit = None
    with log.open("x", encoding="utf-8") as stream:
        if protected_before != FROZEN_PROTECTED:
            stream.write("Gate refused: accepted native protected source hash drift.\n")
            exit_code = 2
            record["gate_error"] = "Accepted native protected source hash drift"
        elif gate == "tests" and any(value is None for value in before.values()):
            stream.write("Gate refused: a declared source file is missing.\n")
            exit_code = 2
            record["gate_error"] = "Declared source file is missing"
        else:
            command_exit = subprocess.run(GATES[gate], cwd=ROOT, env=env,
                                          stdout=stream, stderr=subprocess.STDOUT).returncode
            exit_code = command_exit
    content = log.read_bytes()
    if gate == "tests" and exit_code == 0:
        text = content.decode(errors="replace")
        missing = [name for name in ORACLES
                   if not re.search(r"^test [^\n]*\b" + re.escape(name) + r"\b \.\.\. ok$", text, re.M)]
        if missing:
            exit_code = 2
            record["gate_error"] = "Named preview oracle missing or not passing: " + ", ".join(missing)
    after = file_hashes(SOURCES)
    protected_after = file_hashes(PROTECTED)
    if after != before:
        exit_code = 2
        record["gate_error"] = "Owned source changed during command"
    if protected_after != FROZEN_PROTECTED:
        exit_code = 2
        record["gate_error"] = "Accepted native protected source changed during command"
    record.update(status="finished", end=datetime.datetime.now(datetime.timezone.utc).isoformat(),
                  exit_code=exit_code, command_exit_code=command_exit,
                  log_sha256=hashlib.sha256(content).hexdigest(), source_hashes_after=after,
                  protected_hashes_after=protected_after)
    receipt = out / (stem + "-result.json")
    write_new(receipt, record)
    print(json.dumps({"exit_code": exit_code, "receipt": str(receipt.relative_to(ROOT)),
                      "log": record["log"]}))
    return exit_code

if __name__ == "__main__":
    raise SystemExit(main())
