# R0.3 — E6/E8 source and receipt classification

Scope: careops-sim at `c3c47abbc4321b1f12a48a9bcecc47747d507d09`, runtime pin
`44261eb5cf8275f132f63a3396903d116d701dbe`. Source inventory R0.2 was independently
reproduced and accepted before this audit; R0.4 phase review remains required.
No fresh hardware run, runtime change, pin promotion or clinical acceptance occurs.
The separate careops portfolio repository is outside this work.

## Classification

[source-classification.json](source-classification.json) binds eight exact source
hashes and line observations to the immutable parent revision.

| Evidence | What the inspected code establishes | What remains unverified |
| --- | --- | --- |
| E6.2 `ResidentBufferContract::execute_kernel` | Mutates local buffer state, increments an execution counter and signals a local Boolean fence; WGSL text/layout and constructed device metadata are contract fixtures | Actual adapter/device discovery, shader compilation, queue submission, synchronized GPU readback and independent CPU parity on an identified Metal device |
| E6.3 latency and resident-chain tests | Supplied constants and a bandwidth model populate staging/kernel/CPU timing fields; arithmetic checks model amortization. CPU reference and unavailable-device cases are useful contracts | Measured cold/transfer/compute/readback distributions, real device memory limits, measured equal-workload speedup and cancellation/recovery |
| E8.3 `MultiNodeClusterRunner` | Two local `std::thread` workers exchange messages through `std::sync::mpsc`, irrespective of the labelled node count. Local delays/counters exercise a synthetic protocol fixture | Distinct physical hosts/process transports, actual partition topology, failure/restart recovery and measured distributed throughput/latency |
| E8.3 efficiency and parity fields | `distributed_efficiency` is `1.0 / node_count`, not measured serial/parallel speedup. `state_parity_verified` is a causality-violation check; it does not itself compare full output distributions | Equal-workload serial reference, full state/trace or justified statistical comparisons, initialization/transport/compute decomposition |

The parent careops-ed manifest lists RNG/core/types and serde dependencies;
its test names, comments and constructed metadata do not supply a device runtime.
This bounded finding concerns the inspected fixtures, not an unsupported claim
that no other implementation could ever provide hardware support.

The dated [E6.4](../e6.4-phase-acceptance-20261017.md) and
[E8.4](../e8.4-phase-acceptance-20261017.md) acceptance texts remain preserved.
Their historical checked tasks are not erased, but their physical-device and
multi-host claims cannot be used as current capability receipts. Current
classification is **contract/source evidence only; physical acceptance unverified**.

## C-R0–C-R7 and ownership reconciliation

The existing [PR173 mapping](../pr173-roadmap-integration-20261021.md) already maps
all eight recommendations into numbered work:

- C-R0: R0.1–R0.3/R1.3 exact-source reconciliation and baseline freeze.
- C-R1: R2.1–R2.2/R3.1–R3.3 capability-preserving candidate, parity and rollback.
- C-R2: R0.2–R0.3/R2.2/A3.1 distinguishes named APIs/contracts from execution.
- C-R3: R0.3/R2.3 retains separate Mac Metal, CUDA PC and physical Mac/NAS gates.
- C-R4: A0.2/A6.1/A6.3 qualifies small runner reuse before resource migration.
- C-R5: A3.1–A3.3 qualifies allocation, interruption and cancellation/rollback.
- C-R6: A0.2–A0.3/A5.1–A5.2/A6.1/A7.1/B7 retains neutral consumers and domain adapters.
- C-R7: R3.3/every phase exit binds command/source/input/toolchain/output evidence.

The careops-sim coordinator owns this evidence/status integration. Kairos existing
owners 03/04/21/22/25 retain consumed APIs, replay, IO and compatibility decisions;
32–35 retain device/distributed mechanisms. Tracks67–69 remain planned mechanisms
with exact-file handoffs; their registration does not confer runtime write authority.
Track65/66 reservations remain protected. No upstream source or worker inputs change.

## Required future receipts

R2.3 must identify the actual backend/device/host, run an actual kernel or physical
transport peer, retain source/toolchain/features/seed/input/output hashes and
compare an independent CPU/serial reference. Test unavailability, cancellation,
restart and output recovery without substituting metadata or CPU fallback for
device execution. The historical `pleiades` alias is not a verified CUDA-PC identity.
Hardware remains outside the headless pin gate unless a consumer requires it.

R0.4 may accept this classification without accepting physical hardware capability.
R1 must execute the current CPU/IO/replay baseline under the recorded toolchain;
it cannot convert these synthetic timing fields into measurements.

## Verification

[validation.json](validation.json) records planning integrity and source-bound
positive/negative controls. Changing a source hash is rejected; the classification
is not reusable for a different revision without reinspection. No Rust test pass
is claimed by this source audit. R0.1/R0.2 and this audit require independent phase
review before R1 acceptance can advance. PR174/mainline integration remains separate.
