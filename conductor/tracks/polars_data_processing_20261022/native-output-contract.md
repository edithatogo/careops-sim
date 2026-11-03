# ED native output v1 — bounded production library boundary

This R13.2 slice exports original validated captures, not another simulation.
Default real Parquet uses Zstd; explicit formats are Arrow IPC stream and typed
CSV. Files contain the qualified joined careops.ed.patients.v2 table, a complete
replication-control projection and the existing qualified five metric records.
Raw capture parts are distinct artifacts and are not relabelled joined tables.

A small careops.ed.native-output.v1 manifest completes last. It binds effective
and parsed input hashes (raw input absent for this parsed API), model, producer,
source revisions, units, horizon, canonical replication seeds, ordered physical
column types and fixed domain nullability, contiguous row offsets and actual
file lengths/SHA256. Parquet/IPC embed provenance before hashing through the
accepted Kairos writers. CSV metadata lives in its manifest; the library writer
uses necessary quoting, unquoted empty nulls and quoted empty strings.

Qualified finite ceilings are1,000,000total rows across all tables,4096parts,
256rows/part,4MiB/part including framing/metadata,1MiBcontrol manifest. Callers
may lower budgets. These are logical/encoded bounds, not a universal RSS claim.
Check declared counts before frame materialization; validate actual capture and
summary binding. Plan all bounded parts and final declarations before writing.
Only a pure observed byte-cap hit may split a row chunk at its midpoint; other
errors propagate. No compression monotonicity assumption or physical retry.

Fresh empty real caller-owned destination only; existing publisher provides
same-filesystem no-clobber staging, file sync and directory sync. Preserve
incomplete artifacts on error; absent/invalid completion controls cannot imply
success. Export resumption is not supported. No new storage backend, full-table
JSON, schema-DDL output, clinical interpretation or movement claim is added.

Production CLI/default consumer migration, legacy deletion, independent direct
FFI and final performance/full hosted/clinical gates remain separate tasks.


## Closed native reader/conversion profile

The reader snapshots all declared parts before typed decoding, checks source
hashes, exact ordered schemas/provenance and row coverage, and retains immutable
Polars frames after source files disappear. Conversion reuses the no-clobber
publisher without a simulation rerun; a final conversion receipt binds actual
source and target manifest bytes.

The finite profile caps encoded data at64MiB/dataset and4MiB/part, estimated
decoded payload at64MiB/part and256MiB/dataset, footer1MiB and page/IPC metadata
64KiB. The same profile applies during pure planning and actual file readback.
These are payload/allocation profiles, not a universal peak-RSS claim; R14's
measured RSS gate remains required. Standard generated IPC/Parquet metadata APIs
inspect actual buffer/page bounds before high-level decoding. Only the pinned
flat producer schema, encodings and metadata are accepted. CSV uses explicit
types, quote-aware counting and canonical standard-writer roundtrip; quoted empty
strings remain distinct from null. Historical fixtures require actual readback.

The metadata interfaces are existing locked dependencies, not a new backend.
Their separate strict compatibility receipt does not renew A7 or complete R13.


## Next bounded fixture view contract

This is the reviewed interface for the next packet, not a claim of implemented
or qualified source at63a30b8. CompositeEdFixture has no original capture-backed
BatchRunDocument; never fabricate one or input/model provenance to use the native
ED exporter. Keep to_run_summary as the authoritative online projection.

A narrow composite_fixture::native_fixture::fixture_patient_table accepts that
RunSummary and returns a Polars DataFrame under native-data. Schema is
careops.ed.synthetic-fixture-patients.v2; source_kind is
synthetic_fixture_snapshot. Columns carry fixture_id, seed, time_unit,
horizon_ticks, patient_id, status and high/low UInt64 limbs for arrival_tick,
start_tick, completion_tick, work_ticks, wait_ticks and elapsed_ticks. This u64
producer's present high limbs are zero; optional pairs are both null together.
Preserve every completed, in-flight and queued patient in source canonical order,
with unique patient identity and no row loss/multiplication. Work and duration
facts preserve the existing summary meaning. Caller-declared identity/seed/horizon
is not an input hash, model/runtime identity, replication or observed movement.
Keep the completed-only calibration_table separate for route/transit provenance.

Migrate the three existing C62/E42-heldout/C64 consumers together. Preserve actual
Macro/Micro execution, transit0/20, preemption/work conservation, disjoint and
stochastic cohorts, uncertainty vectors, horizon censoring and existing timing
oracles. C62/E42 wait means include all nonnull starts; C64 means remain
completed-only. Keep actual online busy-bay utilization independent from endpoint
interval occupancy; preserve the latter's end-before-start tie order and value3.
Replace old full-row JSON/descriptor assertions with typed native frame and real
interchange/hash checks. C64's genuine reassembled capture may use the existing
native publisher. Domain/scalar checks stay enabled in minimal graphs; only
actual native capabilities receive feature cfgs. Do not resurrect row/comparison
DTOs or nominal CLI runs as substitutes for real fixture execution.
