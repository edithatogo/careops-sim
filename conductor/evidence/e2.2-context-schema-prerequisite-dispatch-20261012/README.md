# Context schema prerequisite dispatch and ABM dependency stop

This records preparation and dispatch, not acceptance. ABM schema-2 integration tests exposed DES context registration restricted to schema 1. The ABM candidate is preserved as unqualified commit `38ac7cb4ea53c657f89444c934c6bb0eb66123e8`, with both failed executions and the intervening passing Clippy receipt. A malformed-cause fixture correction is included but not gate-qualified.

The separately reviewed DES prerequisite owns checkpoint.rs and one new test file. It permits nonzero caller context versions while retaining exact manifest matching, zero/duplicate rejection and restart version-1 policy. Five substantive proofs and Rust/Cargo 1.99 gates are required. Source implementation is dispatched; gates await frozen-source review.

After acceptance, the ABM candidate will be rebound to the accepted prerequisite with complete failure history retained. DES Image/Ref routing can then proceed in parallel on a separate checkout. Generic Handoff child-process proofs remain required before the parent pin; production ED provenance and 88 fixture bindings remain later work.
