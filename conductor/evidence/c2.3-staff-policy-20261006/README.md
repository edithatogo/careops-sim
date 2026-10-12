# C2.3 staff policy instance — local evidence

Kairos commit `b2e42e7c2a4ed157218e9de15807410d513e65ae` adds a private deterministic selector in `kairo-ecs-calibration`. It ranks requests by urgency descending, FIFO sequence ascending, then request ID; filters staff by availability, exact assigned zone, and required-skill inclusion; and breaks staff ties by stable actor ID. Work, walk, and wait are distinct values; waiting requests do not receive a staff assignment. Duplicate IDs fail closed.

The source and parent pin are integrated in this local checkout. The focused Rust 1.99 test run passed 6/6; strict all-target Clippy and workspace formatting passed. Exact commands, source hashes, and raw logs are in `acceptance.json`.

This accepts only the pure staff-selection instance. It does not schedule Flow transit events, reserve a staff resource, or implement cleaning/reservation lifecycle. C2.3 dispatch integration, C2.2 provider-to-Flow admission, C2.1 paired runtime oracles, hosted exact-head checks, stable API/release, and operational acceptance remain open. No C2 plan checkbox was changed.


## C2.3.dispatch leaf acceptance at current Rust 1.99 pin

A fresh read-only acceptance audit compared this instance to the exact recipe oracle and verified that the selector source at Kairos `7672a67988134438228014e7b3e0f56855cc65e8` is unchanged from accepted source commit `b2e42e7c2a4ed157218e9de15807410d513e65ae`. The leaf has one required selector instance: eligible staff only, deterministic urgency/FIFO ordering and actor-ID tie break, exact zone/skill/availability filtering, and distinct work/walk/wait with wait unassigned. That satisfies the leaf oracle. The older instance receipt's `C2_3_dispatch_leaf_complete: false` describes broader Flow/resource/cleaning integration, which remains outside this selector leaf and open at the C2.3 parent level.

The coordinator's exact-pin Rust 1.99 full package test run in [the transit-bridge acceptance](../c2.3-transit-bridge-rust199-20261006/README.md) includes the unchanged staff selector. Machine-readable leaf receipt: `leaf-acceptance-rust199.json`. Track03 owner review, Flow transit dispatch integration, staff reservation/cleaning lifecycle, C2.3 parent completion, and hosted/release gates remain open.
