#!/usr/bin/env python3
"""Validate a bounded LLVM coverage export against the D3.3 line policy."""

import argparse
import json
import math
from pathlib import PurePosixPath
import sys


_PERCENT_TOLERANCE = 1e-6


def _normalized_path(value, label):
    if not isinstance(value, str) or not value.strip() or "\x00" in value:
        raise ValueError(f"{label} must be a nonempty source path")
    normalized = value.replace("\\", "/")
    path = PurePosixPath(normalized)
    parts = path.parts
    if ".." in parts:
        raise ValueError(f"{label} must not traverse parent directories")
    result = str(path)
    if result in ("", "."):
        raise ValueError(f"{label} must be a source file path")
    return result


def _number(value, label):
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise ValueError(f"{label} must be a finite number")
    try:
        number = float(value)
    except OverflowError as exc:
        raise ValueError(f"{label} must be a finite number") from exc
    if not math.isfinite(number):
        raise ValueError(f"{label} must be a finite number")
    return number


def _counts(summary, label):
    if not isinstance(summary, dict):
        raise ValueError(f"{label} summary must be an object")
    for key in ("count", "covered", "percent"):
        if key not in summary:
            raise ValueError(f"{label} summary is missing {key}")
    count, covered = summary["count"], summary["covered"]
    if isinstance(count, bool) or not isinstance(count, int) or count <= 0:
        raise ValueError(f"{label} count must be a positive integer")
    if isinstance(covered, bool) or not isinstance(covered, int) or covered <= 0:
        raise ValueError(f"{label} covered must be a positive integer")
    if covered > count:
        raise ValueError(f"{label} covered cannot exceed count")
    reported = _number(summary["percent"], f"{label} percent")
    try:
        actual = 100.0 * covered / count
    except OverflowError as exc:
        raise ValueError(f"{label} counts exceed supported numeric bounds") from exc
    if not math.isclose(reported, actual, rel_tol=0.0, abs_tol=_PERCENT_TOLERANCE):
        raise ValueError(f"{label} percent {reported:g} is inconsistent with counts ({actual:g})")
    return {"count": count, "covered": covered, "percent": actual}


def _policy(policy):
    if not isinstance(policy, dict):
        raise ValueError("coverage policy must be an object")
    if "file" not in policy or "minimum_line_percent" not in policy:
        raise ValueError("coverage policy requires file and minimum_line_percent")
    target = _normalized_path(policy["file"], "coverage policy file")
    expected = "crates/careops-ed/src/lib.rs"
    if target != expected:
        raise ValueError(f"coverage policy file must be exactly {expected}")
    minimum = _number(policy["minimum_line_percent"], "minimum_line_percent")
    if not 80.0 <= minimum <= 100.0:
        raise ValueError("minimum_line_percent must be between the fixed 80% floor and 100%")
    return target, minimum


def check_coverage(document, policy):
    """Validate the configured production file and return its measured coverage.

    LLVM's rounded percentage is checked against the reported integer counts;
    the returned percentage is calculated from those counts. This reports line
    and optional function coverage for this one file only.
    """
    target, minimum = _policy(policy)
    if not isinstance(document, dict) or not isinstance(document.get("data"), list):
        raise ValueError("coverage document requires a data array")

    matches = []
    seen = set()
    for data_index, entry in enumerate(document["data"]):
        if not isinstance(entry, dict) or not isinstance(entry.get("files"), list):
            raise ValueError(f"data[{data_index}] requires a files array")
        for file_index, file_record in enumerate(entry["files"]):
            label = f"data[{data_index}].files[{file_index}]"
            if not isinstance(file_record, dict) or "filename" not in file_record:
                raise ValueError(f"{label} requires a filename")
            filename = _normalized_path(file_record["filename"], f"{label} filename")
            if filename in seen:
                raise ValueError(f"duplicate source path: {filename}")
            seen.add(filename)
            if filename == target or filename.endswith("/" + target):
                matches.append(file_record)

    if len(matches) != 1:
        if not matches:
            raise ValueError(f"coverage source not found for exact suffix {target}")
        raise ValueError(f"expected one coverage row for {target}, found {len(matches)}")

    record = matches[0]
    summary = record.get("summary")
    if not isinstance(summary, dict) or "lines" not in summary:
        raise ValueError("target source requires summary.lines")
    result = {"file": target, "lines": _counts(summary["lines"], "lines")}
    if result["lines"]["percent"] < minimum:
        raise ValueError(f"line coverage {result['lines']['percent']:.6f}% is below configured floor {minimum:g}%")
    if "functions" in summary:
        result["functions"] = _counts(summary["functions"], "functions")
    return result


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--coverage", required=True, help="LLVM coverage export JSON")
    parser.add_argument("--config", required=True, help="D3.3 quality gate JSON")
    args = parser.parse_args(argv)
    try:
        with open(args.coverage, encoding="utf-8") as stream:
            document = json.load(stream)
        with open(args.config, encoding="utf-8") as stream:
            config = json.load(stream)
        if not isinstance(config, dict) or "coverage" not in config:
            raise ValueError("config requires a coverage policy")
        result = check_coverage(document, config["coverage"])
    except (OSError, json.JSONDecodeError, ValueError) as error:
        print(f"coverage validation failed: {error}", file=sys.stderr)
        return 1
    print(json.dumps(result, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
