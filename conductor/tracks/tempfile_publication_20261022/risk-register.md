# Risks

- Dependency adds no benefit: value gate.
- Changed semantics: independent oracle.
- Circular migration gates: no dependency on completed Polars for prerequisite tracks.

- PID-only core staging can collide and initial write failure can leave scratch data; qualify an independent baseline before replacement.
- Post-persist directory-sync errors may return Err with a published file; preserve recovery semantics.
- Historical drop/Python smoke tests and unmeasured platform/MSRV/cost labels are not current qualification.
- Shared source-bound manifests/build identity and minimal/default graph changes need explicit successor qualification.

- R53.2 observed14% bounded large-write median overhead and7 added minimal normal nodes; whole-app costs remain R55, no speedup claim.
- Actual core/CLI workspace lock rows change even without new registry versions, invalidating the old byte-identical-lock successor rule; R54 needs a new exact closed receipt and model/recovery proof.
- NamedTempFile defaults0600 and TempDir is not automatically private under every umask; preserve output creation semantics and explicitly protect scratch directories.
- Cached objects from differently built compilers may be ABI-incompatible even with identical release labels; qualify actual compiler/closure and retain old recovery software.
