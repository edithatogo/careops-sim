# Final hosted C2 qualification — 9 October 2026

Draft [Kairos PR243](https://github.com/edithatogo/kairos/pull/243), stacked on PR229,
is at `bc98ae5ec15805d3e82c4e60af8d52d13de4f726`.

- **64 successful checks**, zero failed, zero pending.
- Two conditional skips: `Q5.2 canonical native regression (Ubuntu)` and `Codecov OIDC upload`.
  These are not counted as passes or evidence for their separate acceptance gates.
- Actual Ubuntu x86_64 and macOS ARM: each **468 passed,zero failed,five ignored**,
  across43 groups, using Rust/Cargo1.99.0. Eight Arrow feature/platform lanes pass.
- Both hosts build the example once, save and restore in separate processes, obtain
  byte-identical repeated traces, and require existing-target save exit1 with
  unchanged sentinel bytes. The actual suffix bytes also match across both hosts.
- Static/current installed toolchain policy, Track13 metadata on Ubuntu/Windows,
  changelog, source/security/coverage/docs checks complete successfully.

GitHub compiled synthetic merge checkout `9c5158c145eec87e8a8305b7467f54f0dbaf81ac`.
Its parents are the PR229 base and the reviewed PR243 head. Root checked the
GitHub commit object against local Git: its tree is exactly the PR head tree
`3f561c2cb2a991ea1c33224786fe6c96de321b7b`. Both artifact source logs record this
compiled checkout. This verifies tested source identity; it is not an actual
merge into the default branch.

Structured check/run/checkout readback, downloaded native artifacts, command logs,
actual checkpoints/output traces and hashes are in [hosted-bc98/](hosted-bc98/acceptance.json).
Earlier failing attempts and source corrections are retained in their original
receipts. Root's first final readback rejected an assumption that the PR head SHA
must equal the checkout SHA; the corrected readback explicitly proves identical
Git trees and records the actual compiled commit.

The experimental C2 source and current native CI profile are qualified within this
scope. **C2.4 remains unchecked** because its declared wider exit includes general
Track22 portable checkpointing and stable/release/clinical acceptance. The replay
API implements one sealed synthetic recipe and same-executable compatibility;
it does not provide arbitrary Flow/C5 recovery, cross-platform checkpoint exchange,
an ED model or clinical validation. The draft PR has not been merged or released.

The project-local learning loop stores reviewed, evidence-hashed procedures and
actual reuse feedback. This session added the CI metadata/changelog lesson;
performance gains have not been measured. No external agent framework, background
service or global memory modification was introduced.
