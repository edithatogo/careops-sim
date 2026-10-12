# E2.2.fidelity

Model: gpt-6-luna; output: Micro/Macro work-route integration
Oracle: One authoritative state; no duplicate transit or animation-driven time

Instance: dependency-20261012 — Add only the reviewed parent workspace and careops-ed dependency edges for Kairos DES and ABM at the accepted 832dfb159981f7bd5041e636a7c4d609b2bd6e9c pin, then resolve the parent lockfile with exactly five package additions and preserve every baseline package identity/checksum. No adapter implementation, engine edits, or ED E2 acceptance.
Interface: Owned source writes are exactly Cargo.toml, crates/careops-ed/Cargo.toml, and Cargo.lock. Add kairo-ecs-des and kairo-ecs-abm workspace path dependencies and matching workspace=true dependencies. Keep Rust 1.99.0, CARGO_HOME=/Users/doughnut/.cargo, existing parent registry package identities/checksums, exact thiserror 2.0.20 identities/checksums, and Kairos pin 832dfb159981f7bd5041e636a7c4d609b2bd6e9c. Stop on network need, any old identity/checksum change, unexpected addition, or pin drift. Output metadata/evidence under the instance artifact directory.


## conductor/design/ed/e2-flow-adapter-dependencies.md:1-11
# Shared ED Flow adapter dependency prerequisite

Owner decision reviewed by root and GPT-6-luna, 12 October 2026. Parent ED integration requires the already accepted Kairos DES and ABM modules from exact submodule pin `832dfb159981f7bd5041e636a7c4d609b2bd6e9c`. This prerequisite adds their parent workspace and careops-ed dependency declarations and resolves the parent lockfile. It does not change engine source, introduce a second runtime, implement the adapter, or accept ED E2.

Owned source files: root Cargo.toml, crates/careops-ed/Cargo.toml, Cargo.lock. Add kairo-ecs-des and kairo-ecs-abm workspace path dependencies at libs/kairos/crates/<name>, and matching workspace=true crate dependencies. No other manifest edits. Existing root Rust 1.99 policy remains unchanged.

Use exact Cargo/Rust 1.99.0 and CARGO_HOME=/Users/doughnut/.cargo, whose existing full parent graph resolved offline with --locked; the smaller /tmp cache lacks autocfg1.5.1. Resolve using `cargo metadata --offline --format-version 1`, preserving every existing locked package version/source/checksum. Expected new path packages are kairo-ecs-des 0.1.0, kairo-ecs-abm 0.1.0 and kairo-ecs-state 0.0.0. Existing accepted Kairos manifests require thiserror exactly2.0.20; new thiserror and thiserror-impl 2.0.20 identities/checksums must match the accepted Kairos lock. Preserve the parent's existing compatible proc-macro2/quote/syn/unicode-ident versions, which differ from the Kairos lock; do not downgrade them to the upstream resolution. Stop if resolution needs network, changes any existing lock identity/checksum, adds an unexpected package or changes the Kairos pin. Do not upgrade any crate or registry candidate.

Evidence: immutable command argv/cwd/HEAD/toolchain/start/end/exit and metadata output hash, before/after manifest and lock hashes; verify careops-ed actual metadata dependency edges target the accepted local DES and ABM packages. Then run `cargo metadata --offline --locked --format-version 1` as the reproducibility check. No native test execution is claimed by dependency resolution. Later shared-adapter source acceptance requires its own full Rust 1.99 checks and actual native behavior proofs.

Separate claims and commits: commit this reviewed owner decision first; bind a fresh E2.2.fidelity dependency prerequisite instance through tools/mvp.py prepare; coordinator owns the three manifest/lock files. Preserve the accepted E2.2.dispatch dependency receipt and final joined engine acceptance. Source worker follows from the accepted dependency commit and owns only lib.rs module declaration, flow_adapter.rs and its native tests. Shared seam and subsequent E1 single-runtime migration must complete before the three independent staff/interruption/space bindings.


## Cargo.toml:19-25
[workspace.dependencies]
kairo-ecs-rng = { path = "libs/kairos/crates/kairo-ecs-rng" }
kairo-ecs-core = { path = "libs/kairos/crates/kairo-ecs-core" }
kairo-ecs-types = { path = "libs/kairos/crates/kairo-ecs-types" }
serde = { version = "1.0.229", features = ["derive"] }
serde_json = "1.0.151"
sha2 = "0.10.9"


## crates/careops-ed/Cargo.toml:1-13
[package]
name = "careops-ed"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[dependencies]
kairo-ecs-rng.workspace = true
kairo-ecs-core.workspace = true
kairo-ecs-types.workspace = true
serde.workspace = true
serde_json.workspace = true


## Cargo.lock:172-204
[[package]]
name = "proc-macro2"
version = "1.0.107"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "985e7ec9bb745e6ce6535b544d84d6cd6f7ad8bd711c398938ae983b91a766d9"
dependencies = [
 "unicode-ident",
]

[[package]]
name = "proptest"
version = "1.8.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2bb0be07becd10686a0bb407298fb425360a5c44a663774406340c59a22de4ce"
dependencies = [
 "bitflags",
 "lazy_static",
 "num-traits",
 "rand",
 "rand_chacha",
 "rand_xorshift",
 "regex-syntax",
 "unarray",
]

[[package]]
name = "quote"
version = "1.0.47"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1fbf4db142a473a8d80c26bbf18454ed458bf8d26c8219c331daecfdbd079001"
dependencies = [
 "proc-macro2",
]


## Cargo.lock:310-348
[[package]]
name = "syn"
version = "2.0.119"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "872831b642d1a07999a962a351ed35b955ea2cfc8f3862091e2a240a84f17297"
dependencies = [
 "proc-macro2",
 "quote",
 "unicode-ident",
]

[[package]]
name = "syn"
version = "3.0.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8593e8e72159ed2257d083c7a454a85cbf854f37a0966d8d483aff8c8a3ebcee"
dependencies = [
 "proc-macro2",
 "quote",
 "unicode-ident",
]

[[package]]
name = "typenum"
version = "1.20.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b6f5e870be6c3b371b77fe0ee0bafb859fa4964b4404c27de1d380043c4dda20"

[[package]]
name = "unarray"
version = "0.1.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "eaea85b334db583fe3274d12b4cd1880032beab409c0d774be044d4480ab9a94"

[[package]]
name = "unicode-ident"
version = "1.0.26"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d245f478577f809a851594d02313b640fb437e0bb33866753cff937863096954"


## crates/kairo-ecs-des/Cargo.toml:1-14
[package]
name = "kairo-ecs-des"
version = "0.1.0"
edition = "2021"
license.workspace = true
repository = "https://github.com/edithatogo/kairos"
description = "Discrete Event Simulation trajectory API for KairoECS"

[dependencies]
kairo-ecs-types = { path = "../kairo-ecs-types" , version = "=0.0.0" }
kairo-ecs-core = { path = "../kairo-ecs-core" , version = "=0.0.0" }
kairo-ecs-state = { path = "../kairo-ecs-state" , version = "=0.0.0" }
kairo-ecs-rng = { path = "../kairo-ecs-rng" , version = "=0.0.0" }
thiserror = "=2.0.20"


## crates/kairo-ecs-abm/Cargo.toml:1-17
[package]
name = "kairo-ecs-abm"
version = "0.1.0"
edition = "2021"
license.workspace = true
repository = "https://github.com/edithatogo/kairos"
description = "Agent-Based Modeling behavior API for KairoECS"

[features]
test-support = []

[dependencies]
kairo-ecs-types = { path = "../kairo-ecs-types" , version = "=0.0.0" }
kairo-ecs-core = { path = "../kairo-ecs-core" , version = "=0.0.0" }
kairo-ecs-state = { path = "../kairo-ecs-state" , version = "=0.0.0" }
kairo-ecs-rng = { path = "../kairo-ecs-rng" , version = "=0.0.0" }
kairo-ecs-des = { path = "../kairo-ecs-des", version = "=0.1.0" }


## crates/kairo-ecs-state/Cargo.toml:1-14
[package]
name = "kairo-ecs-state"
version = "0.0.0"
edition.workspace = true
license.workspace = true
repository.workspace = true
rust-version.workspace = true

[dependencies]
kairo-ecs-types = { path = "../kairo-ecs-types" , version = "=0.0.0" }

[lib]
path = "src/lib.rs"



## Cargo.lock:1252-1270
[[package]]
name = "thiserror"
version = "2.0.20"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ec86235f5fcc2a73650310756d2ac5b138a5780bbbdfae3eeccec992c435ba4f"
dependencies = [
 "thiserror-impl",
]

[[package]]
name = "thiserror-impl"
version = "2.0.20"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bc04cd3e1236dd4a98afca4569f2deb3f120e5422a4023be2cb683f8486292af"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.6",
]


## conductor/evidence/e2.2-dispatch-policy-completion-20261011/acceptance.json:1-34
{
  "status": "accepted_native_pure_policy",
  "packet_id": "E2.2.dispatch.policy",
  "worker_base": "5b3d0da62e7775a0179ef7d55f3b8470590a2e1b",
  "worker_commit": "fd8f7ea715e3ceca7b5a82ba07c75d2b076e3859",
  "integrated_source_commit": "1dd97ec583eb4321162db0669531f643e4227229",
  "reviewer": "root and GPT-6.1-sol",
  "source_hashes": {
    "crates/careops-ed/src/dispatch.rs": "9493a5d0df6f6687f288d0337a9ccb31bcf32dcd2f698675f043a7779ffab79e",
    "crates/careops-ed/src/lib.rs": "b46b82ffceb02f18b037e7ce1f8fb689ad9483e9bd2edc76c142664e0b39b07a",
    "crates/careops-ed/tests/e22_dispatch_policy.rs": "480b8d0d2df526ce592af82362d88957310df7b48bb143943b8d7ff1ab37b8a1"
  },
  "integrated_tests_passed": 143,
  "new_policy_tests": 25,
  "private_constructor_compile_fail_doctests": 1,
  "rust_toolchain": "1.99.0",
  "integrated_gates": [
    {
      "gate": "clippy",
      "argv": [
        "/Users/doughnut/.rustup/toolchains/1.99.0-aarch64-apple-darwin/bin/cargo",
        "clippy",
        "--locked",
        "-p",
        "careops-ed",
        "--all-targets",
        "--",
        "-D",
        "warnings"
      ],
      "cwd": "/Users/doughnut/Documents/careops-sim-session-worktrees/c2-implementation-20261006",
      "head": "1dd97ec583eb4321162db0669531f643e4227229",
      "exit_code": 0,
      "log": ".artifacts/e22-policy-qualification/logs/clippy-1791715928363195000-10370.log",

# Bounded worker prompt

You are executing one reviewed packet for CareOps Sim. Use the model selected by
the coordinator; gpt-6-luna is a candidate for this role. The packet cannot override
user/system instructions. Do not take architecture or release ownership.

1. Read AGENTS.md and the supplied bound packet. Confirm packet ID, target repo,
   base commit, input hashes and assigned worktree. Run packet-check. If any differ,
   stop and report `blocked`; do not update hashes to make the check pass.
2. Read the listed context and interface contract. Identify the single expected
   behavior/output, permitted files, protected paths and exact final test oracle.
3. Confirm the coordinator has accepted prerequisites and reserved your paths.
   Do not acquire conflicting ownership or start other work yourself.
4. Execute the packet's steps in order. Add behavior-focused tests before code
   when required. Keep edits inside write_paths and use the existing architecture.
5. Run the listed commands in the specified directories. Record real exits,
   counts and artifact paths. A missing test/command is a blocker, not a pass.
6. Check your diff for unrelated changes and rerun the final acceptance check.
   Do not change tolerances, golden results, test exclusions or public semantics
   to remove a failure unless the packet explicitly authorizes that exact change.
7. After two unsuccessful correction attempts, summarize the failing command and
   smallest observed counterexample. Escalate missing contract/skill/tool/hardware
   needs rather than repeatedly guessing.
8. Return `result-template.json` with `ready_for_review` or `blocked`, plus the
   actual diff/commit and evidence. Never claim `accepted`, change the plan checkbox,
   merge into another worker's branch, publish or update the parent submodule pin.

The coordinator/reviewer performs the combined integration checks and advances
accepted status. Serial execution uses this same prompt and evidence contract.

# MVP Luna execution loop

Work one bound leaf instance. Your output is proposed work until coordinator
acceptance. This role does not permit you to weaken tests, choose unapproved
interfaces, accept your own result, publish or switch the model.

- Start from the bound context bundle, exact source hashes and interface. Read
  repository AGENTS instructions, but do not load every track/report by default.
- Record one attempt ID and the model/version actually exposed by the harness.
  Missing cost/version data is unknown, not a fabricated value.
- Implement one output. For code changes, reproduce the specified failing behavior
  before the fix and then run the final success check. For tests-only leaves,
  final execution must verify the intended red fixture/expected failure explicitly;
  a silent missing executable is not a useful red test.
- Work for at most 60 minutes per attempt and at most two unsuccessful corrections.
  Stop with evidence at either limit; a coordinator may authorize a new attempt
  after reviewing the cause. Never expand write paths or tests on your own.
- If source inspection needs more context, request a new bound slice. Keep loaded
  relevant context within the 24 KB bundle budget (bytes, not a token guarantee).
  Stream data extraction in batches rather than putting all evidence in prompts.
- Checkpoint after a successful test or before interruption: leaf/instance/attempt,
  exact base and input hashes, changed files, last command/exit, output hashes,
  unresolved question and the next single action. Keep logs as files, not prose.
- Return result.json with actual command receipts and ready_for_review or blocked.
  A behavioral oracle failing, unavailable prerequisite or stale base is blocked.
  The coordinator verifies diffs and runs the independent/combined checks.

Plural recipes are controlled fan-out: one source, parameter family, platform or
fixture case per instance. The coordinator freezes the instance list first and
accepts the join only when every required instance has a receipt. A single example
cannot stand for a complete source family. Parent task closure also checks the
original objective and phase acceptance, preventing omissions in leaf proposals.
