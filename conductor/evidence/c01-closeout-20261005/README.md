# C-01 canonical invariance development acceptance — 5 October 2026

Kairos [PR #213](https://github.com/edithatogo/kairos/pull/213), stacked on C1.3
PR #212, retains the exact synthetic qualification archive, checksums and member
inventory at `conductor/evidence/c01-invariance-20261005/` in the child.

- Three long/wide/quarantine profiles; accepted, quarantined, excluded and outcome
  populations match retained baselines by exact bytes, ordered records and hashes.
- Independent IPC batches 1/2, Parquet row groups 1/3 and physical forward/reverse
  orders: 24 adapter invocations, 648 ordered output checks, 216 reader sources.
- All 3,888 normalization points pass: 108 actual executions and 3,780 explicitly
  verified exact-input aliases. Six actual representative output sets retained.
- Row/byte spill triggers and single-run controls pass; independent C0 accounting
  and schema checks cover all three profiles; schema-valid mutations are rejected.
- Executed commands, failures, source/tool/input hashes and independent joins are
  retained. Original policy config files and the byte-preserving supplement remain
  visible. Archive SHA and all 2,207 member hashes were independently read back.

`source-acceptance.json` records the qualified source and exact source/archive
hashes; `independent-review.json` records separate ledger joins and root byte
comparisons. Hosted [native-owner run 37212864498](https://github.com/edithatogo/kairos/actions/runs/37212864498)
passed all eleven jobs at development pin `c441c693eccf666d00e2720a215dd1f238e331ad`;
`hosted-native.json` retains the exact-head readback.

C-01 passes for this declared synthetic matrix. C1.4 remains unchecked. Clinical
and real-feed validity, hostile-decoder memory, stable API, release readiness and
broader inherited upstream CI remain separate. Q5.2 source remains uninstalled.
