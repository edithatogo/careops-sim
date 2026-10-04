# D3.3 development quality gates — 5 October 2026

Status: bounded parent implementation and current-source measurements. Hosted
publication is a separate final gate; its exact source/head must pass before
integration is reported. No clinical/MVP or hardened release acceptance follows.

## Scope and independent review

Three bounded gpt-6-luna workers implemented support/schema drift, flaky-test
inventory, and coverage validation; the metrics worker also implemented the
source-bound mutation-report gate. One writer per path, hashed immutable
contracts, private advisory leases and targeted negative tests were used.
Independent review identified and corrected editable-config co-drift, absolute
config symlink parents, extreme numeric inputs and datetime expiry comparison.
Workers did not publish or mark tasks accepted. Receipts preserve failed probes
and corrected passes separately. An accidentally exposed flake-worker lease was
revoked; its replacement and all subsequent leases were private and released.

No Rust production, dependencies or Kairos pins changed. Track49 dependency work
in another checkout was inspected read-only and remains separate. Rust 1.99.0
is canonical; parent default MSRV remains 1.76.0. Parent manifests have no
features: new feature declarations/config co-drift fail until reviewed. Exact
synthetic physical-v2 fixture identity is checked independently of editable
candidate config, without claiming broader schema evolution compatibility.

## Current native measurements

At parent ec8adfaec70b180f51729bdda96a16d915ca38a2 and Kairos
8cd03c8f791ae58b33e5cc61b244071937a839ac, real Rust 1.99.0 Apple ARM execution:

- cargo-llvm-cov 0.9.1: 25 package tests passed, 487/564 lines = 86.3475177%,
  above the 80% floor selected before measurement. Functions 32/45 =71.1111111%
  are separately reported. Branch and MC/DC totals are zero: not measured.
- cargo-mutants 27.1.0: successful unmutated build/test baseline; 115 generated
  mutants, 110 caught, 2 missed, 3 unviable, zero timeouts. Exit2 denotes missed
  mutants and is not silently converted into success. The strict report gate
  requires the exact source/generator catalog and the two reviewed equivalents.
- The two missed `||`→`&&` mutants at 735:60 and 798:43 match the previously
  reviewed private scheduler/count invariants. Production guards remain intact;
  source drift or a new surviving viable mutant blocks the gate.

This is a single complete new experiment, not a combination of historical runs.
Scope is current synthetic E0 careops-ed library; no CLI/core/ABM/workspace
coverage or hardened-v1 mutation claim is made. Local executable hashes are
observations; official-release asset provenance for those installed Apple ARM
executables is not asserted. Official Rust llvm-tools installation succeeded.
CI uses independently verified official Linux asset archive hashes instead.
Raw exports, command/toolchain/input receipts and original logs are retained
with SHA-256s; see this directory's files and qualification.json.

## CI enforcement and upstream reuse

Existing bounded nextest (retry0, flaky-result fail), separate doctest, Rust
1.76 default-feature tests, dated fuzz/Miri and advisory/licence/source checks
are retained. The required context lane checks support and flake inventory on
every selected context run. Native changes additionally generate actual package
coverage and execute the full 115-mutant experiment with two workers and 180s
per-test timeout, then validate the results. The lane retains bounded outputs
and has a 20-minute budget; no hidden test retries are added.

Official cargo-llvm-cov v0.9.1 Linux archive SHA:
`b3f68e625481fed9b16444174f3fa5ebcdbde4a1878803a35eabe2dcefcdc41a`.
Official cargo-mutants v27.1.0 Linux archive SHA:
`dfe6dc37d0342c891d2829b5a695aa57c2d0edecef7e7d0399a30cc6e206411e`.
GitHub release API digests and downloaded archive hashes/member lists were
checked; runtime versions must match. Tools are adopted pins, not a claim that
all dependencies are currently the newest. Track49 owns parallel dependency work.

Accepted Kairos pin8cd03c8 retains independent Arrow IPC/Parquet bidirectional
interop, Rust1.88 none/ipc/parquet/both × Ubuntu/macOS ARM and synthetic physical
schema readback. PR219 exact head and successful native owner run37230295875
were read back live; fixture hashes/source were inspected. Those upstream
checks were not reimplemented or rerun locally as part of this parent change.
PR219 remains stacked/draft/open; no upstream merge is implied.

## Explicit release hold and next task

There is no owner-approved stable API release baseline for the experimental
parent library. `quality_support --release` fails with that exact reason;
development success explicitly says semver comparison was not performed.
A legitimate release baseline and actual semver comparison remain Track25/D4
release prerequisites, not a fabricated comparison against an arbitrary commit.
The D3.3 development quality scope can close after its own checks and exact-head
hosted CI pass; this does not close D3 or D4. Next is D3.4 representative ED/load,
memory/cancellation/soak budgets, then D3.5 independent phase review. Q/C/E feature
work and release/security/clinical qualifications retain their own gates.

## Hosted Cargo identity correction

First hosted run37237100498 measured the same 487/564 coverage and complete
115/110/2/3/0 mutation results, then correctly failed the old command validator:
Cargo subcommand dispatch records its absolute Rustup executable whereas the
local direct cargo-mutants invocation recorded bare `cargo`. The validator now
accepts an explicitly supplied, exact canonical1.99 Cargo identity; CI resolves
it independently with rustup. Missing/mismatched paths, other versions and
arbitrary commands still fail. Source/catalog, counters and equivalence rules
are unchanged. An independent reviewer rechecked actual retained hosted bytes.
The updated targeted suite passed18 tests and full harness passed474 tests in
60.918s; earlier472-test proof remains accurately retained. See
[the correction receipt](hosted-command-correction.json). This failed attempt
is preserved, and a new source commit must pass hosted CI before merge; no retry
or bypass is used to label the failed source successful.
