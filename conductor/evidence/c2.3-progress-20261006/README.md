# C2.3 transit progress cursor — local integration evidence

Parent pin `401564c35994a27b05c9c6c463a65d7a9c4537d9` adds the C2.3 progress-state foundation after route/time pin `0edd2a05cc552c679e33cf8d677054fd91a9c071`. The source commit is `401564c35994a27b05c9c6c463a65d7a9c4537d9`; it changes only `crates/kairo-ecs-abm/src/spatial.rs` and keeps the cursor crate-private.

`TransitProgressState` retains the immutable route, current segment, elapsed ticks, and remaining duration in a cloneable interruption checkpoint. Tests prove same-edge resume, exact elapsed/remaining ticks, zero-duration edge handling, invalid checkpoint rejection, and atomic over-advance rejection. On the exact source commit, Rust 1.76 focused progress tests passed 4/4; all ABM package tests passed (14 unit, 2 behavior, 3 ABM integration, 7 Flow adapter); Rust 1.99 strict all-target Clippy, workspace formatting, and `git diff --check` passed. The copied structured worker receipt is `progress-result.json`.

This is a local progress-state leaf. It does not create Flow transit start/progress events, actor-carrier integration, Pause/Resume controls, actual service acquisition, staff dispatch, or a portable checkpoint codec. C2.3 parent acceptance, C2.2 stream integration, C2.1 paired runtime oracles, hosted exact-head checks, stable API/release, and ED operational acceptance remain open.
