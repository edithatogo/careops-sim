# E2.1 staff specifications — 11 October 2026

Accepted 20 positive and 12 invalid/repair staff and calendar specifications, all native-unbound until E2.2. They cover explicit urgency/FIFO/ID ordering, exact skill/zone eligibility, half-open calendars, DrainActive and horizon accounting. The count limit uses an actual 1025-staff payload.

Independent GPT-6.1-sol review caught and resolved patient-ID uniqueness, calendar normalization/permutation, FIFO contention and field-bound checks. A final integer-count check rejects Python boolean/float coercion. Six focused worker checks passed. Root independently ran the combined Python test_e*.py family: 114 tests passed on clean integrated source, followed by context and MVP checks. This is specification evidence, not a Rust staff simulator or native binding.

Original worker receipt is preserved. Earlier worker failure logs were overwritten and unavailable; the receipt also omitted final log hashes. Root archived and hashed the retained final logs and independently qualified the final source. No missing failure evidence was recreated or treated as a passed gate.

Next is E2.1.interrupt, then space. E2.1 parent, E2 runtime, MVP, operational validity and release remain open.
