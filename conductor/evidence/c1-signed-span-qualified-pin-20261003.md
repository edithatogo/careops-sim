# Signed timestamp span: qualified development pin

Parent base: `9a5060fdcbaa5b67fbe91dad057bce97e99917e8`. Previous Kairos pin: `76c04dc2ae2280deb8ec342f9f1ffc139929323c`. Qualified pin: `dffd6f6889353d4a13bb39b7981983a53df7d888`.

The pure relative-tick helper rejects PreOrigin and uses sign-biased unsigned subtraction for ordered i128 timestamps. MIN-to-MAX yields u128::MAX. Minute precision and public signatures remain unchanged. These are synthetic numeric cases, not clinical date validation.

Authenticated readback of [owner run37114011919](https://github.com/edithatogo/kairos/actions/runs/37114011919) confirmed exact dffd head and successful Native owner (ubuntu-24.04) and Native owner (macos-15) jobs. Contract hosts are x86_64-unknown-linux-gnu and aarch64-apple-darwin.

Independent source review found no numerical/API blocker. The old helper compiled and failed the full-span assertion on actual Rust1.76 (exit101, source51b925c5cd5548369db8c15d446666b04eea5ef8). Fix receipts record seven selected tests passing separately on actual Rust1.76 and1.98.1 and scoped formatting. Parent qualification repeats only pin/context checks; hosted CI handles integration. Diff from the old pin is trace_time.rs and its two test files. All other existing source contract hashes remain unchanged.

Full C1 ingestion/transport, C2 orchestration and Q3 runtime acceptance remain pending. Active phase/next action and all plan checkboxes remain unchanged.

## Verified local receipt hashes

- `/private/tmp/kairos-trace-time-signed-span-receipt-20261003/.artifacts/track04/relative-ticks-signed-span-v1/result.json`: SHA-256 `096e043be4df7e5c5b9ede501b19fb027742c33809fc7eab6e91001fc064d704`
- `/private/tmp/kairos-trace-time-signed-span-20261003/.artifacts/track04/relative-ticks-signed-span-v1/logs/rust-1.76-red.log`: SHA-256 `f6710ee03b748b3d263e061462c34afdf72c264bb144155ec9eb677d6b35dccc`
- `/private/tmp/kairos-trace-time-signed-span-fix-receipt-20261003/.artifacts/track04/relative-ticks-signed-span-v1/fix/result.json`: SHA-256 `a73b6080bed52fadfcf9fb06927eabcffe3a076a5ed7f1e8bcd37c412b99fba1`

## Attempt lineage

The first isolated attempt stopped when a mutable contract was incorrectly declared as both input and output: the guard rejected the next write for input drift. Its claim was released and checkout preserved. Coordinator authorized this clean continuation; immutable inputs only, copied reviewed metadata under a new guard. No failed-guard write or commit occurred.
