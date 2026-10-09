# Full-state foundations: implemented, reviewed and locally qualified

Root integrated three Luna component implementations and their corrections under
ADR-0016. Compiled sourcee5a1302; docs-only published successorfad0380 in draftPR243.
On Rust1.99.0, the exact staged tree equals the source commit tree: **527 passed,
zero failed,five ignored/54 groups** across core/state/DES/ABM/calibration with
flow/test-support features. Strict five-package Clippy and workspace format pass.
Logs, source/toolchain/command hashes and independent worker receipts are retained.
No native test was inferred from a default-feature package test.

- Scheduler image preserves every physical heap entry, pending membership, exact
  IDs/generations, sequences and counters. Import rejects all declared malformed
  field/counter/order states and unreachable zero-dispatch/nonzero-clock state;
  capture and restore use fallible reservations, while past scheduling stays legal.
- Allocator image preserves every slot, generation, free LIFO and dense order.
  Independent review rejected an initial quadratic validator. Corrected import
  uses bounded O(n) membership scratch before constructing the destination; tests
  include100000fragmented slots, futureIDs, empty/omittedpartitions and MAXgenerationwrap.
- Seed registry image preserves all purposes/identities in canonical seed order.
  Independent review caught a prefix allocation before complete byte-cap preflight;
  corrected two-pass import validates total limits first. Root also rejected new
  public error variants; checkpoint errors now remain crate-private and existing
  public CalibrationSeedError is unchanged. Full flow-feature worker tests pass.

Internal independent reviewers accepted each corrected source. No scheduler/RNG/
allocator algorithm, dependency, manifest, public FFI contract or stable API promise
changed. The new core/state Rust seams are doc-hidden experimental public methods.

These are complete owner component images, not an outer byte codec or complete
Flow restart. Remaining actual implementation: component registry and built-in
Flow state (works/requests/resources/leases/commands/notifications/budgets), named
handler/context codecs and alias policy, fidelity/provider/stream/bridge payloads,
coherent capture frontier, validated transactional new-runtime rebinding, Track22
bounded atomic envelope and fresh-process parity across the full C2 mode/progress
matrix. C5 worker/candidate crash/ranking equivalence and clinical/release gates
remain separate. C2.4 remains unchecked; the goal remains the full C2 outcome.
