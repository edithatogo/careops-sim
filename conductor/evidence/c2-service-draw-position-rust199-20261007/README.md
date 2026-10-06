# C2 routed service draw-position oracle

At Kairos commit `809ade8103239c1cef3f5d0622ca99834abdabbe`, the integrated routed Macro/Micro elapsed-time fixture now checks that enabling the two-tick Micro route preserves the Service draw position, alongside its existing duration, key, and seed checks. This closes the test-matrix coverage gap without changing production behavior or APIs.

The focused test passed on Rust 1.99.0. Strict all-target Clippy, workspace formatting, and `git diff --check` also passed. The exact commands, source hash, and bounded-context note are recorded in [acceptance-rust199.json](acceptance-rust199.json).

The prior 455-pass full-package receipt is bound to Kairos `6784538`; it is not evidence for this new pin. The full package suite was not rerun here. This receipt records only the focused test and static checks at `809ade8`. Track03 and Track22 owner decisions, hosted exact-head checks, release, clinical acceptance, and C2.4 phase review remain open.
