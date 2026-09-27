# Reports 9–12: integration decisions

Date: 2026-09-27. Status: research incorporated into planning; no runtime, agent
qualification, parameter reconciliation or phase-completion claim.

## Evidence intake

Original supplied reports are archived verbatim as inert Markdown text; hashes in
[manifest](supplied/20260927/manifest.json). Embedded commands were not run.

| Report | Result | Incorporated into |
| --- | --- | --- |
| [9](supplied/20260927/report-9.md.txt) | F3 gap audit; zero-row historical crosswalk, not successful reconciliation | P0/P1/P3 evidence and denominator gates |
| [10](supplied/20260927/report-10.md.txt) | Independent replication, conservative PDES and distributed test proposals | C5, E7/E8 |
| [11](supplied/20260927/report-11.md.txt) | Bounded development-worker harness and evaluation proposals | D0/D1/D5, existing execution model |
| [12](supplied/20260927/report-12.md.txt) | Partial-observation calibration, identifiability, holdout and uncertainty | C0/C3/C4/C5/C6, P3 |

Reports 10/11 include full fenced payloads. These have been extracted into
[inert payload files](supplied/20260927/inline-payloads/manifest.json), including
CSV proposals and unexecuted command examples. CSV parsing/column consistency is
only format evidence. Reports 9/12 reference sandbox bundles still unavailable here.
All four research sessions lacked important parent project attachments; the local
contracts govern their integration. No generated payload is a production schema.

## Local corrections and conflicts

1. Local `git -C libs/kairos show -s` verifies HEAD
   `fae901558f07b7b717a676adbafbe2cdc78dea1c`, parent
   `f11e0be7dcf8cad705741a44b651ca953e269ae2`, author/committer date
   `2026-05-20T08:41:11+10:00` (19 May UTC). Report 10's September date and
   `f26633…` parent are incorrect for this object. The parent project's submodule
   pin is HEAD itself, not HEAD's Git parent.
2. Licensing is inconsistent locally: Cargo.toml and README declare Apache-2.0;
   LICENSE.md declares Apache-2.0/MIT and LICENSE-MIT exists. Reports 11/12 reflect
   different surfaces. D0/D4 must reconcile packaging/license metadata; do not
   resolve legal intent from either narrative alone. MSRV 1.76 is a manifest
   declaration, not current dependency compatibility proof.
3. Report 10's counter-based RNG, envelope and checkpoint layouts are proposals.
   Reuse existing owner 01/22/34/35 contracts; any RNG change needs versioning and
   golden-vector compatibility. Do not rewrite RNG for scheduling convenience.
4. Preserve Track 34's observable-state equivalence contract. Trace, state and
   statistical equivalence are distinct; require trace identity only where already
   promised or in controlled serial/independent-replication fixtures. Statistical
   similarity alone cannot validate deterministic CPU PDES.
5. Independent distributed replicas are a useful E8 profile, not a new two-machine
   prerequisite for C5 or E7. Retain CPU → Metal/PDES/distributed project phases.
   No new speedup threshold replaces upstream performance gates.
6. Harness proposals extend tools/tasks.py and current packets, not a second DAG.
   Keep stable logical packet IDs plus distinct attempt IDs and immutable receipts;
   a changed contract gets a new packet revision/digest. Reject stale context before
   dispatch and acceptance. Git ancestry alone is weaker than current exact-base
   and input-hash checks. Managed worktree lifecycle tools take precedence over the
   report's raw worktree shell examples.
7. Report 11's public fixture descriptions are not secret held-out oracles. Create
   undisclosed variants outside worker-readable context and verify actual isolation.
   FIX-005 labels reject while asking a potentially correct unit conversion: split
   correct-output acceptance from seeded wrong-output rejection. Never hard-code
   rejection based solely on fixture ID. Repeated runs of one fixture are not
   independent negative cases for confidence-bound claims.
8. The 20×3 pilot, 60/20/20 data split, 0.25-tolerance MC half-width and 200/1000
   bootstrap counts are proposed policies, not accepted defaults. Choose thresholds
   before test exposure based on intended use, information and available budget.
   External model/release/skill version claims remain unverified candidates; no
   installation, model switch or promotion follows from this report.
9. Report 9 supports retaining the 3,180 count discrepancy as unresolved; a footnote
   for emergency-only presentations cannot prove the residual classification in
   an all-presentation table. Keep rounded Queensland benchmarks separate from
   exact generator probabilities. F3 remains incomplete without the actual registers.
10. Geography editions and hospital point coordinates are provenance/spatial anchors,
    not route graphs. Broader road, retrieval and catchment models remain outside
    current detailed ED scope. Synthetic demand profiles stay separate.

## Accepted planning refinements

### Calibration and parameter evidence

C0 records exogenous, directly observed primitive, clamp-only and target fields,
with temporal knowledge availability, risk start, last observed time, event/cause,
censor reason and cluster IDs where available. Separate training, selection and
locked test periods; boundary censoring prevents future completion leakage.

C3/C5 expose profile/grid ridges and bound hits. A uniquely selected tie-break is
not an identified parameter: materially different indistinguishable candidates
produce an identifiability status. Synthetic recovery alone establishes algorithm
behavior, not identifiability in real observations. Independent additional measures
or justified constraints are required for empirical component claims.

C4 retains W1/KS as marginal diagnostics and adds prespecified conditional/joint
checks. A simple paired joint table suffices initially; energy distance is optional
and must specify scaling and estimator. C6 separately validates held-out historical
exogenous arrivals with free endogenous flow, and the stochastic arrival generator.
These validation modes are orthogonal to Macro/Micro spatial fidelity.

C5/C6 distinguish stochastic-only, input-plus-stochastic and structural uncertainty;
resampling respects demonstrated patient/day/staff/shift dependence, with day blocks
a candidate rather than a universal correct unit. Preserve repeated-visit IDs where
available. Fixed budgets can contain prespecified batched precision checkpoints;
cap exhaustion gives MC-indeterminate. Use sequentially valid intervals or a fixed
confirmation sample, not repeated ordinary CIs presented as guaranteed coverage.
Retain bounded grid search, frozen objective/thresholds and fresh final-test seeds.

### Parallel runtime and recovery

E7 requires a boundary-by-boundary certificate of minimum causal timestamp advance.
Means/quantiles are not lookahead guarantees. Same-time cycles/zero lookahead must
produce approved safe closure/fallback or explicit unsupported rejection, never
unsafe advancement. Mutable resource/queue ownership is unique and explicit.

E8 evaluates versioned identity, deduplication and checkpoint publication with
in-flight messages accounted for. Reject obsolete ownership/incarnation traffic
under a documented recovery protocol; reconcile messages rather than blindly
losing valid in-flight work. Failure fixtures cover duplicate/reordered delivery,
crashes around checkpoint publication, corruption and bounded shutdown. Real
threads/processes/nodes and correctness gates precede profile performance claims.

### Development harness

D1 compares serial/parallel and no-skill/pinned-skill arms, including preflight-only
cases. Record accepted/rejected/escalated outcomes, known-bad acceptance and bad-
among-accepted separately, interventions, cost (unknown if unavailable), latency
and DAG makespan. Record model/harness/skill versions and immutable evidence.
Fresh review cannot waive deterministic gates. No autonomous repair swarm is enabled.

## Hand-computable candidate tests from report 12

These are synthetic proposed oracles, not executed Kairos tests:

| Case | Expected outcome | Task |
| --- | --- | --- |
| Times 1,2,4; event flags 1,1,0 | Exponential rate 2/7, mean 3.5; censor is not completion | C5.1 |
| Walk+work=5; grid solutions (1,4),(2,3),(3,2) | Non-identifiable; observing walk=2 resolves this synthetic grid only | C3.1/C5.1 |
| Pairs (0,0),(1,1) vs (0,1),(1,0) | Marginal distances zero, joint mismatch nonzero | C4.1 |
| [0,0,0,0,100] vs [0,0,0,0,1000] | W1=180, KS D=0.2 | C4.1 |
| Day clusters [0,0] and [10,10] | Two-day bootstrap mean variance 12.5; IID row mean variance 6.25 | C5.1 |
| One server; arrivals 0,1; durations 2 | Starts 0,2; waits 0,1; completions 2,4 | C6.1 |
| Start 3; training cutoff 4; completion 6 | Training duration 1, censored; future completion unavailable | C5.1 |

The report's illustrative forced second start at time 1 violates one-server
capacity. Test rejection/isolation in the observed ledger, not concurrent grants
as an acceptable replay behavior. All fixture implementations await owner review.

## Remaining actions

Obtain referenced bundles for reports 9/12 and earlier parameter reports; do not
repeat broad F3 research without supplying actual files. R8b/R8c and R4 now have
research responses; R5/R6/R7/R8a/R8d remain separate research options. No track
checkbox changes. New task text is still bounded/split through the existing worker
packet protocol before execution.

## Embedded-content clarification

See the [content audit](embedded-content-audit.md): substantial tables, fixtures
and payloads are already supplied inline. Missing standalone bundles do not
justify treating that content as absent or requiring another upload.
