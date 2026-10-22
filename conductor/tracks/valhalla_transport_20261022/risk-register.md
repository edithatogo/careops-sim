# Risks

| Risk | Severity | Control |
| --- | --- | --- |
| Unjustified feature/operational cost | High | No adoption if existing validated travel-time matrices answer the question; C++ service operations must justify extra maintenance. OSRM is an alternative to reassess only if Valhalla fails, not a second installed engine. |
| Misleading physical/clinical fidelity | High | Independent data/method validation and explicit interpolation versus simulation |
| Clock/identity/arrival divergence | High | One movement authority, exact event boundaries and failure/recovery proof |
| Licensing/private input exposure | High | Source-qualified permitted data, local fixtures and attribution |
