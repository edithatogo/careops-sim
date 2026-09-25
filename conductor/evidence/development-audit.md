# Development-readiness audit receipt

Date: 2026-09-25. Parent baseline before this audit: `7c2aa57`.
Kairos pin: `fae901558f07b7b717a676adbafbe2cdc78dea1c` (matches live remote HEAD).
Conductor release pin: `7a5c560a4fdf5297be58594cc37527eb12790272` (latest observed
release v0.4.1); newer unreleased HEAD recorded in dependency policy.

## Executed baseline test

Working directory: `libs/kairos`.
Command: `cargo test --locked --offline -p kairo-ecs-core -p kairo-ecs-state -p kairo-ecs-rng -p kairo-ecs-des -p kairo-ecs-abm`.
Result: exit 0; 60 existing tests passed, no failures. No code or lockfile change.
Host: Apple silicon macOS; rustc 1.98.1 (48a229cea 2026-09-01), cargo 1.98.1.
This does not cover new feature plans, whole workspace, FFI, GPU or distributed use.

## Observed tools

Python 3.14.7; Node v26.9.0; gh 2.101.0; cargo-nextest 0.9.146;
cargo-deny 0.20.2; cargo-audit 0.22.1. Presence is not compatibility evidence.
In particular, host Node is not automatically within Kairos's declared TS range.

## Registry audit

`python3 tools/refresh_versions.py` read 31 public registry/distribution entries.
All returned version observations after filtering the CodeQL action release
separately from its bundle releases. See [machine-readable snapshot](dependency-candidates.json).
No installation, dependency upgrade, remote creation, recurring job or new skill
installation was performed. Exact integration pins remain a D1 task.

## Local bootstrap scope

Added AGENTS map, bounded resume/check command, current-state record, dependency
refresh tool and harness regression tests. These implement D0.1 only. The new
tracks, capability gates, skill evaluations, CI/release and ED work remain planned.
Final validation results are appended after execution below.

## Final local verification

- `python3 -m unittest discover -s tests -v`: exit 0; 11 tests passed, including
  missing context/link, pin drift, unknown phase/dependency, dependency cycle,
  independent phase preservation and missing completion-evidence cases.
- `python3 tools/context.py check`: exit 0; four tracks, 28 milestones, 136 local
  links, no errors.
- `python3 tools/context.py resume`: restored the active D0 next action and source
  pins; final check additionally validates the active phase exists.
- Phase checklist audit: 28 checkpoints, 119 tasks; only D0.1 is marked complete.
- Git whitespace validation passed; submodules remained clean and unchanged.

These are local checks. No hosted CI, complete ED implementation, new feature
acceptance, GPU/PDES/network acceptance or actual release has been claimed.

Artifact hashes (SHA-256):

- `libs/kairos/Cargo.lock`: `0e2674ac90577db9642f07fd306d9b9a59dfb4e0da9132a818eec10a74230645`
- `tools/context.py`: `6ae36ff0bbdb5dd0f1199ffa4e8a0bb608010aa12412b1339446e21d9953c25d`
- `tools/refresh_versions.py`: `f2650728076dcc40b8938a529a75cf40ad298fdd30d801ec6f1cee07c7236f19`
- `tests/test_context.py`: `d5f3cd55698a46cee203b8cf24281640d5bab4e39a762bfbb842cb0a2910a58e`
- `conductor/evidence/dependency-candidates.json`: `59a76605610ea3742b080d22981d6754ed0e1792b41265678ce41cab11b12ebf`
