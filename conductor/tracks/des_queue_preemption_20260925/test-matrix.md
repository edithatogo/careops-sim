# Automated test matrix — queues and preemption

Q0–Q4 have bounded implementation evidence; final Q4.5 integration is pending.
The table names are acceptance identifiers, not necessarily Cargo target names.
Q5 conformance, worker invariance, compatibility and benchmark joins remain
planned. Actual commands, source hashes and results belong in the phase receipts;
a checked design row does not substitute for those results. Each failed invariant
blocks its phase.

| ID / phase | Layer | Cases and automated oracle | Acceptance |
| --- | --- | --- | --- |
| queue_capacity_conservation / Q1–Q5 | Unit + property | Generated spawn/acquire/release/cancel/despawn/grow/shrink; active ≤ capacity; free+active=capacity; no double membership or dangling live leases | Q-01 |
| queue_generational_handles / Q1 | Unit | Recycle entity index; old owner/request/lease handles fail; repeated terminal operations have no effect; sequence/clock overflow rejects | Q-01/02 |
| queue_priority_fifo / Q2 | Unit | Ascending i32 priority, FIFO ties, rekey preserves sequence; shuffle dense component storage and retain output | Q-02/05 |
| queue_deadline_boundary / Q2 | Integration | Release at deadline in either insertion order, zero/past deadlines, first-grant deadline cleared; same tick timeout cannot grant | Q-02/04 |
| queue_same_tick_cancel_reprioritize / Q0 design; Q1/Q2 runtime | Golden trace + integration | Same queued claim gets cancel and reprioritize at one tick; equal scheduler priorities obey both insertion orders; differing scheduler priorities override insertion order; later operation observes success or `AlreadyTerminal`; claim priority remains distinct from Scheduler priority | Q-02/04 |
| queue_preemption_strategies / Q3 | Integration | Low(10) starts 0, high(2) starts 3; high ends 5; low ends 12 Suspend, 15 Restart, absent Abort | Q-03 |
| queue_victim_order / Q3 | Unit + property | Capacity 2+, mixed preemptibility, equal priorities, reprioritized active claim, nested preemption; unique deterministic worst eligible victim | Q-02/03 |
| queue_victim_tie_and_zero_time / Q0 design; Q1/Q3 runtime | Golden trace + integration | Equal-urgency eligible victims evict latest admission; zero-duration claim grants/completes once at one tick; configured per-tick Flow transition cap succeeds exactly at limit, counts delivered notifications, blocks the next transition/notification with a structured outcome, preserves pending work, cannot be reset by another call at that tick, and resets after advancing time | Q-02/03/04 |
| queue_completion_races / Q3 | Integration | Completion at interruption tick, old token dispatch/cancel failure, repeated release/cancel; zero remaining completes exactly once | Q-04 |
| queue_work_accounting / Q3 | Unit | Suspended waits excluded; restart discards attempt progress but retains cumulative busy ticks; unchanged original draw; context restored | Q-03 |
| flow_notifications / Q4 | Integration | Exactly-once transitions, stable ordinal, committed state visible; reentrant callback mutation prohibited; bounded zero-time loops terminate | Q-04/06 |
| flow_shared_world / Q4 | Integration | DES pathway and ABM staff behavior share IDs/clock/registry; cleaning delays next bed availability | Q-01/06 |
| queue_event_boundary_resume / Q4–Q5 | Determinism | Stop before/at/after eviction and grant, continue in the same live runtime; exact canonical trace and terminal state equality against uninterrupted execution. Portable checkpoint restore is deferred to Track 22 | Q-05 |
| queue_worker_invariance / Q5 | Determinism | Same replication IDs/seeds at 1/2/N workers and permuted finish order; canonical integer outputs byte-identical | Q-05 |
| queue_legacy_compat / Q5 | Regression/API | Existing Resource/DESContext FIFO fixtures and protected API surface remain valid; no new mandatory core dependencies | Q-06 |
| queue_scaling / Q5 | Benchmark | Queue sizes 10/1,000/100,000; varied active capacity, tie/churn/interruption rates; record time/memory and legacy comparison | Q-07 |

For randomized operation tests record/shrink the failing seed and promote minimal
counterexamples to permanent fixtures. Include debug/release and supported CPU
platforms. Worker tests cover independent runs; PDES cross-LP final-state parity
is a separate Track 34 gate. Do not substitute one for the other.
