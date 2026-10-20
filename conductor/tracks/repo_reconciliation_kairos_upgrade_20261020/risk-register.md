# Risk register: Repository status reconciliation and Kairos upgrade

| Risk | Impact | Mitigation |
| --- | --- | --- |
| Upgrading Kairos submodule introduces breaking API changes to `kairo-ecs-des` or `kairo-ecs-abm` | Build failure in `careops-ed` | Compile gate in R2 with isolated commit review; Kairos preserves semantic backward compatibility across alpha releases |
| Test suite fails due to python virtualenv path discrepancies | False positive test failures | Standardize runner script invocation to resolve `.artifacts/ci/harness-venv` in R1 |
| Status reconciliation accidentally reopens completed tasks | Audit confusion | Read-only alignment matching existing accepted evidence files in `conductor/evidence/` |
