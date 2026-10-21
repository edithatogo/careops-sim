# Related repository interfaces

Planning inventory: 20 October 2026. Ownership checked through local Git remotes.
No source pin, runtime dependency or deployment changes follow from this note.

| Repository | Role | Interface / gate |
| --- | --- | --- |
| `edithatogo/careops-sim` | Healthcare operations | CC-01–CC-08 healthcare semantics, profiles and cross-service fixtures |
| `edithatogo/kairos` | Rust simulation mechanics | Existing owner tracks; exact-revision qualification before pin promotion |
| `edithatogo/careops` | Portfolio delivery | Independently runnable simulation, immutable candidate pin and module qualification; tenant workflows retain separate acceptance |
| `edithatogo/nhra_gt` | Fiscal/strategic policy model | Versioned offline policy-input/operational-summary exchange before coupled runtime |
| `edithatogo/NwauCore.jl` | Thin Julia CLI wrapper | Verify downstream calculator identity, version, schema and price year; wrapper is not a formula engine |

## Fiscal exchange acceptance

Define period-indexed policy inputs: scenario ID, policy version, effective
period, units, currency/price year, capacity constraints and implementation lag.
Return aggregated activity, costs, outcomes and uncertainty with source/model/input
hashes. Separate funding receipts from economic costs. Avoid counting the NHRA
model's queue response again when CareOps supplies an operational response.

First qualify offline round trips and rejection of mismatched units, periods and
schemas. Later co-simulation must define clock ownership, feedback lag, convergence
and cancellation. Sharing files or a seed does not establish a valid coupling.
Do not rewrite Python/Julia projects simply to make CareOps Rust-native.

## Integration constraints

The inspected NHRA checkout has extensive staged/unstaged changes and deleted
Conductor files. Reconcile its canonical working state and active ownership before
direct integration; preserve those changes rather than reconstruct deleted plans.
Keep its adapter requirements here until that boundary is resolved.

NwauCore's README identifies a CLI wrapper. Calculator/formula ownership and
qualification remain unresolved; do not invent a second NWAU calculator or use
funding weights as clinical utility.

The CareOps portfolio already has a simulation composition track. Reuse it for
candidate qualification; do not create another portfolio framework. Other owned
repositories are not dependencies merely because their subject matter is related.
