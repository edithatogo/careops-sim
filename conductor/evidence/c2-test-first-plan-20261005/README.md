# C2 test-first dependency repair — 2026-10-05

ADR-0008 and the plan/catalog/recipes separate C2.0 preparation from C2.1's full
runtime acceptance. All existing acceptance flags remain unchanged. Original
Macro/no-transit, zero-transit paired CRN/outcome and work-preservation oracles
remain gated behind actual C2.2/C2.3 implementation. No recipe grants dispatch.

`final-gates.json` binds actual planning checks and26task+13MVP tests to source
and log hashes. These are planning/harness checks, not simulation runtime proof.
`draft-gates.json` preserves failed historical package-style unittest invocation
and the stale80-count assertion; corrected invocation/count subsequently pass.
Old draft status describes that earlier state and is not the current result.

Independent c21_paired_preparation inspected the DAG, original requirements,
accepted flags, leaf joins/source hashes and coordinator boundaries; no remaining
semantic findings. It identified stale draft evidence, now resolved by a fresh
final record. Hosted parent CI and merge remain separate publication gates.
