# C2.4 Track03 owner review packet

**Preparation state:** prepared for owner review; not sent and not dispositioned.  
**Requested reviewer:** Track03 `des-api-agent` and `abm-api-agent`, within their owned paths and contracts.  
**This packet does not accept C2.4, close the C2 phase, or substitute for Track22, clinical, release, or hosted evidence.**

## Decision requested

Review whether the C2 execution behavior that touches Track03’s DES/ABM mode, Flow, and transit boundaries is consistent with Track03 contracts at the integrated source pin. Identify any Track03-owned correction, contract/ADR decision, or follow-up and its responsible owner. Record a Track03 owner disposition only; do not infer a broader phase or release decision.

## Exact reviewed state

- Parent review baseline immediately before this packet refresh: `402194800db00a89ed61f2532429311e075cda2b` (this revision changes review documentation only; Kairos remains at the pin below).
- Parent source/execution baseline for the latest exact-pin runtime receipt: `c2e35e064e83ccfd1d436da20adeeb6092e6a520`; the review baseline above includes subsequent evidence/documentation updates.
- Integrated Kairos pin: `678453838f391f3dd869b79fd502077f63f3a067`.
- Q4 test source: `crates/kairo-ecs-calibration/src/flow_bridge.rs`; source SHA-256 `e2c3b5eb5381b85a083f879a017d7be5cd573cdabd23b82eda486bab27e41959`.
- Q4 leaf acceptance receipt: [`acceptance-rust199.json`](../c2.4-q4-interruption-rust199-20261007/acceptance-rust199.json), SHA-256 `88109c5dc3a35eada33d0f047ac33d5271f7a5050f138393ff4a435a265a97c1`.
- Latest current-pin local evidence: [`README.md`](../c2-current-pin-rerun-rust199-20261007-6784538/README.md), SHA-256 `30972edf21df199a06d012b22938e3778d02a931d274a1b9c3a6440ed5c59ae6`; structured result [`acceptance-rust199.json`](../c2-current-pin-rerun-rust199-20261007-6784538/acceptance-rust199.json), SHA-256 `ed0b0b9bf19290d993a9aed7f0bce7df7dfa4d7a786304f021982fbe75b7f04f`.

The Q4 leaf acceptance receipt is historically bound to Kairos pin `6053d8d` and records Rust 1.99.0-only worker verification (164 passed, 4 ignored) and parent exact-pin verification (447 passed, 5 ignored), with strict Clippy, formatting, and independent source review recorded. These are local checks, not hosted checks.

The coordinator reran the three-package DES/ABM/calibration suite at the current Kairos pin `678453838f391f3dd869b79fd502077f63f3a067` using Rust 1.99.0 only: **455 passed, 0 failed, and 5 ignored across 43 result groups**; strict all-target Clippy and workspace formatting passed. See the [current-pin receipt](../c2-current-pin-rerun-rust199-20261007-6784538/acceptance-rust199.json). This is local re-verification of the package suite, not a new leaf acceptance, hosted check, owner disposition, or phase acceptance. The later documentation-only correction and its Rust 1.99.0 formatting/rustdoc evidence are recorded in the [retry-helper visibility receipt](../c2.4-retry-doc-hidden-rust199-20261007/README.md) (receipt SHA-256 `8ed2d2b604e3beaac8c773d5962571290bc5ae94e66c66cbadac17df20592e16`). It adds `#[doc(hidden)]` to a Rust-public experimental helper; it does not make that helper private or feature-gated.

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

Relevant ownership contracts: [CareOps calibration contract](../../tracks/empirical_calibration_20260925/agent-contract.md) assigns mode/agent/transit execution hooks to Track03; Kairos Track03 agent contract limits its source ownership to DES/ABM and Flow example paths and says public contracts require an ADR. The Q4 leaf’s source change at `6053d8d` is a test-only calibration bridge change. The earlier C2.3 source commit `3ec35e5` introduced `TransitContext::schedule_transit_retry` in Track03-owned `crates/kairo-ecs-abm/src/flow_adapter.rs`. The reviewed ADR-0008 C20 retry-seam decision authorizes this as a hidden experimental helper with no stable API promise. At the current pin `6784538`, both `validate_retry_dispatch` and `schedule_transit_retry` are Rust-public and doc-hidden; `TransitContext` is re-exported, and `#[doc(hidden)]` does not make either method private or feature-gated. The C2.3 receipt at `7672a679` describes only its leaf-specific test-support hook and retry/arrival test, so its `production_api_or_default_feature_change:false` field does not classify the earlier `3ec35e5` API. Track03 should review the bounded source/interface behavior and identify any remaining follow-up, without prefilled disposition. This packet is prepared for owner review, but has not been sent or dispositioned.

## Owner checklist

For each item, mark `accepted`, `accepted with follow-up`, `rejected`, or `not reviewed`; cite the evidence or contract clause used.

- [ ] At the Track03 execution boundary, mode selection remains distinct from the Track21-owned replay policy, and already admitted work is not changed by later staged fidelity-policy inputs. This is a boundary check only; it does not accept Track21 replay semantics.
- [ ] Flow work/request identity and busy-boundary behavior remain consistent with Track03’s runtime contract.
- [ ] Macro and explicit-zero Micro emit no transit event in the accepted paired fixture, with service identity preserved.
- [ ] Routed Micro binds only an accepted arrival to the original Flow request and preserves the one-retry/rejection boundary.
- [ ] At the Track03 Flow boundary, Q4 urgent suspension resumes the same work/request without resubmitting work or duplicating lifecycle events. Track21 owns intrinsic-duration provider semantics; this checks that the Flow boundary does not cause a second request.
- [ ] At the Track03 boundary, resource callbacks and observed Flow/transit intervals reconcile in the bounded fixtures. This does not accept Track21 calibration-ledger semantics.
- [ ] Resolve the C2.3 transit-contract questions below, identifying any cross-track owner and follow-up rather than assuming a new Track03 API.
- [ ] No Track03 public contract changed without an ADR; any Track03-owned gap or follow-up is identified below.

## C2.3 transit-contract questions for owner disposition

These are scope questions, not prefilled findings or implementation instructions:

- **Trip purpose:** The C2.3 plan calls for O/D purpose. The current `TransitGraphV1::route` and Flow route request carry origin/destination but no trip-purpose field. The C2.0 provider/transit proposal says a route receipt carries purpose, but that document remains a proposal awaiting owner review. Decide whether purpose belongs in the Track03 request/receipt, an upstream model/calibration record, or another contract; name the owner and ADR/follow-up required.
- **“Mixed-use pauses”:** Current pause/resume coverage exercises a segmented route with one movement mode. Define what “mixed-use” means in this plan clause and whether the existing oracle is sufficient; if not, identify the owning contract and concrete fixture needed before implementation.
- **Sensor-derived distance:** The generic route API consumes configured integer edge lengths and has no sensor input or provenance field. Confirm whether this satisfies the engine boundary, or identify the upstream geometry/data-provenance owner and evidence needed to ensure sensor observations are not treated as path or speed ground truth.

## Track03 disposition record — owner to complete

For each checklist item, identify whether it was reviewed by the DES API owner, the ABM API owner, jointly, or not reviewed. The designated Track03 reviewers should consolidate those item-level outcomes into one Track03 disposition.

- Reviewer identity and Track03 role: single maintainer (user), acting jointly as Track03 DES and ABM API owner. Decision taken from a coordinator-prepared, read-only reviewer summary.
- Review date: 2026-10-12.
- Parent commit and Kairos pin reviewed: parent `98c306a977a88cce6c039a9774b7d2f2676fdf7b`; Kairos `678453838f391f3dd869b79fd502077f63f3a067` (submodule re-synced to pin before review).
- Checklist outcomes and evidence references: all eight receipt SHA-256 values above re-verified with `shasum -a 256` and matched. Items 1–6 `accepted` within their stated boundary/fixture scope; this is receipt-backed, not an independent line-by-line re-derivation of `flow_bridge.rs`/DES Flow test bodies. Item 4 `accepted with follow-up` (`flow_adapter.rs` retry validator ~L151-170; reject hook stays `cfg(feature = "test-support")`). Item 7 `accepted with follow-up` (answers below). Item 8 `accepted with follow-up`.
- Findings / required changes:
  1. ADR-0008 authorizes one hidden experimental function, but at the pin `validate_retry_dispatch`, `schedule_transit_retry` and `expects_event` are Rust-public, `#[doc(hidden)]`, not feature-gated, and reachable through the `TransitContext` re-export. Owner choice: amend ADR-0008 to list all three as experimental with no stable API promise (no code change).
  2. Mixed-use pauses: defined as a pause on a route whose segments use more than one movement mode or shared resource. The single-mode segmented oracle is insufficient; the clause stays unverified until a two-mode fixture suspends at the mode boundary and mid-segment and asserts same work/request, no duplicate lifecycle events and reconciled intervals.
  3. Trip purpose: stays out of the Track03 engine request; belongs in the upstream calibration route receipt (C2.0 provider proposal). A Track03 ADR is needed only if purpose must affect routing.
  4. Sensor-derived distance: engine boundary satisfied (configured integer edge lengths, no sensor input). Upstream data owner records per-edge provenance and a rule that sensor observations are calibration evidence, not path/speed ground truth.
- Responsible owner and follow-up path for each finding: (1) Track03 ABM API owner, ADR-0008 amendment on Kairos branch `codex/adr-0008-retry-seam-amendment`, with Track25 compatibility review before release gates; (2) Track03 ABM owner, new fixture leaf; (3) Track21 / CareOps calibration, C2.0 proposal review; (4) CareOps empirical calibration / site-profile data owner.
- Track03 disposition (`accepted`, `accepted with follow-up`, `rejected`, or `not reviewed`): `accepted with follow-up`.
- Confirmation this disposition covers Track03 ownership only: confirmed. It does not accept C2.4, close C2, or satisfy Track22, hosted, release or clinical gates.

## Separate open gates

The current-pin receipt continues to list Track03 owner/phase review, Track22 portable checkpoint/restart, hosted exact-head checks, release acceptance, clinical acceptance, and the separate C2.4 Conductor phase review as open. This locally prepared packet closes none of them. No owner response or external workflow action is recorded here.
