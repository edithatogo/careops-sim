# Accepted test matrix

Qualified source `c62d53a`, actual tested merge `99c4378`, identical complete tree. [R61 receipt](../../evidence/proptest-r61-acceptance-20261105/acceptance.json) binds exact commands/tools/hashes, completed logs and downloaded objects.

| Boundary | Actual evidence |
| --- | --- |
| Independent current-core properties | Seven bounded contracts; conservation/order/valid-prefix rollback controls, fixed-seed replay, shrink-to10 and saved-case replay with zero new random cases; explicit full-u64 boundaries. |
| Source identity | All61 guard tests pass in fresh full same-head Python suite; all earlier immutable receipts retained. |
| Native platforms/readers |810 tests per Linux/macOS host; zero failed/ignored/skipped; eight birthing replications and independent ED reader suites pass. |
| Default and regenerated lock |798 outer tests across 109 binaries plus two known subprocess executions each; zero failures/ignored/outer filters. Regenerated lock is not promoted. |
| Python/integration |847 +127 full suite tests pass. |
| CLI/output/recovery |26 frozen canonical pairs, 14 adversarial cases, Linux `/dev/full`, preflight/backpressure; three older nine-file artifacts remain readable and unchanged while foreign resume refuses; current-artifact production succeeds. |
| Quality |512/578 ED-library lines,88.58% over unchanged 80% floor;117 mutants:111 caught, two prior reviewed equivalents, four known unviable, zero timeouts. |
| Policy/package/cost | All configured audits/privacy/security; actual Linux ELF/consumer, nine SPDX entries/notices and source lock; frozen Rust suite/build/RSS/binary and full ELF allowances pass. |

Finite generated coverage is not exhaustive. New-format durable resume and end-to-end physical durability remain Track 70 responsibilities. Whole-system/clinical/site/stable-release acceptance is separate. No new retries, skips, exemptions, threshold changes or upstream pin promotion.
