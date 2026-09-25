# Ownership contract — calibration and validation

Primary upstream owner: Track 21 (VVUQ). Proposed reusable calibration crate needs
C0 ADR; do not bury reusable algorithms in the CLI or add them to core scheduling.

| Work | Owner / paths |
| --- | --- |
| Calibration/replay/metrics and validity | 21; proposed `crates/kairo-ecs-calibration` plus existing VVUQ docs |
| IPC/Parquet and schemas | 04; `crates/kairo-ecs-arrow`, schema assets |
| Mode/agent/transit execution hooks | 03; DES/ABM with model policies in CareOps |
| Experiment manifests, CLI and worker orchestration | 22; existing runner/CLI paths |
| Seed/task identities | 01; coordinated versioned contract |
| Fixture/benchmark integration | 12 |
| API/MSRV/dependency review | 25/30 |

Blocked without separate owner scope: replacing core scheduler, rewriting GPU/
PDES/network transports, adding Python runtime requirements, implementing a new
renderer or importing private EHR data. Browser snapshot handoff follows 05/09/33.

Parallel-safe after contracts: C1 Arrow IO, C4 pure metrics and queue work in
separate paths. C2 integrates against Q0; interruption-based C3 waits for Q4. C6
waits for Q5 and C5. A single owner coordinates shared schemas/Cargo manifests.

Every handoff records schema/API versions, fixture seeds, input/artifact hashes,
actual executed gates and limitations. Do not use a Done status as a substitute
for required runtime evidence.
