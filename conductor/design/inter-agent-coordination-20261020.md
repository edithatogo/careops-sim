# Proposal reconciliation and integration — 21 October 2026

Repository scope: careops-sim and its Kairos upstream; the separate careops
portfolio repository is outside this integration. This record replaces the
unverified draft assertion that all agents and workspaces were already aligned.

## Inputs preserved and combined

- Original careops-sim checkout at `fc4fdd6`: 36 modified/untracked planning files
  preserved in a local tar archive and per-file SHA-256 manifest before integration.
- Corrected roadmap branch at `7b671c3`: CC-01–CC-08, R/A/B plans and C-R0–C-R7 mapping.
- Audit branch at `66ce412`: PR173 addendum and current main `8ca0e5e` lineage.
- Kairos audit `fc971d5` and composition `d0637aa`, including registration `0157d0c`.

The audit writer handed off clean worktrees and released its claims. Existing
Kairos runtime reservations remain protected. Integration uses isolated branches;
original dirty workspaces are retained as recoverable draft sources, not dispatch
sources for this reconciled plan. Review the PR head and bind that exact revision.

## Conflict decisions

| Conflicting proposal | Reviewed resolution |
| --- | --- |
| Automatic pin advance to f0d30e2 | R0–R3 preserve qualified Q/C/IO/recovery; compare divergent history and retain the pin unless a candidate qualifies |
| AllOf/AnyOf atomic team allocation | A3 distinguishes event waits from atomic grants, with contention/cancel/rollback or a labelled approximation |
| Pair-only maternal/infant model | A5 and Kairos67 use typed one-to-many links and independent lifetimes; domain meaning remains here |
| Transfer sink equals completed care/birth | A4/B5 require explicit transfer ownership and separate completion callbacks |
| Generic hazards and catalogue of distributions | A1/B1 and Kairos69 require age/covariate/draw semantics and analytic controls before justified families |
| Fixed universal priority categories/targets | Versioned jurisdiction/site policy sources; classification, target, scheduling and safe interruption remain distinct |
| All historical milestones certify physical compute | R0/R2 classify contract evidence separately and require actual device and multi-host receipts |
| Stale task totals and unconditional ready-to-implement labels | 225 tasks, 54 milestones, 8 tracks; 80 new tasks remain pending; reviewed packets precede dispatch |
| Broad upstream ownership and implicit all-of-66 prerequisite | Exact-file handoffs and consumed capability gates; pending registrations confer no runtime authority |

The source design scopes (international profiles, community/private boundaries,
transport, independent progression and surgical/birthing consumers) are retained
in corrected specs and [composition contracts](composition-contracts-20261020.md).
The latest parser adds actionable rejection of malformed/empty numbered phases.
[PR173 mapping](../evidence/pr173-roadmap-integration-20261021.md) connects the audit
to task IDs; the generated catalog is rebuilt from the corrected plans.

## Publication and acceptance

Combining planning commits does not close implementation tasks or promote a pin.
Local tests and hosted checks are recorded in [merge readiness](../evidence/merge-readiness-20261021.md).
The integration history, rather than an assertion of simultaneous workspace sync,
identifies the reviewed content. Older evidence records describe their dated state.

## PR194 compatibility repair — 2 November 2026

Chat `01a19b9f-69e7-7353-9b75-445d5653feb4` owns the isolated managed
worktree `<PR194_REPAIR_WORKTREE>` on `codex/pr194-compatibility-checks`, based
on `0320767`. The original integration worktree and private hook registrations
remain with chat `01a19709-fd7b-7a61-a1e7-1ed85239eba0` (terminal handoff).
Scope: allocation conformance adapter, current-source mutation bindings and tests,
parent/fuzz locks where required, and this repair's Conductor evidence/status.
No native objects, authentication renewal, Kairos source or submodule pin changes.
Next operation: verify staged-runtime conformance against retained pin `0741026`,
then locked workspace, context, mutation and policy gates; independent review
and hosted checks remain required before acceptance.

PR194 repair source is published at `c786fb3`; ordinary hooks reported zero
findings. Independent reviewer `review_pr194` found no actionable source issues.
The original integration checkout remains clean at `0320767`; its local branch
and private bindings were not advanced. This repair's publication fast-forwards
only the existing PR remote ref. Its final source and check receipts are retained
in [the repair closeout](../evidence/pr194-compatibility-20261102/README.md).
R11.1 remains with the Polars owner; A7 remains gated on birthing B7.4. No new
phase, native activation, hardware, clinical or operational acceptance is granted.

## PR191 recovery after PR194 merge — 2 November 2026

Chat `01a19b9f-69e7-7353-9b75-445d5653feb4` reuses its clean managed worktree
`careops-pr194-checks/careops-sim` on `codex/pr191-context-recovery`. PR194
merged at `672d3e8`; its original owner checkout stays at `0320767`. The archived
Polars owner `01a1876f-9b26-71e0-b55c-daeb9d9a1335` retains the clean
`careops-r103-remote/careops-sim` checkout and private bindings at `96e8cc8`.
There is no native service or authentication scope. This writer owns the bounded
merge/conflict repair, workflow/tests, mutation bindings and recovery records
listed in [the packet](../evidence/pr191-context-recovery-20261102/packet.json).
The accepted main packet is imported intact through a normal merge. Next dispatch
operation is exact-head publication, independent review and hosted qualification;
no R11.2 phase acceptance or PR191 merge follows automatically.
