# ADR-0001: ED-scoped native support and release profile

- Status: accepted for CareOps Sim after independent review on 2026-09-28; no Kairos maintainer approval is recorded
- Date: 2026-09-27
- Owners: CareOps Sim coordinator; Kairos compatibility/toolchain owner contracts 25/30 apply
- Scope: CareOps Sim generic ED functional MVP and hardened native v1
- Evidence: [D0.3 support profile](../evidence/d0.3-ed-native-profile-20260927.json)

## Context

CareOps Sim is a Rust-native, hybrid DES/ABM emergency-department model built on
the pinned Kairos submodule. The ED needs a useful headless MVP before hardened
native v1. Later interactive/spatial and accelerated/distributed capabilities
must not become prerequisites for that delivery. Kairos Track 25 governs public
API/schema compatibility; Track 30 governs toolchain support and version floors.
The ED profile must fit these contracts without changing Kairos-wide policy or
claiming that the broader Kairos package ecosystem is ready to publish.

## Decision

1. Qualify the ED in three explicit stages: functional MVP, hardened native v1,
   and post-v1 extensions. The complete module-by-module classification and
   acceptance boundary are in the machine-readable profile.
2. Keep the MVP Rust-native, headless, and local-CPU. Its user surface is a
   documented scenario configuration and CLI/example runner with CSV/JSON
   results. Use deterministic seed/output, conservation and hand-computable
   queue/route fixtures. Do not require Arrow/Parquet, CAD, a renderer, web
   service, FFI, browser, device, network, or within-run parallel/distributed
   backend.
3. Native v1 retains MVP scope and adds the reviewed queue/preemption and
   calibration behavior, bounded Arrow/Parquet data paths, repeated experiments,
   recovery, quality/security gates, and a clean-consumer install/release
   rehearsal. A capability is not considered implemented because its crate,
   feature name, facade, or scaffold exists.
4. Intended ED execution targets are Linux x86_64 and macOS aarch64 with local
   CPU execution. They remain *candidate targets*, not support claims, until D1
   toolchain/consumer checks and D2/D4 platform CI and clean-consumer evidence
   pass. If either target cannot be qualified, the release record must mark it
   unsupported or keep the release blocked. Windows, Linux aarch64, GPU/Metal,
   Wasm/WebGPU and distributed targets are outside native-v1 support.
5. Use the exact tested Rust development toolchain selected in D1.1. Do not
   announce an ED consumer MSRV until D1.2 verifies the feature graph, dependency
   MSRVs, minimum/current consumer resolution and required Kairos-owner review.
   Do not silently raise Kairos core's MSRV or claim Rust 1.76 for an incompatible
   feature graph.
6. Track 25 compatibility review is impact-based: name every changed protected
   root; record unexposed C ABI, Arrow, and host-language surfaces as not in the
   stage's API. Require ADRs for public API/ABI/schema or compatibility-promise
   changes and migration/release decisions where Kairos policy requires them.
   No new binding implementation is required for native v1.
7. Preserve Kairos global publication and release holds. CareOps local runs,
   test artifacts, and dry-run package evidence do not authorize public
   publication, registry writes, or upstream release claims. Kairos Tracks 15,
   16, 20, 25, 28, 30, 42 and 44 remain independently applicable. This ADR does
   not update upstream status or assert approval from upstream maintainers.
8. Keep all reusable simulation and calibration computation Rust-native.
   Python/Mojo/Julia or other tools may be evaluated only as optional development
   or analysis aids; they are not native runtime dependencies. MLX/Metal belongs
   to the separately gated post-v1 acceleration phase, after an explicit API,
   correctness oracle, hardware evidence, and measured value.

## D0.3 independent profile review

The source/contract review is recorded in the
[D0.3 execution receipt](../evidence/d0.3-execution-receipt-20260928.md).
It accepted the scoped profile after these design checks:

- [x] Every pinned Kairos capability is classified exactly once as MVP,
      native-v1, or post-v1, with current source state distinguished from plan.
- [x] MVP and v1 scope match the delivery contract; later visual, spatial,
      clinical-domain, accelerated, or distributed work is not a hidden prerequisite.
- [x] Rust/native API roots and compatibility impact are explicit; unused FFI,
      Arrow and bindings are deferred without weakening their existing gates.
- [x] Rust toolchain/MSRV and platform claims are marked provisional where D1/D2
      evidence is unavailable; candidate versions and unrun CI are not called supported.
- [x] Local ED readiness is not represented as Kairos global release readiness;
      publication holds and protected environments remain in force.
- [x] No private patient data, credentials, or synthetic-to-clinical accuracy
      claim is introduced by the profile.
- [x] The reviewer found and rechecked the verifier fixes for incomplete source
      hashes, missing module allocations, publication-hold drift, and omitted
      acceptance markers; no D0.3 finding remains open.

## Native-v1 release verification checklist

These checks remain open and must pass before a native-v1 support/release claim:

- [ ] Resolve Rust toolchain, consumer MSRV, dependency features and
      minimum/current consumer resolution with actual builds and Track 25/30 review.
- [ ] Qualify each supported platform through CI, clean-consumer install, and
      deterministic fixtures; mark unqualified targets unsupported or hold release.
- [ ] Name exact protected API/schema roots and attach required ADRs, migration
      notes, conformance fixtures and release decisions to changes.
- [ ] Verify queue, RNG/seed, event ordering, schemas, units/timezones, and
      checkpoint/recovery changes with owner-specific test evidence.
- [ ] Complete risk-based security, artifact, checksum, SBOM and provenance checks.
- [ ] Preserve global publication holds unless their separate authorities release them.

## Consequences

This supports a small usable ED model first while keeping a traceable route to a
robust native library. Support claims are earned by D1/D2/D4 evidence rather than
inferred from local hardware or upstream documentation. The ED profile narrows
the features qualified by CareOps Sim; it does not narrow, replace, or waive
Kairos's global policy.

## Revisit triggers

Revisit this ADR before changing stage boundaries, supported targets, minimum
Rust version, public Rust/API/schema promises, dependency/MSRV policy, or the
publication authority boundary. Reopen it if native performance evidence shows
that a deferred backend is required for the stated MVP/v1 workload.
