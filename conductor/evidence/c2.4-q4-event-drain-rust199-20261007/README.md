# C2.4 urgent interruption event-drain oracle

Added a test-only assertion to the actual Flow urgent-interruption helper: after the service request completes and the fixture resource is fully released, the read-only scheduler snapshot must report zero pending Flow events. This closes the local oracle gap identified in the previous C2.4 source review for the isolated fixture.

The exact Kairos source commit is `69878dcaf9d1eac7f1eba84ef19647642b77435d` and the parent pin before this evidence commit is `8f40a0ac2bd4d91bd16b060ebc2c075d744b8586`. Only Rust/Cargo 1.99.0 was used. The targeted urgent Suspend and Restart fixtures pass; the combined DES/ABM/calibration suite passes 457 tests with 0 failures and 5 ignored across 43 result groups; strict all-target Clippy and workspace formatting pass. Commands, raw logs, hashes, and source SHA are in `acceptance-rust199.json` and `logs/`.

This is local test evidence only. It does not accept the C2.4 phase or close Track03/Track22 owner decisions, portable checkpoint/restart, hosted exact-head, release, or clinical gates.
