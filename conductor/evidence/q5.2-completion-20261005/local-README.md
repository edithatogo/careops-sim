# Q5.2 local evidence archive

This directory is a local preservation bundle of Q5.2 raw evidence. It is not a coordinator acceptance, a pin-advance authorization, or a queue-phase closeout. The archive retains both superseded and current failures, and separates local measurements from hosted CI evidence.

## Source identity

- Measured Kairos runtime tree: `e06b4aa3cabcd7a6c84cc06193f72d95b1ed763f`, 39 runtime cases × 5 repeats, all 195 child processes successful, zero timeouts.
- Published Q5.2/C-01 source: `0944b8198e8208f7b2a016ad9774fbb91a895d5a`; Q5.2 inputs remain equal to measured `e06b4aa`. The accepted C-01 source and evidence are disjoint additions.
- Existing native-owner Actions run 37214343886 completed 11/11 jobs on merge producer `64a4544851946fd70bba00eb8d8322532d67f6f8`. The producer tree and PR head tree are both `8dd75d09dcfaacc855e7f457fdef8a20530bc585`. Keep the distinct commit IDs visible: this proves source-tree equivalence but does not replace the planned direct checkout of the PR head with an explicit expected-head assertion.
- The shared published checkout now observes contract-only HEAD `206aef2202d9f7e7e0046033767df4511f3bb058`; artifact receipts remain bound to their recorded producers and are not relabelled to this later HEAD.

## Canonical regression evidence

The original comparison remains preserved with four blocking threshold failures: `create_1m_entities`, `pop_1m_events`, `schedule_1m_events`, `schedule_cancel_1m_mixed`. A separate controlled serial baseline/current pair is also preserved. That pair reports two blocking failures: `component_insert_1m` +91.0063% against the 3% ECS threshold and `pop_1m_events` +10.6969% against the 5% scheduler threshold. The other four rows pass, including advisory hybrid. Thresholds are unchanged. The matched pair is a real result, not a passing gate; Q5.2 performance acceptance remains unresolved.

## Memory and measurement scope

Runtime peak RSS is per-child `wait4` max RSS normalized to bytes; it includes process setup and output readback, not per-operation or per-claim allocations. Dispatch timing includes Flow step, lifecycle classification/counting, and world staging/commit; correctness readback is outside the timed interval. Five-repeat nearest-rank p95 is the maximum sample, not a tail estimate. Legacy comparison is a matched `Resource` FIFO primitive only, not full Flow. Selector/microkernel measurements do not establish whole-runtime speedup or active-victim scan cost unless that production path was actually invoked and separately identified. No general speedup claim is made.

## Contents and integrity

`inventory.json` maps relative archive paths to byte counts and SHA-256 hashes. It excludes symlinks, targets, build products, executables, private lease/session/token/store files. The deterministic `q52-evidence.tar.gz` is built from sorted members with normalized timestamps, ownership, and modes. `archive-build-receipt.json` records the actual inputs, statuses, source identities, and package hashes. No dependency, Cargo manifest, Cargo lock, canonical trace, or threshold changes are included by this evidence task.
