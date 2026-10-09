# Native Flow image review and correction packet

Reviewed immutable worker commit: `5c41632dae5d761a9176a44434f9a86400e9117e`.
Checkpoint source SHA-256:
`c9a393e045ae736c574a91d09d0212b62622447143b0a1b45f56e502aae3e20e`.
Two separate Luna source reviewers found the following blockers. They did not
execute tests or modify source. Root checked the restore and release-dispatch
source before issuing the correction packet; acceptance remains pending.

1. Reject incomplete restart-store manifests while preserving registered empty
   stores. Every registration referencing a restart codec needs its store.
2. Complete allocation-free aggregate count, row, sparse-slot and payload checks
   before allocating duplicate scratch or indexes. Bound actor-domain count.
3. Reject active allocation counts exceeding the resource's total capacity.
4. Reconcile timed request states with actual WorkProgress, segment and completion
   state using production lifecycle rules. Preserve legal queued restart state
   and obsolete completion events.
5. Link pending release/despawn reservations to live command inventory. A release
   may become stale after preemption; production removes its reservation only
   when that command dispatches, so requiring its lease to remain active would
   reject a legal image.
6. Check lifetime counter caps, scheduler count consistency, admission/lease
   bounds and budget/halt coherence against attainable production state.
7. Replace repeated per-work linear searches with bounded borrowed indexes after
   aggregate preflight. A many-work correctness case must avoid timing assertions.

Writer: component_store_codec, same four native Flow source paths, fresh claim at
5c41632, isolated output `.artifacts/c2-flow-corrections`. One commit then release.
Mutation cases must reject before calling owner decoders where structural checks
can do so. Include valid stale release and empty restart stores as positive cases.

Required Rust1.99-only verification: locked tests for core, state, DES, ABM and
calibration with `kairo-ecs-calibration/flow,kairo-ecs-abm/test-support`; strict
Clippy for those packages/all targets with the same features; workspace format.
Record exact commands, toolchain, source identities, exit codes and raw logs.
Root independently reviews and integrates after correction. No whole portable
runtime, fresh-process artifact, C2.4, clinical or release acceptance follows from
this packet alone.
