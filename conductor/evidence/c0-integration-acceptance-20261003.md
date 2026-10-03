# C0 integrated acceptance — 2026-10-03

Disposition: **accepted for local architecture and calibration contracts**.
C0.1–C0.4 are complete at this scope; C1–C6 runtime work remains open.

## Integrated source and review

The coordinator integrated the existing accepted C0 work from parent branch
`codex/c01-calibration-architecture` at `003bc6f` onto current parent base
`888757c`, preserving the accepted P0–P4, E0 and D2 preparation work. Historical
C0 receipts retain their original source identities and executed failures.
The adopted Kairos pin is `f872ad0e6bddd879578569d9d5e9453bef77f71a`, on development
branch `codex/c0-reviewed-contracts`. Its difference from the previous pin
`a71adfd48f42d7c4d04bcb034aad09295c004f40` is restricted to 29 Conductor contract,
source-snapshot and evidence documents; no runtime or Cargo source changed.

Reviewed outputs include the reusable calibration/Arrow/CLI placement ADR,
adapter hooks and resolved VVUQ contract reference; twelve versioned record
variants for trace/mapping/provenance/fidelity/probe/residual/metric semantics;
source and method inventory; synthetic identifiability/split/tolerance oracles;
and the Arrow feature/MSRV ownership handoff. The previous phase review records
the hand-worked arrival/triage/transit/bed/discharge example, including distinct
episode end and censored physical departure.

The reviewed parent contract/schema and upstream contract documents preserve
observed versus predicted ledgers, occurrence versus knowledge/source clocks,
unknown versus zero, censoring versus completion, active work versus elapsed
intervals, domain policy ownership and explicit future seed-algorithm approval.
The accepted sampling and input-pack work does not establish empirical values.

## Current executed verification

Working directory: `/private/tmp/careops-c0-integration`; Python 3.14.7;
Conductor pin: `7a5c560a4fdf5297be58594cc37527eb12790272`.

- Draft 2020-12 meta-validation of `calibration-v1.schema.json` passed. Schema
  SHA-256: `8c46db62f691f243385a4ebdf8a7a3d3670e2655dd0c3f82cba4335ced2a3842`.
- `python3 -m unittest discover -s tests -v`: **336 passed**, exit 0, 20.513 s.
  Log: `.artifacts/c0-integration/python-tests-final.log`; SHA-256
  `39558bf2e465a707211d38e6ff8f15980b7689c80e6a6778598564d336559456`.
  The earlier integration run passed 335/336; its sole stale-recipe-line failure
  was fixed by updating source locators after the plan acceptance link moved.
  Historical CSV provenance failures do not recur at this current source.
- `python3 tools/tasks.py check`, `python3 tools/mvp.py check`,
  `python3 tools/context.py check`, and `git diff --check`: passed on the final
  integrated state. Task/MVP metadata was regenerated/reconciled without
  changing leaf objectives or dependencies.

## Remaining boundaries

C0 specifies contracts; it does not implement the Rust calibration crate, real
Arrow/Parquet interoperability, a runner, seed-purpose derivation, empirical
fits, clinical validation, or local feed conformance. Track 01 exact seed review
and C1 runtime/dependency decisions remain required. C1.1 remains gated on D2.5.

The exact Kairos and Conductor objects were hydrated from existing local
checkouts for this integration. Configured public remote URLs are preserved;
remote fetchability, hosted checks and publication are not accepted here.
