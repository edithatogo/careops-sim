# Risk register — queues

| Risk | Impact | Mitigation / blocking evidence | Owner |
| --- | --- | --- | --- |
| Separate DES and ABM worlds drift | Wrong agent/resource state | Additive shared runtime ADR and integration fixture before Q4 acceptance | 03/01 |
| Same-tick timeout/preemption/completion ambiguous | Nondeterministic/double work | Explicit exclusive deadline/half-open busy interval, insertion-order tests, revision guards | 03/12 |
| Arbitrary context cannot checkpoint | Resume changes behavior | Registered versioned codecs and exact resumed trace comparison | 03/01/22 |
| Public DESContext change breaks callers | Compatibility regression | Retain legacy struct/API, additive runtime, protected-surface review | 03/25 |
| Strict priority starves routine work | Misleading model result | Document lack of starvation guarantee; track waits/censoring, explicit model policy | 03/CareOps |
| Sequential multiple claims deadlock | Run stalls | One resource per work item; staged example; no claim of atomic multi-resource support | 03 |
| Restart repeats model side effects | Double-counted actions | Explicit restartable context contract and attempt audit; simulation-local hooks | 03 |
| Local queue incorrectly shared across LPs | Causality/race failure | LP ownership contract; reject unsupported cross-LP access; Track 34 gate | 34/35 |
| Latest dependency/toolchain breaks support | Unbuildable release | Pin resolved versions, preserve supported matrix or approved migration | 25/30 |
| Passing toy benchmark misrepresents ED load | Performance regression | Queue/churn/interrupt/active-capacity matrix and existing acceptance gates | 12 |

## Historical Q4 disposition — superseded for current phase status by Q5 evidence below

Q4 local and hosted development evidence mitigates shared-world drift, lifecycle
ordering and staged single-resource integration within the frozen contracts.
Final parent integration remains pending. Portable checkpointing, starvation,
atomic multi-resource claims, cross-LP causality, public API compatibility and
realistic scaling remain open or explicitly unsupported. The dependency audit
records newer candidates without silently changing RNG/MSRV contracts. Closure
requires the owning phase evidence, not a design statement.


## Q5 current residual risks — 2026-10-05

Q5.1–Q5.3 are bounded development qualifications, not release acceptance. The
following remain explicit limits or gates handed to later owners:

- Strict priority has no aging/starvation guarantee; routine work can wait
  indefinitely unless the model supplies an explicit policy.
- Sequential multi-resource acquisition can deadlock; atomic/multi-resource
  acquisition is unsupported.
- Restart can repeat simulation-local side effects; the model must use the
  attempt/context contract and audit effects.
- Pause/continue evidence is same-runtime only. Portable checkpoint/restore is
  owned by Track 22 and is not a queue-track acceptance pass.
- Cross-LP shared queues remain unsupported pending Track 34/35 ownership,
  message, causal-time and lookahead contracts; zero-lookahead cycles risk
  deadlock.
- Metal/device Flow queue execution is unsupported; Track 32 owns future device
  parity.
- Public experimental API/version/symbol compatibility remains subject to
  Track 25 review. Security and formal release gates are open.
- Clinical calibration and complete ED MVP acceptance remain separate and open.

The coordinator reports the Q5.4 independent fixture/example and canonical-hash
replay complete. Exact-head hosted acceptance remains pending; these residuals
are not waived by Q5.1–Q5.3 receipts.

## Q5.4 disposition

Queue development risks covered by Q-01–Q-07 are accepted within the frozen contracts and source-bound receipts. Residual limits above are documented owner gates, not silently passing features. Independent review and exact-head development qualification passed; no formal release or clinical acceptance follows.
