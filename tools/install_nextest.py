#!/usr/bin/env python3
"""Install the checksum-pinned cargo-nextest release without package managers."""

from __future__ import annotations

import argparse
import gzip
import hashlib
import io
import os
from pathlib import Path, PurePosixPath
import platform
import re
import subprocess
import sys
import tarfile
import tempfile
import urllib.request


VERSION = "0.9.146"
ARCHIVES = {
    "universal-apple-darwin": {
        "url": "https://github.com/nextest-rs/nextest/releases/download/cargo-nextest-0.9.146/cargo-nextest-0.9.146-universal-apple-darwin.tar.gz",
        "sha256": "39785160b3c2f6ed9a765049cf4fa79f3b39aa02eb7598a5a0e2a1a0b9ffb9a8",
    },
    "x86_64-unknown-linux-gnu": {
        "url": "https://github.com/nextest-rs/nextest/releases/download/cargo-nextest-0.9.146/cargo-nextest-0.9.146-x86_64-unknown-linux-gnu.tar.gz",
        "sha256": "682c21b777c333e96fd532e114d3a5a894e0729ab88d94c0a9f20f8419695428",
    },
}
MAX_DOWNLOAD_BYTES = 64 * 1024 * 1024
MAX_UNPACKED_BYTES = 192 * 1024 * 1024
MAX_MEMBER_BYTES = 128 * 1024 * 1024
MAX_MEMBERS = 512
VERSION_RE = re.compile(r"^cargo-nextest 0\.9\.146(?:\s|$)")


class InstallError(RuntimeError):
    """Raised for unsupported, corrupt, unsafe or unverified releases."""


class _BoundedReader:
    """Count decompressed tar bytes so small gzip bombs fail closed."""

    def __init__(self, reader, limit):
        self._reader = reader
        self._limit = limit
        self.count = 0

    def read(self, size=-1):
        if size < 0 or size > self._limit - self.count + 1:
            size = self._limit - self.count + 1
        data = self._reader.read(size)
        self.count += len(data)
        if self.count > self._limit:
            raise InstallError("unpacked archive exceeds size limit")
        return data


def release_for_host(system=None, machine=None):
    system = system or platform.system()
    machine = (machine or platform.machine()).lower()
    if system == "Darwin" and machine in {"arm64", "aarch64"}:
        return "universal-apple-darwin"
    if system == "Linux" and machine in {"x86_64", "amd64"}:
        return "x86_64-unknown-linux-gnu"
    raise InstallError(f"unsupported cargo-nextest host: {system}/{machine}")


def download_archive(url):
    request = urllib.request.Request(url, headers={"User-Agent": "careops-nextest-installer/1"})
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            length = response.headers.get("Content-Length")
            if length is not None and int(length) > MAX_DOWNLOAD_BYTES:
                raise InstallError("archive exceeds download size limit")
            data = response.read(MAX_DOWNLOAD_BYTES + 1)
    except (OSError, ValueError) as exc:
        raise InstallError(f"could not download pinned archive: {exc}") from exc
    if len(data) > MAX_DOWNLOAD_BYTES:
        raise InstallError("archive exceeds download size limit")
    return data


def _safe_member_name(name):
    if not name or "\\" in name:
        return False
    path = PurePosixPath(name)
    return not path.is_absolute() and ".." not in path.parts and all(part not in {"", "."} for part in path.parts)


def select_binary(archive_bytes):
    if not archive_bytes or len(archive_bytes) > MAX_DOWNLOAD_BYTES:
        raise InstallError("archive is empty or exceeds download size limit")
    payloads = []
    seen = set()
    unpacked = 0
    compressed = io.BytesIO(archive_bytes)
    try:
        with gzip.GzipFile(fileobj=compressed, mode="rb") as gzip_stream:
            source = _BoundedReader(gzip_stream, MAX_UNPACKED_BYTES)
            with tarfile.open(fileobj=source, mode="r|") as archive:
                for index, member in enumerate(archive):
                    if index >= MAX_MEMBERS:
                        raise InstallError("archive has too many members")
                    if not _safe_member_name(member.name):
                        raise InstallError(f"unsafe archive member: {member.name!r}")
                    normalized = str(PurePosixPath(member.name))
                    if normalized in seen:
                        raise InstallError(f"duplicate archive member: {normalized}")
                    seen.add(normalized)
                    if member.issym() or member.islnk() or member.isdev() or member.isfifo():
                        raise InstallError(f"link or special archive member rejected: {member.name}")
                    if member.isdir():
                        continue
                    if not member.isfile():
                        raise InstallError(f"non-regular archive member rejected: {member.name}")
                    if member.size < 0 or member.size > MAX_MEMBER_BYTES:
                        raise InstallError(f"archive member exceeds size limit: {member.name}")
                    unpacked += member.size
                    if unpacked > MAX_UNPACKED_BYTES:
                        raise InstallError("unpacked archive exceeds size limit")
                    if PurePosixPath(member.name).name == "cargo-nextest":
                        extracted = archive.extractfile(member)
                        if extracted is None:
                            raise InstallError("cargo-nextest payload cannot be read")
                        payload = extracted.read(MAX_MEMBER_BYTES + 1)
                        if len(payload) != member.size or len(payload) > MAX_MEMBER_BYTES:
                            raise InstallError("cargo-nextest payload is truncated or oversized")
                        payloads.append(payload)
    except InstallError:
        raise
    except (tarfile.TarError, OSError, EOFError, ValueError) as exc:
        raise InstallError(f"malformed release archive: {exc}") from exc
    if len(payloads) != 1:
        raise InstallError(f"expected one regular cargo-nextest payload, found {len(payloads)}")
    if not payloads[0]:
        raise InstallError("cargo-nextest payload is empty")
    return payloads[0]


def _run_version(binary, runner=subprocess.run):
    try:
        result = runner([str(binary), "--version"], capture_output=True, text=True, check=False, timeout=10)
    except (OSError, subprocess.SubprocessError) as exc:
        raise InstallError(f"cannot execute cargo-nextest version check: {exc}") from exc
    output = (result.stdout or "").strip()
    if result.returncode != 0 or not VERSION_RE.match(output):
        raise InstallError(f"unexpected cargo-nextest runtime version: exit={result.returncode}, output={output!r}")
    return output


def install_nextest(install_dir, system=None, machine=None, downloader=download_archive, runner=subprocess.run):
    platform_key = release_for_host(system, machine)
    release = ARCHIVES[platform_key]
    archive_bytes = downloader(release["url"])
    if len(archive_bytes) > MAX_DOWNLOAD_BYTES:
        raise InstallError("archive exceeds download size limit")
    digest = hashlib.sha256(archive_bytes).hexdigest()
    if digest != release["sha256"]:
        raise InstallError(f"archive SHA-256 mismatch: {digest}")
    payload = select_binary(archive_bytes)

    repository_root = Path(__file__).resolve().parents[1]
    destination = Path(install_dir)
    if not destination.is_absolute():
        destination = repository_root / destination
    destination = destination.resolve()
    try:
        destination.relative_to(repository_root.resolve())
    except ValueError as exc:
        raise InstallError("install destination escapes repository root") from exc
    bin_dir = destination / "bin"
    if bin_dir.is_symlink():
        raise InstallError("install bin directory must not be a symlink")
    bin_dir.mkdir(parents=True, exist_ok=True)
    resolved_bin_dir = bin_dir.resolve()
    try:
        resolved_bin_dir.relative_to(repository_root.resolve())
    except ValueError as exc:
        raise InstallError("install bin directory escapes repository root") from exc
    if resolved_bin_dir != bin_dir:
        raise InstallError("install bin directory resolves through a symlink")
    final_path = bin_dir / "cargo-nextest"
    fd, temporary_name = tempfile.mkstemp(prefix=".cargo-nextest-", dir=bin_dir)
    temporary = Path(temporary_name)
    try:
        with os.fdopen(fd, "wb") as stream:
            stream.write(payload)
            stream.flush()
            os.fsync(stream.fileno())
        temporary.chmod(0o755)
        version_output = _run_version(temporary, runner=runner)
        os.replace(temporary, final_path)
    finally:
        try:
            temporary.unlink()
        except FileNotFoundError:
            pass
    return {"path": str(final_path), "sha256": hashlib.sha256(payload).hexdigest(),
            "archive_sha256": digest, "archive": platform_key, "version": version_output}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--install-dir", default=".artifacts/ci/nextest-tools")
    args = parser.parse_args(argv)
    try:
        result = install_nextest(args.install_dir)
    except InstallError as exc:
        print(f"install-nextest: {exc}", file=sys.stderr)
        return 1
    print(" ".join(f"{key}={value}" for key, value in result.items()))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
