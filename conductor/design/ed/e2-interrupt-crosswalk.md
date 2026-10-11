# E2.1.interrupt fixture crosswalk

This is a behavior specification, not native simulator execution. All 24 cases remain `native_binding=unbound`, target E2.2.

| Topic | Frozen oracle | E2.2 binding obligation |
| --- | --- | --- |
| Consent and urgency | Incoming request consent, incumbent consent, and resource consent are separate inputs. All must permit interruption, and urgency must be strictly greater. Equal urgency never preempts. | Gate preempt-capable admission on all three policies. |
| Suspend and resume | Preemption checkpoints useful work through the native Suspend behavior; the episode resumes its remaining work and sample lineage. The adapter does not issue an extra explicit Release for Suspend. | Prove native suspend/resume claim history and no extra sample draw. |
| Calendar and waiting | Completion/release precedes calendar changes, then interrupt/handover decisions and grants. DrainActive work continues through break/shift unavailability. Waiting and paused work hold no staff claim; waits are separate from work and travel. | Verify exact claim accounting across break and shift transitions. |
| Priority | Flow priority is `!urgency` in i32, including both extremes and zero. The hand oracle executes max, zero, then min urgency in order. | Verify native domain-order mapping and ties. |
| Handover | An explicit event names episode, tick, incumbent and replacement. Handover uses one explicit Release of the old active claim; continuation gets a new segment ID and keeps episode/sample lineage. A busy replacement creates claim-free wait. | Bind actual Release/checkpoint mechanics; no implicit lease transfer or Cancel+Release. |
| Horizon and invalid controls | Observations clip at H, including an at-H zero-work grant. Eight complete valid baselines each have one semantic mutation and a targeted repair that restores the exact baseline. | Bind native diagnostics only after E2.2 freezes their types. |
| Input permutation | Staff, episode and event arrays reverse in a two-episode handover scenario; canonical input and episode/tick outcome oracles remain equal. | Verify deterministic normalized results in native execution. |

Focused tests compare every positive case with a separate literal hand-oracle table and validate specification inputs. They do not run a simulator. See `conductor/design/ed/e2-interrupt-fixture-contract.md` for the full boundary.
