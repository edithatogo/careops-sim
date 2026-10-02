# P4 synthetic profile pack guide

This guide describes the three version-1 P4 JSON data packs in this directory:
`p4-minimal-pack.json`, `p4-nominal-pack.json`, and `p4-surge-pack.json`.
They are compact, invented examples for the input shape and its checks. The
words “minimal”, “nominal”, and “surge” name fixture designs; they do not mean
minimum safe service, a typical ED, a real surge plan, or a recommended staffing
level. No value here is an observed or fitted ED parameter, clinical policy,
operational range, or Cairns value.

## Read the fields

| Field | Meaning and unit |
| --- | --- |
| `profile_pack_schema_version`, `profile_id` | Version and stable fixture label. Version 1 identifies this P4 data envelope, not an executable scenario. |
| `seed` | Unsigned 64-bit integer recorded for reproducibility. It does not define the eventual E2 named random streams or make the pack runnable. |
| `horizon` | Positive duration in seconds (`s`) from relative time zero. |
| `provenance` | Must classify this fixture as `synthetic` and state its invented assumptions. |
| `arrival_table` | Non-overlapping, ordered half-open time intervals `[start,end)` in seconds, with prescribed integer counts for each named arrival mode. Counts are fixture inputs, not rates or a generated arrival process. |
| `acuity_scales`, `case_mix_table` | Named category vocabulary (the examples use synthetic `ATS1`–`ATS5`) and integer counts per arrival interval and mode. Counts partition the supplied arrivals; they are not observed prevalence or an acuity recommendation. |
| `resource_calendars` | For each resource bucket and interval, `open_count` is the number available to operate and `staffed_count` the number supported by staffing. Both are nonnegative counts; they may be below physical capacity. |
| `staffing_calendars` | Nonnegative `present_count` by named role and interval. This is a fixture headcount, not a roster or a claim about concurrent task capacity. |
| `capacity_location` | P0.2 schema-v3 binding: named zones, locations, physical resource buckets, open/staffed capacity, task classes, roles, eligibility, and optional directed distance routes. `physical_count` describes physical inventory; open and staffed counts describe availability. Distances are metres (`m`). |
| `route_graph` | Named nodes reference those locations; directed edges carry nonnegative distance in metres. This is metadata only, not a floor plan, travel-time model, movement policy, or visualization. |
| `distributions` | Finite listed outcomes with an explicit unit and probability mass. Duration units distinguish active work seconds (`active_work_s`) from elapsed turnaround seconds (`elapsed_turnaround_s`); `category` outcomes are labels. |
| `initial_state` | Relative time and explicit occupants/remaining work. Empty arrays mean the fixture starts known empty; they do not mean the initial state is unknown. |

In the minimal example, the horizon is 3,600 s (one hour), seed 11, and arrivals
are one walk-in plus one ambulance arrival during `[0,1800)`, then one walk-in
and zero ambulance arrivals during `[1800,3600)`. Its one synthetic resource
and one role each have count 1 throughout. Triage duration is exactly 60 s and
work duration exactly 120 s in this deterministic fixture. Its route edge is
10 m. Zero counts are explicit zero, not missing information.

The nominal example spans 43,200 s (12 hours), seed 17. Its four three-hour
arrival blocks `[0,10800)`, `[10800,21600)`, `[21600,32400)`, and
`[32400,43200)` contain walk-in/ambulance pairs of 6/2, 8/3, 7/2, and 5/1.
Separately, its treatment-bay calendar has two open and staffed bays for the
first six hours, then two open but one staffed. Its diagnostic device is open
and staffed for the first six hours, then open but unstaffed. Role counts
likewise change at six hours, from 2 nurses, 2 clinicians, and 1 diagnostics
staff member to 1, 1, and 0.
The listed synthetic active triage work outcomes are 180/360/600 s with masses
0.20/0.50/0.30; assessment is 600/1200/2400 s with masses 0.25/0.50/0.25.
Diagnostics turnaround is 900/1800/3600 elapsed seconds with masses
0.20/0.50/0.30; cleaning active work is 300/600/900 s with masses
0.25/0.50/0.25. The disposition labels discharge/admission/transfer have
masses 0.65/0.30/0.05. These probability tables are toy inputs, not estimated
distributions or independent clinical probabilities.

The surge example has the same 12-hour horizon and 43,200-second total, seed
23. Walk-in/ambulance counts across its four three-hour blocks are 6/2, 12/4,
16/6, and 8/3. The surge-bay bucket has physical count 2 while open/staffed
counts move 0/0, 2/1, 1/1, then 0/0; the core bays remain physical count 2,
with staffed capacity falling from 2 to 1. Diagnostics are open but have zero
staff during the final six hours. This encodes a constrained synthetic fixture,
not an approved surge response or actual operational availability. Its finite
outcome tables use the same invented outcomes as the nominal pack.

## Schema bounds, checks, and uncertainty

The JSON Schemas describe representable values, not empirical ranges. For P4
time values, JSON strings must be nonnegative decimal seconds; horizons must be
positive. Counts are nonnegative integers, the seed is between 0 and
18,446,744,073,709,551,615, probabilities are individually in `[0,1]`, and
route distances are nonnegative metres. IDs are nonblank, required fields must
be present, and unknown fields are rejected. The P0.2 v3 capacity schema also
distinguishes known counts from unknown/null counts. These are format/type
bounds only. They do not say that any value in that domain is plausible for an
ED. There is no evidence-backed ED operating range in these packs.

P4.2 semantic validation adds relationships that JSON Schema alone cannot
express: chronological non-overlapping intervals and horizon coverage, case-mix
counts reconciled to arrivals, selected category/resource references, probability
masses, distribution support, resource-calendar counts bounded by physical
capacity, nonnegative staffing counts, and initialization/provenance consistency.
It does not establish staff-to-resource feasibility or validate every route-graph
and capacity/location reference. Known physical capacity is separate from open
and staffed capacity. Unknown availability must remain `null` with a reason;
never convert missing/unknown to zero or to available. The validation corpora
also show malformed, zero, censored, closed, repurposed, and surge cases. They
are tests, not observations.

The outcomes listed within a fixture express only the toy outcome variability
encoded by that table. The examples provide no epistemic uncertainty interval,
parameter confidence, fitted tail, dependence model, sampling-level rule, or
evidence about between-site variation. A single value with probability 1 is
deterministic fixture behavior. Multiple outcomes are not empirical merely
because they carry probabilities. Do not treat the seed as evidence of
reproducible execution until a runtime defines how it consumes this data.

All pack files are classified synthetic in their provenance. The manifest
records SHA-256 hashes over exact file bytes and explicitly says external
redistribution rights are not established. A hash establishes byte identity,
not authorship, truth, a licence, or permission to redistribute. Follow the
manifest's conservative disposition and obtain the applicable rights before
external redistribution.

## Validate and make a separate override

From the repository root, structural validation checks JSON shape and schema
constraints. It is a separate command from P4 semantic validation, which checks
cross-field relationships. Run both for the same pack; for example:

```sh
python3 tools/validate_ed_profile_pack.py model-inputs/ed/profiles/p4-nominal-pack.json
python3 tools/validate_ed_profile_pack_semantics.py model-inputs/ed/profiles/p4-nominal-pack.json
```

The structural validator checks the P4 and referenced capacity/location schemas
and the pack's shape. The semantic validator reads the pack and checks rules
such as ordered intervals, reconciled arrival/case-mix counts, probability
masses, calendar coverage, capacity feasibility, references, initialization,
and provenance. Neither command executes the profile. To check that the checked-
in artifacts still match the exact-byte manifest, run:

```sh
python3 tools/validate_ed_profile_manifest.py model-inputs/ed/profiles/p4-pack-manifest.json
```

To explore a change, copy a pack to a new, clearly named file such as
`p4-nominal-local-override.json`; do not edit the accepted fixture in place.
Update its `profile_id`, `provenance.class`/notes (the current P4 schema only
accepts `synthetic`, so a locally observed profile requires a future reviewed
contract/schema change), and all related fields consistently. For example,
changing an arrival count requires its case-mix categories for the same mode and
interval to reconcile; changing availability requires corresponding calendar
counts and capacity bindings to agree. Keep unknowns explicit, never invent a
value to satisfy a check. Run both validators above against the copy, review
every diagnostic from both validators, document each assumption and source
separately, and preserve the original files and manifest. The commands do not
enforce every correspondence between staffing calendars and resource bindings;
review those together when changing an override. The checked-in manifest covers
only its listed exact files; it does not automatically cover or license an
override.

## Loading boundary

These P4 packs are data-only, versioned examples. They are not E0 scenario JSON
and are not accepted by `careops-ed run`. E0's executable projection has a
different, narrower shape and does not consume these arrival tables, calendars,
probability tables, initial state, or route graph as a full simulation input.
P5 is the planned gate for binding/loading packs through the completed ED model
and calibration runner, and depends on E2 and C5. Passing the current structural,
semantic, or hash validators proves only those checks; it does not prove a
simulation ran, that the values are valid for an ED, or that they apply to
Cairns.
