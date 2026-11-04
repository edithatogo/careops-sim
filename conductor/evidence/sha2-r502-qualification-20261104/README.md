# SHA-2 R50.2 dependency, value and build-identity qualification

## Decision and scope

At parent `f1ef09551e9f07806a29fc14a255a6d64af5c122`, retain the existing exact
`sha2 = "=0.11.1"` pin and nominate its existing default features for the bounded
ED helper replacement after R50.3 review. This is a qualification/value decision;
no production helper, parent manifest/lock, submodule pin or runtime dependency
edge changed. R50.3 and every R51/R52 task remain separate and unaccepted.

The [frozen packet](packet.json), [qualification receipt](qualification.json),
[probe](probe.rs), [runner](qualify.py) and [actual build-script oracle](identity.py)
retain inputs, budgets, exact dependency closure, raw paired measurements and
compatibility limits. Private command logs/targets remain with the existing
coordinator in its external evidence root. Paths use portable placeholders to
preserve the existing privacy gate; the actual checkout and lease are retained
in the private run ledger. This is the existing standard Git SHA2 worktree.
The expanded documentation packet is necessary to keep the plan, generated task
catalog, dependency handoff, registry and resume state consistent; no broader
source reservation or runtime dispatch follows from that mechanical work.

## Dependency and security

The live [exact registry record](https://crates.io/api/v1/crates/sha2/0.11.1)
identifies non-yanked0.11.1 as current stable, released12 October2026. The cached
crate archive matches its lock/registry checksum47d7069b…ac9ad19. Its licence is
MIT OR Apache-2.0; retain upstream notices. The declared MSRV is1.88, below the
parent1.99 floor; the actual local compiler is1.99.1. This does not claim a fresh
1.88 run. The [tagged upstream CI](https://github.com/RustCrypto/hashes/blob/sha2-v0.11.1/.github/workflows/sha2.yml)
covers MSRV/stable, Linux, macOS, no_std and software backends; configuration is
not a claim that this chat ran those platforms. The upstream
[security policy](https://github.com/RustCrypto/hashes/blob/sha2-v0.11.1/SECURITY.md)
supports the most recent release and private advisory reporting.

The current minimal normal parent path is ED→core→sha2; alloc/oid defaults are
already enabled. The proposed direct ED normal edge uses `sha2.workspace = true`
and adds no registry package/version to that normal closure. The isolated probe
retains10 exact parent registry packages/checksums. Parent direct-edge compilation
is still an R51 obligation. No zeroize/backend override or unrelated feature is
introduced. Existing correct core/birthing/build uses retain their contracts.

The normalized exact cargo-deny policy and original private policy SHA256 are
retained in qualification.json; only its database path is replaced by a portable
placeholder. Cargo-deny0.20.2 passed advisories, bans, licences and sources against a freshly
fetched private RustSec database965bf9ad083b58e6340d65829d85217b18be9290, with no
ignores/exemptions. Three unused licence-allowance warnings are disclosed in the
receipt; no failure was suppressed. This is the candidate closure audit, not a
fresh full-parent security or release approval. The existing refresh query was
used for sha2 only because its global candidate list omits sha2; unrelated
candidate files and pins were preserved.

## Independent behaviour and costs

Both the extracted current helper and the proposed library wrapper match Python
hashlib on22 exact cases: empty/abc/standard multiblock, padding boundaries,
all256 byte values, a million `a` bytes,1MiB binary data, and full64-bit values.
Six input-bit-flip/truncated/invalid-expected controls reject in the independent
probe comparator. These are not parent digest-parser or recovery acceptance.

Budgets were fixed before execution: seven counterbalanced pairs per workload,
128 warmup hashes/process, at least100ms per measured sample, candidate median
≤1.10×baseline, with an additional≤1µs excess at1KiB,≤256KiB binary growth,
≤16MiB peak RSS growth, fresh-target build≤30s and warm build≤10s. Every budget
passed without performance retries or threshold changes.

| Bytes/hash | Baseline iterations/sample | Candidate/baseline median |
| --- | --- | --- |
| 1,024 | 200,000 for each implementation | 0.918579 |
| 65,536 | 10,000 for each implementation | 0.098036 |
| 1,048,576 | 1,000 for each implementation | 0.094307 |

Baseline/candidate fresh isolated target builds took2.084/6.760s and warm builds
0.071/0.054s. Both used an already populated shared registry cache; these are not
parent or whole-Polars cold builds. Binary sizes455,888/455,840 bytes differ by
−48 bytes. Candidate peak RSS did not increase on any measured workload. Raw
seven-sample arrays, hashes and conditions are in the receipt. The1KiB samples
show substantial host variance; its result is a finite budget pass, not a
portable8% speedup claim. This local macOS
ARM measurement does not establish Linux, x86, full-application or release costs.

The value is removal of handwritten SHA256 padding/compression using an already
normal-transitive maintained primitive: proposed function74→4 lines, no new
registry package, independent parity and acceptable measured cost. The223 lines
of retained qualification support are additional maintenance burden; there is
no overall net-line reduction claim, runtime speedup guarantee or completed
production deletion. The proposed function keeps lowercase64hex and exact bytes.
No observed wrong digest in the existing standard helper is alleged.

## Model-build and historical artifact compatibility

The actual unchanged ED `build.rs` executable agrees with an independent Python
hashlib transcript on eight cases. Relocation, documentation and restored inputs
retain identity; the proposed helper, normal dependency edge, compiler flags and
feature bindings change it. Input paths sort by Rust path components, with exact
length framing, compiler stdout and environment bindings; model identity remains
source-bound rather than a promise of byte-identical build provenance.

Equal standard digests require no invented digest-format version. A source or
manifest change does change `careops.ed.model-build.v1`; execution resume with
another model identity must continue to reject before mutation. Preserve old
checkpoint/artifact tags and their qualified original software for recovery.
The existing `internally_consistent_foreign_model_tag_is_readable_but_cannot_resume_execution`
contract demonstrates readable foreign-tag artifacts and rejected execution;
its unchanged source and earlier full787-test native qualification are retained
as historical evidence, not relabelled as a fresh R50.2 runtime run.

R51 must add independent failing parent contracts before replacement, exercise
actual old-artifact reading and foreign-model rejection with preserved tags,
qualify the successor source-bound manifest guard, and run the affected native
consumer/platform gates. Never overwrite old receipts, rewrite old tags, bypass
resume validation or retain a second production hash implementation as fallback.
Rollback of an eventual failed replacement is the original qualified source and
binary; this evaluation requires no runtime rollback because none changed.

## Preserved failures and reproduction

The first candidate compile failed with101 because0.11.1's digest array lacks
`LowerHex`. The corrected probe uses the repository's existing per-byte lowercase
formatter; no extra encoding dependency or performance threshold changed.
The first independent build transcript failed because Python slash-string sorting
differs from Rust component path sorting for ED/ED-CLI. Corrected component
ordering matches the actual unchanged script. Both original logs remain hashed
in the receipt; neither failure is erased or converted into success.

Use the verified repo-local Python3.15.1, a fresh private output directory and the
source revision/input hashes in packet.json. The runner refuses runtime/input drift and existing output directories; a later
documentation checkout is permitted only after unchanged runtime/manifests/lock
and original-source ancestry checks. Logs contain exact executed
argv, exit codes, duration and stdout/stderr hashes.

```sh
.venv/bin/python conductor/evidence/sha2-r502-qualification-20261104/qualify.py --repo . --out "$PRIVATE/sha2-probe"
.venv/bin/python conductor/evidence/sha2-r502-qualification-20261104/identity.py --repo . --probe "$PRIVATE/sha2-probe"
```

The current source has the same runtime/manifests/lock input bytes as that
checkpoint. A later documentation commit is distinct; do not relabel it as the
tested source. Existing native, phase, security, clinical and release gates stay
in place. No R51 dispatch follows until the existing R50.3 phase decision.
