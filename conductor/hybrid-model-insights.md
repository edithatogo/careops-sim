# Appraisal of supplied hybrid DES/ABM and Pixi.js notes

User-supplied note reviewed on 2026-09-25. Its recommendations are treated as
design hypotheses, with the following disposition in the requested tracks.
The later [user spatial requirement](spatial-visualization.md) selects a native
Kairos/WebSocket/PixiJS profile; the earlier worker-first preference below is now
an optional profile rather than the default spatial architecture.

| Insight | Decision and concrete incorporation |
| --- | --- |
| Staff identity, zones, skills and interruptions matter | Adopt a minimal agent-policy adapter and capacity-one staff resources; queue Q4 and fidelity C2 fixtures exercise assignment and interruption |
| Patient pathways combine scheduled activities with autonomous choices | Adopt one shared Flow world/clock and explicit Macro/Micro task policy; never duplicate DES and ABM state |
| Layout may affect bottlenecks | Add deterministic route graphs, transit-progress checkpoints and identifiable transit calibration. Congestion/occupancy-sensitive routing is deferred until justified by observations |
| ECS components fit beds, staff and locations | Keep reusable queue/spatial hooks in Kairos; clinical acuity, zone eligibility, cleaning/reservation lifecycle and specific pathways in CareOps adapters |
| Start with simple agent rules | Adopt urgency/FIFO plus explicit zone/skill eligibility; require held-out Macro/Micro comparison before accepting extra behavior complexity |
| Browser execution and Pixi.js are useful | Preserve as a follow-on integration option under Kairos Tracks 05/09/33; require snapshot and binding conformance before dashboard claims |

## Claims requiring qualification

**Hybrid is not automatically better.** Extra spatial/behavioral detail is useful
when it changes the decisions being studied and its parameters can be supported.
SimPy itself can model active agents, so the useful distinction here is Kairos's
native ECS/Flow integration and explicit fidelity contracts, not an inability of
DES tools to express agent behavior. See [SimPy's own overview](https://simpy.readthedocs.io/en/latest/index.html).

**Timestamps do not necessarily identify service, transit or heuristics.** The
calibration plan separates elapsed intervals from intrinsic work, records
censoring and tests confounding. Shadow anchors isolate parameter hypotheses;
free-running validation checks coupled effects. Fatigue, corridor congestion and
supervision rules remain hypotheses until measured and validated.

**Wasm does not imply zero serialization or copying.** Kairos currently exposes a
limited `WasmEngine` API including string status and JSON statistics methods; the reviewed
source does not expose this proposed Flow runtime or a ready-made Pixi ECS view.
The wasm-bindgen guide explicitly describes copies for boxed numeric slices.
A future buffer ABI must specify ownership, memory growth, lifetime, transfer and
copy budgets. See [wasm-bindgen numeric buffers](https://rustwasm.github.io/docs/wasm-bindgen/reference/types/boxed-number-slices.html).

**Animation performance is a separate measurement.** Pixi has its own ticker,
scene-update and rendering cycle; its render loop is not a simulation scheduler.
See [Pixi's render-loop documentation](https://pixijs.com/8.x/guides/concepts/render-loop).
Rendering at 60 FPS or avoiding network calls does not establish simulation
throughput, accuracy, latency or determinism. No Rust-versus-Python speedup or
commercial licensing comparison is assumed by these plans.

## Follow-on presentation contract

Use the existing Track 05 visualization snapshot contract and Track 09 Wasm
binding. Keep model execution in Rust, preferably in a browser worker for an
interactive prototype; the headless native runner remains the initial reference.
Publish versioned snapshots/deltas at a configurable interval, with entity IDs,
virtual timestamps, route segment endpoints and operational states. Pixi may
interpolate between snapshots for display; disabling, pausing or slowing rendering
must leave all model events and results unchanged. Input actions are timestamped
model commands, never direct mutation of ECS memory during render callbacks.

Bulk Arrow data supports analysis/export; a compact snapshot stream may be more
appropriate for animation. Choose using profiling, and document backpressure,
coalescing/drop policy for display snapshots, lossless event recording and worker
message/buffer ownership. Rendered WebGPU support does not prove that simulation
compute uses Track 33's WebGPU backend.

A later UI track should verify native/Wasm fixture parity, large tick/ID handling
(the reviewed Wasm convenience time surface uses u64, core uses u128), deterministic
results at render rates 0/30/60 FPS, bounded memory under a slow consumer, actual
copy counts and frame timings on named hardware. CAD input requires a separate
validated units/connectivity/route-graph conversion. These are handoff requirements,
not additional implementation deliverables in the two requested tracks.
