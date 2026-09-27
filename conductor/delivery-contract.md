# Delivery contract: functional MVP → hardened native v1 → extensions

User direction consolidated 2026-09-27. This contract defines scope and acceptance;
none of the runtime stages below is implemented by writing this document.
Existing upstream compatibility/ownership gates remain in force.

## Stage 1 — Functional headless MVP (E2 closeout)

Use the Rust-native Kairos DES/ABM model, generic public/synthetic inputs and local
CPU execution. Basic usable delivery is a documented CLI/example runner plus
config files and readable output tables; a library-only fixture is insufficient.

Required behavior:

- Configurable arrivals/acuity, triage, assessment, diagnostics, treatment,
  discharge/admission/boarding; explicit supported pathways and exclusions.
- Configurable beds/treatment spaces, open/staffed capacity, staff skills/shifts,
  equipment and named zones; cleaning/reservation and finite admission boundary.
- Priority queues and reviewed staff-task interruption; never infer automatic
  patient eviction or unsupported simultaneous resource claims.
- Minimal agent assignment and optional synthetic route/distance transit using
  shared clock/state; Macro has no hidden transit cost. No CAD/drawing required.
- Validate config, choose seed/horizon, run a scenario and export basic CSV or JSON
  summaries plus a run manifest. Paths/command names freeze in E0; deliver actual
  runnable instructions rather than proposed command placeholders.
- Nominal and constrained-capacity/surge examples, with a controlled comparison
  showing that the selected bed/staff input actually affects a known fixture.
- Report waiting time, length of stay, throughput, occupancy/utilization and
  terminal/unfinished counts, with units and denominators.

Acceptance at E2: clean bootstrap and documented example invocation; repeat seed
reproduces canonical outputs; hand-computable route/queue cases agree; patient and
resource conservation holds; malformed config fails clearly; the two example
scenarios run without private data, renderer, network service or manual code edits.
The MVP may use a simple serial runner before the full E3 interface. It does not
wait for C3–C6 fitting/qualification or E4 packaging and cannot claim those done.

Existing prerequisite closure includes D0–D2, P0–P4, Q0–Q4 and C0–C2. Their work
is limited to the declared native profile. Optional skills/adapters/backends must
not be promoted into prerequisite features. The E0 one-patient skeleton is an
earlier runnable increment, not the full MVP.

## Stage 2 — Hardened native v1 (G1 / E4 closeout)

Retain the MVP scope, with dependable use through a documented Rust API and CLI.
Required core capabilities include the previously requested native resource queue
and calibration contracts. Completion of the broad Kairos ecosystem is not a gate.

| Required increment | Task owners | Evidence |
| --- | --- | --- |
| Validate/run/compare/export with readable errors and stable schemas | E3.1–3 | Actual command tests, exit codes, malformed inputs, two-scenario walkthrough |
| Repeated experiments and uncertainty summaries | C5, E3 | Fixed seeds, 1/2/N worker equivalence, replication counts and incomplete-run flags |
| Real Arrow/Parquet and bounded empirical-data ingestion | C1/C4, E3 | Independent reader/writer checks, units/timezones, provenance, exclusions |
| Bounded calibration, Macro/Micro, shadow and free-running validation | C3–C6 | Identifiable and confounded fixtures, leakage checks, held-out evidence and validity limits |
| Queue/preemption conformance and compatibility | Q5 | Boundary traces, invariants, stale-event/property tests, performance evidence |
| Cancellation, atomic outputs and checkpoint recovery | Q4/C5/E3 | Crash/resume parity, disk-full/partial-output tests and overwrite protection |
| Generic input coverage and usable example profiles | P5, E4 | Catalogue-to-consumer checks and fresh-user successful run/compare/export |
| Supported-platform quality/security/release evidence | D3/D4/E4 | Declared toolchains/platforms, risk-based fuzz/soak, licence/SBOM/provenance, clean consumer install |

A fresh user must be able to install the candidate, inspect/edit a documented
scenario, run and compare two scenarios, export results and explain limitations
without modifying Rust code or receiving undocumented author assistance. Basic
configuration plus useful tables is the v1 interface. Document a troubleshooting
path, support matrix and input/schema migration policy. Freeze thresholds before
acceptance; no empirical accuracy claim follows from synthetic verification.

V1 does not require live EHR integration or Cairns calibration. Unknown real inputs
stay unknown or labelled assumptions. A generic validated fixture is not a clinical
operational endorsement. No automatic publication follows from qualification.

## Stage 3 — Post-v1 extensions

E5: replay-based spatial visualization, PixiJS/dashboard, CAD/capture adapters,
interactive overlays, later WebSocket profile and optional Wasm qualification.
E6–E8: measured Metal, within-run PDES, distributed transports.
D5: optional maintenance/repair automation after E4; essential CI/security is D2–D4.
Cairns-specific adaptation and later clinical domains remain separately evidenced
roadmap work. No detailed later-domain tracks are introduced.

V1 may preserve IDs, units and ordinary state/route telemetry needed for evolution.
It does not build a visualization protocol, replay player, asset server, 3D scene,
generic plugin framework or accelerated backend in anticipation of future use.

## Repository readiness boundary

The local Git repository, pinned initialized submodules, five Conductor tracks,
ownership/test/handoff records, research intake and planning/task harness exist.
The parent Rust application/build bootstrap and hosted CI are not yet implemented;
E0/D1/D2 own these. GitHub creation stays at D2 after its buildable/local-check gate.
Thus repository planning/bootstrap setup is ready to continue, but full development
readiness, the MVP and v1 must still earn their runtime acceptance.
