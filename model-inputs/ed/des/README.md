# DES evidence catalogue

Six proposed schema-v1 family collections cover 61 registered DES-facing IDs:
`demand_case_mix.json` (11), `des_pathways.json` (17), `durations.json` (9),
`resources.json` (14), `patient_behavior.json` (5), and
`hospital_interfaces.json` (5). Deferred entries remain explicit.

Each collection separates typed `records` from `annotations` containing hashed
local evidence references, acquisition plans, unselected candidate families and
accepted usage-matrix gates. [The contract](p13-contract.md) defines the scope;
original source summaries, provenance and readbacks remain in `evidence/`.

No generative value, default, fit or numerical range is established by these
collections. Unknowns are not zeros; IQRs and means are not extrema or fitted
parameters. This catalogue is not an executable ED profile. P3 establishes
identifiability/dependence and fit criteria; P4 provides reviewed synthetic
profiles; E0/E2 and P5 verify real consumers. Generic and later Cairns inputs stay
separate.

Run from the repository root:

```sh
python3 tools/validate_des_catalogue.py
python3 -m unittest discover -s tests -p test_des_catalogue.py -v
```

The checker enforces family/ID coverage, units, source-reference hashes and
semantic membership, unknown/deferred states, candidate status and boarding
output. Its numeric negative fixtures check probability and capacity shapes for
developer validation; they approve no runtime profile or clinical policy.

Two abandonment IDs describe one event: the patient-behaviour policy produces
the competing exit outcome and the pathway edge consumes it. P3/E0 must verify
that no second independent exit draw occurs. Macro aggregate diagnostic TAT
must not be added to Micro queue/work/transit components. Boarding records the
readiness-to-departure outcome, with resource-release and censoring semantics
explicit; it is not a sampled inpatient-capacity substitute.
