#!/usr/bin/env python3
"""Validate one source-bound cargo-mutants 27.1.0 careops-ed result."""

import argparse
from datetime import datetime
import hashlib
import json
import math
from pathlib import Path
import sys


SOURCE_FILE = "crates/careops-ed/src/lib.rs"
SOURCE_SHA256 = "4f7917f3976d84b86a1b2b6837065c803d75b29d6b4f594abbbb7ceaedc57380"
CATALOG_SHA256 = "a2cb9d0d444dece53af296a0082b18dcdd4ad58616a2ceea58df2b0f3c86194e"
MUTANTS_VERSION = "27.1.0"
EXPECTED_TOTAL = 115
EQUIVALENT_MISSES = {
    (735, 60): "crates/careops-ed/src/lib.rs:735:60: replace || with && in run_scenario",
    (798, 43): "crates/careops-ed/src/lib.rs:798:43: replace || with && in run_scenario",
}
ORIGINAL_UNVIABLES = {
    "crates/careops-ed/src/lib.rs:239:5: replace parse_scenario_json -> Result<ScenarioConfig, ScenarioError> with Ok(Default::default())",
    "crates/careops-ed/src/lib.rs:635:5: replace invalid -> Result<T, ScenarioError> with Ok(Default::default())",
    "crates/careops-ed/src/lib.rs:653:5: replace run_scenario -> Result<RunSummary, SimulationError> with Ok(Default::default())",
}
_TOP_LEVEL_KEYS = {
    "cargo_mutants_version", "caught", "end_time", "missed", "outcomes",
    "start_time", "success", "timeout", "total_mutants", "unviable",
}
_ROW_KEYS = {"diff_path", "log_path", "phase_results", "scenario", "summary"}
_PHASE_KEYS = {"argv", "duration", "phase", "process_status"}
_MUTANT_KEYS = {"file", "function", "genre", "name", "package", "replacement", "span"}
_CATALOG_KEYS = _MUTANT_KEYS | {"diff"}


def _object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def _reject_constant(value):
    raise ValueError(f"non-finite JSON value: {value}")


def _require_keys(value, keys, label):
    if not isinstance(value, dict) or set(value) != keys:
        raise ValueError(f"{label} must have exactly these fields: {', '.join(sorted(keys))}")


def _positive_int(value, label):
    if isinstance(value, bool) or not isinstance(value, int) or value <= 0:
        raise ValueError(f"{label} must be a positive integer")
    return value


def _nonnegative_int(value, label):
    if isinstance(value, bool) or not isinstance(value, int) or value < 0:
        raise ValueError(f"{label} must be a nonnegative integer")
    return value


def _point(value, label):
    _require_keys(value, {"line", "column"}, label)
    return (_positive_int(value["line"], f"{label}.line"),
            _positive_int(value["column"], f"{label}.column"))


def _span(value, label):
    _require_keys(value, {"start", "end"}, label)
    start = _point(value["start"], f"{label}.start")
    end = _point(value["end"], f"{label}.end")
    if end < start:
        raise ValueError(f"{label} end precedes start")
    return start, end


def _process_status(value, label):
    if value == "Success" and not isinstance(value, bool):
        return "Success"
    if isinstance(value, dict) and set(value) == {"Failure"}:
        code = value["Failure"]
        if isinstance(code, bool) or not isinstance(code, int) or code == 0:
            raise ValueError(f"{label} failure status requires a nonzero integer exit code")
        return "Failure"
    raise ValueError(f"{label} process_status must be Success or a Failure exit code")


def _cargo_identity(value):
    if value == "cargo":
        return value
    if not isinstance(value, str):
        raise ValueError("expected Cargo executable must be a string")
    path = Path(value)
    if (not path.is_absolute() or ".." in path.parts or len(path.parts) < 3
            or path.parts[-2:] != ("bin", "cargo")
            or path.parts[-3] not in {"1.99.0-x86_64-unknown-linux-gnu", "1.99.0-aarch64-apple-darwin"}):
        raise ValueError("expected Cargo must be cargo or the exact resolved canonical Rust 1.99.0 executable")
    return value


def _phases(value, expected, label, expected_cargo):
    if not isinstance(value, list) or len(value) != len(expected):
        raise ValueError(f"{label} must contain exactly {len(expected)} phases")
    statuses = []
    for index, (phase_record, (phase_name, expected_status)) in enumerate(zip(value, expected)):
        phase_label = f"{label}[{index}]"
        _require_keys(phase_record, _PHASE_KEYS, phase_label)
        if phase_record["phase"] != phase_name:
            raise ValueError(f"{phase_label} must be {phase_name}")
        argv = phase_record["argv"]
        if not isinstance(argv, list) or not argv or any(not isinstance(item, str) or not item for item in argv):
            raise ValueError(f"{phase_label}.argv must be a nonempty string array")
        expected_argv = [expected_cargo, "test"] + (["--no-run"] if phase_name == "Build" else []) + ["--verbose", "--package=careops-ed@0.1.0"]
        if argv.count("--locked") > 1 or [arg for arg in argv if arg != "--locked"] != expected_argv:
            raise ValueError(f"{phase_label}.argv must be the reviewed package-scoped Cargo {phase_name} command")
        duration = phase_record["duration"]
        if isinstance(duration, bool) or not isinstance(duration, (int, float)) or duration < 0:
            raise ValueError(f"{phase_label}.duration must be finite and nonnegative")
        try:
            finite_duration = math.isfinite(float(duration))
        except (OverflowError, ValueError):
            finite_duration = False
        if not finite_duration:
            raise ValueError(f"{phase_label}.duration must be finite and nonnegative")
        status = _process_status(phase_record["process_status"], f"{phase_label}.process_status")
        if status != expected_status:
            raise ValueError(f"{phase_label} has {status}, expected {expected_status}")
        statuses.append(status)
    return statuses


def _timestamp(value, label):
    if not isinstance(value, str) or not value:
        raise ValueError(f"{label} must be an ISO timestamp")
    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError as error:
        raise ValueError(f"{label} must be an ISO timestamp") from error
    if parsed.tzinfo is None:
        raise ValueError(f"{label} must include a timezone")
    return parsed


def _mutant(row, label):
    if not isinstance(row["scenario"], dict) or set(row["scenario"]) != {"Mutant"}:
        raise ValueError(f"{label}.scenario must contain one Mutant record")
    mutant = row["scenario"]["Mutant"]
    _require_keys(mutant, _MUTANT_KEYS, f"{label}.scenario.Mutant")
    if mutant["package"] != "careops-ed" or mutant["file"] != SOURCE_FILE:
        raise ValueError(f"{label} mutant must target {SOURCE_FILE} in package careops-ed")
    for key in ("name", "genre"):
        if not isinstance(mutant[key], str) or not mutant[key]:
            raise ValueError(f"{label}.scenario.Mutant.{key} must be nonempty text")
    if not isinstance(mutant["replacement"], str):
        raise ValueError(f"{label}.scenario.Mutant.replacement must be text")
    function = mutant["function"]
    _require_keys(function, {"function_name", "return_type", "span"}, f"{label}.scenario.Mutant.function")
    if any(not isinstance(function[field], str) or not function[field] for field in ("function_name", "return_type")):
        raise ValueError(f"{label}.scenario.Mutant.function names must be nonempty text")
    _span(function["span"], f"{label}.scenario.Mutant.function.span")
    start, end = _span(mutant["span"], f"{label}.scenario.Mutant.span")
    return mutant, start, end


def _catalog_mutants(catalog_bytes, expected_sha256):
    if not isinstance(catalog_bytes, bytes):
        raise ValueError("mutants catalog must be supplied as bytes")
    digest = hashlib.sha256(catalog_bytes).hexdigest()
    if digest != expected_sha256:
        raise ValueError(f"mutants catalog SHA-256 mismatch: expected {expected_sha256}, got {digest}")
    try:
        catalog = json.loads(catalog_bytes, object_pairs_hook=_object, parse_constant=_reject_constant)
    except (json.JSONDecodeError, ValueError) as error:
        raise ValueError(f"invalid mutants catalog JSON: {error}") from error
    if not isinstance(catalog, list) or len(catalog) != EXPECTED_TOTAL:
        raise ValueError(f"mutants catalog must contain exactly {EXPECTED_TOTAL} records")
    records = {}
    for index, entry in enumerate(catalog):
        label = f"mutants catalog[{index}]"
        _require_keys(entry, _CATALOG_KEYS, label)
        if entry["package"] != "careops-ed" or entry["file"] != SOURCE_FILE:
            raise ValueError(f"{label} must target {SOURCE_FILE} in package careops-ed")
        name = entry["name"]
        if not isinstance(name, str) or not name:
            raise ValueError(f"{label}.name must be nonempty text")
        if name in records:
            raise ValueError(f"duplicate mutant in catalog: {name}")
        records[name] = {key: entry[key] for key in _MUTANT_KEYS}
    return records, digest


def check_outcomes(document, source_bytes, catalog_bytes, *, expected_catalog_sha256=CATALOG_SHA256, expected_cargo="cargo"):
    """Validate counters, outcomes and exact source-bound dispositions.

    This returns native row counts and accepted equivalent-mutation identities;
    it deliberately computes no combined mutation score.
    """
    expected_cargo = _cargo_identity(expected_cargo)
    if not isinstance(source_bytes, bytes):
        raise ValueError("source must be supplied as bytes")
    source_sha256 = hashlib.sha256(source_bytes).hexdigest()
    if source_sha256 != SOURCE_SHA256:
        raise ValueError(f"source SHA-256 mismatch: expected {SOURCE_SHA256}, got {source_sha256}")
    catalog_records, catalog_sha256 = _catalog_mutants(catalog_bytes, expected_catalog_sha256)

    _require_keys(document, _TOP_LEVEL_KEYS, "outcomes document")
    if document["cargo_mutants_version"] != MUTANTS_VERSION:
        raise ValueError(f"cargo-mutants version must be exactly {MUTANTS_VERSION}")
    started = _timestamp(document["start_time"], "start_time")
    ended = _timestamp(document["end_time"], "end_time")
    if ended < started:
        raise ValueError("end_time precedes start_time")

    counters = {key: _nonnegative_int(document[key], key) for key in
                ("total_mutants", "caught", "missed", "unviable", "timeout", "success")}
    if counters["total_mutants"] != EXPECTED_TOTAL:
        raise ValueError(f"expected exactly {EXPECTED_TOTAL} mutant rows")
    if counters["timeout"] != 0:
        raise ValueError("mutation timeouts are not allowed")
    if counters["success"] != 0:
        raise ValueError("unexpected successful mutant outcomes")

    rows = document["outcomes"]
    if not isinstance(rows, list) or len(rows) != EXPECTED_TOTAL + 1:
        raise ValueError(f"outcomes must contain one Baseline plus exactly {EXPECTED_TOTAL} mutants")

    baseline_count = 0
    counts = {"CaughtMutant": 0, "MissedMutant": 0, "Unviable": 0, "Timeout": 0}
    mutant_names = set()
    diff_paths = set()
    equivalent_misses = []
    unviable_names = set()
    for index, row in enumerate(rows):
        label = f"outcomes[{index}]"
        _require_keys(row, _ROW_KEYS, label)
        scenario = row["scenario"]
        if scenario == "Baseline":
            baseline_count += 1
            if row["summary"] != "Success" or row["diff_path"] is not None:
                raise ValueError("the one Baseline must have Success and no diff_path")
            if row["log_path"] != "log/baseline.log":
                raise ValueError("Baseline log_path must be log/baseline.log")
            _phases(row["phase_results"], [("Build", "Success"), ("Test", "Success")], f"{label}.phase_results", expected_cargo)
            continue

        mutant, start, end = _mutant(row, label)
        name = mutant["name"]
        if name in mutant_names:
            raise ValueError(f"duplicate mutant record: {name}")
        if catalog_records.get(name) != mutant:
            raise ValueError(f"outcome mutant does not match its source-bound catalog record: {name}")
        mutant_names.add(name)
        if not isinstance(row["diff_path"], str) or not row["diff_path"] or row["diff_path"] in diff_paths:
            raise ValueError(f"{label}.diff_path must be unique nonempty text")
        diff_paths.add(row["diff_path"])
        if not isinstance(row["log_path"], str) or not row["log_path"]:
            raise ValueError(f"{label}.log_path must be nonempty text")

        summary = row["summary"]
        if summary == "CaughtMutant":
            counts[summary] += 1
            _phases(row["phase_results"], [("Build", "Success"), ("Test", "Failure")], f"{label}.phase_results", expected_cargo)
        elif summary == "MissedMutant":
            counts[summary] += 1
            _phases(row["phase_results"], [("Build", "Success"), ("Test", "Success")], f"{label}.phase_results", expected_cargo)
            identity = (start[0], start[1])
            if (identity not in EQUIVALENT_MISSES or mutant["genre"] != "BinaryOperator"
                    or mutant["replacement"] != "&&"
                    or name != EQUIVALENT_MISSES[identity]
                    or end != (start[0], start[1] + 2)):
                raise ValueError(f"missed mutant lacks an exact reviewed equivalence disposition: {name}")
            if any((item["line"], item["column"]) == identity for item in equivalent_misses):
                raise ValueError(f"duplicate equivalent-mutation disposition at {identity}")
            equivalent_misses.append({"line": start[0], "column": start[1], "name": name})
        elif summary == "Unviable":
            counts[summary] += 1
            if name not in ORIGINAL_UNVIABLES:
                raise ValueError(f"unviable mutant is not part of the original baseline: {name}")
            unviable_names.add(name)
            _phases(row["phase_results"], [("Build", "Failure")], f"{label}.phase_results", expected_cargo)
        else:
            raise ValueError(f"unexpected or unfinished mutant summary: {summary!r}")

    if baseline_count != 1:
        raise ValueError("outcomes must contain exactly one Baseline")
    if mutant_names != set(catalog_records):
        missing = sorted(set(catalog_records) - mutant_names)
        extra = sorted(mutant_names - set(catalog_records))
        raise ValueError(f"outcome mutant set differs from pinned catalog; missing={missing[:3]!r}, extra={extra[:3]!r}")
    if counts["Timeout"] != 0:
        raise ValueError("timeout outcomes are not allowed")
    if counts["Unviable"] > 3 or len(unviable_names) != counts["Unviable"]:
        raise ValueError("at most three distinct original-baseline unviable mutants are allowed")
    for key in ("caught", "missed", "unviable"):
        if counters[key] != counts[{"caught": "CaughtMutant", "missed": "MissedMutant", "unviable": "Unviable"}[key]]:
            raise ValueError(f"top-level {key} counter does not match outcome rows")

    return {
        "source": SOURCE_FILE,
        "source_sha256": source_sha256,
        "catalog_sha256": catalog_sha256,
        "cargo_mutants_version": MUTANTS_VERSION,
        "baseline": "Success",
        "total_mutants": counters["total_mutants"],
        "caught": counters["caught"],
        "missed": counters["missed"],
        "equivalent_misses": sorted(equivalent_misses, key=lambda item: (item["line"], item["column"])),
        "unviable": counters["unviable"],
        "timeout": counters["timeout"],
    }


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--outcomes", required=True, help="cargo-mutants outcomes.json")
    parser.add_argument("--source", required=True, help="scoped production source file")
    parser.add_argument("--cargo", default="cargo", help="exact runtime-resolved canonical Cargo path; defaults to historical bare command")
    args = parser.parse_args(argv)
    try:
        with open(args.outcomes, encoding="utf-8") as stream:
            document = json.load(stream, object_pairs_hook=_object, parse_constant=_reject_constant)
        catalog_path = Path(args.outcomes).with_name("mutants.json")
        catalog_bytes = catalog_path.read_bytes()
        with open(args.source, "rb") as stream:
            source_bytes = stream.read()
        result = check_outcomes(document, source_bytes, catalog_bytes,
                                expected_catalog_sha256=CATALOG_SHA256, expected_cargo=args.cargo)
    except (OSError, json.JSONDecodeError, ValueError) as error:
        print(f"mutation validation failed: {error}", file=sys.stderr)
        return 1
    print(json.dumps(result, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
