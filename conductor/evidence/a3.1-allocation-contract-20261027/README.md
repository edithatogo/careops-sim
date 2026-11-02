# A3.1 allocation contract evidence

- Track: `careops_core_domain_abstractions_20261020`, phase A3
- Task: A3.1, accepted contract/readback only
- CareOps base: `ee5f870615d2e6b41043e4cd29b726999505328e`
- Kairos pin read back: `44261eb5cf8275f132f63a3396903d116d701dbe`
- Decision: [ADR-0011](../../decisions/ADR-0011-staged-resource-claims-and-interruption.md)
- Packet: [A3.1 staged allocation contract](../../execution/packets/A3.1.staged-allocation-contract.json)

## Capability readback

At the exact pinned Kairos commit, Flow requests and leases are single-resource
claims. `ResourceRequest` contains one `ResourceId`; `LeaseId` identifies a
request and revision. The experimental API includes suspend/abort/restart
strategies, continuation callbacks, and per-resource grant eligibility. These
are not evidence of an atomic multi-resource grant. The engine has no
intra-work safe-point API: preemption is available at the resource arbitration
event only for work declared preemptible. CareOps must segment work at safe
boundaries or keep the whole phase non-preemptible. The cited checkpoint formats
do not establish portable persistence of all eligibility/continuation states.
The Kairos Track 03 handoff limits its synthetic example and does not claim
atomic bundle allocation or portable checkpoint/restore.

## Independent review and disposition

Reviewer-agent/subagent: `review_a31_contract`, read-only review against CC-03,
the A3.1 objective, ADR-0011 and the exact pinned Kairos interface. The initial
review found that the first ADR wording implied a safe-workpoint capability not
present in Kairos. The coordinator narrowed the contract to engine arbitration
events and CareOps work segmentation/non-preemptibility. The reviewer rechecked
the correction and confirmed the finding closed; no other blocking contract
issue was reported.

No Kairos source, parent pin, ED schema/default or dependency was changed. This
does not establish upstream acceptance, runtime bundle allocation, clinical
eligibility, portable checkpointing, or cross-process replay. A3.2 must
implement/test the bounded staged mechanism or labelled composite-resource
fallback and stop if true atomic multi-resource grants are required.

## Executed checks and hashes

| Command | Outcome |
| --- | --- |
| `git -C libs/kairos rev-parse HEAD` | Exact expected pin `44261eb5cf8275f132f63a3396903d116d701dbe`. |
| `.venv/bin/python tools/tasks.py packet-check conductor/execution/packets/A3.1.staged-allocation-contract.json` | Exit 0; no packet errors. |
| `git diff --check` | Passed for the A3.1 contract changes. |

SHA-256:

- ADR-0011: `9b99a910c8693ef8a4f9cd4f8a0a1d6296061af136a140a8e138a24934ba368b`
- Kairos `crates/kairo-ecs-des/src/flow.rs`:
  `be8c78946793de3ac93e1ad26e755df60ef65a6236db83893ba0fa303abe21cc`
- Kairos Track 03 handoff:
  `6761badd0cc011acdb2619a8074fdf1d4f205924d235dae2a7c68218692c1cb9`
- Track spec: `070e599846f4b31e174902a31cff8fe819df1d0ba897a3a8195c366223da567c`
- Track plan: `8309283c9236d085cc3ab7100eabf20463475a2587b5d3ba3770a788771f7fa0`
- Composition contracts:
  `4b83254e9ec17cbdd8c44790deda7bebf5893b3b46fc7f0eae2842af325d9249`
