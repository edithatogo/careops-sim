# Independent process matrix review — 10 October 2026

Reviewed source: `84ea6832bb3e182a598e881e0e2d291330879fd0`.
Composite source SHA-256: `20e3aad90936824abb43c8446e65bab160311355bbda14ff22efcc14d50909ee`.
Process test SHA-256: `b8d2be5d55de810ef987e1fc4b0f14ae2300f478f7d441a96b696da7f952e545`.

The component and provider agents independently inspected the clean immutable
source. Fourteen compiled cases invoke an actual child process, rebuild current
trusted codecs/keys/graphs, load the full image, restore native owner state, and
compare exact re-encoded cuts and uninterrupted suffix traces. No original live
objects, prefix replay, or mutable rejection globals are passed to the child.
Worker receipts report matrix, calibration, strict Clippy and formatting passes;
root's final locked integrated join remains pending.

## Required evidence expansion

- Suspend/Restart cases currently restore the initial active cut. Later snapshots
  are captured/re-encoded but not themselves independently restored.
- Arbitrary non-Clone caller context is unchanged after creation. TransitContext
  mutation alone does not establish mutation of the caller-owned type.
- Only one caller context codec and one supported template/factory are covered.
  Composite restore currently requires `T: Clone`; a non-Clone template is not
  supported by this contract, and the matrix must not claim otherwise.
- No future RNG draw is exercised after restoration. Exact saved state equality
  alone is narrower than a future sampling oracle.
- The three envelope fingerprints are fixed fixture constants rather than derived
  from explicit model, configuration and owner schema inventories.
- Rejected start uses real production CounterOverflow. Progress/control rejection
  continuation still needs source-backed coverage where those cuts are reachable.

These are qualification gaps, not demonstrated implementation defects. Root
assigned three disjoint expansion packets; acceptance requires their joined source.

## Resolver-order adjudication

The component reviewer retracted an initial P1 ordering interpretation. ADR0020's
three trusted envelope identities are checked before owner decode. Per-work
binding resolution uses privately staged Flow for linkage and rejects before
exposing a restored result. No artifact-selected executable/graph or external
mutable alias was demonstrated. No API redesign is justified by that retracted
finding.

Full C2, hosted qualification of the new source, release and clinical acceptance
remain unproven. Old hosted success applies only to its recorded older source.
