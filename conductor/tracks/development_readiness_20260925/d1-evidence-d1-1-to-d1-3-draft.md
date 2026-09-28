# D1.6 evidence reconciliation draft: D1.1–D1.3

Status: worker draft for coordinator review. This document does not accept D1.6
or change task/phase status.

## Evidence boundary

This reconciliation is limited to the D1.6 bound context and packet. The context
contains excerpts of the D1.1–D1.3 records and D1 plan, but not the complete
evidence documents or the contents of `conductor/current-state.json`. Therefore
this draft can assess the claims visible in those excerpts, but cannot verify
omitted sections, current-state agreement, or source artifact bytes against the
historical receipt logs. Those checks remain for the coordinator's integrated
phase review.

## Findings

| Task | Bound criteria and visible evidence | Gaps, qualifications, or stale-claim checks | Draft disposition |
| --- | --- | --- | --- |
| D1.1 bootstrap and consumer compatibility | The plan expects missing/wrong-tool handling, macOS ARM/Linux checks, consumer resolution, and explicit MSRV/current toolchain evidence. The bound integration receipt reports all four D1.1 leaves accepted, a 60-test parent suite, Rust 1.76 and 1.98.1 consumer checks, five-package native tests, and context/tasks/MVP checks. It identifies the Linux run as an Ubuntu 24.04.4 x86_64 Lima/QEMU TCG guest with read-only source. | The Linux result is virtualized guest evidence, not a bare-metal Linux run. This excerpt does not establish hosted Actions, Windows, or a clean-checkout reproduction beyond the stated guest run. The receipt's historical parent commit is `22b1204f8a357e776eb5229584f2b83b30d99535`; the excerpt does not show a later coordinator comparison to current state. | Evidence supports the recorded local gates with those environment limits. Do not generalize to Windows, hosted CI, or bare-metal Linux. Coordinator should compare the full receipt/artifact hashes and its historical commit to the integrated phase history. |
| D1.2 dependency/MSRV policy and implementation | The plan requires owner review of Arrow/TOML/Rayon and MSRV boundaries, explicit stable/canary lanes, reviewed version claims, and MSRV tests. The excerpt records the Kairos owner's approved policy, Track 13/30 review, Rust 1.76.0 and 1.98.1 workspace tests (228 tests/64 suites each), beta 1.99.0-beta.8 check, local validator/actionlint/diff checks, and parent integration at Kairos pin `339af4e7365e70ad7e67fe3e934e4fb215fbaf8b`. | Hosted Actions and Windows are explicitly pending. D1.2 historical parent integration is against `e0761a2563f2ae36b42c0f11f77660b423cdbdc6`, not the D1.1 receipt's earlier parent commit; the excerpts indicate sequential work rather than a direct contradiction, but do not establish the complete integration lineage to this D1.6 base. | Local owner-approved policy and recorded checks are supported. Keep hosted/Windows gates open and verify the exact Kairos/parent commit chain from full receipts before phase closeout. |
| D1.3 skill inventory/provenance | The plan expects an inventory of available skills and sourced/authored candidates for gaps. The excerpt records reviewed skill families, three locally authored candidates at `.agents/skills/`, source commit `e9b8c06081c48c7672e988f29eea6d276162acb0`, and no external-copy/license claim; it also says D1.4 must evaluate candidates before promotion. | The visible verification excerpt reports `python3 tools/tasks.py check` only. It says the checks ran after acceptance edits in a working tree, so this excerpt alone does not bind every final D1.3 artifact byte to the cited source commit or prove the final integrated skill hashes. D1.4 later reports a small supervised pilot and no skill promotion; this is consistent with the stated promotion gate, not evidence of technical held-out isolation. | Inventory/provenance claims are supported by the excerpt; promotion remains correctly off. Coordinator should verify final skill hashes/integration and retain the pilot's limited, non-hidden status. |

## Cross-cutting status and open gates

- The visible plan marks D1.1–D1.5 checked and D1.6 unchecked. A checked item is
  not, by itself, evidence that the current state and integrated artifacts agree.
- Current-state content was not included in this worker's bound context, so its
  next-action/commit references have not been reconciled here.
- D1.2 hosted GitHub Actions and Windows remain unverified pending the D2 gate.
- The related D1.4 excerpt explicitly keeps all candidate skills unpromoted due
  to the small supervised sample; it makes no hidden-evaluation or isolation
  claim. Do not describe its cases as technically held out from a shared-host
  worker.
- This draft does not establish the full D1 exit requirements: clean-checkout
  reproduction and phase-level manual verification still need coordinator
  review against the complete receipts and current integrated source.

## Coordinator review checklist

1. Compare `current-state.json`, plan checkboxes, and the integrated D1.1–D1.5
   evidence commits at the current D1.6 base.
2. Rehash/read back the complete D1.1–D1.3 artifact and receipt sets, especially
   the D1.3 candidate skill files after acceptance edits.
3. Preserve the recorded platform boundary: hosted Actions and Windows are not
   proved by local macOS/Linux-guest checks.
4. Record manual clean-checkout reproduction and any remaining failures before
   deciding D1.6 or D1 phase status.
