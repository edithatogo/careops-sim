# A3.2 staged allocation implementation evidence

- Track: `careops_core_domain_abstractions_20261020`, phase A3
- Task: A3.2
- Bound implementation base: `79844d377b5d2af0179a24d0fba154a7deb4c018`
- Kairos pin: `44261eb5cf8275f132f63a3396903d116d701dbe`
- Contract: [ADR-0011](../../decisions/ADR-0011-staged-resource-claims-and-interruption.md)
- Packet: [A3.2 staged allocation core](../../execution/packets/A3.2.staged-allocation-core.json)

## Delivered behavior

`careops-core::allocation` coordinates staged single-resource requests. It
records each partial engine lease, holds claims visible through failed release,
requests rollback in reverse acquisition order, and makes late grants explicit
orphan releases. Claim receipts bind to the observed engine request, resource,
and exact lease revision. Timeout-at-grant ties resolve to timeout; cancellation
and expiry retain residual claims. Work is suspended only at declared segment
boundaries; non-interruptible work rejects suspension without mutation. The
module is orchestration around Kairos; it is not an atomic multi-resource
allocator.

A conformance test runs two CareOps claim tokens against one capacity-one
`FlowRuntime` resource. Kairos reports one active lease and one queued request;
after the first lease is released and read back as `Released`, CareOps
acknowledges its release and the queued request becomes active. The test is
dev-only against the exact pinned local Kairos crate. Production dependencies,
the ED schema/defaults, Kairos source and parent pin are unchanged.

## Independent reviewer-agent review

Reviewer-agent/subagent `review_a32_integrated` reviewed the integrated source,
tests, packet, Cargo delta and exact Kairos pin. The review found:

1. Lease identity was initially not bound to its engine request/resource. Fixed
   by requiring request/resource identity and exact observed revision. Tests
   cover valid revision zero, a request mismatch, and a same-request stale
   revision.
2. FIFO token emission alone did not demonstrate contention. Fixed with the
   pinned-Kairos capacity-one fixture described above.
3. The initial fixture released engine capacity without completing/acknowledging
   the CareOps operation. Fixed by completing the CareOps work, waiting for
   Kairos `Released`, acknowledging the matching claim, and asserting zero
   retained claims plus `Completed`.
4. The `bind_engine_request` call-order documentation was corrected to match
   the adapter sequence.

The reviewer rechecked the fixes and reported no remaining findings. FIFO
issuance describes CareOps request order only; no engine starvation guarantee
is claimed. Portable checkpoint/restore and cross-process replay remain
unverified.

## Executed verification

Toolchain: Rust 1.99.1, Cargo 1.99.1; Python runtime is not used by this task.

| Command | Outcome |
| --- | --- |
| `.venv/bin/python tools/tasks.py packet-check conductor/execution/packets/A3.2.staged-allocation-core.json` | Passed at bound base `79844d3`; packet had no errors. |
| `cargo test --locked -p careops-core --test allocation_contract --test allocation_kairos_conformance` | Passed: 13 CareOps contract cases and 1 pinned-Kairos contention case. |
| `cargo clippy --locked -p careops-core --all-targets -- -D warnings` | Passed, including the test-only Kairos dependency. |
| `cargo fmt --all -- --check` | Passed. |
| `git diff --check` | Passed. |
| `git -C libs/kairos rev-parse HEAD` | `44261eb5cf8275f132f63a3396903d116d701dbe`. |

SHA-256:

- `crates/careops-core/src/allocation.rs`:
  `a2d4005dec1f6b879d488a82a9b2187bb0378789929da06f09adc73d8bc5a733`
- `crates/careops-core/tests/allocation_contract.rs`:
  `114be35cf4253e1fdfd5524b5c7a9e5ab4956ba6433fb05e65eb299708467722`
- `crates/careops-core/tests/allocation_kairos_conformance.rs`:
  `afa24b8cf053af6679865799ec4fa01259cc2d4eb61f2fa8ffbe1b26c1ac8b77`
- `Cargo.lock`:
  `81c674ed9a92bd210c9aa9430f1e5d29f5a2411a161525eeade1ec2336aba080`
- `crates/careops-core/Cargo.toml`:
  `7140e825aded1bfb82b39141e3f02a566833d5fefcd46a6b72897132cb9af8f5`
- `Cargo.toml`:
  `1924b9e34494d99857e1b3986364dc0509c0f2862f0965ee4ed9d00caeb096ce`
- `conductor/execution/packets/A3.2.staged-allocation-core.json`:
  `07472b632ac63f136656699540e7b707d848b7bc9e08cef7537cd66f9a602515`
