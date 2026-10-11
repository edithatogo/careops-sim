# E1.2 provisional disposition contract v1

Coordinator frozen after independent GPT-6.1-sol review; dispatch after diagnostics acceptance. Rust1.99. Experimental synthetic decision revelation only. Preserve legacy pathway APIs. Separate module disposition.rs, coordinator facade.

## Exact interface

Use pathway::{PathwayConfig,PathwayResult,PathwayError,ServiceProvenance,ServiceBinding}. Input types Clone/Debug/Deserialize with strict struct fields; enums snake_case; outputs Clone/Debug/Serialize/PartialEq/Eq.

DispositionIntent { Discharge, Admission }.
DecisionReveal { TreatmentCompletion }.
DecisionPolicy { version:u32, provenance:ServiceProvenance, reveal_at:DecisionReveal }.
DispositionBinding { patient_id:String, latent_intent:DispositionIntent }.
DecisionObservation { policy_version:u32, provenance:ServiceProvenance, observed_tick:u64, provisional_intent:DispositionIntent }.
DispositionResult { pathway:PathwayResult, decision:Option<DecisionObservation> }.
run_with_disposition(config:&PathwayConfig, policy:&DecisionPolicy, decisions:&[DispositionBinding]) -> Result<Vec<DispositionResult>,PathwayError>.
run_with_disposition_and_services(config:&PathwayConfig, seed:u64, services:&[ServiceBinding], policy:&DecisionPolicy, decisions:&[DispositionBinding]) -> Result<Vec<DispositionResult>,PathwayError>.

## Validation and knowledge

Policy version1, reveal_at TreatmentCompletion only, synthetic provenance with nonempty trimmed notes. At most100000 decisions and patients. Require exactly one decision binding per patient; reject missing, duplicate, extra and invalid IDs before native pathway execution or service RNG. All patients must have a full treatment-ending pathway; legacy two-stage inputs reject. Empty population accepts empty bindings and still validates policy/profile through pathway runner.

Run the actual accepted pathway runtime. Decision is None unless the last Treatment episode has completion_tick; then record that exact tick, policy version/provenance and supplied intent. Completion exactly at horizon reveals; active/queued treatment does not. The wrapper may construct observations after the run from completed episodes: it exposes no earlier timestamps or event stream and must not claim a live decision event mechanism. Latent intent is not copied into observable output before reveal; reject-to-error diagnostics must not leak latent values. Acuity knowledge remains a separate triage observation. No clinical prediction or policy optimization implied by supplied synthetic intent.

Discharge intent remains provisional until actual physical discharge; admission intent remains provisional until boarding/offer/actual transfer. These are E1.3 behavior. Do not add terminal counts, terminal timestamps, completed LOS, realized outcome, or silently convert treatment completion into departure.

## Native oracles

Paired configurations differing only in latent intent yield identical entire outputs while treatment unfinished, and identical pathway history always; only decision intent differs after actual completion. Check no diagnostics, one diagnostics and repeated diagnostics; actual sampled service timings drive observation timestamp. Patient/service/decision-binding order permutations produce equal output. Exact-horizon completion reveals, one tick earlier horizon does not. Empty population works. Duplicate/missing/extra bindings, malformed IDs, unknown policy version, untrimmed/non-synthetic provenance, and non-full pathway reject before execution. Include generated-arrival→template→service/pathway→decision composed test. Acuity known only after triage, independent of decision knowledge. No terminal claim and all E1.3 composite terminal assertions remain debt.
