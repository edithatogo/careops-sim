# Test matrix: Generic birthing suite simulation delivery

| Phase | Automated Check | Manual Verification |
| --- | --- | --- |
| B0 | JSON/YAML schema validation and parameter distribution bounds | Compare distributions against AIHW national percentiles |
| B1 | Progression state machine tests (nominal, induction, protracted) | Trace individual patient labor progression |
| B2 | Dynamic neonate generation and linked telemetry tests | Verify linked maternal and neonatal records in Arrow export |
| B3 | Space reservation, pool allocation, and room turnover tests | Inspect room occupancy time-series |
| B4 | 1:1 active labor midwifery invariant and call-in doctor response tests | Review midwife utilization and staffing logs |
| B5 | Emergency CS preemption and 30-min decision-to-delivery tests | Verify preemption event trace and time-to-theatre metrics |
| B6 | Egress sink routing (Ward, Theatre, SCN, Retrieval) tests | Verify patient entity conservation across all egress sinks |
| B7 | End-to-end multi-day scenario runs and Parquet telemetry output | Benchmark run throughput and inspect scenario summary |
