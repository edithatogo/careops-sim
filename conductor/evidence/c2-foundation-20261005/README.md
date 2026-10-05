# C2 foundation and executable test-first plan — 2026-10-05

This accepts only an experimental development pin and planning foundation. No
C2.0/C2.1/C2.2/C2.3/C2.4 checkbox is completed. Full C2 and ED MVP remain open.

## Source qualification

Kairos development source 65858ad, stacked draft PR222 on accepted C4.2 carrier:
241 actual DES tests pass under Rust 1.99; clippy/fmt and default DES Rust 1.76
check pass. Independent reviews cover all 16 mode-scope masks, actual lifecycle,
state preservation, runtime-identity/API and shared native conformance. Real
source disposable guard-removal and boundary-only mutants fail behaviorally
against a passing copied baseline. Child qualification describes exact local
source/commands and separates initial compile/tooling failures from passes.

Hosted native-owner run 37252152532 at exact head 65858ad passes both Ubuntu x86_64
and macOS ARM owners and all 8 Arrow IO Rust 1.88 feature cases. Full child readback
has 43 successful checks and 2 conditional skips; Q5.2 conditional regression and
Codecov upload are skipped, not new performance/coverage evidence. No upstream
main merge or stable-release assertion follows from this stacked source.

## Plan and harness

ADR-0008 separates C2.0 preparation from actual C2.1 runtime acceptance. The
accepted execution order is C2.0 -> C2.2 -> C2.3 -> C2.1 -> C2.4. Original paired
CRN/no-transit/service-work and work-preservation requirements remain mandatory.
Catalog 144 tasks and MVP 81 parents/251 leaves pass exact dependency/closure checks.
Independent planning review confirms no accepted task status was changed.

Parent 491 Python harness tests pass; context, catalog and MVP integrity checks
pass after updating the explicit submodule-pins context field. Precommit context
checks initially caught that stale context pin; it was corrected, not waived.
Resume context now selects 19 active paths and retains historical evidence via
index/registry links. Existing private dirty source and Track49 remain untouched.
Python fixture validator output is not fresh native mutation qualification.

## Next and gates

Bind reviewed C2.0 mode-test/interface/red-paired packets. Provider/admission and
real transit-observation interfaces must be frozen against actual source. Passing
fixed-duration Flow or disconnected RNG tests cannot close C2.1. C2.2 implements
real work-provider/mode/stream support; C2.3 supplies route/progress/dispatch;
C2.1 joins actual execution and C2.4 independently closes the phase. Portable
Track22 codec/rebinding, stable API baseline and release, full ED MVP and v1
remain distinct. C4.3 is separately ready; E1.1 still waits on C2.4.

Parent exact-head hosted CI and merge are pending this publication. Child PR222
remains stacked/draft to preserve experimental carrier and upstream release gates.

## Later hosted readback supersedes the local snapshot

The child `c2-mode-native-qualification-20261005.md` is a pre-hosted source
snapshot. Its statement that final-head native-owner gates remain required
describes that preparation time. The later exact-head37252152532 receipt in
`native-owner-hosted.json` satisfies those development hosted gates at65858ad.
It does not waive the remaining full-C2, portable codec or release gates.
Independent publication review identified this timing ambiguity; this dated
parent note explicitly resolves it without a self-referential child CI commit.
