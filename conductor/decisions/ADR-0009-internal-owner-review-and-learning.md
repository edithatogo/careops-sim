# ADR-0009 — Internal owner reviews and evidence-backed procedural learning

Date: 2026-10-09. Status: coordinator decision under the user's instruction to
proceed with the recommended internal reviews and set up a self-improving system.

## Problem

C2 progress stalled after repeated audits found the same contract questions.
Track responsibilities were treated as presumed external people, although the
prepared execution packet named internal DES/ABM agent roles. Repeated audits,
wrong-worktree reads, and retries of over-budget claim inputs consumed effort
without opening the next implementation step.

## Coordination decision

The coordinator may assign qualified internal reviewers to the named engineering
roles within the user's authorized scope. Each disposition records the role,
reviewed repository/commit, evidence, decisions and remaining action. An internal
review is never represented as an external maintainer response. Genuine external
approval needs an identified authority and a concrete action requiring it; a
track number or unfilled packet alone does not establish that requirement.

Resolve architecture in bounded ADRs before dispatch. Workers implement settled
contracts in separate worktrees; the coordinator owns shared manifests, parent
pins and acceptance. Keep the Rust 1.99.0 session constraint for verification.
Track/API, hosted, portable-restart, release and clinical evidence retain their
declared scopes. This decision does not mark C2.4 or change its prerequisites.

Dispatch reviews to answer a specified decision, not to repeat a completed audit.
After a no-progress result, identify the missing decision or changed evidence
that would permit another attempt. Before reporting a blocker, verify source
state, check existing user authorization and distinguish internal coordination
from external action. Tests use behavioral oracles; planned checks are not passes.

## Learning decision

Maintain a repository-local learning tool and lesson store. Record observations,
procedures, evidence hashes and outcome feedback; retrieve only relevant lessons
before work. Candidate lessons are data, not execution authority. Review and
validate a procedure before recommending reuse, and flag stale or tampered
evidence. Failures and superseded procedures remain visible in history.

Procedural learning does not alter model weights. It cannot execute retrieved
text, grant permissions, rewrite acceptance criteria, ingest private patient
data, install mutable external skills, or modify global Codex memory. Promotion
records must name their evidence and reviewer; the tool does not authenticate
reviewer identity or certify agent competence. Independent review and observable
outcomes remain necessary. Capture lessons after meaningful work or corrections,
not after every unchanged status check. Evaluate improvements by reuse outcomes,
failed attempts and time/context saved; do not infer improvement from record count.

The workflow adapts the memory/procedure separation described by
[Hermes skills](https://hermes-agent.nousresearch.com/docs/user-guide/features/skills/)
and [persistent memory](https://hermes-agent.nousresearch.com/docs/user-guide/features/memory/).
Those descriptions are design references, not evidence this local system improves
performance. No Hermes installation or background service is authorized here.

## Acceptance

The local tool must demonstrate retrieval, immutable historical events, valid
review/promotion transitions, freshness rejection and retained negative feedback
using temporary repository fixtures. Then integrate a bounded lookup and
post-task feedback step into the working map. First-session setup proves the
mechanism; measured improvement requires subsequent independent reuse evidence.
