# C2.3 current-pin route and public API verification

At exact Kairos source commit `69878dcaf9d1eac7f1eba84ef19647642b77435d`, the immutable C20 route fixture passed all 9 named tests and the C23 public API fixture passed all 3 named tests. Both runs used Rust/Cargo 1.99.0 only. Raw logs, exact commands, source hashes, and the disposable-fixture details are recorded in `acceptance-rust199.json`.

C20 ran through `conformance/c20/run_routes_red.py --expect green`, which archived the committed source and injected the frozen fixture into its disposable test tree. C23 used an archived copy of the same commit with the frozen smoke fixture copied into the package test directory.

This refresh verifies current-pin routing and public getters. It does not implement the optional Track21 route-receipt adapter, add a graph hash to geometry, or establish a graph/profile identity comparison across a reconstructed continuation. Track03/21 ownership review remains open, as do C2.4 phase acceptance, Track22 portable restart, hosted exact-head, release, and clinical gates.
