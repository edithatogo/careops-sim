# Ownership contract: Generic birthing suite simulation delivery

Primary responsibility: CareOps clinical domain delivery.

The owner is responsible for implementing the `careops-birthing` simulation crate using `careops-core` and Kairos engine primitives.
All empirical data must maintain source provenance (AIHW / Queensland Health public statistics).
Operating Theatre, Postnatal, and SCN internal dynamics remain strictly external to this crate.

Follow [agent engineering](../../agent-engineering.md) and [single-maintainer harness](../../harness/single-maintainer.md).

## Composition revision ownership

The coordinator owns this planning revision and dependency joins. Runtime workers
receive narrow hashed packets; this document is not authority to change upstream
code, pins or accepted statuses. Follow [CC-01–CC-08](../../design/composition-contracts-20261020.md).
Kairos mechanics require their existing owner and exact-revision qualification;
healthcare semantics and site profiles remain in CareOps. Cross-repository writes
use separate leases and commits. Preserve parallel audit ownership.
