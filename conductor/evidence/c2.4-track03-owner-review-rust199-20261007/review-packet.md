# C2.4 Track03 owner review packet

**Preparation state:** prepared for owner review; not sent and not dispositioned.  
**Requested reviewer:** Track03 `des-api-agent` and `abm-api-agent`, within their owned paths and contracts.  
**This packet does not accept C2.4, close the C2 phase, or substitute for Track22, clinical, release, or hosted evidence.**

## Decision requested

Review whether the C2 execution behavior that touches Track03’s DES/ABM mode, Flow, and transit boundaries is consistent with Track03 contracts at the integrated source pin. Identify any Track03-owned correction, contract/ADR decision, or follow-up and its responsible owner. Record a Track03 owner disposition only; do not infer a broader phase or release decision.

## Exact reviewed state

- Parent review baseline before this evidence and packet refresh: `bbdd87778ae6cb0aa809a6bb85437c2ec507a662`; the reviewed child source is now integrated at the Kairos pin below.
- Parent source/execution baseline before this refresh: `bbdd87778ae6cb0aa809a6bb85437c2ec507a662`; the exact child source and local verification are bound to the integrated pin below.
- Integrated Kairos pin: `6364a92b12a2b1c186ab1477814bc3dfb8a50aef`.
- Current `flow_bridge.rs` source SHA-256 at the reviewed pin: `18663b5407f4992d72bec07f6352b9a05ca6a59828b56299f841aa23a46df66c`. Historical Q4 leaf receipt remains bound to its earlier test source and pin.
- Q4 leaf acceptance receipt: [`acceptance-rust199.json`](../c2.4-q4-interruption-rust199-20261007/acceptance-rust199.json), SHA-256 `88109c5dc3a35eada33d0f047ac33d5271f7a5050f138393ff4a435a265a97c1`.
- Latest exact-pin local evidence: [`README.md`](../c2-test-regressions-rust199-20261007/README.md), SHA-256 `85fe23037d7a4443b501f8e0db41f1b060bec1bfa70d0a9424820ff816c189e4`; structured result [`acceptance-rust199.json`](../c2-test-regressions-rust199-20261007/acceptance-rust199.json), SHA-256 `e1398dca00025af0187d9f2f29856b3bda912b91338a4ec27f5094d45a902783`.

The Q4 leaf acceptance receipt is historically bound to Kairos pin `6053d8d` and records Rust 1.99.0-only worker verification (164 passed, 4 ignored) and parent exact-pin verification (447 passed, 5 ignored), with strict Clippy, formatting, and independent source review recorded. These are local checks, not hosted checks.

The coordinator reran the three-package DES/ABM/calibration suite at Kairos `6364a92b12a2b1c186ab1477814bc3dfb8a50aef` using stable Rust 1.99.0 only: **457 passed, 0 failed, 5 ignored across 43 result groups**; strict all-target Clippy and workspace formatting passed. See the [current exact-pin receipt](../c2-test-regressions-rust199-20261007/acceptance-rust199.json). This is local source/test verification, not a hosted check, owner disposition, or phase acceptance. The earlier 455-test receipt at `6784538` is historical. The later documentation-only correction and its Rust 1.99.0 formatting/rustdoc evidence are recorded in the [retry-helper visibility receipt](../c2.4-retry-doc-hidden-rust199-20261007/README.md) (receipt SHA-256 `8ed2d2b604e3beaac8c773d5962571290bc5ae94e66c66cbadac17df20592e16`). It adds `#[doc(hidden)]` to a Rust-public experimental helper; it does not make that helper private or feature-gated.

## Track03-relevant evidence inventory

Receipt hashes below are recorded for the listed leaf evidence and should be reverified at review time. The C2.3 transit-bridge receipt is supplemental lineage evidence and is separately verified below.

| Area | Receipt | Receipt SHA-256 | Scope limit recorded by receipt |
| --- | --- | --- | --- |
| C2.1 mode and paired parent join | [`acceptance-rust199.json`](../c2.1-parent-rerun-rust199-20261007/acceptance-rust199.json) | `4f1ba5b364dd3c908287c286e7414d19dcdc3da3f1458e95471f66a9a76b6f20` | Local parent join; fixture-scoped Macro/explicit-zero Micro and no-transit-draw oracle. |
| C2.2 provider/policy parent join | [`acceptance-rust199.json`](../c2.2-parent-rerun-rust199-20261007/acceptance-rust199.json) | `72a4f1868bba86aed670aee50fd2b6682166969284bf05fea328488b805675b8` | Local parent join; calibration provider/policy ownership remains Track21. |
| C2.3 Flow resource lifecycle | [`acceptance-rust199.json`](../c2.3-flow-resource-lifecycle-20261007/acceptance-rust199.json) | `fd8d48f939b29901da7e6807f00d065e2414e56a076e3752b6523ef94e096afa` | Local leaf and parent join; Track03 owner disposition remains open. |
| C2.3 transit bridge source (Track03-owned lineage context) | [`acceptance-rust199.json`](../c2.3-transit-bridge-rust199-20261006/acceptance-rust199.json) | `04b35a643a30347e5465d5f37936fd6443698692654cd6ea879cab9567fb80a2` | Source commit `7672a679` is an ancestor of the reviewed pin; `crates/kairo-ecs-abm/src/flow_adapter.rs` has the same Git blob at both commits. This receipt covers the bounded test-support rejection hook and retry/arrival oracle; it does not describe the earlier helper’s API visibility. |
| C2.3 staff-policy selector | [`leaf-acceptance-rust199.json`](../c2.3-staff-policy-20261006/leaf-acceptance-rust199.json) | `0bada6fe15b01e157dbf4ea45849b953add19209c68b550718dec60bfa8ccd5d` | Selector leaf only; does not by itself accept full transit dispatch or resource lifecycle. |
| C2.4 runtime ledger | [`acceptance-rust199.json`](../c2.4-runtime-ledger-rust199-20261007/acceptance-rust199.json) | `b4e250c598d1cd540f32bde27889961365d8709909201b433927616f8a7e3189` | Local leaf and parent join; phase and Track03 review remain open. |
| C2.4 Q4 urgent interruption | [`acceptance-rust199.json`](../c2.4-q4-interruption-rust199-20261007/acceptance-rust199.json) | `88109c5dc3a35eada33d0f047ac33d5271f7a5050f138393ff4a435a265a97c1` | Test-only actual Flow suspend/resume and interval/resource reconciliation; phase remains open. |
| C2.5 in-memory continuation | [`acceptance-rust199.json`](../c2.5-continuation-rust199-20261007/acceptance-rust199.json) | `543a0e9e6b73c7e392280bbe6b9f3cc69fe1b6abd29a635e2ad97444d0536282` | Same-live-Flow continuation only; portable serialization/restart is Track22-owned and open. |
| C2.1 mixed-subsystem runtime oracle (supplemental) | [`acceptance-rust199.json`](../c2-test-regressions-rust199-20261007/acceptance-rust199.json) | `e1398dca00025af0187d9f2f29856b3bda912b91338a4ec27f5094d45a902783` | One live Flow/adapter, Macro and routed Micro decisions, accepted-arrival boundary; local test only. |
| C2.2 provider validation precedence (supplemental) | [`acceptance-rust199.json`](../c2-test-regressions-rust199-20261007/acceptance-rust199.json) | `e1398dca00025af0187d9f2f29856b3bda912b91338a4ec27f5094d45a902783` | Test-only validation-order regression; no production change or reopened task. |
| C2.5 non-empty rejection preservation (supplemental) | [`acceptance-rust199.json`](../c2-test-regressions-rust199-20261007/acceptance-rust199.json) | `e1398dca00025af0187d9f2f29856b3bda912b91338a4ec27f5094d45a902783` | Active blocker, queued waiter, requests/resource/context preserved; portable restart remains open. |

Relevant ownership contracts: [CareOps calibration contract](../../tracks/empirical_calibration_20260925/agent-contract.md) assigns mode/agent/transit execution hooks to Track03; [Kairos Track03 agent contract](../../../libs/kairos/conductor/tracks/03-flow-des-trajectory-abm-behavior/agent-contract.md) limits its source ownership to DES/ABM and Flow example paths and says public contracts require an ADR. The Q4 leaf’s source change at `6053d8d` is a test-only calibration bridge change. The earlier C2.3 source commit `3ec35e5` introduced `TransitContext::schedule_transit_retry` in Track03-owned `crates/kairo-ecs-abm/src/flow_adapter.rs`. The reviewed [ADR-0008 C20 retry-seam decision](../../../libs/kairos/conductor/design/calibration/ADR-0008-c20-transit-retry-seam.md) authorizes this as a hidden experimental helper with no stable API promise. At the reviewed pin `6364a92`, both `validate_retry_dispatch` and `schedule_transit_retry` remain Rust-public and doc-hidden; `TransitContext` is re-exported, and `#[doc(hidden)]` does not make either method private or feature-gated. The C2.3 receipt at `7672a679` describes only its leaf-specific test-support hook and retry/arrival test, so its `production_api_or_default_feature_change:false` field does not classify the earlier `3ec35e5` API. Track03 should review the bounded source/interface behavior and identify any remaining follow-up, without prefilled disposition. This packet is prepared for owner review, but has not been sent or dispositioned.

## Owner checklist

For each item, mark `accepted`, `accepted with follow-up`, `rejected`, or `not reviewed`; cite the evidence or contract clause used.

- [ ] At the Track03 execution boundary, mode selection remains distinct from the Track21-owned replay policy, and already admitted work is not changed by later staged fidelity-policy inputs. This is a boundary check only; it does not accept Track21 replay semantics.
- [ ] Flow work/request identity and busy-boundary behavior remain consistent with Track03’s runtime contract.
- [ ] Macro and explicit-zero Micro emit no transit event in the accepted paired fixture, with service identity preserved.
- [ ] Distinct Macro and routed-Micro subsystem decisions remain associated with their own work and requests in one live Flow; Macro rejection schedules no event and Micro submits only on accepted arrival.
- [ ] Routed Micro binds only an accepted arrival to the original Flow request and preserves the one-retry/rejection boundary.
- [ ] At the Track03 Flow boundary, Q4 urgent suspension resumes the same work/request without resubmitting work or duplicating lifecycle events. Track21 owns intrinsic-duration provider semantics; this checks that the Flow boundary does not cause a second request.
- [ ] At the Track03 boundary, resource callbacks and observed Flow/transit intervals reconcile in the bounded fixtures. This does not accept Track21 calibration-ledger semantics.
- [ ] Resolve the C2.3 transit-contract questions below, identifying any cross-track owner and follow-up rather than assuming a new Track03 API.
- [ ] No Track03 public contract changed without an ADR; any Track03-owned gap or follow-up is identified below.

## C2.3 transit-contract questions for owner disposition

These are scope questions, not prefilled findings or implementation instructions:

- **Trip purpose (cross-track contract reconciliation):** C2.3 requires an origin/destination purpose. The separate C2.0 provider/transit document is explicitly an unaccepted proposal and asks for purpose on a route receipt; the accepted C2.0 execution contract keeps purpose streams and the cryptographic receipt adapter with Track21, while the spatial `route` signature has no purpose field. Confirm with Track21 whether the proposed receipt purpose remains required, and identify whether it belongs in a Track21 receipt or a Track03 route request/context. No route signature or ownership choice is prefilled.
- **“Mixed-use pauses”:** Current pause/resume coverage exercises a segmented route with one movement mode. Define what “mixed-use” means in this plan clause and whether the existing oracle is sufficient; if not, identify the owning contract and concrete fixture needed before implementation.
- **Sensor-derived distance:** The generic route API consumes configured integer edge lengths and has no sensor input or provenance field. Confirm whether this satisfies the engine boundary, or identify the upstream geometry/data-provenance owner and evidence needed to ensure sensor observations are not treated as path or speed ground truth.

- **Graph identity/hash (cross-track contract reconciliation):** The C2.0 provider/transit document is an unaccepted proposal that asks a route receipt to carry graph version/hash. The accepted C2.0 execution contract specifies canonical topology bytes, no SHA dependency in geometry, and an optional Track21 calibration adapter hashing those bytes with SHA256; C2.3 adoption freezes the byte format and independent golden. Current geometry exposes canonical bytes and route identity, not a mandatory digest field. Ask Track21 to confirm whether a graph hash is required in its receipt and ask Track03 to confirm geometry conforms to the accepted canonical-byte contract. Do not add a digest to geometry or change the receipt contract without the owning-track decision and any required ADR.

## Track03 disposition record — owner to complete

For each checklist item, identify whether it was reviewed by the DES API owner, the ABM API owner, jointly, or not reviewed. The designated Track03 reviewers should consolidate those item-level outcomes into one Track03 disposition.

- Reviewer identity and Track03 role:
- Review date:
- Parent commit and Kairos pin reviewed:
- Checklist outcomes and evidence references:
- Findings / required changes:
- Responsible owner and follow-up path for each finding:
- Track03 disposition (`accepted`, `accepted with follow-up`, `rejected`, or `not reviewed`):
- Confirmation this disposition covers Track03 ownership only:

## Separate open gates

The current-pin receipt continues to list Track03 owner/phase review, Track22 portable checkpoint/restart, hosted exact-head checks, release acceptance, clinical acceptance, and the separate C2.4 Conductor phase review as open. This locally prepared packet closes none of them. No owner response or external workflow action is recorded here.
