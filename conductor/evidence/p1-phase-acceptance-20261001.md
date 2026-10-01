# P1 DES evidence phase acceptance — 2026-10-01

## Decision

P1 accepted as a traceable DES source-evidence and explicit-gap inventory phase. P1.1–P1.3 deliverables and P1.4 review are joined in the working tree. This phase acceptance does not assert that empirical inputs are sufficiently identified for a calibrated ED profile or simulation.

## Evidence basis

- P1.1 and P1.2 coordinator receipts document bounded primary-source claims and acquisition gaps.
- P1.3 populated 61 records across six DES families. Each record remains unknown or deferred; no generic values, fitted distributions, or reference defaults were promoted.
- P1.4 reconciled all six catalogue families and 21 evidence references, manually reproduced the diagnostic TAT source row, and retained the AIHW residual and missing raw-fragment limitation.
- Independent review accepted the P1.4 reconciliation at the same bounded level and verified catalogue counts, evidence hashes, the diagnostic XML cell and the 61-of-101 scope boundary.
- Executed gates: `python3 tools/tasks.py check`, `python3 tools/context.py check`, `python3 tools/mvp.py check`, `python3 tools/validate_des_catalogue.py`, P1.4 packet-check, result JSON parsing, and `git diff --check` passed. Current outputs are recorded in P1.3/P1.4 receipts. No Rust runtime, fit, seed, synthetic profile, or clinical validation was required or run for this source-evidence phase.

## Carried limits and next work

P1 contains 61 of 101 registered parameter IDs. The other 40 IDs address ABM/spatial/measurement/experiment/optional scope and remain for later phases. The 3,180 difference between the national annual presentation total and displayed triage subtotal remains unexplained. Time-resolved arrivals, joint case mix, routing denominators, capacity/calendars, primitive service times, interfaces, cleaning, handover, boarding, and patience/censoring remain acquisition gaps. P3 must establish fitting, dependence, identification and bounds; P4 must create reviewed generic fixtures; P5 joins the example profiles to the runnable E2/C5 model and calibration validation gates.

P2 may proceed independently from P1 under the plan; P3 is gated on both P1 and P2. Next serial task is P2.1. No Cairns values, patient data, later care-domain tracks, visualisation, or GitHub publication are authorized/required by this phase receipt.
