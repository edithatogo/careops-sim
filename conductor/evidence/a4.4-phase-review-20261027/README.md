# A4 integrated phase review

Status: **accepted locally** at `095f79b17fc2472e1172af5960b2f0c20354b058`. Evidence is limited to the deterministic local simulation contracts and repository checks below.

## Scope and integrated review

Reviewed range: `c3aacb4aecfb5034f0c3c102a949635581463063..095f79b17fc2472e1172af5960b2f0c20354b058`. A4.1 freezes transfer lifecycle/conservation contracts; A4.2 implements versioned transfer envelopes; A4.3 implements the single-bed ward/community boundary; A4.4 reviews the integrated source and reconciles phase state.

Reviewer agent `/root/a44_final_review_after_all_fixes` examined the exact branch commit and reported no actionable source or contract findings. It confirmed all three findings were fixed: (1) nested compensation rejects atomically while one-level compensation still completes and releases root claims; (2) failed future-tick commands do not commit clock advancement or unrelated expiry; (3) direct `advance_to` rolls back clock and expiry state on revision overflow. Supporting fix reviews are recorded in `a4.4-transfer-fix-20261027/README.md` and `a4.4-advance-fix-20261027/README.md`.

## Verification

Toolchain: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, `cargo 1.99.1 (5f94df478 2026-08-27)`, `aarch64-apple-darwin`; repo-local Python `.venv/bin/python` is Python 3.15.1. Kairos submodule remained pinned at `44261eb5cf8275f132f63a3396903d116d701dbe`.

| Command | Result | Captured stdout+stderr SHA-256 |
| --- | --- | --- |
| `cargo test --locked --workspace` | exit 0; workspace tests and doc-tests passed | `14afdd76c784c71d45fb342c5bf65c4107d69afc0e8c51de51b3985a5944c57b` (91,928 bytes) |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | exit 0 | `afe0bcf81f1d8acfbad01b702978cd571532ab09c80c8c6cb174c7782dd487f0` (435 bytes) |
| `cargo fmt --all -- --check` | exit 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` (empty output) |
| `.venv/bin/python tools/tasks.py check` | exit 0; 401 task records, complete coverage, prerequisites and DAG valid | `ab8b9f02b3213a06d5ea7abe5c5c139318dcecab9d957cb97570eaa7024af7d3` (61 bytes) |
| `.venv/bin/python tools/context.py check` | exit 0; context integrity passed | `e0e3879e52456fe7d990475460dc02aae1860701247c2264df4d55e561a9c1f5` (79 bytes) |
| `.venv/bin/python tools/mvp.py check` | exit 0; MVP joins and hashes passed | `3c7b9a3062f62964126c8db247833e935b8557a4baeffb7a30525cbe5f410966` (86 bytes) |
| `git diff --check` | exit 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` (empty output) |

## Conductor closeout integrity

After accepting A4.4 and updating all dispatch pointers, `tools/tasks.py build` rebuilt the plan-derived catalog. The subsequent check passed with 401 records, and the catalog marks A4.4 accepted with A5.1 depending on A4.4. `tools/context.py check` passed for 25 tracks and 109 milestones with 1,947 valid local links and no errors. `tools/mvp.py check` passed for 82 MVP parents, 258 bounded leaves, complete joins and context/objective hashes. The updated A4.4 packet passed `tools/tasks.py packet-check`; `git diff --check` passed.

This track has 32 leaf tasks: 20 accepted and 12 pending (A5–A7). A5.1 is now the next incomplete task. A4.4 was accepted locally; this does not archive the track or satisfy later clinical, hosted, release or B7 dependent gates.

## Limits

No atomic multi-resource grant, nested compensation, portable Kairos checkpoint, clinical validation, real-service operational acceptance, ED schema migration, hosted CI, or release qualification is claimed. A5.1 is next; A5–A7 remain incomplete.
