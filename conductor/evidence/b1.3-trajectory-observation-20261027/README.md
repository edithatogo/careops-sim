# B1.3 bounded trajectories and observation-only policy — candidate evidence

Status: accepted local synthetic conformance at reviewed commit `56fbef657f442ade5f63d378c00a3f9676182f0c`.

## Scope

Added a helper that composes core `ObservedPolicyState` using the episode's current observed tick and caller-supplied versioned urgency assessment and elapsed wait. The policy type structurally omits patient state, service state and queued future events. Assessments later than the episode observation tick are rejected. No birthing or core policy semantics, targets, or clinical rules were added.

The trajectory control runs seeds `1..=16`, with a test-only hazard rate of `0.25` per discrete simulation tick, an inclusive horizon of 32 ticks, and a maximum of 32 executed events. An event is scheduled at tick 33 as a negative control and must remain unexecuted. All values are synthetic test controls, not fitted or clinical parameters.

## Executed verification

Working directory: repository root. Rust toolchain: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`; `cargo 1.99.1 (5f94df478 2026-08-27)`.

- Packet preparation: final packet was checked with `.venv/bin/python tools/tasks.py packet-check conductor/execution/packets/B1.3.trajectory-observation.json` in a temporary shared clone at exact base `5135a26f0729926cfe33f31ad8e8429764a2bd22`; exit 0; all seven input hashes matched. Reviewer-agent independently confirmed the seven hashes against that base. An earlier archive-only extraction lacked Git metadata and returned base drift; repeating in a Git clone resolved it.
- `cargo fmt --all -- --check`; exit 0.
- `cargo test --locked -p careops-birthing`; exit 0; 7 runtime conformance tests passed, including full-field urgency assessment preservation.
- `cargo test --locked -p careops-core --test policy_classification_tests`; exit 0; 6 core policy tests passed.
- `git diff --check`; exit 0.
- A packet-check invocation in the edited task checkout returned expected base/input drift after HEAD advanced and the two crate files changed. The clean clone at the packet base passed; the post-edit result is not represented as a runtime test pass.

The new tests verify equal policy inputs and decisions across different patient/service states and pending future-event queues, preservation of the complete urgency assessment (jurisdiction, scheme/version, category, assessed tick, effective date, source authority, citation and optional source hash), rejection of a future assessment, same-seed replay across 16 runs, monotone event ticks, the event bound, inclusive censoring at tick 32, and non-execution of the tick-33 event.

## Hashes

- B1.3 packet: `9931deb84eb69b18805b6f50acf089d35473cc46bfabdf8927e88f5800ca307d`.
- `crates/careops-birthing/src/lib.rs`: `0ed92a9ab3e7b90326f5ab56d1736249ccdd2a0c4d29babfdb3c59cb7545a6cf`.
- `crates/careops-birthing/tests/runtime_conformance.rs`: `219293e140c1a809fc0c97fc6a24a57be079fc6466c5bc8a835bda821653ea5d`.
- B1.1 fixture used by the runtime suite: `model-inputs/birthing/fixtures/b1-progression-cases.synthetic.json`, `ba6e039a32ca3c5499a1fdc5e79008a061ac23e005b8ab72b59b4894429f8e45`.
- Core policy contract input: `crates/careops-core/src/pathway/classification.rs`, `861666b93cd38229743a4ba27e04e31514beb4efc04395bdb63f519477761d4b`.

## Reviewer-agent finding and correction

Reviewer-agent review at `b8fc9acedc49a86fe7aa2b6bb3151d70750233cf` found that the test did not assert preservation of every urgency provenance field. The test now compares the complete `VersionedUrgencyAssessment` value, and the packet explicitly requires that full preservation; the packet reviewer verified all seven base input hashes. Fresh reviewer-agent review at `56fbef657f442ade5f63d378c00a3f9676182f0c` passed with no remaining findings. Coordinator acceptance is limited to the local synthetic contract.

## Limits

These are local deterministic synthetic checks. They do not validate clinical prediction, calibration, urgency policy, local site policy, operational outcomes, or multi-domain integration. The policy interface is deliberately limited to the existing core `ObservedPolicyState` fields; patient/service state visibility would require a separately reviewed contract change.
