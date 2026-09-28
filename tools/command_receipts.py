"""Validate metadata-only command attempt receipts without third-party packages."""

from __future__ import annotations

import datetime as _datetime
import json
import math
import re
import sys
from pathlib import Path
from typing import Any
from decimal import Decimal


_SCHEMA_PATH = Path(__file__).resolve().parents[1] / "conductor/evidence/d1.5-command-attempt-receipt.schema.json"
_SECRET_PATTERNS = (
    re.compile(r"(?i)\b(?:api[_-]?key|access[_-]?token|password|passwd|secret|authorization)\s*[:=]\s*\S+"),
    re.compile(r"(?i)\bbearer\s+[A-Za-z0-9._~+/=-]{8,}"),
    re.compile(r"-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----"),
    re.compile(r"\bAKIA[0-9A-Z]{16}\b"),
    re.compile(r"\bgh[pousr]_[A-Za-z0-9]{20,}\b"),
    re.compile(r"\b\d{3}-\d{2}-\d{4}\b"),
    re.compile(r"(?i)\bMRN\s*[:#=]\s*[A-Z0-9-]{4,}"),
    re.compile(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b"),
)
_SAFE_PATH = re.compile(r"^(?:\.|(?!/)(?![A-Za-z]:)(?!.*\\)(?!.*//)(?!.*(?:^|/)\.{1,2}(?:/|$))(?!.*\/$).+)$")
_UTC_PATTERN = re.compile(r"^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}(?:\.[0-9]+)?Z$")


def _load_schema() -> dict[str, Any]:
    with _SCHEMA_PATH.open(encoding="utf-8") as stream:
        return json.load(stream)


def _safe_path(value: Any) -> bool:
    return isinstance(value, str) and "\x00" not in value and len(value) <= 512 and bool(_SAFE_PATH.fullmatch(value))


def _type_matches(value: Any, schema_type: str) -> bool:
    if schema_type == "object":
        return isinstance(value, dict)
    if schema_type == "array":
        return isinstance(value, list)
    if schema_type == "string":
        return isinstance(value, str)
    if schema_type == "integer":
        return (isinstance(value, int) and not isinstance(value, bool)) or (
            isinstance(value, float) and math.isfinite(value) and value.is_integer()
        )
    if schema_type == "number":
        return isinstance(value, (int, float)) and not isinstance(value, bool) and (
            not isinstance(value, float) or math.isfinite(value)
        )
    if schema_type == "boolean":
        return isinstance(value, bool)
    if schema_type == "null":
        return value is None
    return False


def _json_equal(left: Any, right: Any) -> bool:
    """Compare JSON values, where booleans are distinct from numeric values."""
    if isinstance(left, bool) or isinstance(right, bool):
        return isinstance(left, bool) and isinstance(right, bool) and left is right
    if isinstance(left, (int, float)) and isinstance(right, (int, float)):
        return left == right
    return type(left) is type(right) and left == right


def _schema_errors(value: Any, schema: dict[str, Any], root: dict[str, Any], path: str) -> list[str]:
    errors: list[str] = []
    if "$ref" in schema:
        ref = schema["$ref"]
        if not ref.startswith("#/$defs/"):
            return [f"{path}: unsupported schema reference"]
        target = root.get("$defs", {}).get(ref.rsplit("/", 1)[-1])
        if not isinstance(target, dict):
            return [f"{path}: schema definition unavailable"]
        errors = _schema_errors(value, target, root, path)
        if ref == "#/$defs/safePath" and isinstance(value, str) and "\x00" in value:
            errors.append(f"{path}: path contains a forbidden character")
        return errors

    expected_type = schema.get("type")
    if expected_type and not _type_matches(value, expected_type):
        return [f"{path}: expected {expected_type}"]

    if "const" in schema and not _json_equal(value, schema["const"]):
        errors.append(f"{path}: value does not match required constant")
    if "enum" in schema and value not in schema["enum"]:
        errors.append(f"{path}: value is outside the allowed set")

    if isinstance(value, str):
        if len(value) < schema.get("minLength", 0):
            errors.append(f"{path}: string is too short")
        if len(value) > schema.get("maxLength", float("inf")):
            errors.append(f"{path}: string is too long")
        if "pattern" in schema:
            try:
                pattern = schema["pattern"]
                matched = (re.fullmatch(pattern, value) if pattern.startswith("^") and pattern.endswith("$")
                           else re.search(pattern, value)) is not None
            except re.error:
                matched = False
            if not matched:
                errors.append(f"{path}: string has an invalid format")
        if schema.get("format") == "date-time":
            try:
                _parse_utc(value)
            except ValueError:
                errors.append(f"{path}: invalid UTC timestamp")

    if isinstance(value, (int, float)) and not isinstance(value, bool) and (
        not isinstance(value, float) or math.isfinite(value)
    ):
        if value < schema.get("minimum", float("-inf")):
            errors.append(f"{path}: number is below minimum")
        if value > schema.get("maximum", float("inf")):
            errors.append(f"{path}: number is above maximum")

    if isinstance(value, dict):
        for key in schema.get("required", []):
            if key not in value:
                errors.append(f"{path}.{key}: required property is missing")
        properties = schema.get("properties", {})
        for key, child in value.items():
            known_key = next((allowed for allowed in properties if isinstance(key, str) and key == allowed), None)
            child_path = f"{path}.{known_key}" if known_key is not None else f"{path}.[property]"
            if known_key is not None:
                errors.extend(_schema_errors(child, properties[known_key], root, child_path))
            elif schema.get("additionalProperties") is False:
                errors.append(f"{child_path}: additional property is forbidden")
            elif isinstance(schema.get("additionalProperties"), dict):
                errors.extend(_schema_errors(child, schema["additionalProperties"], root, child_path))
        if "propertyNames" in schema:
            for key in value:
                errors.extend(_schema_errors(key, schema["propertyNames"], root, f"{path}.[property-name]"))

    if isinstance(value, list):
        if len(value) < schema.get("minItems", 0):
            errors.append(f"{path}: too few items")
        if len(value) > schema.get("maxItems", float("inf")):
            errors.append(f"{path}: too many items")
        if schema.get("uniqueItems"):
            encoded = [json.dumps(item, sort_keys=True, separators=(",", ":")) for item in value]
            if len(encoded) != len(set(encoded)):
                errors.append(f"{path}: duplicate items are forbidden")
        item_schema = schema.get("items")
        if isinstance(item_schema, dict):
            for index, item in enumerate(value):
                errors.extend(_schema_errors(item, item_schema, root, f"{path}[{index}]"))

    if "oneOf" in schema:
        matches = sum(not _schema_errors(value, option, root, path) for option in schema["oneOf"])
        if matches != 1:
            errors.append(f"{path}: value does not match exactly one allowed form")
    for branch in schema.get("allOf", []):
        errors.extend(_schema_errors(value, branch, root, path))
    condition = schema.get("if")
    if condition is not None and not _schema_errors(value, condition, root, path):
        if "then" in schema:
            errors.extend(_schema_errors(value, schema["then"], root, path))
    return errors


def _parse_utc(value: str) -> tuple[_datetime.datetime, Decimal]:
    if not _UTC_PATTERN.fullmatch(value):
        raise ValueError("not canonical UTC")
    fraction_match = re.search(r"\.([0-9]+)Z$", value)
    fraction = Decimal("0." + fraction_match.group(1)) if fraction_match else Decimal(0)
    whole_seconds = re.sub(r"\.[0-9]+Z$", "Z", value)
    parsed = _datetime.datetime.fromisoformat(whole_seconds[:-1] + "+00:00")
    if parsed.utcoffset() != _datetime.timedelta(0):
        raise ValueError("not UTC")
    return parsed, fraction


def _strings(value: Any):
    if isinstance(value, str):
        yield value
    elif isinstance(value, dict):
        for key, item in value.items():
            if isinstance(key, str):
                yield key
            yield from _strings(item)
    elif isinstance(value, list):
        for item in value:
            yield from _strings(item)


def _has_sensitive_pattern(value: Any) -> bool:
    for text in _strings(value):
        if any(pattern.search(text) for pattern in _SECRET_PATTERNS):
            return True
    if isinstance(value, dict):
        commands = value.get("commands")
        if isinstance(commands, list):
            split_flag = re.compile(r"(?i)^--(?:password|passwd|api[-_]?key|authorization|access[-_]?token|secret)$")
            for command in commands:
                if not isinstance(command, dict):
                    continue
                argv = command.get("argv")
                if not isinstance(argv, list):
                    continue
                for index, argument in enumerate(argv[:-1]):
                    if isinstance(argument, str) and split_flag.fullmatch(argument):
                        if isinstance(argv[index + 1], str) and argv[index + 1]:
                            return True
    return False


def _semantic_errors(receipt: Any) -> list[str]:
    if not isinstance(receipt, dict):
        return []
    errors: list[str] = []
    try:
        attempt_start = _parse_utc(receipt.get("started_at_utc", ""))
        attempt_finish = _parse_utc(receipt.get("finished_at_utc", ""))
        if attempt_start > attempt_finish:
            errors.append("started_at_utc: attempt starts after it finishes")
    except (ValueError, TypeError):
        attempt_start = attempt_finish = None

    commands = receipt.get("commands")
    prior_finish = None
    if isinstance(commands, list):
        for index, command in enumerate(commands):
            if not isinstance(command, dict):
                continue
            label = f"commands[{index}]"
            try:
                start = _parse_utc(command.get("started_at_utc", ""))
                finish = _parse_utc(command.get("finished_at_utc", ""))
            except (ValueError, TypeError):
                continue
            if start > finish:
                errors.append(f"{label}: command starts after it finishes")
            if attempt_start is not None and attempt_finish is not None and (start < attempt_start or finish > attempt_finish):
                errors.append(f"{label}: command interval is outside the attempt")
            if prior_finish is not None and start < prior_finish:
                errors.append(f"{label}: command intervals overlap or are out of order")
            prior_finish = finish

    status = receipt.get("status")
    issues = receipt.get("unresolved_issues")
    if status == "ready_for_review":
        if issues != []:
            errors.append("unresolved_issues: ready_for_review requires no unresolved issues")
        if isinstance(commands, list):
            for index, command in enumerate(commands):
                if isinstance(command, dict) and command.get("exit_code") != command.get("expected_exit"):
                    errors.append(f"commands[{index}]: ready_for_review requires matching exit codes")
        output_hashes = receipt.get("outputs_sha256")
        if not isinstance(output_hashes, dict) or not output_hashes:
            errors.append("outputs_sha256: ready_for_review requires a nonempty output hash map")
        changed_paths = receipt.get("changed_paths")
        if isinstance(changed_paths, list) and isinstance(output_hashes, dict):
            for path in changed_paths:
                if isinstance(path, str) and path not in output_hashes:
                    errors.append("outputs_sha256: ready_for_review requires a hash for every changed path")
    elif status == "blocked" and isinstance(issues, list) and not issues:
        errors.append("unresolved_issues: blocked status requires an issue summary")

    if _has_sensitive_pattern(receipt):
        errors.append("receipt: metadata matches a common credential or private-data pattern")
    return errors


def validate_receipt(receipt: Any) -> list[str]:
    """Return concise, sanitized schema and semantic errors for one receipt."""
    try:
        schema = _load_schema()
    except (OSError, json.JSONDecodeError):
        return ["schema: frozen receipt schema could not be loaded"]
    return _schema_errors(receipt, schema, schema, "receipt") + _semantic_errors(receipt)


def _main(argv: list[str]) -> int:
    if len(argv) != 3 or argv[1] != "validate":
        print("usage: python3 tools/command_receipts.py validate PATH", file=sys.stderr)
        return 2
    try:
        with Path(argv[2]).open(encoding="utf-8") as stream:
            receipt = json.load(stream)
    except (OSError, UnicodeError, json.JSONDecodeError):
        print("invalid JSON receipt input", file=sys.stderr)
        return 1
    errors = validate_receipt(receipt)
    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        return 1
    print("receipt valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(_main(sys.argv))
