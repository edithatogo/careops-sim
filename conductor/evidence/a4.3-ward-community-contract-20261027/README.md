# A4.3 Ward/Community Boundary Contract

## Preparation

- Base: `a940f4f0bb42134041f8ac0899cd99d176b23e8b`
- Packet: `conductor/execution/packets/A4.3.ward-community-contract.json`
- Contract: `conductor/decisions/ADR-0013-ward-community-boundary.md`
- A4.2 dependency: accepted at `ecb24f0`; evidence is in
  `conductor/evidence/a4.2-transfer-core-20261027/README.md`.
- The current MVP catalog has no A4.3 recipe (`tools/mvp.py show A4.3` returns
  `Unknown leaf`); A4.3 is a non-MVP track task. `tools/mvp.py check` passed
  before packet preparation.

ADR-0013 defines a synthetic one-bed boundary: clinical readiness is distinct
from physical departure, arrival and handover. The ward bed's origin claim stays
held through transit and pending handover. That claim-release tick determines
when a ready ED patient can be admitted. Integer virtual time, transport arrival
and inclusive horizon censoring are explicit. It adds no site-specific clinical
thresholds, transport distributions or capacity contention policy.

## Contract review

The contract was approved after an independent reviewer-agent review at base
`a940f4f0bb42134041f8ac0899cd99d176b23e8b`. The reviewer checked physical
timing, boarding feedback, horizon ties and claim boundaries. Two P1 findings
were addressed: (1) physical vacancy at departure is distinct from the origin
claim held through transit, and ED admission waits for claim release; (2) arrival
is distinct from handover completion, with an explicit acceptance delay and a
pending-handover horizon state. The final review found no remaining actionable
findings. Reviewer: `/root/a43_contract_review`; no files were edited by the
reviewer.

Final ADR-0013 SHA-256:
`6c731c84c1307356f222cdfa60ee1e6dbcaeb1f26a9d5cefa567a8f21fde737f`.
Packet validation (`tools/tasks.py packet-check`) and `git diff --check` passed.
The packet is rebound to the final ADR digest. Implementation can proceed under
a separate A4.3 implementation packet.

## Model implementation

Implementation packet `conductor/execution/packets/A4.3.ward-community-model.json`
is bound to base `6a7f339dff1cc046c469db94badf561d513d063f` and accepted ADR-0013.
The implementation adds `careops_core::ward_community::simulate` and a focused
contract suite. It uses no new dependency, ED schema or Kairos change.

- `crates/careops-core/src/ward_community.rs` SHA-256:
  `9c8bdea4615b4245a191c8b2e76f6f35d15cbcdbecc57e234fdd2638098dc846`
- `crates/careops-core/src/lib.rs` SHA-256:
  `103b2e15ea924ea599fcd6b5467107bbc22aceec2760aba5221aacfc5ef49296`
- `crates/careops-core/tests/ward_community_contract.rs` SHA-256:
  `ff734b473d7fd5d4a6b1e5a292b703950e63cbc63a18b04ce75fc09b2693b7d1`
- `cargo test --locked -p careops-core --test ward_community_contract --test transfer_contract`
  — exit 0; 34 passed, 0 failed (24 transfer, 10 ward/community).
- `cargo clippy --locked -p careops-core --all-targets -- -D warnings` — exit 0.
- `cargo fmt --all -- --check` — exit 0.
- `tools/tasks.py packet-check conductor/execution/packets/A4.3.ward-community-model.json`
  — exit 0, no errors. Packet SHA-256:
  `96ad6f14acbc26783f63dde22dbc47e11ea371871bb27dad987dc2a4efacb438`.
- `git diff --check` — exit 0.

Toolchain: `rustc 1.99.1 (58cf8f9c3 2026-10-13)` and
`cargo 1.99.1 (5f94df478 2026-08-27)`. Commands ran in
`<source-worktree>/careops-sim` at the
packet-bound base. Their captured summaries are recorded in this receipt; there
are no separate ignored log artifacts.

Independent reviewer-agent review covered the implementation and tests at the
packet-bound base. Findings were: (1) add an inclusive-horizon departure test;
(2) make duration-field start points explicit. The first was fixed with a
horizon-departure regression; the second by naming and documenting the reported
intervals as post-readiness durations. Final read-only confirmation by
`/root/a43_model_review` at commit
`3bfaf0ebafcd911929c86a7887915bc62839cc90` verified the ADR and implementation
hashes match this receipt and found no new actionable findings. The reviewer
made no edits.

No operational, clinical or full ED integration acceptance is claimed.
