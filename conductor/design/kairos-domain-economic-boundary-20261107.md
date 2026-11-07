# CareOps, Kairos and economic model boundaries

Accepted direction, 7 November 2026; implementation pending in the
[Kairos domain contract track](../tracks/kairos_domain_contracts_20261107/index.md).

Kairos owns Rust time/events/state/RNG and generic allocation, usage, routing
and movement lifecycle. CareOps owns physical patient/staff movement, access,
porter/transport policy, pathways and calibration. A headless operational model
need not have coordinates. Reuse upstream petgraph/geo/rstar primitives through
public APIs; the consumer should not copy a second scheduler or route engine.

A resource request, resource occupancy and resource cost are distinct. Record
admission/release and quantities with units, run/branch/causal IDs; retain
censoring, cancellation and replay corrections. HEOR supplies the explicit
valuation model: currency/price year, perspective, horizon/discounting, utility,
uncertainty and opportunity-cost assumptions. Allocation itself is common to
operations research and economics, not a HEOR-only concept. Keep financial
transfers and revenue distinct from economic cost and avoid inclusive-bed-rate
plus labour double counting.

Start with a separate valuation interface and a Rust worked example. Compare
existing cost-data and economic providers before adopting or creating a package;
extract generic shared valuation only after a second real model demonstrates it.
Voiage consumes paired strategy/draw outcomes or net benefits for VOI; CareOps
must preserve units and pairing instead of making Voiage own the simulation.

UOGTO defines game semantics. [**Paignia**](https://github.com/edithatogo/paignia)
is the maintainer-supplied generic Rust game repository, confirmed empty before
bootstrap. Transfer of `paignia-core`/`paignia-solve` from the reimbursement
incubator is pending; preserve existing T17 IDs under the provider authority and
select actual immutable source/artifacts at dispatch. Package publication and
consumer qualification remain separate. A
Kairos adapter is optional for dynamic games. CareOps owns particular operational
games, policies and payoffs. A matrix solver does not need coordinates or a
mandatory event-engine dependency. R76.5 remains optional and separate from
runtime-only R76.6/R77 qualification. T17.09 qualifies the first real static
application and T17.10 a second distinct application; CareOps is an optional
candidate, not a mandatory consumer.

[Package decision and source audit](careops-library-boundaries-20261108.md) record
why the existing careops-core stays in this repository, the actual duplicate
clock/RNG mechanisms and ED defaults to address, and the scoped A6 composition
evidence. [Task cards](../tracks/kairos_domain_contracts_20261107/reuse-task-cards.md)
map follow-up into existing A/R75–R77 ownership without a parallel programme.

Use independent immutable package/commit dependencies after qualification.
Existing submodule paths are current build machinery, not the public contract.
The inspected current gitlink is `07410266bf54b6f73c59db2f8cbfbc5697c53734`; older
`44261eb` retention records are historical. No pin advances in this plan.

Routine work uses one bounded task and focused result; review numerical/public
contracts and coherent integrations, with no new approval board or duplicate
status ledger. Existing clinical and release acceptance applies only where a
claim requires it, not to every local implementation edit.
