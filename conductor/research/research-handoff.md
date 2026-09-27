# Research completion and delivery handoff

Updated 2026-09-27. This is the current research status; it supersedes earlier
blanket requests for standalone bundles or repeated broad research.

## Completed work

| Completed activity | Evidence |
| --- | --- |
| All requested research themes have supplied responses | [Prompt response register](xml-prompts/README.md) |
| Intake, duplicate identification and source-file hashes | [Archive manifest](supplied/20260927/manifest.json) |
| Embedded content audited and indexed: 15 distinct reports, 88 tables/blocks | [Content audit](embedded-content-audit.md), [excerpt manifest](embedded-content/manifest.json) |
| Findings and conflicts incorporated into five track specs/plans | [First integration](ed-research-incorporation-20260927.md), [9–12](ed-research-incorporation-9-12-20260927.md), [26–30](ed-research-incorporation-26-30-20260927.md) |

These are completed intake/planning activities, not a claim that every source,
parameter, executable fixture or runtime capability has been verified. Existing
numbered task checkboxes retain their original full acceptance requirements.

## Remaining work belongs to existing tasks

| Owner | Remaining deliverable |
| --- | --- |
| P0 | Canonical scope/parameter/coverage map and schema from supplied content; record true gaps, conflicts and provenance |
| P1/P2 | Verify DES/ABM claims against primary sources, document extraction and transfer limits, fill specific gaps |
| P3/P4/P5 | Reviewed distributions/dependence, validated generic input pack, actual-model coverage and loading tests |
| D0/D1 | Current pinned Kairos capability/contract audit, supported toolchain/dependency choices and evaluated harness/skills |
| Q0/Q1–Q5 | Verify reference semantics, freeze local contracts, implement and run queue/preemption tests |
| C0/C1–C6 | Verify methods/standards mappings, freeze schemas, implement ingestion/calibration and validate results |
| E0/E1–E4 | Freeze generic ED boundary and metrics, build skeleton and complete/qualify native model |
| E5/E6–E8 | Dashboard and separately qualified Metal, PDES and distributed backends |

Each adopted claim needs both report provenance (hash/section/line) and primary
source provenance: URL/DOI, version/date, table/page/field/code location, relevant
population/period, units, licence/access, extraction/transformation and verification
outcome. Synthetic assumptions are valid explicit inputs where appropriate, not
empirical evidence. Unavailable evidence gets a named gap, owner and impact.
Never invent missing rows to match a reported register size. No separate generated
file is required when adequate content is present in the report.

## Next execution order

1. **D0.2 now**, followed by D0.3 and D0 closeout: resolve required Kairos
   capabilities, ownership and acceptance gates. Then D1 pins/tests the development
   environment and qualifies bounded workflows.
2. **P0.1 is also ready now**: reconcile supplied evidence into the input coverage
   map; complete P0 schemas/protocol. P1 DES and P2 ABM verification can then run
   independently, joining at P3 and P4. This can progress alongside readiness work.
3. After **D1**, Q0 contracts can proceed; **C0 and E0 additionally require P0**.
   E0 produces the minimal deterministic Rust skeleton. Shared contracts are
   coordinated before dependent implementation packets are dispatched.
4. **D2 follows D1 and E0**: create/configure GitHub at its existing useful-readiness
   gate, then required CI. Q1 and C1 implementation follow their contract/CI gates;
   other C work follows its recorded prerequisites.
5. **E1/E2** integrate reviewed queue/fidelity capabilities and P4 inputs. C5 and
   P5 lead into C6 validation; E3/D4/E4 complete runner/release/native acceptance.
   E5 dashboard and E6 Metal may then proceed in separate ownership areas, followed
   by E7 PDES and E8 distributed qualification as the current DAG specifies.

The authoritative schedule is metadata plus `tools/tasks.py`; this summary does
not remove any detailed dependency. Serial operation is fully supported. Parallel
operation requires reserved disjoint paths, approved interfaces and bounded packets.
No workers are launched merely by listing ready candidates.

## Is more planning needed?

No additional broad research round or new track is needed before starting D0.2
and P0.1. Remaining design decisions are already assigned to D0/D1, P0, Q0, C0
and E0: capability closure, toolchain/MSRV, schemas, APIs, model boundary, metrics
and test oracles. Prepare small exact-input/output worker packets just before
execution. New tracks are warranted only if a demonstrated gap changes scope;
ordinary source verification and missing input fields stay in these tracks.

Current programme: 5 tracks, 34 milestones, 143 numbered tasks; only D0.1 is
accepted complete. The completed research intake above is separately recorded,
so that software-completion figures are not inflated. No user clarification is
needed to start the next tasks. Repository owner/name/visibility is resolved at
D2; Cairns-specific evidence remains a later site-profile requirement.

## MVP clarification

First usefulness is the headless E1/E2 model and its basic reproducible outputs,
not E5 visualization. Bed/staff capacity and minimal locations/routes remain core
inputs; visual assets, CAD/capture and WebSocket UI follow later. E3 and native
qualification evolve this working baseline without implying the early MVP meets
all release or empirical-validation gates.

## Authoritative delivery sequence

[Functional MVP → hardened native v1 → extensions](../delivery-contract.md) defines the
required features and acceptance recipes. E2 is the functional headless MVP; E4
is native v1. Visual/spatial import, live UI and advanced backends remain post-v1.
