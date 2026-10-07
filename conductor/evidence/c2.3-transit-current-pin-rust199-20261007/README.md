# C2.3 actual-Flow transit fixture at current pin

The existing C20 runner passed its named actual provider/Flow transit fixture on Kairos `69878dcaf9d1eac7f1eba84ef19647642b77435d` using only Rust/Cargo 1.99.0. It exercised a positive three-edge route across pre-start and post-start pause/resume, stale start and arrival events, one arrival/acquire, and service completion at tick 39. The runner used a disposable archive, filtered to one exact fixture, and left Cargo.toml/Cargo.lock hashes unchanged.

- Parent commit: `fdcef188d3cb672a60260232afa09caab1109dc8`
- Kairos commit: `69878dcaf9d1eac7f1eba84ef19647642b77435d`
- Command (from `libs/kairos`): `python3 conformance/c20/run_transit_red.py --expect green`
- Test argv: `cargo test --locked -p kairo-ecs-calibration --lib --features flow,kairo-ecs-abm/test-support -- --exact paired_transit_c20::actual_nonzero_route_consumes_stale_start_and_arrival_across_pause_resume_once`
- Result: exit 0; one exact test passed; other package tests were filtered out.
- Fixture SHA-256: `a6169373466b43075e35507bd7d39a044943d88c223b6d39efcca2cc79607fc9`
- Cargo log SHA-256: `a529ab9df949e873c9b997432d0d2c9021bdcd32f84f9c36a9eb3fea5ae64f5e`

Raw runner result, command metadata, toolchain output and Cargo log are preserved beside this receipt. This closes the current-pin local test-evidence gap for this fixture only; it does not decide the optional Track21 route-receipt API or satisfy hosted, release, or clinical acceptance.
