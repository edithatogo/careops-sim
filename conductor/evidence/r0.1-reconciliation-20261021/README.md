# R0.1 — current dispatch and historical evidence reconciliation

21 October 2026. Scope: **careops-sim** and read-only Kairos identity/ownership.
The separate **careops** portfolio repository is outside this change.
Status: R0.1 accepted after independent review; R0 phase acceptance remains pending.

## Immutable inputs and preservation

- Merged careops-sim baseline: `ea42fcf8e588318d792e05481ea20c732fa70da0` (PR173).
- Merged Kairos planning baseline: `55b20e77eb9ca0df5a95b9e82c4fb815d3919a87` (PR266).
- R0.1 start/coordination commit: `8dff0fa`; this is not a new runtime qualification.
- Runtime gitlink remains `44261eb5cf8275f132f63a3396903d116d701dbe`;
  Conductor tooling remains `7a5c560a4fdf5297be58594cc37527eb12790272`.
- Original careops-sim checkout remains at `fc4fdd61cdbf105ae13c23b909cc0d9806a22222`
  with its dirty proposals preserved. All 36 archived file hashes match both the
  archive bytes and the original checkout at readback. No original file was rewritten.
- The existing local archive `20261021-careops-sim-proposals.tar.gz` has SHA-256
  `8dc6f584fd686d516ae9e86ef2c58d82443a22a39e01f526692e6370eab19ff1`.
  It remains under `${PRIVATE_HOME}/.codex/reconciliation-snapshots/` with its
  original manifest. The [preservation manifest](proposal-preservation.json)
  retains per-file archive/source/integrated hashes; do not overwrite later edits
  from this snapshot. Restore only into a separate recovery directory.

The [source readback](source-readback.json) records actual commands, exits and
SHA-256s for retained logs, including both merged PRs. Merged planning is the
binding source; original dirty copies are retained historical inputs. The
[conflict decisions](../../design/inter-agent-coordination-20261020.md) supersede
automatic pin promotion, enum-as-atomic-allocation and unqualified hardware claims.

## Decisions and remaining discrepancies

| Surface | Reconciliation | Evidence / limit |
| --- | --- | --- |
| Parameter registry, metadata and handoff said proposed/P0-next | Align with historical P0–P5 generic catalogue coverage closeout | [P5.4 acceptance](../p5.4-phase-acceptance-20261015.md); no new hospital/clinical/Cairns qualification, original spec-approval flag retained |
| ED registry said E5–E8 not yet done; E8 receipt said repository 100% complete | Record all historical checked tasks, keep capability reconciliation active | [E8.4](../e8.4-phase-acceptance-20261017.md) is dated; [merged capability audit](../../design/kairos-capability-reconciliation-20261020.md) disputes physical scope; R0.3/R2.3 still required |
| Queue registry repeated old publication/next-work state | Link exact dated Q5.4 scope; remove it as a current dispatch instruction | [Q5.4](../q5.4-completion-20261005/README.md) and retained hosted receipt are historical experimental evidence |
| Resume claimed planning local-only; old c*_qualification flags appeared current | Record immutable merges and explicitly mark old qualification objects as historical snapshots | Embedded source hashes and receipt contents unchanged; current qualification remains R0.2–R3 |
| Index/research text competed with current dispatch | Resume + unchecked DAG are authoritative for preparation; old research handoffs are historical | No new research batch or D4 restart inferred; B0.1 remains independently eligible |
| Execution guide and README retained old totals | 8 tracks, 54 milestones, 225 tasks; MVP82 parents/258 leaves | Counts are planning structure, not operational acceptance |

The original 145 accepted task flags and their dated receipts are preserved.
R0.1 acceptance covers this reconciliation only, never full R0 or capability
qualification. The catalog now has 146 checked tasks (145 historical plus R0.1)
and 79 pending tasks; R0.2 is the next serial preparation candidate. R0.2 (current symbols/features/lineage), R0.3 (E6/E8 exact-source
classification) and R0.4 (phase review) remain pending. R1 freezes the regression
baseline; R2/R3 must qualify a capability-preserving candidate before any pin move.
The existence of Kairos main `55b20e7` is not compatibility evidence for that pin.

## Ownership and gates

This chat owns shared R0.1 plan/status/catalog edits. The integration chat
`01a1596c-77d5-7a13-b663-ff43e2ffb034` confirmed no overlapping R0 writes and performs
independent read-only review. Its R0.2 preparation cannot advance accepted status.
Kairos Track65 game-theory and Track66 CLI reservations remain protected; expired
leases do not permit takeover. No Kairos source, registry or worker inputs change.

R0.1 uses an isolated parent worktree, a bounded hashed packet and exact-path
advisory claim. The ten-path sizing exception is synchronization of coordinator
surfaces plus generated output; no Rust, fixture, dependency, CI or pin change.
The packet is retained locally at `/tmp/careops-r01-packet.json`; lease tokens are
not published. Runtime/release/clinical/hardware gates remain unchanged.

## Verification

The first resume check in the fresh worktree detected missing submodule checkouts.
Initializing both recorded gitlinks resolved those errors; no pin was changed.
Subsequent command receipts and negative controls are recorded in
[validation](validation.json). Planning checks do not execute or certify a new
simulation backend. Hosted checks on this R0.1 change remain separate.

## Independent review

The integration agent in chat `01a1596c-77d5-7a13-b663-ff43e2ffb034` reviewed the
concrete diff, P5 boundary and immutable merge readbacks, reran context/task/MVP
checks and independently verified the archive plus all 36 original-file hashes.
It reported no blocking findings and accepted bounded R0.1, explicitly retaining
R0.2–R0.4 as pending. The coordinator then recorded only R0.1 and regenerated the
catalog. Final integrity and unchanged-history checks are in validation.json.
This review is not a GitHub approval, hosted CI result or R0 phase closeout.
