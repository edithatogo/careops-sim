# Risk register: Generic birthing suite simulation delivery

| Risk | Impact | Mitigation |
| --- | --- | --- |
| Long duration of labor (12–24h) causes simulation performance drop | Benchmark slowdown | Event-scheduled DES transitions instead of fine-grained tick polling |
| Mother and infant entity desynchronization causes orphaned events | Telemetry and causal tracking corruption | Strict lifecycle binding in `LinkedEntityPair` with dual-key assertion |
| Downstream ward full blocks birthing room indefinitely | Birthing unit gridlock | EgressSink models realistic boarding delays with explicit bed-blocking telemetry |
| Emergency CS preemption causes starvation of non-acute laboring mothers | Unrealistic queue explosion | Cap preemption duration and measure queue delay variance across acuity |
