# Product guidelines

## Audience and use

- Write for ED clinicians, operations leads and analysts comparing operational
  scenarios. Define technical terms when they affect interpretation.
- Present CareOps Sim as a scenario-planning and analysis tool. Do not frame its
  outputs as clinical advice, individual patient predictions or staffing
  directives.
- Keep the generic ED usable without Cairns-specific or private data. Treat a
  Cairns profile as a later, separately evidenced adaptation.

## Results and evidence

- Make scenario inputs, units, assumptions, random seeds, model revision and
  execution backend available with exported results.
- Distinguish synthetic examples, public evidence, user-supplied empirical data
  and locally validated observations. Preserve source and transformation
  provenance where known.
- Report uncertainty and limitations with comparative results. Do not imply
  empirical calibration, clinical validity or operational fitness until the
  relevant evidence and acceptance gates have passed.
- Prefer clear operational measures: waiting time, length of stay, throughput,
  queue/resource use and patient-flow outcomes.

## Scope and interaction

- Prioritize a reproducible, configurable, headless generic ED MVP and exportable
  results before dashboard or visualization work.
- Keep site-specific policies and profiles distinct from reusable simulation
  engine behavior.
- Expose capacity, staffing, acuity, diagnostics and boarding assumptions in
  understandable scenario configuration rather than hiding them in code.
- Later interfaces should make comparisons, provenance and uncertainty easy to
  inspect; do not make visual rendering a prerequisite for simulation use.

## Privacy and governance

- Use synthetic or appropriately open data in checked-in examples and public
  automation. Never commit patient-identifiable or confidential clinical data.
- Treat CAD/floor-plan content and operational capacity details as governed
  inputs; record their source, validation status and permitted use.
