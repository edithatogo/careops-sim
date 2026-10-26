# R2.1 — proposed source-preserving candidate

**Status: R2.1 candidate selection executed and verified; gate-result.json confirmed candidate_for_qualification at 8282b1ab; pin 44261eb retained.**
This preparation starts at merged careops-sim `5de9e4ef81d5da96f048ee18e7170354d9247c26`, after PR174 and PR175 merged in order. The parent runtime pin remains **44261eb5cf8275f132f63a3396903d116d701dbe**. No Kairos source, manifest, dependency lock, fixture, protected plan/status record or gitlink was edited.

## Proposed candidate and alternatives

The proposed bounded candidate for **later qualification** is
`8282b1ab45669a667bda3be2862272447544ba44`, published at
`codex/e61-numeric-batch-profiling-oracle-20261017`. The [remote readback](remote-refs.log)
resolves its immutable SHA. Pin ancestry passes. All **321** reviewed pin-contract
paths are byte-identical at this candidate; see [raw inventory](selected-inventory.json)
and [source preparation](source-preparation.log). This is source preservation,
not a fresh runtime or upgrade qualification.

[The delta](delta-8282b1ab.log) contains four GPU-crate files only: an additive
numeric_batch module/public re-export and its CPU/reference-oracle tests. The
existing GPU availability implementation, all consumed core/DES/ABM/Arrow-IO/
calibration/CLI paths, Cargo manifests, Cargo.lock and toolchain file are unchanged.
[Manifest comparison](selected-manifest-lock.log) and [public delta](selected-api-delta.log)
retain those distinctions. Additional public GPU helpers still need Track25/32 review;
source preservation does not approve them for release or establish performance.

| Alternative | Source finding | Proposed disposition |
| --- | --- | --- |
| Current pin 44261eb | Frozen R1 baseline remains usable | Retain until R2/R3 qualification and owner acceptance |
| 8282b1a | Pin descendant; 321 reviewed paths unchanged; four bounded GPU-only additions | Proposed minimum candidate for subsequent qualification, pending independent review |
| d6ab1bd / 87dabcc | Pin descendants with larger GPU changes, RealDevice availability variant and host-constructed device metadata | Hold: extra API/availability and physical-device claims need their own owner/oracle review; ancestry alone is insufficient |
| Planning main 55b20e7 | Divergent; R0 inventory found 273 reviewed paths absent and 34 changed | Hold for capability-preserving integration; no automatic pin advance |

The [later availability delta](later-availability-delta.log) exposes why the larger
branches are not interchangeable with the minimum candidate. Their status names
are not actual adapter/driver/readback receipts. GPU or multi-host acceptance is
outside this headless source proposal.

## Owner interfaces and qualification handoff

[Owner input identities](owner-inputs.json) bind specs and plans at current Kairos
planning main 55b20e7. The captured owner logs are source inputs, not sign-offs:

- **03:** preserve scheduler-facing DES/ABM, queue/preemption, allocation/cancellation and checkpoint lifecycle; all consumed source remains unchanged.
- **04:** preserve versioned telemetry schemas and actual IPC/Parquet interop. Source continuity is not independent-reader execution on a newly qualified candidate.
- **21:** preserve replay, seed/manifest/holdout semantics and distinguish verification from clinical validation.
- **22:** preserve actual native C5 workers, outputs and resumability; the R1 original qualifier uses Rust1.99.0 while the other native lanes use1.99.1.
- **25:** preserve protected Rust/FFI/Arrow/host surfaces and review optional GPU public additions before release.
- **32:** treat new batch helpers as CPU/reference contracts; retain backend-unavailable and real-device gates until actual execution is demonstrated.

No owner is declared to have accepted the candidate. Track65/66 reservations and
the original dirty Kairos checkout remain untouched. Future neutral runner work
still follows R3; this proposal introduces no additional framework.

Before any pin decision, execute R2.2 consumer/native/independent-reader gates,
R2.3 applicable runtime checks, R2.4 review and R3 baseline comparison/rollback.
Bind the frozen parent 1fef8d4 and child44261eb, actual features, both toolchains,
source/input/seed/output hashes and six ignored baseline cases. The
[R1 baseline](../r1-native-baseline-20261021/README.md) and
[harness evidence](../r1-harness-baseline-20261021/README.md) remain authoritative
for their recorded source and scope. The current pin is retained throughout.

## Dependency review and dispatch status

The registry refresh at 2026-10-21T13:04:57Z has 33 available observations;
all remain candidate_unvalidated. Only Burn0.22.1, nextest0.9.149 and
Renovate44.175.2 changed version observations from the previous report. Nothing
was installed or upgraded. [Prior snapshot](dependency-snapshot-before.json)
and [refresh log](registry-refresh.log) preserve the actual comparison.
Candidate/pin manifests, lockfile and GPU Apache-2.0 licence declaration are unchanged;
this does not grant fresh advisory, MSRV/platform or release clearance.

The [canonical packet](packet.json) passed tools/tasks.py packet-check at
preparation source5de9e4e with a 23,127-byte context and eight bounded checks.
Its [proposed source gate](inspect_candidate.py), positive selection and negative
selection controls have **not** been executed: the requested independent packet
review is unavailable because the Codex messaging and desktop tools returned
Transport closed. Reopening the desktop app in the background did not restore
those transports. Only the previously reviewed R0 source collector and read-only
Git/registry preparation were executed. [Receipts](preparation-receipts.json)
retain actual argv/cwd/exit/log hashes; no pass is inferred for unexecuted checks.

The packet is a retained preparation snapshot, not future dispatch authority.
After this evidence commit, rebind its base and hashes and obtain the independent
packet/owner review before source-gate execution or R2.1 acceptance. Shared plan,
resume and catalog records still correctly offer R2.1 as unchecked.

The directory-local attributes preserve verbatim whitespace in three raw Git/source logs only; source and other evidence checks remain unchanged.
