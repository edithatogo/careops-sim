# C2.4 accepted synthetic implementation — 10 October 2026

Source `63a72aa563258777686ee3d2e25794628daae4ec`, reviewed tree
`fd58d564b5709f75f102a1c09b02b1e1e9e8e444`, is adopted by the parent.

- Local Rust1.99 locked native join:656 passed/zero failed/five ignored; strict
  Clippy and formatting returned0 separately. Nineteen lifecycle cuts plus generic
  context and nonzero owned-RNG proofs reconcile21 local child runs.
- Hosted run37964052376:66 successful checks/two documented conditional skips.
  Both compiled synthetic merge92fe34ea trees equal the reviewed source. Each
  native host records592/0/5 and21 child artifacts; all42 hosted input/exit/control
  traces independently reconcile. No skip is counted as a pass.
- Original C2.0/1/2/3/5 oracles, staff cleaning/reservation, actual mode boundaries,
  CRN pairs, travel/work/wait and Q4 interruption accounting are reconciled.
- The final compound cut restores paused mid-edge travel and active same-owner/
  resource Macro work/Fidelity state. Activity finishes without advancing travel;
  its20 wall ticks are counted once before route resume and separate Micro service.

[Coordinator acceptance](acceptance.json), [final owner dispositions](owner-dispositions-final.md),
[original task crosswalk](task-crosswalk-final.md), and [hosted readback](hosted/acceptance.json)
record the bounded C-02 implementation/C2.4 review. Earlier preparation files and
failed attempts remain historical, rather than being rewritten as passes.

Stable API/release, empirical/clinical/ED validation, general runner/C5 recovery,
full C-06 worker-study equivalence and G1/G2/MVP remain separate. No full upstream
track or operational clinical acceptance is declared.
