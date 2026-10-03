# C0 recovery and current integration acceptance

Accepted: C0.1–C0.4 architecture/contracts only, at current Kairos `3aa9ae42111ecd66d16efcfa76fe850b88520498`.
Historical C0 receipts and immutable source snapshots retain their original pins.
This receipt supersedes current integration claims in the older receipts.

Recovered only 29 upstream contract/source-snapshot documents from f872ad0 and
selected parent C0 design/evidence files from 2c33ece. No stale gitlink, current
state or task catalog was copied. Independent read-only review verified 18
snapshot files against declared hashes and found no architecture-only blocker.
No C2 manually supplied candidate-observation fixtures were recovered.

Fresh executed checks at `/private/tmp/careops-d2-main-acceptance-20261003`:
- Draft2020-12 meta-validation: pass. Schema SHA256
  `8c46db62f691f243385a4ebdf8a7a3d3670e2655dd0c3f82cba4335ced2a3842`.
- `python3 -m unittest discover -s tests -v`: exit 0, 349 tests, 31.085 s.
- `cargo test --locked -p kairo-ecs-arrow -p kairo-ecs-cli -p kairo-ecs-rng`
  in libs/kairos: exit 0; existing legacy serialization/parser/RNG only.
- Exact-commit native owner CI: Linux x86_64 and macOS ARM64 success,
  https://github.com/edithatogo/kairos/actions/runs/37082789051.

Rust/Cargo 1.98.1 on local macOS ARM64. No new random draws or empirical inputs.
Command logs/hashes and bound recovery packet are retained in the local evidence
bundle. Parent exact-pin and hosted checks are required before remote integration.

Boundary: no calibration runtime, real Arrow IPC, empirical fits or clinical
acceptance. Exact task/purpose/replication seed tuple encoding and draw position
remain a Track01 implementation prerequisite. Rust1.76 is declared policy only,
not a verified resolved graph. Arrow60/Rust1.88 is a dated candidate requiring
C1 dependency/feature review and actual builds; no package adoption occurred.
