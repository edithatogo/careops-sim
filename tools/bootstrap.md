# Rust toolchain preflight

Run `python3 tools/bootstrap.py --check` before native Rust work. The command
reads `bootstrap-profile.json`, accepts macOS ARM64 and Linux x86_64 GNU, and
checks the exact Rust 1.98.1 compiler and Cargo through `rustup run`. It only
reads tool versions and host metadata. It does not install, update, or change
system tools.

Before running the native suite on a clean machine, install its OS linker and
Rustup prerequisites. On macOS ARM, install Apple's Command Line Tools and
Python 3. On Ubuntu/Debian x86_64, install `build-essential`, `curl`, and
`ca-certificates`, then install the minimal pinned Rust toolchain and add Cargo
to the current shell:

```sh
sudo apt-get update
sudo apt-get install -y build-essential curl ca-certificates
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain 1.98.1
. "$HOME/.cargo/env"
```

The supported Linux profile is x86_64 GNU/Linux (glibc); musl and unknown
libc environments are rejected. From a clean checkout with the Kairos
submodule initialized, run `python3 tools/bootstrap.py --test-core` to perform
the pinned-toolchain check and execute `rustup run 1.98.1 cargo test --locked` for
`kairo-ecs-core`, `kairo-ecs-state`, `kairo-ecs-rng`, `kairo-ecs-des`, and
`kairo-ecs-abm` in `libs/kairos`. It does not install or update toolchains. Cargo
may download the locked crates into its cache; compilation writes build artifacts
under `libs/kairos/target`. Initialize a missing submodule from the repository
root with `git submodule update --init --recursive`, then rerun the command.

The profile records exact URLs and checksums for the official Rust 1.98.1
channel manifest
(`a7c8774a5fd8441c997d94c029776cbc5eb111e9d72ab5d256fa69866644347e`) and
platform installer archives. The archive names and checksums identify the
upstream distribution; this preflight does not download or revalidate them.

If rustup is missing, install it explicitly from [rustup.rs](https://rustup.rs/).
If the pinned toolchain is missing, run `rustup toolchain install 1.98.1` and
rerun the check. On an unsupported host, use a supported host or propose a
reviewed profile update. A successful check proves local version/host metadata
only; it does not qualify a clean-machine setup, consumer dependency
compatibility, or release readiness.
Likewise, a successful `--test-core` run proves only that the selected core tests
passed on the current host and checkout. The recorded macOS ARM run does not
establish fresh-machine setup, consumer compatibility, or release readiness. The
recorded Linux run uses a fresh Ubuntu 24.04 x86_64 guest under QEMU emulation on
Apple Silicon; it verifies the Linux kernel/ABI and glibc path, not bare-metal
x86_64 hardware performance or release readiness.
