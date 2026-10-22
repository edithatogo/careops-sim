# CareOps jsonschema: new-dependency-qualification

Planned only; [value policy](../../dependency-value-policy.md) applies.

## Inputs and dependencies

Local milestones ['polars_data_processing_20261022:R14']; upstream track 75. Cross-repository receipts remain unresolved.

## Contract and outputs

Qualify Rust jsonschema only where shared JSON Schema conformance catches demonstrated drift or removes duplicated structural checks beyond Serde. CareOps owns domain/profile/protocol schemas; no automatic new Kairos dependency. Reuse existing Python jsonschema as an independent conformance oracle. Bundle approved schemas/refs, fix draft and format-assertion policy, disable unnecessary network/file resolution, bound input and validation work, reject duplicate-key ambiguity before lost information, and preserve exact wide IDs/ticks. Do not silently coerce/default/remove fields. Keep clinical, cross-record and lifecycle semantics in domain code.

## Ownership

CareOps coordinator and current Rust/schema/dashboard/benchmark owner, limited to the named consumer and dependency. Declare exact file packets before dispatch. No libs/kairos writes; shared manifests and schema changes require handoff.

## Acceptance

Shared positive/negative corpus across Rust/Python, schema validation, unsupported draft/ref/format behaviour, numeric limits and resource limits, stable field-path/error-code mapping; prove fewer maintained checks and acceptable cost. If Serde suffices, record no-go and explicitly remove the prerequisite without claiming adoption.

Version/feature/maintenance/security/licence/target and wrapper-inclusive cost qualification required. Existing-tooling reuse and new adoption are distinct; dependencies are not accomplishments.

## Release and stop conditions

No clinical/production/hosted acceptance. No-go/defer is valid; reconcile downstream registry edges explicitly if no adoption is needed. Keep semantic validation and independent tests.
