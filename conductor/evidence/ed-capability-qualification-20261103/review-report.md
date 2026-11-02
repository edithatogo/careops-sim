# Review Report: Generic ED — E9 capability reconciliation

## Summary
The bounded CPU/CLI capability repair at 18b6e8f passes independent Conductor review and full applicable hosted source gates after both diagnostic findings were fixed.

## Verification Checks
- [x] **Plan Compliance**: Yes - E9.1–E9.4 accepted; E0–E8 history preserved; optional ED G3 and separate Polars/clinical/site/release gates remain open.
- [x] **Style Compliance**: Pass - Rustfmt/Clippy and existing API conventions; no new dependency or runtime pin changes.
- [x] **New Tests**: Yes - Five behavior controls plus duplicate/malformed/CPU-command negatives added for the review fixes.
- [x] **Test Coverage**: Partial - Current CPU/CLI behavior, retained API/batch/recovery/export controls and exact critical-ED oracles; no device/protocol/multi-host or clinical acceptance.
- [x] **Test Results**: Passed - 42 targeted Rust tests initially;5 focused controls after each diagnostic fix; actual native CLI parity/diagnostic readback;766full hosted Python tests; all15 applicable hosted successes plus expected main-only attestation skip.

## Applied findings

- Independent review at 8a283c1 found early unavailable-mode exit8 masked later duplicate/missing backend syntax. bfce6be fixes ordering after a failing control. Exact committed re-review was ready.
- GitHub automated review found wrong-command CPU mode selection incorrectly classified as runtime 8. 18b6e8f returns usage 2, with both controls updated after a meaningful red8versus2 run. Independent re-review was ready; both review threads resolved.
- No remaining in-scope findings. Upstream device/protocol/collective defects are explicitly retained in the exact-source owner handoff and are not certified by this review.
