# Exact private V4 fixture codec repair recommendation

Read-only diagnosis; no edits or gates. Runtime marker registration uses `FlowRuntime::register_domain_plan_hook`, whose internal no-op receipt is a private closure. `DomainBindingImpl::matches` compares both planner and receipt function addresses. A separately defined test no-op is incompatible, even with identical behavior.

## Complete minimal change (root approval required)

In owned `crates/careops-ed/src/flow_adapter.rs`, add a named private receipt:

```rust
fn no_command_marker_receipt(
    _context: &mut MarkerContext,
    _receipt: &kairo_ecs_des::FlowBatchReceipt,
) {}
```

Replace the builder's marker registration with:

```rust
flow.register_domain_plan_hook_with_receipt::<MarkerContext>(
    MARKER_CONTEXT,
    MARKER_KIND,
    no_command_marker_plan,
    no_command_marker_receipt,
)?;
```

In owned `crates/careops-ed/src/flow_adapter/transit.rs`, after MarkerContext codec registration, add:

```rust
codecs
    .register_domain_plan_hook_with_receipt::<MarkerContext>(
        MARKER_CONTEXT,
        MARKER_KIND,
        no_command_marker_plan,
        no_command_marker_receipt,
        FlowCallbackCodeV1 {
            stable_id: "careops-ed.marker.plan".into(),
            version: 1,
        },
        FlowCallbackCodeV1 {
            stable_id: "careops-ed.marker.receipt".into(),
            version: 1,
        },
    )
    .unwrap();
```

Child test module reaches both private parent functions through existing imports. This changes only callback identity to a named function, retaining accepted-empty/no-mutation semantics. It is a production registration change and needs explicit root approval; a fixture-only substitute cannot match the original pointer.

## Complete registration inventory

- MarkerContext context codec, version1: existing.
- Marker planned domain for MARKER_KIND, exact planner and receipt above: missing.
- FrozenService context codec, version1: existing.
- TransitContext owned codec, version2, actual runtime identity, graph and limits: existing.
- Transit planned domain for TRANSIT_KIND, public ABM registration helper providing exact `TransitContext::plan` and accepted callback: existing.
- No WorkHandlers/continuations are registered by `create_work`; do not add unused codec handlers. No restart templates exist in this fixture.

The FrozenService decoder currently constructs a synthetic placeholder and is not an authentic roundtrip codec. That is acceptable only for the current capture-before/after image comparison; do not claim restored service parity or reuse it for persistence acceptance. The production source has no checkpoint API addition.

Preserve all prior failures. After the approved change, freeze both owned files, review the exact diff, then restart the authorized gate sequence only under root's remaining execution budget or explicit owner escalation. Existing foundation tests must still prove marker Accepted-empty behavior. No engine source path or public API expansion is needed.
