# Stable scenario parser assurance

The 20 November 2026 portfolio rollout executes the existing parser round-trip
oracle on canonical Rust 1.99.1. The original libFuzzer target and the new stable
integration suite include one shared oracle from `fuzz/scenario_oracle.rs`.
Accepted inputs must serialize and parse back to the same `ScenarioConfig`.
Invalid UTF-8 is rejected before the string parser. Each oracle input is limited
to 64 KiB; this is a harness budget, not a new production parser guarantee.

The stable suite admits the checked-in synthetic scenario before mutation, then
exercises every prefix and seven replacement bytes at every position. Structured
cases additionally check full-width seed preservation, Unicode identities,
required-field removal, integer/type boundaries and unsupported fields at four
nesting levels. These are deterministic generative tests, not coverage-guided
fuzzing. No private or clinical data is used.

The path-scoped PR, weekly and manual workflow builds with locked dependencies,
then runs the four tests under a 120-second process budget. Logs, compiler and
commit identity are retained for fourteen days. Failures and timeouts fail the
job. Contents permission is read-only and checkout credentials are not persisted.

```bash
cargo +1.99.1 test --locked -p careops-ed --test scenario_robustness \
  -- --test-threads=1 --nocapture
```

The nightly cargo-fuzz/AddressSanitizer obligation remains unverified, as required
by the current dependency and CI policy. This bounded stable suite does not close
that release gate. Existing dependency-adoption PRs, pins, production contracts,
Conductor task status and core owner work remain unchanged. Candidate validation
and the exact source revision are recorded in the implementation PR.
