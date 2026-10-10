# C3 parallel implementation dispatch — 10 October 2026

User commissioned continued implementation with smaller-model workers. C3.1–C3.4 are in scope; C5 and release remain separate. C1.4/C2.4/Q4.5 prerequisites are accepted. Parent task/context and MVP integrity checks pass. C3 lies outside MVP recipes, so general bounded packets apply.

Coordinator accepted ADR-0021 after three independent Luna preparation reviews. Concrete fixes added resource claims/capacities, explicit tick/event limits, run/probe provenance, infeasible outcomes, strict-policy denominators and typed recovery inventory. Contract commit: `500c699` in Kairos. This is dispatch authority within the user scope, not C-03 acceptance.

Three supervised gpt-6-luna leaves: ledger (observed source and snapshots), pool (isolated generic probe lifecycle and typed recovery), report (residuals and policy). Separate Kairos worktrees `kairos-c3-{ledger,pool,report}-20261010`, all from contract commit. Exact hashed packets are retained locally in `.artifacts/c3-dispatch/`; each worker claims only its source, test harness and unique artifact directory. Coordinator owns shared interfaces, module wiring, native Flow/C2 adapters, portable recovery join, Arrow join and phase acceptance.

Rust 1.99.0 only. Worker reports require actual command/log evidence. Native C-03 and hosted qualification remain pending.
