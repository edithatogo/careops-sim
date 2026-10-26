# Acceptance matrix

Required evidence, not automatic pass claims. Bind exact test targets and commands
from the retained candidate and current source before execution.

| Gate | Required oracle |
|---|---|
| Inventory | Every active dev/build/test/automation/CI/container/helper path classified; historical and intentional negative fixtures separated |
| Actual runtime | Repo-local interpreter and invoked subprocess report exactly 3.15.1; wrong interpreter and out-of-line candidate reject |
| Locks and packages | Constraints/resolver targets/pins agree; fresh locked install, package build/import and actual native consumers pass |
| Existing behavior | Meaningful helper/security/binding tests and unchanged simulation/domain contracts; no weakened checks |
| Platforms | Required hosted OS lanes and actual container/native execution or explicit unresolved blocker; manifest alone insufficient |
| Publication | Final PR head required checks, review resolution, protected merge and exact-main post-merge verification |
| Handover | Old operation terminal evidence; new consumer command/path/version confirmed; old private state recoverable |
| Closeout | Full requirement audit and raw command/cwd/commit/toolchain/input/output hashes, exit statuses and artifact paths |

Planning checks: `.venv/bin/python tools/tasks.py check`; `.venv/bin/python tools/context.py check`. Local full Rust tests are needed only where affected; inherited native target contracts remain required. Catalogue/plan validity does not establish runtime acceptance.
