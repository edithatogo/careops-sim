# Parent learning/C2 harness regression repair

Root independently ran all **512 Python tests: pass**, at parent `51839c9`; the 51 focused tests for the three corrected modules also pass. Commands, commit, hashes and raw logs are retained here. The earlier 510-test run with four failures remains in `c2-learning-progress-20261009`; this receipt supersedes its pending parent-regression status only.

Three stale assertions now reflect the accepted 82-parent task graph, the transit-bridge prerequisite, and C2.5 preceding C2.4. Documentation counts match 82 parents and 258 leaves. Five stored lognormal fit values differed by library rounding under CPython 3.11.6 versus the fixture's CPython 3.14.7; the fixture explicitly disclaims cross-platform libm bitwise identity. The test keeps inputs, hashes, seeds, split membership, bootstrap results, scenario fields and same-runtime repeatability exact. Narrow fit roundoff comparison bounds are guarded by negative tests for excessive fit drift, a single-ULP bootstrap mutation, and missing/extra fields. Frozen fixtures and calibration/statistical acceptance thresholds are unchanged.

Fourteen learning-loop tests are included in the broad pass. This verifies the local tooling mechanism, not agent competency, improved calibration, hosted tests or C2.4 acceptance.
