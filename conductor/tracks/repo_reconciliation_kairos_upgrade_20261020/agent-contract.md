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

## PR173 incorporation — 21 October 2026

[Integration record](../../evidence/pr173-roadmap-integration-20261021.md) maps
C-R0–C-R7 to existing task IDs. Resolve overlapping Kairos 67/68/69 and existing
owners at exact-file/interface scope; registration and an empty lease store do
not authorise takeover. Bind only the required capabilities of 64/66, with public
API/checkpoint/schema compatibility proof. Broad catalog reservations remain
conservative planning bounds; runtime leaves need separate reviewed packets.
Do not edit the parallel PR173 addendum, enhancement/readiness links or PR266.
