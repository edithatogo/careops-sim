# C2.4 integrated runtime-ledger leaf — Rust 1.99.0

The `C2.4.runtime-ledger` leaf is accepted locally at Kairos commit `5cc825048a1116740e80e6c067ba046306f2ce10`, parented at that exact pin. The fixture is test-only and drives both fidelity arms through actual Flow requests and completions. A capacity-one blocker holds until tick 5. Macro queues from tick 0; Micro follows a scheduled two-tick route and submits work only after accepted arrival at tick 2. Both arms share the same sampled Service duration (20 ticks) and stream identity, receive the resource at tick 5, and complete at tick 25.

The measured intervals are disjoint: Macro has 5 ticks queued plus 20 service; Micro has 2 ticks transit, 3 queued, and 20 service. The full elapsed time is 25 ticks in both arms. The TransitContext records route progress; the Flow work records actual completion.

The worker's calibration/ABM tests passed (190/0/5), with strict Clippy and formatting on stable Rust 1.99.0. The coordinator reran the combined DES, ABM, and calibration suites at the exact parent pin: 443 passed, 0 failed, 5 ignored across 43 result summaries; strict Clippy and formatting passed. Packet, worker result, logs, exact hashes, and prerequisite receipts are preserved alongside this README.

This accepts one local C2.4 runtime-ledger leaf and its exact-pin parent join. The C2.4 phase remains open: C2.4 evidence reconciliation and owner/phase review remain required. Track03 owner review, hosted checks, portable process checkpoint/restart, release and clinical acceptance are not established.
