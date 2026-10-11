# E1.2 diagnostics/treatment contract v1

Coordinator frozen after GPT-6.1-sol and independent Luna review; dispatch requires accepted triage. Rust/Cargo 1.99 only. Extend experimental pathway runtime, preserve E0 and legacy two-stage inputs/results (except additive episode metadata).

## API

StageKind adds Diagnostics, Treatment. Accept either [Triage,Assessment] or [Triage,Assessment,Diagnostics{0..16},Treatment]. Finite supplied diagnostic repeats are supported; no implicit cycles. Each resource bucket serves one stage kind, and each work item claims one unit of its single named resource.

Keep StageWork.duration_ticks and run_pathway. Add ServiceSpec { version:u32, provenance:ServiceProvenance, law:ServiceLaw }; ServiceProvenance { class:String, notes:String }; ServiceLaw { Fixed { duration_ticks:u64 }, Discrete { outcomes:Vec<ServiceOutcome> } }; ServiceOutcome { duration_ticks:u64, weight:u64 }; ServiceBinding { patient_id:String, stage_index:usize, service:ServiceSpec }. All Clone/Debug/Deserialize/Serialize/PartialEq/Eq, strict struct fields and snake_case enum variants.

run_pathway_with_services(config:&PathwayConfig, seed:u64, bindings:&[ServiceBinding]) -> Result<Vec<PathwayResult>,PathwayError>. Exactly one binding per configured stage, no extras/duplicates/missing. StageWork duration must equal Fixed duration, or be zero as an explicit unsampled sentinel for Discrete in this API only. Legacy run_pathway rejects every zero duration. No hidden default or fitted-service claim.

StageEpisode adds episode_index:u32, sampled_duration_ticks:u64, service:ServiceSpec. Legacy supplied durations are represented as Fixed with config synthetic provenance. Episode index counts occurrences of its kind starting0. Preserve task IDs for Triage/Assessment; Treatment uses patient_id+':treatment'; Diagnostics uses patient_id+':diagnostics:'+episode_index. These task IDs remain unique and input-order invariant. Add PathwayState::TreatmentComplete, a nonterminal continuation; AssessmentComplete remains legacy two-stage end. No realized discharge/admission.

## Validation and execution

All prior limits retained. At most19 stages/patient, at most16 diagnostic episodes; at most20 scheduler events/patient including arrival, total checked before allocation. All input/profile/binding validation precedes service draws, scheduler or output state. Version1, provenance.class synthetic with nonempty trimmed notes. Fixed duration positive. Discrete support1..64 distinct positive durations and positive weights, accepted in any input order and canonicalized by duration before selection and echo; checked sum rejects u64 overflow. Config and all references remain bounded. u128 completion arithmetic preserves long-work censoring. Same-tick release/enqueue/grant barrier, per-resource FCFS and no overlapping patient work remain unchanged.

Sample service exactly once on entering an episode queue, including an episode still queued at horizon. Echo canonical ServiceSpec and sampled duration in the episode. Waiting/grant timing consumes no further draw. Unentered episodes produce no result or draw. Fixed consumes zero draws. Distribution metadata states synthetic assumptions; no clinical empirical independence claim.

## Deterministic sampling

SERVICE_DOMAIN_INDEX=0x434152454F505353. domain_seed=derive_entity_seed(root_seed,EntityId::new(SERVICE_DOMAIN_INDEX,0)). Canonical patient ordinal is bytewise-sorted patient-ID index. episode_slot=checked(patient_ordinal*32+stage_index); episode_seed=derive_entity_seed(domain_seed,EntityId::new(episode_slot,0)); DeterministicStream::from_seed(episode_seed), unchanged pinned Kairos RNG.

For Discrete total weight W, W=1 consumes no draw. Otherwise threshold=W.wrapping_neg()%W; reject next_u64<threshold at most128 attempts; error returns no partial results. Accepted draw%W selects first sorted duration whose cumulative weight exceeds it. Fixed consumes no draws. Patient/stage-set changes may change streams; append stability is not claimed. Permutations, queues, resource grants and other episodes cannot advance this episode's stream.

## Oracles

Legacy triage timings/knowledge remain equal. Full one-patient arrival1 durations2,3,optional4,5 completes treatment11 without diagnostics or15 with one. Two diagnostic repeats have distinct IDs and conserve work/resource intervals; include queued/active repeat censoring. TreatmentComplete never implies terminal outcome/LOS.

Independent root11 vectors must freeze domain/episode seeds and draws for slots2,3 with support[(2,1),(5,2),(9,4)] before dispatch. Require complete native run assertions, support/patient/binding permutations, fixed equivalence, zero-draw tests, invalid distributions/bindings, bounded rejection/exhaustion helper and identical samples under capacity-induced queue delays. Component fixture assertions do not close composite terminal bindings; see e1-runtime-binding-ownership.md.

## Independently calculated vectors

GPT-6.1-sol calculated pinned upstream arithmetic independently: root11 gives domain seed15382257649166804290. Slot2 episode seed11268637777555707280, first raw draw11357213296724872813, modulo7=5, sampled duration9. Slot3 episode seed9156490916263976673, first raw draw7554426951324197283, modulo7=3, duration9. Threshold2; both first draws accepted. Intervals [0,1) map2, [1,3) map5, [3,7) map9. Production tests must assert the full runtime outcomes as well as the sampler.

Require selector boundary tests for every support bucket, controlled threshold rejection and W=1 zero draws; add complete native runs with selected root seeds that reach all support durations (fixed independently before coding assertions). Explicitly test16 diagnostics accepted and17 rejected, sample retained for a queued-at-horizon episode, and capacity changes that alter waits but not samples.
