# Native owner join formatting amendment

The initial native-owner-join/acceptance.json records formatting exit0 in error.
Its raw fmt.log contains diffs, and a fresh pinned Rust1.99 cargo fmt --all --check
at2e27bbe returned exit1. The shell sequence continued to a receipt-writing helper,
whose success was mistaken for the formatting status. Preserve the initial receipt
and raw log; this amendment supersedes only the formatting claim. The 598/0/5 test
result and strict Clippy exit0 were observed independently and remain valid.

The affected calibration files had been formatted with edition2024; this workspace
declares edition2021. Root formatted route_receipt.rs and work_duration.rs with the
project edition atf1a7089; their pinned owned-file check returned exit0. The
flow_bridge.rs writer is correcting its owned file. Workspace formatting requires
a fresh check after joining; it is not accepted by this amendment alone. No new
runtime test result follows from formatting.

Future commands must capture each individual exit status and stop dependent
acceptance work on a failed gate. A later helper status cannot stand in for a
gate. Retain failed logs and append the corrected outcome. The local learning
loop receives this as a proposal, not automatic permission or measured efficacy.
