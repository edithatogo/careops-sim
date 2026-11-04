# Shared trusted-import source qualification — 4 November 2026

This first source snapshot adopts Kairos data473ff2c while retaining runtime0741026 and Polars9643968. The immutable receipt binds15 critical sources and the actual26 frozen inputs/results. It preserves all five earlier receipts at their original Git snapshots; it does not renew A7 acceptance.

Actual external-target qualification:3 populated domain FFI tests,68 focused CI/support tests, format check, activated graph policy (all4 groups),5 normal dependency graphs, historical conversion1 Rust test plus3 independent reader tests and6 value comparisons. All60 original fixture files remain unchanged. Minimal/Wasm evidence here is dependency-graph isolation, not Wasm execution.

Release build41.24s; binary SHA-256 `c4e7ed9f5158b23f6e8d667dbbe69d53a1a1db285bd87232881cc02c6deee033`,91644496bytes, Homebrew Rust1.99.1/LLVM23.1.3. Actual fresh26-case run passed, including14 rehashed-file negatives, zero/oversize inputs, slow stdout and blocked pipe with unchanged10s/256MiB/8MiB budgets. Linux `/dev/full` remains a hosted gate. Policy used canonical Rust1.99.1/LLVM23.1.1.

Private command logs and reports are retained in the external evidence directory, prefix `polars-parent-473-`: release-final.json, historical-readback.json, graphs.json, domain-ffi.log, support-ci-tests.log, policy-all.log and release-build.log. These bind the dirty candidate based on85419da and exact source hashes; this commit is the immutable qualification snapshot, not final-head hosted acceptance.

The separately committed guard must anchor this receipt and critical source bytes to this snapshot before final checks or push. Full R14 performance, package, coverage, mutation, independent review and final hosted acceptance remain open. PR367 exact-source hosted run41667625899 passed all three operating systems; its merge conflict is retained. PR199 remains draft. No merge, release, clinical or site acceptance is claimed.
