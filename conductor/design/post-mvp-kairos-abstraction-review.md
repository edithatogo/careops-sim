# Post-MVP Kairos abstraction review

User direction recorded 2026-10-04: after the functional MVP is accepted, review
which delivered capabilities can serve other domains and therefore belong in Kairos.
This review is post-E2 work; it adds no prerequisite to MVP delivery and authorizes
no extraction, public API change, new domain model or publication by itself.

## Review candidates

| Capability | Current boundary | Post-MVP question |
| --- | --- | --- |
| Shared clocks, exact UTC storage and deterministic seed streams | Generic primitives are already being developed in Kairos | Are temporal roles and provenance independent of clinical terminology, with explicit precision and error semantics? |
| Resource queues, allocation, interruption and lifecycle | Generic Flow runtime belongs in Kairos; ED priority rules stay in the parent | Can neutral tasks and resources support another scheduling example without changing engine defaults? |
| Agent assignment, routes and synthetic transit | Shared ABM/DES adapters belong in Kairos; staff roles and ED layouts stay in the parent | Which route/assignment operations have neutral contracts and deterministic state ownership? |
| Empirical ingestion, exclusions, censoring and validation | Generic Arrow/calibration mechanisms belong in Kairos; clinical field mapping stays in the parent | Can adapters preserve raw input, units, lineage and counted exclusions for another observation schema? |
| Replications, comparisons, uncertainty and bounded calibration | Reusable mathematical/experiment machinery is a Kairos candidate | Which operations consume neutral model outputs rather than clinical metrics or site assumptions? |
| Run manifests, atomic export, cancellation and recovery | Review the delivered application implementation before extraction | Can a small reusable library expose these guarantees without imposing a CLI, filesystem layout or domain schema? |
| Acceptance fixtures and selective verification harness | Repo tooling may be reusable separately from the runtime | Is reuse better achieved through a small test utility or shared CI recipe than a runtime dependency? |

ED pathways, acuity meanings, clinical service distributions, staff skill rules,
boarding/admission policies, Cairns/CHHHS profiles and operational validity claims
remain in careops-sim. A configurable clinical assumption is still a domain policy.

## Evidence required for an extraction proposal

1. Inventory actual MVP source, consumers and accepted receipts. Distinguish
   shipped behavior from proposals and partially qualified foundations.
2. Identify at least one concrete non-ED consumer or small neutral fixture with
   different policy semantics. Do not create a later clinical domain track merely
   to justify an abstraction.
3. Specify the smallest reusable contract: IDs, units, clocks, seed/state ownership,
   error/exclusion semantics, input/output schemas and resource limits.
4. Demonstrate that policies can remain outside that contract. Avoid clinical
   defaults, universal thresholds, competing clocks and speculative plugin systems.
5. Compare keeping the implementation in the parent, extracting a Kairos module,
   or reusing repository tooling. Account for API maintenance, dependencies, MSRV,
   platform support and migration cost; duplication alone is insufficient evidence.
6. Prepare a bounded proposal with Kairos track owners, exact paths, compatibility
   decision, migration/rollback plan and targeted conformance/consumer checks.
7. Integrate accepted Kairos changes on its development branch first, qualify the
   exact head, then update the parent pin and reproduce the ED outputs.

## Acceptance and sequencing

Run the review after functional MVP E2 acceptance; prioritize capabilities with
real reuse and stable semantics. Extraction remains separately reviewed and may
be staged alongside native-v1 hardening when its evidence is sufficient. Preserve
MVP output equivalence and supported toolchains; use affected checks and explicit
integration joins rather than repeating unrelated suites.

Public stable API and release acceptance retain their own compatibility, security
and artifact gates. Completing this review does not close C1-C6, Q5, E4 or D3-D4,
and does not certify any clinical deployment or additional domain capability.
