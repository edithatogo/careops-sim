# Specification: development readiness and hardened delivery

Status: in progress (local audit/context bootstrap only). This extends the current
engine planning; it does not implement the simulation or later clinical domains.

## Objective

Make one-maintainer development reproducible, observable and fast, with current
verified dependency candidates, bounded agent workflows, CI, staged security and
real completion evidence. Close gaps documented in [module readiness](../../module-readiness.md).

Inputs: existing Q/C specs, upstream contracts/registry/source, host/tool inventory,
live registry versions, user direction and generic ED release scope.
Outputs: tested bootstrap/tool locks; context/evidence harness; evaluated skills;
scoped CI; versioned dependency policy; release gates and verification receipts.

## Requirements

- D-R1 A fresh supported machine can run the minimal native-ED checks without all
  polyglot runtimes. Exact toolchain/dependency/action pins and compatibility floors
  are explicit; current releases have a tested adoption or documented deferral.
- D-R2 Context recovery finds the active task, source pins, owner paths and evidence;
  drift/cycles/missing artifacts fail visibly. No check can silently mark runtime
  acceptance from documentation or tool presence.
- D-R3 Agent roles reuse upstream ownership. Source/evaluate/develop missing skills
  with pinned provenance and held-out evals; no blanket tool authority or automatic
  self-approval. See [agent engineering](../../agent-engineering.md).
- D-R4 GitHub creation occurs at the defined D2 value gate; actual checks/settings
  are read back. One-maintainer review remains operable without impossible human
  approval rules.
- D-R5 Unit/integration/determinism/feature/API/MSRV tests, fuzzing, benchmarks and
  release checks are staged by risk and cost. Required failures cannot be masked.
- D-R6 Early secret/dependency controls precede deeper threat review and release
  provenance; private data and network profiles have explicit entry gates.

Owned parent paths: AGENTS.md, tools/, tests/, relevant Conductor docs and future
parent CI. Upstream owners 13/20/27/30/44 retain their paths; modify through scoped
handoffs. Core/binding/backend algorithm rewrites are blocked in this track.
Parallel-safe: contracts/data-source review and Q/C design; avoid shared manifest/
CI writers. Dependencies and details are in [plan](plan.md).

Acceptance: D-R1–6 verified by clean bootstrap, harness/skill negative tests,
current locked candidate integration, real CI required-check behavior and clean
consumer release rehearsal. Audit snapshot/local tests alone complete only the
bootstrap slice. Release follows upstream governance and the selected ED profile.

## Reports 9–12 integration

The [integration decisions and candidate oracles](../../research/ed-research-incorporation-9-12-20260927.md)
apply to the tasks in this track. Proposed policies/versions remain review inputs;
no reported research check substitutes for locally executed acceptance.
