# CareOps Sim Fresh User Walkthrough & Onboarding Guide

Document Version: 1.0.0  
Target Audience: New Simulation Analysts, Research Engineers, and Downstream Consumers  
Effective Date: 2026-10-16  

---

## 1. Overview & Delivery Contract Qualification

This document satisfies the fresh-user walkthrough qualification requirement of **Delivery Contract Stage 2 (Hardened Native v1 / Milestone G1 Closeout)**:
> *"A fresh user must be able to install the candidate, inspect/edit a documented scenario, run and compare two scenarios, export results and explain limitations without modifying Rust code or receiving undocumented author assistance."*

No modifications to the Rust codebase are necessary to configure, execute, compare, or export simulations.

---

## 2. Installation & Verification

### 2.1 Toolchain Prerequisites

Ensure that the standard Rust toolchain (Rust 1.99.0) is active:

```bash
rustc --version
# Expected: rustc 1.99.0 (...)
```

### 2.2 Building the CLI Binary

Build the hardened native CLI binary in release mode:

```bash
cargo build --locked --release -p careops-ed-cli
```

The resulting executable is located at `target/release/careops-ed`.

---

## 3. Fresh User Journey: Step-by-Step Walkthrough

### 3.1 Step 1: Inspect Available Commands and Syntax

Run the CLI help command:

```bash
./target/release/careops-ed --help
```

Expected output:
```text
Usage: careops-ed <command> [options]

Headless Emergency Department Simulation & Experiment Tool.

Commands:
  validate <scenario.json>
  run <scenario.json> [options]
  batch <scenario.json> [options]
  compare <base_run.json> <candidate_run.json>
  export <run.json> [options]

Exit Codes:
  0  Success
  2  Usage error / invalid arguments
  3  File not found
  4  Malformed scenario configuration
  5  Incompatible scenarios for comparison
  6  Output file already exists (overwrite protection)
  7  Output write failed (I/O or disk full)
  8  Simulation failed (runtime or invariant error)
  9  Interrupted / cancelled execution
```

### 3.2 Step 2: Validate a Documented Scenario

Validate the nominal emergency department MVP scenario:

```bash
./target/release/careops-ed validate crates/careops-ed/examples/nominal_mvp.json
```

Expected output:
```text
valid scenario 'nominal-ed-mvp' (schema v1, 20 ticks tick, 5 physical / 5 open beds, 5 patients)
```

Exit code: `0`.

### 3.3 Step 3: Run Baseline Simulation and Produce Run Document

Execute the nominal scenario and save the output to `nominal_run.json`:

```bash
./target/release/careops-ed run crates/careops-ed/examples/nominal_mvp.json --output nominal_run.json
```

The generated document contains:
- `manifest`: Schema version, input file path, FIPS 180-4 SHA-256 digest, seed, time unit, and horizon.
- `validation_taxonomy`: Explicit software verification scope and synthetic data disclaimer.
- `summary`: Arrivals count, completed patients, mean wait ticks, mean elapsed ticks, and individual patient traces.

### 3.4 Step 4: Edit a Documented Scenario (No Rust Code Changes)

Create an edited scenario reflecting a capacity bottleneck (e.g. reducing acute treatment bays from 2 to 1):

1. Copy the baseline scenario:
   ```bash
   cp crates/careops-ed/examples/nominal_mvp.json constrained_run_scenario.json
   ```
2. Edit `constrained_run_scenario.json` in your text editor:
   - Change `"scenario_id"` to `"constrained-user-scenario"`.
   - In `"resource_buckets"`, find the item with `"resource_id": "res.acute"`.
   - Update `"staffed_count": 1`, `"open_count": 1`, and `"physical_count": 1`.
3. Validate the newly created scenario:
   ```bash
   ./target/release/careops-ed validate constrained_run_scenario.json
   ```

### 3.5 Step 5: Run the Edited Scenario

Execute the edited scenario and save the output:

```bash
./target/release/careops-ed run constrained_run_scenario.json --output constrained_run.json
```

### 3.6 Step 6: Compare Simulation Runs

Compare the baseline run against the constrained run:

```bash
./target/release/careops-ed compare nominal_run.json constrained_run.json
```

Expected output:
```json
{
  "base_scenario_id": "nominal-ed-mvp",
  "candidate_scenario_id": "constrained-user-scenario",
  "time_unit": "tick",
  "metrics": [
    {
      "metric_name": "arrivals",
      "unit": "patient",
      "denominator_description": "total presentation count",
      "base_value": 5.0,
      "candidate_value": 5.0,
      "delta": 0.0,
      "percent_change": 0.0
    },
    {
      "metric_name": "completed",
      "unit": "patient",
      "denominator_description": "arrivals",
      "base_value": 5.0,
      "candidate_value": 5.0,
      "delta": 0.0,
      "percent_change": 0.0
    },
    {
      "metric_name": "mean_wait",
      "unit": "tick",
      "denominator_description": "completed patients with wait intervals",
      "base_value": 0.4,
      "candidate_value": 4.2,
      "delta": 3.8,
      "percent_change": 950.0
    },
    {
      "metric_name": "peak_occupancy",
      "unit": "bay",
      "denominator_description": "peak concurrent bay occupancy",
      "base_value": 2.0,
      "candidate_value": 1.0,
      "delta": -1.0,
      "percent_change": -50.0
    }
  ],
  "diagnostics": [
    "Constrained capacity or demand increased mean wait by 3.80 tick"
  ],
  "validation_taxonomy": {
    "verification_scope": "software_verification",
    "validation_scope": "none",
    "provenance_class": "synthetic",
    "disclaimer": "Software verification of simulation mechanics (queueing, state invariants, determinism) does NOT constitute clinical domain validation for any healthcare site."
  }
}
```

### 3.7 Step 7: Export Run Artifacts

Export the simulation outputs into tabular formats:

```bash
# Export individual patient trajectories to CSV
./target/release/careops-ed export nominal_run.json --format csv --output nominal_patients.csv

# Export Arrow-compatible schema metadata
./target/release/careops-ed export nominal_run.json --format arrow --output nominal_schema.json

# Export Parquet schema DDL
./target/release/careops-ed export nominal_run.json --format parquet --output nominal_schema.sql
```

---

## 4. Troubleshooting & Intentional Input Error Catalog

The system provides deterministic exit codes and readable diagnostic messages when errors occur:

| Scenario / Action | Observed Command & Error Output | Exit Code | Resolution |
|---|---|---|---|
| **Missing Scenario File** | `./target/release/careops-ed run missing.json`<br>`careops-ed: scenario file not found: 'missing.json'` | `3` | Verify the path and filename. |
| **Malformed Schema / JSON** | `./target/release/careops-ed run malformed.json`<br>`careops-ed: invalid scenario: schema_version must be 1` | `4` | Validate scenario syntax with `careops-ed validate`. |
| **Output File Exists** | `./target/release/careops-ed run nominal.json --output nominal_run.json`<br>`careops-ed: [FILE_ALREADY_EXISTS (6)] output file 'nominal_run.json' already exists; use --overwrite to replace it` | `6` | Add `--overwrite` or supply a new output file path. |
| **Incompatible Comparison** | `./target/release/careops-ed compare run_ticks.json run_seconds.json`<br>`careops-ed: incompatible time units: base uses 'tick', candidate uses 'second'` | `5` | Ensure both scenario configurations use the same `time_unit` and `schema_version`. |
| **Missing Command** | `./target/release/careops-ed`<br>`careops-ed: missing command` | `2` | Run `careops-ed --help` to view valid commands. |
