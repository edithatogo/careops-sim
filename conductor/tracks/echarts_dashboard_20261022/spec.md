# Apache ECharts analytical dashboard adoption

Maturity: planned experimental alpha adoption; implementation and runtime acceptance have not started. Authority: user-approved library adoption plan and explicit request for a separate track per library/dependency, 22 October 2026. rand and rand_distr are deliberately separate tracks. No dependency is installed by this planning registration.

Retain Kairos scheduler, ECS identity/lifecycle, DES/resource arbitration and clinical/model policy. Polars owns applicable tabular processing; DataFusion remains deferred without an executable adoption track. No parallel backend framework, publication, deployment or release is authorized by registration.

Exact versions/features, licences/advisories, MSRV, platform/browser support and build/binary/bundle/runtime costs must be qualified before dependency mutation. Record baseline owned code and replacement candidates; delete superseded machinery after qualified parity, retaining historical outputs and provenance. Never count dependency lines or test deletion as a maintenance benefit. Missing tools or runtime evidence remain unresolved, not green.

## Inputs, outputs and ownership

Inputs: accepted prerequisite commits, existing contracts/source, synthetic fixtures and baseline measurements. Outputs: library-backed behaviour, versioned contracts where changed, deleted/retained-code audit and reproducible acceptance receipts.

Owner: CareOps coordinator + domain/dashboard owner. Prospective owned implementation paths: `apps/dashboard/`. Planning owns this track and coordinated registry records only. Cargo/package manifests, lockfiles, parent pin, CI, FFI and shared schemas require a bounded coordinator handoff from current owners. Blocked paths: unrelated tracks, scheduler/resource/clinical semantics and other repositories. Upstream changes happen in a Kairos worktree, never as an unreviewed parent submodule edit.

Dependencies: rand_distr_sampling_20261022:R25. Accept CareOps sampling R25 and the preceding Polars/RNG chain; qualify current dashboard/native-v1 interfaces before UI changes. Parallel-safe work: read-only review only by default; delivery is serial. Fresh claims are mandatory; historical ownership and expired leases do not authorize takeover.

## Library and behaviour contract

CareOps apps/dashboard owns ECharts; no chart dependency enters Kairos core or browser/Wasm simulation packages. Use selective component imports and Canvas initially; qualify SVG for exports/small charts only when beneficial. Pin a stable version and lockfile; no extra chart library or framework.

Polars/native analytics supplies authoritative bins, distributions, intervals, weighting, denominators and scenario comparisons. ECharts formats and renders values, selections and navigation only. Version the minimal typed presentation payload with units, run/scenario identity, null semantics and provenance; preserve IDs/ticks losslessly and explicitly bound/downsample presentation without modifying analytical truth. Do not rerun simulations for display/export.

Deliver distributions, time series, heatmaps and comparisons. Keep accessible HTML tables/summaries, keyboard-operable controls, ARIA descriptions and non-colour encodings. Reject or safely escape untrusted labels/tooltips; no executable HTML from inputs. Dispose charts/listeners on unmount, resize correctly and handle empty/missing/large data. Hospital geometry remains the spatial viewer's responsibility.

## Acceptance, quality gates and release implications

All rows in [test matrix](test-matrix.md) must pass with independent behaviour evidence, version/feature provenance and before/after costs. API/ABI/checkpoint/schema changes require ADR, compatibility assessment, conformance fixtures and red-team response. A historical Done label or planning validator cannot accept runtime capabilities. Code/pin rollback uses a fresh output directory; never reinterpret old outputs as the new format. No deployment or publication is implied.

## References to requalify at entry

- https://echarts.apache.org/handbook/en/best-practices/canvas-vs-svg/
- https://echarts.apache.org/handbook/en/best-practices/aria/

## Dependency value gate

Apply [robustness-first policy](../../dependency-value-policy.md): identify actual code/consumer benefit, freeze costs and independent tests, and stop rather than add an unjustified dependency.

## Robustness-first entry refinement

R26.1 also requires accepted Kairos Track 78 and verified parent consumer/pin evidence. This inserts existing-system robustness before new chart work; a local ready-list proposal cannot waive the external gate.

## Validated dashboard boundary

R27 requires accepted Ajv R67 schema-validation receipt. R26 inventory may run before it. Native analytical values remain authoritative. Schema acceptance does not certify metrics, clinical rules or live transport.
