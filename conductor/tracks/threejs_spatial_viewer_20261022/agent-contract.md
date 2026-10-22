# Ownership contract

## Inputs, outputs and ownership

Inputs: accepted prerequisite commits, existing contracts/source, synthetic fixtures and baseline measurements. Outputs: library-backed behaviour, versioned contracts where changed, deleted/retained-code audit and reproducible acceptance receipts.

Owner: CareOps coordinator + domain/dashboard owner. Prospective owned implementation paths: `apps/dashboard/`. Planning owns this track and coordinated registry records only. Cargo/package manifests, lockfiles, parent pin, CI, FFI and shared schemas require a bounded coordinator handoff from current owners. Blocked paths: unrelated tracks, scheduler/resource/clinical semantics and other repositories. Upstream changes happen in a Kairos worktree, never as an unreviewed parent submodule edit.

Dependencies: petgraph_spatial_routing_20261022:R31. Accept CareOps routing R31, the preceding chart/sampling/data chain and existing native-v1 interface evidence before viewer implementation. Parallel-safe work: read-only review only by default; delivery is serial. Fresh claims are mandatory; historical ownership and expired leases do not authorize takeover.

Use the single-maintainer harness: isolated worktree, repository-wide claim, bounded hashed context and precommit scope check. Inputs must exclude files intentionally edited. One commit per claim; release after commit. Review mutable shared surfaces with existing owners. This registration grants no runtime completion or automatic dispatch.
