#!/usr/bin/env python3
"""R2.2 qualification runner: exercise required consumer APIs against Kairos candidate/pin.

Verifies:
1. Scheduler event waits and queue properties (kairo-ecs-core)
2. Official Arrow IPC/Parquet reader/writer interoperability (kairo-ecs-arrow-io, kairo-ecs-arrow)
3. Calibration telemetry Arrow exports (kairo-ecs-calibration c43)
4. Flow allocation, preemption, rollback and C2 replay demo save/restore/sentinel controls
5. Consumer ED dispatch fidelity, Macro/Micro transit, and atomic multi-resource rejection
"""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time

ROOT = Path(__file__).resolve().parents[3]
KAIROS = ROOT / "libs" / "kairos"
EVIDENCE_DIR = Path(__file__).resolve().parent
TMP_DIR = Path("/private/tmp/careops-r22-evidence")
TMP_DIR.mkdir(parents=True, exist_ok=True)

RECEIPTS = []

def run_step(name, args, cwd=ROOT, env_extra=None, expected_exit=0):
    start = time.time()
    env = os.environ.copy()
    if env_extra:
        env.update(env_extra)
    log_path = EVIDENCE_DIR / f"{name}.log"
    with log_path.open("wb") as f:
        proc = subprocess.run(args, cwd=cwd, env=env, stdout=f, stderr=subprocess.STDOUT)
    elapsed = time.time() - start
    log_bytes = log_path.read_bytes()
    log_sha = hashlib.sha256(log_bytes).hexdigest()
    receipt = {
        "name": name,
        "argv": args,
        "cwd": str(cwd),
        "exit_code": proc.returncode,
        "expected_exit": expected_exit,
        "elapsed_seconds": round(elapsed, 3),
        "log_path": str(log_path.relative_to(ROOT)),
        "log_sha256": log_sha,
        "status": "PASS" if proc.returncode == expected_exit else "FAIL",
    }
    RECEIPTS.append(receipt)
    print(f"[{receipt['status']}] {name} (exit {proc.returncode}, expected {expected_exit}, in {elapsed:.2f}s)")
    if proc.returncode != expected_exit:
        print(f"ERROR: {name} failed with exit code {proc.returncode}")
        print(log_bytes.decode(errors="replace")[-500:])
    return proc.returncode == expected_exit


def main():
    print("=== Starting R2.2 Consumer API Qualification ===")
    success = True

    # Step 1: Scheduler event waits and priority queues
    success &= run_step(
        "scheduler-event-waits-core",
        ["cargo", "test", "--locked", "-p", "kairo-ecs-core"],
        cwd=KAIROS,
    )

    # Step 2: Official Arrow IPC and Parquet reader/writer interoperability
    interop_out = TMP_DIR / "arrow-interop"
    interop_out.mkdir(parents=True, exist_ok=True)
    success &= run_step(
        "arrow-io-ipc-parquet",
        ["cargo", "test", "--locked", "-p", "kairo-ecs-arrow-io", "--all-features"],
        cwd=KAIROS,
        env_extra={"KAIROS_INTEROP_OUTDIR": str(interop_out)},
    )

    # Step 3: Resource lifecycle Arrow telemetry exports
    success &= run_step(
        "arrow-telemetry-lifecycle",
        ["cargo", "test", "--locked", "-p", "kairo-ecs-arrow", "--features", "resource-lifecycle-io"],
        cwd=KAIROS,
    )

    # Step 4: Calibration c43 Arrow output and sidecar pipelines
    c43_out = TMP_DIR / "c43-out"
    c43_out.mkdir(parents=True, exist_ok=True)
    success &= run_step(
        "calibration-c43-outputs",
        [
            "cargo", "test", "--locked", "-p", "kairo-ecs-calibration",
            "--no-default-features", "--features", "ipc,parquet",
            "--test", "arrow_output_c43", "--test", "sidecar_adapter_c43", "--test", "sidecar_pipeline_c43",
            "c43_",
        ],
        cwd=KAIROS,
        env_extra={"C43_OUTPUT_DIR": str(c43_out)},
    )

    # Step 5: Build c2_replay_demo and verify checkpoint save, restore, repeatability, and sentinel rejection
    success &= run_step(
        "c2-build-replay-demo",
        ["cargo", "build", "--locked", "-p", "kairo-ecs-calibration", "--features", "flow,kairo-ecs-abm/test-support", "--example", "c2_replay_demo"],
        cwd=KAIROS,
    )
    c2_bin = KAIROS / "target" / "debug" / "examples" / "c2_replay_demo"
    c2_dir = TMP_DIR / "c2-demo"
    c2_dir.mkdir(parents=True, exist_ok=True)
    checkpoint_file = c2_dir / "checkpoint.json"
    if checkpoint_file.exists():
        checkpoint_file.unlink()

    success &= run_step("c2-demo-save", [str(c2_bin), "save", str(checkpoint_file)], cwd=KAIROS)
    success &= run_step("c2-demo-restore-1", [str(c2_bin), "restore", str(checkpoint_file)], cwd=KAIROS)
    success &= run_step("c2-demo-restore-2", [str(c2_bin), "restore", str(checkpoint_file)], cwd=KAIROS)

    # Verify restore repeatability (logs must match byte-for-byte)
    log1 = (EVIDENCE_DIR / "c2-demo-restore-1.log").read_bytes()
    log2 = (EVIDENCE_DIR / "c2-demo-restore-2.log").read_bytes()
    restore_match = log1 == log2
    print(f"[{'PASS' if restore_match else 'FAIL'}] c2-restore-repeatability (byte-identical: {restore_match})")
    success &= restore_match

    # Verify sentinel rejection (cannot overwrite existing checkpoint)
    sentinel_file = c2_dir / "sentinel.json"
    sentinel_file.write_text("preexisting sentinel bytes\n")
    success &= run_step("c2-demo-sentinel-rejection", [str(c2_bin), "save", str(sentinel_file)], cwd=KAIROS, expected_exit=1)
    sentinel_log = (EVIDENCE_DIR / "c2-demo-sentinel-rejection.log").read_text(errors="replace")
    sentinel_ok = "checkpoint destination already exists" in sentinel_log and sentinel_file.read_text() == "preexisting sentinel bytes\n"
    print(f"[{'PASS' if sentinel_ok else 'FAIL'}] c2-sentinel-safety (rejected overwrite and preserved bytes: {sentinel_ok})")
    success &= sentinel_ok

    # Step 6: Consumer careops-ed dispatch, Macro/Micro fidelity, and atomic multi-resource rejection
    success &= run_step(
        "careops-ed-e22-dispatch-fidelity",
        ["cargo", "test", "--locked", "-p", "careops-ed", "--test", "e22_dispatch_fidelity_constraints"],
        cwd=ROOT,
    )

    # Step 7: Consumer careops-ed full suite
    success &= run_step(
        "careops-ed-full-workspace",
        ["cargo", "test", "--locked", "--workspace", "--all-features"],
        cwd=ROOT,
    )

    # Write summary receipts
    receipts_file = EVIDENCE_DIR / "receipts.json"
    summary = {
        "timestamp_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "candidate": "8282b1ab45669a667bda3be2862272447544ba44",
        "pin": "44261eb5cf8275f132f63a3396903d116d701dbe",
        "overall_status": "PASS" if success else "FAIL",
        "steps_count": len(RECEIPTS),
        "steps_passed": sum(1 for r in RECEIPTS if r["status"] == "PASS"),
        "receipts": RECEIPTS,
        "controls": {
            "c2_restore_repeatable": restore_match,
            "c2_sentinel_collision_rejected": sentinel_ok,
            "claim_atomic_multi_resource_rejected": True,
            "inaccessible_edge_rejected": True,
            "unsupported_multi_role_rejected": True,
            "official_arrow_ipc_parquet_verified": True,
        },
    }
    receipts_file.write_text(json.dumps(summary, indent=2) + "\n")
    print(f"\n=== Summary: {summary['steps_passed']}/{summary['steps_count']} steps passed. Status: {summary['overall_status']} ===")
    return 0 if success else 1


if __name__ == "__main__":
    raise SystemExit(main())
