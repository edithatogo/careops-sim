# E1.2 arrivals contract v1

Coordinator frozen 2026-10-11 after GPT-6.1-sol independent planning review.
Preparation only until E1.1 accepted and packet bound. Experimental, synthetic
count-conditioned arrival placement. Rust/Cargo1.99.0 only. Independently authored
integer-tick inputs; no full P4 loader, seconds conversion, Poisson process,
pathway assignment or clinical prevalence claim. P4 specifies counts, so retain
exact mode/category counts. Within-bin placement is an explicit new synthetic
assumption, not a measured dependence model. Full pack consumption remains P5.

## Interface

One module crates/careops-ed/src/arrivals.rs, with unit tests. Coordinator adds
`#[doc(hidden)] pub mod arrivals;` and existing pinned kairo-ecs-rng dependency.
Retain E0 schema1. This is an experimental development API, no stable API claim.
All input structs derive Clone, Debug, Deserialize and deny_unknown_fields:

- ArrivalConfig { version:u32 (1), seed:u64, horizon_ticks:u64,
  provenance:String (synthetic_tick_fixture), bins:Vec<ArrivalBin> }
- ArrivalBin { start_tick:u64, end_tick:u64, modes:Vec<ArrivalModeCounts> }
- ArrivalModeCounts { mode:String, total:u64, acuity_counts:[u64;5] }
- GeneratedArrival derives Clone,Debug,PartialEq,Eq,Serialize, with
  patient_id:String, arrival_tick:u64, mode:String, acuity:u8.
- ArrivalError derives Clone,Debug,PartialEq,Eq: InvalidVersion,
  InvalidProvenance, InvalidHorizon, LimitExceeded, InvalidMode, DuplicateMode,
  InvalidBin, OverlappingBins, CountMismatch, ArithmeticOverflow,
  SamplingExhausted. Implement Display and std::error::Error.
- pub fn generate_arrivals(config:&ArrivalConfig)
  -> Result<Vec<GeneratedArrival>,ArrivalError>.

## Validation before output allocation or sampling

Positive horizon; 0<=start<end<=horizon; nonoverlapping bins including duplicate
interval rejection. Unsorted bins/modes accepted and canonicalized. At most4096
bins, at most32 mode rows per bin and32 distinct modes overall, at most100000
patients. Mode identifiers1..64 ASCII characters [A-Za-z0-9_.-], case-sensitive.
Reject duplicate modes within bin. Each total equals checked sum of five acuity
counts; total across bins checked before output allocation. Check per-bin vector
limits before cloning/sorting. Invalid-version/provenance/horizon and size checks
precede semantic checks; callers should rely on named variants, not a global
priority among multiple simultaneous invalid fields. JSON serde rejects unknown
fields and acuity-array lengths other than5. Empty configuration/mode lists and
zero totals allowed; an omitted mode means zero count for that bin.

## Identity and ordering

Canonical traversal: bins (start,end), modes ASCII bytes, acuity1..5, then ordinal
within category. Global modes sorted bytewise for mode index. Before each sample
assign `arrival-v1-b{bin_index:04}-m{mode_index:02}-a{acuity}-n{ordinal:06}`.
Indices/ordinal zero-based. Return sorted (arrival_tick,patient_id). Input
permutations preserve bytes; seed changes preserve the ID set. Changing bin/mode
structure may change IDs; append stability is not claimed.

## Dedicated RNG and integer placement

ARRIVAL_DOMAIN_INDEX=0x434152454F505341u64. Use existing pinned
kairo_ecs_rng::derive_entity_seed(seed, EntityId::new(ARRIVAL_DOMAIN_INDEX,0))
and DeterministicStream::from_seed. Domain reserved exclusively for arrivals;
future service/transit streams must use different domains. No upstream RNG edits.

For width=end-start: width1 returns start with zero draws. Otherwise threshold=
width.wrapping_neg()%width. At most128 next_u64 draws: reject x<threshold; accept
x%width, then checked start+offset. 128 rejections => SamplingExhausted with no
returned output. Never float conversion, hash iteration ordering or biased direct
modulo without rejection. Entire schedule uses this one dedicated stream in
canonical order. This is count-conditioned discrete uniform placement; simultaneous
arrivals are allowed, not a continuous-time point process or independence claim
about clinical phenomena. Seed changes need not change every tick.

## Verification

Independent vector generation reproduces the pinned upstream seed derivation and
state recurrence directly, never invokes this new generator. Check derived seed,
raw draws, IDs/ticks for non-power-of-two width. Unit tests must include empty,
width1/no draws, adjacent bins/excluded ends, exact mode/acuity counts, permutation,
stable IDs across seeds, total100000 accepted and100001 rejected, every validation
variant, serde rejection, rejection threshold and128-rejection exhaustion through
a private controlled-draw helper. Real generator golden vectors remain mandatory.
No helper-only test can establish end-to-end generator correctness.

E1.2 arrival acceptance is this native generator only. It does not satisfy the
unbound triage/diagnostics/disposition/boarding cases or complete parent E1.2.
