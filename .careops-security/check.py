# Copyright (c) 2026 CareOps contributors.
# SPDX-License-Identifier: Apache-2.0
# ruff: noqa: INP001 -- standalone CLI scripts, not an importable package.
"""Index-based privacy checks. Findings contain rule and path, never matched text."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path, PurePosixPath

GIT = shutil.which("git")

RULES = {
    "windows-machine-path": re.compile(
        rb"(?i)\b[a-z]:\\+(?:Users|Documents and Settings)\\+"
        rb"[^\\\r\n\"\x27]{1,2048}(?=\\|[\"\x27\r\n]|$)"
    ),
    "organisation-email": re.compile(
        rb"(?i)[a-z0-9._%+\-]{1,64}@(?:[a-z0-9-]{1,63}\.){0,5}health\.[a-z.]+\.gov\.au"
    ),
    "private-machine-path": re.compile(rb'/(?:Users|Volumes)/[^/\s"\x27]+/'),
    "patient-identifier": re.compile(
        rb'(?i)\b(?:MRN|UR\s*number|patient[_ -]?id)\s*[=:]\s*["\x27]?\d{6,}\b'
    ),
    "private-key": re.compile(rb"-----BEGIN (?:[A-Z0-9]+ )*PRIVATE KEY-----"),
}
REVIEW = {
    ".doc",
    ".docx",
    ".docm",
    ".pdf",
    ".xls",
    ".xlsx",
    ".xlsm",
    ".xlsb",
    ".ppt",
    ".pptx",
    ".zip",
    ".gz",
    ".7z",
    ".tar",
    ".png",
    ".jpg",
    ".jpeg",
    ".gif",
    ".webp",
    ".sqlite",
    ".db",
    ".pst",
    ".eml",
    ".msg",
    ".pfx",
    ".p12",
    ".pem",
    ".key",
}
MAX_BYTES = 5 * 1024 * 1024


def git(*args: str) -> bytes:
    """Read a supported Git operation without invoking a shell."""
    return subprocess.check_output(  # noqa: S603 -- fixed Git command, no shell
        [GIT, *args]
    )


def read_snapshot(path: str, revision: str | None = None) -> bytes:
    """Read the exact indexed or committed blob."""
    return git("show", (revision or "") + ":" + path)


def findings(path: str, blob: bytes, approvals: dict) -> list[str]:
    """Return rule identifiers without returning sensitive matched content."""
    issues = []
    parts = PurePosixPath(path).parts
    name = parts[-1].lower()
    private_parts = {
        ".local",
        ".flowagent",
        ".auth",
        "private",
        "local-only",
        "receipts",
        "outputs",
        "__pycache__",
        "node_modules",
    }
    if private_parts.intersection(part.lower() for part in parts) or any(
        part.lower().startswith(".venv") for part in parts
    ):
        issues.append("private-or-generated-directory")
    if name == ".env" or (
        name.startswith(".env.") and not name.endswith((".example", ".sample", ".template"))
    ):
        issues.append("environment-file")
    if name in {
        "credentials.json",
        "token.json",
        "tokens.json",
        "id_rsa",
        "id_ed25519",
    } or name.endswith((".private.json", ".private.jsonl")):
        issues.append("credential-or-private-file")
    if len(blob) > MAX_BYTES:
        return [*issues, "oversize-file-review-required"]
    issues += [rule for rule, pattern in RULES.items() if pattern.search(blob)]
    try:
        blob.decode("utf-8")
        binary = b"\0" in blob
    except UnicodeDecodeError:
        binary = True
    if binary or PurePosixPath(path).suffix.lower() in REVIEW:
        approval = approvals.get(path, {})
        if (
            approval.get("sha256") != hashlib.sha256(blob).hexdigest()
            or not str(approval.get("reason", "")).strip()
        ):
            issues.append("document-or-binary-review-required")
    return issues


def check(  # noqa: C901, PLR0912, PLR0915 -- one consolidated index/history audit
    base: str | None = None,
    *,
    all_files: bool = False,
    revision: str | None = None,
    history: bool = False,
) -> list[tuple[str, str]]:
    """Validate source continuity and collect index or outgoing-commit findings."""
    entries = {}
    errors = []
    listing = git("ls-tree", "-r", "-z", revision) if revision else git("ls-files", "--stage", "-z")
    for item in listing.split(b"\0"):
        if not item:
            continue
        meta, raw = item.split(b"\t", 1)
        path = raw.decode("utf-8")
        if revision:
            mode, _, _ = meta.decode().split()
        else:
            mode, _, stage = meta.decode().split()
            if stage != "0":
                errors.append(("unmerged-index", path))
        entries[path] = mode
    policy = json.loads(read_snapshot(".careops-security/policy.json", revision))
    inventory_name = (
        "PUBLIC_SOURCE_FILES.json"
        if "PUBLIC_SOURCE_FILES.json" in entries
        else ".careops-security/inventory.json"
    )
    inventory = json.loads(read_snapshot(inventory_name, revision))["files"]
    if len(inventory) != len(set(inventory)):
        errors.append(("duplicate-inventory-entry", inventory_name))
    errors.extend(("inventory-mismatch", path) for path in sorted(set(entries) ^ set(inventory)))
    if all_files or history:
        selected = set(entries)
    else:
        args = ["diff", "--name-only", "--diff-filter=ACMRT", "-z"]
        if not revision:
            args.append("--cached")
        if base:
            args.append(base)
        if revision:
            args.append(revision)
        selected = {p.decode("utf-8") for p in git(*args).split(b"\0") if p}
    approvals = policy.get("reviewedArtifacts", {})
    # Revalidate exact prior approvals when policy changes, including revocations.
    if ".careops-security/policy.json" in selected:
        previous = base or "HEAD"
        prior = subprocess.run(  # noqa: S603 -- controlled local process arguments, no shell
            [GIT, "show", previous + ":.careops-security/policy.json"],
            capture_output=True,
            check=False,
        )
        old = json.loads(prior.stdout) if prior.returncode == 0 else {}
        selected |= set(old.get("reviewedArtifacts", {})) | set(approvals)
    seen = set()

    def inspect(path: str, ref: str | None, mode: str) -> None:
        if mode == "160000":
            return
        if mode not in {"100644", "100755"}:
            errors.append(("unsupported-file-mode", path))
            return
        spec = (ref or "") + ":" + path
        oid = git("rev-parse", "--verify", spec).strip()
        if (path, oid) in seen:
            return
        seen.add((path, oid))
        if int(git("cat-file", "-s", spec)) > MAX_BYTES:
            errors.append(("oversize-file-review-required", path))
            return
        errors.extend((rule, path) for rule in findings(path, read_snapshot(path, ref), approvals))

    for path in sorted(selected & entries.keys()):
        inspect(path, revision, entries[path])
    # Scan intermediate commit blobs too: add-then-delete must not evade the gate.
    if base or history:
        tip = revision or "HEAD"
        selection = base + ".." + tip if base else tip
        commits = git("rev-list", selection).decode().splitlines()
        for commit in commits:
            changes = git(
                "diff-tree",
                "--root",
                "--no-commit-id",
                "-r",
                "-m",
                "--no-renames",
                "--diff-filter=ACMT",
                "--name-only",
                "-z",
                commit,
            )
            for raw in changes.split(b"\0"):
                if not raw:
                    continue
                path = raw.decode("utf-8")
                entry = git("ls-tree", "-z", commit, "--", path).split(b"\t", 1)[0]
                if entry:
                    inspect(path, commit, entry.decode().split()[0])
    return sorted(set(errors))


def main() -> int:
    """Run the repository-local CLI and emit content-free findings."""
    root = Path(git("rev-parse", "--show-toplevel").decode().strip()).resolve()
    if (
        sys.version_info[:3] != (3, 15, 1)
        or sys.prefix == sys.base_prefix
        or not Path(sys.prefix).resolve().is_relative_to(root)
    ):
        message = "Use a verified repository-local Python 3.15.1 virtual environment."
        raise SystemExit(message)
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", help="Compare this commit to the index (CI push/PR range)")
    parser.add_argument(
        "--all", action="store_true", help="Audit all tracked files, including legacy content"
    )
    parser.add_argument("--revision", help="Exact commit to inspect instead of the index")
    parser.add_argument(
        "--history",
        action="store_true",
        help="Audit every reachable commit; use for a new remote ref",
    )
    args = parser.parse_args()
    for value in [args.base, args.revision]:
        if value and not re.fullmatch(r"[a-f0-9]{40}", value):
            parser.error("Revisions must be full commit object IDs.")
    for rule, path in (
        errors := check(args.base, all_files=args.all, revision=args.revision, history=args.history)
    ):
        print(json.dumps({"rule": rule, "path": path}, ensure_ascii=True), file=sys.stderr)  # noqa: T201 -- CLI findings contain no matched content
    print(f"Privacy check: {len(errors)} finding(s). No matched content logged.")  # noqa: T201 -- CLI summary
    return bool(errors)


if __name__ == "__main__":
    raise SystemExit(main())
