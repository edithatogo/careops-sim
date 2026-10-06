# C2.1 exact-pin local parent join — Rust 1.99.0

The parent join ran DES, ABM, and calibration tests, strict Clippy with warnings denied, and workspace formatting at exact Kairos pin `bfbefa87ba15488837b01020f343685907ae09ac`. All checks passed under `rustup run 1.99.0`. The combined test run reported 442 passed, 0 failed, 5 existing ignored tests across 43 summaries.

Both required leaf instances are separately accepted: `C2.1.mode` covers all 16 policy scope masks and actual Pending/Active/Suspended Flow boundary preservation; `C2.1.paired` compares actual Macro Route and explicit-zero Micro Flow outcomes under a shared Service seed identity. The paired fixture’s Transit statement is bounded to its actual deterministic context, which owns no RNG stream.

This is local implementation acceptance only. Track03 owner/phase review and C2.4 phase review remain open, as do portable process checkpoint/restart, hosted, release, and clinical acceptance.
