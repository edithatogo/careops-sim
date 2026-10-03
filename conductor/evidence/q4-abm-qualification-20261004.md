# Q4 shared ABM adapter qualification

Kairos development commit `c8f1558fba700a8c3cc506348d4f64e9113725a9` joins
accepted buffered despawn and the optional Arrow/seed foundations, adds the
reviewed ABM-to-Flow bridge, and includes its read-only pre-work registration
prerequisite. The bridge uses Flow's actual context row, world, committed time
and entity-derived deterministic stream. It introduces no second world or clock.
Legacy ABM source is unchanged except module and public export declarations.

## Executed local and hosted evidence

Actual Rust 1.88.0 and 1.98.1 local commands each passed 20 ABM and 160 DES tests.
The seven public behavior families cover shared time/lifetime, independent actor
streams and pause boundaries, recycled generations, registration/carrier failures,
rejected batches without callback replay, preconsume budget preservation, and
real task leases with carrier acquisition excluded. The private gate inspected
the actual typed carrier's raw RNG state, state and behavior count before and
after budget rejection and repeated fail-stop. Creation consumes no RNG draw.

Both actual local compilers rejected the view and callback-snapshot lifetime
escapes using the actual JSON-resolved library artifacts. These are local
compiler proofs; the hosted workflow does not execute those negative files or
an ABM/DES Rust 1.88 lane. The earlier private-test result-handling failure,
late-registration behavioral failure, prerequisite compile-red and artifact
selection correction remain retained in the bound local lineage. The successful
Rust 1.88 suite output was reused for lifetime checks rather than repeated.

[Owner run 37161242184](https://github.com/edithatogo/kairos/actions/runs/37161242184)
completed successfully at the exact child commit. All eleven jobs passed. Native
Ubuntu x86_64 and macOS ARM owners each recorded Rust 1.98.1, 20 ABM and 160 DES
tests, including all seven public families and the private raw RNG/state gate;
the complete reusable owner lane recorded 273 results on each host. Ten retained
artifact ZIPs matched their GitHub API SHA-256 digests and passed ZIP CRC checks.
Archive integrity does not add clinical or release qualification.

The [combined local and hosted receipts](q4-abm-qualified-receipts-20261004.json)
retain exact command, source, toolchain, job, oracle-location and raw-log hashes.
Root independently reviewed the source and repeated the receipt/archive checks
before authorizing this bounded pin. External temporary raw-log paths are
collection provenance, not a promise of durable local retention. Original receipt
status records remain historical; this note records the bounded root disposition.

## Remaining gates

This qualifies the ABM adapter and registration prerequisite, not full Q4 or C1.
Q4's Track 04 per-transition resource lifecycle snapshots/sidecar, synthetic
staff/bed workflow and phase review remain incomplete. Lifecycle snapshots must
capture the causal prior lease separately from resulting active state at the
staged transition, not reconstruct it from final state. C1 normalization,
clinical mapping, replay and full acceptance remain pending. Experimental public
API compatibility and release holds are retained.

Parent integration remains governed by exact-head required hosted checks, review
and PR merge readback. Parent billing has been confirmed blocked; that failure
is not a passing or waived gate. Kairos availability is workflow-specific: this
exact owner workflow actually succeeded. The child proof does not establish
parent hosted acceptance or operational/clinical validation.
