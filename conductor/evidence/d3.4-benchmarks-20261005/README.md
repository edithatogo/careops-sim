# D3.4 native benchmark qualification

Status: local and hosted Linux qualification complete at bb81846; all PR checks
passed in run37245641553. D3.4 is checked as implementation-qualified; final closeout publication remains
subject to final-head CI and merge of PR63. D3.5 phase review is next.

The [frozen contract](../../design/d34-benchmark-contract-20261005.md) defines
synthetic E0 workload, independent row oracle and development budgets. The
[qualification receipt](qualification.json) binds source, toolchain, commands,
exits and limits; [raw local measurements](local-report.json) retain all 26 rows.
491 canonical Python tests after the Linux repair and 80 native workspace tests
passed (the initial qualification passed490 tests). The 26-run matrix peaked
at 0.791 seconds and 14.88 MB RSS, below 10 seconds and 256 MiB per child.
All repeat/20-run soak summaries matched. Slow-consumer, real blocked Rust CLI
termination, zero-duration rejection and harness oversized-input refusal passed.
MacOS has no `/dev/full`; hosted Linux verifies that control in
[the retained report](hosted-linux-report.json). All26 hosted measurements pass.
The first hosted attempt failed due to a missing Path import; its log is retained.
The repair adds a cross-platform sink-wrapper regression. A test-only100ms startup
deadline proved flaky under suite load; 500ms plus explicit SIGKILL verification
passed the canonical suite. Production budgets remain unchanged.

Independent gpt-6-luna review found cleanup/provenance issues, which were fixed;
final review found none in its stated scope. Model selection is not served-model
attestation or general model qualification. Earlier failed per-read consumer
attempts and scheduler scope/command deviations are retained, not pooled with
accepted measurements. A logical64KiB consumer quantum replaces OS-dependent
per-read pacing before final qualification; frozen budgets were not increased.

The source is split into ED oracles and POSIX process handling for bounded
context recovery. Both modules fit the 24,000-byte source cap individually;
use one selected slice with its relevant tests/contract. Actual tests cover
numeric/hash/row/unit drift, bounded draining, timeout escalation and cleanup.

C4.1 pin a2cdeab and checkout6.1.0 were synchronized from accepted main. Core/
types source, existing Q5.2 thresholds, C1.4 evidence and D3.3 ED production
source/mutation binding remain unchanged. No new dependency was introduced.

This accepts an experimental development benchmark, not a clinical model, ED
MVP, stable API or native v1. E3.1/E3.2/D4.1 still own native bounded input/output,
atomic output, cancellation/recovery and long-lived runner qualification. The
existing Track25/D4 release-baseline hold remains explicit. Next after actual
hosted acceptance: D3.5 phase review; parallel C4.2 ownership is preserved.
