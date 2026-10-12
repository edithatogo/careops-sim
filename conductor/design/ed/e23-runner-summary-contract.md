# E2.3 Headless Runner & Summary Contract

Status: approved-contract
Date: 2026-10-12
Authority: coordinator contract for Generic ED leaves `E2.3.runner`, `E2.3.summary`, and `E2.3.handoff`.

## 1. Scope & Objective

- **`E2.3.runner`**:
  - Deliver config-driven nominal and capacity-constrained scenario execution via `careops-ed run <scenario.json>`.
  - Fresh execution without requiring source code modifications, clinical private data, or graphical UI.
  - Fail closed with clear diagnostics on malformed or unsupported inputs.
- **`E2.3.summary`**:
  - Structured JSON/CSV outputs including manifest, throughput, wait times, and occupancy.
  - Verification that capacity changes between nominal and constrained fixtures produce expected shifts in utilization and wait times.
  - Exact repeatability under identical seeds.
- **`E2.3.handoff`**:
  - Reproductibility recipe linking exact commands, configuration, seed, and input hashes.
