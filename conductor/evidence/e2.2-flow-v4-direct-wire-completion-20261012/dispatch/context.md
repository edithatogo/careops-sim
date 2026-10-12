# Direct V4 wire bounded dispatch

Base 02529ec07ef35b09a7eadf8d4a4e3727d6379780

# Flow checkpoint V4 direct wire contract

Status: reviewed native source stage; proposed bounded wire implementation contract, runtime acceptance pending. Root is Track03 scoped owner under full delivery authority; independent Sol review supplied framing/source analysis. Native final result d973b9d50bec8f6177f42abcea89671d20ba33f781caa949270fbdffac4eb6ad:403passed/0failed/1ignored,10nativeproofs. Accepted native source commit a178a46b2399c48b00755c13e8c209ad63dd5685. Durable parent archive conductor/evidence/e2.2-flow-v4-native-completion-20261012 is committed at parent61786408b7c85bc8336a15244eb1b4b56424f820, acceptanceSHA e07bc557dcae5157801e6325e91250befb3c6696b4bc0bd460825fcc20c4c181. Root reviewed this owner contract; wire implementation and runtime acceptance remain pending. Bind the commit containing this contract before source dispatch.

## Public surface and ownership

Add only FlowCheckpointV4::encode_wire_v4(&self,FlowCheckpointWireLimits)->Result<Vec<u8>,FlowCheckpointWireError> and decode_wire_v4(&[u8],FlowCheckpointWireLimits)->Result<Self,FlowCheckpointWireError>. They mirror current hidden experimental V3 direct APIs. No new limits/error type, C ABI/Arrow/root rename, native DTO alteration, Image/Ref variant, automatic version selection, runtime callback export or legacy conversion. Existing callers require no edits. Source ownership exactly crates/kairo-ecs-des/src/flow/checkpoint/wire.rs and new crates/kairo-ecs-des/tests/flow_checkpoint_wire_v4.rs. Inline private scanner allocation instrumentation belongs in owned wire.rs. Existing native helpers remain read-only.

## Bytes and canonical order

Eight-byte magic KFLOWV4 followed by NUL; u16 little-endian wire schema1; u32 native version4. Preserve all V1/V2/V3 framing and bytes unchanged. After header, write the exact existing V3 body field order and representations, from config/callback/batch/budget through scheduler/world, identities/registrations/stores/commands/notifications/pending operations/counters/grant flags/frontier/allocation_modes. Use existing V3 command tags. Append queue_order_modes then request_queue_orders.

Each vector has checked u64 count. Mode row: full ResourceId(u64index,u32generation), u8 mode0AdmissionOrder/1StableKey. Request row: full RequestId, u8 order0AdmissionOrder/1StableKey; tag1 has u64 length plus opaque bytes1..128. Rows strictly ascend by full ID; duplicate key VALUES are valid. Exactly one mode per live resource and one order per retained request; removed-resource terminal groups preserve full generation and consistent mode. No derived sidecars serialized. Native version/structural join validation remains authoritative; direct wire owns no history authenticity.

## Limits, validation and allocation boundary

Use existing FlowCheckpointWireLimits(flow,max_wire_bytes,max_total_records). New mode/request rows count toward existing resource/request bounds and aggregate total records. Stable byte totals share max_key_bytes with serialized registration/component strings using checked arithmetic. max_payload_bytes remains caller codec payload, unchanged. Existing capture and restore key-accounting minima differ (runtime registry versus repeated DTO strings); wire checks its serialized DTO strings, without claiming equal capture/restore minima.

Encoder: check native version; count/key/payload bounds; checked full byte measurement; canonical order/structural coverage validation; one fallible exact output allocation; write. Decoder: original whole-wire cap/header/schema; complete borrowed scanner checks all counts, per-key1..128, aggregate records/keys/payload, primitive tags/bools/options and checked integer conversions, canonical row ordering and trailing bytes; only then owned DTO allocation/decode; validate complete DTO canonical/structural joins. Full native restore then validates runtime cross-joins before caller codecs under accepted native policy. No caller codec is called by direct wire encode/decode. Bounded temporary join maps after count caps are allowed; do not claim all joins before any bounded allocation. Invalid final metadata or trailing data must reject before owned DTO collections.

Error taxonomy follows existing wire: UnsupportedSchema(u16), InvalidTag, InvalidBoolean, InvalidUtf8, IntegerOverflow, Truncated, TrailingBytes, LimitExceeded(label), AllocationFailed, InvalidValue(label). Freeze exact new labels/first-error ordering in source packet before execution; no silent fallback to older format. Bad magic uses the existing V3 InvalidTag convention; wrong native version established Flow checkpoint version error. Structural validation cannot recover fully erased removed-resource history.

## Exact required proof names and oracles

1. v4_wire_actual_mixed_runtime_roundtrip_continues: actual native capture with both resource modes, Pending/Queued/Active/Suspended and duplicate keys; direct encode/decode DTO equality, real restored continuation dispatch/progress/context/fullimage equality.
2. v4_wire_removed_generation_terminal_rows_roundtrip: actual removed resource terminalkeys, later reused slot generation isolated; exactmetadata/accessor roundtrip and disagreement rejection.
3. v4_wire_literal_golden_and_legacy_bytes_unchanged: new literal V4 expected bytes/hash from independently reviewed field inventory; unchanged existing V1/V2/V3 literal sourcegoldens remain passing. Candidate encoder output alone is not expected golden.
4. v4_wire_exact_byte_record_key_limits: exact totalwire/records/resource/request/key cap success and onebelow failure; aggregate strings+128key, payloadcaps unchanged; no guessedmultipliers.
5. v4_wire_checked_lengths_and_integer_overflow: checked measurement/count/key sums and u64→usize boundary controls, no oversized allocation.
6. v4_wire_header_tags_and_truncation_matrix: unknownmagic/schema/nativeversion; every new mode/order tag; zero/129key; bool/option and primitive truncation, each isolated positive baseline.
7. v4_wire_duplicate_missing_unsorted_metadata_rejected: duplicate/unknown/missing mode/request IDs, swapped rows, live mode/tag mismatch and inconsistent removedgroups; duplicate keyvalues accepted.
8. v4_wire_bad_final_row_and_trailing_precede_owned_decode: borrowed scanner proves malformed last row/trailing byte zero owned DTO decode allocations, preserving instrumentation counters and positivevalidbaseline.
9. v4_wire_measurement_precedes_output_allocation: exactlimit/truncated/count/measurement failure zero outputallocation, valid oneallocation; no count-driven copies beforecaps.
10. v4_wire_direct_restore_has_no_legacy_fallback: keyed image decodes V4 and native restores actualeffectiveorder; olddirectcodec rejects V4 magic, V4 limit/error remains visible; no Image/Ref routing added.

## Gate budget and acceptance

Format first. Three Rust/Cargo1.99 gate commands: strict DES all-target Clippy, full DESpackage tests with all ten exactnames and unchanged legacy tests, two-owned-path format. Maximum three failed DEVELOPMENT executions; stop eachfailure for root review, retain all started/result/rawlogs and no counterreset. Freeze sourcehashes/pairedreceipt chronology/currenthead and source result before independent acceptance/commit. Source-stage success is directwire only. Clean joined four-package qualification, automatic routing, version-aware consumers/26processcases/4V3traces/19outergoldens, parentpin and ED remain separate gates.

## Compatibility and red-team response

Additive hidden Rust alpha/experimental method at crates/kairo-ecs-des; no C ABI include/kairo_ecs.h or Arrow schemas/arrow/event_log_v1.schema.json change. Copied DTO/result bytes; no dispatch/replay or scheduler change. ReturnVec allocation is explicit bounded/fallible. Malicious counts/keys reject before ownership; canonicalmetadata preserves authoritative native semantics without exporting EDpolicy. Shared fixturecatalog/beta/stable/release approvals are separate and not claimed.


## Proposed frozen error contract
{
  "bad_magic": "InvalidValue(\"Flow checkpoint magic\")",
  "native_version": "InvalidValue(\"Flow checkpoint version\")",
  "schema": "UnsupportedSchema(value)",
  "mode_or_order_tag": "InvalidTag",
  "empty_key": "InvalidValue(\"stable queue-order key\")",
  "oversize_key": "LimitExceeded(\"queue order key bytes\")",
  "aggregate_keys": "LimitExceeded(\"key bytes\")",
  "noncanonical_metadata": "InvalidValue(\"queue-order metadata canonical order\")",
  "coverage_or_join": "InvalidValue(\"queue-order metadata coverage\")"
}
Validation order: native encode version, count/key/payload limits, measurement, canonicality, outputallocation. Decode wire bytecap, header/schema/nativeversion, borrowed complete scan primitive/count/key/canonical row checks, trailing-byte rejection, then owned decode/bounded joins. Root owner must explicitly accept labels before dispatch.

## AGENTS.md
# Repository agent guide

KairoECS is a Rust-native discrete-event and agent-based simulation library with
multiple language bindings. Keep engine contracts, replay determinism, and
security boundaries explicit in every change.

## Start here

- Read `README.md` and `CONTRIBUTING.md` for project workflow and contribution rules.
- Read `SECURITY.md` before changing CI, dependencies, FFI, publishing, or artifact handling.
- Use `conductor/workflow.md`, `conductor/track-map.md`, and `conductor/tracks.yaml` to find the active owner and acceptance gates.
- Check `conductor/subagents.md` and the selected track's `spec.md`, `plan.md`, and `test-matrix.md` before changing owned paths.

## Change and validation rules

- Keep patches within the selected track's ownership contract; hand off before crossing owners.
- Preserve deterministic scheduling and shared conformance contracts. Public API, ABI, schema, or fixture changes require the review steps in `CONTRIBUTING.md`.
- Run `just ci` as the one-command Rust validation lane. It uses one coverage-instrumented workspace test pass, checks the core coverage floor, and runs formatting, lint, docs, and dependency policy checks.
- Run `just quality-drift` for a fresh read-only GitHub settings/source readback. It writes `artifacts/quality-frontier-drift.json`; `pass`, `drift`, `pending`, and `unavailable` remain distinct, and non-pass results exit nonzero.
- For a narrower change, run the smallest relevant validator and report skipped checks. A local pass is not evidence of a hosted Actions run or release acceptance.
- Record command, working directory, commit, toolchain, deterministic seed/input hash where applicable, exit status, and artifact path for validation claims.
- Keep secrets and private vulnerability details out of source and public CI artifacts. Do not publish packages, create releases, or change protected branch settings as an implicit consequence of a passing test.


## Selected native public API/metadata validation seams
impl FlowRuntime {
    /// Capture a complete native V4 image, including stable queue-order metadata.
    #[doc(hidden)]
    pub fn capture_checkpoint_v4(
        &self,
        codecs: &FlowCheckpointCodecs,
        limits: FlowCheckpointLimits,
    ) -> Result<FlowCheckpointV4, FlowCheckpointError> {
        let metadata = capture_queue_order_metadata(self, limits)?;
        let base_limits = remaining_key_budget(limits, metadata.key_bytes)?;
        let base = self
            .capture_checkpoint_parts_with_queue_order(codecs, base_limits, false, true)?
            .into_v3()
            .0;
        Ok(FlowCheckpointV4::from_v3(
            base,
            metadata.modes,
            metadata.requests,
        ))
    }

    /// Restore a complete native V4 image, including stable queue-order metadata.
    #[doc(hidden)]
    pub fn restore_checkpoint_v4(
        image: FlowCheckpointV4,
        codecs: &FlowCheckpointCodecs,
        limits: FlowCheckpointLimits,
    ) -> Result<Self, FlowCheckpointError> {
        Self::restore_checkpoint_v4_with_rebind(image, codecs, limits).map(|(runtime, _)| runtime)
    }

    /// Restore privately and return the validated native V4 rebind view.
    #[doc(hidden)]
    pub fn restore_checkpoint_v4_with_rebind(
        image: FlowCheckpointV4,
        codecs: &FlowCheckpointCodecs,
        limits: FlowCheckpointLimits,
    ) -> Result<(Self, FlowCheckpointRebindV1), FlowCheckpointError> {
        if image.version != FLOW_CHECKPOINT_VERSION_V4 {
            return Err(FlowCheckpointError::UnsupportedVersion(image.version));
        }
        if image.resources.len() > limits.max_resources
            || image.requests.len() > limits.max_requests
        {
            return Err(FlowCheckpointError::LimitExceeded(
                "queue-order identity rows",
            ));
        }
        if image.queue_order_modes.len() > limits.max_resources {
            return Err(FlowCheckpointError::LimitExceeded("queue order mode rows"));
        }
        let key_bytes =
            preflight_request_key_rows(&image.request_queue_orders, limits.max_requests)?;
        if image
            .builtins
            .requests
            .as_ref()
            .is_some_and(|store| store.rows.len() > limits.max_requests)
        {
            return Err(FlowCheckpointError::LimitExceeded("requests"));
        }
        let base_limits = remaining_key_budget(limits, key_bytes)?;
        let parts = image.into_parts();
        preflight_v3_normalization(&parts.base, base_limits)?;
        let metadata = validate_queue_order_metadata(&parts.base, &parts.modes, &parts.requests)?;
        Self::restore_checkpoint_parts(
            v2::FlowCheckpointParts::from_v3(parts.base),
            codecs,
            base_limits,
            Some(metadata),
        )
    }
}


## Frozen first-error ordering
Encoder first errors: wrong native version; per-vector/aggregate count and key/payload bounds in field order; checked full measurement; canonical ordering then coverage/tag consistency; output allocation. Decoder first errors: max_wire_bytes; truncated/header magic InvalidTag; UnsupportedSchema; native version InvalidValue("Flow checkpoint version"); existing V3-body borrowed scan in byte order; mode count limit then each mode ID/tag/local ordering; request-order count limit then each ID/tag/key length and aggregate key bytes/local ordering; trailing bytes; owned DTO decode; bounded full coverage/resource/request joins. Within a metadata row, primitive/truncation and tag errors precede local canonicality; collection count caps precede row access; live/removed coverage joins occur after complete borrowed scan. Multi-defect tests assert this ordering; isolated valid baseline accompanies each malformed control.
