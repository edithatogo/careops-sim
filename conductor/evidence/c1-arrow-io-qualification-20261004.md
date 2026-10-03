# C1 optional Arrow IO foundation qualification

## Scope

The parent adopts Kairos development commit `3fccca9a305703447701ad458bc6c170c852c604` from `codex/careops-c1-arrow-io-qualification`. This joins the optional Arrow IPC/Parquet IO crate, narrowly scoped tiny-keccak license exception, reviewed feature-floor CI and the existing Q4 builder/example runtime prerequisites. Shared ABM documents are design inputs; the adapter is not implemented.

The crate has independent none/ipc/parquet/both feature selections and Rust 1.88 minimum. Arrow/Parquet dependencies stay optional. Limits bound input/output and table dimensions; they do not establish peak allocator or hostile-input guarantees. No clinical mapping, UTC/DST normalization, calibration replay or full C1 acceptance is claimed.

## Executed evidence

At the joined source, explicit local Rust 1.88 and 1.98 each passed 140 tests: 112 DES and 28 IO. Independent PyArrow readback passed on fresh Rust outputs for both compilers. Local evidence remains in `/private/tmp/kairos-c1-builder-serial-join-20261004/.artifacts/c1-builder-serial-join/`; raw failures and corrective attempts remain preserved.

[Hosted owner run 37151260049](https://github.com/edithatogo/kairos/actions/runs/37151260049) completed successfully at exact child `3fccca9a305703447701ad458bc6c170c852c604`: two native jobs, all eight Rust 1.88 OS/feature jobs, and the required aggregate. The both-feature lanes execute frozen Python fixture checks and independent readback of fresh Rust IPC/Parquet artifacts. Native jobs retain optional calibration floor coverage. This proves the bounded IO foundation on Linux x86_64 and macOS arm64; parent PR and merge checks remain separate.

Python oracle uses exact CPython 3.14.8, NumPy 2.5.3 and PyArrow 25.0.1 with hash-pinned binary wheels. The rejected yanked NumPy 2.4.0 draft was replaced before CI publication. The tiny-keccak exception names exact package/version/license and does not broaden advisory policy or authorize PR #195 exceptions.

## Remaining acceptance

C1 clinical/event mapping, canonical normalization across batch/row-group layouts, timestamp/precision/timezone policy, malformed/unknown-field disposition and downstream trace lineage remain separate required work. Q4 shared carrier integration, captured lifecycle telemetry and synthetic workflow joins also remain open. No plan checkbox is closed by this pin change.
