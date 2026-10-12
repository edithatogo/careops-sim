# C2.4 evidence reconciliation after urgent-interruption integration

This current inventory accepts the bounded local evidence below at integrated Kairos pin `6053d8dd2f8057c423decc929c90a8bcd10837f9`. It verifies local receipt identities and their stated scopes. **It does not accept the C2.4 phase or full C2.** Track03 owner/phase review, Track22 portable restart, hosted exact-head checks, release, and clinical acceptance remain separate open gates.

## Current local evidence

- C2.1 mode and paired runtime joins: [acceptance](../c2.1-parent-rerun-rust199-20261007/acceptance-rust199.json).
- C2.2 provider, policy, seed, and bridge joins: [acceptance](../c2.2-parent-rerun-rust199-20261007/acceptance-rust199.json).
- C2.3 Flow resource lifecycle and staff-policy selector leaves: [resource acceptance](../c2.3-flow-resource-lifecycle-20261007/acceptance-rust199.json) and [dispatch acceptance](../c2.3-staff-policy-20261006/leaf-acceptance-rust199.json).
- C2.4 integrated queue/transit/service ledger: [acceptance](../c2.4-runtime-ledger-rust199-20261007/acceptance-rust199.json).
- C2.4 Q4 urgent service interruption through the calibration bridge: [acceptance](../c2.4-q4-interruption-rust199-20261007/acceptance-rust199.json).
- C2.5 same-live-Flow in-memory continuation: [acceptance](../c2.5-continuation-rust199-20261007/acceptance-rust199.json).
- Standalone synthetic accounting arithmetic remains separate: [manual ledger](../c2-manual-ledger-20261007/README.md).

The Q4.5 accepted development commit `1125b5268bb349a5befe12f5789045042faab3e3` is an ancestor of the current Kairos pin. The new C2.4 leaf supplies local exact-pin integration evidence only; Q4's hosted run is not a hosted result for this newer head.

The coordinator readback at [current-readback-c2.4-q4](../c2.4-closeout-rust199-20261007/current-readback-c2.4-q4.md) records the withheld phase disposition and remaining gates.
