# C1 independent normalized physical readback v1

Coordinator reviewed 5 October 2026 for the user's explicit manual readback.
Scope: direct second-implementation reads and accounting, not full C1.4,
clinical validity, source changes, dependency changes or release acceptance.
Parent base62a2866; accepted Kairos c441c693eccf666d00e2720a215dd1f238e331ad.

Use pinned PyArrow25.0.1 direct IPC file/stream and Parquet readers. Import no
production codec, encoder, decoder, Rust schema helper or C01 interop validator.
Declare independent expected physical-v2 types, field order/nullability and
schema/field metadata from frozen c1 physical schema v1/v2 contracts and C0
schema. Never infer expectations from observed files, cast or normalize away
mismatches. Read all648 C01 Rust outputs (24 layouts,3 tables,3 formats,3 writer
limits),9 Rust current-joined C1.2 outputs and9 retained C1.1 fixture files.
Bind each input file to accepted manifests/archive checksums before decoding.

Check fixed16-byte UTC signed little-endian i128, unit ns_since_unix_epoch,
against exact RFC3339 text using integer calendar arithmetic; ticks unsigned
little-endian u128, unit1ns, against UTC minus declared origin. No float timestamp
or native Arrow timestamp narrowing. Check UInt32 occurrence, UInt64 order,
canonical decimal UTF8 ranks, bools, lists, nested structs and enum/payload values
against the actual retained logical records. All clock precision/offset/lineage
values survive. UTC text remains exact, even with zero fractional digits beyond
nanosecond precision. Parent-null clocks do not expose child-buffer values.

presence_fields is sorted unique absent logical paths. Preserve absent vs
explicit null vs present empty values, including nullable clocks/actor/detail,
absent-only quality_flags/tick_resolution/rank/source_fields and empty required
cluster_ids. raw_time_values is non-null list of non-null key/value entry structs,
keys sorted unique, values nullable. Verify recursive nullability and report
observed null/missing/empty counts and all physical field types/metadata.

Reconcile each C01 profile separately: source/candidate/mapper-accepted/excluded/
failed/unresolved, validator valid+quarantine, outcome observed/not_censored/right
and validator window-censored cases. Long profiles7rows/7candidates/6mapper events/
1exclusion/2outcomes; wide3rows/9candidates/6events/3exclusions/2outcomes.
Quarantine partitions6mapper events into3valid+3quarantined. Each profile has
1observed and1right-censored outcome, zero window-censored cases. Distinct profile
totals are fixture totals:17rawrows,23candidates,18mapper events,5exclusions,
6outcomes including3right-censored. No double counting format/layout replicas.

Separately reconcile all51 C1.1 actual mapper requests, including classified
failures, against accepted snapshot SHA and manifest request membership. This
heterogeneous fixture collection is not one ingestion dataset/cohort. Verify
31trace events,17exclusions,4outcomes in physical replicas; reconcile actual
source/candidate/failed/unresolved and censor counts from snapshot before claim.

Reader-negative controls must detect an altered unit, wrong integer width/type,
null in required field, UTC-byte/text mismatch and censored event clock conflict,
while untouched actual data passes. Retain exact command/cwd/source/tool/input/
output hashes, per-file receipts and count tables. No new Rust execution needed:
this verifies retained actual artifacts. Preserve C1.4 and unrelated Q5.2 holds.
