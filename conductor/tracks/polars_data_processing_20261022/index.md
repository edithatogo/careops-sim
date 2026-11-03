# CareOps whole-layer Polars migration

- [spec.md](spec.md)
- [plan.md](plan.md)
- [metadata.json](metadata.json)
- [agent-contract.md](agent-contract.md)
- [test-matrix.md](test-matrix.md)
- [risk-register.md](risk-register.md)
- [handoff.md](handoff.md)
- [Planning review](review-report.md)

Local engineering checkpoint through63a30b8 includes the real native CLI
entrypoint. The earlier source-bound release checkpoint passed26case compatibility
and unchanged stdout/RSS/time budgets; final-source qualification remains open.
Reviewed CI/Python/Rust/capture/dashboard packets are integrated; obsolete
example runner and unused fabricated dashboard exporters are removed.
See [dated review checkpoint](review-report.md#r13-continued-local-cutover-checkpoint--3-november-2026).

R13.2/R13.3 remain unchecked: remaining C62/E42-heldout/C64 fixture
consumers, production alpha serializers and current
instructions require migration. R14 populated parent FFI, full native/consumer,
performance/build-cost, clean-package, coverage/mutation/security/policy and
current-head hosted gates remain open. Known whole-workspace native_reader.rs
rustfmt drift still needs an exact-source qualified repair. No archive yet.
Runtime074/dataf347/Polars964 pins and existing ownership are preserved.
