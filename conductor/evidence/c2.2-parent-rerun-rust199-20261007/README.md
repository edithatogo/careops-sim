# C2.2 parent local join — Rust 1.99.0

The combined parent rerun passed on parent commit `ab269f3c3289f1b727d50a6e8dcd1ad85aba4056` with Kairos pinned at `9b360f433210d38868e1e5f102473fd6f148fccf`. Rust and Cargo were both 1.99.0. The three native packages produced 43 result summaries: 441 passed, 0 failed, 5 ignored. The exact C20 nonzero-route pause/resume retention fixture passed once (82 filtered); strict all-target Clippy and workspace formatting passed.

The joined C2.2 leaf instances and their receipt hashes are listed in `acceptance-rust199.json`; the reviewed parent packet is preserved as `packet.json`. The detailed commands, outputs, log hashes, toolchain versions and C20 backup/restore audit are in `verification-manifest.json`, `logs/`, `c20-current-pin/`, and `c20-prerun-manifest.json`. The C20 runner’s original ignored artifact tree was restored with the full 3,764-member inventory and all member hashes verified.

This is local C2.2 parent acceptance only. C2.3 parent and Track03 owner review, C2.1 runtime, portable process checkpoint/restart, C2.4 phase review, hosted checks, release and clinical acceptance remain open.
