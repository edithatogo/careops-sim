# Flow checkpoint V4 image/ref routing

Status: root-reviewed owner contract; implementation and runtime acceptance pending. Direct wire accepted at de8afe50322cd708e868543758231359b0926cd3, result SHA f6117b533876fc2ec7f0532c743d8bde17cd370a0a8903a58a0db411130a1190: 417 passed, 0 failed, 1 ignored, all ten direct wire proofs. Durable parent archive conductor/evidence/e2.2-flow-v4-direct-wire-completion-20261012 is committed at e84770f4f3d29f7e35b143658012af8ed94d7b40; acceptance SHA af5b23ced99420b4309a7aee632bf961edff86525eadf84f63ae2969536c5264. Root and independent GPT-6.1-sol reviewed actual API lifetimes, exhaustive matches, exact legacy selection and proof scope. Bind the commit containing this contract before source dispatch; this contract does not itself dispatch a worker.

## Scope and signatures

Exactly four source paths: crates/kairo-ecs-des/src/flow/checkpoint/v2.rs; crates/kairo-ecs-des/src/flow/checkpoint.rs; crates/kairo-ecs-des/src/flow/checkpoint/wire.rs; new crates/kairo-ecs-des/tests/flow_checkpoint_routing_v4.rs. Existing exhaustive production Image/Ref matches are confined these files; other consumers use generic accessors/let-else. Existing V3 routing test match includes wildcard.

Add enum variants FlowCheckpointImage::V4(FlowCheckpointV4) and FlowCheckpointRef::V4(&FlowCheckpointV4). Keep signatures unchanged:
- Image::as_ref(&self)->FlowCheckpointRef<'_>;
- Image::restore_with_rebind(self,&FlowCheckpointCodecs,FlowCheckpointLimits)->Result<(FlowRuntime,FlowCheckpointRebindV1),FlowCheckpointError>;
- FlowCheckpointRef<'a>::scheduler(self)->&'a SchedulerCheckpointV1;
- FlowCheckpointRef<'a>::work_registrations(self)->&'a [FlowWorkRegistrationV1];
- FlowCheckpointRef<'a>::context_stores(self)->&'a [FlowContextStoreV1];
- FlowCheckpointRef<'a>::commands(self)->impl Iterator<Item=(EventId,FlowCommandV3)> + 'a;
- FlowRuntime::capture_checkpoint_image(&self,&FlowCheckpointCodecs,FlowCheckpointLimits)->Result<FlowCheckpointImage,FlowCheckpointError>;
- Image::encode_wire(&self,FlowCheckpointWireLimits)->Result<Vec<u8>,FlowCheckpointWireError>;
- Image::decode_wire(&[u8],FlowCheckpointWireLimits)->Result<Self,FlowCheckpointWireError>.

V4 Ref borrows existing fields and commands reuse V3 command iterator; no command version4, copied temporary DTO or downconversion. Image restore delegates accepted native restore_checkpoint_v4_with_rebind. Wire routing delegates accepted encode_wire_v4/decode_wire_v4 on KFLOWV4 NUL. Direct old formats remain unchanged/failclosed.

## Deterministic source selection

Select representability from authoritative source inventory BEFORE capture: any live StableKey resource mode OR any retained request key requiresV4. Empty StableKey resource stillV4. Terminal key after full-resource removal stillV4; reused slot generation cannot downgrade old keys. Pending/Queued/Active/Suspended keyed requests stayV4. Missing/corrupt live mode/cache/key/unknown keys cannot be hidden by fallback: chosen strict capture validates source metadata/parity and fails. Invalid retainedkey belonging unknownrequest also triggersV4 validation. Use native accepted map inventory, not catch errors to infer version.

Only automatic capture entrypoint bypasses its old unconditional legacy ordering guard when it selectsV4, by directly calling accepted native capture_checkpoint_v4. Legacy direct V1/V2/V3 APIs retain original guard. All AdmissionOrder/no-key source states preserve existing minimal selection: Deferred or pending/allocation Flush command requiresV3; otherwise convert the captured V3 base to V2 using existing validation, then chooseV1 exactly when all resource_grant_flags are true AND commands contain no GrantEnabled; chooseV2 otherwise. Existing V2-to-V1 conversion validation remains unchanged. A started frontier alone is not a source selection predicate. Existing source-inventory validation and error policy unchanged. No failedV4capture retriesV3/V2/V1; no failedV3capture downgrades.

## Exact required proof names

1. routing_v4_empty_stable_resource_selects_before_capture: actual emptyStableKeyresource returnsV4, directcapture equality, controls validbaseline.
2. routing_v4_all_keyed_lifetimes_and_removed_generation: actual Pending/Queued/Active/Suspended/terminalremoved keys all chooseV4; slotreuse separate generations; all metadata preserved.
3. routing_v4_ref_borrowed_accessors_and_commands_complete: actual imageRef scheduler/registrations/contextstores and commands exact field/iterator equality for all V3commandtags where actual acceptedfixtures support them; no missingFlush/GrantEnabled/DomainControl/Completion.
4. routing_v4_image_direct_native_restore_suffix_equal: real native keyed runtime, direct vsImage restored event/state/progress/context/fullimage suffix equality; actual rebind identity scoped to returned runtime.
5. routing_v4_image_wire_magic_roundtrip: actualImage encode/decodeV4 bytes equal directacceptedwire and actualrestored suffix; oldV1/V2/V3magics still route correctly; unknown/truncated/errorpropagation exact.
6. routing_v4_corrupt_source_and_limits_never_fallback: independently mutated cache/mode/key source or ownedfixture controls, chosenV4 fails before caller codecs where native authority specifies; exactlimit/excess failures remainV4 error, no oldcodec invocation or version fallback.
7. routing_v4_legacy_minimal_selection_unchanged: actual AdmissionOrder/no-key Eager/disabled/Deferred/currenttick/halted/pendingFlush examples route sameV1/V2/V3 as old source and exactdirectcapture; existinglegacytests and literalbytes unchanged.
8. routing_v4_old_direct_capture_remains_failclosed: V1/V2/V3 direct captures reject liveStableKey and retained terminalkeys before codecs; automaticV4capture succeeds positivecontrol without weakeningoldguards.

Malformed V4 wire/schema/order errors belong accepted directwire proofs; routing preserves them exactly, no new framing or alterederrorprecedence. In-source unit controls may access private metadata for corruption only inside ownedcheckpoint.rs; no fixture edits outside scope without reviewed newpacket.

## Commands and acceptance

Format first. Three pinned Rust/Cargo1.99 gates: strict DES all-target Clippy, full DESpackage tests with all8exactnames/oldtests, four-sourceformat. Maximum3failed DEVELOPMENT executions; everyfailure root-review STOP, retainimmutable started/result/rawlogs. Fresh root authority binds packet/driver/worktree/base and exact complete failedreceiptlist for reviewed correction. Ready only latest ordered terminal gates exit0 at identicalcurrent4hashes with actualrawnames/counts. Frozen independent source review before commit.

Routing acceptance is additive hidden Rust alpha/experimental enum/API support, no C ABI include/kairo_ecs.h or Arrow schemas/arrow/event_log_v1.schema.json change, no scheduler/dispatch/replay. Adding enumvariant is an exhaustive-match source compatibility change; bounded inventory found required production matches in scope, future callers must account for newvariant. No release reviewer/beta/stable approval. Downstream preview/coherent/portable consumers wait acceptedcommonDESrouting+ABMwire join for fullnewversionqualification; inherited26childcases/4V3traces/19literaloutergoldens remain and newHandoff childproofs precede parentpin. ParentEDadapter/88fixtures/clinicalrelease separate.


## Bound routing proposal (serial prerequisite superseded by accepted evidence)

## Proposed bounded source ownership

Exactly four Rust paths after rebind:

1. `crates/kairo-ecs-des/src/flow/checkpoint/v2.rs` — add V4 Image/Ref variants, borrowed accessors and complete command iterator match.
2. `crates/kairo-ecs-des/src/flow/checkpoint.rs` — automatic version selection and V4 image restore delegation.
3. `crates/kairo-ecs-des/src/flow/checkpoint/wire.rs` — V4 image encode/decode routing only.
4. `crates/kairo-ecs-des/tests/flow_checkpoint_routing_v4.rs` — eight required integration proofs.

Observed pre-prerequisite hashes (not dispatch bindings):

- `v2.rs`: `f1527c04bcd278027324347d7dd52c3e3eed4773457ffca4de5a2cf21fb87232`
- `checkpoint.rs`: `4a99028ac60255061cf8115133528f16bc0e521ac05e51f757d8f92da2bd4a74`
- `wire.rs`: `e3ba638f1ae2e7c8f21d93a64409e676f83a120daf4d45123427b6b5a9fc2abb`
- V4 routing test: absent at observed HEAD.

No other source, fixture, manifest, pin, or parent path is proposed.

## Implementation seams

In `checkpoint/v2.rs:122-211`, extend `FlowCheckpointImage` and `FlowCheckpointRef`; keep all existing accessor signatures. V4 Ref must borrow the original V4 fields. Its `commands()` can use the existing `FlowCommandIter::V3` over V4's `(EventId, FlowCommandV3)` slice; do not project or clone into V3. Add the V4 branches to `Image::as_ref` and `restore_with_rebind`, delegating restore to `FlowRuntime::restore_checkpoint_v4_with_rebind`.

In `checkpoint.rs` near `FlowRuntime::capture_checkpoint_image` (around line 1361), inspect authoritative resource-mode and retained-request-key inventory before `ensure_queue_ordering_checkpoint_supported`. Any live StableKey resource or retained request key selects direct `capture_checkpoint_v4`; do not catch a V4 error and downgrade. This includes an empty StableKey resource and retained terminal keys after removal/reuse. Strict native capture validates incomplete/corrupt sidecars and reports their error. For no-key AdmissionOrder states preserve current selector exactly: Deferred mode or pending Flush => V3; otherwise capture the existing V3 base and retain current V1/V2 conversion and grant-flag/GrantEnabled conditions. A started frontier alone must not select a newer image. Keep direct V1/V2/V3 capture guards unchanged.

In `checkpoint/wire.rs:1130-1163`, add image V4 encoding/decoding by delegating to accepted direct V4 wire APIs and `KFLOWV4\0`. Leave V1/V2/V3 routing, bytes, and failure behavior unchanged; no fallback between magics.

The accepted native V4 API/DTO in `checkpoint/v4.rs` and direct wire implementation are protected inputs. Existing V3 routing tests are also protected; use them as fixture examples, not as edit targets.

## Required eight proof names and substance

1. `routing_v4_empty_stable_resource_selects_before_capture`: create a real empty StableKey resource; automatic image selection is V4 and equals direct V4 capture; include an AdmissionOrder control.
2. `routing_v4_all_keyed_lifetimes_and_removed_generation`: cover Pending, Queued, Active, Suspended, terminal retained key, resource removal, and reused slot with distinct generation; verify V4 selection and retained metadata.
3. `routing_v4_ref_borrowed_accessors_and_commands_complete`: compare scheduler, work registrations, context stores, and complete V3-command iterator to actual V4 fields, without projection or mutation.
4. `routing_v4_image_direct_native_restore_suffix_equal`: restore the same keyed runtime through image and direct V4 APIs; compare event/state/progress/context and complete subsequent image suffixes.
5. `routing_v4_image_wire_magic_roundtrip`: check V4 magic, equality with direct accepted wire bytes and restored suffix; exercise legacy magics and unknown/truncated/error propagation.
6. `routing_v4_corrupt_source_and_limits_never_fallback`: independently corrupt mode/cache/key metadata and test exact/excess limits; assert V4 errors and no legacy codec invocation where native preflight requires it.
7. `routing_v4_legacy_minimal_selection_unchanged`: exercise Eager/disabled/Deferred, pending/current-tick Flush, and halted states; compare selected V1/V2/V3 and direct legacy image, retaining legacy byte parity.
8. `routing_v4_old_direct_capture_remains_failclosed`: V1/V2/V3 direct captures still reject live StableKey and retained removed keys before codecs; automatic V4 capture is the positive control.

## Qualification proposal

Pin Rust/Cargo 1.99.0. Format the four owned Rust paths first, then run the immutable reviewed driver in this order: strict `cargo clippy --locked -p kairo-ecs-des --all-targets -- -D warnings`; full `cargo test --locked -p kairo-ecs-des -- --test-threads=1`; rustfmt check for exactly the four owned paths with `skip_children=true`. Preserve every attempt; maximum three failed development executions. Require all eight named proofs to execute nonzero and pass, and all final receipts to bind one unchanged four-source hash set. Root/Sol source review and clean-commit qualification remain separate.



## Accepted-base source excerpts

### crates/kairo-ecs-des/src/flow/checkpoint/v2.rs:122-211
122: pub enum FlowCheckpointImage {
123:     V1(FlowCheckpointV1),
124:     V2(FlowCheckpointV2),
125:     V3(FlowCheckpointV3),
126: }
127: 
128: #[doc(hidden)]
129: #[derive(Clone, Copy, Debug, Eq, PartialEq)]
130: pub enum FlowCheckpointRef<'a> {
131:     V1(&'a FlowCheckpointV1),
132:     V2(&'a FlowCheckpointV2),
133:     V3(&'a FlowCheckpointV3),
134: }
135: 
136: enum FlowCommandIter<'a> {
137:     V1(std::slice::Iter<'a, (EventId, FlowCommandV1)>),
138:     V2(std::slice::Iter<'a, (EventId, FlowCommandV2)>),
139:     V3(std::slice::Iter<'a, (EventId, FlowCommandV3)>),
140: }
141: 
142: impl Iterator for FlowCommandIter<'_> {
143:     type Item = (EventId, FlowCommandV3);
144: 
145:     fn next(&mut self) -> Option<Self::Item> {
146:         match self {
147:             Self::V1(iter) => iter.next().map(|(id, command)| (*id, (*command).into())),
148:             Self::V2(iter) => iter.next().map(|(id, command)| (*id, (*command).into())),
149:             Self::V3(iter) => iter.next().copied(),
150:         }
151:     }
152: }
153: 
154: impl FlowCheckpointImage {
155:     pub fn as_ref(&self) -> FlowCheckpointRef<'_> {
156:         match self {
157:             Self::V1(image) => FlowCheckpointRef::V1(image),
158:             Self::V2(image) => FlowCheckpointRef::V2(image),
159:             Self::V3(image) => FlowCheckpointRef::V3(image),
160:         }
161:     }
162: 
163:     pub fn restore_with_rebind(
164:         self,
165:         codecs: &FlowCheckpointCodecs,
166:         limits: FlowCheckpointLimits,
167:     ) -> Result<(FlowRuntime, FlowCheckpointRebindV1), FlowCheckpointError> {
168:         match self {
169:             Self::V1(image) => FlowRuntime::restore_checkpoint_with_rebind(image, codecs, limits),
170:             Self::V2(image) => {
171:                 FlowRuntime::restore_checkpoint_v2_with_rebind(image, codecs, limits)
172:             }
173:             Self::V3(image) => {
174:                 FlowRuntime::restore_checkpoint_v3_with_rebind(image, codecs, limits)
175:             }
176:         }
177:     }
178: }
179: 
180: impl<'a> FlowCheckpointRef<'a> {
181:     pub fn scheduler(self) -> &'a SchedulerCheckpointV1 {
182:         match self {
183:             Self::V1(image) => &image.scheduler,
184:             Self::V2(image) => &image.scheduler,
185:             Self::V3(image) => &image.scheduler,
186:         }
187:     }
188: 
189:     pub fn work_registrations(self) -> &'a [FlowWorkRegistrationV1] {
190:         match self {
191:             Self::V1(image) => &image.work_registrations,
192:             Self::V2(image) => &image.work_registrations,
193:             Self::V3(image) => &image.work_registrations,
194:         }
195:     }
196: 
197:     pub fn context_stores(self) -> &'a [FlowContextStoreV1] {
198:         match self {
199:             Self::V1(image) => &image.context_stores,
200:             Self::V2(image) => &image.context_stores,
201:             Self::V3(image) => &image.context_stores,
202:         }
203:     }
204: 
205:     pub fn commands(self) -> impl Iterator<Item = (EventId, FlowCommandV3)> + 'a {
206:         match self {
207:             Self::V1(image) => FlowCommandIter::V1(image.commands.iter()),
208:             Self::V2(image) => FlowCommandIter::V2(image.commands.iter()),
209:             Self::V3(image) => FlowCommandIter::V3(image.commands.iter()),
210:         }
211:     }

### crates/kairo-ecs-des/src/flow/checkpoint.rs:1360-1405
1360:     #[doc(hidden)]
1361:     pub fn capture_checkpoint_image(
1362:         &self,
1363:         codecs: &FlowCheckpointCodecs,
1364:         limits: FlowCheckpointLimits,
1365:     ) -> Result<FlowCheckpointImage, FlowCheckpointError> {
1366:         self.ensure_queue_ordering_checkpoint_supported()?;
1367:         // Select validation policy from the source inventory before capturing.
1368:         // Corrupt Deferred entries/cache/commands still select strict validation;
1369:         // no failed capture is retried or downgraded through a legacy image.
1370:         let requires_v3 = self
1371:             .allocation_modes
1372:             .values()
1373:             .any(|mode| *mode == FlowAllocationMode::Deferred)
1374:             || !self.pending_allocation_flushes.is_empty()
1375:             || self
1376:                 .commands
1377:                 .values()
1378:                 .any(|command| matches!(command, Command::AllocationFlush(..)));
1379:         let image = self
1380:             .capture_checkpoint_parts(codecs, limits, !requires_v3)?
1381:             .into_v3()
1382:             .0;
1383:         if requires_v3 {
1384:             return Ok(FlowCheckpointImage::V3(image));
1385:         }
1386:         let image = image.into_v2()?;
1387:         if image
1388:             .resource_grant_flags
1389:             .iter()
1390:             .all(|(_, enabled)| *enabled)
1391:             && image
1392:                 .commands
1393:                 .iter()
1394:                 .all(|(_, command)| !matches!(command, FlowCommandV2::GrantEnabled(..)))
1395:         {
1396:             Ok(FlowCheckpointImage::V1(image.into_v1()?))
1397:         } else {
1398:             Ok(FlowCheckpointImage::V2(image))
1399:         }
1400:     }
1401: 
1402:     fn validate_grant_checkpoint_cache(
1403:         &self,
1404:         limits: FlowCheckpointLimits,
1405:         strict_live_commands: bool,

### crates/kairo-ecs-des/src/flow/checkpoint/wire.rs:1130-1163
1130: impl FlowCheckpointImage {
1131:     /// Encode while preserving the captured native format and all its fields.
1132:     #[doc(hidden)]
1133:     pub fn encode_wire(
1134:         &self,
1135:         limits: FlowCheckpointWireLimits,
1136:     ) -> Result<Vec<u8>, FlowCheckpointWireError> {
1137:         match self {
1138:             Self::V1(image) => image.encode_wire_v1(limits),
1139:             Self::V2(image) => image.encode_wire_v2(limits),
1140:             Self::V3(image) => image.encode_wire_v3(limits),
1141:         }
1142:     }
1143: 
1144:     /// Dispatch only by the three known wire magics.
1145:     #[doc(hidden)]
1146:     pub fn decode_wire(
1147:         bytes: &[u8],
1148:         limits: FlowCheckpointWireLimits,
1149:     ) -> Result<Self, FlowCheckpointWireError> {
1150:         match bytes.get(..8) {
1151:             Some(magic) if magic == MAGIC => {
1152:                 FlowCheckpointV1::decode_wire_v1(bytes, limits).map(Self::V1)
1153:             }
1154:             Some(magic) if magic == MAGIC_V2 => {
1155:                 FlowCheckpointV2::decode_wire_v2(bytes, limits).map(Self::V2)
1156:             }
1157:             Some(magic) if magic == MAGIC_V3 => {
1158:                 FlowCheckpointV3::decode_wire_v3(bytes, limits).map(Self::V3)
1159:             }
1160:             Some(_) => Err(FlowCheckpointWireError::InvalidTag),
1161:             None => Err(FlowCheckpointWireError::Truncated),
1162:         }
1163:     }

### crates/kairo-ecs-des/src/flow/checkpoint/v4.rs:1-55
1: use super::*;
2: use std::collections::{BTreeMap, BTreeSet};
3: 
4: /// Native Flow checkpoint version 4.
5: #[doc(hidden)]
6: pub const FLOW_CHECKPOINT_VERSION_V4: u32 = 4;
7: 
8: /// Queue-order tag for every retained request in a V4 image.
9: #[doc(hidden)]
10: #[derive(Clone, Debug, Eq, PartialEq)]
11: pub enum FlowRequestQueueOrderV4 {
12:     AdmissionOrder,
13:     StableKey(Vec<u8>),
14: }
15: 
16: type QueueOrderModeRows = Vec<(ResourceId, FlowQueueOrdering)>;
17: type RequestQueueOrderRows = Vec<(RequestId, FlowRequestQueueOrderV4)>;
18: 
19: struct CapturedQueueOrderMetadata {
20:     modes: QueueOrderModeRows,
21:     requests: RequestQueueOrderRows,
22:     key_bytes: usize,
23: }
24: 
25: struct FlowCheckpointV4Parts {
26:     base: FlowCheckpointV3,
27:     modes: QueueOrderModeRows,
28:     requests: RequestQueueOrderRows,
29: }
30: 
31: /// Complete lossless native Flow image with explicit stable queue-order metadata.
32: #[doc(hidden)]
33: #[derive(Clone, Debug, Eq, PartialEq)]
34: pub struct FlowCheckpointV4 {
35:     pub version: u32,
36:     pub config: FlowConfig,
37:     pub callback_config: FlowCallbackConfig,
38:     pub next_batch_identity: u64,
39:     pub budget_tick: Option<SimTime>,
40:     pub budget_consumed: u64,
41:     pub budget_halt: Option<FlowBudgetHalt>,
42:     pub scheduler: SchedulerCheckpointV1,
43:     pub world: WorldCheckpointV1,
44:     pub resources: Vec<ResourceId>,
45:     pub requests: Vec<RequestId>,
46:     pub actors: Vec<EntityId>,
47:     pub actor_domains: Vec<(EntityId, EntityId)>,
48:     pub work_registrations: Vec<FlowWorkRegistrationV1>,
49:     pub contexts: Vec<FlowContextRegistrationV1>,
50:     pub handlers: Vec<FlowCallbackRegistrationV1>,
51:     pub continuations: Vec<FlowCallbackRegistrationV1>,
52:     pub domains: Vec<FlowCallbackRegistrationV1>,
53:     pub builtins: FlowBuiltInStoresV1,
54:     pub context_stores: Vec<FlowContextStoreV1>,
55:     pub restart_store_manifest: Vec<(String, u32)>,
