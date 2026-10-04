# Linux x86_64 bootstrap

The tested Linux profile is x86_64 GNU/Linux with glibc. Alpine/musl and
unknown-libc hosts are rejected by `tools/bootstrap.py`. The recorded guest was
Ubuntu 24.04.4 LTS; the test does not qualify every glibc distribution or
bare-metal performance.

On Ubuntu/Debian, install the native linker and Rustup prerequisites:

```sh
sudo apt-get update
sudo apt-get install -y build-essential curl ca-certificates
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain 1.99.0
. "$HOME/.cargo/env"
```

From a clean checkout with the Kairos submodule initialized, run
`python3 tools/bootstrap.py --check` to verify the exact pinned compiler/Cargo
and host tuple, then run `python3 tools/bootstrap.py --test-core` for the locked
`kairo-ecs-core`, `kairo-ecs-state`, `kairo-ecs-rng`, `kairo-ecs-des`, and
`kairo-ecs-abm` test suites. Cargo may download locked crates into its cache;
compilation writes under `libs/kairos/target` unless `CARGO_TARGET_DIR` is set.

See [the D1.1 Linux execution receipt](../conductor/evidence/d1.1-linux-bootstrap-20260928.md)
for the exact host, toolchain, hashes, commands, test counts, and limitations.
