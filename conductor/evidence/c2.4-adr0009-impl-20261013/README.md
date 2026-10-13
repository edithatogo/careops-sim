# C2.adr0009-impl binding — 2026-10-13

This directory binds the ADR-0009 fresh-process restore contract that
`envelope-spec.md` §6 left open. It does not change Rust, the Kairos pin, or
`conductor/current-state.json`.

## Artifacts

- `impl-binding.md`: byte layout, codec rule, event-*k* disposition, non-public
  entry, fail-closed negatives, and C5 non-goals.
- Packet: `conductor/execution/packets/C2.adr0009-impl.json`.

## Readback

`conductor/evidence/c2.4-adr0009-envelope-20261012/implementation-attempt-readback.json`
is absent on `origin/main` at `0de745a50706392c8e8fea7d4a47b73a8249677c`.

`python3 tools/tasks.py packet-check` passed against that base. A later
dispatch must rebind `base_commit` to the commit that contains this packet.
