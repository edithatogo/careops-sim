# V4 preview acceptance

Accepted `07c3584f99b42382ab2d0f5b1b0d00fc66c1f0a1` on Rust/Cargo 1.99.0. Strict Clippy, full calibration tests (394 passed, 0 failed, 5 ignored; all eight V4 preview cases), and formatting passed at identical source hashes. Source review corrected fixture mistakes before gate execution.

Coherent Acquire and portable Handoff remain in progress. Their accepted join requires four-package qualification before the parent pin and ED adapter. See [acceptance](acceptance.json) and [manifest](manifest.json).
