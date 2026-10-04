# Rust toolchain preflight

Run `python3 tools/bootstrap.py --check` before native Rust work. The command
reads `bootstrap-profile.json`, accepts macOS ARM64 and Linux x86_64 GNU, and
checks the exact Rust 1.99.0 compiler and Cargo through `rustup run`. It only
reads tool versions and host metadata. It does not install, update, or change
system tools.

From a clean checkout with the Kairos submodule initialized, run
`python3 tools/bootstrap.py --test-core` to perform that same pinned-toolchain
check and then execute `rustup run 1.99.0 cargo test --locked` for
`kairo-ecs-core`, `kairo-ecs-state`, `kairo-ecs-rng`, `kairo-ecs-des`, and
`kairo-ecs-abm` in `libs/kairos`. It does not install or update toolchains. Cargo
may download the locked crates into its cache; compilation writes build artifacts
under `libs/kairos/target`. Initialize a missing submodule from the repository
root with `git submodule update --init --recursive`, then rerun the command.

The profile records exact URLs and checksums for the official Rust 1.99.0
channel manifest
(`ce6dddc886364f8d786514771212cebe9b731ba82d6b859951c6b0ccc516b6a2`) and
platform installer archives. The archive names and checksums identify the
upstream distribution; this preflight does not download or revalidate them.

If rustup is missing, install it explicitly from [rustup.rs](https://rustup.rs/).
If the pinned toolchain is missing, run `rustup toolchain install 1.99.0` and
rerun the check. On an unsupported host, use a supported host or propose a
reviewed profile update. A successful check proves local version/host metadata
only; it does not qualify a clean machine, Linux execution, dependency
compatibility, or release readiness.
Likewise, a successful `--test-core` run proves only that the selected core tests
passed on the current host and checkout. The recorded macOS ARM run does not
establish fresh-machine setup, Linux execution, consumer compatibility, or release
readiness; those remain unverified until separately executed and reviewed.
