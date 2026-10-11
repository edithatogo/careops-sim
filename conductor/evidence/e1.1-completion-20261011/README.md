# E1.1 fixture-specification acceptance — 11 October 2026

Accepted 46 synthetic cases: 23 endpoints (including all20 acuity/pathway/diagnostic
combinations), six extremes and17 invalid inputs. Three GPT-6-luna workers wrote
disjoint families; coordinator and cross-worker reviews checked the integrated
sources. Twelve fixture tests passed, with mutation/repair checks. Separately,
28 existing E0 native tests passed on Rust1.99.0. Initial offline dependency-cache
failure is retained; locked online retry passed and Cargo.lock did not change.

This accepts behavior-first specifications under ADR-0010, not a running ED
pathway. Every case remains explicitly unbound in runtime-binding-debt.json.
E1.2/E1.3 must attach actual runtime assertions; E1.4 cannot close until all pass.
Python is used only for independent test arithmetic, not the simulation runtime.

See acceptance.json for integrated source/hash bindings, workers/ for exact
packets and logs, reviews.json for independent review, and fixture-tests.json
for coordinator execution. Expected counters in invalid fixtures specify desired
pre-scheduling rejection; they do not prove native transactional behavior.

Next: E1.2.arrivals only, against the reviewed e1-arrivals-contract. A thread
heartbeat is configured to perform fresh acceptance/claim checks and avoid
repeated dispatch. It retires after that leaf; no indefinite execution loop.
