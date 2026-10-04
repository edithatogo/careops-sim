# Q4 development phase acceptance

Q4.3 lifecycle snapshots and the opt-in 27-field Arrow/IPC sidecar are implemented.
Q4.4's runnable staff/bed/cleaning example and six frozen integration tests are
accepted locally. Both dispatch modes compare exact live state, typed context and
raw lifecycle output. Bed capacity remains held until cleaning completes.

Canonical current stable Rust 1.99.0 is pinned in both repositories. Clean Kairos
source `1123ad4bd0c9121a4a8f5f0be1229fbafc9861f6` passed:

- Combined DES/ABM/Arrow feature suite: 224 tests on Rust 1.99.0 and 224 on 1.88.0.
- Default Arrow: 23 tests on Rust 1.76.0; native-owner packages: 293 on 1.99.0.
- DES release: 174 tests; runnable example, formatting and scoped strict Clippy.
- Phase, DAG and strict clean-tree checks.
- [Two-host native owner run 37192093779](https://github.com/edithatogo/kairos/actions/runs/37192093779): all 11 jobs passed, with 46 hashed artifact files retained.

DES release tests passed despite rust-objcopy debug-stripping warnings; stripped
packaging is unverified. Broader pre-existing Arrow IO lint findings remain
separate from the scoped strict Clippy result.

Final governance commit `1125b5268bb349a5befe12f5789045042faab3e3` passed local phase,
DAG and strict clean-tree gates. [Exact-head hosted run 37192692770](https://github.com/edithatogo/kairos/actions/runs/37192692770) passed all 11 jobs. The parent pin/contract now records this qualified head, and Q4.5 is accepted for bounded development. Parent PR checks/merge remain pending.

The dependency audit is recorded in the Rust adoption receipt. It identifies
newer CI actions, Serde/thiserror patches and Wasm candidates without claiming
those candidates are adopted or qualified. The website npm advisory remains open.

Q5 conformance/performance, portable Track 22, stable Track 25, calibration C1/C2,
full ED MVP and release acceptance remain separate. Detailed source/hash/command
provenance is in the accompanying JSON and preserved worktree archive.
