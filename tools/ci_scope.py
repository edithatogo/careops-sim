#!/usr/bin/env python3
"""Conservative CI lane selection from a complete Git tree diff."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
import re
import subprocess
import sys
from pathlib import Path

_SHA = re.compile(r"^[0-9a-fA-F]{40,64}$")
_PLANNING_AREAS = {"evidence", "design", "tracks", "execution"}
_PLANNING_SUFFIXES = (".md", ".json")


@dataclass(frozen=True)
class Change:
    status: str
    old_path: str | None
    new_path: str | None


def _git(args: list[str]) -> bytes:
    result = subprocess.run(["git", *args], stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
    if result.returncode:
        message = result.stderr.decode("utf-8", "replace").strip()
        raise RuntimeError(message or f"git {' '.join(args)} exited {result.returncode}")
    return result.stdout


def _parse_name_status(data: bytes) -> list[Change]:
    if data and not data.endswith(b"\0"):
        raise ValueError("nonempty Git name-status output is missing its terminal NUL")
    fields = data.split(b"\0")
    if fields and fields[-1] == b"":
        fields.pop()
    changes: list[Change] = []
    index = 0
    while index < len(fields):
        try:
            status = fields[index].decode("ascii")
        except UnicodeDecodeError as exc:
            raise ValueError("non-ASCII Git status") from exc
        index += 1
        if not status or status[0] not in "ACDMRTUXB" or (status[0] in "RC" and not status[1:].isdigit()) or (status[0] not in "RC" and status != status[0]):
            raise ValueError(f"unknown or malformed Git status {status!r}")
        count = 2 if status[0] in "RC" else 1
        if len(fields) - index < count:
            raise ValueError("truncated Git name-status record")
        paths = [_decode_path(field) for field in fields[index : index + count]]
        index += count
        if any(not _safe_repo_path(path) for path in paths):
            raise ValueError("unsafe or malformed repository path")
        kind = status[0]
        if kind == "A":
            changes.append(Change(status, None, paths[0]))
        elif kind == "D":
            changes.append(Change(status, paths[0], None))
        elif kind in "RC":
            changes.append(Change(status, paths[0], paths[1]))
        elif kind in "MT":
            changes.append(Change(status, paths[0], paths[0]))
        else:
            raise ValueError(f"Git status {status!r} requires full checks")
    return changes


def _decode_path(raw: bytes) -> str:
    path = raw.decode("utf-8", "strict")
    if "\x00" in path:
        raise ValueError("NUL in repository path")
    return path


def _safe_repo_path(path: str) -> bool:
    return bool(path) and not path.startswith("/") and "\\" not in path and all(part not in ("", ".", "..") for part in path.split("/"))


def _tree_modes(revision: str, paths: set[str]) -> dict[str, str]:
    if not paths:
        return {}
    pathspecs = [":(literal)" + path for path in sorted(paths)]
    data = _git(["ls-tree", "-rz", "--full-tree", revision, "--", *pathspecs])
    modes: dict[str, str] = {}
    for record in data.split(b"\0"):
        if not record:
            continue
        try:
            metadata, raw_path = record.split(b"\t", 1)
            mode = metadata.split(b" ", 1)[0].decode("ascii")
            path = _decode_path(raw_path)
        except (ValueError, UnicodeDecodeError) as exc:
            raise ValueError("malformed git ls-tree output") from exc
        if not _safe_repo_path(path):
            raise ValueError("unsafe path in git ls-tree output")
        modes[path] = mode
    return modes


def _planning_path(path: str, mode: str | None) -> bool:
    parts = path.split("/")
    return (
        len(parts) >= 3
        and parts[0] == "conductor"
        and parts[1] in _PLANNING_AREAS
        and Path(parts[-1]).suffix in _PLANNING_SUFFIXES
        and mode == "100644"
    )


def classify(base: str, head: str) -> dict[str, bool]:
    """Return literal lane booleans; any uncertainty selects every lane."""
    try:
        if not _SHA.fullmatch(base) or not _SHA.fullmatch(head):
            raise ValueError("base and head must be full Git object IDs")
        _git(["cat-file", "-e", f"{base}^{{commit}}"])
        _git(["cat-file", "-e", f"{head}^{{commit}}"])
        raw = _git(["diff", "--name-status", "-z", "--find-renames", "--find-copies", "--find-copies-harder", base, head, "--"])
        changes = _parse_name_status(raw)
        if not changes:
            return {"changed": False, "native": False, "context": False, "policy": False}
        old_paths = {change.old_path for change in changes if change.old_path is not None}
        new_paths = {change.new_path for change in changes if change.new_path is not None}
        old_modes = _tree_modes(base, old_paths)
        new_modes = _tree_modes(head, new_paths)
        planning_only = True
        for change in changes:
            if change.old_path is not None and not _planning_path(change.old_path, old_modes.get(change.old_path)):
                planning_only = False
            if change.new_path is not None and not _planning_path(change.new_path, new_modes.get(change.new_path)):
                planning_only = False
        if planning_only:
            return {"changed": True, "native": False, "context": True, "policy": True}
    except (OSError, RuntimeError, ValueError, UnicodeError) as exc:
        print(f"ci scope: conservative full selection: {exc}", file=sys.stderr)
    return {"changed": True, "native": True, "context": True, "policy": True}


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--base", required=True)
    parser.add_argument("--head", required=True)
    parser.add_argument("--output", required=True, help="GitHub Actions output file")
    args = parser.parse_args(argv)
    result = classify(args.base, args.head)
    output = Path(args.output)
    with output.open("a", encoding="utf-8", newline="\n") as stream:
        for name in ("changed", "native", "context", "policy"):
            stream.write(f"{name}={'true' if result[name] else 'false'}\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
