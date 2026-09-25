# Risk register — empirical calibration

| Risk | Impact | Mitigation / blocking evidence | Owner |
| --- | --- | --- | --- |
| Smoke bytes mistaken for Arrow | Unusable empirical pipeline | Actual IPC/Parquet interoperability prerequisite C1 | 04 |
| Timestamp reflects charting rather than action | Biased parameters | Explicit mapping semantics, quality flags, observation support | 21/model owner |
| Service/transit/queue not identifiable | Plausible but false fit | Independent anchors, constrained parameters, confounding fixture, Macro fallback | 21 |
| Anchors erase errors | Artificially perfect fit | Unclamped probe predictions, signed residuals, free-running held-out validation | 21/22 |
| Late probe leaks future state | Invalid replay causality | Isolated ledger/probes, bounded endpoints and isolation tests | 21/03 |
| Ties/dependence invalidate KS p-values | Misleading statistical claim | D by default; explicit assumptions for inference and cluster-level resampling | 21 |
| Censoring/missing cases improve fit silently | Selection bias | Counts, coverage, penalties and declared estimand; no zero fill | 21 |
| Mixed fidelity counts transit twice | Biased delays | Distribution interval semantics and paired zero-transit fixtures | 03/21 |
| Candidate scheduling changes results | Irreproducible calibration | Stable seed keys, fixed batches, canonical reduction and worker/resume tests | 01/22 |
| Arrow upgrade raises MSRV | Breaks supported targets | Resolve versions/features with existing compatibility/toolchain owners | 04/25/30 |
| Renderer controls simulation timing | Frame-rate-dependent outcomes | Future immutable snapshot contract; native/Wasm parity and render-off test | 05/09 |
| Extra agent rules overfit | Poor generalization | Minimal interpretable rules and held-out Macro/Micro ablation | 21/CareOps |
| Large input exhausts memory | Failed batch jobs | Bounded reading, deterministic spill sort and memory benchmark | 04/22 |

All risks remain open until corresponding tests and data evidence exist. Synthetic
recovery establishes tool behavior, not suitability for a particular hospital.
