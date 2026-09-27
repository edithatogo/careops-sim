# XML Deep Research prompt pack

Each XML file contains the shared instructions; paste the whole file into ChatGPT.
The prompts themselves are instructions; their referenced reports are evidence.

## Existing-session follow-ups

1. [F1: recover/audit model research](F1-existing-model-report.xml) — report (3)'s session.
2. [F2: recover/audit standards research](F2-existing-standards-report.xml) — report (4)'s session.
3. [F3: reconcile corrected inventories](F3-existing-session-reconciliation.xml) — after both bundles; attach the other session's outputs. If both reports share a session, use that session for all three.

If original artifacts cannot be recovered, reconstruction must be labelled. F3 cannot certify row-level completeness without the registers.

## Separate sessions

- [Find missing DES primitive inputs and reusable public examples](R2-separate.xml)
- [Spatial movement, staff task decisions and interruption evidence](R3-separate.xml)
- [Calibration and validation under partial observation](R4-separate.xml)
- [Minimum ED boundary and operational decision cases](R5-separate.xml)
- [Verify the minimal event and standards crosswalk](R6-separate.xml)
- [Queue/preemption reference semantics and test cases](R7-separate.xml)
- [Rust-native Metal and MLX assessment](R8a-metal-separate.xml)
- [Deterministic multicore, PDES and distributed research](R8b-parallel-separate.xml)
- [Single-person context and agent harness evaluation](R8c-harness-separate.xml)
- [Staged Rust CI, quality and security research](R8d-ci-separate.xml)

## Suggested order

Start F1/F2 now. R2 (DES evidence) and R3 (spatial/behavior evidence) can run in parallel from known gaps without waiting; their IDs remain provisional until F3. R4 methods and R5 boundary research can also start independently, with final choices reconciled against accepted inputs. R6/R7 are targeted contract support. R8a–d are separate optional engineering searches, most useful with current Kairos source attached.

Attach only the documents listed inside each prompt. Paths refer to the CareOps checkout, not files a remote ChatGPT session can automatically read. Research does not close Conductor tasks or replace local implementation verification.

## Research returned on 2026-09-27

F1/F2 returned reports 7/8 with failed original recovery and newly reconstructed
packages; R2/R3 returned reports 6/5. See [incorporation ledger](../ed-research-incorporation-20260927.md).
Obtain their actual artifact files before F3. Do not rerun completed broad searches
or treat the smaller reconstructions as authenticated originals.

## Further research received

Reports 9–12 returned F3 (gap audit only), R8b, R8c and R4.
See [integration decisions](../ed-research-incorporation-9-12-20260927.md).
Reports 10/11 contain extractable inline payloads; reports 9/12 still need their
linked bundles. Avoid another F3 run without the actual inventories attached.

## Final research themes received

Reports 26/27/28/29/30 return R8a/R7/R8d/R5/R6 respectively. Reports 23–25
are byte-identical to 12/11/10. See [decisions](../ed-research-incorporation-26-30-20260927.md).
All listed themes now have narrative responses. Retrieve the linked artifact
bundles and reconcile against local contracts before further broad research;
no F3 completeness claim is possible without the actual inventories.
