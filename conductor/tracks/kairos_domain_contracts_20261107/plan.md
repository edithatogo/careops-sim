# Bounded implementation plan

All implementation is pending. Each leaf has one behaviour and at most five
owned files; bind base/context through the existing tools/tasks.py and lease
harness before dispatch. Commands below are run from the repository root with
the qualified runtime; semicolon-separated commands are independent checks, not
a requirement to execute a shell chain. Future test targets are declared outputs.
A local worker stops implementation on missing upstream APIs or unsettled scientific assumptions. A known defect may be documented as a red reproducer; pair it with its bounded fix before green integration, and never treat the reproducer alone as runtime acceptance.
No prototype or print-only result is accepted as native execution.

## R75 — Contracts

- [ ] R75.1 Reconcile the actual consumer pin and public symbols
  Executor: specialist. Requires: none.
  Files: `conductor/design/kairos-consumed-api-inventory.md`.
  Check: `git ls-tree HEAD libs/kairos; cargo test --locked -p careops-core --test allocation_kairos_conformance`.
  Acceptance: Read Cargo.toml, allocation.rs, transfer.rs and the actual pinned provider. Record public symbols/feature needs and existing evidence; no pin change or assumption that upstream main preserves them.

- [ ] R75.2 Settle the thin movement/resource adapter interface
  Executor: specialist. Requires: R75.1.
  Files: `conductor/design/kairos-domain-adapter-contract.md`.
  Check: `cargo test --locked -p careops-core --test transfer_contract`.
  Acceptance: Specify identity, time units, cancellation/lease ordering, stale topology and explicit unsupported capabilities. CareOps policy remains here; route/allocator mechanics remain upstream.

- [ ] R75.3 Settle one explicit HEOR outcome/valuation interface
  Executor: specialist. Requires: R75.2.
  Files: `conductor/design/heor-valuation-interface.md`.
  Check: `cargo test --locked -p careops-core --test capacity_contract`.
  Acceptance: Declare usage versus consumption/transfer, currency/price year, perspective/horizon/discounting, utility and double-counting rules. Compare existing provider contracts before proposing a new crate; assign subsequent small implementation leaves.

- [ ] R75.4 Conductor — review and verify phase (workflow.md)
  Executor: specialist. Requires: R75.1–R75.3.
  Files: `conductor/tracks/kairos_domain_contracts_20261107/handoff.md`.
  Check: `python tools/context.py check; python tools/tasks.py check`.
  Acceptance: One interface review resolves the public contract and economic assumptions for this batch; no runtime acceptance or extra approval board.

## R76 — Adapters and worked fixtures

- [ ] R76.0 Bind the candidate in an isolated consumer worktree
  Executor: specialist. Requires: R75.4 and a selected immutable upstream candidate.
  Files: `Cargo.toml`, `Cargo.lock`, `libs/kairos` (gitlink only), `conductor/tracks/kairos_domain_contracts_20261107/candidate-selection.md`.
  Checks: `cargo generate-lockfile`; `cargo metadata --locked --format-version 1`.
  Acceptance: In an isolated parent worktree, select the exact candidate through either the consumer manifest or its existing submodule gitlink and regenerate/review the lock. Record resolved provider SHA/features before any consumer test. The parent gitlink selection has its own coordinated provider checkout; do not edit provider source under a parent claim. Keep this candidate on the branch until qualification; subsequent R76/R77 commands must run against this recorded configuration, not the old pin. Promotion merges the already-tested change.

- [ ] R76.1 Execute a standalone immutable Kairos consumer
  Executor: specialist. Requires: R76.0 and actual upstream candidate capability.
  Files: `tests/fixtures/kairos-consumer-smoke/Cargo.toml`, `tests/fixtures/kairos-consumer-smoke/Cargo.lock`, `tests/fixtures/kairos-consumer-smoke/src/main.rs`.
  Checks: `cargo generate-lockfile --manifest-path tests/fixtures/kairos-consumer-smoke/Cargo.toml`; `cargo run --locked --manifest-path tests/fixtures/kairos-consumer-smoke/Cargo.toml`.
  Acceptance: Generate the fixture lock during implementation. Exercise scheduler/entity/RNG/resource behaviour outside the provider checkout. Use a qualified immutable Git dependency or built package; the existing relative path smoke alone cannot close installed acceptance.

- [ ] R76.2 Map one domain journey to upstream scheduled movement
  Executor: local. Requires: R76.1 and upstream arrival/cancel contract.
  Files: `crates/careops-core/src/movement.rs`, `crates/careops-core/src/lib.rs`, `crates/careops-core/tests/movement_adapter.rs`.
  Check: `cargo test --locked -p careops-core --test movement_adapter`.
  Acceptance: New target: one porter/patient journey arrives once, cancellation releases the lease and leaves location unchanged. CareOps supplies accessibility/duration rules; no duplicate routing engine.

- [ ] R76.3 Export neutral usage from actual resource admissions
  Executor: local. Requires: R76.1 and upstream usage contract.
  Files: `crates/careops-core/src/usage.rs`, `crates/careops-core/src/lib.rs`, `crates/careops-core/tests/usage_adapter.rs`.
  Check: `cargo test --locked -p careops-core --test usage_adapter`.
  Acceptance: New target: queued time is not occupancy; release/promotion has adjacent intervals; resource/run/branch/causal identities and units survive export; duplicate/retracted usage is not summed.

- [ ] R76.4 Apply two declared valuations to the same usage fixture
  Executor: local. Requires: R75.3 and R76.3.
  Files: `crates/careops-core/tests/heor_valuation_fixture.rs`, `tests/fixtures/economic-usage-v1.json`.
  Check: `cargo test --locked -p careops-core --test heor_valuation_fixture`.
  Acceptance: New test/example only: hand-calculated porter/bed/consumable cost under two named perspectives; changing valuation never changes engine events. Include censored occupancy and bundled-rate double-counting negative. This does not create a production HEOR engine in careops-core.

- [ ] R76.5 Define one optional domain game policy profile
  Executor: specialist. Requires: R75.2 and generic game package contract.
  Files: `conductor/design/game-policy-profile.md`.
  Check: `cargo test --locked -p careops-core --test policy_classification_tests`.
  Acceptance: Specify actors/actions/information/payoffs and UOGTO mappings for one operational decision; reference generic solver/result capabilities. Keep clinical rules/equilibrium interpretation local; no solver copy or mandatory game dependency.

- [ ] R76.6 Conductor — review and verify phase (workflow.md)
  Executor: specialist. Requires: R76.1–R76.5.
  Files: `conductor/tracks/kairos_domain_contracts_20261107/handoff.md`.
  Check: `cargo test --locked -p careops-core; python tools/tasks.py check`.
  Acceptance: Review the coherent adapter/valuation fixture batch once; record actual source/commands and only the supported behaviour.

## R77 — Consumer qualification

- [ ] R77.1 Qualify a candidate provider change against the ED consumer
  Executor: specialist. Requires: R76.6.
  Files: `conductor/tracks/kairos_domain_contracts_20261107/provider-qualification.md`.
  Check: `cargo test --locked -p careops-core --test allocation_kairos_conformance; cargo test --locked -p careops-ed`.
  Acceptance: Record before/after immutable provider revisions and real model replay/errors/features from the isolated R76.0 candidate configuration. Keep the accepted main pin until this consumer passes; source-only compatibility or an old-pin test is insufficient.

- [ ] R77.2 Conductor — review and verify phase (workflow.md)
  Executor: specialist. Requires: R77.1.
  Files: `conductor/tracks/kairos_domain_contracts_20261107/handoff.md`.
  Check: `python tools/context.py check; python tools/tasks.py check`.
  Acceptance: Accept only the demonstrated adapter/fixture scope. Publish a separate reviewed parent pin change if justified; preserve unfinished domain/hardware/clinical obligations.

## Dispatch and evidence

Use the existing preparation catalogue, not a new dispatcher. R-prefix broad
reservations are only preparation hints: bind the explicit files above before
writing and never mutate libs/kairos under a parent-only claim. A routine leaf
returns diff/commit and focused command results; review coherent contract,
numerical/replay and integration milestones. Reuse unchanged source-bound
results and update ledgers only when status changes. External provider acceptance
is checked at the consuming leaf, not a blanket blocker on independent design.
