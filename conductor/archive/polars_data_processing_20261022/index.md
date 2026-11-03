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

R14 engineering is accepted at258bb6b and this track is archived. All20 tasks
remain registered and accepted; the three downstream R14 prerequisites retain
their IDs. Runtime074/data473/Polars964 remain unchanged. See the
[final source receipt](../../evidence/polars-r14-qualification-20261104/README.md)
for complete hosted gates, actual benchmarks, retained failures, review, positive
maintenance cost and rollback. No main-merge, stable-release or clinical claim
is added; no new implementation task is dispatched.
