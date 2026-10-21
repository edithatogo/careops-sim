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
