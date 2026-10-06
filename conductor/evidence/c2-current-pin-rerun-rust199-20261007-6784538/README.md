# C2 exact-pin verification — Rust 1.99.0

Parent source commit: `c2e35e064e83ccfd1d436da20adeeb6092e6a520`. Exact Kairos pin: `678453838f391f3dd869b79fd502077f63f3a067`. This rerun follows the C2.2 provider sampling test addition and verifies the integrated DES/ABM/calibration feature set at the current pin.

The three package suites completed **455 passed, 0 failed, 5 ignored across 43 test groups**. The test log includes the actual-Flow Q4 urgent interruption test, paired C2.1 behavior, and new C2.2 work-duration sampling tests. Strict all-target Clippy and workspace formatting passed.

This is exact-pin local verification only. It refreshes compatibility evidence and does not replace task-specific leaf joins or accept C2.4. Track03/Track22 owner decisions, portable restart, hosted checks, release, clinical acceptance, and the Conductor phase review remain open.

## Commands

All commands ran from `libs/kairos` using Rust/Cargo 1.99.0 only. Exact argv, exit statuses, hashes, source hashes, and generated context are recorded in [the packet](packet.json), [binding](binding.json), and [worker result](worker-result.json).

- [Test output](logs/test.log): 455 passed, 0 failed, 5 ignored across 43 groups.
- [Clippy output](logs/clippy.log): strict all-target check passed with `-D warnings`.
- [Format output](logs/fmt.log): workspace format check passed.

Coordinator disposition: exact-pin local verification accepted for this bounded rerun; C2.4 phase acceptance remains withheld.
