# Native C2 sealed save/restore demo — 9 October 2026

Root independently verified exact Kairos `adf5fed6a86670b5b07be5e7522f174f7823ef0c` using only Rust/Cargo1.99.0 and a fully bound Rustup toolchain/cache. The new doc-hidden experimental API shares its production implementation with tests. The native example saves a coherent paused checkpoint at tick1 and restores in a separate process, replaying the complete closed prefix before validating actual carrier receipt/frontier/IDs/RNG and running the suffix.

- Save and both separate restore processes exited0; both canonical traces are byte-identical.
- Route: five useful travel ticks, two paused ticks; arrival7. Service: twenty useful ticks; one linked request completes27 and releases its resource.
- Corrupted input and existing destination reject (exit1), missing/unknown arguments reject (exit2), and existing sentinel bytes remain unchanged.
- Feature-off example check passes; full native suite **468 passed,0 failed,5 ignored across43 groups**. Current-pin paired-v2 and original transit fixtures pass; strict all-target Clippy and workspace formatting pass.

Commands, exact source/toolchain/build/checkpoint hashes, raw logs and retained actual checkpoint are here. Use the native example README in `libs/kairos/crates/kairo-ecs-calibration/examples/README.md` for stable1.99 build-once save/restore commands. Save requires an existing nonsymlink parent and never overwrites. Restore requires the same built executable.

This is one sealed synthetic recipe with conservative compatibility and unsupported-state rejection. It is not general arbitrary Flow checkpointing, C5 worker/candidate recovery, an ED model, clinical validation or release/hosted/C2.4 acceptance. The original paired-v1 oracle and its failure history are retained separately; v2 explicitly corrects the terminal-time observation while preserving parity/RNG/resource requirements.

The project-local learning CLI/skill/hooks are installed; append-only lessons, reviewed supersession, source freshness and observed feedback are tested. Fourteen focused learning tests and the repaired512-test parent suite passed in their separate receipts. Learning records do not grant permissions or prove model/clinical quality.
