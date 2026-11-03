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
