#!/usr/bin/env python3
"""R3.3 rollback rehearsal script: proves safe recovery and rollback of submodule pin
in an isolated worktree without modifying or disrupting the primary repository checkout.
"""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import time

ROOT = Path(__file__).resolve().parents[3]
EVIDENCE_DIR = Path(__file__).resolve().parent
REHEARSAL_DIR = Path("/private/tmp/careops-r33-rollback-rehearsal-run")

LOG_LINES = []

def log(msg):
    print(msg)
    LOG_LINES.append(msg)

def run():
    log("=== Starting R3.3 Submodule Pin Rollback Rehearsal ===")
    start_time = time.time()
    
    if REHEARSAL_DIR.exists():
        shutil.rmtree(REHEARSAL_DIR)

    log(f"1. Creating isolated git worktree at {REHEARSAL_DIR}...")
    subprocess.run(["git", "worktree", "add", "--detach", str(REHEARSAL_DIR), "HEAD"], cwd=ROOT, check=True)

    try:
        log("2. Initializing submodule libs/kairos in isolated worktree...")
        subprocess.run(["git", "submodule", "update", "--init", "libs/kairos"], cwd=REHEARSAL_DIR, check=True)

        res1 = subprocess.run(["git", "rev-parse", "HEAD:libs/kairos"], cwd=REHEARSAL_DIR, capture_output=True, text=True, check=True)
        initial_pin = res1.stdout.strip()
        log(f"   Initial parent gitlink: {initial_pin}")
        assert initial_pin == "44261eb5cf8275f132f63a3396903d116d701dbe"

        log("3. Simulating pin advance to candidate 8282b1ab in isolated worktree...")
        subprocess.run(["git", "checkout", "8282b1ab45669a667bda3be2862272447544ba44"], cwd=REHEARSAL_DIR / "libs/kairos", check=True)
        subprocess.run(["git", "add", "libs/kairos"], cwd=REHEARSAL_DIR, check=True)
        subprocess.run(["git", "commit", "-m", "chore: advance pin to candidate 8282b1ab (simulated)"], cwd=REHEARSAL_DIR, check=True)

        res2 = subprocess.run(["git", "rev-parse", "HEAD:libs/kairos"], cwd=REHEARSAL_DIR, capture_output=True, text=True, check=True)
        advanced_pin = res2.stdout.strip()
        log(f"   Advanced parent gitlink: {advanced_pin}")
        assert advanced_pin == "8282b1ab45669a667bda3be2862272447544ba44"

        log("4. Executing documented rollback procedure (git revert + submodule update)...")
        subprocess.run(["git", "revert", "--no-edit", "HEAD"], cwd=REHEARSAL_DIR, check=True)
        subprocess.run(["git", "submodule", "update", "--init", "libs/kairos"], cwd=REHEARSAL_DIR, check=True)

        res3 = subprocess.run(["git", "rev-parse", "HEAD:libs/kairos"], cwd=REHEARSAL_DIR, capture_output=True, text=True, check=True)
        reverted_pin = res3.stdout.strip()
        log(f"   Reverted parent gitlink: {reverted_pin}")
        assert reverted_pin == "44261eb5cf8275f132f63a3396903d116d701dbe"

        res4 = subprocess.run(["git", "rev-parse", "HEAD"], cwd=REHEARSAL_DIR / "libs/kairos", capture_output=True, text=True, check=True)
        submodule_head = res4.stdout.strip()
        log(f"   Submodule HEAD commit:   {submodule_head}")
        assert submodule_head == "44261eb5cf8275f132f63a3396903d116d701dbe"

        elapsed = time.time() - start_time
        log(f"5. Rollback rehearsal completed successfully in {elapsed:.2f}s!")
        status = "PASS"
    except Exception as e:
        log(f"ERROR during rollback rehearsal: {e}")
        status = "FAIL"
    finally:
        log("6. Cleaning up isolated worktree...")
        subprocess.run(["git", "worktree", "remove", "--force", str(REHEARSAL_DIR)], cwd=ROOT, check=True)

    log_content = "\n".join(LOG_LINES) + "\n"
    log_file = EVIDENCE_DIR / "rollback-rehearsal.log"
    log_file.write_text(log_content)

    receipt = {
        "timestamp_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "status": status,
        "initial_pin": "44261eb5cf8275f132f63a3396903d116d701dbe",
        "simulated_candidate": "8282b1ab45669a667bda3be2862272447544ba44",
        "reverted_pin": "44261eb5cf8275f132f63a3396903d116d701dbe",
        "rehearsal_log_sha256": hashlib.sha256(log_content.encode("utf-8")).hexdigest(),
        "elapsed_seconds": round(time.time() - start_time, 3),
    }
    (EVIDENCE_DIR / "rollback-rehearsal-receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    return 0 if status == "PASS" else 1

if __name__ == "__main__":
    raise SystemExit(run())
