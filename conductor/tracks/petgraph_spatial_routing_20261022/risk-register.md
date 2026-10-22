# Risk register

| Risk | Required control |
| --- | --- |
| Incompatible dependency/platform or unsupported version | Entry qualification and reviewed compatibility policy; stop instead of silent fallback |
| Behavioural drift hidden by updated goldens | Independent test-matrix oracles and explicitly versioned changes |
| More adapters/code than removed | Before/after production inventory, dependency and runtime cost; justify retained exceptions |
| Stale or circular cross-repository acceptance | Verify exact commit/contract receipts in the programme order; local DAG is not external proof |
| Shared owner conflict | Bounded handoff and fresh advisory claim; no expiry takeover |
| False completion | Keep all runtime boxes pending until actual evidence and review |

Track-specific failure modes and required controls are in [specification](spec.md) and [test matrix](test-matrix.md).
