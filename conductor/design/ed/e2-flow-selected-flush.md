# Selected-resource Flush for the shared ED adapter

Status: reviewed-candidate design; implementation and runtime acceptance pending.

## Purpose and authority

Preserve the E1 terminal barrier in `grant_terminal_phase`: grant ED spaces, decide offers and cancel superseded waiting care, then grant clinical/care/transfer work. The current all-dirty Flush cannot express this barrier while all waiting requests remain real native requests. The terminal validator prohibits space and transfer resources from aliasing each other or clinical/care pools. Selecting resources therefore preserves this separation without changing native queue ordering.

This is a small parent adapter extension. Keep one Flow runtime, the existing native StableKey queues, existing phases and budgets. No Kairos API change, extra clock, outside-engine shadow queue, or new ED policy in engine code is required. Root coordinates acceptance and dispatch; this private proposal is not source authority.

## Interface

Add to `EdFlowAdapter`:

```rust
pub fn flush_dirty_selected(
    &mut self,
    resource_ids: &[&str],
) -> Result<Vec<EventId>, AdapterError>;
```

Keep `flush_dirty()` with its existing signature and behavior. Both methods share a private implementation. The wrapper selects all adapter resources; the new method selects a caller-specified subset. Resource names are existing validated adapter identities, not a new identity namespace. Use existing error variants where suitable; bind exact labels to the accepted adapter before dispatch.

## Selection and atomicity

Validate the entire selection before scheduling or changing any ledger:

- Every supplied resource must exist. Reject an unknown name even if another selected resource is dirty and valid.
- Reject duplicate names rather than silently coalescing caller mistakes.
- The existing global pending-cancellation guard remains: any unconsumed cancellation batch rejects Flush, including a selection of unrelated resources.
- A known clean resource is a no-op. A resource with an outstanding Flush at the current tick is also a no-op.
- Candidates are selected dirty resources without an outstanding current-tick Flush, in canonical resource-name order. Caller slice order must not change event order.

Preflight the whole candidate batch using existing Flush and scheduled-event limits and existing native resource/tick checks. Failed selection or budget preflight must leave the native image, adapter counters, dirty set, pending Flush set and event map unchanged. Do not discover a later invalid selected resource after scheduling an earlier one.

Schedule candidates with the existing `schedule_allocation_flush(resource, now, 60)` call and the existing event/Flush accounting. Unselected dirty resources remain dirty. Selected resources are removed from the dirty ledger only when their actual Flush dispatch is consumed, exactly as today. Repeated calls before that dispatch must not schedule duplicate native `(resource, tick)` keys.

## Budget and phase compatibility

This method partitions existing dirty obligations; it does not create a new event class or require a multiplier. Each scheduled Flush consumes the same existing Flush budget and actual scheduler scheduled-event accounting. Outstanding-key uniqueness and lifecycle-generated dirty obligations remain unchanged.

Submit remains priority 50, Flush 60 and waiting cancellation 70. The parent marker cursor governs parent-origin phases only. Native causal priority-0 completion/release events remain legitimate; do not impose a global monotone-priority rule. No phase replay is introduced by the selection API.

## E1 use

At a terminal barrier, the future migrated executor will:

1. Consume completion/ingress effects and submit eligible intents as real native requests.
2. Flush only ED-space resources and consume their actual manual grant observations.
3. Run offer policy once for that tick. An allocated offer cancels superseded queued care; consume the complete cancellation batch before any subsequent Flush.
4. Flush the separate clinical/care/transfer resources and consume native work observations.
5. For abandonment after the first grant pass, decide the full abandonment batch, consume all cancellations/releases, then grant again without replaying offer allocation.

Offer allocation's strict increasing-tick rule remains. This extension only supplies resource selection; offer, abandonment and clinical policy stay in the E1 executor. The complete E1 migration still requires its own output/seed/censoring parity acceptance.

## Bounded ownership and predecessor

Prospective owned source paths:

- `crates/careops-ed/src/flow_adapter.rs`
- `crates/careops-ed/tests/flow_adapter_selected_flush.rs` (new)

Protect all other foundation and transit source/tests. The transit worker currently owns the adapter; prepare and dispatch this extension only after transit acceptance and release, bound to its actual accepted HEAD and source hashes. No overlapping writer is permitted. A private method refactor inside the adapter is allowed only to share existing preflight/scheduling behavior; no service or transit API changes.

## Required actual-runtime proofs

1. `selected_flush_only_selected_dirty_resources`: two distinct Deferred resources contain real queued requests. Selecting A grants only A; B remains queued and dirty. A later B selection grants B.
2. `selected_flush_selection_atomicity`: valid-plus-unknown and duplicate selections reject before any mutation. Exhausted batch budget also rejects before scheduling. Place this proof as a unit test inside the owned adapter module: it may inspect private native checkpoint and adapter dirty/pending/event ledgers and counters. No public full-Flow checkpoint accessor exists, and FoundationSnapshot does not expose these ledgers; do not add a persistence accessor. The exhausted-budget negative may use an explicitly artificial lowered internal limit only after constructing a real scheduled baseline. Valid public bounds need not make this failure reachable.
3. `selected_flush_pending_and_wrapper_parity`: repeated selection before actual Flush schedules no duplicate key; after dispatch, a new legitimate dirty obligation can Flush again. On identical real runtimes, the all-dirty wrapper and full resource selection have matching dispatch/image/counter results.
4. `selected_flush_space_offer_care_barrier`: real manual ED-space claim and queued care work use distinct resources. Space selection grants space without consuming care. The resulting offer decision cancels care before work Flush; care never grants or progresses. Include a no-offer control where care subsequently grants.
5. `selected_flush_cancel_batch_and_budget`: a pending cancellation anywhere retains the existing global Flush rejection. After consuming it, the selected batch succeeds with exact scheduled-event/Flush accounting, including empty/clean selections.

Tests must exercise `step` and actual `FlowDispatch.records`, not detached selectors. Preserve all existing foundation/transit tests. Use existing private image helpers or public snapshots as available; do not add a persistence API solely for tests.

## Verification and trigger

Bind a narrow packet after accepted transit: Rust/Cargo 1.99, strict all-target careops-ed Clippy, full careops-ed tests and exact owned-file edition-2021 format. Format before expensive source gates. The execution budget is the initial attempt plus at most two reviewed corrections (three failed development executions maximum), each stopped for root review with receipts retained; any out-of-scope need stops the packet. Freeze source and evidence for independent review before commit.

After selected-Flush acceptance, bind the serial E1 migration packet. Subsequent staff32, interruption24 and space32 fixture bindings may run on disjoint owned paths after the shared policy seams are accepted. The existing continuation heartbeat is sufficient; no hook or duplicate automation is needed.

## Proof placement clarification

The integration test file exercises supported public selection and actual dispatch behavior. Full private-state atomicity, artificially lowered budget-boundary rejection and exact internal ledger comparisons belong in unit tests inside the owned `flow_adapter.rs`. Artificial limits are proof controls, not new production rejection conditions or weakened public bounds. Retain a positive actual-scheduling baseline for each such control.

## Dispatch binding

Root approves this independently reviewed contract after native transit acceptance at9690b2a405ab3e62ba5947e1da308f0362941dcc. Source packet must bind current HEAD. Earlier proposal status is superseded. Native private proof helpers are available in flow_adapter/transit.rs; do not edit that protected file.
