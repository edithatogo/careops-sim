# CareOps ajv: new-dependency-qualification

Planned only; [value policy](../../dependency-value-policy.md) applies.

## Inputs and dependencies

Local milestones ['jsonschema_validation_20261022:R64']; upstream track none; schema contract is parent-owned. Cross-repository receipts remain unresolved.

## Contract and outputs

Validate unknown incoming dashboard messages and user inputs against accepted versioned schemas before history/state/callback mutation. Replace JSON.parse casts as validation assumptions; keep authoritative runtime/clinical logic in Rust. Prefer build-time standalone validators via Ajv library API; no ajv-cli or extra plugin by default. Use the right draft entrypoint, bounded inputs and strict non-mutating settings. Test JavaScript precision limits: wide integer strings must remain strings. Align UI/native payloads explicitly rather than assuming similarly named configuration models are identical.

## Ownership

CareOps coordinator and current Rust/schema/dashboard/benchmark owner, limited to the named consumer and dependency. Declare exact file packets before dispatch. No libs/kairos writes; shared manifests and schema changes require handoff.

## Acceptance

Browser tests reject malformed/missing/extra/wrong-type/invalid-version messages before state changes, exact wide IDs/ticks, shared Rust/Python corpus parity, trusted-schema build generation, CSP/bundle/performance, accessible error reporting and valid replay. No custom clinical validators deleted just because structural schemas pass.

Version/feature/maintenance/security/licence/target and wrapper-inclusive cost qualification required. Existing-tooling reuse and new adoption are distinct; dependencies are not accomplishments.

## Release and stop conditions

No clinical/production/hosted acceptance. No-go/defer is valid; reconcile downstream registry edges explicitly if no adoption is needed. Keep semantic validation and independent tests.
