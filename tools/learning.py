#!/usr/bin/env python3
"""Evidence-bound, append-only local lessons. Procedures are suggestions only."""
from __future__ import annotations

import argparse
from contextlib import contextmanager
import fcntl
import hashlib
import json
import os
from pathlib import Path
import sys
import time
from typing import Iterator

ROOT = Path(__file__).resolve().parents[1]
STORE = ROOT / "conductor/harness/lessons.jsonl"
MAX_STORE = 1_000_000
MAX_LINE = 32_000
MAX_EVIDENCE = 10_000_000
ZERO = "0" * 64
EVENT_KINDS = {"propose", "review", "validate", "outcome", "use"}
OUTCOMES = {"helped", "partly_helped", "did_not_help", "not_applicable"}


class LearningError(ValueError):
    pass


@contextmanager
def _locked_store() -> Iterator[int]:
    """Lock the store inode for the complete read/transition/append operation."""
    root = ROOT.resolve()
    store = Path(STORE)
    if store.is_symlink():
        raise LearningError("lesson store must not be a symlink")
    resolved = store.resolve(strict=False)
    if root not in resolved.parents:
        raise LearningError("lesson store path is outside repository")
    store.parent.mkdir(parents=True, exist_ok=True)
    flags = os.O_RDWR | os.O_CREAT | os.O_APPEND | getattr(os, "O_NOFOLLOW", 0)
    try:
        fd = os.open(store, flags, 0o600)
    except OSError as exc:
        raise LearningError(f"cannot safely open lesson store: {exc}") from exc
    try:
        fcntl.flock(fd, fcntl.LOCK_EX)
        # Recheck after opening to catch a path swap before acquiring the lock.
        current = Path(STORE)
        if current.is_symlink() or root not in current.resolve().parents:
            raise LearningError("lesson store path changed or escaped repository")
        yield fd
    finally:
        try:
            fcntl.flock(fd, fcntl.LOCK_UN)
        finally:
            os.close(fd)


def _hash_file(path: Path, limit: int) -> str:
    try:
        size = path.stat().st_size
    except OSError as exc:
        raise LearningError(f"cannot stat evidence file {path}: {exc}") from exc
    if size > limit:
        raise LearningError(f"evidence file exceeds {limit} byte limit: {path}")
    hasher = hashlib.sha256()
    total = 0
    try:
        with path.open("rb") as stream:
            while True:
                block = stream.read(64 * 1024)
                if not block:
                    break
                total += len(block)
                if total > limit:
                    raise LearningError(f"evidence file grew beyond {limit} byte limit: {path}")
                hasher.update(block)
    except OSError as exc:
        raise LearningError(f"cannot read evidence file {path}: {exc}") from exc
    return hasher.hexdigest()


def digest(path: Path) -> str:
    """Return a bounded evidence-file hash for seed or proposal construction."""
    return _hash_file(Path(path), MAX_EVIDENCE)


def _canonical(value: dict) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()


def _events(fd: int) -> list[dict]:
    size = os.fstat(fd).st_size
    if size > MAX_STORE:
        raise LearningError("lesson store exceeds 1 MB")
    raw = os.pread(fd, size + 1, 0)
    if len(raw) != size:
        raise LearningError("lesson store changed while locked")
    events, previous = [], ZERO
    for number, line in enumerate(raw.splitlines(), 1):
        if len(line) > MAX_LINE:
            raise LearningError(f"event {number} exceeds line limit")
        try:
            event = json.loads(line)
        except (UnicodeDecodeError, json.JSONDecodeError) as exc:
            raise LearningError(f"invalid event {number}: {exc}") from exc
        if not isinstance(event, dict):
            raise LearningError(f"invalid event {number}: expected an object")
        claimed = event.pop("event_hash", None)
        if not isinstance(claimed, str) or not isinstance(event.get("prev_hash"), str):
            raise LearningError(f"invalid event {number}: missing hash fields")
        if event.get("prev_hash") != previous or claimed != hashlib.sha256(_canonical(event)).hexdigest():
            raise LearningError(f"event chain invalid at line {number}")
        kind = event.get("kind")
        lesson_id = event.get("lesson_id")
        if not isinstance(kind, str) or kind not in EVENT_KINDS or not isinstance(lesson_id, str) or not lesson_id.strip():
            raise LearningError(f"invalid event {number}: malformed kind or lesson id")
        if kind == "propose":
            try:
                _validate_lesson(event.get("lesson"))
            except LearningError as exc:
                raise LearningError(f"invalid event {number}: {exc}") from exc
            if event["lesson"]["id"] != lesson_id:
                raise LearningError(f"invalid event {number}: lesson id mismatch")
        elif kind == "review":
            review_value = event.get("review")
            if (not isinstance(review_value, dict) or
                    not isinstance(review_value.get("reviewer"), str) or
                    not isinstance(review_value.get("note"), str)):
                raise LearningError(f"invalid event {number}: malformed review")
        elif kind == "validate":
            if not isinstance(event.get("at"), int):
                raise LearningError(f"invalid event {number}: malformed validation")
        elif kind == "outcome":
            outcome_value = event.get("outcome")
            if (not isinstance(outcome_value, dict) or not isinstance(outcome_value.get("result"), str) or
                    outcome_value["result"] not in OUTCOMES or
                    not isinstance(outcome_value.get("note"), str)):
                raise LearningError(f"invalid event {number}: malformed outcome")
        elif kind == "use":
            use_value = event.get("use")
            if not isinstance(use_value, dict) or not isinstance(use_value.get("note"), str):
                raise LearningError(f"invalid event {number}: malformed use")
        event["event_hash"] = claimed
        previous = claimed
        events.append(event)
    return events


def _validate_lesson(lesson: object) -> None:
    if not isinstance(lesson, dict):
        raise LearningError("lesson must be a JSON object")
    for field in ("id", "title", "procedure"):
        value = lesson.get(field)
        if not isinstance(value, str) or not value.strip():
            raise LearningError(f"lesson requires a nonempty string {field}")
    topics = lesson.get("topics")
    if (not isinstance(topics, list) or not topics or
            any(not isinstance(topic, str) or not topic.strip() for topic in topics)):
        raise LearningError("lesson topics must be a nonempty list of nonempty strings")
    refs = lesson.get("evidence")
    if not isinstance(refs, list) or not refs:
        raise LearningError("lesson needs at least one evidence reference")
    for ref in refs:
        if not isinstance(ref, dict):
            raise LearningError("evidence entries must be objects")
        path, sha = ref.get("path"), ref.get("sha256")
        if not isinstance(path, str) or not path.strip():
            raise LearningError("evidence path must be a nonempty string")
        if (not isinstance(sha, str) or len(sha) != 64 or
                any(char not in "0123456789abcdef" for char in sha)):
            raise LearningError("evidence sha256 must be a lowercase 64-character hex digest")
    supersedes = lesson.get("supersedes")
    if supersedes is not None and (not isinstance(supersedes, str) or not supersedes.strip()):
        raise LearningError("supersedes must be a nonempty existing lesson ID")


def _check_supersedes(lesson: dict, state: dict[str, dict]) -> None:
    target = lesson.get("supersedes")
    if target is None:
        return
    if target == lesson["id"] or target not in state:
        raise LearningError("supersedes must reference an existing earlier lesson")
    visited = {lesson["id"]}
    current = target
    while current is not None:
        if current in visited:
            raise LearningError("supersedes relation would create a cycle")
        visited.add(current)
        previous = state.get(current)
        current = previous.get("supersedes") if previous else None


def _state(fd: int) -> dict[str, dict]:
    state: dict[str, dict] = {}
    for e in _events(fd):
        lesson_id = e["lesson_id"]
        if e["kind"] == "propose":
            if lesson_id in state:
                raise LearningError(f"duplicate lesson id: {lesson_id}")
            _check_supersedes(e["lesson"], state)
            state[lesson_id] = {**e["lesson"], "id": lesson_id, "status": "proposed", "uses": [], "outcomes": []}
        elif lesson_id not in state:
            raise LearningError(f"event references unknown lesson: {lesson_id}")
        elif e["kind"] == "review":
            if state[lesson_id]["status"] != "proposed":
                raise LearningError("review event has invalid prior state")
            state[lesson_id]["status"] = "reviewed"
            state[lesson_id]["review"] = e["review"]
        elif e["kind"] == "validate":
            if state[lesson_id]["status"] != "reviewed":
                raise LearningError("validation event without reviewed state")
            state[lesson_id]["status"] = "validated"
            state[lesson_id]["validated_at"] = e["at"]
            predecessor_id = state[lesson_id].get("supersedes")
            if predecessor_id is not None:
                predecessor = state.get(predecessor_id)
                if predecessor is None:
                    raise LearningError("validated successor references unknown predecessor")
                predecessor["status"] = "superseded"
                predecessor["superseded_by"] = lesson_id
        elif e["kind"] == "outcome":
            if state[lesson_id]["status"] != "validated":
                raise LearningError("outcome event has invalid prior state")
            state[lesson_id]["outcomes"].append(e["outcome"])
        elif e["kind"] == "use":
            if state[lesson_id]["status"] != "validated":
                raise LearningError("use event has invalid prior state")
            state[lesson_id]["uses"].append(e["use"])
    for lesson in state.values():
        counts: dict[str, int] = {}
        for item in lesson["outcomes"]:
            counts[item["result"]] = counts.get(item["result"], 0) + 1
        lesson["use_count"] = len(lesson["uses"])
        lesson["outcome_counts"] = counts
    return state


def _check_evidence(lesson: dict) -> None:
    root = ROOT.resolve()
    for ref in lesson["evidence"]:
        try:
            path = Path(ref["path"])
            target = (root / path).resolve(strict=True)
        except (OSError, RuntimeError, ValueError) as exc:
            raise LearningError(f"evidence path missing or invalid: {ref.get('path')}") from exc
        if root not in target.parents or not target.is_file():
            raise LearningError(f"evidence path missing or outside repository: {ref['path']}")
        if _hash_file(target, MAX_EVIDENCE) != ref["sha256"]:
            raise LearningError(f"stale evidence: {ref['path']}")


def _seed() -> list[dict]:
    refs = ["conductor/harness/single-maintainer.md",
            "conductor/evidence/c2-track22-checkpoint-owner-review-20261007/review-packet.md",
            "tools/agent_session.py"]
    evidence = [{"path": p, "sha256": digest(ROOT / p)} for p in refs]
    return [
        {"id": "bind-exact-worktree", "title": "Bind work to an exact clean worktree and HEAD",
         "topics": ["worktree", "git", "claim"],
         "procedure": "Verify the named worktree, branch, HEAD and clean status; acquire a path claim there before writing.",
         "evidence": evidence[:1]},
        {"id": "roles-are-not-people", "title": "Treat owner roles as decision gates, not assumed people",
         "topics": ["ownership", "review", "authority"],
         "procedure": "Record the required owner role and decision; do not infer a person, response, or acceptance when the evidence is blank.",
         "evidence": evidence[:2]},
        {"id": "stop-repeating-stale-audits", "title": "Do not repeat a no-change audit without a new signal",
         "topics": ["audit", "progress", "evidence"],
         "procedure": "Compare the current evidence hash and state with the last readback; if unchanged, record no progress and wait for a new signal or choose an independent bounded action.",
         "evidence": evidence[:2]},
        {"id": "inspect-claim-inputs", "title": "Inspect claim inputs and bounds before retrying",
         "topics": ["claim", "budget", "harness"],
         "procedure": "Read the command help and input sizes, reduce the selected context to required tracked files, then retry once; do not repeat the same failing arguments.",
         "evidence": [evidence[0], evidence[2]]},
    ]


def _append(fd: int, event: dict) -> None:
    events = _events(fd)
    event = {**event, "prev_hash": events[-1]["event_hash"] if events else ZERO}
    event["event_hash"] = hashlib.sha256(_canonical(event)).hexdigest()
    encoded = _canonical(event) + b"\n"
    old_size = os.fstat(fd).st_size
    if len(encoded) > MAX_LINE or old_size + len(encoded) > MAX_STORE:
        raise LearningError("append would exceed store limits")
    if os.write(fd, encoded) != len(encoded):
        raise LearningError("short append; store may need repair")


def _propose(fd: int, lesson: dict) -> None:
    _validate_lesson(lesson)
    state = _state(fd)
    if lesson["id"] in state:
        raise LearningError("lesson IDs are immutable and cannot be reused")
    _check_supersedes(lesson, state)
    _check_evidence(lesson)
    _append(fd, {"kind": "propose", "lesson_id": lesson["id"], "lesson": lesson, "at": int(time.time())})


def propose(lesson: dict) -> None:
    with _locked_store() as fd:
        _propose(fd, lesson)


def review(lesson_id: str, reviewer: str, note: str) -> None:
    with _locked_store() as fd:
        state = _state(fd)
        lesson = state.get(lesson_id)
        if not lesson or lesson["status"] != "proposed":
            raise LearningError("only a proposed lesson can be reviewed")
        if not reviewer.strip() or not note.strip():
            raise LearningError("reviewer and review note are required")
        _check_evidence(lesson)
        _append(fd, {"kind": "review", "lesson_id": lesson_id,
                     "review": {"reviewer": reviewer, "note": note}, "at": int(time.time())})


def validate(lesson_id: str) -> None:
    with _locked_store() as fd:
        state = _state(fd)
        lesson = state.get(lesson_id)
        if not lesson or lesson["status"] != "reviewed":
            raise LearningError("only a reviewed lesson can be validated")
        _check_evidence(lesson)
        _append(fd, {"kind": "validate", "lesson_id": lesson_id, "at": int(time.time())})


def search(query: str, limit: int = 5, include_superseded: bool = False) -> list[dict]:
    if not isinstance(limit, int) or limit < 1 or limit > 20:
        raise LearningError("search limit must be between 1 and 20")
    terms = set(query.casefold().split())
    with _locked_store() as fd:
        found = []
        for lesson in _state(fd).values():
            if lesson["status"] == "superseded" and not include_superseded:
                continue
            haystack = " ".join([lesson["title"], lesson["procedure"], *lesson["topics"]]).casefold()
            if not terms or all(term in haystack for term in terms):
                _check_evidence(lesson)
                if len(found) < limit:
                    found.append(lesson)
        return found


def show(lesson_id: str) -> dict:
    with _locked_store() as fd:
        lesson = _state(fd).get(lesson_id)
        if not lesson:
            raise LearningError(f"unknown lesson: {lesson_id}")
        _check_evidence(lesson)
        return lesson


def _summary(lesson: dict) -> dict:
    return {key: lesson[key] for key in
            ("id", "title", "status", "procedure", "evidence", "use_count", "outcome_counts")}


def outcome(lesson_id: str, result: str, note: str) -> None:
    with _locked_store() as fd:
        lesson = _state(fd).get(lesson_id)
        if not lesson or lesson["status"] != "validated":
            raise LearningError("outcomes are recorded only for validated lessons")
        if result not in OUTCOMES or not note.strip():
            raise LearningError("outcome requires an allowed result and note")
        _check_evidence(lesson)
        _append(fd, {"kind": "outcome", "lesson_id": lesson_id,
                     "outcome": {"result": result, "note": note, "at": int(time.time())}})


def record_use(lesson_id: str, note: str) -> None:
    with _locked_store() as fd:
        lesson = _state(fd).get(lesson_id)
        if not lesson or lesson["status"] != "validated":
            raise LearningError("only validated procedures can be recorded as used")
        if not note.strip():
            raise LearningError("use note is required")
        _check_evidence(lesson)
        _append(fd, {"kind": "use", "lesson_id": lesson_id,
                     "use": {"note": note, "at": int(time.time())}})


def _seed_all() -> int:
    with _locked_store() as fd:
        existing = _state(fd)
        added = 0
        for lesson in _seed():
            if lesson["id"] not in existing:
                _propose(fd, lesson)
                existing[lesson["id"]] = lesson
                added += 1
        return added


def _read_lesson_file(path: str) -> dict:
    file_path = Path(path)
    try:
        size = file_path.stat().st_size
        if size > MAX_LINE:
            raise LearningError("proposal file exceeds 32 KB")
        with file_path.open("rb") as stream:
            raw = stream.read(MAX_LINE + 1)
    except OSError as exc:
        raise LearningError(f"cannot read proposal file: {exc}") from exc
    if len(raw) > MAX_LINE or len(raw) != size:
        raise LearningError("proposal file changed or exceeds 32 KB")
    try:
        value = json.loads(raw)
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise LearningError(f"invalid proposal JSON: {exc}") from exc
    _validate_lesson(value)
    return value


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    sub.add_parser("seed", help="append four evidence-backed proposals once")
    s = sub.add_parser("search"); s.add_argument("query", nargs="*", default=[])
    s.add_argument("--limit", type=int, default=5)
    s.add_argument("--include-superseded", action="store_true")
    d = sub.add_parser("show", help="show the full record and feedback history for one lesson"); d.add_argument("id")
    r = sub.add_parser("review"); r.add_argument("id"); r.add_argument("--reviewer", required=True); r.add_argument("--note", required=True)
    v = sub.add_parser("validate"); v.add_argument("id")
    o = sub.add_parser("outcome"); o.add_argument("id"); o.add_argument("result", choices=sorted(OUTCOMES)); o.add_argument("--note", required=True)
    u = sub.add_parser("use", help="record an actual use of a validated procedure"); u.add_argument("id"); u.add_argument("--note", required=True)
    a = sub.add_parser("add", help="append a proposed lesson from a JSON file"); a.add_argument("json_file")
    args = parser.parse_args(argv)
    try:
        if args.command == "seed":
            print(f"appended {_seed_all()} proposed lessons")
        elif args.command == "add":
            propose(_read_lesson_file(args.json_file))
        elif args.command == "show": print(json.dumps(show(args.id), indent=2))
        elif args.command == "review": review(args.id, args.reviewer, args.note)
        elif args.command == "validate": validate(args.id)
        elif args.command == "outcome": outcome(args.id, args.result, args.note)
        elif args.command == "use": record_use(args.id, args.note)
        else: print(json.dumps([_summary(lesson) for lesson in search(
            " ".join(args.query), args.limit, args.include_superseded)], indent=2))
        return 0
    except (LearningError, OSError, json.JSONDecodeError, KeyError, TypeError, ValueError) as exc:
        print(f"learning: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
