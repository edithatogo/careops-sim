# Planning and implementation workflow

## Current scope and authority

The user requested two reviewable specifications and phased plans. Record both
as proposed; no implementation checkbox is complete merely because a document
exists. Existing explicit product and Rust-native decisions provide this planning
context. The wider product draft has not received final setup approval.

Use [Kairos's workflow](../libs/kairos/conductor/workflow.md) for implementation
inside its repository: contract first, core first, bindings second. Each local
track has a spec, plan, metadata, index, ownership contract, test matrix, risk
register and handoff. Local `metadata.json` files are authoritative for these two
coordination tracks; `tracks.md` is the human-readable index. Upstream
`tracks.yaml` remains authoritative for upstream dependencies/status.

## Phase protocol

1. Review the specification and the phase's entry dependencies. Resolve contract
   questions in an ADR with the owning upstream track before implementation.
2. Add behavior-focused failing tests; implement the smallest compatible change.
3. Run targeted unit, integration, determinism and compatibility gates. Use
   representative benchmarks where behavior or performance can change.
4. Run Conductor review and the phase's manual verification. Record commands,
   toolchain, seed/configuration, input and artifact hashes, outcomes and gaps.
5. Mark tasks complete only with evidence. Synchronize affected upstream plans,
   handoffs, `tracks.yaml`, `tracks.md`, `phase-closeout.yaml` and `status.md`, plus
   readiness/map records if affected. Do not reopen unrelated work.
6. Run upstream Conductor phase/DAG and applicable domain gates. Commit Kairos
   changes on a development branch in the submodule, then update the parent pin
   only to the reviewed compatible commit. Remote publication is a separate
   action, outside this planning deliverable.

Every implementation phase ends with the checklist item
`Conductor — review and verify phase (workflow.md)`. A checkpoint is evidence of
that phase only; GPU hardware and distributed acceptance require their own runs.

## Scope management

Honor [the existing alignment policy](../docs/kairos-alignment.md). Preserve
public API/schema compatibility or record a versioned migration. Keep synthetic
ED examples here and general fixtures upstream. No extra Git submodules are
required for these enhancements. Dependency upgrades must record exact resolved
versions, MSRV/features and verification evidence.

Conductor's optional skills catalog was checked; its Firebase/GCP entries do not
apply to this Rust library planning increment.
