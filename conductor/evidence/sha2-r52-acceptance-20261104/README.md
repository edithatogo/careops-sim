# SHA2 R52 acceptance

Status: pending hosted acceptance, final cost/security audit and review.
[Original default-lane failure](default-first-failure.json) preserves the exact
9c016ba linker SIGBUS failure before tests ran. Resource pressure is a hypothesis,
not an established cause. The bounded recipe serializes Cargo, disables only
incremental/debug-symbol payload and records before/after disk/memory. The
original default-feature workspace command, feature closure, assertions, limits,
zero retries and required aggregate remain unchanged. Telemetry retains the
original test exit. Only this PR's configured CI concurrency may cancel its
own obsolete run; interrupted jobs are not accepted results. Fresh exact-head
hosted qualification remains required.
