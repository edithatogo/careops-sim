# Test matrix: Repository status reconciliation and Kairos upgrade

| Phase | Automated Check | Manual Verification |
| --- | --- | --- |
| R0 | `python3 tools/context.py check` returns 0 errors | Verify `tracks.md` and `index.md` reflect 100% completion |
| R1 | `python3 -m unittest discover -s tests -v` with harness venv | Confirm all python tests pass without missing modules |
| R2 | `cargo check --workspace --tests` | Verify zero compiler errors or warnings against upgraded Kairos |
| R3 | `cargo test --workspace` and replay benchmarks | Replay trace checksum comparison against reference baseline |
