# C2.4 current phase readback after Q4 urgent-interruption integration

**Disposition: phase acceptance withheld.** The current integrated Kairos pin is `6053d8dd2f8057c423decc929c90a8bcd10837f9`. The new test-only C2.4 urgent-interruption leaf is locally accepted after exact Rust 1.99.0 worker and parent-pin checks plus independent source review. The refreshed evidence inventory includes this leaf; earlier C2.4 readbacks remain historical snapshots.

The test pairs Macro with routed nonzero-transit Micro through actual Flow Suspend preemption and resume. It preserves the original timed service work and request through an urgent two-tick interruption and reconciles queue, transit, useful service, and interruption intervals to the actual completion time. It does not change a production API or establish portable checkpoint/restart.

The C2.4 phase checkbox remains unchecked. Track03 owner/phase review, Track22 portable checkpoint/restart, hosted checks at the exact current head, release acceptance, clinical acceptance, and the Conductor phase review remain open. Q4.5 itself is accepted and its qualified source is an ancestor of this pin; its historic hosted run does not certify the current C2 head.

The standalone manual ledger remains a synthetic arithmetic readback, separate from this runtime evidence. See [the refreshed reconciliation](../c2.4-evidence-after-q4-rust199-20261007/README.md) and [the C2.4 Q4 interruption receipt](../c2.4-q4-interruption-rust199-20261007/README.md) for exact source, log, and receipt hashes.
