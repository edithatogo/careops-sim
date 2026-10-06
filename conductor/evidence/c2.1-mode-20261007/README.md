# C2.1.mode — Rust 1.99.0 local leaf acceptance

The existing exact-head mode fixtures are accepted at Kairos pin `dbd12c98b1a72a30760996590efa6a0f445368a6`. The bounded test run reports **442 passed, 0 failed, 5 ignored** across 43 result summaries. Strict Clippy with warnings denied and workspace formatting also pass.

The tested named oracles cover all 16 policy scope masks, preservation of actual Pending/Active/Suspended work when policy replacement is blocked, and an actual Flow Macro route paired with explicit-zero Micro that creates timed work without a scheduled transit event while service draws match. Two independent packet reviewers and two independent source/result reviewers returned READY.

This accepts only the `C2.1.mode` leaf. The `C2.1.paired` full canonical outcome, transit-stream-position, and mode-transition comparison remains open; C2.1 parent runtime acceptance is not claimed. Hosted, release, and clinical acceptance also remain open.
