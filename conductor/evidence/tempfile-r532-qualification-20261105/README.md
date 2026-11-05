# Tempfile R53.2 qualification

Source `f2d7ffc3792321c731c893d797dd4fcd10f1a1f7`; execution checkout52108bf contains only the separately reviewed browser-policy updates before these owned qualification changes. [Structured evidence](qualification.json) and [frozen packet](packet.json) bind actual inputs, commands, private raw-log hashes and limits. R53.2 decision is accepted after independent review; no parent dependency/runtime adoption or R53 phase closeout.

## Exact dependency, compatibility and security

The [registry](https://crates.io/api/v1/crates/tempfile/3.27.0), read through the internal browser, confirms current stable3.27.0, default getrandom, MIT OR Apache-2.0 and declared MSRV1.63. The resolved native default closure reaches MSRV1.85; actual candidate contracts compile/run on parent floor1.99.0 and qualified1.99.1. All16 probe registry archives match parent checksums and their published files match cached source. No new registry versions are admitted.

Fresh cargo-deny0.20.2 checks pass for the exact isolated closure and full configured locked/all-feature workspace. RustSec commit965bf9ad083b58e6340d65829d85217b18be9290 matches the [live primary source](https://github.com/RustSec/advisory-db/commit/965bf9ad083b58e6340d65829d85217b18be9290), checked in the internal browser. Policy/exemptions remain unchanged; smaller private-closure unmatched existing skip warnings are retained. These are bounded software checks, not security certification or a complete binary notice audit.

## Independent behavior and measured value

Actual current core IO/error source is the baseline. Two child umasks0022/0077 each pass17 Rust control groups, plus8 independent Python byte/mode cases and2 actual128-byte file-size-limit write failures. Both helpers preserve overwrite and error category. The baseline destroys a pre-existing PID staging sentinel and leaves128-byte scratch after write failure; the candidate preserves the sentinel and cleans scratch while preserving the original destination. Candidate output creation uses0666 & umask to preserve current behavior, rather than assuming NamedTempFile's0600 default is compatible. Test-directory candidates explicitly request0700. No public no-clobber argument is added; persist_noclobber is a separate primitive control.

An extracted actual ED checkpoint sync seam with IO-error adapters confirms that an error on the second sync can occur after publication. This is not a full ED compilation or physical crash proof. Existing ED native capture/checkpoint semantics, process-kill/disk-full and manifest-last gates remain unchanged.

Seven alternating pairs per1KiB/1MiB workload, each at least100ms, pass the frozen limits. Median candidate/baseline latency ratios are1.0622/1.1385, with binary growth7520bytes and RSS within16MiB. Fresh isolated-target builds using a populated registry cache take6.917/10.339s (baseline/candidate), warm builds0.029/0.034s. These are page-cache IO and small probe costs, not full-application cold builds, persistence latency or portable speedup. The non-test prefix decreases42→28physical lines (-14), including documentation/imports/blanks. The identity fixture preserves inline tests byte-for-byte and adds one separator, producing a whole file58→45 (-13). Added279 qualification-support lines and receipts are disclosed separately. No overall net-code reduction is claimed. The value is observed collision and error-cleanup robustness using an already present maintained primitive.

## Graph, lock and build identity

Minimal core gains7 normal nodes (25→32); zero new global registry versions does not mean zero dependency cost or unchanged Cargo.lock. Offline metadata-only fixtures preserve all actual manifest bytes except named edges and use source stubs that are never compiled. Core-normal and CLI-dev variants change only their respective workspace lock dependency rows; ED-dev needs no lock-row change because its optional normal dependency already names tempfile. Combined changes affect only core and CLI rows. All registry packages/versions/sources/checksums and other lock entries remain identical.

The actual build.rs and independent component-ordered framed hashlib transcript pass14 controls, including actual resolved-lock changes. Source/manifest/lock/compiler/flags/features alter the controlled model ID; relocation/documentation do not. These fixture IDs are not final runtime artifact IDs. R54 needs a new exact immutable successor source/lock receipt and actual candidate artifact/recovery proof; original Polars and SHA2 receipts/anchors must remain unchanged. The old SHA2 byte-identical-lock rule is insufficient for this workspace-node delta.

## Corrections and remaining gates

Retained preparation failures cover a stale probe-only Serde pin, a real disk-full compile, cross-flavour cached Rust ABI rejection, independent path-order correction and strict metadata-fixture guards. Final evidence uses the unchanged parent version set and frozen budgets; no timed sample was discarded or tolerance relaxed. Only this lane's idle compiler cache was reclaimed; final binaries, external release libraries and historical receipts remain.

R53.3 must review this decision before any R54 tests-first/replacement. R55 hosted/platform/integration/security/coverage/mutation and actual whole-application costs remain gates. Existing ignored write errors at four experiment-runner sites are recorded, with no silent error-policy change. No upstream write, pin promotion, merge, release or clinical/site acceptance is implied.

[Final review and focused planning proof](review.json) records reviewer Go,63 focused tests and context/tasks/MVP/diff success. Ownership,405task IDs/dependencies and unrelated records are preserved; only R53.2 is newly checked.
