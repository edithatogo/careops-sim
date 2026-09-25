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

All risks are open at planning time; closure requires phase evidence, not a design
statement. Q0 review may refine contracts without weakening determinism guarantees.
