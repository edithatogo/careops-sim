# Decomposition guide for all four tracks

The catalog covers every plan task. The rows below constrain **how to prepare
small worker packets** within those tasks; they do not mark code ready or bypass
phase dependencies. Freeze task-specific signatures/paths/oracles during contract
phases and bind them into packets before dispatch. Every split has an integration
join at the original task and another at phase closeout.

## Queue/preemption track

| Phase | Suggested worker leaves | Coordinator/reviewer responsibility |
| --- | --- | --- |
| Q0 | Extract existing signatures; enumerate event-kind usage; encode reviewed golden traces | Choose/freeze runtime API, same-time semantics, codecs and compatibility ADR |
| Q1 | One component schema at a time; live-handle validation; lease release; despawn cleanup; one invariant fixture each | Approve shared types/registry boundaries; integrate full capacity conservation |
| Q2 | Priority-key ordering; queue insert/remove/rekey; deadline admission; timeout token; capacity drain | Verify exclusive deadline and scheduler/resource-priority distinction |
| Q3 | Progress arithmetic; deterministic victim selection; stale-token guard; Suspend; Abort; Restart | Golden low(10)/urgent(2) fixture: low completes 12/absent/15; high=5; integrated single notifications |
| Q4 | Builder validation; notification dispatch; one registered context codec; resource sidecar encoding; checkpoint restore; migration example | Single world/clock, public API and cross-crate checkpoint integration |
| Q5 | One conformance fixture; one benchmark workload; compatibility documentation; repeat/worker-count comparison | Accept whole suite and measured performance; upstream closeout and pin |

Q3.2 accounting can prepare independently of token-guard tests after fixed types;
eviction integration waits for both. Strategy implementations share state-machine
code, so use distinct strategy modules only if Q0 approved that interface;
otherwise serialize them. Do not force a new architecture to create parallel work.

## Calibration track

| Phase | Suggested worker leaves | Coordinator/reviewer responsibility |
| --- | --- | --- |
| C0 | Inventory actual fields/metadata; draft schema tables; construct known timestamps | Set mapping, mode, probe isolation, objective and statistical validity contracts |
| C1 | IPC reader; Parquet reader; timezone normalization; chronology checks; stable sort; provenance output | Single schema/IO interface; dependency/MSRV and combined row-count/hash tests |
| C2 | Fidelity precedence; immutable task mode; distribution adapter; purpose seed keys; shortest-path tie fixture; transit progress | No double-counting, correct staff/clinical assumptions, shared Flow integration |
| C3 | Anchor ledger; snapshot extraction; isolated probe lifecycle; early/late residual recording; probe resume | No future-state leak; late/missing target and infeasible occupancy review |
| C4 | W1 implementation/tests; KS D implementation/tests; paired summaries; counts/validity flags; Arrow metric writer | Fixed metric interface and tolerance; W1([0,2],[1,3])=1, KS=.5; avoid unjustified p-values |
| C5 | Candidate enumeration; seed allocation; objective assembly; stable ranking; CLI adapter; worker shard merge; resume | Split integrity, no completion-order-dependent search, existing manifest compatibility |
| C6 | Known-speed recovery case; confounded case; held-out report; one compatibility/benchmark run | Identifiability/statistical review and actual generic-ED model validation |

IPC/Parquet readers may run in parallel after a common trait is frozen, in distinct
files. W1/KS can similarly run independently after metric types and reference
fixtures are accepted. Schema/Cargo.lock integration is a separate exclusive task.

## Development-readiness track

| Phase | Suggested worker leaves | Coordinator/reviewer responsibility |
| --- | --- | --- |
| D0 | Manifest inventory canary; upstream gate extraction; broken-reference report | Required/deferred scope, capability readiness, dependency closure and profile ADR |
| D1 | One tool pin/bootstrap check; one negative context eval; one bounded skill draft; one evidence schema check | MSRV/compatibility and skill trust/permission decisions; qualify model/task class |
| D2 | Workflow static check; one local CI job; one negative required-check fixture; repository settings readback | Repo creation gate/visibility, branch rules, secrets, required checks and shared CI integration |
| D3 | One property/fuzz harness; one mutation target; one feature/API lane; one benchmark | Choose risk coverage, thresholds and security-query scope; avoid false passes |
| D4 | Package dry-run; SBOM/checksum fixture; clean-consumer test; restore rehearsal | Threat model, release holds, provenance trust and any publication authority |
| D5 | No-change report; idempotency/timeout fixture; dependency grouping rule | Automation budget/permissions/pause policy, enablement and merge authority |

The supplied D0.2.inventory packet is a low-risk extraction canary, not the full
capability/architecture decision. After its output, a coordinator can assign an
independent gate-extraction leaf and integrate both into D0.2.

## Generic ED track

| Phase | Suggested worker leaves | Coordinator/reviewer responsibility |
| --- | --- | --- |
| E0 | Config validation fixture; minimal crate/example; one public-source record | Package boundaries, generic/site split, metric definitions, licensing |
| E1 | Arrival schedule; one pathway transition; one disposition; censoring/horizon accounting | Patient conservation and service-vs-wait semantics across the complete pathway |
| E2 | One shift boundary; skill/zone filter; cleaning lifecycle; one reservation rule; interruption fixture | Multi-resource/shift policy and full shared state/queue integration |
| E3 | Config error mapping; one CLI operation; cancellation; atomic artifact writer; one recovery fixture | Cross-crate seed/state/artifact consistency and public API |
| E4 | One analytic case; downstream install example; one workload benchmark; support docs | Statistical/domain review, release profile and completion evidence |
| E5 | One snapshot field mapping; one worker command; one screen interaction; render-rate test | ABI/time/ID widths, buffer ownership, browser/native parity and accessible product flow |
| E6 | One fixed-signature WGSL kernel; device capability error; CPU/device oracle; benchmark capture | Select worthwhile kernel, numerical/ranking contract and actual Metal acceptance |
| E7 | One message codec; one lookahead fixture; worker queue adapter; GVT test | LP/resource ownership, causality, final-state parity and real threaded acceptance |
| E8 | One transport envelope; dedup guard; reconnect fixture; MPI/gRPC smoke | Auth/security, migration ownership, recovery and real multi-node acceptance |

Cairns and later clinical-domain details remain outside these generic-model
packets. Hardware/runtime availability is a prerequisite for the relevant leaf;
an unavailable backend is `blocked`, never replaced with an unlabeled mock pass.

## Integration checklist for every parent task

- Every declared leaf has an accepted result against its recorded base/input hashes.
- Changed paths match reservations; shared files were integrated by their owner.
- Public contract/API/schema changes have the required reviewed decision.
- Combined behavior tests pass on the integrated head, including previously
  passing sibling leaves. Test counts/artifacts correspond to that head.
- Only then update the parent checkbox/catalog and unlock dependents.
