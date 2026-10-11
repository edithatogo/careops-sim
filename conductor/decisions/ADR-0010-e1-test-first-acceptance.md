# ADR-0010: E1 test-first acceptance and bounded parallel fixtures

Accepted engineering decision, 2026-10-11. User commissioned E1.1 implementation
and bounded follow-on work. Coordinator review plus GPT-6.1-sol independent review
confirmed the E0.1 precedent in e0.1-patient-trace-acceptance-20261002.md and
e0.1-schema-fixtures-acceptance-20261002.md: behavior-first executable fixtures
may be accepted before the later task implements the runtime.

E1.1 acceptance means complete reviewed fixture specifications, independent
arithmetic/validation tests and mutation evidence, not working ED pathways.
Every unbound native assertion must remain inventoried for E1.2/E1.3. E1.4
requires actual native conformance of every case; no skip or fixture-only pass
substitutes. Existing E0 behavior remains separate and unchanged.

Freeze [the shared contract](../design/ed/e1-fixture-contract.md) before dispatch.
Replace only E1.1's conservative internal serial edges with the same six accepted
external closeouts for each leaf. There is no produced-data dependency between
these families: each writes its own JSON/test file and consumes the shared
contract read-only. All three instance sets remain mandatory at the join.
Any contract drift stops/rebinds affected workers. Runtime E1.2 leaf dependencies
remain serial; use writer/reviewer/next-packet planner roles there.

No private data, new public API, upstream modifications, release acceptance or
unbounded unattended work is authorized by this decision. A thread heartbeat
may resume only the next E1.2.arrivals leaf after exact E1.1 acceptance readback.
