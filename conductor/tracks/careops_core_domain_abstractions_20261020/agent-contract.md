# Ownership contract: Multi-domain core simulation abstractions and egress boundaries

Primary responsibility: CareOps core architecture.

The owner is responsible for designing `careops-core` and refactoring `careops-ed` without breaking existing behavioral contracts or test suites.
Any changes to Kairos crates must follow the submodule workflow (`libs/kairos` development branch, separate commit and pin update).

Follow [agent engineering](../../agent-engineering.md) and [single-maintainer harness](../../harness/single-maintainer.md).

## Composition revision ownership

The coordinator owns this planning revision and dependency joins. Runtime workers
receive narrow hashed packets; this document is not authority to change upstream
code, pins or accepted statuses. Follow [CC-01–CC-08](../../design/composition-contracts-20261020.md).
Kairos mechanics require their existing owner and exact-revision qualification;
healthcare semantics and site profiles remain in CareOps. Cross-repository writes
use separate leases and commits. Preserve parallel audit ownership.
