# Test matrix: Multi-domain core simulation abstractions and egress boundaries

| Phase | Automated Check | Manual Verification |
| --- | --- | --- |
| A0 | `cargo check --workspace` with `careops-core` | Verify crate boundary separation and dependency graph |
| A1 | Dynamic pathway cyclic, invalid transition, and terminal tests | Review pathway JSON configuration schema |
| A2 | Capability match, insufficiency, and pool allocation tests | Inspect capability-based resource resolution logs |
| A3 | Multi-role concurrent claim, preemption, and timeout tests | Inspect team assembly and release trace |
| A4 | Egress sink handover, delay, and backpressure hold tests | Verify patient entity arrives at sink with full handover payload |
| A5 | Maternal-neonate dynamic spawning and linked lifecycle tests | Verify entity ID mapping and dual-patient telemetry |
| A6 | Full `cargo test -p careops-ed` and ED benchmark regression | Compare ED trace hashes against reference oracle |
