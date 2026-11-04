# Risks

- Dependency adds no benefit: value gate.
- Changed semantics: independent oracle.
- Circular migration gates: no dependency on completed Polars for prerequisite tracks.

- PID-only core staging can collide and initial write failure can leave scratch data; qualify an independent baseline before replacement.
- Post-persist directory-sync errors may return Err with a published file; preserve recovery semantics.
- Historical drop/Python smoke tests and unmeasured platform/MSRV/cost labels are not current qualification.
- Shared source-bound manifests/build identity and minimal/default graph changes need explicit successor qualification.
