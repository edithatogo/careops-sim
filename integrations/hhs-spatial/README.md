# Optional HHS spatial adapter

This independent Rust workspace reads HHS provisional Parquet packages and calls
CareOps' existing `disposition::operations` movement API. It is not included in the
root workspace and adds no mandatory dependency to CareOps or the Python producer.
Initialize the repository's exact `libs/kairos` gitlink before building. Rust 1.99.1
and the committed adapter Cargo.lock are the qualified dependency boundary.

```
cargo clippy --manifest-path integrations/hhs-spatial/Cargo.toml --locked --all-targets -- -D warnings
cargo run --manifest-path integrations/hhs-spatial/Cargo.toml --locked -- PACKAGE EXPECTED_MANIFEST_SHA256
```

The final command emits the retained manifest/model and decoded tables. Geometry
is returned as WKB hex; labels retain null versus empty strings. Keep output for
private packages in private storage outside this repository. To generate a v1
package, use the standalone HHS producer.

```
hhs-infra --workspace PRIVATE_WORKSPACE interchange --model MODEL_JSON --exports EXPORT_DIRECTORY --output NEW_PACKAGE_DIRECTORY --classification private_source_derived
```

For a v2 campus package, use the HHS producer with its topology and floor matrix.

```sh
python -m tools.spatial_interchange --schema-version 2 --spaces SPACES_PARQUET \
  --doors DOORS_PARQUET --floors FLOOR_MATRIX --output NEW_PACKAGE_DIRECTORY \
  --classification private_source_derived
```

The producer independently validates the model revision and export file hashes.
Supply the resulting exact manifest hash from its trusted receipt. Hashes detect
corruption; they do not authenticate the author or certify physical geometry.
An optional fourth CLI argument is a JSON file with `path` (ordered node IDs) and
`assumptions`: `accept_provisional`, `assumed_metres_per_source_unit`,
`speed_m_per_s`, `allowed_edge_ids`. Unapproved edges and disconnected paths fail.
The result retains these assumptions and all physical-verification flags false.
No automatic unit calibration, access policy, clinical resource binding, live
simulation topology replacement or navigation safety approval is supplied.

Use `Session::replace` to validate an entire new revision before changing the
active model. A failure leaves the previous model available. Persist the bytes
from `checkpoint()` in caller-owned storage; `restore(package, checkpoint)` checks
the pinned hash before replacing the session. Keep old immutable package folders
for rollback; never edit a partial/corrupt package to force acceptance. Re-export
into a new directory or restore its original trusted bytes. Unknown package and
checkpoint schemas reject instead of silently migrating. This is adapter revision
recovery, not a replacement for the simulation engine's own state checkpoints.

## Contract tests

The repository contains readable rows and schemas for fictional fixtures from
the HHS producer. Reconstruct the original Parquet files in the ignored
`tests/fixture-packages` directory. The generator verifies each file against
its retained manifest SHA-256, including the malformed fixtures used to check
rejection behavior. Use an isolated fixture environment with Python 3.15.2
and the pinned fixture dependencies.

```sh
uv venv --python 3.15.2 .venv-hhs-fixtures
uv pip sync --python .venv-hhs-fixtures/bin/python --require-hashes integrations/hhs-spatial/tests/requirements.txt
.venv-hhs-fixtures/bin/python -I -B integrations/hhs-spatial/tests/generate_fixtures.py
rustup run 1.99.1 cargo test --manifest-path integrations/hhs-spatial/Cargo.toml --locked
```

The dedicated tests cover v1 and v2, full table values, routing, refusal and
checkpoint recovery. [Contract](CONTRACT.md) records the exact interface.
