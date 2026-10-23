# Python migration inventory — 23 October 2026

This is an active-use inventory and evidence map, not completion. [Hashed inventory](inventory.json) lists 158 parent-owned Python source/test files and 8 workflows at the recorded base. Actual local runtime reports 3.15.1 and the `.venv` prefix.

| Surface | Current route | Remaining proof |
|---|---|---|
| Development/helpers/tests | Verify `.venv` using `tools/python_runtime.py`; activate or invoke explicitly; child invocations inherit `sys.executable`/qualified PATH | Terminal handover in retained main checkout |
| Parent CI (seven Python workflows) | Exact setup selector bootstraps verified local `.venv`; subsequent calls use absolute `CAREOPS_PYTHON` | Final full required hosted checks and main runtime logs |
| Schema lock/candidate qualification | Exact 3.15.1 audit/helper environments; development discovery limited to 3.15; all environments beneath checkout | Actual scheduled/manual candidate results when invoked; none claimed from a manifest |
| Dependencies | Five exact schema roots and hash lock; fresh wheel-only install on local 3.15.1 passes; previous latest qualification retained | Hosted platform execution |
| Parent package/container defaults | No tracked parent pyproject or Dockerfile exists; Rust native packaging invokes qualified helper | Hosted package workflow proof |
| Pinned Kairos/Conductor sources | Independent repositories/owners; preserve exact gitlinks | Kairos migration and existing pin reconciliation remain separate |
| Shared security | `source-security.yml` pins CareOps policy `6319d0c...`; direct readback shows Python 3.15.0 and base environment. Current upstream main readback is `c18b43b...` with 3.15.1 but still no repo-local guard | Upstream owner qualification, accepted immutable pin and real runtime/prefix proof |
| Historical/synthetic versions | Dated evidence and intentional version-negative fixtures retained | Never used as active execution defaults |

The full 717-test local run found only three old command-spelling fixtures; their eight focused tests pass after correction. Final full integration, clean release/security gates and safe environment handover remain required. No RNG/clinical/domain policy, native source, private history or Kairos runtime pin changed.

Testing policy uses native unittest/Cargo selectors. Optional pytest plugins are not added to this unittest suite. The upstream-qualified pytest-difftest 0.3.0 PyO3 blocker is retained without repeating the failed build or bypassing Python support. Caches remain per worktree/environment; retries default to zero.
