# C2.2 Macro and Zero Micro Flow bridge — 2026-10-06

## Result

Integrated Kairos source commit `ff764e7d57a577f39bab30fce752d0eb9b415623`
(parent source commit `54753b7d1f378cb647950a0cf18f88ebbfec2e13`) from
parent base `412ce3147306be46a0522d392740ae2d184655cf`.

The optional calibration `flow` feature now contains a private production
adapter using the experimental `FidelityAdmissionPermit` and actual
`FlowRuntime`. It accepts Macro and explicit Zero-transit Micro, owns the
Service stream and sampled duration across prepare/create/bind/submit failures,
retains the actual work handle for retries, submits timed Flow work, and checks
the returned request against both the saved request and work association before
issuing a success receipt. Micro Route fails before Service sampling until
C2.3 provides transit dispatch. Macro does not add a transit draw or event.

The parent coordinator reviewed the final source and integrated this bounded
instance after exact local verification. This is development implementation
proof only, not full C2.2 or C2.1 acceptance.

## Exact local verification

Commands ran from `/tmp/kairos-c2-flow-reconcile`, against source commit
`ff764e7d57a577f39bab30fce752d0eb9b415623`, with target artifacts under
`/tmp/careops-c22-flow-bridge-target`:

- `CARGO_TARGET_DIR=/tmp/careops-c22-flow-bridge-target rustup run 1.88.0 cargo test --locked -p kairo-ecs-calibration` — exit 0; 67 passed, 2 ignored.
- `CARGO_TARGET_DIR=/tmp/careops-c22-flow-bridge-target rustup run 1.88.0 cargo test --locked -p kairo-ecs-calibration --features flow` — exit 0; 74 passed, 2 ignored. Six focused tests exercised actual Flow-backed Macro/Zero submission, route and Service no-draw behavior, and retained create/bind/submit failures.
- `CARGO_TARGET_DIR=/tmp/careops-c22-flow-bridge-target rustup run 1.99.0 cargo clippy --locked -p kairo-ecs-calibration --all-targets --features flow -- -D warnings` — exit 0.
- `rustup run 1.99.0 cargo fmt --all -- --check` — exit 0.
- `git diff --check` — exit 0.

Logs are in this directory: `default-tests.log`, `feature-tests.log`,
`strict-clippy.log`, and `format.log`. Their SHA-256 values are respectively
`05eec6545f4b1888814ba61ee40563ea79c89c36e71d5a673f35e0c7b34c3867`,
`608748bde5ffacc00511376622f6ed2ea62fe3fe87a700cd41cb584496b23923`,
`d350c80163e5cb39ed25f92cd43270d5962a45c98cd196aae9da711cdd094011`, and
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.

The adapter uses deterministic fixture seed `19`. Contract input
`conductor/design/calibration/c20-admission-bridge-v1.md` matched SHA-256
`3a659df216fd1b035e0c561b5df566a73a34660289239dc947992502f36049f9`; the
prerequisite `c2.2-flow-submit-atomicity-20261006/README.md` matched its packet
receipt hash `285b75d55c8be41d98211aa8f59466f0f569768c318d2234bea4157ff415efae`.

## Remaining C2 work

C2.3 actual transit dispatch/progress, C2.2 callback receipts and checkpoint /
resume joins, the full paired C2.1 Macro/Micro runtime, hosted exact-head checks,
and parent/full C2 acceptance remain open. No stable API, portable checkpoint,
release, or clinical claim follows from this local slice.

## Corrected C2.2.flow-macro-zero local acceptance — 2026-10-06

The old recipe's blanket requirement that nonzero Micro routes reject before
Service sampling was stale against the frozen bridge contract and active C2.3
route plan. The coordinator corrected the recipe at parent commit `ab5bc9b`; an
independent read-only review confirmed that this leaf covers Macro and explicit-
zero Micro direct submission while nonzero Micro Route dispatch and accepted-
arrival reconciliation belong to `C2.3.transit-bridge`. The historical result
above remains evidence for its original source commit and does not satisfy the
corrected recipe by itself.

At Kairos pin `aecb28c7089b1861b72cbd96ce29fc2577ffe47f`, the coordinator
locally accepted the corrected leaf after full-package verification using only
Rust 1.99.0:

- `rustup run 1.99.0 cargo test --locked -p kairo-ecs-calibration -p kairo-ecs-abm --features kairo-ecs-calibration/flow` — exit 0; 186 passed, 0 failed, 4 ignored. The paired Macro/zero-Micro actual timed work/acquisition regression passed. Log `/tmp/c22-macro-zero-rust199-tests.log`, SHA-256 `fba23cd3d436d58379d695b7618c11909905609febdd4da5ad1e81d45ef18f1c`.
- `rustup run 1.99.0 cargo clippy --locked -p kairo-ecs-calibration -p kairo-ecs-abm --features kairo-ecs-calibration/flow --all-targets -- -D warnings` — exit 0. Log `/tmp/c22-macro-zero-rust199-clippy.log`, SHA-256 `f2066ca264b9a8598ecd436c70919e1efe1acea4098ad2a13b5d82d67bc049ec`.
- `rustup run 1.99.0 cargo fmt --all -- --check` — exit 0. Log `/tmp/c22-macro-zero-rust199-fmt.log`, SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.

The corrected recipe and fresh packet bind source, prerequisite and command hashes;
the prior C2.2.flow-submit-atomicity leaf proves cumulative scheduler-cap
preflight does not orphan a request. No separate adapter-level cap retry fixture
was added. This accepts only the C2.2 Macro/zero-Micro leaf: C2.2 parent joins,
C2.3 calibration transit-bridge acceptance and Track03 owner review, C2.1 paired
runtime, hosted gates, release and clinical acceptance remain open. Receipt:
`acceptance-rust199-corrected.json`.
