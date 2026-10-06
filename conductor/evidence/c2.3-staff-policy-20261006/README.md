# C2.3 staff policy instance — local evidence

Kairos commit `b2e42e7c2a4ed157218e9de15807410d513e65ae` adds a private deterministic selector in `kairo-ecs-calibration`. It ranks requests by urgency descending, FIFO sequence ascending, then request ID; filters staff by availability, exact assigned zone, and required-skill inclusion; and breaks staff ties by stable actor ID. Work, walk, and wait are distinct values; waiting requests do not receive a staff assignment. Duplicate IDs fail closed.

The source and parent pin are integrated in this local checkout. The focused Rust 1.99 test run passed 6/6; strict all-target Clippy and workspace formatting passed. Exact commands, source hashes, and raw logs are in `acceptance.json`.

This accepts only the pure staff-selection instance. It does not schedule Flow transit events, reserve a staff resource, or implement cleaning/reservation lifecycle. C2.3 dispatch integration, C2.2 provider-to-Flow admission, C2.1 paired runtime oracles, hosted exact-head checks, stable API/release, and operational acceptance remain open. No C2 plan checkbox was changed.
