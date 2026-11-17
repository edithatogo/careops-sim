# ICU liaison simulation — execution plan

## IL0 — source, authority and provenance

- [x] IL0.1 Confirm careops-sim as simulation owner, existing Conductor
  submodule commit 7a5c560 (v0.4.1), and no need for a duplicate vendor.
- [x] IL0.2 Inventory ANZICS and study definitions; redact/curate earlier
  working-chat decisions and external source links. No raw chats or private
  workbook or local hospital material into this public repo.
- [x] IL0.3 Establish separate source status and immutable synthetic fixture.

## IL1 — bounded probabilistic economic simulation

- [x] IL1.1 Implement versioned validation and editable JSON parameters,
  keeping all values and uncertainty ranges out of Rust source.
- [x] IL1.2 Use existing Kairos deterministic RNG for triangular-uncertainty
  PSA and report discharge-capacity, early-readmission, gross and net results.
- [x] IL1.3 Add unit oracles for reproducibility, no-effect, negative-effect,
  accounting identity and invalid provenance/ranges.
- [ ] IL1.4 Verify exact PR-head hosted format/Clippy/native tests and
  independent review; retain all failures and update receipt.

## IL2 — local data and cost qualification (not started)

- [ ] IL2.1 Establish an authorised CHHHS profile outside public Git; reconcile
  APD timestamps, ward-movement denominator, completeness and case mix.
- [ ] IL2.2 Reconcile local incremental costing and CareOps Costing contract;
  explicitly exclude overlapping readmission/exit-block periods and add
  ward/health outcome changes only with evidence.
- [ ] IL2.3 Calibrate uncertainty, perform validation, sensitivity and a
  prospective causal design; independent economic/clinical/governance review.

## IL3 — temporal event simulation (not started)

- [ ] IL3.1 Bind qualified inputs to Kairos event scheduling and capacity
  constraints; build ICU/ward/staff/transfer events with shared clock and
  deterministic seeds, rather than writing a new DES core.
- [ ] IL3.2 Qualify occupancy, safe transfer, backlog and throughput with
  synthetic fixtures followed by approved observational validation.
- [ ] IL3.3 Independently compare PSA and DES outputs, reporting non-overlap
  and model structural uncertainty; release only after clinical acceptance.

Existing rand/rand_distr and Kairos upgrade tracks retain their ownership;
this pilot reuses current pins and does not dispatch their work.
