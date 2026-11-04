# SHA2 R51 tests-first and bounded replacement

[Tests-first receipt](tests-first.json) records exact source/test hashes,22
independent vectors, current-parent baseline success, genuine wrong-digest
failures and the deliberately red closed manifest successor interface. Ten
native recovery tests pass, including24 actual process-kill/recovery cases and
four malformed/corrupt digest consumer cases with no mutation.

The original pre-replacement native binary produced the preserved synthetic
artifact before any helper/manifest change. Its actual model ID, binary hash and
file hashes are frozen in the receipt. The no-environment auxiliary test return
is not cross-build proof. R51.2 must explicitly consume these original bytes
under the successor model, assert the changed model ID, retain readability and
reject execution resume before changing any file.

Reviewer `/root/ffi_packet_prepare` approves the tests-first checkpoint only.
The original disk-full build failure and no-debug-profile recovery are retained;
no gate, oracle, old source receipt, pin, production helper or manifest changed
in this checkpoint. Native linking's compact-unwind warning is disclosed.
R51/R52 acceptance remains pending.

## Bounded source qualification checkpoint

[Qualified source receipt](qualified-source.json) records the exact helper and
normal workspace edge, unchanged lockfile and14 other shared-profile sources.
Fresh actual native tests pass:2 independent digest tests,10 recovery tests
(including24 process crashes),5 C63 tests with the unchanged100ms threshold,
and2 minimal-graph digest tests. Strict affected Clippy,28 workflow-policy tests
and canonical toolchain/locked metadata policy pass.

The actual successor reads the frozen original artifact with a different model
ID, refuses execution resume and preserves all9 original files. Both native test
binaries are losslessly compressed with verified decompressed SHA256; original
release bytes remain untouched. The candidate CLI passes26 frozen input/summary
pairs,14 negatives, zero/oversized input and slow/blocked stdout under original
budgets. macOS has no `/dev/full`; that new-source Linux check remains unverified
for R52. The first missing-reader attempt and disk/partial-link failures remain
separate from successful receipts; no partial output was treated as executable
evidence. Serial links and verified compression bounded storage without skips.

The actual rustfmt helper is8 lines (the R50.2 compact prototype was4);74
handwritten function lines and obsolete allow/documentation are removed. Added
contracts, guard and qualification support are counted separately; no overall
net-line reduction is claimed. Runtime/core/birthing/RNG/canonical identity
semantics and pins stay unchanged. The historical shared-source guard still
refuses this new manifest until a second commit anchors this actual checkpoint
and new receipt. Do not regenerate old Polars/R501/R502 receipts or infer renewed
A7, full hosted/workspace, coverage/mutation, release or clinical acceptance.

## R51 phase review and successor guard

[Phase review](phase-review.json) binds the immutable implementatione44946b and
tests-firstc6bccc5 to the reviewed closed successor guard. The old receipt stays
validated against its original profile. Only the exact qualified ED manifest
edge may differ; the lock, other14 critical shared sources and5 digest/build/
contract sources stay bound. Mutable source/receipt co-drift and unknown profile
changes refuse. No historical receipt, source anchor or upstream pin is rewritten.

The39-case guard run had38 passes and one obsolete historical-positive assertion
failure. Review corrected that one assertion to retain the old receipt/anchor
check, require the current immutable successor, and prove the exact single
manifest delta. Its focused rerun passes;38 unchanged results are reused, not
relabeled as a fresh full-suite run. R51 bounded local engineering is complete
after final review; all R52 full acceptance tasks remain open. No track archive,
main/PR197/198 merge, stable release or clinical acceptance is implied.
