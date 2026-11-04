# Tempfile R53.1 prerequisite and consumer inventory

Source `88bdfd67c622d06868331af80dc4db54daa98891`; [structured inventory](inventory.json) binds current files, manifest edges, lock and immutable historical receipts. Scope is R53.1 only. No dependency installation, runtime change or R53 phase acceptance.

## Upstream prerequisite

Kairos PR304 is merged as `5692e6a9ee866131262d60103a84e2544dc51e4a`; Track94 metadata accepts phases0–2. That commit is an ancestor of runtime074 and separately qualified data473. Exact receipt/source hashes and dated hosted check statuses are recorded:35successes and2failures. Both failing setup jobs report missing historical ledger commits for Tracks70/93/94; their logs are hashed in the inventory. The accepted binding is actual merged5692 and its source, not the missing ledger references. No entirely green historical run or repaired upstream closeout is claimed. These are historical prerequisite evidence, not fresh current-version or physical durability certification.

Historical lock and current parent lock resolve tempfile3.27.0 despite the phase0 report naming3.17.1. The current exact graph/features/licence/MSRV/security/value decision belongs to R53.2. Data473 changes protected directory handling and Unix post-persist sync error propagation; immutable historical receipt bytes do not certify changed source.

## Current consumers and value boundary

Current root optional native FFI, ED native-data and birthing test edges already declare exact3.27.0. Existing ED native capture stages beside destination with NamedTempFile, file sync, persist_noclobber and directory sync; preserve it. Current manual scratch wrappers and the core experiment writer remain bounded candidates; their exact paths/lines are inventoried. Historical manual helpers migrated in place are separated from retained candidates. Python standard-library tempfile uses are retained and outside Rust adoption scope.

The core writer uses PID-only staging, does not sync files/directories and can leave staging data after an initial write error. ED checkpoint staging already uses create_new plus an authoritative sync/recovery seam. These are different contracts: qualify useful replacements and independent failures before editing either. Source-bound manifests/build identity and minimal/default graphs remain gates. No new generic staging framework is authorized.

## Receipt limits and review fixes

The upstream staged “crash/abort” test performs normal RAII drop, without a killed process, disk-full or injected sync/write failure. Python tests exercise Python TemporaryDirectory and ordinary writes of Rust smoke bytes, not Rust atomic publication; their second determinism test creates no directories. Synthetic should_panic baselines and unsupported MSRV/platform/allocation/timing claims are not current acceptance evidence. After persist, a directory-sync error may leave a published file; future recovery must not assume Err means no publication.

The unrelated Valhalla/travel-matrix text in the old Tempfile handoff is replaced by this scoped ownership/evidence handoff; original bytes remain in source history. Existing Polars end-to-end durability gates and accepted receipts remain intact. R53.2, R53.3, R54 and R55 remain unchecked. No release/clinical or upstream pin promotion is implied.

Independent reviewer `/root/ffi_packet_prepare` approves R53.1 only. [Focused verification and review](review.json) records63distinct planning tests,9affected handoff tests rerun after review fixes, context/tasks/MVP/diff success and unchanged405task IDs/dependencies. All8later Tempfile tasks remain pending.
