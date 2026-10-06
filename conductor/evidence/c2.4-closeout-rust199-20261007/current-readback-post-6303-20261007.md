# C2.4 current Conductor readback — Kairos 6303a255

**Disposition: phase acceptance withheld.** This readback reconciles the current local source and evidence. It does not accept C2.4, close the C2 phase, or substitute for owner/domain/external gates.

## Current source and local evidence

- Parent source baseline: `3e2cf21bcde297f27888f02eb54bdf058e07e013`; current Kairos pin: `6303a2552257f1fcbd2aaa002d38352a2a467592`.
- The Q4 urgent-interruption runtime leaf is locally accepted at its tested Kairos pin `6053d8dd2f8057c423decc929c90a8bcd10837f9`. Its tested `flow_bridge.rs` SHA-256 is `e2c3b5eb5381b85a083f879a017d7be5cd573cdabd23b82eda486bab27e41959`, unchanged at the current pin. The only source delta from `6053d8d` to `6303a25` is `#[doc(hidden)]` on `TransitContext::schedule_transit_retry` in `crates/kairo-ecs-abm/src/flow_adapter.rs`. The exact-parent DES/ABM/calibration suite has now passed at `6303a255` using Rust 1.99.0 only: 447 passed, 0 failed, 5 ignored across 43 result groups; strict all-target Clippy and workspace formatting also passed. Commands and log hashes are recorded in the [current-pin verification receipt](../c2.4-current-pin-rerun-6303a255-20261007/acceptance-rust199.json). This updates local verification only; the owner, hosted, release, clinical, and phase gates remain open.
- The manual same-case elapsed-time readback is [recorded here](../c2.4-runtime-manual-readback-20261007/README.md). The base ledger is 25 ticks per arm: Macro `5 queue + 20 service`; Micro `2 transit + 3 queue + 20 service`. The separate Q4 interruption fixture is 27 ticks per arm: Macro `5 queue + 20 service + 2 pause`; Micro `2 transit + 3 queue + 20 service + 2 pause`. Each arm is one elapsed window; paired totals and separate fixture runs are not additive. The readback is a manual source inspection with no new test run.
- The [Track03 packet](../c2.4-track03-owner-review-rust199-20261007/review-packet.md) and [Track22 contract packet](../c2-track22-checkpoint-owner-review-20261007/review-packet.md) are prepared but unsent. No owner disposition is recorded.

## Remaining gates

Track03 owner disposition; Track22 portable checkpoint/restart contract and new-process restoration; exact-head hosted checks; release acceptance; clinical acceptance; and the separate C2.4 Conductor phase review remain open. The Track22 smoke and CLI scaffolds do not implement portable restoration. Clinical policy/acceptance cannot be inferred from synthetic runtime fixtures.

The C2.4 phase checkbox remains unchecked. No C2.4 phase acceptance or full C2 acceptance is claimed. The older closeout README and JSON are retained as historical snapshots; use this readback and `conductor/current-state.json` for the current source and gate status.
