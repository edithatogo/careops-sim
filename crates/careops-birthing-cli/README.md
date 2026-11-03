# Bounded synthetic birthing batch

Build with canonical Rust1.99.1. Use an existing private output root:

```sh
mkdir -m700 /tmp/birthing-output
cargo run --locked -p careops-birthing-cli -- crates/careops-birthing-cli/fixtures/synthetic.json /tmp/birthing-output 4
.venv/bin/python crates/careops-birthing-cli/read_batch.py /tmp/birthing-output crates/careops-birthing-cli/fixtures/synthetic.json
```

Use `crates/careops-birthing-cli/fixtures/synthetic.json` as the explicit input.
Install the pinned independent reader in the repo-local Python3.15.1 environment
from `requirements-reader.txt`. Append `--resume` after WORKERS only for an
existing valid checkpoint. A new run never overwrites an existing batch.
Completed replication files are validated and reused; interrupted publication
attempts remain intact and restart that replication into a fresh bounded attempt.
This is completed-replication recovery, not durable within-run state restoration.

Two serial cases per day in each independent replication use B1 progression,
B3/B4 spaces and actual duties, B5 actual operative Birth callbacks, B2 identities
and B6 maternal/infant split routes with retrieval fallback. Distinct per-case
bookings plus serial case execution prevent cross-boundary escort reuse.
A single canonical identity batch seals each actual effect tick; responsibility
is audited after complete global tick batches. The original capture is exported;
there is no export-time simulation rerun.

The finite coordinator declares seconds,1–7days,1–3infants,1–32replications,
1–4replication workers, at most4096rows/replication and64rows/part. Native data
uses one process-wide thread budget. Supported case timings are explicitly
validated; entry-to-birth must be positive, LDR cleaning precedes downstream
arrival, post-birth theatre completion leaves at least3seconds for reservations,
and all declared component times are0–30seconds with sum<=150. These are
synthetic test-envelope limits, not clinical constraints or site defaults.
Operative actual family departure equals the bounded theatre's completion
callback. Vaginal co-location holds the LDR until the last modeled occupant
leaves; destination scarcity produces boarding. The coordinator gates physical
reuse against prior/current logical origin claims until actual handover.

Actual Parquet/Zstd and IPC stream parts preserve generic Schema v2 typed IDs,
u128 tick limbs and original sequence, with domain capture in payload_ref.
The required domain manifest declares tick=second, normalized effective input,
component revisions, exact normalized effective-input UTF-8 bytes and hashes. The generic file time_scale remains `ticks`;
files alone do not establish a seconds mapping. Both formats are independently
read with PyArrow26.0.0 and compared by capture sequence.

This is a generic synthetic engineering fixture. It does not implement full
postnatal/SCN/theatre internals, shared ED/birthing scheduling, cross-process
transport, fetal identity, clinical validation, site acceptance or full-spec approval.

Operative `theatre_case` records preserve the complete actual callback list, all timing fields, transfer/incision/birth metrics (including non-breaches and zero targets), and their assessment/policy provenance. The independent reader checks the foreign keys, elapsed values, policy identity/rules and breach decisions against those actual records.

Recovery requires the same model/build contract. A build-time SHA-256 binds sorted owned CareOps core/birthing/CLI Rust sources, their manifests, root manifest/lock, build receipt code, actual Rust compiler readback and target/profile/flags. The tag is part of canonical effective-input bytes, checkpoint metadata and each completed replication. A mismatched tag rejects before reusing files or starting pending work. This is an implementation/build contract under the qualified upstream pins, not a binary signature or permission to promote a runtime pin.

Domain manifest v2 explicitly declares observation uncertainty contract v1 for
actual delay-seconds, origin-occupied-seconds and target-breach observations.
`method=not_estimated` means there is no declared sampling estimand for these case
observations. The observation unit is `synthetic_case`; the experiment unit is
`independent_replication`. Cases are not asserted IID, stochastic jitter is not
called deterministic, and no zero uncertainty, population/batch estimate or CI is
claimed. Independent replications can support a later explicitly defined estimator.
Rust readback/recovery and the independent reader require the exact declaration;
old v1 manifests are historical artifacts and do not silently gain this contract.
