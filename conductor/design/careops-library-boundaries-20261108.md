# CareOps package boundaries and Paignia reconciliation

Direction authorized by the maintainer; source audit dated 8 November 2026.
This is a design and executable-work decomposition, not runtime qualification.
Implementation uses the existing [A0–A7 plan](../tracks/careops_core_domain_abstractions_20261020/plan.md)
and [R75–R77 plan](../tracks/kairos_domain_contracts_20261107/plan.md), with
[bounded task cards](../tracks/kairos_domain_contracts_20261107/reuse-task-cards.md).
All 414 existing task IDs and their accepted/pending states are preserved.

## Decision: strengthen the package already present

`crates/careops-core` is the reusable healthcare operations/composition package.
Keep it in careops-sim now; improve its public interface and dependency boundary
before considering a standalone repository. `careops-ed` and future specialty
packages supply domain pathways. CLI/UI, site profiles, empirical inputs and
application orchestration remain above those packages. A second repository is
considered only after an independently installed package and a second real
healthcare consumer demonstrate reuse. A synthetic logistics runner is useful
for generic experiment machinery, but is not the second healthcare consumer.

The shared healthcare layer owns person/episode meaning, clinical responsibility,
service eligibility, healthcare capacity interpretation, staffing and escort
constraints, observed policy state and outcome mapping. Specialty birth/theatre
rules are named domain modules, not generic Kairos requirements. Avoid a universal
workflow framework or mandatory dependency on every specialty package.

## Ownership and dependency direction

| Owner | Responsibility | Consumption boundary |
| --- | --- | --- |
| Kairos | Rust time/events/state, ECS lifetime, RNG, neutral resource and movement mechanics, replay and reusable experiment execution | CareOps consumes qualified public APIs at an immutable package/commit; Kairos never depends on CareOps |
| CareOps shared package | Healthcare identity, responsibility, eligibility, capacity/staffing interpretation, service composition and clinical observation semantics | Domain adapters translate to Kairos; no second scheduler or RNG engine |
| CareOps domain packages | ED/birthing/ward/theatre/community processes, policies, outcomes and calibration assumptions | Depend on the shared healthcare package; service interaction uses explicit contracts |
| CareOps applications/profiles | Site/jurisdiction packs, data ingestion, study/scenario orchestration, CLI/UI and reporting | Keep local data and policy defaults out of the generic engine |
| Paignia | Generic game representation, strategic algorithms and solver diagnostics | Optional CareOps adapter; static games do not require Kairos; dynamic games have one execution authority |
| UOGTO | Ontological terms, identifiers and semantic mappings | Supplies versioned meaning to game/domain adapters, not the execution engine |
| HEOR/valuation providers | Explicit economic valuation of resource use and outcomes | Consume observations with units/perspective/time basis; never own resource allocation or simulation time |
| Innovate / Voiage | Innovation/adoption models / value of information | Optional application-level integrations with versioned inputs and paired outputs; no core dependency cycle |

**Paignia is the chosen name and repository:**
[edithatogo/paignia](https://github.com/edithatogo/paignia), supplied by the maintainer.
The repository was empty at confirmation; bootstrap and transfer of the generic
Rust seed from `new-drug-reimbursement-game` are pending. The renamed packages
are `paignia-core` and `paignia-solve`; retain original `uogto-game-*` names only
as migration/provenance aliases. Existing T17 IDs follow the provider work into
Paignia; reimbursement becomes a consumer, not a second solver owner. Select the
actual immutable source/artifact at dispatch. Repository existence does not
qualify a published package or an installed CareOps dependency.

CareOps defines an operational game's actors, available information, actions,
constraints and payoff interpretation. Paignia supplies a supported solver and
result diagnostics. A game profile and its executed consumer receipt remain
optional R76.5 work. T17.09 qualifies the first real static application and T17.10
a second distinct application; together they establish the two-consumer join.
CareOps is an optional candidate rather than a required application. Ordinary
CareOps runtime qualification does not require a game solver.

## Source snapshot and remaining gaps

The audit inspected CareOps main
`a44a8c9f1461839c5306dbf44a1fa9a3b1981c50` (tree
`7dced13104140ef2a5a43ae3f949f88d22cd56a1`). Its Kairos gitlink is
`07410266bf54b6f73c59db2f8cbfbc5697c53734`. Kairos main
`e76b97be2ea3712645933d903822ff6381c8883e` was inspected separately for candidate
interfaces; its presence is not compatibility evidence for the retained pin.
Rebind source and public symbols before implementation. No algorithm, pin,
lockfile, package publication or runtime capability changed in this audit.

| Observed source | Implication and existing owner |
| --- | --- |
| `crates/careops-core/src/pathway/adapter.rs` has `for current_tick in 0..=self.horizon_ticks`, stage queues and patient scans | A1/A6 evidence does not establish one reusable event-driven runtime. R75.2 settles time semantics; R76.2 cards replace duplicate execution through Kairos and measure sparse-horizon cost |
| `pathway/streams.rs::SubsystemStream` is documented as SplitMix but its state update is an LCG recurrence | Record a pending documentation/implementation correction. Qualify a versioned Kairos RNG adapter; do not silently change replay or claim the correction has been made |
| `pathway/progression.rs::sample_event` samples at most one cause over a unit interval | A continuous-time `SurvivalClock` substitution is a model change, not byte-parity refactoring; define units, ties, censoring, covariate updates and independent reference cases first |
| Kairos candidate has `kairo-ecs-rng::{DeterministicStream, CompetingHazards}` and `kairo-ecs-des::SurvivalClock` | Reuse mechanics after actual pin/API qualification; clinical causes, covariates and acuity mapping remain in CareOps |
| `pathway/adapter.rs::build_ed_canonical_graph` and `identity.rs::ed_*` helpers with `careops.ed.key.v1` encoding live in careops-core | Move ED defaults/encoding adapters into careops-ed through bounded cards; preserve explicit source-key encoding and fixture behavior |
| `careops-core/src/experiment/` is generic and `tests/logistics_consumer.rs` exercises it | R75.1 hands the narrow lifecycle contract to existing Kairos experiment owners; reconcile `kairo-ecs-cli/src/ensemble.rs` instead of introducing another runner or a domain dependency on the CLI |
| `allocation.rs` coordinates staged claims; `allocation_kairos_conformance.rs` exercises Kairos readback; `kairo-ecs-des` is a dev-dependency | This is not production-wide resource integration or atomic bundle support. Qualify a production adapter and retain clinical admission/interruption policy locally |
| ED `src/lib.rs` and `src/pathway.rs` use Kairos Scheduler, while the A6 composition test runs ED then a single-bed ward/community oracle | Preserve the useful oracle; add live downstream-to-upstream feedback on one scheduler before claiming concurrent cross-service composition |

The existing A0–A6 receipts retain their scoped local acceptance. A7.1 still
requires B7.4 and an actual second healthcare domain; no new reuse claim follows
from renaming a package. Runtime tests, performance measurements and empirical
hospital validation were not performed for this source audit.

## Movement, resource use and economics

Movement records location/transit/arrival/cancellation. Allocation records
requests, grants, occupancy/consumption and release. HEOR interprets those
observations under an explicit valuation model. A porter journey can involve
all three without making movement or resource allocation a HEOR primitive.

Use neutral usage observations with resource/run/branch/causal identity, quantities,
units, interval boundaries, censoring and correction/retraction semantics. Keep
clinical access and transport policy in CareOps. Apply valuation separately:
currency/price year, perspective, horizon/discounting, utility and uncertainty.
Distinguish economic costs from payments and revenue; exclude bundled-rate double
counting. Two valuations of the same usage must leave engine events unchanged.
R75.3/R76.3–R76.4 and A7.3 already own this work. Compare existing providers before
extracting shared valuation after a second real economic model needs it.

## Delivery and maintenance

The task cards decompose existing parents into small packets with concrete files,
inputs and checks. They do not add a dispatcher, approval board or acceptance
ledger. One routine leaf returns its diff and focused result; public/scientific
contracts and coherent integration receive the existing review. Record observed
limitations rather than collecting ceremonial evidence. Automatic checks should
cover changed documentation links, task coverage, dependency direction, deterministic
fixtures and installed-package smoke; measured performance gates should use stable
baselines and declared budgets, not noisy one-run speed claims.
