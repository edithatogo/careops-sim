# Specification

CareOps consumes Kairos as an independent Rust dependency. Generic engine
mechanisms remain upstream; CareOps owns physical/clinical movement policy,
pathway/resource classifications, domain games and model assumptions. Resource
allocation/use is separate from HEOR valuation and Voiage VOI. Reuse the generic
Rust [**Paignia**](https://github.com/edithatogo/paignia) packages through an optional
adapter; do not add generic solvers to careops-core or the Kairos kernel. The
maintainer supplied the existing empty repository; bootstrap and transfer of
`paignia-core`/`paignia-solve` from reimbursement are pending. Preserve provider
T17 IDs and select actual immutable source at dispatch; package and consumer
qualification are separate. R76.5 remains outside runtime-only qualification.
T17.09 qualifies a first real static application and T17.10 a second distinct
application; CareOps is an optional candidate, not a required consumer.

Initial registration inputs: parent main `672d3e8` and gitlink `07410266bf54b6f73c59db2f8cbfbc5697c53734`,
core allocation/transfer contracts and Kairos Track 114. Historical `44261eb`
retention is not the current pin. New Kairos main is a candidate only after the
actual consumed API is checked. No current pin/dependency/runtime change here.

Outputs: thin public API adapters; one installed consumer; physical movement,
usage and paired economic-outcome fixtures. A proposed HEOR module begins as
an explicit domain interface/worked example; extract a package after a second
real model demonstrates shared valuation semantics. Do not put prices/QALYs
in the generic allocator or infer opportunity cost from accounting expenditure.

Acceptance: actual Rust execution, replay/error/unit checks and a consumer-owned
immutable pin decision; no host fallback, copied engine, private workspace
coupling or source-metadata claim of native acceptance. Existing A7/birthing,
clinical, hardware and release obligations remain separate and open as recorded.

The [bounded plan](plan.md) is the numbered status source; [reuse task cards](reuse-task-cards.md)
decompose its existing parents, preserving all 414 IDs/statuses. The
[package decision/source audit](../../design/careops-library-boundaries-20261108.md)
keeps careops-core in this repository, identifies current implementation gaps and
records the inspected a44a8c9 parent and retained 0741026 provider. Reconcile new
source before dispatch. The required small companion files link to these rather
than duplicate dossiers. Routine leaves need a diff and
focused result; public/numerical/replay milestones receive specialist review.
No repeated user approval is needed for authorized reversible implementation.
