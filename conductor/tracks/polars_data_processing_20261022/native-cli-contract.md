# Native ED CLI execution API — staged consumer cutover

The existing CLI package gains a callable execution API. The binary entrypoint
stays unchanged until benchmark and every known consumer can switch coherently.
No parallel user-facing commands or extra runtime/backend are introduced.

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
small disk-provenance control. Actual binary/slow-pipe and strict native benchmark
proof, main cutover, export/compare consumers and legacy removal remain separate
R13/R14 work; this staged library validation cannot accept those gates.
