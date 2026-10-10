# C3 parallel implementation dispatch — 10 October 2026

User commissioned continued implementation with smaller-model workers. C3.1–C3.4 are in scope; C5 and release remain separate. C1.4/C2.4/Q4.5 prerequisites are accepted. Parent task/context and MVP integrity checks pass. C3 lies outside MVP recipes, so general bounded packets apply.

Coordinator accepted ADR-0021 after three independent Luna preparation reviews. Concrete fixes added resource claims/capacities, explicit tick/event limits, run/probe provenance, infeasible outcomes, strict-policy denominators and typed recovery inventory. Contract commit: `500c699` in Kairos. This is dispatch authority within the user scope, not C-03 acceptance.

Three supervised gpt-6-luna leaves: ledger (observed source and snapshots), pool (isolated generic probe lifecycle and typed recovery), report (residuals and policy). Separate Kairos worktrees `kairos-c3-{ledger,pool,report}-20261010`, all from contract commit. Exact hashed packets are retained locally in `.artifacts/c3-dispatch/`; each worker claims only its source, test harness and unique artifact directory. Coordinator owns shared interfaces, module wiring, native Flow/C2 adapters, portable recovery join, Arrow join and phase acceptance.

Rust 1.99.0 only. Worker reports require actual command/log evidence. Native C-03 and hosted qualification remain pending.

## Wave 2

Protocol join `2b7cb7d` passes focused ledger/pool/report tests and strict Clippy on Rust 1.99.0. Macro/Micro fidelity is now independent of ShadowAnchored replay role. Review identified snapshot visibility, failed-dispatch accounting and terminal restore gaps; phase acceptance remains open.

Three reviewed Luna packets at that exact base: native-model (actual C2 worlds and executable recovery fixture), runner (trusted observed replay and typed recovery), contract-fixes (review defects and negative fixtures). Disjoint worktrees and claims; root owns module wiring, combined fixture, Arrow integration and final review. Local packets contain exact input hashes and bounded commands.

Native model structural handoff: two executable import approaches failed because nested private modules resolve differently under path/include imports. WIP `d3eb4de` is not accepted. Packet v3 explicitly transfers private `lib.rs` wiring to the native worker and uses the actual crate test executable; root defers conflicting module edits until handoff. No public API is added. Independent review also required exact restored holder inventory, matching-event error propagation and the correct 20-byte outer header bound.
