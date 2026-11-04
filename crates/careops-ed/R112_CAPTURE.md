# R11.2 native capture candidate

Retained implementation candidate integrated from PR191 exact4b0a264 onto the
combined ED/core/birthing source. No phase acceptance is inferred. Native data
uses the already qualified Git4134/Polars964 chain while simulation remains on
the unchanged path074 runtime. Current workspace advisory/bans/license/source
checks pass without new exemptions; the older blockers below remain history.

## Data path

With the default `native-data` feature, `run_batch_with_workers` invokes each
replication once. The original patient results pass through synchronous bounded
capture, then Kairos's Polars Parquet/Zstd writer. Export validates the completed
capture manifest, checkpoint, input/configuration binding, part hashes, patient
coverage and summaries before reading those rows. It never invokes the simulator.
The schema identifier is `careops.ed.patient-result.v2`; `.part` files are genuine
Parquet, not JSON or an internal lookalike. Every row carries schema, capture schema,
run ID, effective parsed-input hash, time unit, replication index, seed and sequence.
Patient identifiers remain strings. Tick columns use UInt64 high/low limbs;
nullable pairs must be null together. The current ED model is u64: high limbs are
zero, and nonzero high limbs reject rather than truncate. Required fields and
status values reject null/unknown values in the decoder.

The effective-input hash covers canonical Serde serialization of ScenarioConfig
with effective horizon and batch base seed. `parsed_input_sha256` separately
covers the supplied ScenarioConfig before those overrides. Neither attests the
original input-file bytes. Per-replication seeds bind separately in control state
and each part. Schema/library identity is in the codec identifier. Complete source
identity and toolchain receipts remain in the bounded work packet; R13 owns the
complete externally persisted provenance/output contract.

The final batch control document is version 2 and refers to capture storage.
Persistent runs use `<output-dir>/capture/`; their checkpoint is
`<output-dir>/capture/checkpoint.json`. Existing full-table JSON export and spatial
projection remain transitional R13 surfaces. Their rows now come from original
capture, and horizon/seed provenance uses the effective run. This does not turn
synthetic spatial projections into observed movement or add scheduler event logs.

## Ownership and controls

An output directory must be exclusively owned. Parts and final documents use
same-directory staging, flush, file sync, no-clobber publication, and directory
sync. Completed data is not overwritten. Checkpoints commit completed-replication
part references and sequence boundaries before the capture manifest. Pending
orphan parts reject; no implicit deletion or replay is attempted. Process-crash
and platform-specific durability qualification remains R11.3, not established by
injected write failures or successful local fsync.

Without an output directory, a document holds shared temporary-directory ownership.
Clones keep it alive; dropping the last owner removes it. Serialization preserves
only the locator, not temporary ownership. A deserialized locator fails after its
owner drops. Use a persistent output directory for durable serialized documents.
Summary-only legacy checkpoints can still yield summary results, but export
explicitly rejects them because they lack original captured rows.

Cancellation and progress use the existing generic runner. A cancelled run can
resume completed replications from the capture checkpoint; a completed destination
is rejected before execution. Distributed partition callers use index-derived
seeds and finalize only after complete, unique coverage and actual part validation.
This adds no distributed scheduler or transport runtime.

## Bounds and thread budget

Defaults: 256 rows, 4 MiB encoded bytes per part, 4096 bytes per patient ID.
Actual codec framing controls splitting; oversized single rows fail. Writers
acknowledge synchronously, with no hidden capture queue. Parquet metadata row and
uncompressed-size limits are checked before decoding; string and typed-schema
checks follow. Library scratch/decompression allocations, original simulation
model/summary memory, checkpoint metadata and the legacy export bundle are not a
global RSS bound. R14 retains measured memory/performance qualification.

The native CLI sets `POLARS_MAX_THREADS=1` if absent, once at single-threaded
process entry before workers or Polars initialization. A supplied positive integer
is retained; invalid values fail. Library callers must configure the environment
before initializing Polars. No concurrent library call mutates the environment,
and Kairos's bookkeeping-only thread-budget accessor is not treated as pool setup.
Minimal core remains free of Polars. ED summary-only builds can disable default
features; native export then fails explicitly.

## Historical candidate evidence and former dependency blocker

The original invocation tests are green for 1/2/4 workers. Production tests cover
real Parquet/Zstd types, original-row/override parity, actual cancellation/resume,
clone/drop/serialization, corrupt and missing storage, bounds, write errors,
partition-finalization negatives, reordered partitions and occupied destinations.
The existing boundary fault/backpressure tests remain active. No golden is changed.

`cargo-deny --workspace --locked check` fails with:

- Zstd 0.13.3: RUSTSEC-2026-0391; fixed version requires >=0.14.3.
- zstd-safe 7.3.0: RUSTSEC-2026-0398 and RUSTSEC-2026-0403; fixed versions require >=8.2.0.
- Existing parent duplicate-version bans and license allowances reject parts of
  the newly admitted native dependency graph (including LLVM exception, Zlib and
  BSL-1.0 licenses).

The pinned Polars 0.45.1 chain constrains older Zstd versions. Its reader uses
streaming Zstd decoding with legacy support present. A passing writer roundtrip
is not a security waiver. Resolution requires an accepted compatible upstream
Kairos/Polars codec dependency update plus parent policy review; do not patch or
suppress this locally to claim acceptance. Source rollback is the parent commit
rollback with a fresh output directory; never reinterpret old output silently.

## Integration qualification — 3 November 2026

The exact prior PR191 source passed hosted gates at4b0a264. This integration
retains current workspace members, E9 CPU/backend guards, existing B7 dependency
decisions and runtime074. ED native data consumes exact Git4134/Polars964,
matching the accepted B7 chain; no Kairos file/pin or other owner checkout changes.
Current locked policy and pin contracts pass. Integrated targets:43EDlibrary,
5E9,10capture-boundary and11production-capture tests passed.

R11.3 remains unaccepted: initial/control checkpoint sync, pending orphan attempt
recovery, capture-manifest/final-document crash window, compiled-model binding and
full checkpoint-envelope validation need real fault/process controls. R12–R14
remain separate processing/output/performance/removal phases.

## R11.3 durability candidate — 3 November 2026

Capture control is now schema version2, `careops.ed.capture-boundary.v2`. Old
unacceptedv1capture directories remain preserved and reject rather than being
reinterpreted. The ED coordinator persists an initial checkpoint before workers,
then serializes each completed-replication acknowledgment through file sync, atomic
replacement and directory sync. It reuses the generic independent-seed runner
without its best-effort checkpoint writer; generic core/engine code is unchanged.

Recovery validates the full control envelope, represented index coverage and every
acknowledged part before mutation. Known pending replication parts are moved to
exclusive `recovery/attempt-*` directories on explicit resume; bytes are preserved.
Unknown namespaces and corrupt/missing acknowledged data reject. A complete valid
capture manifest can finalize a missing batch document without republishing parts
or rerunning completed work. Repeated capture recovery is idempotent; an already
published batch document remains protected from overwrite.

The compiled model contract hashes owned core/ED/CLI implementation and manifests,
actual consumed path-runtime source, root lock, build script, actual rustc/target/
profile/flags/features. It is stored in checkpoint metadata, replication records
and typed Parquet parts. Execution recovery requires the current build identity;
read-only analysis validates internally consistent recorded producer identity.
The original actual-compiler metadata is retained. This is an implementation
binding, not a binary signature, clinical validation or cross-platform restart
claim. Source/pin rollback uses its own compatible output, never mixed artifacts.

New native controls cover24actual child-process kills across8publication windows
and1/2/4recovery workers; exact serial row/count/seed coverage, preserved acknowledged
bytes and pending-attempt bytes, call counts and idempotence. File and directory
sync failures are separately injected at the real control publication boundary.
A controlled omission of model binding fails the intended refusal assertion; exact
source is restored before green verification. Seven recovery targets (including
the child fixture), retained21capture targets and5E9controls pass locally, with
Clippy warnings denied. Independent committed review, parser-fuzz build-dependency
lock qualification and full combined R11phase/hosted gates remain pending.

These results establish local-filesystem process-crash/completed-replication
recovery only. Codec/model/control metadata memory and actual platform/power-loss
limitations remain explicit; there is no within-replication simulation checkpoint.
