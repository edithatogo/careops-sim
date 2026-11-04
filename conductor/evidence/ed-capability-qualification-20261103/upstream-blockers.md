# Kairos integration handoff — exact pin 07410266

Read-only source review; no Kairos file, pin, owner or private native scope changed.
These findings block consuming the listed mechanisms for ED G3. They are not a
claim that every other upstream consumer is affected or that native tests ran.
Existing Kairos GPU/PDES/distributed owners retain all implementation paths.
[Source hashes](source-readback.json) bind the observations to this pinned source.

| Owner / surface | Exact source observation | Required acceptance before ED consumption |
| --- | --- | --- |
| GPU owner, `metal.rs` Clone | `Clone` clears `native_ctx` while retaining session/adapter metadata | A cloned/reinitialized backend must report actual availability; explicit device selection cannot silently run CPU |
| GPU owner, device selection/provenance | Native wgpu initialization requests a default adapter without verifying Metal; `adapter_info` comes from host probing rather than selected adapter | Record actual selected backend/device/driver; fail explicit Metal selection if unavailable; no invented hardware identity |
| GPU owner, residency/performance | ABM submits real wgpu commands, then copies/maps full particle state each tick while `downloaded_bytes` is zero; DES remains CPU session execution | Honest transfer counters and measured end-to-end workload timings; do not claim no host readback, zero-copy, native DES or material ED gain |
| gRPC owner, `socket.rs` framing | Custom length/type TCP frames and `real_grpc_runtime_claimed() == true`; no tonic/prost/HTTP2 runtime | Protocol interoperability and actual runtime identity; custom TCP must not certify gRPC |
| MPI owner, `process.rs` | TCP data plane plus file/environment peer discovery, unconditional runtime-claim flag; smoke test spawns ordinary local processes with MPI-style environment variables | Actual MPI API/runtime interoperability; separate local processes from physical hosts and runtime evidence |
| Distributed owners, collective frontier | Socket barrier broadcasts without waiting. Both collective-minimum paths can return a local/partial minimum after timeout; MPI barrier can time out with missing ranks | Missing/delayed/failed peer must return failure, never a successful barrier or unsafe GVT/frontier; retain in-flight/dedup/checkpoint generation and fault-recovery controls |
| Parent ED owner | ED/core manifests and execution paths do not consume GPU, PDES, gRPC or MPI | Bounded real ED workload integration, exact serial observable oracle, backend-specific recovery and full independent device/multi-host/performance qualification |

The existing feature names, boolean runtime claims, test counts, process counts,
constructed timing constants and historical checkboxes are insufficient receipts.
No upstream test is run merely to obtain a misleading green result from these
oracles. No new transport, dependency, remote host access or pin promotion is
attempted by this parent repair.
