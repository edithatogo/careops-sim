# E2 checkpoint V2 preparation and consumer join

Status: coordinator preparation, internally reviewed with GPT-6.1-sol.
Runtime source `382b333fdc18bcbdf8bbf231fc633a79ab6ec86a` is qualified at
Kairos `9cd9361aa68fc2fd38c4918d1a6fb2b26c3bc9ea` (reviewed V2 ADR included).
Parent runtime evidence is accepted in `c586d06`; native V2 implementation was
dispatched in `84e6bed` and remains unaccepted. The parent pin remains
`6d733f489225da4bf6ceec2673b02369c7918bbe`. Rust/Cargo 1.99.0 only.
This preparation does not authorize future workers. Bind their packets to the
accepted native source and reviewed `ADR-e2-flow-checkpoint-v2.md` at dispatch.

## Required delivery

Persist and restore enabled flags, every pending gate command and its real
scheduler identity, and the started frontier without losing continuation state.
Complete this join before parent single-runtime E2 binding. Runtime-only gate
checks, Python specifications and a prepared design do not satisfy that join.

## Recommended version boundary

Keep every V1 DTO, method, wire magic, tag and golden byte unchanged. V1 capture
continues to reject disabled resources or pending gate state. No new FlowError
variant or V1 nested dispatch/error tag is needed.

Explicit APIs frozen by the reviewed Kairos owner ADR:

- FlowCheckpointV2 and FlowCommandV2.
- FlowRuntime::capture_checkpoint_v2, restore_checkpoint_v2 and
  restore_checkpoint_v2_with_rebind.
- FlowCheckpointV2::encode_wire_v2 and decode_wire_v2.
- A lossless FlowCheckpointImage enum for V1/V2 decoding and a borrowed
  version-aware view over common fields and complete command variants.

V2 contains every common V1 field without loss. Its command enum includes all
legacy commands and GrantEnabled(ResourceId, bool). Canonical flag rows are sorted by ResourceId, unique and
cover exactly the live resources; V2 has no implicit missing flags. Derive the pending index from full command rows and their live scheduler entries;
do not serialize redundant boundary rows. Command row keys supply EventId; the
scheduler supplies tick and priority; the command supplies resource and enabled.
Capture independently derives the index and requires equality with the runtime
cache. Restore validates the join before rebuilding that private cache.
The optional started frontier must agree with
persisted scheduler dispatch state. The exact runtime representation remains
private; the ADR must freeze DTO fields and validation before implementation.

Use distinct eight-byte magic `KFLOWV2\0` and image version 2. Legacy V2 command
tags remain 0–11 and the new gate uses tag 12 only in V2. V1 uses its existing
magic, tags and nested FlowDispatch encoding unchanged. Route decoding by inner
magic; reject unknown versions/tags, malformed booleans and trailing bytes.
Freeze the V2 wire schema discriminator separately from image version 2.
An unchanged outer C2 envelope does not mean an old reader can read inner V2.

Do not strip gates or scheduler events to construct a fake V1 image, restore
that image and reinsert them. Shared validation may use a lossless common view,
but command/event joins and all new state must survive validation intact.

Existing context/template codec schemas and FlowCheckpointRebindV1 may remain
if their identity/rebinding semantics are unchanged; that rebind is not a stripped
Flow image. Reuse FlowBuiltInStoresV1 only if accepted gate state remains separate
from built-in component stores; otherwise define a V2 store type. Confirm these
choices against the frozen runtime before the owner ADR.

## Bounded, atomic restore

Reuse existing limits without silently increasing capacities: flag rows count
against max_resources; command/derived-index counts against max_commands; combined pending
releases, despawns and gates against max_pending_operations. Include new rows
and payloads in aggregate record, byte and payload limits with checked arithmetic.
Preflight before allocation or caller codec execution.

Before returning a restored runtime, validate:

1. Exact, unique live resource/flag coverage with generational identity checks.
2. Unique resource/tick boundaries and a one-to-one join between gate commands
   and live scheduled EventIds, including tick, priority and payload. Derive the
   index only from this validated join.
3. No pending Remove for a resource with gate boundaries.
4. Frontier consistency with scheduler dispatch counters and now. Initial zero
   differs from executed zero; semantic rejection advances the frontier, while
   precommit budget failure does not.
5. Existing context, callback, work, ABM transit and coherent-cut invariants.

A predeclared pending boundary at the current started tick T is valid after a
completion-first dispatch at T and before the gate token at T runs. Restore must
not apply the fresh-admission frontier rejection to persisted commands.
Decode produces detached data; restore publishes no runtime/rebind on failure.
The source runtime and caller-owned inputs remain unchanged.

## Consumer impact and ownership

Track03 owns reusable DES runtime/checkpoint semantics; calibration/Track22 owns
consumer integration. Current source has direct FlowCheckpointV1 dependencies in
c2_portable_checkpoint.rs and flow_bridge/checkpoint_wire.rs; the latter's
validate_coherent_cut reads scheduler, registration, context and command data.
Review flow_bridge.rs, shadow_native.rs and c5_native.rs as part of the join.
Replace V1-only assumptions with lossless version-aware access, preserving ABM
transit coherent-cut validation and the same authoritative Flow clock.

## Scheduling and acceptance

1. Review and freeze current runtime worker source, including fail-closed V1.
2. In a separate Kairos contract packet, freeze V2 APIs, DTO, wire and restore
   invariants with internal owner review. Contract preparation may run now.
3. Implement V2 after runtime ownership of shared source paths is released.
   Runtime and checkpoint writers must not overlap flow.rs/checkpoint.rs.
4. Bind calibration consumer packets after the V2 interface freezes. Read-only
   consumer inventory and oracle planning may proceed alongside engine coding.
5. Run combined DES/core/ABM/calibration qualification on the integrated Kairos
   development head; preserve actual command/source/toolchain/log evidence.
6. Update the parent pin under a separate claim, then implement single-Flow E2
   behavior and bind all 32 staff, 24 interruption and 32 space specifications,
   retaining E1 parity. E2.3/E2.4 and later delivery remain open.

Mandatory checkpoint oracles include disabled state; enabled state with a future
toggle; capture before/after a same-tick gate and after a precommit budget halt;
restored versus uninterrupted full histories; malformed joins/resources/versions/
tags/limits; and unchanged legacy V1 bytes/readback. Native package checks alone
do not establish hosted/platform, empirical, clinical or release acceptance.

## Next disjoint consumer packet

After native V2 acceptance, wire implementation can run alongside a calibration
preview packet owning only `crates/kairo-ecs-calibration/src/shadow_native.rs`
and `crates/kairo-ecs-calibration/src/c5_native.rs`, plus its result and logs.
Put focused unit tests in those files. Freeze exact native API/source hashes
before dispatch; neither worker owns the native checkpoint implementation.
Portable checkpoint and coherent-cut byte consumers wait for accepted V2 wire.

Use `capture_checkpoint_image(...).as_ref().scheduler()` in the actual preview
paths, propagating errors. Shadow's adapter accepts a gate-configured model.
C5 free_run currently creates an ungated runtime: changing its capture call does
not add gate configuration to C5 task inputs. Extract a private preview helper
used by the real C5 loop and qualify that same helper with gated runtime state.
Do not claim C5 task/config gate admission from this seam qualification.

Required witnesses:

- Shadow: disable at 0, waiting work at 0 with duration 5, enable at 3;
  previews and actual execution reach grant 3 and completion 8. At horizon 6,
  work remains active and unfinished. Stop before event 8; do not force the
  authoritative event clock to 6 or invent a progress sample at that tick.
- C5 helper: pending disable 2 / enable 7 with active completion 5. V1 capture
  rejects pending state; the helper previews 2 without mutating time, dispatch
  count, claims or RNG. After step 2 it previews 5; active work drains at 5.
- Never-enabled queued work has no next event and no invented completion.
- Capture failure from malformed or over-budget state propagates without fallback.
- Preserve existing ungated C5 studies and shadow budget/hook behavior.

Three future gates use the pinned Rust 1.99 binaries: `cargo test --locked -p
kairo-ecs-calibration --features flow`; `cargo clippy --locked -p
kairo-ecs-calibration --features flow --all-targets -- -D warnings`; and
`rustfmt --edition 2021 --check --config skip_children=true` on the two owned
files. The flow feature is required to compile these consumers. The recorder
must require nonzero named shadow and C5 preview tests and protect all other
source. These are planned commands, not executed results.
