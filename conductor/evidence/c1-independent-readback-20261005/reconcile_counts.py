#!/usr/bin/env python3
"""Reconcile retained synthetic C-01 profiles and C-1.1 mapper requests.

Uses Python's standard library only. It reads JSON/NDJSON, hashes retained
files, and checks C-1.2 fixture-manifest membership. It imports no production
mapper or physical codec and does not execute Rust.
"""

from __future__ import annotations

import argparse
import collections
import hashlib
import json
import pathlib
import sys
from typing import Any


CONTRACT_SHA256 = "9ca7b274409c39d3d114797ae6c1eb728af4fe6a1b1b31dc744d533b0de930cf"
MAPPER_SNAPSHOT_SHA256 = "f71bc569c11253b9505ed4935cf001d49778b1b847af5e0c3b97c53e74b683cf"
C01_EXPECTED = {
    "long-valid": (7, 7, 6, 1, 6, 0, 2),
    "wide-valid": (3, 9, 6, 3, 6, 0, 2),
    "long-quarantine": (7, 7, 6, 1, 3, 3, 2),
}
C01_TOTALS = {
    "source_rows": 17,
    "candidate_units": 23,
    "mapper_accepted_units": 18,
    "mapper_excluded_units": 5,
    "failed_units": 0,
    "unresolved_units": 0,
    "valid_events": 15,
    "quarantined_events": 3,
    "outcomes": 6,
    "observed_outcomes": 3,
    "not_censored_outcomes": 3,
    "right_censored_outcomes": 3,
    "other_outcomes": 0,
    "validator_window_censored_cases": 0,
}
C11_TOTALS = {
    "source_rows": 56,
    "candidate_units": 59,
    "accepted_units": 31,
    "excluded_units": 17,
    "failed_units": 6,
    "unresolved_units": 5,
    "cohort_denominator": 56,
    "missing_triage": 2,
    "trace_events": 31,
    "trace_exclusions": 17,
    "outcomes": 4,
    "observed_outcomes": 2,
    "not_censored_outcomes": 2,
    "right_censored_outcomes": 2,
    "other_outcomes": 0,
}
MAX_FILE_BYTES = 32 * 1024 * 1024


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def read(path: pathlib.Path) -> tuple[bytes, Any]:
    size = path.stat().st_size
    if size > MAX_FILE_BYTES:
        raise ValueError(f"input exceeds {MAX_FILE_BYTES} bytes: {path}")
    raw = path.read_bytes()
    if len(raw) != size or len(raw) > MAX_FILE_BYTES:
        raise ValueError(f"input changed or grew while being read: {path}")
    return raw, json.loads(raw)


def read_ndjson(path: pathlib.Path) -> tuple[bytes, list[dict[str, Any]]]:
    raw = path.read_bytes()
    if len(raw) > MAX_FILE_BYTES:
        raise ValueError(f"NDJSON exceeds {MAX_FILE_BYTES} bytes: {path}")
    if raw and not raw.endswith(b"\n"):
        raise ValueError(f"NDJSON missing final LF: {path}")
    rows = [json.loads(line) for line in raw.splitlines()]
    if any(not isinstance(row, dict) for row in rows):
        raise ValueError(f"NDJSON contains a non-object row: {path}")
    return raw, rows


def canonical_ndjson(rows: list[dict[str, Any]]) -> bytes:
    return b"".join(
        (json.dumps(row, sort_keys=True, separators=(",", ":"), ensure_ascii=False) + "\n").encode()
        for row in rows
    )


def outcome_counts(rows: list[dict[str, Any]]) -> dict[str, int]:
    statuses = collections.Counter(row.get("censor_status") for row in rows)
    return {
        "outcomes": len(rows),
        "observed_outcomes": sum(row.get("event_observed") is True for row in rows),
        "not_censored_outcomes": statuses.get("not_censored", 0),
        "right_censored_outcomes": statuses.get("right", 0),
        "other_outcomes": sum(n for key, n in statuses.items() if key not in ("not_censored", "right")),
        "null_event_time_outcomes": sum(row.get("event_time") is None for row in rows),
    }


def read_c01(c01_root: pathlib.Path) -> dict[str, Any]:
    rust_root = c01_root / "rust-transport-final"
    physical_root = c01_root / "physical-attempt1"
    physical_index_raw, physical_index = read(physical_root / "transport-index.json")
    physical_qualification_raw, physical_qualification = read(physical_root / "qualification.json")
    matrix_raw, matrix = read(rust_root / "actual-matrix-summary.json")
    _, representative_index = read(rust_root / "representative-index.json")
    if len(physical_index.get("bundles", [])) != 216 or physical_qualification.get("bundle_count") != 216:
        raise ValueError("C-01 physical input bundle count must be 216")
    if matrix.get("matrix_points") != 3888 or matrix.get("actual_executions") != 108 or matrix.get("aliases") != 3780:
        raise ValueError("C-01 Rust matrix totals differ from frozen readback contract")
    if matrix.get("index_sha256") != sha(physical_index_raw):
        raise ValueError("C-01 Rust matrix does not bind the physical transport index")
    if physical_qualification.get("transport_index_sha256") != sha(physical_index_raw):
        raise ValueError("C-01 physical receipt does not bind the transport index")
    representatives = representative_index.get("representatives", [])
    if len(representatives) != 6:
        raise ValueError("C-01 must retain six representative output sets")

    profiles: dict[str, Any] = {}
    total = collections.Counter()
    source_hashes: dict[str, Any] = {}
    engine_commits: set[str] = set()
    bundles_by_path = {row["path"]: row for row in physical_index["bundles"]}
    bundle_root = (physical_root / physical_index["bundle_root"]).resolve()
    for name, frozen in C01_EXPECTED.items():
        capture = c01_root / "capture" / name
        source_raw, source_rows = read(capture / "source.json")
        capture_source_sha256 = sha(source_raw)
        manifest_raw, manifest = read(capture / "manifest.json")
        receipt_raw, receipt = read(capture / "baseline-receipt.json")
        if not isinstance(source_rows, list):
            raise ValueError(f"{name}: source must be a JSON array")
        if name.startswith("long-"):
            candidates = sum("id" in row and "kind" in row for row in source_rows)
        else:
            candidates = sum(
                sum(key in row for key in ("arrival_id", "bed_entered_id", "physical_departure_id"))
                for row in source_rows
            )
        event_rows = read_ndjson(capture / "events.ndjson")[1]
        quarantine_rows = read_ndjson(capture / "quarantine.ndjson")[1]
        exclusion_rows = read_ndjson(capture / "exclusions.ndjson")[1]
        outcome_rows = read_ndjson(capture / "outcomes.ndjson")[1]
        statuses = outcome_counts(outcome_rows)
        mapper_accepted = len(event_rows) + len(quarantine_rows)
        validator_window = manifest["validation"].get("censored_cases", 0)
        expected = dict(
            zip(
                ("source_rows", "candidate_units", "mapper_accepted_units", "mapper_excluded_units", "valid_events", "quarantined_events", "outcomes"),
                frozen,
            )
        )
        actual = {
            "source_rows": len(source_rows),
            "candidate_units": candidates,
            "mapper_accepted_units": mapper_accepted,
            "mapper_excluded_units": len(exclusion_rows),
            "failed_units": manifest.get("failed_units", 0),
            "unresolved_units": manifest.get("unresolved_units", 0),
            "valid_events": len(event_rows),
            "quarantined_events": len(quarantine_rows),
            **statuses,
            "validator_window_censored_cases": validator_window,
            "window_end_outcome_labels": sum(row.get("censor_reason") == "window_end" for row in outcome_rows),
            "right_censored_event_time_is_null": all(
                row.get("event_time") is None for row in outcome_rows if row.get("censor_status") == "right"
            ),
            "right_censored_last_observed_present": all(
                row.get("last_observed") is not None for row in outcome_rows if row.get("censor_status") == "right"
            ),
            "cohort_denominator": manifest["cohort_denominator"],
            "missing_triage": manifest["missing_triage"],
        }
        expected.update(
            failed_units=0,
            unresolved_units=0,
            observed_outcomes=1,
            not_censored_outcomes=1,
            right_censored_outcomes=1,
            other_outcomes=0,
            null_event_time_outcomes=1,
            validator_window_censored_cases=0,
            window_end_outcome_labels=1,
            cohort_denominator=frozen[0],
            missing_triage=0,
        )
        actual["right_censored_event_time_is_null"] = all(
            row.get("event_time") is None for row in outcome_rows if row.get("censor_status") == "right"
        )
        actual["right_censored_last_observed_present"] = all(
            row.get("last_observed") is not None for row in outcome_rows if row.get("censor_status") == "right"
        )
        for key, value in expected.items():
            if actual[key] != value:
                raise ValueError(f"C-01 {name}: {key}={actual[key]} expected {value}")
        if candidates != mapper_accepted + len(exclusion_rows):
            raise ValueError(f"C-01 {name}: candidate conservation failed")
        if manifest["validation"]["valid_events"] != len(event_rows):
            raise ValueError(f"C-01 {name}: valid-event count mismatch")
        if manifest["validation"]["quarantined_events"] != len(quarantine_rows):
            raise ValueError(f"C-01 {name}: quarantined-event count mismatch")
        ndjson = canonical_ndjson(source_rows)
        if sha(ndjson) != receipt["source_sha256"] or len(ndjson) != receipt["source_bytes"]:
            raise ValueError(f"C-01 {name}: source NDJSON fingerprint mismatch")

        rep_rows = [rep for rep in representatives if rep["profile"] == name]
        if len(rep_rows) != 2:
            raise ValueError(f"C-01 {name}: expected two retained input-order representatives")
        rep_hashes = []
        population_hashes = {}
        for rep in rep_rows:
            exec_id = rep["execution_id"]
            actual_dir = rust_root / "representatives" / exec_id
            manifest_path = rust_root / rep["manifest_path"]
            _, actual_manifest = read(manifest_path)
            if actual_manifest["inputs"][0]["sha256"] != rep["source_ndjson_sha256"]:
                raise ValueError(f"C-01 {exec_id}: representative input SHA mismatch")
            if actual_manifest["inputs"][0]["bytes"] != rep["source_ndjson_bytes"]:
                raise ValueError(f"C-01 {exec_id}: representative input length mismatch")
            bundle = bundles_by_path.get(rep["transport_bundle_path"])
            if bundle is None:
                raise ValueError(f"C-01 {exec_id}: source bundle is absent from physical index")
            relative = pathlib.PurePosixPath(bundle["path"])
            if relative.is_absolute() or any(part in (".", "..") for part in relative.parts):
                raise ValueError(f"C-01 {exec_id}: unsafe source bundle path")
            source_path = (bundle_root / pathlib.Path(*relative.parts)).resolve()
            if not source_path.is_relative_to(bundle_root):
                raise ValueError(f"C-01 {exec_id}: source bundle escapes allowlisted root")
            if source_path.stat().st_size > MAX_FILE_BYTES:
                raise ValueError(f"C-01 {exec_id}: physical source exceeds byte limit")
            source_raw = source_path.read_bytes()
            if sha(source_raw) != bundle["sha256"] or sha(source_raw) != rep["source_array_sha256"]:
                raise ValueError(f"C-01 {exec_id}: source array hash mismatch")
            source_ndjson = canonical_ndjson(json.loads(source_raw))
            if sha(source_ndjson) != rep["source_ndjson_sha256"] or len(source_ndjson) != rep["source_ndjson_bytes"]:
                raise ValueError(f"C-01 {exec_id}: source NDJSON hash/length mismatch")
            engine_commits.add(actual_manifest["execution"]["engine_commit"])
            rep_hashes.append(rep["source_ndjson_sha256"])
            for filename in ("events.ndjson", "quarantine.ndjson", "exclusions.ndjson", "outcomes.ndjson", "diagnostics.ndjson"):
                reference = (capture / filename).read_bytes()
                retained = (actual_dir / filename).read_bytes()
                if reference != retained:
                    raise ValueError(f"C-01 {exec_id}/{filename}: retained bytes differ from capture")
                population_hashes[filename] = sha(retained)
        if len(set(rep_hashes)) != 2:
            raise ValueError(f"C-01 {name}: expected two distinct ordered source fingerprints")
        actual["representative_executions"] = [r["execution_id"] for r in rep_rows]
        actual["ordered_input_sha256"] = sorted(set(rep_hashes))
        actual["representative_population_sha256"] = population_hashes
        actual["distinct_profile_totals_are_not_layout_sums"] = True
        profiles[name] = actual
        source_hashes[name] = {
            "source_json_sha256": capture_source_sha256,
            "source_ndjson_sha256": sha(ndjson),
            "manifest_sha256": sha(manifest_raw),
            "baseline_receipt_sha256": sha(receipt_raw),
        }
        for key in C01_TOTALS:
            total[key] += actual[key]
    if dict(total) != C01_TOTALS:
        raise ValueError(f"C-01 distinct-profile total mismatch: {dict(total)}")
    if len(engine_commits) != 1:
        raise ValueError(f"C-01 representative engine commits differ: {sorted(engine_commits)}")
    return {
        "profiles": profiles,
        "distinct_profile_totals": dict(total),
        "source_hashes": source_hashes,
        "source_commits": sorted(engine_commits),
        "physical_index_sha256": sha((physical_root / "transport-index.json").read_bytes()),
        "physical_qualification_sha256": sha(physical_qualification_raw),
        "physical_bundles": 216,
        "physical_adapter_runs": physical_qualification["actual_adapter_invocations"],
        "physical_ordered_files": physical_qualification["verified_rust_files"],
        "rust_matrix_sha256": sha(matrix_raw),
        "rust_matrix_points": matrix["matrix_points"],
        "rust_actual_executions": matrix["actual_executions"],
        "rust_aliases": matrix["aliases"],
    }


def candidate_units(request: dict[str, Any]) -> int:
    rows = request.get("rows", [])
    bindings = request.get("event_bindings", [])
    if request.get("shape") == "wide":
        return len(rows) * len(bindings)
    if request.get("shape") != "long":
        raise ValueError(f"unsupported mapper request shape: {request.get('shape')}")
    kind_field = request.get("event_kind_field")
    if not kind_field:
        return len(rows) * len(bindings)
    return sum(
        sum(row.get(kind_field) == binding.get("kind") for binding in bindings)
        for row in rows
    )


def mapper_audit(snapshot_path: pathlib.Path, manifest_path: pathlib.Path) -> dict[str, Any]:
    snapshot_raw, requests = read(snapshot_path)
    if sha(snapshot_raw) != MAPPER_SNAPSHOT_SHA256 or len(requests) != 51:
        raise ValueError("C-1.1 snapshot SHA/request-count mismatch")
    _, fixture_manifest = read(manifest_path)
    if fixture_manifest.get("actual_snapshot_sha256") != sha(snapshot_raw):
        raise ValueError("C-1.2 manifest does not bind this C-1.1 snapshot")

    totals = collections.Counter()
    shapes = collections.Counter()
    classifications = collections.Counter()
    record_types = collections.Counter()
    diagnostics = collections.Counter()
    all_outcomes = []
    source_records: dict[str, list[dict[str, Any]]] = collections.defaultdict(list)
    per_request = []
    for index, item in enumerate(requests):
        request, result = item["request"], item["result"]
        account = result["accounting"]
        shapes[request["shape"]] += 1
        classifications[result["classification"]] += 1
        derived = candidate_units(request)
        if derived != account["candidate_units"] or account["source_rows"] != len(request.get("rows", [])):
            raise ValueError(f"C-1.1 request {index}: source/candidate derivation mismatch")
        partition = sum(account.get(k, 0) for k in ("accepted_units", "excluded_units", "failed_units", "unresolved_units"))
        if partition != derived:
            raise ValueError(f"C-1.1 request {index}: candidate partition does not conserve")
        records = collections.Counter(record.get("record_type") for record in result.get("records", []))
        if records.get("trace_event.v1", 0) != account["accepted_units"]:
            raise ValueError(f"C-1.1 request {index}: accepted/event count mismatch")
        if records.get("trace_exclusion.v1", 0) != account["excluded_units"]:
            raise ValueError(f"C-1.1 request {index}: excluded/exclusion count mismatch")
        outcomes = result.get("outcomes", [])
        all_outcomes.extend(outcomes)
        for record in result.get("records", []):
            source_records[record["record_type"]].append(record)
        source_records["outcome_observation.v1"].extend(outcomes)
        record_types.update(records)
        diagnostics.update(d.get("reason") for d in result.get("diagnostics", []))
        for key in ("source_rows", "candidate_units", "accepted_units", "excluded_units", "failed_units", "unresolved_units", "cohort_denominator", "missing_triage"):
            totals[key] += account.get(key, 0)
        per_request.append({
            "index": index,
            "request_sha256": sha(json.dumps(request, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()),
            "shape": request["shape"],
            "classification": result["classification"],
            "source_rows": account["source_rows"],
            "candidate_units": derived,
            "accepted_units": account["accepted_units"],
            "excluded_units": account["excluded_units"],
            "failed_units": account["failed_units"],
            "unresolved_units": account["unresolved_units"],
            "cohort_denominator": account["cohort_denominator"],
            "missing_triage": account["missing_triage"],
            "outcomes": outcome_counts(outcomes),
        })
    outcome_summary = outcome_counts(all_outcomes)
    totals.update({
        "trace_events": record_types.get("trace_event.v1", 0),
        "trace_exclusions": record_types.get("trace_exclusion.v1", 0),
        "outcomes": outcome_summary["outcomes"],
        "observed_outcomes": outcome_summary["observed_outcomes"],
        "not_censored_outcomes": outcome_summary["not_censored_outcomes"],
        "right_censored_outcomes": outcome_summary["right_censored_outcomes"],
        "other_outcomes": outcome_summary["other_outcomes"],
    })
    for key, expected in C11_TOTALS.items():
        if totals[key] != expected:
            raise ValueError(f"C-1.1 {key}={totals[key]} expected {expected}")

    c12_counts = {key: len(rows) for key, rows in fixture_manifest["records"].items()}
    expected_records = {
        "trace_event.v1": totals["trace_events"],
        "trace_exclusion.v1": totals["trace_exclusions"],
        "outcome_observation.v1": totals["outcomes"],
    }
    if c12_counts != expected_records:
        raise ValueError(f"C-1.2 fixture records {c12_counts} differ from C-1.1 logical counts")
    request_membership = {}
    for record_type, expected_rows in fixture_manifest["records"].items():
        indices = fixture_manifest["row_contexts"][record_type]
        if len(indices) != len(expected_rows) or any(not isinstance(i, int) or i < 0 or i >= len(requests) for i in indices):
            raise ValueError(f"C-1.2 invalid request membership for {record_type}")
        members = collections.Counter(
            json.dumps(row, sort_keys=True, separators=(",", ":"), ensure_ascii=False)
            for i in indices
            for row in source_records[record_type]
            if row in requests[i]["result"].get("records", [])
            or (record_type == "outcome_observation.v1" and row in requests[i]["result"].get("outcomes", []))
        )
        expected = collections.Counter(
            json.dumps(row, sort_keys=True, separators=(",", ":"), ensure_ascii=False)
            for row in expected_rows
        )
        if members != expected:
            raise ValueError(f"C-1.2 row_contexts do not match C-1.1 {record_type} request membership")
        request_membership[record_type] = {"requests": indices, "records": len(expected_rows)}
    file_receipts = {}
    for filename, expected_sha in fixture_manifest["files"].items():
        raw = (manifest_path.parent / filename).read_bytes()
        actual_sha = sha(raw)
        if actual_sha != expected_sha:
            raise ValueError(f"C-1.2 retained fixture file SHA mismatch: {filename}")
        file_receipts[filename] = {"bytes": len(raw), "sha256": actual_sha}
    return {
        "snapshot_sha256": sha(snapshot_raw),
        "request_count": len(requests),
        "request_shapes": dict(shapes),
        "classifications": dict(classifications),
        "request_totals": dict(totals),
        "outcome_statuses": outcome_summary,
        "record_types": dict(record_types),
        "diagnostic_reasons": dict(diagnostics),
        "per_request": per_request,
        "c12_fixture_collection": {
            "manifest_sha256": sha(manifest_path.read_bytes()),
            "actual_snapshot_sha256_matches": True,
            "fixture_collection_not_dataset": fixture_manifest["fixture_collection_not_dataset"],
            "synthetic_only": fixture_manifest["synthetic_only"],
            "logical_record_counts": c12_counts,
            "row_context_membership": request_membership,
            "retained_fixture_file_count": len(file_receipts),
            "retained_fixture_files": file_receipts,
        },
    }


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--c01-root", required=True, type=pathlib.Path)
    parser.add_argument("--mapper-snapshot", required=True, type=pathlib.Path)
    parser.add_argument("--output", required=True, type=pathlib.Path)
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    root = pathlib.Path(__file__).resolve().parents[3]
    contract_sha = sha((root / "conductor/design/calibration/c1-independent-readback-v1.md").read_bytes())
    if contract_sha != CONTRACT_SHA256:
        raise ValueError("frozen count-readback contract SHA changed")
    c01_root = args.c01_root.resolve(strict=True)
    repo = c01_root.parents[1]
    manifest_path = repo / "crates/kairo-ecs-arrow-io/tests/fixtures/calibration_physical_v2/manifest.json"
    report = {
        "schema_version": "c1.independent-count-readback.v1",
        "contract_sha256": contract_sha,
        "c01": read_c01(c01_root),
        "c11": mapper_audit(args.mapper_snapshot.resolve(strict=True), manifest_path),
        "scope_note": (
            "C-01 profile fixtures, C-1.1 requests, and C-1.2 fixture tables are separate collections, not a single dataset. "
            "Format/layout copies are replicas; outcome rows are separate from trace/exclusion/quarantine rows."
        ),
    }
    output = args.output.resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    if output.exists():
        raise FileExistsError(f"refusing to overwrite output: {output}")
    output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps({"status": "counts_reconciled", "output": str(output), "sha256": sha(output.read_bytes())}, sort_keys=True))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, KeyError, TypeError, json.JSONDecodeError) as exc:
        print(f"count reconciliation failed: {exc}", file=sys.stderr)
        raise SystemExit(2)
