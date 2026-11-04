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
