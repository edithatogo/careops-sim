# MVP leaf work breakdown

Generated from [recipes.json](recipes.json). All leaves target gpt-6-luna with
coordinator acceptance; historical accepted tasks are not re-executed. See the
[binding protocol](README.md). Plural outputs fan out by source/family/case, with
an explicit instance set and acceptance join. Runtime commands/paths bind only
after prerequisite interfaces exist.

## D0.1

Parent plan: [D0.1](../../tracks/development_readiness_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| D0.1.historical | Existing accepted audit evidence | Verify existing receipt; do not repeat accepted work or infer new runtime capability | . / worker |

## D0.2

Parent plan: [D0.2](../../tracks/development_readiness_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| D0.2.inventory | Manifest inventory JSON | All current crates occur once with exact direct dependencies/features; empty backend features are not availability proof | . / worker |
| D0.2.capabilities | Capability/source evidence table | Every MVP capability maps to an existing API or an explicit implementation gap with source lines | . / worker |
| D0.2.gates | Upstream prerequisites and licensing discrepancy record | Missing contract/gate evidence blocks ready status; Cargo/README/LICENSE differences remain explicit | . / worker |
| D0.2.join | Reviewed capability closure | Every required capability has one owner and gate; post-v1 features do not enter MVP closure | . / proposal_or_review |

## D0.3

Parent plan: [D0.3](../../tracks/development_readiness_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| D0.3.profile | Native support-profile proposal | Targets/toolchains and unsupported features are explicit; no unexecuted platform passes | . / worker |
| D0.3.classification | MVP/v1/post-v1 capability mapping | Every capability belongs to a stage and optional CAD/UI/device features cannot gate MVP | . / worker |
| D0.3.review | Approved scope ADR | Coordinator records decision/rationale and preserved upstream publication holds | . / proposal_or_review |

## D0.4

Parent plan: [D0.4](../../tracks/development_readiness_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| D0.4.evidence | Phase evidence reconciliation | Every parent task has integrated output and executed gates; unresolved failures prevent acceptance | . / worker |
| D0.4.closeout | Coordinator phase acceptance record | Independent readback agrees with evidence and upstream gates; only coordinator updates checkbox/catalog/pin | . / proposal_or_review |

## D1.1

Parent plan: [D1.1](../../tracks/development_readiness_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| D1.1.tool-errors | Missing/wrong-tool bootstrap fixtures | Absent or incompatible tool produces actionable failure without modifying system tools | . / worker |
| D1.1.mac-bootstrap | macOS ARM bootstrap recipe/evidence | Clean supported macOS executes declared minimal native command; unavailable host stays unverified | . / worker |
| D1.1.linux-bootstrap | Linux bootstrap recipe/evidence | Clean supported Linux executes same native profile with platform-specific checksums | . / worker |
| D1.1.consumer | Minimum/current consumer build evidence | Both declared toolchain profiles resolve locked dependencies or expose a reviewed incompatibility | . / worker |

## D1.2

Parent plan: [D1.2](../../tracks/development_readiness_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| D1.2.compatibility | Dependency/MSRV compatibility matrix | Arrow/TOML/Rayon and backend candidates have verified source/version/feature requirements | . / worker |
| D1.2.decision | Toolchain-transition ADR | Coordinator approves supported floor and canary policy before manifests change | . / proposal_or_review |
| D1.2.kairos-pins | Kairos manifest/toolchain policy change | Minimal and current builds agree with approved MSRV; no unreviewed dependency upgrade | libs/kairos / worker |
| D1.2.parent-pins | Parent pin/docs alignment | Parent consumes reviewed Kairos commit and docs/tool checks report the same version policy | . / worker |

## D1.3

Parent plan: [D1.3](../../tracks/development_readiness_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| D1.3.inventory | Skill provenance inventory | Each relevant skill has source/licence/hash/permissions or a missing-evidence flag | . / worker |
| D1.3.gap | Skill behavior gap decision | Only missing useful behaviors are selected; no broad mutable skill installation | . / worker |
| D1.3.candidate | One bounded skill draft per approved gap | Inputs/outputs/stop conditions match gap and contain no installation or acceptance authority | . / worker |

## D1.4

Parent plan: [D1.4](../../tracks/development_readiness_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| D1.4.oracles | Reviewer-owned hidden evaluation manifest | Expected pass/reject/escalate outcomes are frozen and unreadable to evaluated worker | . / proposal_or_review |
| D1.4.baseline | No-skill serial canary receipt | Actual model/version/outcome/cost-unknown status recorded; false completion is rejection | . / worker |
| D1.4.candidate | Pinned-skill and serial/parallel comparison receipt | Same task classes and independent hidden variants; known-bad acceptance denominators separated | . / worker |
| D1.4.promotion | Model/task-class qualification decision | Only demonstrated task classes promoted; failures retain supervised status and rationale | . / proposal_or_review |

## D1.5

Parent plan: [D1.5](../../tracks/development_readiness_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| D1.5.receipts | Versioned command/attempt receipt format | Exit status/output hashes and immutable attempt IDs cannot be replaced by claimed success | . / worker |
| D1.5.drift | Dispatch/acceptance drift checks | Changed inputs/base/skills cause rejection before dispatch or acceptance | . / worker |
| D1.5.recovery | Killed-worker resume fixture | Resume reports exact prior step/artifacts and never repeats accepted side effects blindly | . / worker |
| D1.5.ownership | Concurrent-writer and hidden-oracle isolation evidence | Conflicting reservation rejected; held-out oracle inaccessible to worker | . / worker |

## D1.6

Parent plan: [D1.6](../../tracks/development_readiness_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| D1.6.evidence | Phase evidence reconciliation | Every parent task has integrated output and executed gates; unresolved failures prevent acceptance | . / worker |
| D1.6.closeout | Coordinator phase acceptance record | Independent readback agrees with evidence and upstream gates; only coordinator updates checkbox/catalog/pin | . / proposal_or_review |

## D2.1

Parent plan: [D2.1](../../tracks/development_readiness_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| D2.1.fast-lanes | Local fmt/clippy/unit/doctest/context lane definitions | One failing check fails aggregate while unchanged scope is explicit | . / worker |
| D2.1.trust | Untrusted-PR smoke fixture | No secret/privileged execution on untrusted inputs | . / worker |
| D2.1.artifact | Missing-artifact negative fixture | Absent required result cannot yield a green aggregate | . / worker |

## D2.2

Parent plan: [D2.2](../../tracks/development_readiness_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| D2.2.readiness | Repository creation readiness record | D1/E0 accepted, local fixture passes, owner/name/visibility resolved and existing repo checked | . / proposal_or_review |
| D2.2.create | Authorized repository create/link evidence | Coordinator executes once against resolved identity; no duplicate or unsolicited publication | . / proposal_or_review |
| D2.2.rules | Required-check configuration/readback | Failing check actually blocks acceptance; single-maintainer process remains operable | . / worker |

## D2.3

Parent plan: [D2.3](../../tracks/development_readiness_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| D2.3.pins | Action/tool immutable-pin audit | Mutable tags including v4 are rejected; all adopted versions have verified provenance | . / worker |
| D2.3.policy | Full dependency/secret/workflow checks | Advisories/bans/licences/sources all execute; actionlint/zizmor failures remain visible | . / worker |
| D2.3.trust | Permission/cache/concurrency policy fixture | Untrusted writes cannot poison trusted release caches or obtain privileged tokens | . / worker |
| D2.3.platforms | Hosted native-profile evidence | Declared Linux/macOS checks execute on actual hosts with bounded timeout and readable logs | . / worker |

## D2.4

Parent plan: [D2.4](../../tracks/development_readiness_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| D2.4.upstream | Kairos owner CI integration | Required owner checks reference the reviewed upstream commit | libs/kairos / worker |
| D2.4.parent | Parent exact-gitlink integration lane | A deliberately incompatible Kairos pin fails parent integration even with upstream green results | . / worker |

## D2.5

Parent plan: [D2.5](../../tracks/development_readiness_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| D2.5.evidence | Phase evidence reconciliation | Every parent task has integrated output and executed gates; unresolved failures prevent acceptance | . / worker |
| D2.5.closeout | Coordinator phase acceptance record | Independent readback agrees with evidence and upstream gates; only coordinator updates checkbox/catalog/pin | . / proposal_or_review |

## P0.1

Parent plan: [P0.1](../../tracks/ed_parameter_evidence_20260927/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| P0.1.des-map | DES input coverage proposal | Demand/pathways/capacity/durations map to supplied excerpts or named gaps without invented values | . / worker |
| P0.1.abm-map | ABM/spatial input coverage proposal | Behavior/skills/transit/interruptions have source lineage or explicit assumptions | . / worker |
| P0.1.controls-map | Experiment/measurement input coverage proposal | Seeds/windows/fidelity/censoring/metric denominators included; outputs not mislabeled primitives | . / worker |
| P0.1.reconcile | Canonical merge/conflict map | Duplicate meanings merge with lineage; historical headline counts are not completeness evidence | . / worker |

## P0.2

Parent plan: [P0.2](../../tracks/ed_parameter_evidence_20260927/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| P0.2.ids | Parameter ID and ownership dictionary | Every in-scope input has stable identity/consumer or deferred status; duplicates rejected | . / worker |
| P0.2.schema | Evidence/range/schema proposal and negative examples | Unknown differs from zero; support/range/uncertainty/search bounds remain distinct | . / worker |
| P0.2.capacity | Capacity/location schema fixture | Physical beds differ from open/staffed capacity; minimal location/route units require no drawing | . / worker |
| P0.2.validator | Approved schema validator | Malformed units/provenance/IDs fail and minimal valid synthetic record passes | . / worker |

## P0.3

Parent plan: [P0.3](../../tracks/ed_parameter_evidence_20260927/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| P0.3.protocol | Source-verification record template | Primary URL/DOI/revision/table/population/licence/transform and report hash/line are separate | . / worker |
| P0.3.packets | Bounded parameter-family extraction rules | One family/source per instance; searches only named gaps; unavailable evidence remains unknown | . / worker |

## P0.4

Parent plan: [P0.4](../../tracks/ed_parameter_evidence_20260927/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| P0.4.evidence | Phase evidence reconciliation | Every parent task has integrated output and executed gates; unresolved failures prevent acceptance | . / worker |
| P0.4.closeout | Coordinator phase acceptance record | Independent readback agrees with evidence and upstream gates; only coordinator updates checkbox/catalog/pin | . / proposal_or_review |

## P1.1

Parent plan: [P1.1](../../tracks/ed_parameter_evidence_20260927/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| P1.1.arrivals | Arrival/case-mix source records | Population/period/denominators and conditioning preserved; no joint distribution inferred from marginals | . / worker |
| P1.1.pathways | Pathway and public-model source records | Route probabilities/field meanings and code revisions are reproducible with licence limits | . / worker |
| P1.1.offers | Admission-offer boundary records | Eligible specialty/time offers differ from raw ward discharges and boarding durations | . / worker |

## P1.2

Parent plan: [P1.2](../../tracks/ed_parameter_evidence_20260927/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| P1.2.capacity | As-operated capacity/calendar records | Drawn physical bays, staffed/open beds, monitoring/closures/surge separated with effective dates | . / worker |
| P1.2.work | Triage/clinical-work source extraction | Gerdtz/Bucknall and IHACPA claims verified or unresolved; mean alone does not determine a distribution | . / worker |
| P1.2.support | Diagnostic/cleaning/handover duration records | Intrinsic work differs from elapsed waiting/transit; unknown components explicit | . / worker |
| P1.2.exit | Patience/route/boarding evidence records | Censored/unobserved exits are not zero-duration events or assumed empirical hazards | . / worker |

## P1.3

Parent plan: [P1.3](../../tracks/ed_parameter_evidence_20260927/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| P1.3.records | One normalized DES family per source | Units/value class/full parameterization and transform link to primary evidence | . / worker |
| P1.3.checks | DES catalogue validity checks | Invalid probabilities/impossible combinations/missing sources fail with record IDs | . / worker |
| P1.3.review | Independent DES extraction receipt | Reviewer reproduces selected values from source and records gaps/transfer limitations | . / proposal_or_review |

## P1.4

Parent plan: [P1.4](../../tracks/ed_parameter_evidence_20260927/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| P1.4.evidence | Phase evidence reconciliation | Every parent task has integrated output and executed gates; unresolved failures prevent acceptance | . / worker |
| P1.4.closeout | Coordinator phase acceptance record | Independent readback agrees with evidence and upstream gates; only coordinator updates checkbox/catalog/pin | . / proposal_or_review |

## P2.1

Parent plan: [P2.1](../../tracks/ed_parameter_evidence_20260927/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| P2.1.eligibility | Staff roles/skills/zones evidence records | Eligible action sets and persistent assignments have explicit source or policy class | . / worker |
| P2.1.interruptions | Interruption/resumption behavior records | Incoming prompts, actual switches, ancestry and eventual resumption are separate | . / worker |
| P2.1.calendars | Shift/break/handover policy records | Availability changes and unfinished work have an explicit model policy, not an inferred clinical mandate | . / worker |

## P2.2

Parent plan: [P2.2](../../tracks/ed_parameter_evidence_20260927/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| P2.2.graph | Spatial-source/topology records | Scale/units/connectivity/access and revision verified or declared synthetic; CAD not required | . / worker |
| P2.2.movement | Movement-mode/speed evidence records | Actor/task/mobility and walk/wait/work separate; no universal walking-speed default | . / worker |
| P2.2.capture | Geometry/provenance acquisition notes | CAD source retained with derived layer manifest; GeoJSON/metric coordinates not conflated; capture uncertainty explicit | . / worker |

## P2.3

Parent plan: [P2.3](../../tracks/ed_parameter_evidence_20260927/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| P2.3.records | One normalized ABM family per source | Assumed heuristic differs from observed behavior and links to primary/report provenance | . / worker |
| P2.3.checks | ABM graph/unit/overlap checks | Impossible routes and duplicated service/transit intervals rejected | . / worker |
| P2.3.review | Independent ABM evidence/gap receipt | Every unresolved claim has owner/acquisition route/impact and cannot become an empirical default | . / proposal_or_review |

## P2.4

Parent plan: [P2.4](../../tracks/ed_parameter_evidence_20260927/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| P2.4.evidence | Phase evidence reconciliation | Every parent task has integrated output and executed gates; unresolved failures prevent acceptance | . / worker |
| P2.4.closeout | Coordinator phase acceptance record | Independent readback agrees with evidence and upstream gates; only coordinator updates checkbox/catalog/pin | . / proposal_or_review |

## P3.1

Parent plan: [P3.1](../../tracks/ed_parameter_evidence_20260927/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| P3.1.fits | Known-distribution and sparse-support fixtures | Recovery tolerance fixed before fits; tails/support and insufficient-data status tested | . / worker |
| P3.1.censoring | Censor/leakage/dependence fixtures | Censor is not event, future completion unavailable at training cutoff, cluster membership preserved | . / worker |
| P3.1.identifiability | Confounded versus identifiable fixture | Multiple walk/work decompositions cannot be reported uniquely identified | . / worker |

## P3.2

Parent plan: [P3.2](../../tracks/ed_parameter_evidence_20260927/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| P3.2.candidate | One family empirical/parametric comparison | Same frozen data/split/units; fit diagnostics and rejected alternatives retained | . / worker |
| P3.2.holdout | Held-out distribution validation record | No training/selection leakage and uncertainty/transfer limits visible | . / worker |
| P3.2.decision | Distribution/assumption selection ADR | Coordinator approves choice; insufficient data yields explicit synthetic assumption or unknown | . / proposal_or_review |

## P3.3

Parent plan: [P3.3](../../tracks/ed_parameter_evidence_20260927/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| P3.3.sampling | Conditional generation order and seed-purpose proposal | Arrival mode generated before dependent state; realized future outcome cannot guide early decisions | . / worker |
| P3.3.uncertainty | Variability/parameter-uncertainty contract | Within-run variation and replication-level uncertainty are separate with documented bounds | . / worker |
| P3.3.review | Reviewed DES/ABM sampling join | Transit/wait not counted again in intrinsic work; ATS denominators/missingness separately tested | . / proposal_or_review |

## P3.4

Parent plan: [P3.4](../../tracks/ed_parameter_evidence_20260927/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| P3.4.evidence | Phase evidence reconciliation | Every parent task has integrated output and executed gates; unresolved failures prevent acceptance | . / worker |
| P3.4.closeout | Coordinator phase acceptance record | Independent readback agrees with evidence and upstream gates; only coordinator updates checkbox/catalog/pin | . / proposal_or_review |

## P4.1

Parent plan: [P4.1](../../tracks/ed_parameter_evidence_20260927/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| P4.1.minimal | Minimal deterministic input profile | All supported input shapes validate and have hand-computable route/resource outcome | . / worker |
| P4.1.nominal | Generic nominal input profile | Every value traces to a source or explicit synthetic assumption; no private data | . / worker |
| P4.1.surge | Overload/constrained-capacity input profile | Changed capacity/demand intentionally stresses known bottleneck with valid units/provenance | . / worker |

## P4.2

Parent plan: [P4.2](../../tracks/ed_parameter_evidence_20260927/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| P4.2.invalid | Malformed/missing/censored input corpus | Expected errors distinguish missing from zero and censor from completion | . / worker |
| P4.2.consistency | Joint/profile/calendar checks | Conditional sums, support, initial state and resource feasibility validated | . / worker |
| P4.2.capacity | Closed/repurposed/surge capacity cases | Unknown bay status cannot mean open; physical/open/staffed counts reconcile | . / worker |
| P4.2.manifest | Input-pack hashes/licence manifest | Every profile artifact has reproducible content hash and redistribution disposition | . / worker |

## P4.3

Parent plan: [P4.3](../../tracks/ed_parameter_evidence_20260927/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| P4.3.guide | Parameter/loading/override documentation | Reader distinguishes values, bounds, uncertainty and assumptions without code edits | . / worker |
| P4.3.cairns | Future Cairns mapping/gap document | No local conformance assumed; source timestamps/location rules/ETL requirements explicit | . / worker |
| P4.3.handoff | Frozen generic-pack interface receipt | E1/C5 consumers get exact schema/input hashes and unresolved validity limits | . / worker |

## P4.4

Parent plan: [P4.4](../../tracks/ed_parameter_evidence_20260927/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| P4.4.evidence | Phase evidence reconciliation | Every parent task has integrated output and executed gates; unresolved failures prevent acceptance | . / worker |
| P4.4.closeout | Coordinator phase acceptance record | Independent readback agrees with evidence and upstream gates; only coordinator updates checkbox/catalog/pin | . / proposal_or_review |

## Q0.1

Parent plan: [Q0.1](../../tracks/des_queue_preemption_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| Q0.1.surface | Current Flow/core/API inventory | Source references distinguish legacy behavior from new additive proposals | libs/kairos / worker |
| Q0.1.runtime | Shared runtime/handle/error ADR proposal | One world/clock; public types/generation ownership explicit without legacy removal | libs/kairos / worker |
| Q0.1.codec | Context/snapshot ownership proposal | Registered context codecs and upstream owners identified; no second snapshot standard | libs/kairos / worker |
| Q0.1.review | Approved API compatibility ADR | Coordinator resolves public contracts before runtime changes | libs/kairos / proposal_or_review |

## Q0.2

Parent plan: [Q0.2](../../tracks/des_queue_preemption_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| Q0.2.queue | Priority/FIFO/repriority golden traces | Lower priority wins; equal FIFO sequence retained on rekey; no equal-priority eviction | libs/kairos / worker |
| Q0.2.boundaries | Deadline/completion/capacity golden traces | No grant at deadline; completion-at-T wins; shrink below active rejected; idle zero allowed | libs/kairos / worker |
| Q0.2.strategies | Suspend/Abort/Restart golden traces | 10 at 0 with urgent 2 at 3 ends low 12/absent/15 and urgent 5; restart reuses draw | libs/kairos / worker |
| Q0.2.reference | Pinned reference/difference table | SimPy semantics verified; intentional local differences and zero-time bounds reviewed | libs/kairos / worker |

## Q0.3

Parent plan: [Q0.3](../../tracks/des_queue_preemption_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| Q0.3.events | Event-kind collision inventory/proposal | Every new Flow kind is uniquely assigned through core owner contract | libs/kairos / worker |
| Q0.3.sidecar | Lifecycle sidecar join/schema proposal | run/event/ordinal uniquely joins transitions; event_log.v1 unchanged | libs/kairos / worker |
| Q0.3.snapshot | Snapshot extension and broken-link handoff | Queue revisions/context/seeds routed through owners 01/04/22 with accepted interfaces | libs/kairos / worker |

## Q0.4

Parent plan: [Q0.4](../../tracks/des_queue_preemption_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| Q0.4.evidence | Phase evidence reconciliation | Every parent task has integrated output and executed gates; unresolved failures prevent acceptance | libs/kairos / worker |
| Q0.4.closeout | Coordinator phase acceptance record | Independent readback agrees with evidence and upstream gates; only coordinator updates checkbox/catalog/pin | libs/kairos / proposal_or_review |

## Q1.1

Parent plan: [Q1.1](../../tracks/des_queue_preemption_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| Q1.1.handles | Entity liveness/recycle/duplicate-release tests | Old handle cannot release new lease; terminal release cannot free twice | libs/kairos / worker |
| Q1.1.capacity | Zero/overflow/shrink/removal tests | Rejected command leaves state unchanged and capacity never underflows | libs/kairos / worker |

## Q1.2

Parent plan: [Q1.2](../../tracks/des_queue_preemption_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| Q1.2.resource | ResourceCapacity and ActiveAllocations implementation | Each live lease links exactly one live owner/request and active count conserves capacity | libs/kairos / worker |
| Q1.2.request | ResourceRequest/work/context component implementation | Invalid generation or partial request cannot attach inconsistent state | libs/kairos / worker |
| Q1.2.queue | ClaimQueue/index component implementation | No request simultaneously active and waiting; registry reuse preserves deterministic identity | libs/kairos / worker |
| Q1.2.admission | Checked command admission/despawn cleanup | Invalid command is atomic and owner removal cleans claims without leaks | libs/kairos / worker |

## Q1.3

Parent plan: [Q1.3](../../tracks/des_queue_preemption_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| Q1.3.lease | Manual lease grant/release lifecycle | Grant consumes one unit; valid release returns one unit once | libs/kairos / worker |
| Q1.3.inspection | Canonical state inspection | Output ordering independent of dense-store/HashMap iteration | libs/kairos / worker |
| Q1.3.errors | Past-command and invalid-builder rejection | Typed error leaves component/queue state byte-equivalent to prior state | libs/kairos / worker |

## Q1.4

Parent plan: [Q1.4](../../tracks/des_queue_preemption_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| Q1.4.evidence | Phase evidence reconciliation | Every parent task has integrated output and executed gates; unresolved failures prevent acceptance | libs/kairos / worker |
| Q1.4.closeout | Coordinator phase acceptance record | Independent readback agrees with evidence and upstream gates; only coordinator updates checkbox/catalog/pin | libs/kairos / proposal_or_review |

## Q2.1

Parent plan: [Q2.1](../../tracks/des_queue_preemption_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| Q2.1.order | FIFO/repriority/insertion permutation fixtures | Original admission sequence retained; cancel/resubmit gets new sequence | libs/kairos / worker |
| Q2.1.deadline | Cancel/timeout/growth-drain fixtures | Both timeout/release orders reject grant at exclusive deadline | libs/kairos / worker |

## Q2.2

Parent plan: [Q2.2](../../tracks/des_queue_preemption_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| Q2.2.index | Ordered queue insert/remove/rekey | Reference ordering preserved under churn and sequence overflow rejected | libs/kairos / worker |
| Q2.2.timeouts | Deadline events and invalidation | Stale timeout has no mutation after first grant/cancel | libs/kairos / worker |
| Q2.2.arbitration | Resource arbitration transaction | Resource priority does not alter scheduler order; commits precede notifications | libs/kairos / worker |

## Q2.3

Parent plan: [Q2.3](../../tracks/des_queue_preemption_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| Q2.3.invariants | Generated queue-state invariant suite | Every operation preserves capacity and disjoint active/waiting membership | libs/kairos / worker |
| Q2.3.ties | Equal-time boundary regression suite | Multiple insertion orders and scheduler priorities obey local completion/deadline rules | libs/kairos / worker |

## Q2.4

Parent plan: [Q2.4](../../tracks/des_queue_preemption_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| Q2.4.evidence | Phase evidence reconciliation | Every parent task has integrated output and executed gates; unresolved failures prevent acceptance | libs/kairos / worker |
| Q2.4.closeout | Coordinator phase acceptance record | Independent readback agrees with evidence and upstream gates; only coordinator updates checkbox/catalog/pin | libs/kairos / proposal_or_review |

## Q3.1

Parent plan: [Q3.1](../../tracks/des_queue_preemption_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| Q3.1.strategies | Primary three-strategy tests | Low 10 at 0 urgent 2 at 3 gives 12/absent/15; no duplicate terminal records | libs/kairos / worker |
| Q3.1.secondary | Independent 0/4/7 timeline tests | Urgent ends 7; low Suspend13/Restart17/Abort4; old completion10 is stale | libs/kairos / worker |
| Q3.1.nested | Nested/multiple-victim/eligibility tests | Equal/nonpreemptible/zero-remaining tasks are not eligible victims | libs/kairos / worker |

## Q3.2

Parent plan: [Q3.2](../../tracks/des_queue_preemption_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| Q3.2.victim | Deterministic eviction selection | Strictly worse eligible priority; latest original sequence and request-ID tie rules | libs/kairos / worker |
| Q3.2.transaction | Atomic replacement and rollback | One victim/unit replaced with conserved capacity and no intermediate callback mutation | libs/kairos / worker |
| Q3.2.accounting | Elapsed/remaining/cumulative work updates | Suspend retains progress; Restart resets remaining but preserves effort; no negative ticks | libs/kairos / worker |
| Q3.2.revisions | Attempt/token invalidation and suspended cancel | Old completion cannot complete new attempt; cancelled suspended work cannot resume | libs/kairos / worker |

## Q3.3

Parent plan: [Q3.3](../../tracks/des_queue_preemption_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| Q3.3.resume | Suspend and Restart continuation handlers | Stored context resumes; restart reuses draw and emits correct attempt notification | libs/kairos / worker |
| Q3.3.abort | Abort terminal handler | No subsequent grant/resume and one terminal transition | libs/kairos / worker |
| Q3.3.cycles | Repeated-preemption/stale-event property fixtures | Effort reconciles across cycles and every transition emitted exactly once | libs/kairos / worker |

## Q3.4

Parent plan: [Q3.4](../../tracks/des_queue_preemption_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| Q3.4.evidence | Phase evidence reconciliation | Every parent task has integrated output and executed gates; unresolved failures prevent acceptance | libs/kairos / worker |
| Q3.4.closeout | Coordinator phase acceptance record | Independent readback agrees with evidence and upstream gates; only coordinator updates checkbox/catalog/pin | libs/kairos / proposal_or_review |

## Q4.1

Parent plan: [Q4.1](../../tracks/des_queue_preemption_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| Q4.1.builder | Public fluent API fixtures | Validation has no partial attachment and handles expose pending admission correctly | libs/kairos / worker |
| Q4.1.callbacks | Non-reentrant notification fixtures | Observers see committed state; new requests dispatch later | libs/kairos / worker |
| Q4.1.shared | DES/ABM single-world/time fixture | Both adapters reference identical authoritative entity/time state | libs/kairos / worker |

## Q4.2

Parent plan: [Q4.2](../../tracks/des_queue_preemption_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| Q4.2.builder | Fluent builder implementation | Approved signatures submit checked commands rather than mutate arbitration inline | libs/kairos / worker |
| Q4.2.dispatch | Domain handler and notification dispatch | Unrelated domain events are handled explicitly; canonical notification order | libs/kairos / worker |
| Q4.2.codec | Registered context encode/decode implementation | Unknown codec/revision rejected; valid suspended context roundtrips | libs/kairos / worker |
| Q4.2.limits | Zero-duration loop budget and legacy example | One same-tick completion; bounded feedback gives structured failure; old FIFO API still works | libs/kairos / worker |

## Q4.3

Parent plan: [Q4.3](../../tracks/des_queue_preemption_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| Q4.3.emit | Lifecycle record producer | No double-counted release/completion; key is run/event/transition ordinal | libs/kairos / worker |
| Q4.3.encode | Arrow sidecar encoding | Schema matches owner04 types and event_log.v1 remains unchanged | libs/kairos / worker |
| Q4.3.snapshot | Queue/context/scheduler snapshot integration | Sequences/revisions/leases/commands/notifications/RNG restored exactly | libs/kairos / worker |
| Q4.3.resume | Interrupted checkpoint parity fixture | Uninterrupted and resumed canonical resource outcomes match | libs/kairos / worker |

## Q4.4

Parent plan: [Q4.4](../../tracks/des_queue_preemption_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| Q4.4.staff | Named staff urgent-interruption example | Uses public builder only and retains context/priority semantics | libs/kairos / worker |
| Q4.4.bed | Staged bed/cleaning example | No premature reuse, unsupported atomic claim or clinical rule in generic core | libs/kairos / worker |
| Q4.4.join | Composed Flow example evidence | One shared state conserves resources and survives suspended checkpoint | libs/kairos / proposal_or_review |

## Q4.5

Parent plan: [Q4.5](../../tracks/des_queue_preemption_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| Q4.5.evidence | Phase evidence reconciliation | Every parent task has integrated output and executed gates; unresolved failures prevent acceptance | libs/kairos / worker |
| Q4.5.closeout | Coordinator phase acceptance record | Independent readback agrees with evidence and upstream gates; only coordinator updates checkbox/catalog/pin | libs/kairos / proposal_or_review |

## C0.1

Parent plan: [C0.1](../../tracks/empirical_calibration_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| C0.1.placement | Calibration crate/Arrow feature placement ADR | Reusable math separate from CLI and core scheduling; smallest compatible change | libs/kairos / worker |
| C0.1.hooks | Runner/model-adapter interface proposal | Exact owner21/22 hooks and missing VVUQ reference resolved before implementation | libs/kairos / worker |

## C0.2

Parent plan: [C0.2](../../tracks/empirical_calibration_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| C0.2.trace | Trace identity/time/mapping schema proposal | Occurrence/recorded/message time and episode-end/physical-departure remain distinct | libs/kairos / worker |
| C0.2.provenance | Missing/censor/knowledge/role schema proposal | No future-outcome leakage; source precision and observed/derived/defaulted lineage explicit | libs/kairos / worker |
| C0.2.fidelity | Fidelity precedence and ledger/probe contract | Mode changes cannot discard work; observed clamps differ from predicted residuals | libs/kairos / worker |
| C0.2.metrics | Residual/metric/seed schema proposal | Units/validity/counts and versioned purpose streams preserve existing telemetry contract | libs/kairos / worker |

## C0.3

Parent plan: [C0.3](../../tracks/empirical_calibration_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| C0.3.sources | Public-data/method/standards verification record | Primary fields/versions/licences retained; no claimed local conformance from standards existence | libs/kairos / worker |
| C0.3.oracles | Synthetic calibration and split/tolerance proposal | Identifiable/confounded cases and objective/scales frozen before held-out exposure | libs/kairos / proposal_or_review |
| C0.3.dependencies | Arrow feature/MSRV decision handoff | Exact supported dependencies reconcile D1 and owners25/30 without unsupported latest claims | libs/kairos / worker |

## C0.4

Parent plan: [C0.4](../../tracks/empirical_calibration_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| C0.4.evidence | Phase evidence reconciliation | Every parent task has integrated output and executed gates; unresolved failures prevent acceptance | libs/kairos / worker |
| C0.4.closeout | Coordinator phase acceptance record | Independent readback agrees with evidence and upstream gates; only coordinator updates checkbox/catalog/pin | libs/kairos / proposal_or_review |

## C1.1

Parent plan: [C1.1](../../tracks/empirical_calibration_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| C1.1.formats | Independent IPC/Parquet read/write fixtures | Real external reader/writer parity, not custom smoke self-roundtrip | libs/kairos / worker |
| C1.1.timestamps | Timezone/units/DST/overflow fixtures | Ambiguous conversion rejected or explicit policy; no fabricated seconds | libs/kairos / worker |
| C1.1.quality | Null/duplicate/chronology/censor fixtures | Stable exclusion counts and distinct physical/administrative endpoints | libs/kairos / worker |
| C1.1.mapping | Negative semantic-transform fixtures | meta.lastUpdated/MSH-7/A08/OMOP defaults cannot silently become physical observations | libs/kairos / worker |

## C1.2

Parent plan: [C1.2](../../tracks/empirical_calibration_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| C1.2.deps | Reviewed optional IO feature/dependency patch | Feature-minimal/MSRV build preserved; no Arrow dependency in generic queue core | libs/kairos / worker |
| C1.2.ipc | Bounded IPC RecordBatch IO | Typed nullable schema roundtrips through independent implementation with memory limits | libs/kairos / worker |
| C1.2.parquet | Bounded Parquet IO | Row-group batching preserves types/units and independent reader parity | libs/kairos / worker |
| C1.2.legacy | Smoke-format compatibility check | Legacy bytes remain identified as custom format, never advertised as IPC | libs/kairos / worker |

## C1.3

Parent plan: [C1.3](../../tracks/empirical_calibration_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| C1.3.normalize | Origin/timestamp/mapping normalization | Checked signed conversion and quality lineage retained per accepted schema | libs/kairos / worker |
| C1.3.sort | Stable external sorting | Canonical output hash invariant across physical row orders/batch sizes with stable keys | libs/kairos / worker |
| C1.3.validate | Partial-order/occupancy exclusions | Invalid chronology/capacity gives declared errors or diagnostic exclusion, never silent repair | libs/kairos / worker |
| C1.3.provenance | Ingestion count/hash manifest | Accepted/excluded/censored/input counts reconcile with hashes and privacy-safe identifiers | libs/kairos / worker |

## C1.4

Parent plan: [C1.4](../../tracks/empirical_calibration_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| C1.4.evidence | Phase evidence reconciliation | Every parent task has integrated output and executed gates; unresolved failures prevent acceptance | libs/kairos / worker |
| C1.4.closeout | Coordinator phase acceptance record | Independent readback agrees with evidence and upstream gates; only coordinator updates checkbox/catalog/pin | libs/kairos / proposal_or_review |

## C2.1

Parent plan: [C2.1](../../tracks/empirical_calibration_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| C2.1.mode | Mode resolution/boundary fixtures | Macro emits no transit; active/suspended work cannot disappear on mode changes | libs/kairos / worker |
| C2.1.paired | Paired Macro/zero-transit Micro oracle | Same purpose draws and equal canonical outcomes in zero-transit fixture | libs/kairos / worker |

## C2.2

Parent plan: [C2.2](../../tracks/empirical_calibration_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| C2.2.durations | Empirical intrinsic-work provider | Samples exclude queue/transit; units/support/invalid parameters checked | libs/kairos / worker |
| C2.2.policy | Fidelity resolution implementation | Declared precedence applies at approved task boundary only | libs/kairos / worker |
| C2.2.seeds | Stable task/purpose stream integration | No thread/wall-clock identity in keys; resume preserves mode and stream state | libs/kairos / worker |

## C2.3

Parent plan: [C2.3](../../tracks/empirical_calibration_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| C2.3.routing | Deterministic shortest-path implementation | Known route length correct; equal routes tie stably; unreachable gives typed error | libs/kairos / worker |
| C2.3.time | Movement-mode/tick conversion | Positive valid speed and checked distance/time rounding; no wrap/negative durations | libs/kairos / worker |
| C2.3.progress | Interruptible transit progress | Checkpoint/interruption preserves edge/progress and remaining duration | libs/kairos / worker |
| C2.3.dispatch | Minimal urgency/FIFO/zone/skill adapter | Eligible staff only; work/walk/wait separate and cleaning/reservation remains explicit | libs/kairos / worker |

## C2.4

Parent plan: [C2.4](../../tracks/empirical_calibration_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| C2.4.evidence | Phase evidence reconciliation | Every parent task has integrated output and executed gates; unresolved failures prevent acceptance | libs/kairos / worker |
| C2.4.closeout | Coordinator phase acceptance record | Independent readback agrees with evidence and upstream gates; only coordinator updates checkbox/catalog/pin | libs/kairos / proposal_or_review |

## E0.1

Parent plan: [E0.1](../../tracks/generic_ed_delivery_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| E0.1.schema | Scenario validation negative fixtures | Invalid count/units/schema or unresolved required input fails clearly | . / worker |
| E0.1.boundary | Native module-boundary check | ED policies/site inputs separate from Kairos reusable core and UI | . / worker |
| E0.1.patient | One-patient expected trace | Arrival/start/end and final count are hand-computable with fixed seed | . / worker |

## E0.2

Parent plan: [E0.2](../../tracks/generic_ed_delivery_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| E0.2.workspace | Minimal Rust library/workspace scaffold | Builds with reviewed toolchain/path pin and no UI/network/runtime Python dependency | . / worker |
| E0.2.config | Public config/errors and named-location types | Counts/staff/zones/optional route units match P0 contract and explicit defaults | . / worker |
| E0.2.runner | Minimal serial executable runner | Documented command runs one-patient example and returns useful typed errors | . / worker |
| E0.2.outputs | Basic summary/run manifest | Units/seed/horizon/input hashes and patient totals reflect the executed run | . / worker |

## E0.3

Parent plan: [E0.3](../../tracks/generic_ed_delivery_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| E0.3.evidence | Shared catalogue/domain assumptions review | Reuse P catalogue; no duplicate source inventory or unlabelled empirical default | . / worker |
| E0.3.boundary | Bed-offer/offload boundary contract | Compatible finite offers, explicit lifetime/transfer and distinct ambulance clocks | . / worker |
| E0.3.metrics | Metric/horizon/warmup/support definitions | Wait/LOS/throughput/utilization denominators and censoring specified before acceptance | . / worker |

## E0.4

Parent plan: [E0.4](../../tracks/generic_ed_delivery_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| E0.4.evidence | Phase evidence reconciliation | Every parent task has integrated output and executed gates; unresolved failures prevent acceptance | . / worker |
| E0.4.closeout | Coordinator phase acceptance record | Independent readback agrees with evidence and upstream gates; only coordinator updates checkbox/catalog/pin | . / proposal_or_review |

## E1.1

Parent plan: [E1.1](../../tracks/generic_ed_delivery_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| E1.1.endpoints | Empty/one-patient/pathway fixtures | Each patient is terminal or explicitly unfinished exactly once | . / worker |
| E1.1.extremes | Zero/long service and saturated-demand fixtures | Bounded run handles overload and no false completion | . / worker |
| E1.1.invalid | Impossible-resource/config fixtures | Unsupported combinations fail early without partial state | . / worker |

## E1.2

Parent plan: [E1.2](../../tracks/generic_ed_delivery_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| E1.2.arrivals | Seeded time-varying arrival generator | IDs and sequence stable; configured demand model and dependence match P contract | . / worker |
| E1.2.triage | Triage/assessment transitions | Correct queues/acuity and eligible work assignments without hidden queue duration in samples | . / worker |
| E1.2.diagnostics | Diagnostic/treatment transitions | Completion/repeat paths conserve patients/resources and use typed work durations | . / worker |
| E1.2.disposition | Disposition state/knowledge transitions | Provisional decision separate from realized outcome and no future information used | . / worker |

## E1.3

Parent plan: [E1.3](../../tracks/generic_ed_delivery_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| E1.3.offers | Compatible finite offer consumption | One offer consumed once; wrong destination cannot accept; expiry policy explicit | . / worker |
| E1.3.boarding | Boarding retention and recurrent care | ED space retained to physical transfer; recurrent tasks continue without full-time nurse reservation | . / worker |
| E1.3.exits | Discharge/optional abandonment/transfer | Terminal reason and timestamp correct; disabled policy never silently activates | . / worker |
| E1.3.horizon | Incomplete/censored endpoint reporting | Horizon truncation not completion; no causal ward/fleet claims from external offers | . / worker |

## E1.4

Parent plan: [E1.4](../../tracks/generic_ed_delivery_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| E1.4.evidence | Phase evidence reconciliation | Every parent task has integrated output and executed gates; unresolved failures prevent acceptance | . / worker |
| E1.4.closeout | Coordinator phase acceptance record | Independent readback agrees with evidence and upstream gates; only coordinator updates checkbox/catalog/pin | . / proposal_or_review |

## E2.1

Parent plan: [E2.1](../../tracks/generic_ed_delivery_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| E2.1.staff | Shift/break/skill-zone fixtures | Ineligible/unavailable staff cannot claim and in-flight shift policy follows shrink contract | . / worker |
| E2.1.interrupt | Urgent/handover fixture | Clinical task policy and resource consent both required; work accounting conserved | . / worker |
| E2.1.space | Cleaning/reservation/diagnostic/boarding tests | Space unavailable while occupied/cleaning/reserved; blocking cannot leak capacity | . / worker |

## E2.2

Parent plan: [E2.2](../../tracks/generic_ed_delivery_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| E2.2.dispatch | Minimal agent task selection | Eligible urgency/FIFO dispatch matches approved contract and deterministic ties | . / worker |
| E2.2.fidelity | Micro/Macro work-route integration | One authoritative state; no duplicate transit or animation-driven time | . / worker |
| E2.2.claims | Staged staff/space ownership integration | Single-unit/resource limitation explicit; no implicit atomic multi-resource grant | . / worker |
| E2.2.limits | Unsupported multi-role policy validation | Sedation/resus/assisted transfer rejected or explicitly approved dedicated composite approximation | . / worker |

## E2.3

Parent plan: [E2.3](../../tracks/generic_ed_delivery_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| E2.3.fixture | Integrated nominal ED fixture | No duplicate/leaked allocation; simultaneous/restart/resume cases conserve patients and work | . / worker |
| E2.3.boundary | ST001–ST009 acceptance cases | Compatible offers/retained care/offload sensitivity and unsupported claims match reviewed expected outcomes | . / worker |
| E2.3.runner | Config-only nominal and constrained MVP runs | Fresh user runs both without Rust edits/private data/UI; malformed input returns clear error | . / worker |
| E2.3.summary | CSV/JSON outputs and capacity-comparison oracle | Wait/throughput/occupancy counts reconcile; fixed seed repeats and known constrained fixture changes as expected | . / worker |
| E2.3.handoff | MVP reproduction recipe and shared C6 fixture | Exact commands/config/seed/hash/results retained with limitations; no v1 completion claim | . / worker |

## E2.4

Parent plan: [E2.4](../../tracks/generic_ed_delivery_20260925/plan.md)

| Leaf | Output | Oracle | Repository / role |
| --- | --- | --- | --- |
| E2.4.evidence | Phase evidence reconciliation | Every parent task has integrated output and executed gates; unresolved failures prevent acceptance | . / worker |
| E2.4.closeout | Coordinator phase acceptance record | Independent readback agrees with evidence and upstream gates; only coordinator updates checkbox/catalog/pin | . / proposal_or_review |

