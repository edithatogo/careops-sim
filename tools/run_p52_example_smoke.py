#!/usr/bin/env python3
"""P5.2 lean smoke: run executable E0 scenario examples twice and compare output."""

from __future__ import annotations

import hashlib
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EXAMPLES = [
    "crates/careops-ed/examples/one_patient.json",
    "crates/careops-ed/examples/nominal_mvp.json",
    "crates/careops-ed/examples/constrained_mvp.json",
    "model-inputs/ed/profiles/p4-minimal-deterministic.json",
    "model-inputs/ed/profiles/p4-generic-nominal.json",
    "model-inputs/ed/profiles/p4-constrained-surge.json",
]


def run(path: str) -> str:
    result = subprocess.run(
        ["cargo", "run", "--quiet", "--locked", "-p", "careops-ed-cli", "--", "run", path],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise RuntimeError(f"{path}: {result.stderr.strip()}")
    return result.stdout.strip()


def main() -> int:
    errors = []
    for rel in EXAMPLES:
        try:
            first = run(rel)
            second = run(rel)
            if first != second:
                errors.append(f"{rel}: non-repeatable output")
            else:
                h = hashlib.sha256(first.encode()).hexdigest()
                print(f"PASS {rel}: {h}")
        except RuntimeError as exc:
            errors.append(str(exc))
    if errors:
        for e in errors:
            print(f"ERROR: {e}", file=sys.stderr)
        return 1
    print("P5.2 smoke PASS: executable scenario examples repeat identically")
    return 0


if __name__ == "__main__":
    sys.exit(main())