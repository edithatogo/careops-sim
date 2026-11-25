# HHS provisional interchange

Explicit T4 assignment, 30 October 2026. An optional standalone Rust workspace
consumes files produced independently by hhs_infra. No producer source imports,
Python runtime, root Cargo changes or Kairos pin changes. Synthetic fixtures only
in this repository. The owner is chat 01a187a3-d624-7082-8736-99209fa8720f.

A manifest binds schema_version=1, format=hhs-provisional-spatial, floor/source/
model revision identities, frame, explicit assumptions/exclusions, classification,
provisional=true, spatial_acceptance=false, units_verified=false, graph status and
three fixed SHA-256/row-count/schema-bound Parquet tables: objects, nodes and
edges, with labels represented in objects. Objects contain id, role, nullable label,
geometry_wkb and enabled; nodes id/floor_id/x/y; edges id/from_id/to_id/length.
There are three tables; the fourth bound file is model.json retaining correction
history, provenance, frame and original model objects. Coordinates are local XY
source units; GLB x,z,-y is display-only and never used as routing coordinates.
WKB is preserved as opaque geometry bytes in the consumer, not a new geometry
kernel. Python validates polygon/label geometry; Rust independently validates
Parquet types, nulls, identities, finite coordinates and topology/edge distances.

Manifest identity is SHA-256 of its exact bytes. Loads require the expected
manifest hash, reject unsupported versions/flags/frames, symlinks, extra entries,
corrupt files, duplicate IDs, invalid references and over-limit files. The loader
is immutable and publishes nothing. An unavailable graph stays unavailable.
A live adapter Session replaces its current model only after the whole candidate
validates; checkpoint/restore uses the expected manifest hash. A failed load leaves
its previous revision unchanged. Caller-owned release paths/history are retained.

Routing requires explicit acceptance of provisional input, an assumed positive
metres-per-source-unit scale, speed and explicit allowed edges. Undirected source
edges are oriented along the requested continuous node path; disconnected or
unapproved edges reject. Length conversion and execution duration use the real
CareOps SyntheticGraph/calculate_execution_duration API. Assumptions remain in
the result; never mark source units verified or infer access policy from CAD.
No automatic site-resource binding, clinical approval, live topology mutation,
multi-floor connectors, UI sync or downstream scheduling policy is implied.

Tests must prove Python→Parquet→Rust parity (IDs/null/WKB/frame/provenance),
known-distance timing through the CareOps API, denied/disconnected/invalid cases,
version/flag/hash/schema rejection, failed replacement and checkpoint rollback.

## Campus interchange v2

Version 2 consumes a Python-produced manifest and its three Parquet tables for
spaces, doors and floors. The manifest binds exact file hashes, sources,
coordinate frame, exclusions and row counts. The reader preserves the tables'
scalar columns, including nullable labels, and checks required types, IDs, floor
counts, and same-floor connectivity. The six shell/core exclusions and four
Mental Health missing-plan states are mandatory.

Local coordinates use source units. EPSG:7855 uses metres and requires mapping
metadata bound to the source tables. Movement takes an explicit allowed edge
set and speed, and requires scale one for metre coordinates. Traversal uses
provisional centroid distances. The package flags for physical verification,
verified units and final acceptance remain false.

The contract tests read the Python producer's fictional campus package and
compare every emitted table value with the independently decoded rows. This
package uses the HHS producer's synthetic input generator with one null and
one empty label. The fixtures contain only fictional geometry and identifiers.
