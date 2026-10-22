# Three.js unified spatial viewer adoption

Maturity: planned experimental alpha adoption; implementation and runtime acceptance have not started. Authority: user-approved library adoption plan and explicit request for a separate track per library/dependency, 22 October 2026. rand and rand_distr are deliberately separate tracks. No dependency is installed by this planning registration.

Retain Kairos scheduler, ECS identity/lifecycle, DES/resource arbitration and clinical/model policy. Polars owns applicable tabular processing; DataFusion remains deferred without an executable adoption track. No parallel backend framework, publication, deployment or release is authorized by registration.

Exact versions/features, licences/advisories, MSRV, platform/browser support and build/binary/bundle/runtime costs must be qualified before dependency mutation. Record baseline owned code and replacement candidates; delete superseded machinery after qualified parity, retaining historical outputs and provenance. Never count dependency lines or test deletion as a maintenance benefit. Missing tools or runtime evidence remain unresolved, not green.

## Inputs, outputs and ownership

Inputs: accepted prerequisite commits, existing contracts/source, synthetic fixtures and baseline measurements. Outputs: library-backed behaviour, versioned contracts where changed, deleted/retained-code audit and reproducible acceptance receipts.

Owner: CareOps coordinator + domain/dashboard owner. Prospective owned implementation paths: `apps/dashboard/`. Planning owns this track and coordinated registry records only. Cargo/package manifests, lockfiles, parent pin, CI, FFI and shared schemas require a bounded coordinator handoff from current owners. Blocked paths: unrelated tracks, scheduler/resource/clinical semantics and other repositories. Upstream changes happen in a Kairos worktree, never as an unreviewed parent submodule edit.

Dependencies: petgraph_spatial_routing_20261022:R31. Accept CareOps routing R31, the preceding chart/sampling/data chain and existing native-v1 interface evidence before viewer implementation. Parallel-safe work: read-only review only by default; delivery is serial. Fresh claims are mandatory; historical ownership and expired leases do not authorize takeover.

## Library and behaviour contract

Use one Three.js scene with orthographic top-down viewing first, then perspective/multi-floor geometry. Start with WebGLRenderer/WebGL2; defer WebGPU until measured benefit. This browser renderer belongs in CareOps apps/dashboard. Consume generic Kairos snapshots/IDs; no Three.js, browser GPU or analytical-library requirement in headless core.

Reuse conductor/spatial-visualization.md: stable geometry/location/door/route IDs, floors, metric units/axes/origin/transforms, revision and provenance. Keep full generational IDs and tick representations lossless; JavaScript numbers are not arbitrary-width identity storage. Consume recorded replay before live deltas. Validate sequence/base/revision; recover gaps by authoritative snapshot, never silently invent state. Clearly distinguish simulated position from display interpolation. Commands remain backend validated and acknowledged.

Qualify current actual rendering, including SVG and declared PixiJS use; package.json or comments are not proof of a functioning Pixi renderer. Deliver selection/picking, overlays, labels, floor switching and replay parity before perspective and multi-floor controls. Keep HTML keyboard controls and accessible tables. Handle WebGL unavailability/context loss with an honest accessible state view; do not add a permanent second graphics engine.

Measure camera/picking/text usability, frame/interaction latency, initial bundle, CPU/GPU memory and repeated load/dispose behaviour on named supported browsers/devices and representative small/large fixtures. Freeze budgets from the baseline before replacement; no unreviewed material regression. Delete superseded rendering code/Pixi dependency only after parity and disadvantage review. If qualification fails, retain the working renderer and keep this track incomplete; no unconditional replacement claim. CAD capture/import remains separate future work.

## Acceptance, quality gates and release implications

All rows in [test matrix](test-matrix.md) must pass with independent behaviour evidence, version/feature provenance and before/after costs. API/ABI/checkpoint/schema changes require ADR, compatibility assessment, conformance fixtures and red-team response. A historical Done label or planning validator cannot accept runtime capabilities. Code/pin rollback uses a fresh output directory; never reinterpret old outputs as the new format. No deployment or publication is implied.

## References to requalify at entry

- https://threejs.org/docs/pages/OrthographicCamera.html
- https://threejs.org/docs/pages/WebGLRenderer.html

## Dependency value gate

Apply [robustness-first policy](../../dependency-value-policy.md): identify actual code/consumer benefit, freeze costs and independent tests, and stop rather than add an unjustified dependency.
