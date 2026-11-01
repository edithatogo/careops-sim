# A4.1 transfer lifecycle contract evidence

- Track: `careops_core_domain_abstractions_20261020`, phase A4
- Task: A4.1
- Contract base: `c3aacb4aecfb5034f0c3c102a949635581463063`
- Reviewer: independent reviewer-agent/subagent `review_a41_contract`
- Disposition: contract accepted; reviewer confirms A4.2 is ready to implement
- Contract: [ADR-0012](../../decisions/ADR-0012-transfer-lifecycle-and-conservation.md)
- Packet: [A4.1 transfer contract](../../execution/packets/A4.1.transfer-contract.json)

## Review and fixes

The reviewer checked the contract against CC-02, the composition and transport
designs, the A4 plan, existing CC-02 negative tests, and the packet. Three
ambiguities were found and corrected before acceptance:

1. The `InTransit → Arrived` transition now converts the already-created
   in-flight destination hold to occupied capacity atomically.
2. Request-level expiry is explicit from `Requested`; it retains origin claims
   and holds/releases no destination claim.
3. The deadline oracle now distinguishes request expiry before offer, offer
   expiry before reservation, reservation expiry before dispatch/departure,
   pre-departure dispatch expiry, and in-flight expiry at arrival. Only an
   existing pre-departure destination reservation is released, exactly once;
   an in-flight hold becomes occupancy at arrival without release.

The reviewer rechecked the final state transition and expiry semantics, reported
no remaining findings, and confirmed that A4.2 is ready to implement. The final
contract SHA-256 is `47b75af12339c469343166a5a2eace02058170679a65d3b36856482b7ebff7dc`.

## Contract scope

ADR-0012 freezes operation identity and versioning, state transitions, unique
responsibility ownership, independent physical location, origin/destination
claim release points, event idempotency, deadline ties, compensation, horizon
censoring, and checkpoint/restart oracles. It explicitly avoids claiming
network exactly-once delivery, clinical eligibility, local transport policy,
distributed transport, or portable Kairos checkpoint support. The existing
test-only CC-02 stand-ins are not treated as runtime implementation or proof.

## Executed verification

| Command | Outcome | Captured output SHA-256 |
| --- | --- | --- |
| `.venv/bin/python tools/tasks.py packet-check conductor/execution/packets/A4.1.transfer-contract.json` | Passed; no packet errors. | `2abaac2832351140c25f21f6fe23ba23c80e9d02f9b5043cee4c5c0d13d2ebe5` |
| `git diff --check` | Passed; no whitespace errors. | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

This task freezes behavior and test oracles; it changes no Rust runtime source,
so no runtime test is claimed. A4.2 must implement and execute the cases in
ADR-0012, including checkpoint/restart or an explicit unverified blocker.

## Bound input hashes

- `conductor/tracks/careops_core_domain_abstractions_20261020/plan.md`:
  `f918ea0410a500f4ffbc6450d95721803d72ae5b183b0bd03645a54242d0ee34`
- `conductor/design/composition-contracts-20261020.md`:
  `4b83254e9ec17cbdd8c44790deda7bebf5893b3b46fc7f0eae2842af325d9249`
- `conductor/design/transport-retrieval-interfacility-flow.md`:
  `e65c0bf720c3539874ccb897a24663f0d74fdee8d214cfdba98954bc0858674a`

The validated packet SHA-256 is
`76c7edccb7a1623d3ee1040acd71e8983b938045c325238014433ac6487d3694`.
