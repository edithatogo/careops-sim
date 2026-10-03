# E1 preparation package — 2026-10-03

Status: reviewed preparation proposal, not an accepted runtime contract, executable scenario schema, bound packet or completed E1 task. All numbers below are invented integer-tick test inputs. No public-data estimates, clinical defaults or Cairns assumptions are introduced.

- [Pathway and metric decisions](design.md)
- [Synthetic oracle catalogue](oracles.json)
- [Bounded worker preparation manifest](worker-preparation.json)

Implementation still requires the existing C1.4, C2.4, D2.5, E0.4, P4.4 and Q4.5 gates. These files do not change their acceptance, the existing recipes, current state or source code. They are scoped to this new proposal directory so Q3/C1/D3 and upstream security writers can continue independently.

## Use at dispatch

1. Independently verify prerequisite evidence against the then-current parent/submodule commits; confirm the exact reviewed Q/C runtime APIs.
2. Review the decisions below, select/version the public schema and observation semantics, and reconcile the existing metric proposal. Preserve E0's accepted behavior.
3. Select one existing E1.1 leaf and one case family. Use `tools/mvp.py show` for that exact leaf, then prepare a coordinator-reviewed binding with `tools/mvp.py prepare`. Resolve real commands and expected failure reasons at dispatch; this preparation manifest cannot be passed to the binder as a ready packet.
4. Claim disjoint test/output paths, hash the bounded inputs and run against actual model output. A hand-authored expected trace is not engine proof. A compiler failure for unrelated missing APIs is not a successful red behavioral test.
5. Independent review accepts each leaf; only the full phase join can advance E1.

Preparation verification: JSON parse, catalogue arithmetic/invariants, source/hash checks, local links, context/task/MVP integrity and diff checks. None is an ED simulation run or clinical validation.
