# C3 accepted synthetic implementation — 10 October 2026

C3.1–C3.4 and C-03 are accepted for experimental private implementation at Kairos
`da327cf263ff6e4507f48f597c201bb269680d51`. The parent pin advances from accepted C2
`63a72aa563258777686ee3d2e25794628daae4ec`; C1/C2/C4/Q4 evidence is preserved.
[Draft PR244](https://github.com/edithatogo/kairos/pull/244) remains stacked and open.
This is development acceptance, not a merge or release.

## Delivered

- Canonical observed replay with knowledge-visible historical snapshots and explicit resource claims.
- Isolated native Flow/C2 probes, fixed observed anchors, unclamped predictions and bounded virtual-time execution.
- Strict/diagnostic missingness, feasibility and signed residuals; execution fidelity stays independent of replay role.
- Complete bounded checkpoint inventory and actual fresh-process native transit/overdue-work recovery without resampling or duplicate endpoints.
- Native physical IPC/Parquet residual and metric round trips through existing C4 schemas.
- Slow-walk positive residual, unchanged p1 image, interrupted transit/work, exact-budget/anchor, duplicate-event and walk/work identifiability fixtures.

Three GPT-6-luna workers implemented disjoint leaves and performed independent
reviews; the coordinator owned contracts, integration and acceptance. Review fixes
and failed attempts remain preserved. [Task crosswalk](crosswalk.json) maps each
C3 requirement to actual source oracles.

## Executed evidence

Rust/Cargo 1.99.0 only. Local full calibration: 351 passed/5 named ignores;
release shadow: 26 passed/1 child-entrypoint ignore; default calibration:
255 passed/4 named ignores; DES/ABM/CLI: 348 passed/1 named ignore. Strict all-target
Clippy, Rustfmt, owner docs, phase/DAG and clean-tree gates pass. The subprocess
entrypoint is explicitly invoked by the passing parent; other ignores retain
pre-existing scope and are not counted as passes.

[Final hosted run38048446666](https://github.com/edithatogo/kairos/actions/runs/38048446666)
passed on Ubuntu x86_64 and macOS aarch64. Final PR checks: 47 success, two explicit
conditional skips (Q5.2 branch-only canonical regression, Codecov OIDC upload).
Each native C3 suite passed 26 tests and five child invocations. Independent
reviewers matched source tree `63c767fc08687ed3f9947182cc5fe5b9304741ee`, all listed
source hashes and Rust 1.99 hosts. The CI merge checkout differs in commit ID
from the PR head but has the identical tracked tree.

Baseline, transit-restored and work-restored results and checkpoint bytes match
across both hosted architectures and local runs. The source ledger is frontier2
(anchor0, observed target1); native route3 + work7 completes at10 with residual+9.
Slowing the route moves completion10 to13 without rewriting source anchors. This
is a bounded synthetic observation, not general architecture or clinical proof.

- [Acceptance receipt](acceptance.json), [checks](checks.json), [run](run.json), [PR binding](pr.json).
- [Two-host readback](readback.json), [manifest](archive-manifest.json), [retained hosted evidence](hosted-evidence.tar.gz).
- [Local source-bound evidence](../../../libs/kairos/conductor/evidence/c3-local-qualification-20261010/README.md).
- [Upstream scoped owner closeout](../../../libs/kairos/conductor/evidence/c3-hosted-qualification-20261010/README.md).

All 40 retained final-evidence archive members were read back against their hashes.
Independent reviews by c3_semantics_preparation (Ubuntu), c3_recovery_preparation
(macOS), and c3_fixture_preparation (task crosswalk) found no remaining C3 source
oracle gap. Pending statements in earlier dispatch/local records are historical
snapshots and are superseded by this acceptance.

Next: C5 reproducible fitting, CLI and CPU worker scheduling. Public API review,
clinical/empirical and ED validation, whole MVP/G1/G2, release and merge remain open.
Per-inventory metadata caps do not claim a bound on total process RSS. The
repository-local learning loop retains an independently reviewed artifact-reservation
correction; it is advisory and grants no unattended authority.
