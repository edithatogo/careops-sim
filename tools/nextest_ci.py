#!/usr/bin/env python3
"""Run the pinned native test suite and prove runner discovery/failure semantics."""

from __future__ import annotations

import argparse
from collections import Counter
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import sys

from install_nextest import InstallError, install_nextest


VERSION = "1.98.1"
SELECTION = ["--workspace", "--all-features", "--lib", "--bins", "--tests", "--locked"]
RED_TEST = "injected_failure_is_detected"
ANSI_RE = re.compile(r"\x1b\[[0-9;]*m")
CARGO_RUNNING_LINE = re.compile(r"^\s*Running .+ \((.+)\)\s*$")
CARGO_TEST_LINE = re.compile(r"^(.+): test(?: \(ignored\))?$")
SUMMARY_RE = re.compile(r"\b(\d+) tests? run:\s*(.*)$")


class QualificationError(RuntimeError):
    """Raised when toolchain, discovery, control or test checks fail."""


def supported_host(system=None, machine=None):
    system = system or platform.system()
    machine = (machine or platform.machine()).lower()
    if system == "Darwin" and machine in {"arm64", "aarch64"}:
        return "aarch64-apple-darwin"
    if system == "Linux" and machine in {"x86_64", "amd64"}:
        return "x86_64-unknown-linux-gnu"
    raise QualificationError(f"unsupported nextest qualification host: {system}/{machine}")


def parse_cargo_listing(text, assume_ignored=False):
    current_binary = None
    tests = []
    ignored = []
    for raw_line in text.splitlines():
        line = raw_line.strip()
        running = CARGO_RUNNING_LINE.fullmatch(line)
        if running:
            current_binary = Path(running.group(1)).name
            continue
        case = CARGO_TEST_LINE.fullmatch(line)
        if case:
            if not current_binary:
                raise QualificationError(f"Cargo listed a test before identifying its binary: {line}")
            identity = (current_binary, case.group(1))
            tests.append(identity)
            if assume_ignored or line.endswith(" (ignored)"):
                ignored.append(identity)
    return tests, ignored


def parse_nextest_listing(text):
    try:
        document = json.loads(text)
    except (TypeError, json.JSONDecodeError) as exc:
        raise QualificationError(f"nextest listing is not JSON: {exc}") from exc
    count = document.get("test-count")
    suites = document.get("rust-suites")
    if isinstance(count, bool) or not isinstance(count, int) or count < 0 or not isinstance(suites, dict):
        raise QualificationError("nextest listing is missing test-count or rust-suites")
    names = []
    ignored = []
    for suite_name, suite in suites.items():
        cases = suite.get("testcases") if isinstance(suite, dict) else None
        if not isinstance(cases, dict):
            raise QualificationError(f"nextest suite {suite_name!r} has no testcases map")
        binary_path = suite.get("binary-path")
        if not isinstance(binary_path, str) or not binary_path:
            raise QualificationError(f"nextest suite {suite_name!r} has no binary path")
        binary_id = Path(binary_path).name
        for name, metadata in cases.items():
            if not isinstance(name, str) or not isinstance(metadata, dict) or not isinstance(metadata.get("ignored"), bool):
                raise QualificationError(f"nextest test case {name!r} has invalid ignored metadata")
            identity = (binary_id, name)
            names.append(identity)
            if metadata["ignored"]:
                ignored.append(identity)
    if len(names) != count:
        raise QualificationError(f"nextest test-count {count} differs from listed testcase count {len(names)}")
    return names, ignored


def compare_discoveries(cargo_all, cargo_ignored, nextest_names, nextest_ignored):
    if not cargo_all:
        raise QualificationError("Cargo discovery returned no tests")
    if Counter(cargo_all) != Counter(nextest_names):
        only_cargo = list((Counter(cargo_all) - Counter(nextest_names)).elements())[:10]
        only_nextest = list((Counter(nextest_names) - Counter(cargo_all)).elements())[:10]
        raise QualificationError(f"test discovery mismatch: cargo_only={only_cargo}, nextest_only={only_nextest}")
    if Counter(cargo_ignored) != Counter(nextest_ignored):
        only_cargo = list((Counter(cargo_ignored) - Counter(nextest_ignored)).elements())[:10]
        only_nextest = list((Counter(nextest_ignored) - Counter(cargo_ignored)).elements())[:10]
        raise QualificationError(f"ignored-test discovery mismatch: cargo_only={only_cargo}, nextest_only={only_nextest}")
    if len(cargo_all) != len(nextest_names):
        raise QualificationError("test discovery counts differ")
    return {"test_count": len(cargo_all), "ignored_count": len(cargo_ignored)}


def red_control_proved(exit_code, output, test_name=RED_TEST):
    normalized = ANSI_RE.sub("", output)
    failure_lines = [line for line in normalized.splitlines() if re.match(r"^\s*FAIL\b", line)]
    failure_ids = {
        re.sub(r"^\s*FAIL\s*\[[^\]]+\]\s*(?:\(\d+/\d+\)\s*)?", "", line).strip()
        for line in failure_lines
    }
    summaries = [SUMMARY_RE.search(line) for line in normalized.splitlines()]
    summaries = [match for match in summaries if match]
    if (exit_code != 100 or not failure_lines or len(failure_ids) != 1
            or not all(test_name in identity for identity in failure_ids) or len(summaries) != 1):
        return False
    run_count = int(summaries[0].group(1))
    counts = summaries[0].group(2)
    failed = re.search(r"\b(\d+) failed\b", counts)
    passed = re.search(r"\b(\d+) passed\b", counts)
    if run_count != 1 or failed is None or int(failed.group(1)) != 1:
        return False
    return passed is None or int(passed.group(1)) == 0


def parse_run_summary(text):
    normalized = ANSI_RE.sub("", text)
    summaries = [SUMMARY_RE.search(line) for line in normalized.splitlines()]
    summaries = [match for match in summaries if match]
    if len(summaries) != 1:
        raise QualificationError("nextest output did not contain exactly one run summary")
    summary = summaries[0]
    counts = summary.group(2)

    def count(label):
        match = re.search(rf"\b(\d+) {label}\b", counts)
        return int(match.group(1)) if match else 0

    return {"run_count": int(summary.group(1)), "passed": count("passed"),
            "failed": count("failed"), "skipped": count("skipped")}


def validate_run_summary(summary, discovery):
    expected_run_count = discovery["test_count"] - discovery["ignored_count"]
    if summary["run_count"] != expected_run_count:
        raise QualificationError(
            f"executed count {summary['run_count']} differs from non-ignored discovery {expected_run_count}"
        )
    if summary["skipped"] != discovery["ignored_count"]:
        raise QualificationError(
            f"nextest skipped count {summary['skipped']} differs from ignored inventory {discovery['ignored_count']}"
        )
    if summary["passed"] + summary["failed"] + summary["skipped"] != discovery["test_count"]:
        raise QualificationError("nextest summary counts do not reconcile with discovered test inventory")


def _run(argv, cwd, env, log_path, json_stdout=False, merge_stderr=False):
    log_path.parent.mkdir(parents=True, exist_ok=True)
    try:
        if merge_stderr:
            completed = subprocess.run(argv, cwd=cwd, env=env, stdout=subprocess.PIPE,
                                       stderr=subprocess.STDOUT, text=True, check=False)
        else:
            completed = subprocess.run(argv, cwd=cwd, env=env, capture_output=True, text=True, check=False)
    except OSError as exc:
        raise QualificationError(f"could not execute {argv[0]}: {exc}") from exc
    if json_stdout:
        log_path.write_text(completed.stdout)
        if completed.stderr:
            with (log_path.parent / "environment.log").open("a") as stream:
                stream.write(f"\n[{log_path.name} stderr]\n{completed.stderr}")
    else:
        log_path.write_text(completed.stdout + ("\n[stderr]\n" + completed.stderr if completed.stderr else ""))
    if completed.returncode != 0:
        tail = (completed.stderr or completed.stdout)[-3000:]
        raise QualificationError(f"command exit {completed.returncode}: {argv!r}\n{tail}")
    return completed


def _toolchain_paths(version, env):
    paths = {}
    for tool in ("rustc", "rustdoc", "cargo"):
        result = subprocess.run(["rustup", "which", tool, "--toolchain", version],
                                capture_output=True, text=True, check=False, env=env)
        if result.returncode != 0:
            raise QualificationError(f"rustup could not resolve {tool} for {version}: {result.stderr.strip()}")
        path = Path(result.stdout.strip())
        if not path.is_file() or not os.access(path, os.X_OK):
            raise QualificationError(f"resolved {tool} is not executable: {path}")
        paths[tool] = str(path)
    return paths


def _record_toolchain(paths, env, out_dir):
    rustc = subprocess.run([paths["rustc"], "-vV"], capture_output=True, text=True, check=False, env=env)
    rustdoc = subprocess.run([paths["rustdoc"], "--version"], capture_output=True, text=True, check=False, env=env)
    cargo = subprocess.run([paths["cargo"], "-vV"], capture_output=True, text=True, check=False, env=env)
    if rustc.returncode or rustdoc.returncode or cargo.returncode:
        raise QualificationError("pinned compiler identity command failed")
    host_match = re.search(r"(?m)^host: (\S+)$", rustc.stdout)
    if not host_match:
        raise QualificationError("rustc -vV omitted host triple")
    expected = supported_host()
    if host_match.group(1) != expected:
        raise QualificationError(f"detected compiler host {host_match.group(1)} disagrees with machine {expected}")
    if not rustc.stdout.startswith(f"rustc {VERSION} ") or not rustdoc.stdout.startswith(f"rustdoc {VERSION} "):
        raise QualificationError("resolved rustc/rustdoc version differs from pinned toolchain")
    cargo_host = re.search(r"(?m)^host: (\S+)$", cargo.stdout)
    if not cargo.stdout.startswith(f"cargo {VERSION} ") or not cargo_host or cargo_host.group(1) != expected:
        raise QualificationError("resolved Cargo version or host differs from pinned toolchain")
    out_dir.mkdir(parents=True, exist_ok=True)
    env_log = out_dir / "environment.log"
    env_log.write_text(
        f"host={expected}\n"
        f"rustc_path={paths['rustc']}\n{rustc.stdout}"
        f"rustdoc_path={paths['rustdoc']}\n{rustdoc.stdout}"
        f"cargo_path={paths['cargo']}\n{cargo.stdout}"
    )
    return expected


def _write_red_control(root, out_dir):
    fixture = out_dir / "red-control"
    (fixture / "src").mkdir(parents=True, exist_ok=True)
    (fixture / "Cargo.toml").write_text(
        '[package]\nname = "nextest-red-control"\nversion = "0.0.0"\nedition = "2021"\n\n[workspace]\n'
    )
    (fixture / "Cargo.lock").write_text(
        '# This file is automatically @generated by Cargo.\n# It is not intended for manual editing.\nversion = 3\n\n'
        '[[package]]\nname = "nextest-red-control"\nversion = "0.0.0"\n'
    )
    (fixture / "src" / "lib.rs").write_text(
        '#[cfg(test)]\nmod tests {\n'
        '    #[test]\n'
        '    fn injected_failure_is_detected() {\n'
        '        assert_eq!(1, 2, "nextest red-control fixture must fail");\n'
        '    }\n'
        '}\n'
    )
    return fixture


def qualify(toolchain=VERSION):
    if toolchain != VERSION:
        raise QualificationError(f"only Rust {VERSION} is supported for nextest qualification")
    root = Path(__file__).resolve().parents[1]
    out_dir = root / ".artifacts" / "ci" / "nextest"
    install_dir = root / ".artifacts" / "ci" / "nextest-tools"
    out_dir.mkdir(parents=True, exist_ok=True)

    initial_env = os.environ.copy()
    initial_env["PYTHONDONTWRITEBYTECODE"] = "1"
    paths = _toolchain_paths(toolchain, initial_env)
    toolchain_bin = str(Path(paths["rustc"]).parent)
    host = _record_toolchain(paths, initial_env, out_dir)

    install = install_nextest(install_dir)
    (out_dir / "environment.log").open("a").write(
        f"nextest_host={host}\nnextest_path={install['path']}\n"
        f"nextest_version={install['version']}\nnextest_archive={install['archive']}\n"
        f"nextest_archive_sha256={install['archive_sha256']}\nnextest_binary_sha256={install['sha256']}\n"
    )

    command_env = initial_env.copy()
    command_env.update({
        "RUSTUP_TOOLCHAIN": toolchain,
        "RUSTC": paths["rustc"],
        "RUSTDOC": paths["rustdoc"],
        "CARGO_TARGET_DIR": str(out_dir / "target"),
        "PATH": f"{Path(install['path']).parent}:{toolchain_bin}:{os.environ.get('PATH', '')}",
    })
    cargo = paths["cargo"]
    all_list = _run([cargo, "test", *SELECTION, "--", "--list"], root, command_env,
                    out_dir / "cargo-all-list.log", merge_stderr=True)
    ignored_list = _run([cargo, "test", *SELECTION, "--", "--list", "--ignored"], root,
                        command_env, out_dir / "cargo-ignored-list.log", merge_stderr=True)
    nextest_list = _run([cargo, "nextest", "list", *SELECTION, "--message-format", "json"],
                         root, command_env, out_dir / "nextest-list.json", json_stdout=True)
    cargo_names, _cargo_all_ignored = parse_cargo_listing(all_list.stdout)
    _ignored_only_all, cargo_ignored = parse_cargo_listing(ignored_list.stdout, assume_ignored=True)
    nextest_names, nextest_ignored = parse_nextest_listing(nextest_list.stdout)
    discovery = compare_discoveries(cargo_names, cargo_ignored, nextest_names, nextest_ignored)
    with (out_dir / "environment.log").open("a") as stream:
        stream.write(f"discovery_test_count={discovery['test_count']}\n")
        stream.write(f"discovery_ignored_count={discovery['ignored_count']}\n")

    fixture = _write_red_control(root, out_dir)
    red_argv = [cargo, "nextest", "run", "--manifest-path", str(fixture / "Cargo.toml"),
                "--locked", "--retries", "0", "--no-fail-fast", "--no-tests", "fail"]
    red = subprocess.run(red_argv, cwd=root, env=command_env, capture_output=True, text=True, check=False)
    red_output = red.stdout + ("\n[stderr]\n" + red.stderr if red.stderr else "")
    (out_dir / "red-control.log").write_text(red_output)
    if not red_control_proved(red.returncode, red_output):
        raise QualificationError(f"red control was not proven (exit={red.returncode}); see {out_dir / 'red-control.log'}")
    with (out_dir / "environment.log").open("a") as stream:
        stream.write(f"red_control_exit={red.returncode}\nred_control_result=one named test failure\n")

    run_argv = [cargo, "nextest", "run", *SELECTION, "--retries", "0", "--no-fail-fast",
                "--no-tests", "fail", "--flaky-result", "fail"]
    result = _run(run_argv, root, command_env, out_dir / "tests.log")
    summary = parse_run_summary(result.stdout + result.stderr)
    if summary["failed"] != 0 or summary["run_count"] == 0 or summary["passed"] == 0:
        raise QualificationError(f"unexpected nextest test summary: {summary}")
    validate_run_summary(summary, discovery)
    with (out_dir / "environment.log").open("a") as stream:
        stream.write(f"nextest_run_count={summary['run_count']}\n")
        stream.write(f"nextest_passed={summary['passed']}\nnextest_failed={summary['failed']}\n")
        stream.write(f"nextest_skipped={summary['skipped']}\nnextest_exit={result.returncode}\n")
    return {"host": host, "toolchain": toolchain, **install, **discovery, **summary,
            "red_control_exit": red.returncode}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--toolchain", choices=[VERSION], required=True)
    args = parser.parse_args(argv)
    try:
        result = qualify(args.toolchain)
    except (InstallError, QualificationError, OSError, ValueError) as exc:
        print(f"nextest-ci: {exc}", file=sys.stderr)
        return 1
    print(json.dumps(result, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
