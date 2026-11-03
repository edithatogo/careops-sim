# CareOps whole-layer Polars migration

- [spec.md](spec.md)
- [plan.md](plan.md)
- [metadata.json](metadata.json)
- [agent-contract.md](agent-contract.md)
- [test-matrix.md](test-matrix.md)
- [risk-register.md](risk-register.md)
- [handoff.md](handoff.md)
- [Planning review](review-report.md)

R13 local engineering is accepted at f7563a8. All known CLI, Python/Rust,
capture, fixture, calibration, dashboard and B7 consumers use native tables;
obsolete alpha production serializers/dispatcher/descriptor exports are removed.
The coherent workspace passed776 tests including doctests with no ignored or
filtered tests. Top-level Python passed796; all127 ED cases are accounted by
126 passes from full discovery and the corrected exact E61 case.

[Phase acceptance](../../evidence/polars-r13-acceptance-20261104/README.md) preserves
exact receipts, original failures, independent reviews and the new source-bound
26-case release/stdout pass. The formatting drift is repaired at7b9892b.

R14 remains open: the upstream shared trusted-import crate must pass its source,
native, policy, Git-consumer and hosted gates before an exact parent default-off
FFI/data-source packet. Final historical conversion, native/consumer/performance,
maintained-code/build-cost, package, coverage/mutation/security/policy and
current-head hosted evidence remain separate. No archive yet. Parent
runtime074/dataf347/Polars964 and existing ownership are unchanged.
