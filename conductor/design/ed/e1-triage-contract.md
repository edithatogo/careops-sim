# E1.2 triage/assessment contract v1

Coordinator frozen 11 October 2026 after independent GPT-6.1-sol review. Rust 1.99 only. Additive experimental pathway module; E0 schema1 remains unchanged. Synthetic tick inputs and supplied work, no clinical or stable-public-API claim.

## Exact API

All structs public, Clone/Debug; input structs Deserialize with deny_unknown_fields; outputs Serialize/PartialEq/Eq. Enums use snake_case serde representation.

- PathwayConfig { version:u32, horizon_ticks:u64, provenance:crate::Provenance, capacity_location:crate::CapacityLocationProfile, patients:Vec<PathwayPatient> }
- PathwayPatient { patient_id:String, arrival_tick:u64, mode:String, acuity:u8, stages:Vec<StageWork> }
- StageKind { Triage, Assessment }, Copy/Eq/Ord.
- StageWork { kind:StageKind, resource_id:String, task_class_id:String, duration_ticks:u64 }
- StageEpisode { task_id:String, kind:StageKind, queue_entry_tick:u64, start_tick:Option<u64>, completion_tick:Option<u64>, observed_work_ticks:u64 }
- PathwayState { Queued(StageKind), Active(StageKind), AssessmentComplete }
- PathwayResult { patient_id:String, mode:String, acuity:u8, known_acuity:Option<u8>, arrival_tick:u64, episodes:Vec<StageEpisode>, state:PathwayState }
- StageTemplate { mode:String, acuity:u8, stages:Vec<StageWork> }
- PathwayError implementing Display/Error; typed enum variants InvalidInput(String), Runtime(String).
- run_pathway(&PathwayConfig) -> Result<Vec<PathwayResult>,PathwayError>
- bind_arrivals(&[crate::arrivals::GeneratedArrival], &[StageTemplate]) -> Result<Vec<PathwayPatient>,PathwayError>

## Validation and limits

Version1, positive horizon, provenance.class synthetic and nonempty trimmed notes. At most100000 patients, at most1024 resource buckets, zones, locations, task classes, staff roles, eligibility rows, routes; at most32 counts per role and32 resource capabilities or required classes. All profile identifier fields and references (excluding free-text names/notes), patient IDs, stage references and modes1..64 ASCII alphanumeric/underscore/dot/hyphen. Unique patient IDs; acuity1..5; arrival<horizon; exact stage order triage then assessment, positive duration. Distinct resource IDs for stage kinds across the entire run (patients may name different resource buckets for the same stage kind; each work item claims one unit of its single named resource; no resource may serve both kinds). Checked completion arithmetic uses u128 internally, so long u64 durations beyond horizon remain censored without overflow. Maximum scheduled events is three per patient (arrival plus two completions); never schedule completions beyond horizon.

Validate all inputs before creating Kairos Scheduler or output state. Coordinator exposes crate-private validate_capacity_work(profile,work_items,horizon), extracted from E0 validation; this checks profile even for empty patients. Flatten stage references into unique synthetic validation work-item IDs without changing actual task IDs. Existing E0 validation still rejects empty patients. Require known positive physical/open/staffed resource counts for E1 in addition to shared validation. Resource assignment is explicit and must satisfy class and role/zone/location eligibility; no staff identity or multi-resource claims implied.

bind_arrivals validates all rows, unique template (mode,acuity) keys and exact two-stage shapes, IDs/count bounds and positive durations; missing templates reject. Mode/acuity mapping is supplied, never inferred clinically. Templates may be unused. Cap templates1024. Full profile/horizon checks occur in run_pathway. Preserve generator IDs/times/modes/acuities exactly.

## Execution and knowledge

Use pinned Kairos Scheduler and deterministic event processing. Canonical result order(patient_id). FCFS is queue-entry tick then patient ID, no acuity priority. Process all same-tick arrivals/completions and enqueue next stages before granting queued work, independently of insertion order. Then grant in deterministic resource-ID and queue order. Capacity=min(open,staffed). Each patient has at most one active stage. Completion exactly at horizon counts and may start the next stage at horizon with zero observed work. No grants beyond horizon. Queue entries equal arrival or preceding completion. Episodes include only entered stages, including queued ones; work is min(completion/horizon,start+duration)-start, zero while queued. AssessmentComplete is a continuation state, never discharge/admission. Acuity is latent input, known_acuity remains None until triage completion; assessment never observes future knowledge.

Task IDs are collision-free stable strings patient_id + ':' + stage name; colon is forbidden in IDs. Stage interval durations exclude queue waiting. No random service draw in this leaf; supplied durations retain config provenance. Later diagnostics leaf must add explicit configurable service distributions and provenance rather than rebrand supplied constants as empirical fits.

## Native acceptance oracles

Empty valid profile gives empty result. Arrival1, triage2, assessment3 gives [1,3),[3,6), observed work5. All acuity1..5 retain label and same FCFS timings; known_acuity appears only after triage completion. Two same-time patients capacity1 yield assessment intervals[3,6),[6,9), second assessment wait1. Reversed patient/resource/template order produces identical results. Include same-tick completion/arrival competition, queued and active censoring, exact-horizon completion and start, independent stage pools, capacity>1, stable task IDs, generated-arrival adapter, maximum bounds and invalid mutations (IDs, acuity, stages, duration, reference/class/eligibility, unknown/zero/inconsistent counts). Reject before returning partial output. Native assertions use hand-fixed expectations, not fixture arithmetic as runtime. All composite fixture bindings remain unbound until their complete owner behavior is implemented.
