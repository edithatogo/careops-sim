# Test-output scope correction

Coordinator source review after hosted146c1e0 found that `c5_workers::tests::temp_dir`
walked three parents from CARGO_MANIFEST_DIR (`kairos/crates/kairo-ecs-cli`), placing
its `.artifacts/c5-workers/test-output` under the checkout's parent directory.
The declared output reservation was checkout-relative. Therefore earlier worker
fixture writes were outside that reservation even though Git scope checks passed
(ignored/outside-checkout outputs are not automatically discovered by the harness).
Most fixtures remove their own unique output directory; shared leftovers have not
been deleted or attributed to this session without evidence.

The coordinator commissioned correction to exactly two parents, root/manifest
assertions, and an exact-source rerun before final qualification. This supersedes
any earlier broad claim that every C5 test output stayed within its reservation,
including the earlier `reserve-nested-test-artifacts` helped outcome. The lesson
remains useful, but ownership must also be checked against actual resolved output
paths; declaring a path alone is not proof that the code uses it.

Runtime calibration/shard behavior is unaffected by this test-only path defect.
Historical test exits and results are retained, not rewritten as scope-compliant.

During the final Windows CLI packet, `c5_cli` reported it had already launched
`cargo test -p kairo-ecs-cli --locked` before the coordinator's scope reminder.
Its claim covered only `.artifacts/c5-cli-prefix-win`; that full suite generated
additional `.artifacts/c5-cli` and `.artifacts/c5-legacy-resume-plan` outputs, and
its base still contained the worker path defect above. The 19+7 tests passed on
macOS, but that run is not scope-compliant. The worker was instructed to leave
outputs untouched, retain this deviation, avoid more tests, and commit only its
owned source. No retrospective claim is treated as authorization for those writes.
Final integration must use corrected paths and an explicit complete reservation.
