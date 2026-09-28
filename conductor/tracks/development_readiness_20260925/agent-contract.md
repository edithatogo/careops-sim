# Ownership contract

Primary responsibility: 13/20/27/30/44. See [spec](spec.md) for owned/blocked paths and
[module readiness](../../module-readiness.md) for cross-track interfaces.

The parent owns ED domain logic, local developer tooling and integration. Kairos
owners retain reusable engine/Arrow/runner/FFI/backend changes. Review shared
manifests/contracts before edits and use separate commits/pin updates.

Project-local Agent Skills under `.agents/skills/` are parent-owned developer
workflow assets. D1.3 may inventory and author bounded candidates there; D1.4
owns held-out evaluation and promotion. Do not copy third-party skill text into
that directory without source/license review and a separate provenance record.

Parallel-safe work must have disjoint paths and bounded acceptance. One writer
owns a shared file. Delegate only under applicable session authorization; this
contract defines responsibilities and does not start agents. No new public
publication, private-data ingestion or scope expansion follows merely from a
passing local check. Follow [agent engineering](../../agent-engineering.md).


## Execution modes

This track supports serial execution or bounded parallel subagents through
[the shared execution protocol](../../execution-model.md). Workers receive one
reviewed packet, exact source hashes, write reservations, commands and behavioral
oracles. The coordinator owns shared files, integration and accepted task status.
Use [the decomposition guide](../../execution/decomposition.md) to size work for
simpler models; unresolved design decisions escalate before implementation.
