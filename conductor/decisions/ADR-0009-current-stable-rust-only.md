# ADR-0009 — Rust 1.99 as the sole pinned development target

Date: 2026-10-05. Status: accepted by explicit user direction; local foundation implementation under review.
Owner decision: Rust 1.99.0 is the sole pinned Rust version and the sole declared workspace floor. Development targets current stable Rust; older Rust versions are not maintained as compatibility targets.

## Decision

The workspace manifest declares `rust-version = "1.99"`. The existing exact
`rust-toolchain.toml` pin remains `1.99.0`. The D3 quality configuration keeps
its schema v1 `default_msrv` key, but the key now means the sole declared target
floor and must be `1.99`; it does not describe a backward compatibility matrix.
The quality validator rejects any different floor.

When a new stable Rust release is adopted, update the exact pin and declared
floor together under a reviewed owner decision. Historical manifests, test logs,
and compatibility receipts remain unchanged as provenance; they do not promise
continued support or acceptance on older versions.

## Scope and follow-up boundaries

This foundation change updates the parent workspace floor, D3 quality policy and
validator, focused policy tests, and this owner decision. It does not modify the
Kairos submodule or pin, CI workflows, dependencies, manifests below the parent
workspace, unrelated tests, ED logic, task checkboxes, or release metadata. CI
lane reconciliation and metadata alignment require their separately reviewed
owner packets. Nightly-only Miri and fuzz coverage are unverified by this stable
Rust 1.99 foundation check and must be classified explicitly in the separate CI
policy work.
