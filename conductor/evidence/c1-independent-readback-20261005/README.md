# C1 independent manual readback — 5 October 2026

Accepted for the requested **synthetic manual readback only**. Full C1.4 remains
open. Kairos development pin `c441c693eccf666d00e2720a215dd1f238e331ad` is unchanged;
Q5.2 parallel work and inherited source-PR/release gates are unaffected.

## Executed readback

Direct CPython 3.14.8 / PyArrow 25.0.1 IPC file, IPC stream and Parquet readers
checked **666 files**: 648 C-01 Rust outputs, 9 current-joined C1.2 Rust outputs,
and 9 retained C1.1 fixtures. The independent verifier declares schemas locally
and imports no production codec, schema builder or interop validator.

Exact field order, Arrow types, nested nullability, schema/field/list-element
metadata and ordered logical values match. Clock integers are fixed binary16:
signed little-endian i128 UTC nanoseconds since Unix epoch and unsigned
little-endian u128 relative ticks, unit1ns. UTC text and origin subtraction are
checked using integer calendar arithmetic. Occurrence is UInt32, order UInt64,
and arbitrary decimal rank is UTF8. No native Arrow timestamp narrowing occurs.

Null parents suppress child inspection. Absent, explicit null and empty values
remain distinct; absent-only properties reject unmarked null. Typed raw-time map
entries preserve sorted keys and nullable values. Per-field schema catalogs and
logical null/absent/empty observations are retained. All **8** in-memory negative
controls reject: unit, integer width, required null, two absent-only nulls,
UTC bytes/text disagreement, right-censor event clock and unknown censor status.

## Count reconciliation

| C-01 profile | Source rows | Candidates | Mapper accepted | Excluded | Valid | Quarantine | Observed outcomes | Right-censored outcomes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long-valid | 7 | 7 | 6 | 1 | 6 | 0 | 1 | 1 |
| wide-valid | 3 | 9 | 6 | 3 | 6 | 0 | 1 | 1 |
| long-quarantine | 7 | 7 | 6 | 1 | 3 | 3 | 1 | 1 |
| Distinct profile fixture totals | 17 | 23 | 18 | 5 | 15 | 3 | 3 | 3 |

C-01 failed/unresolved candidates and validator window-censored cases are zero.
Outcomes are a separate population, not events or exclusions.

Separately, all51 heterogeneous C1.1 mapper requests reconcile **56 source rows
and59 candidates =31 accepted +17 excluded +6 failed +5 unresolved**. Physical
logical tables contain31 events,17 exclusions and4 outcomes (2 observed,
2 right-censored). The51 requests classify as24 accepted,22 partially excluded
and5 failed. These are fixture collections, not one clinical cohort. Format,
layout and writer replicas add no distinct logical records.

## Retained evidence and limits

[Acceptance](acceptance.json), [independent review](independent-review.json),
[member inventory](artifact-inventory.json), [archive](readback.tar.gz), and
[checksum](SHA256SUMS) bind actual inputs, commands, cwd, source commits/hashes,
tool versions, logs and output hashes. The archive retains the actual51-request
mapper snapshot and9 Rust C1.2 binaries. The original C-01 archive at the accepted
Kairos pin supplies the648 binaries and all2207 verified members; it is not
copied again here. C1.1 fixture inputs are checked against the accepted manifest.

C1.2 original receipts retained source/log hashes but no output-file inventory.
Its copied9 files were freshly SHA-bound before decoding, independently matched
to accepted logical records, and retained here; fresh hashes are not presented
as historical output hashes. Historical local paths in receipts remain evidence
of that run; a replay must resolve input roots and bind a fresh working inventory.

Only `outputs/readback-attempt3.json` and `outputs/counts-attempt2.json` are the
accepted reports. Earlier failed attempts and the superseded attempt2 logical
observation report remain for audit. Correcting verifier assumptions did not
change source data. No Rust execution was repeated for this readback.

The standard-library [count verifier](reconcile_counts.py) and independent
[physical reader](inspect_readback.py) are retained source. Their CLI help lists
explicit input/output arguments. Use an initialized accepted Kairos checkout,
the verified C-01 archive, and the retained fresh joined inventory for replay.
