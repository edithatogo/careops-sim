# A2.1 capacity and calendar contract

- Task: `careops_core_domain_abstractions_20261020` / A2.1
- Contract: [ADR-0010](../../decisions/ADR-0010-effective-capacity-and-calendar-boundaries.md)
- Bound packet: [A2.1.capacity-contract.json](../../execution/packets/A2.1.capacity-contract.json)
- Base revision: `94d2792bb4bebebc5cc08087282993be37d41603`
- Contract commit: `7e05673a169dbd8d2f291a7d1c65e8aacc7be4d1`
- Reviewer: reviewer-agent/subagent, independent contract and acceptance lens
- Review result: initial double-counting ambiguity between open ceilings and
  unavailable dispositions was fixed; pool acceptance was narrowed to
  deduplicated snapshot accounting; tick conversion precision was deferred to
  a future adapter. Follow-up review found no remaining finding.

## Executed checks

| Command | Outcome | Oracle |
| --- | --- | --- |
| `.venv/bin/python tools/tasks.py packet-check conductor/execution/packets/A2.1.capacity-contract.json` | Passed | No packet validation errors; base and context hashes were bound. |
| `.venv/bin/python -m json.tool conductor/execution/packets/A2.1.capacity-contract.json` | Passed | Packet parses as JSON. |
| `git diff --cached --check` | Passed | No whitespace errors in the staged contract and packet. |

No runtime behavior is claimed by A2.1. Executable positive/negative cases are
implemented and verified with A2.2; no ED schema/default, Kairos source/pin, or
timezone database change is included.
