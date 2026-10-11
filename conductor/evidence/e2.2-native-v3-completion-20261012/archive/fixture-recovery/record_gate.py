"""Local bounded gate capture; no dispatch or general harness claim."""
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
PACKAGES = ['-p', 'kairo-ecs-core', '-p', 'kairo-ecs-des', '-p', 'kairo-ecs-abm', '-p', 'kairo-ecs-calibration', '--features', 'kairo-ecs-calibration/flow']
SOURCES = ['crates/kairo-ecs-des/src/flow/checkpoint.rs', 'crates/kairo-ecs-des/src/flow/checkpoint/v2.rs', 'crates/kairo-ecs-des/src/flow/checkpoint/v3.rs', 'crates/kairo-ecs-des/tests/flow_checkpoint_v3.rs']
GATES = {
    "compile": [str(TOOLCHAIN / "cargo"), "test", "--no-run", "--locked", *PACKAGES],
    "focused": [str(TOOLCHAIN / "cargo"), "test", "--locked", "-p", "kairo-ecs-des", "native_v3_", "--", "--test-threads=1"],
    "tests": [str(TOOLCHAIN / "cargo"), "test", "--locked", *PACKAGES, "--", "--test-threads=1"],
    "clippy": [str(TOOLCHAIN / "cargo"), "clippy", "--locked", *PACKAGES, "--all-targets", "--", "-D", "warnings"],
    "format": [str(TOOLCHAIN / "rustfmt"), "--edition", "2021", "--check", "--config", "skip_children=true", *SOURCES],
}

def hashes():
    return {p: hashlib.sha256((ROOT / p).read_bytes()).hexdigest() if (ROOT / p).is_file() else None for p in SOURCES}

def write_new(path, value):
    with path.open("x") as stream:
        json.dump(value, stream, indent=2)
        stream.write("\n")

def main():
    if len(sys.argv) != 2 or sys.argv[1] not in GATES:
        raise SystemExit("choose compile, focused, tests, clippy, or format")
    gate = sys.argv[1]
    out = ROOT / ".artifacts/e22-v3-fixture-recovery-20261012/logs"
    out.mkdir(parents=True, exist_ok=True)
    stem = f"{gate}-{time.time_ns()}-{os.getpid()}"
    log = out / (stem + ".log")
    record = {"gate": gate, "argv": GATES[gate], "cwd": str(ROOT),
              "head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
              "python": sys.version, "start": datetime.datetime.now(datetime.timezone.utc).isoformat(),
              "source_hashes": hashes(), "log": str(log.relative_to(ROOT)), "status": "started"}
    if gate == "focused":
        record["classification"] = "development runtime only; not full qualification"
    write_new(out / (stem + "-started.json"), record)
    versions = {name: subprocess.check_output([str(TOOLCHAIN / name), "--version"], text=True).strip() for name in ("rustc", "cargo")}
    if not all(value.startswith(name + " 1.99.0 ") for name, value in versions.items()):
        raise SystemExit("Required Rust/Cargo 1.99.0 unavailable")
    record["toolchain"] = versions
    env = dict(os.environ, PYTHONDONTWRITEBYTECODE="1", PATH=str(TOOLCHAIN)+os.pathsep+os.environ.get("PATH", ""), CARGO_HOME="/tmp/c2-route-receipt-cargo-home", CARGO_TARGET_DIR=str(out / "target"), CARGO_INCREMENTAL="0", CARGO_PROFILE_DEV_DEBUG="0", CARGO_PROFILE_TEST_DEBUG="0", CARGO_BUILD_JOBS="2")
    env["KAIRO_C2_PROCESS_ARTIFACT_DIR"] = str(out / (stem + "-process"))
    command_exit_code = None
    with log.open("x") as stream:
        if gate == "tests" and any(not (ROOT / source).is_file() for source in SOURCES):
            stream.write("Gate refused: declared test file is missing.\n")
            exit_code = 2
            record["gate_error"] = "Refused: declared test file is missing"
        else:
            command_exit_code = subprocess.run(GATES[gate], cwd=ROOT, env=env, stdout=stream, stderr=subprocess.STDOUT).returncode
            exit_code = command_exit_code
    content = log.read_bytes()
    if gate == "tests" and exit_code == 0:
        required = ['native_preview_handles_v2_pending_disable_and_horizon_without_advancing', 'native_preview_disabled_queue_has_no_next_event_or_completion', 'c5_native_preview_preserves_pending_disable_and_completion', 'c5_native_preview_limit_error_propagates_without_v1_fallback', 'coherent_v2_transit_and_control_join_rejects_independent_mutations', 'coherent_v2_requires_observed_domain_frontier', 'coherent_v2_observed_arrival_requires_queued_submit_join', 'nested_queue_and_allocation_order_corruption_is_rejected_by_nonowning_scan', 'full_owner_image_restores_in_a_new_process_and_matches_uninterrupted_suffix']
        required += ['portable_v2_gated_process_continuation', 'portable_v1_full_envelope_golden_parity', 'portable_v2_malformed_inner_sections_reject_before_codecs', 'deferred_submit_and_release_wait_for_explicit_flush', 'deferred_calendar_gate_blocks_flush', 'deferred_completion_created_readiness_precedes_resume', 'deferred_flush_admission_rejections_are_atomic', 'deferred_precommit_halt_preserves_pending_flush', 'deferred_legacy_capture_rejects_before_codecs']
        required += ['native_v3_roundtrip_preserves_complete_deferred_state', 'native_v3_mixed_interior_cut_preserves_continuation', 'native_v3_preflush_halt_preserves_pending_state', 'native_v3_bounds_reject_before_codecs', 'native_v3_malformed_modes_and_flush_joins_reject_before_codecs', 'native_v3_capture_cache_corruption_rejects_before_codecs', 'native_v3_legacy_normalization_preserves_v1_v2_behavior', 'native_v3_legacy_capture_rejects_deferred_before_codecs', 'native_v3_current_tick_flush_restores_and_readmits']
        missing = [name for name in required if not re.search(rb"test [^\n]*" + name.encode() + rb" \.\.\. ok", content)]
        suites = [b"kairo_ecs_core-", b"kairo_ecs_des-", b"kairo_ecs_abm-", b"kairo_ecs_calibration-", b"flow_checkpoint_v2", b"flow_checkpoint_wire_v2", b"flow_resource_grant_gate_v1"]
        missing += [name.decode() for name in suites if name not in content]
        if missing:
            exit_code = 2
            record["gate_error"] = "Missing successful required tests/suites: " + repr(missing)
    if gate == "focused" and exit_code == 0:
        required = ['native_v3_roundtrip_preserves_complete_deferred_state', 'native_v3_mixed_interior_cut_preserves_continuation', 'native_v3_preflush_halt_preserves_pending_state', 'native_v3_bounds_reject_before_codecs', 'native_v3_malformed_modes_and_flush_joins_reject_before_codecs', 'native_v3_capture_cache_corruption_rejects_before_codecs', 'native_v3_legacy_normalization_preserves_v1_v2_behavior', 'native_v3_legacy_capture_rejects_deferred_before_codecs', 'native_v3_current_tick_flush_restores_and_readmits']
        missing = [name for name in required if not re.search(rb"test [^\n]*" + name.encode() + rb" \.\.\. ok", content)]
        if missing:
            exit_code = 2
            record["gate_error"] = "Missing focused native V3 successes: " + repr(missing)
        record["classification"] = "development runtime only; not full qualification"
    after = hashes()
    if after != record["source_hashes"]:
        exit_code = 2
        record["gate_error"] = "Owned source changed during command"
    record.update(status="finished", end=datetime.datetime.now(datetime.timezone.utc).isoformat(),
                  exit_code=exit_code, command_exit_code=command_exit_code, log_sha256=hashlib.sha256(content).hexdigest(), source_hashes_after=after)
    result = out / (stem + "-result.json")
    write_new(result, record)
    print(json.dumps({"exit_code": exit_code, "receipt": str(result.relative_to(ROOT)), "log": record["log"]}))
    if exit_code:
        print(content.decode(errors="replace")[-3000:])
    return exit_code

if __name__ == "__main__":
    raise SystemExit(main())
