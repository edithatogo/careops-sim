# Ownership contract: Repository status reconciliation and Kairos upgrade

Primary responsibility: CareOps coordinator; Kairos engine maintainers.

The coordinator owns top-level Conductor registry consistency and submodule pin coordination.
No clinical simulation logic or existing ED data fixtures may be altered during this track.

Follow [agent engineering](../../agent-engineering.md) and [single-maintainer harness](../../harness/single-maintainer.md).

## Composition revision ownership

The coordinator owns this planning revision and dependency joins. Runtime workers
receive narrow hashed packets; this document is not authority to change upstream
code, pins or accepted statuses. Follow [CC-01–CC-08](../../design/composition-contracts-20261020.md).
Kairos mechanics require their existing owner and exact-revision qualification;
healthcare semantics and site profiles remain in CareOps. Cross-repository writes
use separate leases and commits. Preserve parallel audit ownership.
