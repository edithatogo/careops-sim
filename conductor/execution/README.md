# Execution artifacts

Start with [the protocol](../execution-model.md).

- [Task catalog](tasks.json): generated from all five track plans and metadata.
- [Decomposition](decomposition.md): bounded worker leaves and coordinator joins.
- [Packet template](packet-template.json): bind before dispatch; never executable as-is.
- [First inventory recipe](packets/D0.2.inventory.json): source-extraction canary.
- [Worker prompt](worker-prompt.md): usable by a serial worker or parallel subagent.
- [Result template](result-template.json): worker evidence, not accepted completion.

Commands from the parent repository:

```sh
python3 tools/context.py check
python3 tools/tasks.py check
python3 tools/tasks.py ready --mode serial
python3 tools/tasks.py ready --mode parallel --workers 4
python3 tools/tasks.py schedule --mode parallel --workers 4
```

`ready` uses accepted plan checkboxes and supports repeated `--reserved-path`
arguments for the coordinator's active reservations. `schedule` computes
hypothetical future waves only; it does not update completion, acquire locks or
predict elapsed runtime. Actual dispatch requires reviewed leaf packets, accepted
leaf dependencies, active-reservation checks and isolated target-repo worktrees.

After plan changes run `python3 tools/tasks.py build`, inspect the catalog diff,
then run the checks. Bind the inventory recipe only from a clean committed
checkout; local packets/artifacts under `.artifacts/` are ignored by Git.
