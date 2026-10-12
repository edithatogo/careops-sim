"""Immutable receipt driver for the bounded caller context codec schema-version packet."""
import datetime
import hashlib
import json
import os
import re
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
ART = ROOT / ".artifacts/e22-context-schema-versions-20261012/dispatch"
PACKET_PATH = ART / "packet.json"
RESULT_REL = ".artifacts/e22-context-schema-versions-20261012/dispatch/logs/result.json"
LOG_PREFIX = ".artifacts/e22-context-schema-versions-20261012/dispatch/logs/"
TOOLCHAIN = Path("/Users/doughnut/.rustup/toolchains/1.99.0-aarch64-apple-darwin/bin")
AUTHORITY_PATH = Path("/tmp/careops-e22-context-schema-versions-dispatch-authority-20261012.json")
OWNED = ['crates/kairo-ecs-des/src/flow/checkpoint.rs', 'crates/kairo-ecs-des/tests/flow_context_codec_schema_versions.rs']

GATE_ORDER = ["clippy", "tests", "format"]
GATES = {
    "clippy": [str(TOOLCHAIN / "cargo"), "clippy", "--locked", "-p", "kairo-ecs-des", "--all-targets", "--", "-D", "warnings"],
    "tests": [str(TOOLCHAIN / "cargo"), "test", "--locked", "-p", "kairo-ecs-des", "--", "--test-threads=1"],
    "format": [str(TOOLCHAIN / "rustfmt"), "--edition", "2021", "--check", "--config", "skip_children=true", *OWNED],
}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def file_hash(path):
    candidate = ROOT / path
    return digest(candidate.read_bytes()) if candidate.is_file() else None


def git(*args):
    proc = subprocess.run(["git", *args], cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    if proc.returncode:
        raise SystemExit("git evidence check failed")
    return proc.stdout


def git_text(*args):
    return git(*args).decode("utf-8").strip()


def source_hashes(require_present=False):
    hashes = {path: file_hash(path) for path in OWNED}
    if require_present and any(value is None for value in hashes.values()):
        raise SystemExit("an owned source file is missing")
    return hashes


def base_tree_hashes():
    hashes = {}
    for path in OWNED:
        probe = subprocess.run(["git", "cat-file", "-e", f"HEAD:{path}"], cwd=ROOT,
                               stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        hashes[path] = digest(git("show", f"HEAD:{path}")) if probe.returncode == 0 else None
    return hashes


def changed_paths():
    raw = git("status", "--porcelain=v1", "-z", "--untracked-files=all")
    return {entry[3:] for entry in raw.decode("utf-8").split("\0") if entry}


def packet_authority_check(packet):
    authority = packet["dispatch_authority_path"]
    if authority != str(AUTHORITY_PATH):
        raise SystemExit("dispatch authority path differs from immutable driver")
    try:
        data = json.loads(AUTHORITY_PATH.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        raise SystemExit("root dispatch authority is missing or invalid")
    expected = {
        "packet_path": str(PACKET_PATH.relative_to(ROOT)),
        "packet_sha256": digest(PACKET_PATH.read_bytes()),
        "driver_sha256": digest(Path(__file__).read_bytes()),
        "target_worktree": str(ROOT),
        "base_commit": packet["base_commit"],
        "dispatch_status": "GO",
    }
    if any(data.get(key) != value for key, value in expected.items()):
        raise SystemExit("root dispatch authority does not bind this packet and base")
    return expected["packet_sha256"]


def authorize_failed_recovery(packet, rows, action):
    failures = [str(row["result_path"].relative_to(ROOT)) for row in rows if row["result"].get("exit_code") != 0]
    authority = json.loads(AUTHORITY_PATH.read_text(encoding="utf-8"))
    approved = authority.get("repair_authorized_after_failures", [])
    if failures:
        if authority.get("repair_authorized") is not True or approved != failures:
            raise SystemExit("failed receipts require fresh root authorization of the complete chronological list")
        last = max(i for i, row in enumerate(rows) if row["result"].get("exit_code") != 0)
        recovery = rows[last + 1:]
    else:
        if authority.get("repair_authorized") is True or approved:
            raise SystemExit("repair authority has no matching failures")
        recovery = rows
    if [row["result"].get("gate") for row in recovery] != GATE_ORDER[:len(recovery)]:
        raise SystemExit("gate recovery must restart Clippy and follow exact order")
    if action != "finalize" and (len(recovery) >= len(GATE_ORDER) or action != GATE_ORDER[len(recovery)]):
        raise SystemExit("gate call violates the ordered execution/recovery sequence")



def bound_input_check(packet):
    for path, expected in packet["input_hashes"].items():
        if path in OWNED:
            continue
        if file_hash(path) != expected:
            raise SystemExit("a bound immutable input changed")
    evidence = packet["external_evidence"]
    acceptance_path = Path(evidence["native_acceptance_path"])
    if not acceptance_path.is_file() or digest(acceptance_path.read_bytes()) != evidence["native_acceptance_sha256"]:
        raise SystemExit("accepted native durable evidence changed")
    accepted = json.loads(acceptance_path.read_text(encoding="utf-8"))
    if (accepted.get("status") != evidence["native_acceptance_status"]
            or accepted.get("accepted_commit") != evidence["native_accepted_commit"]
            or accepted.get("final_result_sha256") != evidence["native_final_result_sha256"]):
        raise SystemExit("accepted native envelope status/commit/result binding differs")
    if base_tree_hashes() != packet["base_source_hashes"]:
        raise SystemExit("owned source baseline differs from packet")
    if git_text("rev-parse", "HEAD") != packet["base_commit"]:
        raise SystemExit("base HEAD differs from packet")


def source_scope_check(packet):
    allowed = set(OWNED)
    outside = {
        path for path in changed_paths()
        if path not in allowed and not path.startswith(LOG_PREFIX)
    }
    if outside:
        raise SystemExit("working tree contains a path outside the two-source context-version packet")
    for path in packet["initially_absent_paths"]:
        if path not in OWNED:
            raise SystemExit("initially absent path is outside the two-source context-version packet")
        probe = subprocess.run(["git", "cat-file", "-e", f"{packet['base_commit']}:{path}"],
                               cwd=ROOT, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        if probe.returncode == 0:
            raise SystemExit("initially absent source path unexpectedly exists in base")


def write_new(path, value):
    with path.open("x", encoding="utf-8") as stream:
        json.dump(value, stream, indent=2)
        stream.write("\n")


def parse_test_log(log_bytes, required):
    text = log_bytes.decode("utf-8", errors="replace")
    passed = re.findall(r"^test (\S+) \.\.\. ok$", text, flags=re.MULTILINE)
    failed = re.findall(r"^test (\S+) \.\.\. FAILED$", text, flags=re.MULTILINE)
    missing = [name for name in required if not any(actual == name or actual.endswith("::" + name) for actual in passed)]
    summary_rows = re.findall(r"test result: .*?(\d+) passed; (\d+) failed; (\d+) ignored", text)
    return {
        "successful_named_tests": passed,
        "failed_named_tests": failed,
        "successful_test_count": sum(int(row[0]) for row in summary_rows),
        "failed_test_count": sum(int(row[1]) for row in summary_rows),
        "ignored_test_count": sum(int(row[2]) for row in summary_rows),
        "required_success_names": required,
        "missing_required_success_names": missing,
    }


def receipts(logs, packet_sha, packet):
    started = {path.name[:-len("-started.json")]: path for path in logs.glob("*-started.json")}
    results = {path.name[:-len("-result.json")]: path for path in logs.glob("*-result.json") if path.name != "result.json"}
    if set(started) != set(results):
        raise SystemExit("unmatched started or terminal gate receipt")
    rows = []
    for stem, started_path in started.items():
        start = json.loads(started_path.read_text(encoding="utf-8"))
        result_path = results[stem]
        result = json.loads(result_path.read_text(encoding="utf-8"))
        if start.get("packet_sha256") != packet_sha or result.get("packet_sha256") != packet_sha:
            raise SystemExit("receipt is not bound to this packet")
        if start.get("head") != packet["base_commit"] or result.get("head") != packet["base_commit"]:
            raise SystemExit("receipt head differs from exact packet base")
        if start.get("argv") != GATES.get(start.get("gate")) or result.get("gate") != start.get("gate"):
            raise SystemExit("receipt argv or gate differs from immutable driver")
        if start.get("status") != "started" or result.get("status") != "finished":
            raise SystemExit("receipt pair has an invalid lifecycle status")
        if result.get("source_hashes") != start.get("source_hashes"):
            raise SystemExit("terminal receipt source snapshot differs from its start record")
        log_path = ROOT / start["log"]
        if not log_path.is_file() or digest(log_path.read_bytes()) != result.get("log_sha256"):
            raise SystemExit("receipt raw log is missing or changed")
        if result.get("started_receipt_sha256") != digest(started_path.read_bytes()):
            raise SystemExit("terminal receipt does not bind its start receipt")
        if result.get("toolchain") != start.get("toolchain"):
            raise SystemExit("terminal receipt toolchain differs from started receipt")
        versions = result.get("toolchain", {})
        if not versions.get("rustc", "").startswith("rustc 1.99.0 ") or not versions.get("cargo", "").startswith("cargo 1.99.0 "):
            raise SystemExit("receipt is not bound to Rust/Cargo 1.99.0")
        rows.append({"started": start, "started_path": started_path, "result": result,
                     "result_path": result_path, "when": int(start["start_ns"])})
    return sorted(rows, key=lambda row: row["when"])


def latest_by_gate(rows):
    latest = {gate: None for gate in GATE_ORDER}
    for row in rows:
        gate = row["result"].get("gate")
        if gate in latest:
            latest[gate] = row
    return latest


def require_prior_gates(gate, current, rows):
    index = GATE_ORDER.index(gate)
    latest = latest_by_gate(rows)
    for prior in GATE_ORDER[:index]:
        row = latest[prior]
        if row is None or row["result"].get("exit_code") != 0:
            raise SystemExit("prior gate has no latest passing receipt")
        if row["result"].get("source_hashes") != current or row["result"].get("source_hashes_after") != current:
            raise SystemExit("prior gate was run at different source hashes")


def main():
    if len(sys.argv) != 2 or sys.argv[1] not in {*GATE_ORDER, "finalize"}:
        raise SystemExit("choose exactly one action: clippy, tests, format, or finalize")
    packet = json.loads(PACKET_PATH.read_text(encoding="utf-8"))
    if packet.get("source_write_paths") != OWNED or packet.get("gate_source_paths") != OWNED:
        raise SystemExit("packet source scope differs from immutable driver")
    if packet.get("verification_order") != GATE_ORDER:
        raise SystemExit("packet gate order differs from immutable driver")
    rows_by_id = {row["id"]: row for row in packet.get("verification", [])}
    if set(rows_by_id) != set(GATE_ORDER) or any(rows_by_id[name].get("actual_argv") != GATES[name] for name in GATE_ORDER):
        raise SystemExit("packet gate commands differ from immutable driver")
    if packet.get("required_success_names") != rows_by_id["tests"].get("required_success_names"):
        raise SystemExit("packet required test names differ from immutable driver contract")
    packet_sha = packet_authority_check(packet)
    source_scope_check(packet)
    bound_input_check(packet)
    logs = ROOT / packet["logs_path"]
    logs.mkdir(parents=True, exist_ok=True)
    prior = receipts(logs, packet_sha, packet)
    failures = sum(row["result"].get("exit_code") != 0 for row in prior)
    authorize_failed_recovery(packet, prior, sys.argv[1])

    if sys.argv[1] == "finalize":
        current = source_hashes(require_present=True)
        latest = latest_by_gate(prior)
        ordered = [latest[name] for name in GATE_ORDER]
        ready = all(row is not None for row in ordered)
        if ready:
            ready = (ordered[0]["when"] < ordered[1]["when"] < ordered[2]["when"]
                     and all(row["result"].get("exit_code") == 0 for row in ordered)
                     and all(row["result"].get("source_hashes") == current
                             and row["result"].get("source_hashes_after") == current
                             and not row["result"].get("post_run_integrity_errors") for row in ordered))
        if ready:
            ready = not latest["tests"]["result"].get("missing_required_success_names") and not latest["tests"]["result"].get("failed_test_count")
        status = "ready_for_review" if ready else ("stopped_failure_ceiling" if failures >= packet["development_failure_limit"] else "stopped_failure_or_incomplete")
        gate_rows = []
        for gate in GATE_ORDER:
            row = latest[gate]
            if row is not None:
                gate_rows.append({**row["result"], "receipt_path": str(row["result_path"].relative_to(ROOT)),
                                  "receipt_sha256": digest(row["result_path"].read_bytes())})
        result = {
            "status": status, "ready_for_review": ready, "base_commit": packet["base_commit"],
            "head": git_text("rev-parse", "HEAD"), "packet_sha256": packet_sha,
            "source_hashes": current, "source_hashes_after": current,
            "protected_source_hashes": packet["protected_candidate_source_hashes"],
            "gates": gate_rows, "required_success_names": packet["required_success_names"],
            "fresh_failed_development_attempts": failures, "all_receipts": [
                {"started_path": str(row["started_path"].relative_to(ROOT)),
                 "started_sha256": digest(row["started_path"].read_bytes()),
                 "result_path": str(row["result_path"].relative_to(ROOT)),
                 "result_sha256": digest(row["result_path"].read_bytes()),
                 "log_path": row["started"]["log"], "log_sha256": row["result"]["log_sha256"]}
                for row in prior],
        }
        write_new(ROOT / packet["result_path"], result)
        print(json.dumps({"status": status, "result_path": packet["result_path"],
                          "result_sha256": digest((ROOT / packet["result_path"]).read_bytes()),
                          "failed_development_attempts": failures}))
        return 0

    gate = sys.argv[1]
    if failures >= packet["development_failure_limit"]:
        raise SystemExit("development failure ceiling reached; stop")
    current = source_hashes(require_present=True)
    require_prior_gates(gate, current, prior)
    rustc = subprocess.run([str(TOOLCHAIN / "rustc"), "--version"], cwd=ROOT,
                           stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    cargo = subprocess.run([str(TOOLCHAIN / "cargo"), "--version"], cwd=ROOT,
                           stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    if rustc.returncode or cargo.returncode or not rustc.stdout.startswith("rustc 1.99.0 ") or not cargo.stdout.startswith("cargo 1.99.0 "):
        raise SystemExit("Rust/Cargo 1.99.0 is unavailable")
    versions = {"rustc": rustc.stdout.strip(), "cargo": cargo.stdout.strip()}
    stem = f"{gate}-{time.time_ns()}-{os.getpid()}"
    log = logs / f"{stem}.log"
    started = {"gate": gate, "argv": GATES[gate], "cwd": str(ROOT), "head": git_text("rev-parse", "HEAD"),
               "packet_sha256": packet_sha, "start_ns": time.time_ns(),
               "start": datetime.datetime.now(datetime.timezone.utc).isoformat(),
               "toolchain": versions, "source_hashes": current, "log": str(log.relative_to(ROOT)),
               "status": "started"}
    started_path = logs / f"{stem}-started.json"
    write_new(started_path, started)
    env = dict(os.environ)
    env.update(packet["environment"])
    env["PATH"] = str(TOOLCHAIN) + os.pathsep + env.get("PATH", "")
    with log.open("x", encoding="utf-8") as stream:
        command_exit = subprocess.run(GATES[gate], cwd=ROOT, env=env, stdout=stream,
                                      stderr=subprocess.STDOUT).returncode
    log_bytes = log.read_bytes()
    after = source_hashes(require_present=True)
    integrity_errors = []
    try:
        source_scope_check(packet)
        bound_input_check(packet)
    except SystemExit as exc:
        integrity_errors.append(str(exc))
    if after != current:
        integrity_errors.append("owned source changed while gate ran")
    receipt = {**started, "status": "finished", "end": datetime.datetime.now(datetime.timezone.utc).isoformat(),
               "exit_code": command_exit, "command_exit_code": command_exit,
               "source_hashes_after": after, "log_sha256": digest(log_bytes),
               "started_receipt_path": str(started_path.relative_to(ROOT)),
               "started_receipt_sha256": digest(started_path.read_bytes()),
               "post_run_integrity_errors": integrity_errors}
    if gate == "tests":
        summary = parse_test_log(log_bytes, packet["required_success_names"])
        receipt.update(summary)
        if command_exit == 0 and (summary["missing_required_success_names"] or summary["failed_test_count"]):
            receipt["exit_code"] = 2
            receipt["gate_error"] = "required context schema-version proof missing or failed"
    if integrity_errors:
        receipt["exit_code"] = 2
        receipt["gate_error"] = "; ".join(integrity_errors)
    result_path = logs / f"{stem}-result.json"
    write_new(result_path, receipt)
    print(json.dumps({"gate": gate, "exit_code": receipt["exit_code"],
                      "receipt_path": str(result_path.relative_to(ROOT)),
                      "receipt_sha256": digest(result_path.read_bytes()),
                      "log_path": str(log.relative_to(ROOT)), "log_sha256": digest(log_bytes),
                      "successful_test_count": receipt.get("successful_test_count"),
                      "failed_test_count": receipt.get("failed_test_count"),
                      "ignored_test_count": receipt.get("ignored_test_count")}))
    return int(receipt["exit_code"])


if __name__ == "__main__":
    raise SystemExit(main())
