# Transit retry helper visibility correction — Rust 1.99.0

This evidence records a narrow source correction authorized by ADR-0008: `TransitContext::schedule_transit_retry` remains an experimental Rust-public helper but is hidden from generated API documentation and carries no stable API promise. It does not make the method private or feature-gated.

## Source and checks

- Kairos source commit: `6303a2552257f1fcbd2aaa002d38352a2a467592` (`docs(abm): hide transit retry helper`), parent `6053d8dd2f8057c423decc929c90a8bcd10837f9`.
- Change: one `#[doc(hidden)]` attribute on `TransitContext::schedule_transit_retry` in `crates/kairo-ecs-abm/src/flow_adapter.rs`.
- Toolchain recorded by the worker: `rustc 1.99.0 (b940084d7 2026-09-28)`.
- `cargo fmt --all -- --check` in the Kairos worktree: exit 0; output log SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
- `cargo doc --locked -p kairo-ecs-abm --no-deps` in the Kairos worktree: exit 0; output log SHA-256 `ae64ed3105d11067a20ed3f2f78c24fc64de54d901ea7b8f2aadc8d8b35a8886`.
- Generated `TransitContext` rustdoc omits `schedule_transit_retry`; HTML SHA-256 `b2223311ac5c6ade7af6d9fbc1598f539a35ece3df30717325d2aa098c7aa432` (artifact retained in the Kairos worktree under `target/doc/kairo_ecs_abm/struct.TransitContext.html`, not committed).

The exact-parent runtime suite, strict Clippy and the broader C2 checks have **not** been rerun at this pin. No tests were run for this correction. The Rust 1.99 formatting and rustdoc checks are narrow documentation/source evidence only.

## Gate status

The parent C2 branch now points to this Kairos commit. Track03 owner disposition and the distinct C2.4 Conductor phase review remain open, along with Track22 portable checkpoint/restart, hosted exact-head checks, release and clinical acceptance. This receipt does not accept C2.4 or C2.

Worker logs retained in this directory: [`rustc-version.log`](logs/rustc-version.log), [`fmt.log`](logs/fmt.log), and [`doc.log`](logs/doc.log).
