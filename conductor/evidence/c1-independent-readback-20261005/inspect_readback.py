#!/usr/bin/env python3
"""Independent direct PyArrow readback for retained C1 physical artifacts.

This verifier declares its schemas locally from the frozen physical-v1/v2
contracts. It deliberately does not import a repository encoder, decoder, or
schema builder. It reads accepted mapper/calibration artifacts without writing
to any input path.
"""
from __future__ import annotations

import argparse
import collections
import copy
import hashlib
import json
import re
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, NoReturn

import pyarrow as pa
import pyarrow.ipc as ipc
import pyarrow.parquet as pq

FORMAT = "careops.calibration.physical"
LOGICAL_SCHEMA_SHA256 = "8c46db62f691f243385a4ebdf8a7a3d3670e2655dd0c3f82cba4335ced2a3842"
EXPECTED_PROFILES = {
    "long-valid": {"source_rows": 7, "candidate_units": 7, "events": 6,
                   "quarantine": 0, "exclusions": 1, "outcomes": 2},
    "wide-valid": {"source_rows": 3, "candidate_units": 9, "events": 6,
                   "quarantine": 0, "exclusions": 3, "outcomes": 2},
    "long-quarantine": {"source_rows": 7, "candidate_units": 7, "events": 3,
                        "quarantine": 3, "exclusions": 1, "outcomes": 2},
}
FORMATS = ("ipc_file", "ipc_stream", "parquet")
TABLE_FILE_STEMS = {
    "trace_event.v1": "trace_event.v1",
    "trace_exclusion.v1": "trace_exclusion.v1",
    "outcome_observation.v1": "outcome_observation.v1",
}
FIXTURE_STEMS = {
    "trace_event.v1": "trace",
    "trace_exclusion.v1": "exclusion",
    "outcome_observation.v1": "outcome",
}
ABSENT_ONLY = {
    "event_kind_rank", "quality_flags", "raw_event.source_fields",
    "occurrence_time.tick_resolution", "source_recorded_time.tick_resolution",
    "message_created_time.tick_resolution", "knowledge_availability.available_at.tick_resolution",
    "risk_start.tick_resolution", "last_observed.tick_resolution", "event_time.tick_resolution",
}
TIME_PATTERN = re.compile(
    r"([0-9]{4})-([0-9]{2})-([0-9]{2})T([0-9]{2}):([0-9]{2}):([0-9]{2})"
    r"(?:\.([0-9]+))?(Z|[+-][0-9]{2}:[0-9]{2})\Z"
)
RANK_PATTERN = re.compile(r"(?:0|[1-9][0-9]*)\Z")


class ReadbackError(ValueError):
    """Fail-closed mismatch in an input, declared schema, or payload."""


def fail(message: str) -> NoReturn:
    raise ReadbackError(message)


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def canonical(value: Any) -> str:
    return json.dumps(value, ensure_ascii=False, sort_keys=True,
                      separators=(",", ":"), allow_nan=False)


def _pairs(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    out: dict[str, Any] = {}
    for key, value in pairs:
        if key in out:
            fail(f"duplicate JSON key {key!r}")
        out[key] = value
    return out


def read_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=_pairs,
                          parse_constant=lambda value: fail(f"non-finite JSON value {value}"))
    except (OSError, UnicodeError, json.JSONDecodeError, ReadbackError) as exc:
        raise ReadbackError(f"cannot read strict JSON {path}: {exc}") from exc


def read_ndjson(path: Path) -> list[dict[str, Any]]:
    try:
        raw = path.read_bytes()
        text = raw.decode("utf-8")
    except (OSError, UnicodeError) as exc:
        raise ReadbackError(f"cannot read UTF-8 NDJSON {path}: {exc}") from exc
    if raw and not raw.endswith(b"\n"):
        fail(f"NDJSON final newline missing: {path}")
    rows: list[dict[str, Any]] = []
    for line_number, line in enumerate(text.splitlines(), 1):
        if not line:
            fail(f"blank NDJSON line: {path}:{line_number}")
        try:
            value = json.loads(line, object_pairs_hook=_pairs,
                               parse_constant=lambda item: fail(f"non-finite JSON {item}"))
        except (json.JSONDecodeError, ReadbackError) as exc:
            raise ReadbackError(f"invalid NDJSON {path}:{line_number}: {exc}") from exc
        if not isinstance(value, dict):
            fail(f"NDJSON row must be object: {path}:{line_number}")
        rows.append(value)
    return rows


def metadata(**values: str) -> dict[bytes, bytes]:
    return {key.encode("utf-8"): value.encode("utf-8") for key, value in values.items()}


def schema_json(item: pa.Schema | pa.DataType, prefix: str = "") -> Any:
    fields = list(item) if isinstance(item, (pa.Schema, pa.StructType)) else None
    if fields is not None:
        return [{"name": field.name, "type": str(field.type), "nullable": field.nullable,
                 "metadata": {k.decode(): v.decode() for k, v in sorted((field.metadata or {}).items())},
                 "children": schema_json(field.type, prefix + field.name)} for field in fields]
    if pa.types.is_list(item):
        value_field = item.value_field
        return {"list_value_name": value_field.name,
                "list_value_type": str(value_field.type),
                "list_value_nullable": value_field.nullable,
                "list_value_metadata": {k.decode(): v.decode() for k, v in sorted((value_field.metadata or {}).items())},
                "children": schema_json(item.value_type, prefix + "[]")}
    return []


def observation_counts(rows: list[dict[str, Any]]) -> dict[str, dict[str, int]]:
    counts: dict[str, collections.Counter[str]] = collections.defaultdict(collections.Counter)
    def walk(value: Any, path: str) -> None:
        if value is None:
            counts[path]["null"] += 1
        elif isinstance(value, dict):
            if not value:
                counts[path]["empty"] += 1
            for key, child in value.items():
                walk(child, f"{path}.{key}" if path else key)
        elif isinstance(value, list):
            if not value:
                counts[path]["empty"] += 1
            for child in value:
                walk(child, path + "[]")
        elif value == "":
            counts[path]["empty"] += 1
    for row in rows:
        walk(row, "")
    return {path: dict(sorted(counter.items())) for path, counter in sorted(counts.items())}


def logical_observation_counts(rows: list[dict[str, Any]], declared: pa.Schema) -> dict[str, dict[str, int]]:
    counts: dict[str, collections.Counter[str]] = collections.defaultdict(collections.Counter)
    def walk(value: Any, field: pa.Field, path: str) -> None:
        value_key = "source_fields" if field.name == "source_fields_json" else field.name
        if value_key not in value:
            counts[path]["absent"] += 1
            return
        child = value[value_key]
        if child is None:
            counts[path]["null"] += 1
        elif pa.types.is_struct(field.type):
            for nested in field.type:
                walk(child, nested, path + "." + nested.name)
        elif pa.types.is_list(field.type):
            if not child:
                counts[path]["empty"] += 1
            if pa.types.is_struct(field.type.value_type):
                for entry in child:
                    for nested in field.type.value_type:
                        walk(entry, nested, path + "[]." + nested.name)
            else:
                for entry in child:
                    if entry is None:
                        counts[path + "[]"]["null"] += 1
        elif child == "":
            counts[path]["empty"] += 1
    for row in rows:
        for field in declared:
            if field.name != "presence_fields":
                walk(row, field, (field.metadata or {}).get(b"logical_path", field.name.encode()).decode())
    return {path: dict(sorted(counter.items())) for path, counter in sorted(counts.items())}


def f(name: str, dtype: pa.DataType, nullable: bool = True, path: str | None = None,
      encoding: str | None = None, unit: str | None = None) -> pa.Field:
    attrs = {"logical_path": path or name}
    if encoding:
        attrs["encoding"] = encoding
    if unit:
        attrs["unit"] = unit
    return pa.field(name, dtype, nullable=nullable, metadata=metadata(**attrs))


def lineage(path: str) -> pa.StructType:
    return pa.struct([
        f("status", pa.string(), False, path + ".status"),
        f("mapping_version", pa.string(), False, path + ".mapping_version"),
        f("evidence_ref", pa.string(), True, path + ".evidence_ref"),
        f("derivation", pa.string(), True, path + ".derivation"),
    ])


def time_value(path: str) -> pa.StructType:
    return pa.struct([
        f("raw", pa.string(), False, path + ".raw"),
        f("utc_text", pa.string(), False, path + ".utc"),
        f("utc_i128_le", pa.binary(16), False, path + ".utc", "signed_i128_le", "ns_since_unix_epoch"),
        f("source_offset_or_zone", pa.string(), True, path + ".source_offset_or_zone"),
        f("relative_ticks", pa.binary(16), False, path + ".relative_ticks", "unsigned_u128_le", "1ns"),
        f("tick_resolution", pa.string(), True, path + ".tick_resolution"),
        f("source_precision", pa.string(), False, path + ".source_precision"),
        f("lineage", lineage(path + ".lineage"), False, path + ".lineage"),
    ])


def raw_event_type() -> pa.StructType:
    return pa.struct([
        f("source_family", pa.string(), False, "raw_event.source_family"),
        f("source_event_type", pa.string(), False, "raw_event.source_event_type"),
        f("source_record_id", pa.string(), True, "raw_event.source_record_id"),
        f("source_fields_json", pa.string(), True, "raw_event.source_fields"),
    ])


def schema(record_type: str, columns: list[pa.Field]) -> pa.Schema:
    presence = f("presence_fields", pa.list_(pa.field("element", pa.string(), nullable=False)),
                 False, "@presence")
    return pa.schema(columns + [presence], metadata=metadata(
        format=FORMAT, physical_version="2", logical_schema="calibration-v1",
        logical_schema_sha256=LOGICAL_SCHEMA_SHA256, record_type=record_type,
        byte_order="little", optional_presence="sorted_logical_paths"))


TRACE_SCHEMA = schema("trace_event.v1", [
    f("record_type", pa.string(), False), f("schema_version", pa.string(), False),
    f("dataset_id", pa.string(), False), f("mapping_version", pa.string(), False),
    f("case_key", pa.string(), False), f("source_event_key", pa.string(), False),
    f("occurrence", pa.uint32(), False), f("event_kind", pa.string(), False),
    f("relative_ticks", pa.binary(16), False, encoding="unsigned_u128_le", unit="1ns"),
    f("source_order", pa.uint64(), False), f("event_kind_rank", pa.string(), True),
    f("occurrence_time", time_value("occurrence_time"), False),
    f("source_recorded_time", time_value("source_recorded_time")),
    f("message_created_time", time_value("message_created_time")),
    f("time_lineage", pa.struct([
        f("occurrence", lineage("time_lineage.occurrence"), False),
        f("source_recorded", lineage("time_lineage.source_recorded"), False),
        f("message_created", lineage("time_lineage.message_created"), False),
    ]), False),
    f("resource_key", pa.string()), f("actor_key", pa.string()), f("location_key", pa.string()),
    f("quality_flags", pa.list_(pa.field("element", pa.string(), nullable=False))),
    f("raw_event", raw_event_type(), False), f("disposition", pa.string(), False),
    f("knowledge_availability", pa.struct([
        f("status", pa.string(), False, "knowledge_availability.status"),
        f("available_at", time_value("knowledge_availability.available_at"), True,
          "knowledge_availability.available_at"),
    ]), False),
])

EXCLUSION_SCHEMA = schema("trace_exclusion.v1", [
    f("record_type", pa.string(), False), f("schema_version", pa.string(), False),
    f("dataset_id", pa.string(), False), f("mapping_version", pa.string(), False),
    f("source_event_key", pa.string(), False), f("raw_event", raw_event_type(), False),
    f("raw_time_values", pa.list_(pa.field("element", pa.struct([
        f("key", pa.string(), False, "raw_time_values.@key"),
        f("value", pa.string(), True, "raw_time_values.@value"),
    ]), nullable=False)), False, encoding="sorted_unique_entries_v2"),
    f("exclusion_reason", pa.string(), False), f("lineage", lineage("lineage"), False),
    f("detail", pa.string()),
])

OUTCOME_SCHEMA = schema("outcome_observation.v1", [
    f("record_type", pa.string(), False), f("schema_version", pa.string(), False),
    f("dataset_id", pa.string(), False), f("case_key", pa.string(), False),
    f("endpoint", pa.string(), False), f("risk_start", time_value("risk_start")),
    f("last_observed", time_value("last_observed")), f("event_observed", pa.bool_(), False),
    f("event_time", time_value("event_time")), f("event_cause", pa.string()),
    f("censor_status", pa.string(), False), f("censor_reason", pa.string()),
    f("cluster_ids", pa.list_(pa.field("element", pa.string(), nullable=False)), False),
    f("lineage", lineage("lineage"), False),
])

SCHEMAS = {
    "trace_event.v1": TRACE_SCHEMA,
    "trace_exclusion.v1": EXCLUSION_SCHEMA,
    "outcome_observation.v1": OUTCOME_SCHEMA,
}


def utc_ns(text: str) -> int:
    """RFC3339 to signed Unix nanoseconds using integer Gregorian arithmetic."""
    match = TIME_PATTERN.fullmatch(text) if isinstance(text, str) else None
    if not match:
        fail(f"invalid offset-bearing RFC3339 value: {text!r}")
    year, month, day, hour, minute, second = (int(part) for part in match.groups()[:6])
    fraction, zone = match.group(7) or "", match.group(8)
    if zone == "-00:00":
        fail("unknown UTC offset -00:00 is not a valid instant")
    if second > 59:
        fail("leap-second RFC3339 text is outside this frozen schema profile")
    if len(fraction) > 9 and any(character != "0" for character in fraction[9:]):
        fail("RFC3339 value contains non-zero sub-nanosecond precision")
    nanos = int(fraction[:9].ljust(9, "0") or "0")
    if zone == "Z":
        offset = timezone.utc
    else:
        sign = 1 if zone[0] == "+" else -1
        offset_hour, offset_minute = int(zone[1:3]), int(zone[4:6])
        if offset_hour > 23 or offset_minute > 59:
            fail("RFC3339 offset is invalid")
        offset = timezone(sign * timedelta(hours=offset_hour, minutes=offset_minute))
    try:
        local = datetime(year, month, day, hour, minute, second, tzinfo=offset)
        utc = local.astimezone(timezone.utc)
    except (ValueError, OverflowError) as exc:
        raise ReadbackError(f"RFC3339 calendar value invalid/out of range: {text}") from exc
    epoch = datetime(1970, 1, 1, tzinfo=timezone.utc)
    delta = utc - epoch
    return (delta.days * 86400 + delta.seconds) * 1_000_000_000 + nanos


def check_schema(actual: pa.Schema, expected: pa.Schema, origin: str) -> None:
    if not actual.equals(expected, check_metadata=True):
        fail(f"exact declared schema/metadata mismatch: {origin}\nexpected={expected}\nactual={actual}")


def read_arrow_file(path: Path, fmt: str, expected_schema: pa.Schema) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    if not path.is_file():
        fail(f"missing physical input file: {path}")
    try:
        if fmt == "ipc_file":
            with pa.memory_map(str(path), "r") as source:
                reader = ipc.open_file(source)
                check_schema(reader.schema, expected_schema, str(path))
                batches = [reader.get_batch(i) for i in range(reader.num_record_batches)]
            rows = [row for batch in batches for row in batch.to_pylist()]
            boundaries = [batch.num_rows for batch in batches]
            groups: list[int] = []
        elif fmt == "ipc_stream":
            with pa.memory_map(str(path), "r") as source:
                reader = ipc.open_stream(source)
                check_schema(reader.schema, expected_schema, str(path))
                batches = list(reader)
            rows = [row for batch in batches for row in batch.to_pylist()]
            boundaries = [batch.num_rows for batch in batches]
            groups = []
        elif fmt == "parquet":
            reader = pq.ParquetFile(path)
            check_schema(reader.schema_arrow, expected_schema, str(path))
            groups = [reader.metadata.row_group(i).num_rows for i in range(reader.metadata.num_row_groups)]
            table = reader.read()
            rows = table.to_pylist()
            boundaries = []
        else:
            fail(f"unsupported format {fmt}")
    except (OSError, pa.ArrowException) as exc:
        raise ReadbackError(f"Arrow decode failed {path}: {exc}") from exc
    boundary_rows = boundaries if fmt.startswith("ipc_") else groups
    if boundary_rows and sum(boundary_rows) != len(rows):
        fail(f"physical batch/group totals disagree with decoded rows: {path}")
    return rows, {"rows": len(rows), "ipc_batch_rows": boundaries,
                  "parquet_row_group_rows": groups, "sha256": sha256_file(path)}


def check_values(row: Any, field: pa.Field, path: str) -> None:
    if row is None:
        if not field.nullable:
            fail(f"required field is null: {path}")
        return
    dtype = field.type
    if pa.types.is_struct(dtype):
        if not isinstance(row, dict) or set(row) != {child.name for child in dtype}:
            fail(f"struct shape mismatch at {path}")
        for child in dtype:
            check_values(row[child.name], child, path + "." + child.name)
    elif pa.types.is_list(dtype):
        if not isinstance(row, list):
            fail(f"list type mismatch at {path}")
        item = dtype.value_field
        for index, value in enumerate(row):
            check_values(value, item, f"{path}[{index}]")
    elif pa.types.is_fixed_size_binary(dtype):
        if not isinstance(row, bytes) or len(row) != dtype.byte_width:
            fail(f"fixed-size binary width mismatch at {path}")
    elif pa.types.is_string(dtype):
        if not isinstance(row, str):
            fail(f"UTF-8 scalar mismatch at {path}")
    elif pa.types.is_boolean(dtype):
        if not isinstance(row, bool):
            fail(f"boolean scalar mismatch at {path}")
    elif pa.types.is_integer(dtype):
        if isinstance(row, bool) or not isinstance(row, int):
            fail(f"integer scalar mismatch at {path}")
        if pa.types.is_unsigned_integer(dtype) and row < 0:
            fail(f"unsigned integer is negative at {path}")
        bounds = {
            pa.uint32(): (0, (1 << 32) - 1), pa.uint64(): (0, (1 << 64) - 1),
        }
        if dtype in bounds and not bounds[dtype][0] <= row <= bounds[dtype][1]:
            fail(f"integer outside physical width at {path}")
    else:
        fail(f"unhandled declared Arrow type {dtype} at {path}")


def decode_lineage(raw: Any, path: str, absent: set[str]) -> dict[str, Any] | None:
    if raw is None:
        return None
    result: dict[str, Any] = {}
    for name in ("status", "mapping_version", "evidence_ref", "derivation"):
        child = path + "." + name
        if child in absent:
            if raw[name] is not None:
                fail(f"absence marker contradicts nonnull {child}")
            continue
        result[name] = raw[name]
    return result


def has_descendant(absent: set[str], path: str) -> bool:
    return any(item.startswith(path + ".") for item in absent)


def decode_time(raw: Any, path: str, absent: set[str], expected: Any, origin_utc: str) -> Any:
    if raw is None:
        if has_descendant(absent, path):
            fail(f"null parent clock exposes child presence markers: {path}")
        return None
    if not isinstance(raw, dict):
        fail(f"clock struct has wrong shape: {path}")
    utc_text = raw["utc_text"]
    utc_bytes = raw["utc_i128_le"]
    ticks_bytes = raw["relative_ticks"]
    if not isinstance(utc_bytes, bytes) or len(utc_bytes) != 16:
        fail(f"UTC clock is not signed i128 LE bytes: {path}")
    if not isinstance(ticks_bytes, bytes) or len(ticks_bytes) != 16:
        fail(f"tick clock is not unsigned u128 LE bytes: {path}")
    utc_encoded = int.from_bytes(utc_bytes, "little", signed=True)
    exact_utc = utc_ns(utc_text)
    if utc_encoded != exact_utc:
        fail(f"UTC integer bytes disagree with exact UTC text: {path}")
    ticks = int.from_bytes(ticks_bytes, "little", signed=False)
    if expected is not None:
        if utc_text != expected.get("utc"):
            fail(f"exact UTC text changed at {path}")
        if ticks != int(expected["relative_ticks"]):
            fail(f"tick bytes differ from logical ticks at {path}")
    origin = utc_ns(origin_utc)
    if exact_utc - origin != ticks:
        fail(f"UTC minus declared origin differs from unsigned ticks at {path}")
    out = {
        "raw": raw["raw"], "utc": utc_text, "relative_ticks": str(ticks),
        "source_precision": raw["source_precision"],
        "lineage": decode_lineage(raw["lineage"], path + ".lineage", absent),
    }
    for name in ("source_offset_or_zone", "tick_resolution"):
        logical = path + "." + name
        if logical in absent:
            if raw[name] is not None:
                fail(f"absent clock property is physically populated: {logical}")
            continue
        if name == "tick_resolution" and raw[name] is None:
            fail(f"present tick_resolution cannot be null: {logical}")
        out[name] = raw[name]
    if expected is not None:
        for key, value in out.items():
            if key in expected and value != expected[key]:
                fail(f"clock lineage/precision/property differs at {path}.{key}")
        if path + ".tick_resolution" in absent and "tick_resolution" in expected:
            fail(f"tick_resolution is absent physically but present logically: {path}")
        if path + ".source_offset_or_zone" in absent and "source_offset_or_zone" in expected:
            fail(f"source offset is absent physically but present logically: {path}")
    return out


def decode_struct(raw: Any, dtype: pa.StructType, path: str, absent: set[str],
                  expected: Any, origin_utc: str) -> Any:
    if raw is None:
        if has_descendant(absent, path):
            fail(f"null parent struct exposes child presence markers: {path}")
        return None
    if any(field.name == "utc_text" for field in dtype):
        return decode_time(raw, path, absent, expected, origin_utc)
    if not isinstance(raw, dict) or set(raw) != {field.name for field in dtype}:
        fail(f"nested struct fields differ from declared schema: {path}")
    result: dict[str, Any] = {}
    for field in dtype:
        logical_path = (field.metadata or {}).get(b"logical_path", (path + "." + field.name).encode()).decode()
        output_key = "source_fields" if field.name == "source_fields_json" else field.name
        parent = expected.get(output_key) if isinstance(expected, dict) else None
        if logical_path in absent:
            if not field.nullable or raw[field.name] is not None:
                fail(f"invalid nested optional absence: {logical_path}")
            if logical_path in ABSENT_ONLY and parent is not None:
                fail(f"absent-only field is present in logical record: {logical_path}")
            continue
        if logical_path in ABSENT_ONLY and raw[field.name] is None:
            fail(f"present absent-only field is null: {logical_path}")
        value = decode_value(raw[field.name], field.type, logical_path, absent,
                             parent, origin_utc)
        if logical_path in ABSENT_ONLY and value is None:
            fail(f"present absent-only field contains null: {logical_path}")
        result[output_key] = value
    return result


def decode_value(raw: Any, dtype: pa.DataType, path: str, absent: set[str],
                 expected: Any, origin_utc: str) -> Any:
    if raw is None:
        if has_descendant(absent, path):
            fail(f"null parent exposes child absence markers: {path}")
        return None
    if pa.types.is_struct(dtype):
        return decode_struct(raw, dtype, path, absent, expected, origin_utc)
    if pa.types.is_list(dtype):
        if not isinstance(raw, list):
            fail(f"list encoding mismatch at {path}")
        if path == "raw_time_values":
            entries: list[tuple[str, str | None]] = []
            for index, entry in enumerate(raw):
                if not isinstance(entry, dict) or set(entry) != {"key", "value"}:
                    fail(f"raw_time_values entry shape mismatch at index {index}")
                if not isinstance(entry["key"], str) or (entry["value"] is not None and not isinstance(entry["value"], str)):
                    fail(f"raw_time_values key/value type mismatch at index {index}")
                entries.append((entry["key"], entry["value"]))
            keys = [key for key, _ in entries]
            if keys != sorted(set(keys)):
                fail("raw_time_values keys are not sorted and unique")
            result = {key: value for key, value in entries}
            if expected is not None and result != expected:
                fail("raw_time_values differs from logical source map")
            return result
        return [decode_value(item, dtype.value_type, path + "[]", absent,
                             expected[index] if isinstance(expected, list) and index < len(expected) else None,
                             origin_utc) for index, item in enumerate(raw)]
    if path == "raw_event.source_fields":
        if not isinstance(raw, str):
            fail("raw_event.source_fields must be canonical JSON text")
        try:
            value = json.loads(raw, object_pairs_hook=_pairs,
                               parse_constant=lambda item: fail(f"invalid JSON constant {item}"))
        except (json.JSONDecodeError, ReadbackError) as exc:
            raise ReadbackError(f"invalid raw source fields JSON: {exc}") from exc
        if not isinstance(value, dict) or canonical(value) != raw:
            fail("raw_event.source_fields JSON is not canonical finite UTF-8")
        if expected is not None and value != expected:
            fail("raw_event source_fields differ from normalized record")
        return value
    if path.endswith("relative_ticks"):
        if not isinstance(raw, bytes) or len(raw) != 16:
            fail(f"unsigned ticks field is not 16-byte LE: {path}")
        value = str(int.from_bytes(raw, "little", signed=False))
        if expected is not None and value != str(expected):
            fail(f"unsigned ticks differ from logical record: {path}")
        return value
    if path == "event_kind_rank":
        if not isinstance(raw, str) or not RANK_PATTERN.fullmatch(raw):
            fail("event_kind_rank is not canonical nonnegative decimal UTF-8")
        value = int(raw)
        if expected is not None and value != expected:
            fail("event_kind_rank differs from normalized record")
        return value
    if isinstance(raw, bytes):
        fail(f"unexpected binary physical value at {path}")
    if expected is not None and raw != expected:
        fail(f"physical value differs from normalized record: {path}")
    return raw


def decode_record(raw: dict[str, Any], record_type: str, expected: dict[str, Any],
                  origin_utc: str) -> dict[str, Any]:
    declared = SCHEMAS[record_type]
    check_values(raw, pa.field("row", pa.struct(list(declared)), nullable=False), record_type)
    absence = raw.get("presence_fields")
    if not isinstance(absence, list) or any(not isinstance(value, str) for value in absence) \
            or absence != sorted(set(absence)):
        fail("presence_fields must be sorted unique logical paths")
    known: set[str] = set()
    def collect(field: pa.Field) -> None:
        path = (field.metadata or {}).get(b"logical_path")
        if path is not None and field.nullable:
            known.add(path.decode())
        if pa.types.is_struct(field.type):
            for child in field.type:
                collect(child)
        elif pa.types.is_list(field.type) and pa.types.is_struct(field.type.value_type):
            for child in field.type.value_type:
                collect(child)
    for field in declared:
        if field.name != "presence_fields":
            collect(field)
    known.add("@presence")
    if not set(absence) <= known:
        fail(f"presence_fields contains unknown paths: {sorted(set(absence) - known)}")
    result: dict[str, Any] = {}
    for field in declared:
        if field.name == "presence_fields":
            continue
        logical_path = (field.metadata or {}).get(b"logical_path", field.name.encode()).decode()
        logical_key = "source_fields" if field.name == "source_fields_json" else field.name
        if logical_path in absence:
            if not field.nullable or raw[field.name] is not None:
                fail(f"invalid top-level absence marker for {field.name}")
            if field.name in expected:
                fail(f"logical field {logical_key} is present but marked absent")
            continue
        if logical_path in ABSENT_ONLY and raw[field.name] is None:
            fail(f"present absent-only field is null: {logical_path}")
        if logical_key not in expected:
            fail(f"logical record lacks {logical_key} but physical presence marker does not say absent")
        result[logical_key] = decode_value(raw[field.name], field.type, logical_path,
                                           set(absence), expected[logical_key], origin_utc)
    if result != expected:
        fail(f"decoded {record_type} record differs from exact normalized logical row")
    if record_type == "outcome_observation.v1":
        validate_outcome_semantics(result)
    return result


def validate_outcome_semantics(row: dict[str, Any]) -> None:
    observed = row.get("event_observed")
    event_time = row.get("event_time")
    status = row.get("censor_status")
    if status not in {"not_censored", "right"}:
        fail("outcome has unsupported C0 censor_status")
    if bool(observed) != isinstance(event_time, dict):
        fail("outcome observed flag conflicts with event clock presence")
    if status == "not_censored" and (observed is not True or not isinstance(event_time, dict)):
        fail("not-censored outcome must be observed and carry event clock")
    if status == "right" and (observed is not False or event_time is not None or row.get("last_observed") is None):
        fail("right-censored outcome conflicts with event/last-observed clocks")


def read_physical(path: Path, fmt: str, record_type: str, expected_rows: list[dict[str, Any]],
                  origin_utc: str) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    raw_rows, physical_receipt = read_arrow_file(path, fmt, SCHEMAS[record_type])
    if len(raw_rows) != len(expected_rows):
        fail(f"row count differs from accepted logical manifest at {path}")
    decoded = [decode_record(raw, record_type, expected, origin_utc)
               for raw, expected in zip(raw_rows, expected_rows, strict=True)]
    physical_receipt["ordered_payload_sha256"] = sha256_bytes(
        ("\n".join(canonical(value) for value in decoded) + "\n").encode("utf-8"))
    return decoded, physical_receipt


def c01_layouts(c01_root: Path) -> tuple[dict[str, Any], dict[str, Any], dict[str, Any]]:
    capture = c01_root / "capture"
    physical_root = c01_root / "physical-attempt1"
    qualification_path = physical_root / "qualification.json"
    qualification = read_json(qualification_path)
    if qualification.get("status") != "ready_for_review" or qualification.get("actual_adapter_invocations") != 24 \
            or qualification.get("verified_rust_files") != 648 or qualification.get("bundle_count") != 216:
        fail("C01 physical qualification receipt does not declare 24/648/216 coverage")
    command_receipt_path = c01_root / "physical-command-receipt.json"
    command_receipt = read_json(command_receipt_path)
    if command_receipt.get("exit_code") != 0 or command_receipt.get("head") != "7081f29168c9ac2813c1d45dc909f992e8bb2fe3":
        fail("C01 physical command receipt is not the accepted actual run")
    if not isinstance(qualification.get("layout_receipts"), list) or len(qualification["layout_receipts"]) != 24:
        fail("C01 qualification has no complete set of layout receipts")
    prep_path = c01_root.parent / "input-preparation.json"
    prep = read_json(prep_path)
    if prep.get("verified_archive_members") != 2207 or not re.fullmatch(r"[0-9a-f]{64}", prep.get("archive_sha256", "")) \
            or not re.fullmatch(r"[0-9a-f]{64}", prep.get("inventory_sha256", "")):
        fail("C01 extracted archive is not bound by complete verified input-preparation inventory")
    profiles: dict[str, dict[str, Any]] = {}
    for name, expected in EXPECTED_PROFILES.items():
        directory = capture / name
        manifest_path = directory / "manifest.json"
        manifest = read_json(manifest_path)
        source = read_json(directory / "source.json")
        if not isinstance(source, list) or any(not isinstance(row, dict) for row in source):
            fail(f"C01 {name} source.json must be a raw JSON object array")
        if manifest.get("source_rows") != expected["source_rows"] or len(source) != expected["source_rows"]:
            fail(f"C01 {name} source row count mismatch")
        profile_receipt = qualification.get("capture_profile_receipts", {}).get(name, {})
        baseline_path = directory / "baseline-receipt.json"
        if profile_receipt.get("source_sha256") != sha256_file(directory / "source.json") \
                or profile_receipt.get("receipt_sha256") != sha256_file(baseline_path):
            fail(f"C01 {name} source/baseline receipts differ from accepted capture binding")
        if manifest.get("candidate_units") != expected["candidate_units"]:
            fail(f"C01 {name} candidate count mismatch")
        population_rows = {
            "events": read_ndjson(directory / "events.ndjson"),
            "quarantine": read_ndjson(directory / "quarantine.ndjson"),
            "exclusions": read_ndjson(directory / "exclusions.ndjson"),
            "outcomes": read_ndjson(directory / "outcomes.ndjson"),
        }
        if {key: len(rows) for key, rows in population_rows.items()} != {
            key: expected[key] for key in population_rows
        }:
            fail(f"C01 {name} normalized population row counts differ from frozen fixture")
        for key, rows in population_rows.items():
            evidence = manifest.get("populations", {}).get(key, {})
            path = directory / f"{key}.ndjson"
            if evidence.get("sha256") != sha256_file(path) or evidence.get("bytes") != path.stat().st_size \
                    or evidence.get("rows") != len(rows):
                fail(f"C01 {name} manifest hash/count mismatch for {key}")
        profiles[name] = {
            "directory": directory, "manifest": manifest, "source": source,
            "rows": population_rows,
            "origin_utc": manifest.get("origin_utc"),
        }
        if not isinstance(profiles[name]["origin_utc"], str):
            fail(f"C01 {name} manifest origin_utc missing")
    outputs: list[dict[str, Any]] = []
    seen: set[str] = set()
    layout_receipts = qualification["layout_receipts"]
    for receipt in layout_receipts:
        profile = receipt.get("profile")
        layout = receipt.get("layout")
        if profile not in EXPECTED_PROFILES or not isinstance(layout, str):
            fail("C01 layout receipt has unknown profile/layout identity")
        # Historical absolute paths stay untouched; select the identical
        # published archive member using its stable profile/layout identity.
        report_path = physical_root / "profiles" / profile / layout / "layout-result.json"
        if not report_path.is_file():
            fail(f"C01 archive is missing accepted layout member {report_path}")
        if sha256_file(report_path) != receipt.get("report_sha256"):
            fail(f"C01 layout report archive hash mismatch: {report_path}")
        report = read_json(report_path)
        if report.get("profile") != profile or report.get("layout") not in (None, layout):
            fail(f"C01 layout report identity mismatch: {report_path}")
        output_dir = physical_root / "profiles" / profile / layout / "rust-output"
        if not output_dir.is_dir():
            fail(f"C01 Rust output archive directory is missing: {output_dir}")
        rust_hashes = report.get("rust_outputs")
        if not isinstance(rust_hashes, dict) or len(rust_hashes) != 27:
            fail(f"C01 Rust output receipt lacks the required 27 members: {report_path}")
        for filename, expected_sha in rust_hashes.items():
            path = (output_dir / filename).resolve()
            if output_dir.resolve() not in path.parents or not path.is_file():
                fail(f"C01 output archive member missing/escaped: {path}")
            archive_member = (Path("profiles") / profile / layout / "rust-output" / filename).as_posix()
            if archive_member in seen:
                fail(f"duplicate C01 output file in archive receipts: {filename}")
            seen.add(archive_member)
            if sha256_file(path) != expected_sha.get("file_sha256"):
                fail(f"C01 Rust output hash differs from accepted layout receipt: {path}")
            match = re.fullmatch(r"(trace_event\.v1|trace_exclusion\.v1|outcome_observation\.v1)\.reversed\.limit-([123])\.(ipc_file|ipc_stream|parquet)", filename)
            if not match:
                fail(f"unexpected C01 Rust output member name: {filename}")
            record_type, limit, fmt = match.group(1), int(match.group(2)), match.group(3)
            row_order = layout.rsplit("-", 1)[-1]
            if row_order not in {"forward", "reverse"}:
                fail(f"unknown physical row order in layout identity: {layout}")
            pop_key = {"trace_event.v1": "events", "trace_exclusion.v1": "exclusions",
                       "outcome_observation.v1": "outcomes"}[record_type]
            expected_rows = list(profiles[profile]["rows"][pop_key])
            if record_type == "trace_event.v1":
                expected_rows += profiles[profile]["rows"]["quarantine"]
            # Each Rust output reverses the selected physical input order.
            if row_order == "forward":
                expected_rows.reverse()
            decoded, file_receipt = read_physical(path, fmt, record_type, expected_rows,
                                                   profiles[profile]["origin_utc"])
            file_receipt.update({"profile": profile, "layout": layout,
                                 "archive_member": archive_member, "writer_limit": limit,
                                 "record_type": record_type, "format": fmt,
                                 "accepted_archive_sha256": expected_sha})
            outputs.append(file_receipt)
    if len(outputs) != 648 or len(seen) != 648:
        fail(f"C01 archive did not independently read all 648 unique Rust files ({len(outputs)})")
    return profiles, qualification, {"output_files": outputs, "qualification_sha256": sha256_file(qualification_path),
                                     "input_preparation_sha256": sha256_file(prep_path),
                                     "archive_sha256": prep["archive_sha256"], "inventory_sha256": prep["inventory_sha256"],
                                     "physical_root": str(physical_root.resolve())}


def fixture_records(manifest: dict[str, Any]) -> dict[str, list[dict[str, Any]]]:
    records = manifest.get("records")
    if not isinstance(records, dict) or set(records) != set(SCHEMAS):
        fail("accepted C11 fixture manifest lacks exact logical records for three tables")
    return records


def mapper_totals(snapshot: list[dict[str, Any]], validation: dict[str, Any],
                  fixture: dict[str, Any]) -> tuple[dict[str, Any], dict[str, Any], dict[str, list[dict[str, Any]]]]:
    if len(snapshot) != 51 or fixture.get("request_count") != 51:
        fail("C1.1 mapper snapshot must contain exactly 51 requests")
    request_rows: list[dict[str, Any]] = []
    per_type: dict[str, list[dict[str, Any]]] = {record_type: [] for record_type in SCHEMAS}
    for index, item in enumerate(snapshot):
        request, result = item.get("request"), item.get("result")
        if not isinstance(request, dict) or not isinstance(result, dict):
            fail(f"C1.1 snapshot request/result malformed at index {index}")
        accounting = result.get("accounting")
        if not isinstance(accounting, dict):
            fail(f"C1.1 snapshot accounting absent at index {index}")
        candidates = accounting.get("candidate_units")
        partition = {name: accounting.get(name) for name in
                     ("accepted_units", "excluded_units", "failed_units", "unresolved_units")}
        if any(isinstance(value, bool) or not isinstance(value, int) or value < 0
               for value in [candidates, *partition.values()]):
            fail(f"C1.1 snapshot accounting values invalid at index {index}")
        if sum(partition.values()) != candidates or accounting.get("candidate_conservation") is not True:
            fail(f"C1.1 per-request candidate partition does not conserve at index {index}")
        rows = request.get("rows")
        if not isinstance(rows, list) or accounting.get("source_rows") != len(rows):
            fail(f"C1.1 source-row accounting mismatch at index {index}")
        result_records = result.get("records", [])
        if not isinstance(result_records, list) or any(not isinstance(row, dict) for row in result_records):
            fail(f"C1.1 records array malformed at index {index}")
        type_counts = collections.Counter(row.get("record_type") for row in result_records)
        for record in result_records:
            record_type = record.get("record_type")
            if record_type not in SCHEMAS:
                fail(f"unsupported C1.1 record type at request {index}: {record_type}")
            per_type[record_type].append(record)
        outcomes = result.get("outcomes", [])
        if not isinstance(outcomes, list) or any(not isinstance(row, dict) for row in outcomes) \
                or not all(row.get("record_type") == "outcome_observation.v1" for row in outcomes):
            fail(f"C1.1 outcomes must be an accounted array of objects at request {index}")
        for outcome in outcomes:
            if not isinstance(outcome, dict) or outcome.get("record_type") != "outcome_observation.v1":
                fail(f"C1.1 request {index} has malformed outcome record")
            per_type["outcome_observation.v1"].append(outcome)
        type_counts.update(row.get("record_type") for row in outcomes)
        outcome_observed = sum(row.get("event_observed") is True for row in outcomes)
        outcome_right = sum(row.get("censor_status") == "right" for row in outcomes)
        outcome_other = sum(
            row.get("censor_status") not in {"right", "not_censored"}
            for row in outcomes
        )
        # The retained fixture validation receipt independently binds the same
        # index, candidate partition, raw row count, and outcome count.
        checks = validation.get("per_request")
        if not isinstance(checks, list) or len(checks) != 51:
            fail("C11 C0 validation receipt does not cover 51 requests")
        check = checks[index]
        if check.get("request_index") != index or check.get("source_rows") != len(rows) \
                or check.get("candidate_units") != candidates \
                or check.get("partition") != partition \
                or check.get("outcome_observations") != len(outcomes):
            fail(f"C0 validation receipt differs from snapshot request {index}")
        request_rows.append({
            "request_index": index, "profile_version": request.get("profile_version"),
            "source_rows": len(rows), "candidate_units": candidates,
            **partition, "outcome_observations": len(outcomes),
            "outcome_observed": outcome_observed,
            "outcome_right_censored": outcome_right,
            "outcome_other_censored": outcome_other,
            "records": dict(sorted((str(key), value) for key, value in type_counts.items())),
            "classification": result.get("classification"),
        })
    for record_type, actual in per_type.items():
        expected = fixture_records(fixture)[record_type]
        if collections.Counter(map(canonical, actual)) != collections.Counter(map(canonical, expected)):
            fail(f"C1.1 snapshot records differ from accepted C11 fixture manifest: {record_type}")
    totals = {
        key: sum(row[key] for row in request_rows)
        for key in ("source_rows", "candidate_units", "accepted_units", "excluded_units", "failed_units", "unresolved_units", "outcome_observations")
    }
    totals["outcome_observed"] = sum(row["outcome_observed"] for row in request_rows)
    totals["outcome_right_censored"] = sum(row["outcome_right_censored"] for row in request_rows)
    totals["outcome_other_censored"] = sum(row["outcome_other_censored"] for row in request_rows)
    totals["window_censored_cases"] = validation.get("window_censored_cases", 0)
    totals["requests"] = len(request_rows)
    totals["record_counts"] = {key: len(value) for key, value in per_type.items()}
    if totals["record_counts"] != {"trace_event.v1": 31, "trace_exclusion.v1": 17,
                                   "outcome_observation.v1": 4}:
        fail(f"C1.1 accepted mapper record totals differ from frozen physical collection: {totals['record_counts']}")
    return totals, {"per_request": request_rows}, per_type


def readback_joined(kairos_root: Path, joined_root: Path, snapshot_path: Path,
                    snapshot: list[dict[str, Any]], fixture_manifest: dict[str, Any],
                    fixture_manifest_path: Path, c12_root: Path) -> dict[str, Any]:
    if fixture_manifest.get("fixture_collection_not_dataset") is not True:
        fail("C11 fixture archive is not declared as a heterogeneous collection")
    if fixture_manifest.get("physical_version") != "2" or fixture_manifest.get("pyarrow") != "25.0.1":
        fail("C11 archive is not accepted physical-v2/PyArrow25 fixture evidence")
    if sha256_file(snapshot_path) != fixture_manifest.get("actual_snapshot_sha256"):
        fail("C11 fixture manifest does not bind the supplied actual mapper snapshot")
    validation_path = kairos_root / "crates/kairo-ecs-arrow-io/tests/fixtures/calibration_physical_v2/c0-validation.json"
    if sha256_file(validation_path) != fixture_manifest.get("c0_validation", {}).get("sha256"):
        fail("C11 fixture C0 validation receipt hash differs from accepted manifest")
    validation = read_json(validation_path)
    if validation.get("exit_code") != 0 or validation.get("candidate_expansion_checks") != 51 \
            or validation.get("candidate_partition_checks") != 51 or validation.get("outcome_population_checks") != 51:
        fail("C11 retained C0 validation receipt does not cover all 51 requests")
    totals, membership, expected_by_type = mapper_totals(snapshot, validation, fixture_manifest)
    expected_records = fixture_records(fixture_manifest)

    # C1.2 outputs are independently rehashed before reading. The new inventory
    # is a fresh binding, while result/log hashes provide historical provenance.
    inventory_path = joined_root.parent / "joined-readback-inputs.json"
    inventory = read_json(inventory_path)
    if inventory.get("schema_version") != "c1.joined-readback-inputs.v1" \
            or inventory.get("binding_kind") != "fresh_pre_readback_not_historical_output_hashes":
        fail("joined-readback-inputs.json has unsupported schema_version")
    provenance = inventory.get("provenance")
    if not isinstance(provenance, dict):
        fail("joined readback fresh inventory lacks provenance")
    result_path = c12_root / "result.json"
    if provenance.get("result_path") != str(result_path.resolve()) \
            or provenance.get("result_sha256") != sha256_file(result_path):
        fail("joined input inventory does not bind accepted C1.2 result.json")
    for log_key, filename in (("native_log_sha256", "native.log"), ("pyarrow_log_sha256", "pyarrow.log")):
        log_path = c12_root / filename
        if provenance.get(log_key) != sha256_file(log_path):
            fail(f"joined input inventory {log_key} differs from retained accepted log")
    file_hashes = inventory.get("files")
    expected_names = {f"{stem}.{fmt}" for stem in FIXTURE_STEMS.values() for fmt in FORMATS}
    if not isinstance(file_hashes, dict) or set(file_hashes) != expected_names:
        fail("joined-readback-inputs.json must bind the exact nine Rust output files")
    for name, expected_sha in file_hashes.items():
        path = joined_root / name
        if sha256_file(path) != expected_sha:
            fail(f"fresh C1.2 output SHA binding mismatch: {path}")

    fixture_dir = kairos_root / "crates/kairo-ecs-arrow-io/tests/fixtures/calibration_physical_v2"
    fixture_files = fixture_manifest.get("files")
    if not isinstance(fixture_files, dict) or set(fixture_files) != expected_names:
        fail("accepted C11 fixture manifest must bind the exact nine retained fixture files")
    row_contexts = fixture_manifest.get("row_contexts")
    if not isinstance(row_contexts, dict) or set(row_contexts) != set(SCHEMAS):
        fail("C11 manifest lacks per-type row_contexts membership")
    origin_by_type: dict[str, list[str]] = {}
    for record_type, expected_rows in expected_records.items():
        contexts = row_contexts[record_type]
        if not isinstance(contexts, list) or len(contexts) != len(expected_rows):
            fail(f"C11 row_contexts cardinality mismatch for {record_type}")
        origins: list[str] = []
        assigned: dict[int, list[dict[str, Any]]] = collections.defaultdict(list)
        for logical, index in zip(expected_rows, contexts, strict=True):
            if isinstance(index, bool) or not isinstance(index, int) or not 0 <= index < len(snapshot):
                fail(f"C11 invalid row_contexts index for {record_type}")
            request = snapshot[index].get("request", {})
            result = snapshot[index].get("result", {})
            origin = request.get("origin_utc")
            population = result.get("outcomes", []) if record_type == "outcome_observation.v1" else result.get("records", [])
            if not isinstance(origin, str) or collections.Counter(map(canonical, population))[canonical(logical)] < 1:
                fail(f"C11 row_contexts membership does not bind {record_type} to snapshot request {index}")
            origins.append(origin)
            assigned[index].append(logical)
        for index in range(len(snapshot)):
            result = snapshot[index]["result"]
            actual = result.get("outcomes", []) if record_type == "outcome_observation.v1" else result.get("records", [])
            actual = [row for row in actual if row.get("record_type") == record_type]
            if collections.Counter(map(canonical, actual)) != collections.Counter(map(canonical, assigned.get(index, []))):
                fail(f"C11 row_contexts do not exactly partition {record_type} membership at request {index}")
        origin_by_type[record_type] = origins
    all_files: list[dict[str, Any]] = []
    for record_type, stem in FIXTURE_STEMS.items():
        logical_rows = expected_records[record_type]
        for fmt in FORMATS:
            name = f"{stem}.{fmt}"
            fixture_path = fixture_dir / name
            fixture_sha = sha256_file(fixture_path)
            if fixture_sha != fixture_files[name]:
                fail(f"C11 retained fixture archive hash mismatch: {fixture_path}")
            for label, path, accepted_sha in (
                ("c12_rust_current", joined_root / name, file_hashes[name]),
                ("c11_retained_fixture", fixture_path, fixture_files[name]),
            ):
                origins = origin_by_type[record_type]
                # Every record carries its request-specific origin; identical
                # logical rows necessarily share the same encoded tick value.
                decoded = []
                raw_rows, receipt = read_arrow_file(path, fmt, SCHEMAS[record_type])
                if len(raw_rows) != len(logical_rows):
                    fail(f"joined/fixture {name} row count differs from accepted record manifest")
                for raw, logical, origin in zip(raw_rows, logical_rows, origins, strict=True):
                    decoded.append(decode_record(raw, record_type, logical, origin))
                if [canonical(row) for row in decoded] != [canonical(row) for row in logical_rows]:
                    fail(f"actual ordered payload differs from accepted manifest records: {path}")
                receipt.update({"collection": label, "file": str(path.resolve()),
                               "accepted_or_fresh_sha256": accepted_sha,
                               "fresh_binding": label == "c12_rust_current"})
                all_files.append(receipt)
    return {
        "fixture_manifest_sha256": sha256_file(fixture_manifest_path),
        "snapshot_sha256": sha256_file(snapshot_path),
        "c0_validation_sha256": sha256_file(validation_path),
        "joined_fresh_inventory_sha256": sha256_file(inventory_path),
        "joined_provenance": provenance,
        "mapper_totals": totals,
        "per_request_membership": membership,
        "file_receipts": all_files,
        "fixture_collection_not_dataset": True,
    }


def count_profile(profile_name: str, profile: dict[str, Any]) -> dict[str, Any]:
    manifest = profile["manifest"]
    rows = profile["rows"]
    validation = manifest.get("validation", {})
    status_counts = collections.Counter(row.get("censor_status") for row in rows["outcomes"])
    observed = sum(row.get("event_observed") is True for row in rows["outcomes"])
    window_censored = validation.get("censored_cases")
    counts = {
        "source_rows": manifest.get("source_rows"),
        "candidate_units": manifest.get("candidate_units"),
        "mapper_accepted_units": manifest.get("mapper_accepted_units"),
        "mapper_excluded_units": manifest.get("mapper_excluded_units"),
        "failed_units": manifest.get("failed_units"),
        "unresolved_units": manifest.get("unresolved_units"),
        "valid_events": validation.get("valid_events"),
        "quarantined_events": validation.get("quarantined_events"),
        "events_rows": len(rows["events"]), "quarantine_rows": len(rows["quarantine"]),
        "exclusion_rows": len(rows["exclusions"]), "outcome_rows": len(rows["outcomes"]),
        "observed_outcomes": observed,
        "right_censored_outcomes": status_counts.get("right", 0),
        "other_censored_outcomes": sum(value for key, value in status_counts.items()
                                        if key not in {"right", "not_censored"}),
        "window_censored_cases": window_censored,
        "manifest_window_censored_cases": validation.get("censored_cases"),
        "outcome_censor_reason_counts": dict(sorted(collections.Counter(
            str(row.get("censor_reason")) for row in rows["outcomes"]
        ).items())),
        "censor_status_counts": dict(sorted((str(key), value) for key, value in status_counts.items())),
    }
    if counts["source_rows"] != EXPECTED_PROFILES[profile_name]["source_rows"]:
        fail("C01 profile accounting source count differs from frozen profile")
    if sum(manifest.get(key, 0) for key in ("mapper_accepted_units", "mapper_excluded_units", "failed_units", "unresolved_units")) != counts["candidate_units"]:
        fail("C01 candidate partition does not conserve candidates")
    if counts["valid_events"] + counts["quarantined_events"] != validation.get("input_events"):
        fail("C01 validation event partition does not reconcile")
    if counts["valid_events"] != counts["events_rows"] or counts["quarantined_events"] != counts["quarantine_rows"]:
        fail("C01 event populations do not match validator accounting")
    if counts["outcome_rows"] != manifest.get("outcomes"):
        fail("C01 outcome population does not match manifest accounting")
    if counts["outcome_rows"] != 2 or counts["observed_outcomes"] != 1 \
            or counts["right_censored_outcomes"] != 1 or counts["other_censored_outcomes"] != 0 \
            or status_counts.get("not_censored", 0) != 1:
        fail(f"C01 {profile_name} outcome statuses differ from frozen per-profile contract")
    if counts["window_censored_cases"] != 0 or counts["manifest_window_censored_cases"] != 0:
        fail("C01 has nonzero window-censored case count")
    return counts


def negative_controls(sample: tuple[dict[str, Any], str, dict[str, Any], str]) -> list[dict[str, Any]]:
    raw_row, record_type, expected, origin = sample
    controls: list[dict[str, Any]] = []
    def expect_failure(name: str, fn: Any) -> None:
        try:
            fn()
        except ReadbackError as exc:
            controls.append({"control": name, "rejected": True, "reason": str(exc)})
        else:
            fail(f"negative control did not reject: {name}")
    original_schema = SCHEMAS[record_type]
    utc_field_index = next(i for i, item in enumerate(original_schema) if item.name == "occurrence_time")
    field0 = original_schema.field(utc_field_index)
    clock_children = list(field0.type)
    ticks_index = next(i for i, child in enumerate(clock_children) if child.name == "utc_i128_le")
    tick_field = clock_children[ticks_index]
    bad_meta = dict(tick_field.metadata or {})
    bad_meta[b"unit"] = b"milliseconds"
    clock_children[ticks_index] = tick_field.with_metadata(bad_meta)
    wrong_unit = original_schema.set(utc_field_index, field0.with_type(pa.struct(clock_children)))
    expect_failure("unit", lambda: check_schema(wrong_unit, original_schema, "in-memory negative"))
    wrong_width = original_schema.set(utc_field_index, field0.with_type(pa.struct([
        child.with_type(pa.binary(8)) if child.name == "utc_i128_le" else child
        for child in field0.type
    ])))
    expect_failure("integer_width", lambda: check_schema(wrong_width, original_schema, "in-memory negative"))
    required_bad = copy.deepcopy(raw_row)
    required_bad["dataset_id"] = None
    expect_failure("required_null", lambda: decode_record(required_bad, record_type, expected, origin))
    for name in ("quality_flags", "event_kind_rank"):
        if name not in raw_row:
            fail(f"negative control sample lacks {name}")
        absent_only_bad = copy.deepcopy(raw_row)
        absent_only_bad[name] = None
        expect_failure(f"unmarked_null_{name}", lambda b=absent_only_bad: decode_record(b, record_type, expected, origin))
    utc_bad = copy.deepcopy(raw_row)
    old = utc_bad["occurrence_time"]["utc_i128_le"]
    changed = (int.from_bytes(old, "little", signed=True) + 1).to_bytes(16, "little", signed=True)
    utc_bad["occurrence_time"]["utc_i128_le"] = changed
    expect_failure("utc_text_integer_mismatch", lambda: decode_record(utc_bad, record_type, expected, origin))
    outcome = {
        "record_type": "outcome_observation.v1", "event_observed": False,
        "event_time": {"utc": "2020-01-01T00:00:01Z"}, "censor_status": "right",
    }
    expect_failure("right_censor_clock_conflict", lambda: validate_outcome_semantics(outcome))
    unsupported = {"event_observed": False, "event_time": None, "censor_status": "unknown", "last_observed": {}}
    expect_failure("unknown_censor_status", lambda: validate_outcome_semantics(unsupported))
    return controls


def qualify(args: argparse.Namespace) -> dict[str, Any]:
    kairos_root = args.kairos_root.resolve()
    c01_root = args.c01_root.resolve()
    snapshot_path = args.mapper_snapshot.resolve()
    joined_root = args.joined_root.resolve()
    output = args.output.resolve()
    if output.exists():
        fail(f"output JSON path must be new: {output}")
    if not kairos_root.is_dir() or not c01_root.is_dir() or not joined_root.is_dir() or not snapshot_path.is_file():
        fail("one or more explicit input roots/files are unavailable")
    if pa.__version__ != "25.0.1":
        fail(f"independent readback requires pinned PyArrow 25.0.1, found {pa.__version__}")
    fixture_dir = kairos_root / "crates/kairo-ecs-arrow-io/tests/fixtures/calibration_physical_v2"
    fixture_manifest_path = fixture_dir / "manifest.json"
    fixture_manifest = read_json(fixture_manifest_path)
    snapshot = read_json(snapshot_path)
    if not isinstance(snapshot, list):
        fail("actual mapper snapshot must be a JSON request/result array")
    profiles, qualification, c01_receipt = c01_layouts(c01_root)
    preparation = read_json(c01_root.parent / "input-preparation.json")
    if preparation.get("mapper_snapshot_sha256") != sha256_file(snapshot_path):
        fail("C01 extracted archive preparation does not bind the supplied mapper snapshot")
    joined_receipt = readback_joined(kairos_root, joined_root, snapshot_path,
                                     snapshot, fixture_manifest, fixture_manifest_path,
                                     joined_root.parent)
    profile_counts = {name: count_profile(name, profile) for name, profile in profiles.items()}
    totals = {key: sum(profile[key] for profile in profile_counts.values())
              for key in ("source_rows", "candidate_units", "mapper_accepted_units",
                          "mapper_excluded_units", "failed_units", "unresolved_units",
                          "valid_events", "quarantined_events", "events_rows", "quarantine_rows",
                          "exclusion_rows", "outcome_rows", "observed_outcomes",
                          "right_censored_outcomes", "other_censored_outcomes", "window_censored_cases")}
    if totals["source_rows"] != 17 or totals["candidate_units"] != 23 \
            or totals["mapper_accepted_units"] != 18 or totals["mapper_excluded_units"] != 5 \
            or totals["outcome_rows"] != 6 or totals["right_censored_outcomes"] != 3:
        fail(f"C01 profile aggregate totals differ from frozen contract: {totals}")
    # Build one valid physical sample from an already independently decoded file.
    first_profile = profiles["long-valid"]
    output_dir = Path(c01_receipt["physical_root"]) / "profiles/long-valid/b1-g1-forward/rust-output"
    sample_name = "trace_event.v1.reversed.limit-1.ipc_file"
    sample_path = output_dir / sample_name
    sample_rows, _ = read_arrow_file(sample_path, "ipc_file", TRACE_SCHEMA)
    sample_expected_reverse = list(reversed(first_profile["rows"]["events"] + first_profile["rows"]["quarantine"]))
    sample_expected_row = sample_expected_reverse[0]
    controls = negative_controls((sample_rows[0], "trace_event.v1", sample_expected_row,
                                  first_profile["origin_utc"]))
    # C11 physical population totals are checked separately from the C01 profiles.
    joined_totals = joined_receipt["mapper_totals"]
    if joined_totals["source_rows"] != 56 or joined_totals["candidate_units"] != 59 \
            or joined_totals["accepted_units"] != 31 or joined_totals["excluded_units"] != 17 \
            or joined_totals["failed_units"] != 6 or joined_totals["unresolved_units"] != 5:
        fail("C1.1 heterogeneous mapper request source/candidate totals are inconsistent")
    expected_c11 = {"trace_event.v1": 31, "trace_exclusion.v1": 17,
                    "outcome_observation.v1": 4}
    if joined_totals["record_counts"] != expected_c11:
        fail("C1.1 fixture collection record counts differ from frozen contract")
    if joined_totals["outcome_observed"] != 2 or joined_totals["outcome_right_censored"] != 2 \
            or joined_totals["outcome_other_censored"] != 0:
        fail("C1.1 outcomes differ from frozen observed/right-censored population")
    result = {
        "schema_version": "c1.independent-readback.v1",
        "status": "ready_for_review",
        "scope": "Independent synthetic C1 physical readback; not full C1.4 or release acceptance",
        "toolchain": {"python": sys.version, "pyarrow": pa.__version__},
        "schema_catalog": {record_type: {
            "schema_metadata": {key.decode(): value.decode() for key, value in sorted((declared.metadata or {}).items())},
            "fields": schema_json(declared),
        } for record_type, declared in SCHEMAS.items()},
        "logical_observations_once_per_collection": {
            "c01_by_profile": {name: {record_type: logical_observation_counts(rows, SCHEMAS[record_type])
                for record_type, rows in {
                    "trace_event.v1": profile["rows"]["events"] + profile["rows"]["quarantine"],
                    "trace_exclusion.v1": profile["rows"]["exclusions"],
                    "outcome_observation.v1": profile["rows"]["outcomes"],
                }.items()} for name, profile in profiles.items()},
            "c11_fixture_collection": {record_type: logical_observation_counts(rows, SCHEMAS[record_type])
                for record_type, rows in fixture_records(fixture_manifest).items()},
        },
        "inputs": {
            "kairos_root": str(kairos_root), "c01_root": str(c01_root),
            "mapper_snapshot": str(snapshot_path), "joined_root": str(joined_root),
            "c11_fixture_manifest_sha256": sha256_file(fixture_manifest_path),
            "c01_qualification_sha256": c01_receipt["qualification_sha256"],
            "c01_input_preparation_sha256": c01_receipt["input_preparation_sha256"],
            "c01_archive_sha256": c01_receipt["archive_sha256"],
            "c01_inventory_sha256": c01_receipt["inventory_sha256"],
        },
        "c01": {"profiles": profile_counts, "totals": totals,
                "verified_rust_output_files": len(c01_receipt["output_files"]),
                "qualification_status": qualification["status"],
                "file_receipts": c01_receipt["output_files"]},
        "c11_c12": joined_receipt,
        "negative_controls": controls,
        "limitations": ["Three synthetic C01 profiles and heterogeneous C1.1 mapper fixtures only",
                        "No source/runtime changes or C1.4 acceptance claim"],
    }
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return result


def positive_path(raw: str) -> Path:
    return Path(raw).expanduser().resolve()


def parser() -> argparse.ArgumentParser:
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument("--kairos-root", required=True, help="accepted c441 Kairos source tree")
    cli.add_argument("--c01-root", required=True, help="retained C01 qualification archive root")
    cli.add_argument("--mapper-snapshot", required=True, help="actual 51-request C1.1 mapper snapshot JSON")
    cli.add_argument("--joined-root", required=True, help="C1.2 current Rust joined output directory")
    cli.add_argument("--output", required=True, help="new independent readback JSON receipt path")
    return cli


def main(argv: list[str] | None = None) -> int:
    cli = parser()
    args = cli.parse_args(argv)
    args.kairos_root = positive_path(args.kairos_root)
    args.c01_root = positive_path(args.c01_root)
    args.mapper_snapshot = positive_path(args.mapper_snapshot)
    args.joined_root = positive_path(args.joined_root)
    args.output = positive_path(args.output)
    try:
        result = qualify(args)
    except (OSError, ReadbackError, pa.ArrowException) as exc:
        print(json.dumps({"status": "failed", "error": str(exc)}, ensure_ascii=False), file=sys.stderr)
        return 2
    print(json.dumps(result, sort_keys=True, indent=2, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
