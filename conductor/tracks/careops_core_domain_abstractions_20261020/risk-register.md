# Risk register: Multi-domain core simulation abstractions and egress boundaries

| Risk | Impact | Mitigation |
| --- | --- | --- |
| Refactoring `careops-ed` alters event scheduling order or breaks deterministic replay | Loss of qualified milestone G1 baseline | Phase A6 regression gate with byte-by-byte fixture replay check before PR acceptance |
| Dynamic pathway engine introduces runtime overhead | Simulation throughput drops | Use index-based contiguous vector representation (`kairo-ecs-state`) with zero allocations in inner event loops |
| Multi-role team allocation causes deadlocks when multiple procedures contend for doctors/midwives | Simulation hangs | Leverage Kairos `ConditionWait` atomic acquisition or strict priority-based resource ordering |
| Egress sink boundary leaks memory or drops entities | Causal accounting violation | EgressSink must emit explicit terminal or handover events with strict entity conservation accounting |
