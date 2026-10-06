# C2.2 policy admission permit — local implementation receipt

This local integration advances C2.2 at Kairos `f1f510e6c9ff9c3923b63ad733c05c19c495b46c`, based on the locally integrated routing/progress branch. It accepts only the C2.2 policy/permit instances recorded here; C2.2’s provider-to-Flow owned-state bridge and the full C2.1 paired runtime remain open.

## Implemented

- Relocated the existing private FidelityPolicy/FidelityAdapter lineage and precedence suites into Kairos crate-internal unit tests, preserving the original cases while they exercise the real implementation alongside a crate-private Flow validation helper.
- Added `FidelityAdapter::prepare_admission` and a non-Clone `FidelityAdmissionPermit`. The permit holds the mutable adapter borrow from policy resolution through binding, validates the real actor and Flow runtime, and freezes the selected decision.
- Permit binding validates same-runtime identity, duplicate status, actual task owner, Pending state, and exact sampled original duration. Errors return the same permit before adapter mutation. Success binds runtime lineage and the immutable decision once.
- Added real Flow tests for invalid actor retry, colliding foreign runtimes, wrong owner and duration retry, non-Pending work, duplicate admission, and successful binding.

## Verification

From `/Users/doughnut/Documents/careops-sim-session-worktrees/kairos-c22-admission-permit-20261006` at source commit `f1f510e6c9ff9c3923b63ad733c05c19c495b46c`:

- `env CARGO_TARGET_DIR=/tmp/careops-sim-c22-policy-target rustup run 1.76.0 cargo test --locked -p kairo-ecs-des` — 246 passed, 0 failed, 1 existing ignored, across 26 targets.
- `env CARGO_TARGET_DIR=/tmp/careops-sim-c22-policy-target rustup run 1.99.0 cargo clippy --locked -p kairo-ecs-des --all-targets -- -D warnings` — pass.
- `env CARGO_TARGET_DIR=/tmp/careops-sim-c22-policy-target rustup run 1.99.0 cargo fmt --all -- --check` — pass.
- `git diff --check` — pass before the source commit.

The focused and full DES results include the C2.1 mode and lineage tests; no calibration dependency or stable public export was added. The bound result receipts and raw test, Clippy, and formatting logs are preserved beside `acceptance.json`. These are local native checks, not hosted CI or final C2 runtime acceptance.

## Remaining C2 work

C2.2 still needs the production provider-to-Flow preparation/create/bind/submit lifecycle with retained streams and actual acquisitions. C2.3 dispatch still needs TransitContext, actor carrier, actual progress/control scheduling and arrival claim. C2.1 paired Macro/Micro runtime and interruption joins, hosted exact-head gates, portable checkpoint codec, and ED operational acceptance remain open.
