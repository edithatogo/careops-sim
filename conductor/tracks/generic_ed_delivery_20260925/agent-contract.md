# Ownership contract

Primary responsibility: CareOps; Kairos 03/21/22/05/09/32/34/35. See [spec](spec.md) for owned/blocked paths and
[module readiness](../../module-readiness.md) for cross-track interfaces.

The parent owns ED domain logic, local developer tooling and integration. Kairos
owners retain reusable engine/Arrow/runner/FFI/backend changes. Review shared
manifests/contracts before edits and use separate commits/pin updates.

Parallel-safe work must have disjoint paths and bounded acceptance. One writer
owns a shared file. Delegate only under applicable session authorization; this
contract defines responsibilities and does not start agents. No new public
publication, private-data ingestion or scope expansion follows merely from a
passing local check. Follow [agent engineering](../../agent-engineering.md).
