# Bounded reuse task cards

These are pending decompositions of the existing R75–R77 tasks, informed by
[A0–A7](../careops_core_domain_abstractions_20261020/plan.md) and the
[package decision/source audit](../../design/careops-library-boundaries-20261108.md).
The numbered plan remains the status source; these cards add no task catalogue
or accepted capability. Historical A0–A6 receipts are not reopened. The cards
settle follow-up obligations without silently inheriting wider acceptance.

## Dispatch rules

Each card is one output with normally at most five files and three focused
commands. Use the existing task/lease/context harness once at dispatch. Bind
actual base, hashes, exact public signatures and test oracles then; proposed
outputs below do not exist merely because they are listed. Run native checks
with the repository's qualified Rust toolchain and the selected provider from
R76.0. Missing upstream behavior stops only its consuming card. Routine workers
implement settled contracts; the coordinator resolves scientific/API choices.
No new approval panel or duplicate evidence dossier is introduced.

Cards with the same parent serialize if their files overlap. Parent prerequisites
and release joins remain those in metadata; the dependencies below additionally
order sibling packets. Do not mark a parent complete until its claimed scope and
all corresponding cards pass. Optional Paignia R76.5 never becomes a prerequisite
of R76.6/R77. A7.1's birthing B7.4 join remains independent of R75 preparation.

## R75.1 — Inventory and upstream handoff

### R75.1.package-map — specialist; supports A0.3/A6/A7

- Input: the inspected public modules, Cargo manifests, actual Kairos gitlink and
  package decision. Dependency: none.
- Output: `conductor/design/kairos-consumed-api-inventory.md` and
  `conductor/design/careops-library-boundaries-20261108.md` (at most two files).
- Steps: record consumed symbols/features and package direction; classify each
  exported module; bind ED-specific defaults/identity encodings to their domain
  adapters and specialty modules to explicit consumers.
- Check: `cargo test --locked -p careops-core --test allocation_kairos_conformance`.
  Record the exact tested provider; inspecting upstream main alone is insufficient.
- Done: no reverse Kairos-to-CareOps dependency, no claim that an existing
  package is independently installable, and no new healthcare repository yet.

### R75.1.runner-handoff — specialist; supports A0.2

- Input: `crates/careops-core/src/experiment/`, ED `batch_runner.rs`, logistics
  test and the selected Kairos experiment interface. Dependency: package-map.
- Output: `conductor/design/experiment-lifecycle-handoff.md` and the consumed API
  inventory. This CareOps card changes documentation only; Kairos implementation
  requires its own owner, worktree and bounded packet.
- Steps: separate neutral indexed-replication/seed/cancel/checkpoint/publication
  contracts from ED summaries; compare the Kairos ensemble runner; name the
  upstream library-facing interface and a later thin CareOps facade replacement.
- Checks: `cargo test --locked -p careops-core --test experiment_runner_contract`;
  `cargo test --locked -p careops-core --test logistics_consumer`.
- Done: preserve 1/2/4-worker and recovery contracts, including incomplete output
  and duplicate-index negatives. Do not depend on the Kairos CLI to run a library.
  The upstream owner must split implementation/consumer replacement into separate
  small packets; a documented handoff is not a completed extraction.

## R75.2 — Shared runtime and scientific contract

### R75.2.clock-hazard — specialist; supports A1/A4/A6

- Input: unit-interval progression, custom pathway streams, ED scheduler, transfer
  time semantics and candidate public APIs. Dependency: R75.1.
- Output: `conductor/design/kairos-domain-adapter-contract.md` and
  `conductor/design/pathway-time-rng-contract.md` (two files).
- Steps: specify one scheduler/event authority, time units and event ordering;
  choose and version discrete-interval versus continuous-time hazard semantics;
  define covariate-change/residual-hazard, horizon/censoring and observation rules;
  fix named stream ownership and old-to-new replay compatibility expectations.
- Checks: `cargo test --locked -p careops-core --test pathway_failing_cases_tests`;
  `cargo test --locked -p careops-core --test policy_classification_tests`.
- Done: finite/invalid-rate handling, zero-hazard and analytic competing-risk
  references, stale generation/tie behavior and no future-outcome policy access
  are explicit. An administrative pause cannot stop biological progression unless
  the declared model says so. Record the LCG-versus-SplitMix documentation mismatch
  as pending correction; this contract card does not change a numerical algorithm.

## R76.1 — Package seam and installed consumption

### R76.1.ed-graph — local; supports A0.3/A6.1

- Dependency: R75.2 and R76.0; fixed graph behavior is an input, not worker choice.
- Prospective files: core `src/pathway/adapter.rs`, core
  `tests/configurable_pathway_adapter_tests.rs`, ED `src/pathway.rs`, ED
  `tests/pathway_conformance.rs`, and `conductor/design/ed/e0-assumptions-and-boundary.md`.
- Steps: move the canonical ED graph builder/defaults to the ED package; keep
  generic graph execution independent of ED names; update explicit callers and
  fixtures without introducing a core-to-ED dependency.
- Checks: `cargo test --locked -p careops-core --test configurable_pathway_adapter_tests`;
  `cargo test --locked -p careops-ed --test pathway_conformance`.
- Done: the same declared ED graph behavior passes and an arbitrary non-ED graph
  still executes. Any public API removal has an explicit migration note.

### R76.1.ed-encoding — local; supports A5/A6.1

- Dependency: ed-graph; bind byte-exact identity fixtures before moving helpers.
- Prospective files: core `src/identity.rs`, core `tests/identity_contract.rs`, ED
  `src/identity.rs`, ED `tests/identity_relationship_contract.rs`, and the consumed
  API inventory. All source/test paths are under the corresponding package.
- Steps: move ED source-person/admission encoding adapters out of the shared
  healthcare API; keep person/episode/cardinality semantics reusable.
- Checks: `cargo test --locked -p careops-core --test identity_contract`;
  `cargo test --locked -p careops-ed --test identity_relationship_contract`.
- Done: source-key encoding/domain separator, duplicate admission and readmission
  results remain explicit and tested. No silent identity re-keying or lifetime
  coupling to ECS handles; no circular crate dependency.

### R76.1.installed — local after candidate selection

- Dependency: ed-encoding and an actual qualified provider surface; R76.0 owns
  root manifests/locks/gitlink. Installation is tested outside the provider tree.
- Outputs: existing plan's proposed `tests/fixtures/kairos-consumer-smoke/`
  Cargo manifest, generated lock and `src/main.rs` (three files). Resolve an
  immutable package/Git source and feature set, not a mutable branch/path alias.
- Checks: `cargo generate-lockfile --manifest-path tests/fixtures/kairos-consumer-smoke/Cargo.toml`;
  `cargo run --locked --manifest-path tests/fixtures/kairos-consumer-smoke/Cargo.toml`.
- Done: scheduling/entity/RNG/resource behavior actually executes. Bind a second
  narrow external-consumer packet under R77.1 for the CareOps package artifact
  once its R76 adapters exist; that packet must run without ED checkout/private
  profile assumptions. A path-only workspace run is not installed acceptance.

## R76.2 — One execution authority and live movement

### R76.2.rng-adapter — local; supports A1/A6

- Dependency: R76.1 and the versioned R75.2 contract.
- Files: `crates/careops-core/src/pathway/streams.rs`,
  `crates/careops-core/tests/pathway_failing_cases_tests.rs`,
  `crates/careops-core/tests/configurable_pathway_adapter_tests.rs`, and
  `conductor/design/pathway-time-rng-contract.md` (four files).
- Steps: replace the custom LCG implementation with qualified Kairos streams;
  correct documentation; preserve independent progression/service/routing names
  and implement the settled version/migration behavior. No silent legacy core.
- Checks: the two existing core test targets named above.
- Done: same-version replay, independent streams and checkpoint round-trip are
  covered; changed samples are identified as the declared migration, not hidden
  behind weakened equality checks. Add a separate provider packet if a required
  stream/checkpoint API is missing.

### R76.2.pathway-events — local after the specialist contract

- Dependency: rng-adapter and qualified scheduler/hazard APIs.
- Files: core `src/pathway/adapter.rs`, `src/pathway/progression.rs`,
  `tests/configurable_pathway_adapter_tests.rs`, `tests/pathway_failing_cases_tests.rs`
  and `conductor/design/pathway-time-rng-contract.md` (five files).
- Steps: schedule arrivals, work completion and progression through Kairos;
  remove the second tick-driven execution authority. Keep clinical cause/acuity,
  covariate and policy interpretation here. Required manifest changes belong in
  the separately bound R76.0 candidate packet before this card.
- Checks: the two existing core pathway test targets; independent analytic
  references are added there under the selected model version.
- Done: one authority for virtual time and tie order; invalid/censored/stale
  events cannot change state; progression during waiting follows the declared
  semantics and does not reuse service draws. No bit-parity claim across models.

### R76.2.sparse-cost — local measurement; supports A1/A6

- Dependency: bind baseline before pathway-events; compare after its completion.
- Outputs: proposed `crates/careops-core/benches/pathway_sparse.rs`, its package
  bench declaration and a concise result in `conductor/design/pathway-performance.md`.
- Steps: freeze a long sparse horizon and a dense control with identical model
  semantics; measure event count, allocations/peak memory and elapsed time on the
  same toolchain/hardware. Separate RNG/model changes from execution changes.
- New command after creating the target: `cargo bench --locked -p careops-core --bench pathway_sparse`.
- Done: repeated baseline/candidate results and a stated workload/budget, including
  a regression decision. Never infer a speedup from asymptotics or one timing;
  do not compare different hazard models as if they were an optimization.

### R76.2.allocation — local after public interface qualification; supports A3

- Dependency: R76.1 and a qualified production allocator interface. Bind any
  manifest change in R76.0, not an unclaimed downstream edit.
- Prospective files: core `src/allocation.rs`, core
  `tests/allocation_kairos_conformance.rs`, ED `src/pathway.rs`, ED
  `tests/pathway_conformance.rs`, and the adapter contract.
- Steps: wire one actual ED admission/release path to provider grants and lease
  readback; retain local eligibility/priority/interruption rules. Separate any
  generic staged-coordinator upstream extraction into its own provider packet.
- Checks: `cargo test --locked -p careops-core --test allocation_kairos_conformance`;
  `cargo test --locked -p careops-ed --test pathway_conformance`.
- Done: capacity-one contention, stale lease revision, timeout/grant ties,
  cancellation and late-grant release preserve capacity. Staged acquisition is
  not represented as atomic all-or-none resource allocation.

### R76.2.movement — local; supports A4

- Dependency: allocation and the upstream arrival/cancel contract.
- Outputs: proposed core `src/movement.rs`, its `src/lib.rs` export and
  `tests/movement_adapter.rs` (the original R76.2 three-file leaf).
- New command: `cargo test --locked -p careops-core --test movement_adapter`.
- Done: one porter/patient journey arrives once; cancellation releases its lease
  and leaves location unchanged. CareOps supplies accessibility, responsibility
  and duration policy. No copied routing engine or compulsory coordinates.

### R76.2.live-feedback — local after shared interface is settled; supports A6/A7.1

- Dependency: pathway-events, allocation and movement; serialize overlapping ED
  paths. A7.1 still separately waits on the real B7 consumer.
- Prospective files: ED `src/pathway.rs`, core `src/ward_community.rs`, core
  `src/transfer.rs`, ED `tests/ed_ward_community_composition.rs`, and the adapter
  contract (five files).
- Steps: retain the single-bed oracle; add a bounded ED/ward/transport/community
  fixture whose events share one scheduler and whose downstream capacity changes
  affect upstream boarding during execution, not after ED completion.
- Checks: `cargo test --locked -p careops-ed --test ed_ward_community_composition`;
  `cargo test --locked -p careops-core --test transfer_contract`.
- Done: conservation after every event, independent location/claim/responsibility,
  delayed/rejected/expired offers, horizon censoring and actual feedback. A
  sequence of already-completed independent models does not satisfy this card.

## R75.3 / R76.3–R76.4 — Neutral usage and valuation

The existing R75.3 interface decision, R76.3 usage adapter and R76.4 two-valuation
fixture are already bounded cards in [plan.md](plan.md); keep their file budgets
and commands. R76.3 consumes actual grants/releases from the allocation card.
A7.3 owns the broader activity/cost/funding/outcome/equity ledger and fiscal join.

The interface must retain quantities/units, resource/run/branch/causal IDs, interval
boundaries, censoring and corrections. The fixture values the same usage under
two explicit perspectives; engine events remain identical. Include a bundled-rate
and labor double-counting negative and distinguish consumption from transfers.
Compare existing providers first; production HEOR extraction waits for a second
real economic model and does not become another generic CareOps core.

## R76.5 — Optional Paignia profile and consumer

R76.5 keeps its existing single design output and focused policy check. Bind the
chosen [Paignia repository](https://github.com/edithatogo/paignia) to actual
immutable `paignia-core`/`paignia-solve` provider source before a consumer
implementation. The repository was confirmed empty before bootstrap; seed
transfer from reimbursement is pending. Preserve the existing T17 IDs under
provider authority and retain old package names only as migration aliases.

A subsequent R76.5 consumer packet must specify one operational game's actors,
actions, information and locally owned payoffs, with independently derived expected
results and solver diagnostics. Bind at most five real adapter/test/manifest files
when the public contract is available. A static game runs without Kairos; a
claimed dynamic game additionally proves native dispatch, one clock/state/RNG and
replay through the optional adapter. Include no generic solver in CareOps.

T17.09 qualifies the first real static application and T17.10 a second distinct
application. CareOps is an optional candidate; two invocations of one harness do
not qualify. Its selection is not a CareOps release blocker. R76.5 stays pending
until its own profile and executed receipt exist, independently of runtime-only
R76.6/R77. The supplied repository's existence does not qualify registry
publication, supported algorithms or an installed dependency.

## R76.6 / R77 — Integration and independent reuse

At the existing coherent runtime join, review only the implemented adapter and
valuation scope and record unfinished cards explicitly. R77.1 retains exact
before/after provider qualification and adds the separately bound external
CareOps package-consumer packet described above. It must prove actual execution
through the installed package without a sibling provider checkout, private inputs
or undeclared features. Select the immutable artifact and generated lock at that
packet's dispatch; do not invent a published version now.

Use existing core/ED integration checks once; keep A7.1's real second-domain join,
empirical hospital validation and optional game acceptance distinct. Standalone
healthcare-repository extraction is reconsidered only after independent packaging
and the second healthcare consumer pass. R77 does not close A7 or publish a release.
