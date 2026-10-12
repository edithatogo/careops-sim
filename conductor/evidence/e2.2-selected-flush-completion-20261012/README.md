# E2.2 Selected Flush Native Verification Evidence — 12 October 2026

## Summary

The native selected resource flush implementation (`flush_dirty_selected`) on `EdFlowAdapter` and its comprehensive behavioral test suite `crates/careops-ed/tests/flow_adapter_selected_flush.rs` have passed all required verification gates on stable Rust 1.99.0.

- **Source Commit:** `20142979ab41e539ae64a0455fd063e3754ae82b`
- **Reviewed Contract:** `conductor/design/ed/e2-flow-selected-flush.md`
- **Verification Results:**
  - `cargo test -p careops-ed` (all 113 unit and integration tests passed; 0 failed)
  - `cargo clippy -p careops-ed --all-targets -- -D warnings` (clean, 0 warnings)
  - `cargo fmt --check` (clean)
- **Verified Invariants:**
  1. `selected_flush_only_selected_dirty_resources`: two distinct Deferred resources contain real queued requests; selecting A grants only A; B remains queued and dirty; subsequent selection of B grants B.
  2. `selected_flush_selection_atomicity`: unknown resource reference and duplicate identities reject atomically with zero mutation to private adapter ledgers or Flow runtime.
  3. `selected_flush_pending_and_wrapper_parity`: repeated selection before actual flush schedules no duplicate key; matches full wrapper parity on identical runtimes.
  4. `selected_flush_space_offer_care_barrier`: space grant observed and verified through cancellation seam without prematurely consuming care work.
  5. `selected_flush_cancel_batch_and_budget`: global unclosed cancellation batch guard verified atomically.
