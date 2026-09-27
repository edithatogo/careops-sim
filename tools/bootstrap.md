# Rust toolchain preflight

Run `python3 tools/bootstrap.py --check` before native Rust work. The command
reads `bootstrap-profile.json`, accepts macOS ARM64 and Linux x86_64 GNU, and
checks the exact Rust 1.98.1 compiler and Cargo through `rustup run`. It only
reads tool versions and host metadata. It does not install, update, or change
system tools.

The profile records exact URLs and checksums for the official Rust 1.98.1
channel manifest
(`a7c8774a5fd8441c997d94c029776cbc5eb111e9d72ab5d256fa69866644347e`) and
platform installer archives. The archive names and checksums identify the
upstream distribution; this preflight does not download or revalidate them.

If rustup is missing, install it explicitly from [rustup.rs](https://rustup.rs/).
If the pinned toolchain is missing, run `rustup toolchain install 1.98.1` and
rerun the check. On an unsupported host, use a supported host or propose a
reviewed profile update. A successful check proves local version/host metadata
only; it does not qualify a clean machine, Linux execution, dependency
compatibility, or release readiness.
