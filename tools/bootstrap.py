#!/usr/bin/env python3
"""Read-only validation of the repository's pinned Rust toolchain."""

import argparse
import json
import platform
import re
import shutil
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PROFILE_PATH = ROOT / "tools" / "bootstrap-profile.json"
KAIROS_PATH = ROOT / "libs" / "kairos"
CORE_PACKAGES = (
    "kairo-ecs-core",
    "kairo-ecs-state",
    "kairo-ecs-rng",
    "kairo-ecs-des",
    "kairo-ecs-abm",
)


class BootstrapError(Exception):
    pass


def detected_host(system=None, machine=None, libc=None):
    system = system or platform.system()
    machine = (machine or platform.machine()).lower()
    if system == "Darwin" and machine in {"arm64", "aarch64"}:
        return "aarch64-apple-darwin"
    if system == "Linux" and machine in {"x86_64", "amd64"}:
        libc = (libc if libc is not None else platform.libc_ver()[0]).lower()
        if libc == "glibc":
            return "x86_64-unknown-linux-gnu"
    return None


def _run(rustup, toolchain, executable):
    try:
        result = subprocess.run(
            [rustup, "run", toolchain, executable, "--version", "--verbose"],
            check=False,
            capture_output=True,
            text=True,
        )
    except OSError as exc:
        raise BootstrapError(f"Could not run rustup: {exc}") from exc
    output = result.stdout + result.stderr
    if result.returncode:
        raise BootstrapError(
            f"rustup could not run {executable} for toolchain {toolchain!r} "
            f"(exit {result.returncode}). Install that exact toolchain with "
            f"'rustup toolchain install {toolchain}' and retry. Details: {output.strip()}"
        )
    return output


def _field(output, name):
    match = re.search(rf"^{re.escape(name)}:\s*(\S+)\s*$", output, re.MULTILINE)
    return match.group(1) if match else None


def check(profile_path=PROFILE_PATH, host=None, rustup_path=None):
    profile = json.loads(Path(profile_path).read_text(encoding="utf-8"))
    version = profile["rust_version"]
    toolchain = profile["rustup_toolchain"]
    supported = profile["supported_hosts"]
    host = host if host is not None else detected_host()
    if host not in supported:
        choices = ", ".join(supported)
        raise BootstrapError(
            f"Unsupported host {host or (platform.system() + '/' + platform.machine())}. "
            f"This profile supports: {choices}. Use a supported host or propose and "
            "review a profile update."
        )

    rustup = rustup_path or shutil.which("rustup")
    if not rustup:
        raise BootstrapError(
            "rustup was not found on PATH. Install rustup from https://rustup.rs/, "
            "then install the pinned toolchain with "
            f"'rustup toolchain install {toolchain}' and rerun this check."
        )

    rustc_output = _run(rustup, toolchain, "rustc")
    cargo_output = _run(rustup, toolchain, "cargo")
    rustc_line = rustc_output.splitlines()[0] if rustc_output.splitlines() else ""
    cargo_line = cargo_output.splitlines()[0] if cargo_output.splitlines() else ""
    if not re.match(rf"^rustc {re.escape(version)}(?:\s|$)", rustc_line):
        raise BootstrapError(
            f"Pinned rustc {version} required, received {rustc_line or 'no version output'}. "
            f"Run 'rustup toolchain install {toolchain}' and retry."
        )
    if not re.match(rf"^cargo {re.escape(version)}(?:\s|$)", cargo_line):
        raise BootstrapError(
            f"Pinned cargo {version} required, received {cargo_line or 'no version output'}. "
            f"Run 'rustup toolchain install {toolchain}' and retry."
        )
    rustc_host = _field(rustc_output, "host")
    cargo_host = _field(cargo_output, "host")
    if rustc_host != host or cargo_host != host:
        raise BootstrapError(
            f"Host mismatch: machine is {host}, rustc reports {rustc_host or 'unknown'}, "
            f"and cargo reports {cargo_host or 'unknown'}. Install/use the pinned "
            f"{toolchain} toolchain for {host} and retry."
        )
    print(f"Rust {version} ready for {host} (rustc and cargo verified via rustup).")


def test_core(profile_path=PROFILE_PATH):
    """Verify the pinned Rust tools, then run Kairos' locked core test suite."""
    check(profile_path=profile_path)
    cargo_manifest = KAIROS_PATH / "Cargo.toml"
    if not cargo_manifest.is_file():
        raise BootstrapError(
            f"Kairos submodule is missing or uninitialized at {KAIROS_PATH}. "
            "From the repository root, run 'git submodule update --init --recursive', "
            "then rerun 'python3 tools/bootstrap.py --test-core'."
        )

    profile = json.loads(Path(profile_path).read_text(encoding="utf-8"))
    toolchain = profile["rustup_toolchain"]
    command = ["rustup", "run", toolchain, "cargo", "test", "--locked"]
    for package in CORE_PACKAGES:
        command.extend(("-p", package))
    try:
        result = subprocess.run(command, cwd=KAIROS_PATH, check=False)
    except OSError as exc:
        raise BootstrapError(f"Could not run pinned Cargo test suite: {exc}") from exc
    if result.returncode:
        raise BootstrapError(
            f"Pinned Kairos core test suite failed with exit {result.returncode}."
        )


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--check", action="store_true", help="check the pinned toolchain")
    group.add_argument(
        "--test-core", action="store_true",
        help="check the pinned toolchain and run the locked Kairos core tests",
    )
    args = parser.parse_args(argv)
    try:
        if args.test_core:
            test_core()
        else:
            check()
    except (BootstrapError, OSError, ValueError, KeyError) as exc:
        print(f"Rust toolchain check failed: {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
