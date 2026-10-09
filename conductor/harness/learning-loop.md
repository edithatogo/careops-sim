# Local learning loop

`python3 tools/learning.py` stores bounded, repository-local suggestions in
`lessons.jsonl`. The JSON Lines log is append-only, SHA-256 hash chained, capped
at 1 MB and 32 KB per event. It contains no credentials or private operational
data. Hash chaining detects accidental or unsophisticated edits; it is not a
signature, trusted timestamp, or protection from a writer able to rewrite the
whole log.

## Workflow

1. `python3 tools/learning.py seed` appends four initial proposals once.
2. `python3 tools/learning.py search worktree` retrieves up to five concise
   matching records by default, checks evidence hashes on every matching
   record, and fails closed on a stale match. Use `--limit N` for 1–20 records;
   `--include-superseded` includes replaced history. `python3 tools/learning.py
   show ID` reads one record's full feedback history.
3. A coordinator or independent reviewer within existing task authorization
   reviews a bounded procedure with `review ID --reviewer ROLE_OR_LABEL --note
   TEXT`. This internal review does not expand task scope or grant permission.
4. `validate ID` promotes only a reviewed lesson whose evidence hashes remain
   current. This command checks source freshness; it does not execute commands
   or establish that the procedure is effective. Success reports alone cannot
   promote a lesson. Every entry remains advisory and never grants task
   authority.
5. After choosing to apply a validated procedure, record `use ID --note TEXT`.
   Append `outcome ID helped|partly_helped|did_not_help|not_applicable --note
   TEXT` after observing the result. Search output aggregates actual use counts
   and outcomes by category; failures remain in the log. Feedback does not
   automatically promote or demote a lesson.

Custom proposals use `add lesson.json`. A proposal has `id`, `title`, `topics`,
`procedure`, and nonempty `evidence`, an array of repository-relative `{path,
sha256}` entries. An optional `supersedes` names an existing lesson ID. A
proposed successor never hides its predecessor; only validating the successor
marks the predecessor superseded in default search. The event log retains both
records and their feedback without rewriting either procedure. IDs are
permanent and cannot be reused. Keep procedures bounded, reviewable and free of
secret or patient information.

## Seed evidence boundary

Initial records are proposals only. Exact worktree/HEAD binding and bounded
claims follow the single-maintainer guide and its implementation. Owner-role
handling is grounded in the Track22 packet's empty disposition and explicit
statement that no response or acceptance is recorded. The no-repeat-audit
proposal is a heuristic motivated by that same unchanged evidence boundary; it
is not a claim that an audit was repeated. Every proposal records evidence
hashes so later source changes make retrieval and promotion fail closed.

No loaded lesson executes itself. This is a local evidence index, not global
memory, a model-training system, independent review, task dispatch, a
permission system, or a recurring job.
Reviewer role/identity is caller-supplied metadata and is not authenticated;
the tool does not enforce independence. Evidence hash validation is separate
from reviewer opinion. Actual permission or scope changes continue to require
the existing authority and Conductor gates. Append access is cooperative; do
not treat local log integrity as external attestation.
