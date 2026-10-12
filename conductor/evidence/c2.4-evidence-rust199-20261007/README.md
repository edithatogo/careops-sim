# C2.4 phase evidence reconciliation — Rust 1.99.0 scope

**Coordinator status:** `C2.4.evidence` locally accepted at parent evidence base `a084862`; C2.4 phase acceptance remains open. See [the acceptance receipt](acceptance-rust199.json).

This inventory reconciles the accepted local evidence at parent base `b55b46ca840598cb800146b0815edd91b0e426d7`, whose Kairos pin is `5cc825048a1116740e80e6c067ba046306f2ce10`. The machine-readable receipt links the exact accepted records and their hashes.

## Accepted local joins and their scope

- **C2.1**, pin `bfbefa87`: mode precedence/boundary behavior and paired Macro Route versus explicit-zero Micro Flow outcomes; 442 passed, 0 failed, 5 ignored. This is local exact-pin evidence.
- **C2.2**, pin `9b360f4`: provider, policy, Flow submit/macro-zero/transit retention and seed continuation; 441 passed, 0 failed, 5 ignored. Its `C2_1_runtime_accepted=false` records the earlier prerequisite snapshot; the later C2.1 parent join is separately accepted.
- **C2.3 flow/resource lifecycle**, pin `dbd12c9`: typed Flow actor requests, timed work and cleaning observation, accepted callback lineage, rejection and conservation; 442 passed, 0 failed, 5 ignored. Its older `C2_1_runtime_accepted=false` is historical relative to the later C2.1 join.
- **C2.3.dispatch**, pin `7672a67`: one staff-policy selector instance for eligibility, urgency/FIFO/stable tie-break, zone/skill/availability, and separate work/walk/wait. Flow transit dispatch, resource reservation and cleaning are outside this instance.
- **C2.4.runtime-ledger**, pin `5cc8250`: a test-only integrated Macro/Micro elapsed-time fixture through actual Flow work and accepted transit arrival; 443 passed, 0 failed, 5 ignored, with strict Clippy and formatting passing on Rust 1.99.0.

These are bounded local joins and leaf scopes. They do not constitute C2 phase acceptance.

## Manual accounting and open gates

The separate synthetic manual ledger reconciles Macro elapsed time of 37.5 seconds with Micro queue (4.0), active clinical work (12.0), transit (18.5), and interruption pause (3.0), also totaling 37.5 seconds. Both describe one window. This is standalone arithmetic only; it is not integrated runtime or Q4 pause/resume proof.

C2.4 phase review, Track03 owner/phase review, Q4 interruption acceptance, portable checkpoint/restart, hosted exact-head checks, release, and clinical acceptance remain open. This evidence packet does not alter the phase checkbox or any parent/source pin. All Rust evidence referenced here is limited to stable Rust 1.99.0.
