# Proposed ED ABM parameter catalogue

These schema-v1 catalogues list the ABM inputs and evidence/gap state for P2.3.
They are not executable profiles, calibrated values or a validated generic ED
behavior model.

| Catalogue | Registry family | Rows | Scope |
| --- | --- | ---: | --- |
| [Staff behavior](staff_behavior.json) | `abm_staff_behavior` | 9 | Staff attributes, assignment, eligibility, task choice, reassessment, priority, handover, interruption and switching |
| [Spatial ABM](spatial.json) | `spatial_abm` | 9 | Layout graph, scale/distance, locations, connectivity, movement, trips, route choice, travel interruption and deferred congestion |
| [Optional complexity](optional_complexity.json) | `optional_complexity` | 4 | Fatigue, learning, crowding feedback and detailed physiology, all deferred |

Active candidates have unknown values and all five range types (physical,
observed, generic scenario, uncertainty and calibration) explicitly unknown.
Congestion and optional-complexity records are deferred as specified by the
usage matrix. No observed summary, location displacement, interruption count or
synthetic fixture value is an ED default. Candidate families remain unselected;
dependence and sampling order remain P3 decisions.

Evidence links carry SHA-256 and must directly name the parameter ID. Missing
evidence is represented with an acquisition plan stating required evidence and
impact. Source populations and limitations stay in the linked P2 records.
Macro DES elapsed/wait/work measures and Micro movement/interruptions require a
separate P3 reconciliation; do not add an aggregate elapsed measure to its
disaggregated intervals.

The catalogue and synthetic accounting contracts are in
[p23-contract.md](p23-contract.md). The hand-worked graph fixture is
[p23-synthetic-accounting.json](p23-synthetic-accounting.json); every number in
it is invented solely to exercise units, direction, reachability and interval
accounting. It is not a clinical or operational estimate.

Run from the repository root:

```sh
python3 tools/validate_abm_catalogue.py
python3 tools/validate_abm_example.py
python3 -m unittest discover -s tests -p 'test_abm_*.py' -v
```

Passing these checks establishes catalogue structure and example arithmetic
only. It does not establish local capacity, routes, patient/staff behavior,
clinical validity or Cairns applicability.
