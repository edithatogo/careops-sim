# Native ED CLI execution API and consumer cutover

The existing CLI package provides a callable execution API. The binary entrypoint
uses it directly with the real stdout writer and defaults the Polars pool to one.
The separately versioned native benchmark reads actual typed files and reconstructs
the frozen summary only inside its test oracle. Historical drivers and receipts
remain immutable. Known consumer migration is a separate integration packet;
this source packet alone does not qualify every consumer or R14.

Run/batch require explicit fresh output roots. Atomically create a private0700
root on Unix, retain bounded raw input bytes and versioned immutable invocation
controls/checksum, use the original capture runner and native data exporter,
then publish a small artifacts-complete control last. It hashes the invocation,
actual batch control and data manifest. Disk artifact completeness is distinct
from successful stdout delivery; any write/flush failure remains exit7.
Raw bytes are bound to normalized parsed input and the native data manifest
without rewriting any native file or pretending the parsed library API had raw
provenance. JSON remains configuration/checkpoint/provenance control only.

Capability and argument preflight precedes input/output side effects. Preserve
CPU serial/independent-replication meaning and explicit unavailable-mode8,
malformed-argument2 behavior. The finite frontend envelope is32replications,
32workers,1million total export rows, existing10MiBinput and bounded identifiers.
The model, scheduler, seed allocation and scientific contracts stay unchanged.
The caller owns Polars initialization; the binary retains its entrypoint policy.

Known pre-dispatch cancellation returns before input/output. Capture-only
resume verifies original raw input, invocation checksum, config,
current model, exact same-root checkpoint, native codec and default capture
limits before the original recovery code runs. Workers may change explicitly;
initial requested workers remain immutable and actual workers are in the hashed
batch control. Completed roots and incomplete export are never retried in place.
Interrupted initialization without a valid checkpoint needs a new fresh root;
partial artifacts are preserved. This is not arbitrary export resumption.

Explicit CSV stdout uses the validated native patient projection and pinned
library CsvWriter through an actual8MiBcapped Write sink. It is not a String
bundle; partial stdout on failure is not success. Other formats return only the
small disk-provenance control. Export reads a qualified immutable native dataset,
converts through the shared publisher, and returns only bounded hashes/paths.
Compare reads both actual datasets, requires the exact five qualified metric names,
canonical units/estimands/denominators and matching model/runtime/data contracts,
and reports finite point differences with uncertainty explicitly not_estimated.
No full-table JSON path or legacy command alias is introduced.

The benchmark retains ten-second, 256MiB RSS and 8MiB stdout gates. Its unread
pipe probe reserves observation/TERM grace inside the same total deadline;
artifact publication may consume startup time. A failed old five-second readiness
probe and subsequent real ten-second slow-consumer timeouts are retained privately,
not converted into successful acceptance. Linux /dev/full remains mandatory;
macOS absence is explicitly unverified. Final receipts describe their actual scope.

CI, packaging, example consumers, support guides and legacy API removal remain
the next owned packet. Whole-layer/R14 completion and archive depend on those
consumer checks, policy gates and Conductor reconciliation by the integrator.

Local CLI checkpoint: final release26-case parity and14rehashed native-file
negatives pass, with slow/unread stdout and unchanged process budgets. Native
CLI13tests, source-guard20tests, benchmark4tests and CLI Clippy pass. The full
workspace rustfmt check exposes a pre-existing native_reader.rs wrapping drift
at the qualified reader checkpoint; it remains a final gate to repair under a
fresh scoped claim. This CLI checkpoint does not close R13/R14 or hosted gates.

Consumer integration checkpoint: reviewed CI/support changes retain exact five
metadata feature edges, install qualified native readers before the harness and
switch CI to the native gated benchmark. The22support,2D34 and27workflow tests
pass. Four migrated Python suites retain29tests; all executed against the exact
qualified release binary, with a focused E31 incompatible-unit fixture repair
reviewed separately after the initial failure. Six P5.2 scenarios each repeat
with equal typed data and stable logical provenance; native file bytes need not
match. The incompatible-unit fixture is explicitly retagged test transport, not
a simulated time conversion. The ED model still rejects non-tick input.

This combines two frozen reviewed packets (broader path count justified by the
coherent CI/consumer join); source blobs match their private hash handoffs.
Remaining live runner callers: D41 trust boundaries and ED E41/E42 analytical
validation/sensitivity mirrors. Runner removal, Rust legacy serializers/consumers,
calibration/dashboard/B7 boundaries, populated parent FFI and R14 remain open.


## Remaining Python consumer cutover — 3 November 2026

Coordinator chat `01a19b9f-69e7-7353-9b75-445d5653feb4` owns bounded
`Polars.R13.remaining-python-callers.20261103` in the existing registered Polars
worktree. It migrates D41 trust, E41 analytical and E42 sensitivity callers to the
actual native CLI and removes the example runner after no active callers remain.
Native scope is the local synthetic CLI only; next operation is source/evidence
review, then coordinator integration with the other consumer packets.

Independent analytical vectors remain unchanged: zero contention, exact D/D/1
schedule, synthetic right-skew/triage ordering, state conservation, capacity and
demand sensitivity, warm-up windows and existing timing/throughput thresholds.
Tests consume typed patient limbs and replication/metric tables without an alpha
JSON reconstruction adapter. Per-run uncertainty remains separate from native
comparison's unestimated paired difference. Software verification does not confer
clinical/site validation.

The former forged Macro/Micro JSON comparison cannot certify a domain producer.
Its real fidelity/transit/preemption/conservation oracle remains the C62 Rust
holdout comparison; the Python native boundary rejects forged alpha documents
and checks actual neutral finite native comparisons. The actual parent FFI
lifetime/release lane remains R14. Python checks real Arrow capsules cannot be
pickled, real IPC crosses a child process, and production readers reject actual
truncated, invalid-framing, unsupported-schema/type and oversized IPC parts.
Python network-client mocking observes the harness process only; it is not a
claim of OS-enforced native-child network confinement.
