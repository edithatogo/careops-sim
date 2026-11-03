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

R14 source qualification now includes reviewed upstream data473ff2c, all three
upstream hosted platforms, parent adoption a61f259 and immutable guard86dcc7c.
Runtime074/Polars964 and existing ownership remain unchanged. Populated FFI,
historical conversion, release26-case/stdout, bounded ensemble/conversion/worker
measurements and the current unpacked package have scoped local receipts.
[Current R14 checkpoint](../../evidence/polars-r14-qualification-20261104/README.md)
records their limits and the positive maintained-code cost. R14 remains open:
finish coherent native/coverage/mutation checks and exact-head hosted CI/release
readbacks after disk recovery. All R14 boxes remain unchecked; no archive yet.
