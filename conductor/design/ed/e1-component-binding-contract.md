# E1.2 native fixture-component binding contract

Coordinator frozen after independent GPT-6.1-sol review. Dispatch after diagnostics accepted as a separate E1.2.disposition fixture-components instance, parallel to native disposition writer. Both instances join before parent acceptance. Only owned source crates/careops-ed/tests/e12_fixture_components.rs; no production/fixture edits. Rust1.99.

Read the frozen three E1 family JSONs via include_str and run actual pathway APIs. Assert exact case IDs and counts:23 endpoints,6 extremes,17 invalid; no skipped/unmapped cases. Fixture arithmetic must not replace runtime; compare independent frozen expected fields against actual native episodes.

## Endpoints

Construct explicitly synthetic capacity profile for named stage resources and task eligibility. Use only input patient IDs, arrival ticks, latent acuities, stages/durations; supply mode synthetic. Empty fixture runs an empty native population with a valid profile. Compare result identity, acuity and known acuity, episode kind/start/completion, first-care wait, sum of stage waits and observed work against expected fields. Assert stage input resource mapping corresponds to frozen resource IDs, while noting StageEpisode currently does not expose resource identity; do not falsely claim an output resource trace. All22 nonempty stage paths execute through TreatmentComplete. Do not assert actual terminal outcomes/counts/ticks, bed offers, boarding/transfer, completed LOS or whole-case binding.

## Extremes

Map generic service interval to triage, append positive assessment using a distinct resource pool with adequate capacity. Compare only triage first start/completion/observed work and first-care wait; any count is explicitly triage completed/unfinished, never patient terminal. Zero-service must reject before output. Compare saturated/permuted triage projections exactly. Later assessment is outside the single-stage expected projection.

## Invalids

Each of17 cases has an explicit mapping to native fields. Build an otherwise legal two-stage synthetic baseline, assert native success, apply only the targeted fixture mutation, assert PathwayError::InvalidInput with reviewed native message substring, repair only the mutation group and assert success. Disordered/repeated sequence remains intentional, not accidentally repaired by appending a stage. Duplicate-ID case duplicates a legal patient. Resource/class/task/eligibility mutations target actual profile/stage fields. No generic is_err-only claims or irrelevant schema error masking. Match fixture offending_path/category metadata to the mapping so drift fails. Returned error demonstrates rejection; validation-before-scheduler ordering is established separately by source review, not fabricated event-count telemetry.

All bindings are partial native component evidence. Historical runtime_status stays unbound in frozen specification JSON. Whole endpoints and E1.3 terminal assertions remain debt. E1.4 must eventually bind complete cases to native end-to-end runtime.

## Reviewed invalid diagnostic mapping

Read actual malformed fixture values; do not replace them with invented invalid constants. Capacity fixtures may change several count fields together: preserve the full malformed triple and repair its count group consistently (e.g. all-zero becomes positive known1/1/1; open0/staffed0 repaired together). The rest of native baseline remains unchanged. For repeated/disordered stages map fixture kinds to the otherwise legal stage positions; retain distinct physical resource assignments so sequence rejection is isolated. Diagnostics repeat support does not legalize repeated triage.

| Case suffix | Native mutation/field | Required error substring |
| --- | --- | --- |
| acuity_invalid | patient.acuity | acuity must be 1..5 |
| arrival_at_horizon | patient.arrival_tick | arrival must be before horizon |
| capacity_inconsistent | resource physical/open/staffed from fixture | open_count exceeds physical_count |
| capacity_open_zero | resource count triple | known positive physical, open, and staffed capacity |
| capacity_staffed_over_open | resource count triple | staffed_count exceeds open_count |
| capacity_staffed_zero | resource count triple | known positive physical, open, and staffed capacity |
| capacity_unknown | physical Unknown/None; other counts retained | known positive physical, open, and staffed capacity |
| capacity_zero | resource count triple | known positive physical, open, and staffed capacity |
| duration_nonpositive | triage duration | stage duration must be positive |
| eligibility_missing | remove only triage matching eligibility, retain assessment | no matching staff/task/zone eligibility |
| horizon_nonpositive | horizon_ticks | horizon_ticks must be positive |
| patient_id_duplicate | patient IDs from fixture | patient IDs must be unique |
| resource_class_incompatible | triage resource class porter, task requires nurse | incompatible with resource |
| resource_unresolved | triage resource reference missing | refers to unknown resource |
| stage_disordered | assessment then triage kinds | stages must |
| stage_repeated | triage then triage kinds | stages must |
| task_unresolved | triage task reference missing | refers to unknown task class |

Native stage error wording may gain supported full-pathway alternatives in diagnostics; keep stable structural substring plus explicit InvalidInput variant. Baseline task/resource references map synthetic fixture names explicitly. No typed fixture diagnostic taxonomy is claimed in production.

Acceptance requires all46 cases enumerated,17 passing invalid repair controls, and both disposition instances accepted before E1.2 closeout. Recheck diagnostic substrings against integrated diagnostics source at binding; stale wording requires reviewed rebind, never weaker assertions.
