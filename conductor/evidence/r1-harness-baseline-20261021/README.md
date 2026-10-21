# R1.1–R1.2 — harness audit and regression baseline

Source base: `1fef8d4778cb0e136c1096bdaf04eb2b1a635e8d`, with independently accepted R0 reconciliation on the development branch. Runtime pin remains `44261eb5cf8275f132f63a3396903d116d701dbe`. No shared plan/status, runtime, fixture, lock or gitlink was edited by this work.

## Environment audit and retained drift

The existing parent harness uses CPython3.15.0 and jsonschema4.26.0, resolving jsonschema inside the selected environment. Its rpds-py is **2026.5.1**, while the current lock requires **2026.9.1**. The audit correctly returns exit1; [existing-environment.log](existing-environment.log) preserves the mismatch. This qualifies the scope of earlier local runs without converting them into current-lock runs. No existing environment was upgraded.

A separate temporary environment was created using the existing CPython3.15 interpreter and the repository's exact hash-locked binary wheels. [Installation](locked-install.log) retains actual output; [receipts](receipts.json) bind exact commands, working directories, exits and log hashes. All five current lock versions match, and the jsonschema import resolves within the selected environment. The coordinator independently reviewed and accepted the R1.1 audit and the canonical environment before R1.2 began.

The [read-only audit](audit_environment.py) records interpreter and resolved executable, environment prefixes, actual jsonschema module, package METADATA/RECORD hashes and the requirements-lock hash. Installed metadata hashes are environment evidence, not a fresh claim about the provenance of the older environment's wheel downloads. The new installation uses --require-hashes and --only-binary=:all:.

## Regression and input identity

**709 tests passed in 178.643 seconds** under the fresh exact-lock environment. Context, 225-task DAG, MVP82/258 and reviewed Kairos pin checks all returned exit0. The canonical environment audit remained byte-identical before/after, and the original environment audit remained byte-identical with its retained rpds-py mismatch. All source-input hashes and the base HEAD remained unchanged. No existing/global package changes occurred.

[source-inputs.json](source-inputs.json) freezes the parent commit/tree, both gitlinks and explicit hashes for Python tool/test sources, the dependency lock and direct planning inputs. Committed fixture and test seed definitions are bound by that immutable tree. No new stochastic seed is supplied by this harness invocation. [Receipts](receipts.json) retain every executed command and its result; a missing/nonzero required result blocks acceptance.

## Acceptance boundary

This evidence covers the local Python harness and planning/pin integrity only. Rust ED/Q/C, official Arrow readers, native save/restore/recovery, representative replay baselines and rollback belong to the separately reviewed R1.3 evidence. R1.4 phase review remains required. The original environment's mismatch is retained; its unchanged before/after audit and the new environment's unchanged audit must both be verified. Local tests do not establish a hosted run, release, clinical, GPU or multi-host acceptance. No change to the current pin is authorized by this evidence.
