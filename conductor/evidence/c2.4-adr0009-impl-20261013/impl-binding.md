# ADR-0009 fresh-process restore binding

Status: binding packet for a later codec worker. No Rust in this leaf.
Parent base: `0de745a50706392c8e8fea7d4a47b73a8249677c`.
Kairos pin (gitlink, unchanged): `7af989c8951c232f7bc4e6a2f543c4ee0061af99`.
Closes the format, codec, tolerance, and public-API items that
`conductor/evidence/c2.4-adr0009-envelope-20261012/envelope-spec.md` §6 left open.
Crash and commit ordering stay unbound and remain C5.

Layout below is the private codec already present at that pin. A later worker
implements and proves the oracle against these bytes. It must not invent a
second envelope.

## 1. Envelope byte layout

Source: `libs/kairos/crates/kairo-ecs-calibration/src/checkpoint_envelope.rs`
SHA-256 `5c444f2edfcce23d64b69a2099b3b591b82496a76486cd0a24d8679c7dcdc5d0`.

All multi-byte integers are unsigned little-endian. Fixed header is 148 bytes:

| Offset | Width | Field |
| --- | --- | --- |
| 0 | 8 | Magic `KC2ENV01` (`0x4B 43 32 45 4E 56 30 31`) |
| 8 | u32 | Schema version `1`. Any other value is unknown. |
| 12 | 32 | Caller-trusted `model_code` |
| 44 | 32 | Caller-trusted `configuration` |
| 76 | 32 | Caller-trusted `owner_schemas` |
| 108 | u64 | Body length in bytes |
| 116 | 32 | SHA-256 of the body only |

The body begins at offset 148 and occupies exactly `body_len` bytes. The file
length must equal `148 + body_len`. A short header, a short body, or one
trailing byte is truncated/malformed. The digest detects body corruption. The
three 32-byte identities are compared with the caller-supplied binding; bytes
in the file do not select code, graphs, or decoders.

Encode measures `148 + body_len` against `max_file_bytes` and `max_body_bytes`
before the single output allocation. Decode reads the fixed header and checks
magic, schema, binding, and exact length before allocating the body.

Publication stays the existing crate-private `write_file_no_clobber`:
same-directory exclusive temporary file, write header then body, `sync_all`,
hard-link onto a path that must not already exist, remove the temporary, then
best-effort directory sync. Hard-link failure is an error. There is no rename
or partial-file fallback.

## 2. Composite body the envelope frames

Source: `libs/kairos/crates/kairo-ecs-calibration/src/checkpoint_sections.rs`
SHA-256 `87748498d61f32491a1e3da4724e7f62d7ef29e0a875fe0f935f0f89c7fbdeb1`.

The envelope body is one section directory. Header is 14 bytes: magic
`KCSBODY1`, schema `u16 = 1`, `section_count` as `u32`. Unknown schema or tag
fails closed. Trailing bytes fail closed.

Exactly four singletons, in this order, each `u8` tag, `u64` payload length,
then payload:

| Tag | Owner payload |
| --- | --- |
| 1 | DES Flow checkpoint image |
| 2 | Fidelity policy/decisions |
| 3 | Intrinsic-work provider |
| 4 | Calibration seed-registry snapshot |

Then zero or more records merged in ascending `EntityId` order. A record is
`u8` tag, `u64` index, `u32` generation, `u64` payload length, payload.
Tag 5 is a bound bridge item. Tag 6 is a submitted bridge item. Duplicate or
descending ids are rejected. `FlowRuntimeIdentity` is absent from every
payload (ADR-0006); logical ids are rebound by the caller.

Owner payload codecs at the same pin, which this leaf does not redefine:

- DES Flow: `libs/kairos/crates/kairo-ecs-des/src/flow/checkpoint/wire.rs`,
  magics `KFLOWV1\0`, `KFLOWV2\0`, `KFLOWV3\0`, `KFLOWV4\0`, SHA-256
  `79a605b608ee6d3b8a80ef909e9edcb54c6b6cfb13483387d8e089638ba81446`.
  Scheduler frontier and engine RNG travel inside this image.
- Calibration seeds: `libs/kairos/crates/kairo-ecs-calibration/src/seed_map/checkpoint_wire.rs`,
  magics `KSMW`, `KSTW`, `KSKW`, version byte `1`, SHA-256
  `68b56ecb59914192e90342d7e542661d4e3a038c76797e408946ea6b6c05f876`.
- ABM transit state travels inside the bridge record payloads already framed
  by tags 5 and 6. An active or suspended transit is not a legal first-leaf
  boundary.

## 3. Codec rule

DES, ABM transit, engine RNG, and calibration seed/provider/fidelity images use
explicit little-endian integer and length-prefixed byte codecs. Serde, JSON,
and bincode are not the checkpoint codec for those types. Counts and lengths
are checked before allocation. Unknown versions, duplicate entries, overflow,
non-canonical order, invalid UTF-8, and trailing bytes reject before any
caller-visible runtime mutation.

## 4. Event k

`k` is **unbound**. No numeric event index is asserted in the fixture source.

Lookup performed, read-only, at pin `7af989c8951c232f7bc4e6a2f543c4ee0061af99`:

- File: `libs/kairos/conformance/c20/paired_flow_c20_v2.rs`
- SHA-256: `40b0c380ddea4247297a2f36ba1bba315059344cb8430132f721d9b399ab76dc`
- Test: `macro_and_explicit_zero_micro_pair_actual_provider_work_without_transit_events_or_draws`
- The test builds one `FidelityMode::Macro` scenario and one
  `FidelityMode::Micro` scenario with `TransitRequest::Zero`, then calls
  `flow.run_for(128)` on each.
- It asserts Macro/Micro dispatch equality, `service_draw_position() == 1`,
  one `LifecycleTransition::Queued` record, and one `Completed` record at
  tick 30 with `completion_at == None`. It compares
  `budget_snapshot().scheduler.dispatched_events` between the two scenarios
  and does not require that count to equal any literal.

`128` is the `run_for` cap. Tick `30` is the completion time. Neither is event
*k*. The later worker binds *k* by reading
`budget_snapshot().scheduler.dispatched_events` after `run_for(128)` returns
in that test, at this pin, for both modes, and recording the equal count.
Until that measurement is in the receipt, the oracle stays `k: unbound`.

## 5. Non-public entry

Restore and publish stay crate-private on `kairo-ecs-calibration`:

- `C2PortableCheckpointV1::capture`
- `C2PortableCheckpointV1::save_no_clobber`
- `C2PortableCheckpointV1::read_file`
- `C2PortableCheckpointV1::restore`
- `checkpoint_envelope::{encode, decode, read_file, write_file_no_clobber}`

A `#[cfg(test)]` harness inside that crate is allowed when an integration test
cannot see `pub(crate)`. Track 25 forbids a new public, `pub use`, C ABI, or
doc-hidden exported restore/publish API in this leaf.

## 6. Fail-closed negatives

Each case rejects before any existing runtime is mutated. A fresh-process
restore that fails drops its staged runtime. The four required cases:

1. **Corrupt.** Body byte or SHA-256 digest does not match. Error class:
   envelope `InvalidFormat`.
2. **Truncated.** Fewer than 148 header bytes, body shorter than the `u64`
   length, or a trailing byte. Error class: `InvalidFormat`.
3. **Unknown.** Envelope schema other than `1`, section schema other than `1`,
   or a section tag outside 1–6. Error class: `UnsupportedSchema` or
   `InvalidTag`.
4. **Active work.** Active or suspended transit, an in-flight callback batch,
   submission, permit, partial batch, or shard result, or Flow work other than
   empty or one idle-queued bridge item. Rejection happens before mutation.
   The test keeps a live runtime and shows its event ids and Service draw
   position unchanged.

Validation order remains envelope spec §4: version and exact inventory, hashes
and lengths, limits before allocation, registered codecs, then the boundary
check.

## 7. Oracle and tolerance

Fixture: the C2.1 paired Macro / explicit-zero Micro test named in §4.
Procedure, once *k* is measured: run to event *k*, publish the envelope, exit;
restore in a new process and continue. Require exact equality of event ids,
the canonical integer trace hash, and Service draw positions with the
uninterrupted run. Tolerance is exact equality. This leaf adds no absolute or
relative numeric tolerance.

Bind the receipt to Kairos pin `7af989c`, Rust 1.99.0, and the seed and config
hashes of that fixture (`CalibrationSeedMap::new(1, "study-α", 1234)`,
`crn-v1` / replication `7` / `case-0001` / `triage:1` / `SeedPurpose::Service`).

## 8. Non-goals

- Worker counts 1/2/N, permuted completion order, crash recovery, and
  mid-study resume remain C5 test-matrix row `calibration_worker_resume`.
- Crash and commit-ordering details stay unbound.
- No Kairos pin bump, no `crates/careops-ed` edit, no `Cargo.lock` edit, no
  Renovate change.
- Transit-progress checkpoint remains the second leaf.
- No C2 phase, hosted, release, or clinical acceptance claim.
