# SPDX-License-Identifier: Apache-2.0
"""Regenerate exact fictional Parquet packages from their readable values."""

import hashlib
import io
import json
from pathlib import Path

import polars as pl

TYPES = {
    "Binary": pl.Binary,
    "Boolean": pl.Boolean,
    "Float64": pl.Float64,
    "Int64": pl.Int64,
    "String": pl.String,
}
TABLES = {"objects", "nodes", "edges", "spaces", "doors", "floors"}


def decode_cell(value, kind):
    if value is None:
        return None
    if kind == "Binary":
        return bytes.fromhex(value)
    if kind == "Float64" and isinstance(value, str):
        if value not in {"nan", "inf", "-inf"}:
            raise ValueError("invalid nonfinite fixture value")
        return float(value)
    return value


def save(path, data):
    if path.is_file() and path.read_bytes() == data:
        return
    with path.open("xb") as output:
        output.write(data)


def generate(source, target):
    count = 0
    for spec_path in sorted(source.rglob("tables.json")):
        package = spec_path.parent
        folder = target / package.relative_to(source)
        manifest_bytes = (package / "manifest.json").read_bytes()
        manifest = json.loads(manifest_bytes)
        if manifest["classification"] != "synthetic":
            raise ValueError("fixture generator requires fictional inputs")
        tables = json.loads(spec_path.read_bytes())
        if set(tables) != {
            name for name in manifest["files"] if name.endswith(".parquet")
        }:
            raise ValueError(f"fixture inventory differs at {package.name}")
        folder.mkdir(parents=True, exist_ok=True)
        for name, table in tables.items():
            if name not in {f"{table_name}.parquet" for table_name in TABLES}:
                raise ValueError(f"unknown fixture table at {package.name}")
            schema = [(column, TYPES[kind]) for column, kind in table["schema"]]
            rows = [
                [
                    decode_cell(value, kind)
                    for value, (_, kind) in zip(row, table["schema"], strict=True)
                ]
                for row in table["rows"]
            ]
            data = io.BytesIO()
            pl.DataFrame(rows, schema=schema, orient="row").write_parquet(data)
            encoded = data.getvalue()
            if hashlib.sha256(encoded).hexdigest() != manifest["files"][name]:
                raise ValueError(
                    f"Parquet fixture digest differs at {package.name}/{name}"
                )
            save(folder / name, encoded)
            count += 1
        if "model.json" in manifest["files"]:
            save(folder / "model.json", (package / "model.json").read_bytes())
        save(folder / "manifest.json", manifest_bytes)
    if count == 0:
        raise ValueError("no synthetic fixtures found")
    print(f"Reconstructed {count} byte-identical synthetic Parquet fixtures")


if __name__ == "__main__":
    tests = Path(__file__).resolve().parent
    generate(tests / "fixtures", tests / "fixture-packages")
