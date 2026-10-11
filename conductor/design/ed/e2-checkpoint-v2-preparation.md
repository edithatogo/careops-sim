# E2 checkpoint V2 preparation and consumer join

Status: coordinator preparation, reviewed in principle with GPT-6.1-sol.
This is not implementation authority or a claim of runtime/checkpoint acceptance.
Source baseline: Kairos `63cd1cd5a53c59e255fce4db09bea2e219254c90`, whose
reviewed resource-grant ADR extends parent pin
`6d733f489225da4bf6ceec2673b02369c7918bbe`. Rust/Cargo 1.99.0 only.
The active runtime worker owns flow.rs and flow/checkpoint.rs. Freeze and review
its actual types before a separate Kairos owner ADR and executable packet.

## Required delivery

Persist and restore enabled flags, every pending gate command and its real
scheduler identity, and the started frontier without losing continuation state.
Complete this join before parent single-runtime E2 binding. Runtime-only gate
checks, Python specifications and a prepared design do not satisfy that join.

## Recommended version boundary

Keep every V1 DTO, method, wire magic, tag and golden byte unchanged. V1 capture
continues to reject disabled resources or pending gate state. No new FlowError
variant or V1 nested dispatch/error tag is needed.

Proposed explicit APIs (names require the next owner ADR):

- FlowCheckpointV2 and FlowCommandV2.
- FlowRuntime::capture_checkpoint_v2, restore_checkpoint_v2 and
  restore_checkpoint_v2_with_rebind.
- FlowCheckpointV2::encode_wire_v2 and decode_wire_v2.
- A lossless FlowCheckpointImage enum for V1/V2 decoding and a borrowed
  version-aware view over common fields and complete command variants.

V2 contains every common V1 field without loss. Its command enum includes all
legacy commands and ResourceGrantEnabled(ResourceId, bool). Canonical flag rows are sorted by ResourceId, unique and
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
