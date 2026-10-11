# E1.4 evidence reconciliation — 11 October 2026

This archive reconciles the accepted synthetic E1 native fixture families against their frozen contracts and actual retained runtime traces. It records 46 unique fixture IDs: 23 full terminal endpoints, six E0 single-stage projections and 17 invalid-input controls. The manual readback covers all 20 acuity/pathway combinations plus empty, one-patient and no-offer cases, seeded generator-to-terminal composition, and the two actual abandonment scenarios.

The E1.3 integrated source is `af3dd4a2e0d152ba4e3eda2caf185c3adeaf7181`. Its accepted receipt records 117 native package tests, strict Clippy and rustfmt at unchanged source; this evidence task reused those receipts and did not rerun native gates. The E1.1 frozen-fixture Python suite and Conductor context check are recorded in this task's `result.json` and logs.

`fixture-runtime-crosswalk.json` maps every frozen case ID to its actual native test and evidence type. Endpoint rows contain concise fields extracted from actual native serialized results. E0 extreme rows are explicitly projections. Invalid rows record native baseline, rejection-substring and repaired-control assertions; the error strings were not printed individually and are not represented as captured traces.

`manual-trace-readback.md` independently reconciles stage intervals, decision time, offer wait, physical departure, ED space occupancy, completed LOS, horizon censoring, composition conservation and abandonment timing.

## Limits

This evidence covers the native generic library and synthetic profiles. It does not establish clinical validity, Cairns-site behavior, CLI E1 integration (the CLI remains E0 schema 1), E-R4 warmup/uncertainty/operational utilization, MVP behavior, hosted execution, release readiness or operational acceptance. E1.4 remains subject to coordinator review and plan closeout.
