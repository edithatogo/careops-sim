# Reports 26–30: integration decisions

Date: 2026-09-27. Planning evidence only; no implementation, runtime qualification,
source-version verification or clinical validation is claimed by this intake.

## Intake and provenance

Original bytes and SHA-256 hashes are retained in the [manifest](supplied/20260927/manifest.json).
Reports 23/24/25 match archived reports 12/11/10 respectively, byte for byte.
Their aliases are recorded without duplicate archive files.

| Report | Research response | Task destinations |
| --- | --- | --- |
| [26](supplied/20260927/report-26.md.txt) | R8a Metal/MLX | E6, D1 |
| [27](supplied/20260927/report-27.md.txt) | R7 queue conformance | Q0/Q3/Q5 |
| [28](supplied/20260927/report-28.md.txt) | R8d staged CI/security | D1–D4 |
| [29](supplied/20260927/report-29.md.txt) | R5 minimum ED boundary | E0/E1/E2, P1/P4 |
| [30](supplied/20260927/report-30.md.txt) | R6 minimal event crosswalk | C0/C1, P4 |

The sessions lacked important local specifications. Local contracts take precedence
in reconciliation. Commands inside reports remain unexecuted proposals. Linked
sandbox bundles are not available here; report counts/digests do not authenticate
unseen artifacts. Embedded citation tokens are retained, not treated as locally
resolved source verification. Versions, licences, vulnerabilities and performance
claims require primary-source verification at the task that adopts them.

## Metal: retain the Rust-native architecture

Profile elementwise transforms, sorted W1 reductions, KS/CDF, histogram, scan,
candidate evaluation and full-sort costs before selecting a supported kernel.
The equal-sized presorted W1 example is not a replacement for C4's general
weighted/unequal-sample metric. Host-generated fixed inputs and deterministic
indexing retain CPU RNG, clock, queue arbitration and event-insertion authority.

Measure cold startup/compilation, staging, encoding, kernel, synchronization,
readback and end-to-end time separately, plus resident chains, peak memory and
median/p95. Unified physical memory does not prove zero-copy API behavior; lazy
MLX execution must be forced before ending timings. Compare with production
multicore CPU while retaining Track 32's existing million-agent/single-thread
performance target as a separate gate. No reported benchmark has been run here.

Freeze per-kernel/dtype tolerances from intended use before held-out evaluation;
never enlarge them simply to pass a backend. Integer counts/order remain exact,
with overflow tests. Float ranking or threshold changes require CPU recheck or
explicit failure/fallback, not silently different authoritative decisions.

wgpu remains the planned owner-32 route. MLX's C/C++ implementation and Rust FFI
wrapper are possible comparison tooling, not a Rust-native replacement module.
Reported wgpu/MLX/CubeCL/Candle versions and MSRVs remain dependency candidates;
no package, backend or submodule was added by this research intake.

## Queue semantics: preserve settled local choices

Report 27 did not have the local Q specification. Resolve its open questions as
follows, with Q0/Q1/Q2 executable contract tests still outstanding:

| Question | Local required behavior |
| --- | --- |
| Capacity shrinks below active allocations | Reject atomically; do not accept temporary over-capacity drain |
| Capacity zero | Allowed when no allocations remain |
| Deadline and release at T | Deadline is exclusive; never grant at T |
| Completion and preemption at T | Materialize completed work first; no zero-remaining victim |
| Restart sampling | Reuse original draw; no new random sample |
| Equal resource priorities | FIFO original admission sequence; no equal-priority eviction |
| Request preemption flags | Incoming permission and victim interruption strategy remain separate |

Add a second synthetic timeline: A starts at 0 with duration 10; urgent B arrives
at 4 with duration 3. B completes 7; A completes 13 for Suspend, 17 for Restart,
or aborts at 4. A's old completion at 10 must not mutate current state. These
are hand-derived expected tests, not completed executions. Existing 0/3/5 fixture
remains unchanged. Pin SimPy source/version for bounded differential tests and
record intentional differences, including owner-checked release and ties.
AllOf over independent requests is not atomic multi-resource acquisition.

## CI/security: concrete staged acceptance

D1 verifies advertised MSRV and current consumer compatibility, tool/action pins
and reviewed feature/toolchain transitions. D2 checks full dependency policy,
immutable action commits, exact recursive source pins and secret/cache isolation.
The report's workflow defect list is an audit input requiring current local source
readback, not a claim that this turn reproduced every defect. Its upstream R0–R4
maturity terminology does not rename this project's D0–D5 phases or move D2.

D3 preserves independent bidirectional real Arrow tests, bounded feature matrices,
separate doctests and measured mutation/semver baselines. Fuzz/Miri/sanitizer lanes
must use supported dated toolchains; advisory claims require verification. Arrow
C Data raw pointers are a trusted in-process interface; process boundaries use
bounded validated IPC. D4 binds build-once artifact bytes, checksums, SBOM and
attestation, with independent consumer verification and rejection of substituted
or missing bytes. Clean release inputs and immutable-version rollback rehearsals
precede publication; this intake does not authorize publication.

## ED boundary: finite downstream opportunities

E0 freezes finite destination-compatible ED-eligible bed offers as the minimum
external admission boundary. An offer represents a ready eligible opportunity,
including declared competing-demand assumptions; raw ward discharge timestamps
are insufficient. Offer lifetime, persistence, withdrawal and unused-offer policy
must be explicit, evidenced or labelled synthetic. Immediate expiry is not an
accepted universal default. Transfer delay and physical departure remain separate.

Boarders occupy their ED location until physical transfer and remain eligible for
recurrent care tasks; do not reserve a full-time nurse per boarder by default.
Ambulance arrival, triage, responsibility/handover, offload and crew release remain
distinct. Receiving-resource constraints can affect offload; an external arrival
stream does not establish fleet/community response effects. External offer
scenarios do not establish causal effects of ward discharge policies.

Q v1 supports one unit of one resource per work item, not an extra space resource
implicitly attached to a staffing grant. Separate occupancy/reservation lifecycles
need explicit coordination and no double allocation. Simultaneous multi-person
sedation, resuscitation and assisted transfer are unsupported or visibly qualified
approximations until separately designed. A composite team is acceptable only
with dedicated/non-overlapping membership and declared limits on role-specific
staffing inference. No silent atomic claims or unsupported deterioration hazards.

E2's planned ST001–ST009 cases cover compatible offers, unused offers, retained
occupancy, ongoing boarder care, saturated offload, receiver-staff effects, explicit
sedation/resuscitation limits and ward-policy inference limits. Later clinical
services and whole-hospital modelling remain roadmap items without detailed tracks.

## Trace semantics and candidate crosswalk

C0's existing `observed_at` is now explicitly occurrence time. Planned nullable
`recorded_at` and `message_created_at`, with per-time lineage, preserve distinct
clocks without creating another simulation clock. Keep location intervals and
separate episode-end/physical-departure events. Report 30's eleven provisional
field IDs do not replace canonical case/event IDs or mandate a standards platform.

Candidate mappings for source verification during C0/C1:

| Source | Preserve | Do not assume |
| --- | --- | --- |
| FHIR R4/AU | Event-specific periods, location periods, linked audit recording | meta.lastUpdated equals source event recording; encounter end equals both ED endpoints |
| HL7 v2 | EVN-6 occurrence, EVN-2 recording, MSH-7 message time with profile evidence | A08 proves physical movement; A03 is always physical departure |
| AIHW ED milestones | Non-admitted episode end and physical departure separately | Episode reporting supplies staff/bed intervals or local source availability |
| OMOP | Visit/detail projection plus ETL lineage | Derived/defaulted visit end is observed physical departure |

Reported editions FHIR R4 4.0.1, AU Base 6.0.0, AU Core 2.0.0, HL7 v2.5.1,
NAPEDC 2026–27 and OMOP 5.5 are research baselines to verify and pin, not an assertion
of Cairns conformance or today's latest releases. Preserve source minute precision,
unknown timezone and observed/derived/defaulted status. Missing endpoints stay
unknown; no synthetic seconds or interchangeable timestamps. Cancellation-aware
movement reconstruction requires a site location dictionary/boundary.

C1 negative cases reject reversed intervals, departure before known arrival and
four misleading transformations in the table. Positive synthetic chronology:
09:14 arrival, 09:19 triage, 09:47 care start, 12:55 episode end, 13:32 boarding
location move, 14:16 physical departure. This illustrates valid retained ED
occupancy, not empirical input or a universal pathway.

## Remaining evidence and bounded execution

All requested research themes now have narrative responses. Obtain the actual
bundles for row-level reconciliation, source manifests and fixture import rather
than repeating broad searches. Reported new bundles contain 7 workloads/8 options
(26), 11 conformance rows/14 tests/40 timeline rows (27), 12 fixtures/15 commands/
16 controls (28), 21 decisions/9 stories (29), and 44 mappings/20 sources (30).
These are report claims, not locally counted artifact rows.

The existing 143-task DAG and phase gates are retained. Added objectives must be
split into bounded leaf packets (one oracle/output each) before dispatch; research
intake does not make long task descriptions safe single-worker packets. No task
checkbox, current next-task, upstream completion status or parent pin is advanced.
