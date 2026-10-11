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
GATES = {
    "fixtures": ["python3", "-B", "-m", "unittest", "discover", "-s", "tests", "-p", "test_e21_interrupt.py", "-v"],
    "context": ["python3", "-B", "tools/context.py", "check"],
    "mvp": ["python3", "-B", "tools/mvp.py", "check"],
}
SOURCES = ["conductor/design/ed/e2-interrupt.json", "tests/test_e21_interrupt.py", "conductor/design/ed/e2-interrupt-crosswalk.md"]

def hashes():
    return {p: hashlib.sha256((ROOT / p).read_bytes()).hexdigest() if (ROOT / p).is_file() else None for p in SOURCES}

def write_new(path, value):
    with path.open("x") as stream:
        json.dump(value, stream, indent=2)
        stream.write("\n")

def main():
    if len(sys.argv) != 2 or sys.argv[1] not in GATES:
        raise SystemExit("choose fixtures, context, or mvp")
    gate = sys.argv[1]
    out = ROOT / ".artifacts/mvp/E2.1.interrupt.fixtures/logs"
    out.mkdir(parents=True, exist_ok=True)
    stem = f"{gate}-{time.time_ns()}-{os.getpid()}"
    log = out / (stem + ".log")
    record = {"gate": gate, "argv": GATES[gate], "cwd": str(ROOT),
              "head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
              "python": sys.version, "start": datetime.datetime.now(datetime.timezone.utc).isoformat(),
              "source_hashes": hashes(), "log": str(log.relative_to(ROOT)), "status": "started"}
    write_new(out / (stem + "-started.json"), record)
    env = dict(os.environ, PYTHONDONTWRITEBYTECODE="1")
    command_exit_code = None
    with log.open("x") as stream:
        if gate == "fixtures" and not (ROOT / SOURCES[1]).is_file():
            stream.write("Gate refused: declared test file is missing.\n")
            exit_code = 2
            record["gate_error"] = "Refused: declared test file is missing"
        else:
            command_exit_code = subprocess.run(GATES[gate], cwd=ROOT, env=env, stdout=stream, stderr=subprocess.STDOUT).returncode
            exit_code = command_exit_code
    content = log.read_bytes()
    observed = re.search(rb"Ran (\d+) tests?", content)
    if gate == "fixtures" and exit_code == 0 and (not observed or int(observed.group(1)) == 0):
        exit_code = 2
        record["gate_error"] = "Successful command did not report a nonzero test count"
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
