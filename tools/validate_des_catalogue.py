#!/usr/bin/env python3
"""Fail-closed developer checks for the proposed P1.3 DES catalogue.

This validates catalogue structure and a small set of frozen future-value
shapes. It does not approve a runtime profile, clinical policy, parameter fit,
or empirical transfer.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import re
import sys
from collections import Counter
from pathlib import Path, PurePosixPath
from typing import Any, Mapping

from jsonschema import Draft202012Validator
from jsonschema.exceptions import SchemaError


ROOT = Path(__file__).resolve().parents[1]
DES_DIR = Path("model-inputs/ed/des")
EVIDENCE_DIR = DES_DIR / "evidence"
FAMILIES = (
    "demand_case_mix",
    "des_pathways",
    "durations",
    "resources",
    "patient_behavior",
    "hospital_interfaces",
)
RANGE_FIELDS = (
    "physical_limits",
    "observed_sample_range",
    "generic_scenario_range",
    "uncertainty_interval",
    "calibration_bounds",
)
BOARDING_ID = "ed.durations.boarding"
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")


def _error(parameter_id: str, message: str) -> str:
    return f"{parameter_id}: {message}"


def _read_json(path: Path, label: str) -> tuple[Any | None, str | None]:
    try:
        return json.loads(path.read_text(encoding="utf-8")), None
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        return None, f"cannot read {label} ({path}): {exc}"


def _is_number(value: Any) -> bool:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        return False
    try:
        return math.isfinite(value)
    except (OverflowError, TypeError, ValueError):
        return False


def _nonnegative_count(value: Any) -> bool:
    return isinstance(value, int) and not isinstance(value, bool) and value >= 0


def semantic_errors(record: Any) -> list[str]:
    """Check only the small value/range shapes frozen by the P1.3 check design.

    These developer fixtures are not an approved runtime profile. Unknown
    structured shapes fail closed until a later contract defines them.
    """
    if not isinstance(record, Mapping):
        return ["record must be an object"]
    parameter_id = record.get("parameter_id")
    label = parameter_id if isinstance(parameter_id, str) and parameter_id else "<unknown-record>"
    errors: list[str] = []
    value_status = record.get("value_status")
    if not isinstance(value_status, Mapping):
        return [_error(label, "value_status must be an object")]
    status = value_status.get("status")
    value_present = "value" in record

    if status == "known":
        if not value_present:
            errors.append(_error(label, "known value_status requires value"))
        elif isinstance(record.get("value_kind"), str) and record.get("value_kind") in {"empirical_sample", "parametric_distribution"}:
            errors.append(_error(label, "known empirical/distribution value shape is unsupported; later parameterization contract required"))
        else:
            value = record["value"]
            kind = record.get("type")
            if kind == "time":
                if not _is_number(value) or value < 0:
                    errors.append(_error(label, "known time value must be finite and nonnegative"))
            elif kind == "count_or_capacity" and label == "ed.resources.beds":
                if not isinstance(value, Mapping) or set(value) != {"physical", "staffed", "open"}:
                    errors.append(_error(label, "unsupported known-value shape; later contract required"))
                elif not all(_nonnegative_count(value.get(k)) for k in ("physical", "staffed", "open")):
                    errors.append(_error(label, "bed counts must be nonnegative integers"))
                elif not value["open"] <= value["staffed"] <= value["physical"]:
                    errors.append(_error(label, "bed counts must satisfy open <= staffed <= physical"))
            elif kind == "count_or_capacity":
                if not _nonnegative_count(value):
                    errors.append(_error(label, "known count/capacity must be a nonnegative integer or a contracted structured shape"))
            elif kind == "probability_or_conditional_table":
                if _is_number(value):
                    if not 0 <= value <= 1:
                        errors.append(_error(label, "probability must be within [0,1]"))
                elif isinstance(value, Mapping) and set(value) == {
                    "probabilities", "alternatives", "denominator", "decision_point", "conditioning"
                }:
                    probabilities = value["probabilities"]
                    alternatives = value["alternatives"]
                    if not isinstance(probabilities, list) or not probabilities or not all(
                        _is_number(x) and 0 <= x <= 1 for x in probabilities
                    ):
                        errors.append(_error(label, "probability table entries must be finite values within [0,1]"))
                    if (
                        not isinstance(alternatives, list)
                        or not alternatives
                        or not all(isinstance(x, str) and x.strip() for x in alternatives)
                        or len(set(alternatives)) != len(alternatives)
                    ):
                        errors.append(_error(label, "probability table alternatives must be unique nonempty labels"))
                    if isinstance(probabilities, list) and isinstance(alternatives, list) and len(probabilities) != len(alternatives):
                        errors.append(_error(label, "probability and alternative counts must match"))
                    if not _nonnegative_count(value["denominator"]) or value["denominator"] == 0:
                        errors.append(_error(label, "probability table denominator must be a positive integer"))
                    if not isinstance(value["decision_point"], str) or not value["decision_point"].strip():
                        errors.append(_error(label, "probability table decision_point must be nonempty"))
                    if not isinstance(value["conditioning"], Mapping) or not value["conditioning"]:
                        errors.append(_error(label, "probability table conditioning must be a nonempty object"))
                    if isinstance(probabilities, list) and all(_is_number(x) for x in probabilities):
                        if not math.isclose(sum(probabilities), 1.0, rel_tol=0.0, abs_tol=1e-12):
                            errors.append(_error(label, "probability table must sum to 1 within 1e-12"))
                else:
                    errors.append(_error(label, "unsupported known-value shape; later contract required"))
            else:
                errors.append(_error(label, "unsupported known-value shape; later contract required"))
    elif isinstance(status, str) and status in {"unknown", "deferred", "not_applicable"}:
        if value_present:
            errors.append(_error(label, f"{status} value_status cannot include value"))
    else:
        errors.append(_error(label, "value_status.status is unsupported"))

    unit = record.get("unit")
    for field in RANGE_FIELDS:
        state = record.get(field)
        if not isinstance(state, Mapping):
            errors.append(_error(label, f"{field} must be an object"))
            continue
        range_status = state.get("status")
        if not isinstance(range_status, str) or range_status not in {"known", "unknown", "not_applicable", "deferred"}:
            errors.append(_error(label, f"{field} status is unsupported"))
            continue
        if range_status != "known":
            if any(key in state for key in ("minimum", "maximum", "values")):
                errors.append(_error(label, f"{field} non-known state cannot contain numeric bounds"))
            continue
        minimum, maximum = state.get("minimum"), state.get("maximum")
        if not _is_number(minimum) or not _is_number(maximum):
            errors.append(_error(label, f"{field} known bounds must be finite numbers"))
            continue
        if minimum > maximum:
            errors.append(_error(label, f"{field} minimum exceeds maximum"))
        if state.get("unit") != unit:
            errors.append(_error(label, f"{field} unit must match the record unit"))
        if record.get("type") == "time" and minimum < 0:
            errors.append(_error(label, f"{field} lower bound must be nonnegative"))
        if record.get("type") == "count_or_capacity":
            if not _nonnegative_count(minimum) or not _nonnegative_count(maximum):
                errors.append(_error(label, f"{field} count bounds must be nonnegative integers"))
        if record.get("type") == "probability_or_conditional_table" and not (0 <= minimum <= maximum <= 1):
            errors.append(_error(label, f"{field} probability bounds must be within [0,1]"))
    return errors


def _load_contract(root: Path) -> tuple[Any | None, dict[str, Any] | None, dict[str, Any] | None, list[str]]:
    errors: list[str] = []
    schema, err = _read_json(root / "model-inputs/ed/schema/parameter-record.schema.json", "record schema")
    if err:
        errors.append(err)
    ids_doc, err = _read_json(root / "model-inputs/ed/schema/parameter-ids.json", "parameter registry")
    if err:
        errors.append(err)
    matrix_doc, err = _read_json(root / "model-inputs/ed/schema/parameter-usage-matrix.json", "usage matrix")
    if err:
        errors.append(err)
    if errors:
        return None, None, None, errors
    if not isinstance(ids_doc, Mapping) or not isinstance(ids_doc.get("entries"), list):
        errors.append("parameter registry: entries must be a list")
    if not isinstance(matrix_doc, Mapping) or not isinstance(matrix_doc.get("entries"), list):
        errors.append("usage matrix: entries must be a list")
    if errors:
        return None, None, None, errors
    def index_entries(entries: list[Any], label: str) -> dict[str, Any]:
        indexed: dict[str, Any] = {}
        for i, entry in enumerate(entries):
            if not isinstance(entry, Mapping):
                errors.append(f"{label} entry[{i}] must be an object")
                continue
            parameter_id = entry.get("parameter_id")
            if not isinstance(parameter_id, str) or not parameter_id:
                errors.append(f"{label} entry[{i}] parameter_id must be a nonempty string")
                continue
            if parameter_id in indexed:
                errors.append(_error(parameter_id, f"duplicate {label} entry"))
                continue
            indexed[parameter_id] = entry
        return indexed
    registry = index_entries(ids_doc["entries"], "parameter registry")
    matrix = index_entries(matrix_doc["entries"], "usage matrix")
    if errors:
        return None, None, None, errors
    try:
        Draft202012Validator.check_schema(schema)
        validator = Draft202012Validator(schema)
    except (SchemaError, TypeError) as exc:
        return None, None, None, [f"invalid record schema: {exc}"]
    return validator, registry, matrix, errors


def _resolve_evidence_root(root: Path) -> tuple[Path | None, str | None]:
    repo_root = root.resolve()
    try:
        evidence_root = (repo_root / EVIDENCE_DIR).resolve()
        evidence_root.relative_to(repo_root)
    except (OSError, RuntimeError, ValueError) as exc:
        return None, f"evidence directory escapes repository root or cannot be resolved: {exc}"
    return evidence_root, None


def _evidence_records(root: Path, errors: list[str], evidence_root: Path | None = None) -> list[tuple[Path, Mapping[str, Any]]]:
    if evidence_root is None:
        evidence_root, root_error = _resolve_evidence_root(root)
        if root_error:
            errors.append(root_error)
            return []
    if not evidence_root.is_dir():
        errors.append(f"cannot read evidence directory: {evidence_root}")
        return []
    result: list[tuple[Path, Mapping[str, Any]]] = []
    try:
        paths = sorted(evidence_root.glob("*.json"))
    except (OSError, RuntimeError) as exc:
        errors.append(f"cannot list evidence directory: {exc}")
        return []
    for path in paths:
        try:
            resolved = path.resolve()
        except (OSError, RuntimeError) as exc:
            errors.append(f"evidence file path cannot be resolved {path.name}: {exc}")
            continue
        try:
            resolved.relative_to(evidence_root)
        except ValueError:
            errors.append(f"evidence file escapes evidence root: {path.name}")
            continue
        data, err = _read_json(resolved, "evidence record")
        if err:
            errors.append(err)
            continue
        if not isinstance(data, Mapping):
            errors.append(f"evidence record {path.name}: top level must be an object")
            continue
        result.append((resolved, data))
    return result


def _validate_ref(parameter_id: str, ref: Any, root: Path, evidence_root: Path | None) -> tuple[Path | None, Mapping[str, Any] | None, str | None]:
    if not isinstance(ref, Mapping):
        return None, None, _error(parameter_id, "evidence_refs entries must be objects")
    rel = ref.get("path")
    digest = ref.get("sha256")
    if evidence_root is None:
        return None, None, _error(parameter_id, "evidence directory is unavailable within repository root")
    if not isinstance(rel, str) or not rel or "\\" in rel:
        return None, None, _error(parameter_id, "evidence reference path must be a repository-relative POSIX path")
    pure = PurePosixPath(rel)
    if pure.is_absolute() or ".." in pure.parts or pure.as_posix() != rel or not rel.startswith(EVIDENCE_DIR.as_posix() + "/"):
        return None, None, _error(parameter_id, f"unsupported evidence path {rel!r}")
    if not isinstance(digest, str) or not SHA256_RE.fullmatch(digest):
        return None, None, _error(parameter_id, "evidence reference sha256 must be 64 lowercase hexadecimal characters")
    path = root / Path(*pure.parts)
    try:
        resolved = path.resolve()
    except (OSError, RuntimeError) as exc:
        return None, None, _error(parameter_id, f"evidence path cannot be resolved: {exc}")
    try:
        resolved.relative_to(evidence_root)
    except ValueError:
        return None, None, _error(parameter_id, f"evidence reference escapes evidence root: {rel}")
    if not resolved.is_file():
        return None, None, _error(parameter_id, f"evidence reference is missing or not a file: {rel}")
    try:
        raw = resolved.read_bytes()
    except OSError as exc:
        return None, None, _error(parameter_id, f"evidence record cannot be read: {rel}: {exc}")
    actual = hashlib.sha256(raw).hexdigest()
    if actual != digest:
        return None, None, _error(parameter_id, f"evidence reference hash mismatch: {rel}")
    data, err = _read_json(resolved, "referenced evidence record")
    if err or not isinstance(data, Mapping):
        return None, None, _error(parameter_id, f"referenced evidence record is unreadable: {rel}")
    review = data.get("review")
    if not isinstance(review, Mapping) or review.get("outcome") != "accepted":
        return None, None, _error(parameter_id, f"evidence record is not coordinator-accepted: {rel}")
    if not isinstance(data.get("evidence_type"), str) or not data["evidence_type"].strip():
        return None, None, _error(parameter_id, f"evidence record has no evidence_type: {rel}")
    if not isinstance(data.get("parameter_ids"), list) or parameter_id not in data["parameter_ids"]:
        return None, None, _error(parameter_id, f"evidence record does not explicitly cover this ID: {rel}")
    return resolved, data, None


def catalogue_errors(collections: Any, root: Path = ROOT) -> list[str]:
    """Return contract violations for six family collection objects.

    ``root`` is the repository root used for fixed schemas, registries and
    evidence. ``collections`` maps each family name to its decoded JSON object.
    """
    root = Path(root).resolve()
    errors: list[str] = []
    validator, registry, matrix, load_errors = _load_contract(root)
    errors.extend(load_errors)
    if load_errors:
        return sorted(errors)
    if not isinstance(collections, Mapping):
        return ["collections: expected a mapping of family names to objects"]
    expected_ids: dict[str, set[str]] = {family: set() for family in FAMILIES}
    for parameter_id, entry in registry.items():
        if (
            isinstance(parameter_id, str)
            and isinstance(entry, Mapping)
            and isinstance(entry.get("family"), str)
            and entry.get("family") in expected_ids
        ):
            expected_ids[entry["family"]].add(parameter_id)
    actual_families = set(collections)
    for family in FAMILIES:
        if family not in actual_families:
            errors.append(f"{family}: family collection is missing")
    for family in sorted(actual_families - set(FAMILIES), key=str):
        errors.append(f"{family}: unexpected family collection")

    evidence_errors: list[str] = []
    evidence_root, evidence_root_error = _resolve_evidence_root(root)
    if evidence_root_error:
        evidence_errors.append(evidence_root_error)
    evidence_records = _evidence_records(root, evidence_errors, evidence_root)
    errors.extend(evidence_errors)
    accepted_by_id: dict[str, list[Mapping[str, Any]]] = {}
    for _, evidence in evidence_records:
        review = evidence.get("review")
        parameter_ids = evidence.get("parameter_ids")
        if isinstance(review, Mapping) and review.get("outcome") == "accepted" and isinstance(parameter_ids, list):
            for parameter_id in parameter_ids:
                if isinstance(parameter_id, str):
                    accepted_by_id.setdefault(parameter_id, []).append(evidence)

    for family in FAMILIES:
        if family not in collections:
            continue
        collection = collections[family]
        if not isinstance(collection, Mapping):
            errors.append(f"{family}: collection must be an object")
            continue
        if type(collection.get("schema_version")) is not int or collection.get("schema_version") != 1 or collection.get("status") != "proposed" or collection.get("family") != family:
            errors.append(f"{family}: schema_version/status/family header mismatch")
        records = collection.get("records")
        annotations = collection.get("annotations")
        if not isinstance(records, list):
            errors.append(f"{family}: records must be a list")
            continue
        if not isinstance(annotations, Mapping):
            errors.append(f"{family}: annotations must be an object")
            continue
        counts: Counter[str] = Counter()
        for index, item in enumerate(records):
            if not isinstance(item, Mapping):
                errors.append(_error(family, f"record[{index}] must be an object"))
                continue
            candidate_id = item.get("parameter_id")
            if not isinstance(candidate_id, str) or not candidate_id:
                errors.append(_error(family, f"record[{index}] parameter_id must be a nonempty string"))
                continue
            counts[candidate_id] += 1
        for parameter_id, count in counts.items():
            if count > 1:
                errors.append(_error(str(parameter_id), "duplicate record ID"))
        actual_ids = set(counts)
        for missing in sorted(expected_ids[family] - actual_ids):
            errors.append(_error(missing, "registered family record is missing"))
        for extra in sorted(actual_ids - expected_ids[family], key=str):
            errors.append(_error(str(extra), "record ID is not registered in this family"))
        for missing in sorted(expected_ids[family] - set(annotations)):
            errors.append(_error(missing, "annotation is missing"))
        for extra in sorted(set(annotations) - expected_ids[family], key=str):
            errors.append(_error(str(extra), "annotation ID is not registered in this family"))

        for record in records:
            if not isinstance(record, Mapping):
                errors.append(_error(family, "record must be an object"))
                continue
            parameter_id = record.get("parameter_id")
            if not isinstance(parameter_id, str) or parameter_id not in expected_ids[family]:
                continue
            matrix_entry = matrix.get(parameter_id)
            registry_entry = registry.get(parameter_id)
            if not isinstance(matrix_entry, Mapping) or not isinstance(registry_entry, Mapping):
                errors.append(_error(parameter_id, "registry or usage-matrix entry is malformed/missing"))
                continue
            for schema_error in validator.iter_errors(record):
                location = ".".join(str(x) for x in schema_error.absolute_path) or "record"
                errors.append(_error(parameter_id, f"schema {location}: {schema_error.message}"))
            unit_class = matrix_entry.get("unit_class")
            expected_unit = "s" if unit_class == "time" else matrix_entry.get("unit_contract")
            expected_fields = {
                "semantic_name": registry_entry.get("semantic_name"),
                "owner": registry_entry.get("owner"),
                "consumer": matrix_entry.get("consumer"),
                "entity_task_pathway": matrix_entry.get("config_key"),
                "type": unit_class,
                "unit": expected_unit,
            }
            for field, expected in expected_fields.items():
                if record.get(field) != expected:
                    errors.append(_error(parameter_id, f"{field} does not match registry/matrix contract"))
            mode = record.get("mode")
            modes = [mode] if isinstance(mode, str) else mode
            if (
                not isinstance(modes, list)
                or not modes
                or not all(isinstance(item, str) and item in {"Macro", "Micro"} for item in modes)
            ):
                errors.append(_error(parameter_id, "mode must contain at least one supported Macro/Micro label"))
            deferred = matrix_entry.get("value_role") == "deferred"
            value_status = record.get("value_status")
            if not isinstance(value_status, Mapping) or not isinstance(value_status.get("status"), str) or value_status.get("status") not in {"unknown", "deferred"}:
                errors.append(_error(parameter_id, "current catalogue value_status must be unknown or deferred"))
            elif deferred and value_status.get("status") != "deferred":
                errors.append(_error(parameter_id, "matrix-deferred parameter must remain deferred"))
            elif not deferred and value_status.get("status") != "unknown":
                errors.append(_error(parameter_id, "non-deferred matrix parameter must remain unknown"))
            if "value" in record or "reference_default" in record or "distribution" in record:
                errors.append(_error(parameter_id, "current catalogue cannot contain value, reference_default or distribution"))
            if record.get("evidence_class") == "fitted":
                errors.append(_error(parameter_id, "fitted evidence class is not accepted in the current catalogue"))
            for field in RANGE_FIELDS:
                state = record.get(field)
                if (
                    not isinstance(state, Mapping)
                    or not isinstance(state.get("status"), str)
                    or state.get("status") not in {"unknown", "deferred"}
                ):
                    errors.append(_error(parameter_id, f"{field} must remain unknown or deferred"))

            annotation = annotations.get(parameter_id)
            if not isinstance(annotation, Mapping):
                continue
            if annotation.get("unit_contract") != matrix_entry.get("unit_contract"):
                errors.append(_error(parameter_id, "annotation unit_contract does not match usage matrix"))
            if annotation.get("value_role") != matrix_entry.get("value_role"):
                errors.append(_error(parameter_id, "annotation value_role does not match usage matrix"))
            plan = annotation.get("acquisition_plan")
            if not isinstance(plan, str) or not plan.strip():
                errors.append(_error(parameter_id, "acquisition_plan must be nonempty"))
            if annotation.get("downstream_gates") != matrix_entry.get("open_gate"):
                errors.append(_error(parameter_id, "downstream_gates do not match usage matrix"))
            candidates = annotation.get("candidate_families")
            if not isinstance(candidates, list) or not all(isinstance(x, str) and x.strip() for x in candidates):
                errors.append(_error(parameter_id, "candidate_families must be a list of nonempty research labels"))
            candidate_status = annotation.get("candidate_status")
            if not isinstance(candidate_status, str) or not candidate_status.startswith("unselected"):
                errors.append(_error(parameter_id, "candidate_status must begin with unselected"))
            if parameter_id == BOARDING_ID and isinstance(candidates, list) and candidates:
                errors.append(_error(parameter_id, "boarding has no generative candidate families"))
            if parameter_id == BOARDING_ID and (
                matrix_entry.get("value_role") != "observed_target" or record.get("value_kind") != "output"
            ):
                errors.append(_error(parameter_id, "boarding must be represented as an observed output"))
            refs = annotation.get("evidence_refs")
            if not isinstance(refs, list):
                errors.append(_error(parameter_id, "evidence_refs must be a list"))
                refs = []
            ref_digests: list[str] = []
            for ref in refs:
                path, evidence, ref_error = _validate_ref(parameter_id, ref, root, evidence_root)
                if ref_error:
                    errors.append(ref_error)
                    continue
                if evidence is not None and path is not None:
                    ref_digests.append(hashlib.sha256(path.read_bytes()).hexdigest())
            if accepted_by_id.get(parameter_id) and not refs:
                errors.append(_error(parameter_id, "accepted matching evidence exists but no contextual evidence ref is linked"))
            provenance = record.get("provenance")
            if isinstance(provenance, Mapping):
                expected_hash = ref_digests[0] if ref_digests else None
                if provenance.get("content_hash") != expected_hash:
                    errors.append(_error(parameter_id, "provenance content_hash must match first evidence ref or be null"))
                if not refs and provenance.get("retrieved_at") is not None:
                    errors.append(_error(parameter_id, "retrieved_at must be null when no evidence refs are linked"))
            errors.extend(semantic_errors(record))
    return sorted(errors)


def _load_collections(directory: Path) -> tuple[dict[str, Any], list[str]]:
    collections: dict[str, Any] = {}
    errors: list[str] = []
    for family in FAMILIES:
        path = directory / f"{family}.json"
        data, err = _read_json(path, f"{family} collection")
        if err:
            errors.append(f"{family}: {err}")
        else:
            collections[family] = data
    return collections, errors


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--directory", type=Path, default=ROOT / DES_DIR)
    args = parser.parse_args(argv)
    collections, errors = _load_collections(args.directory)
    if not errors:
        errors = catalogue_errors(collections, root=ROOT)
    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        print(f"DES catalogue INVALID: {len(errors)} error(s)", file=sys.stderr)
        return 1
    print("DES catalogue PASS: six family collections structurally valid; no empirical/runtime acceptance")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
