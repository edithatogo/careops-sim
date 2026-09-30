# P0.2 row replacement drafts

The three JSON files contain one replacement for each of the 60 rows marked
`proposed_revise` in the provisional 101-row disposition. Three disjoint
`gpt-6-luna` worker packets drafted ranges 001–035, 036–070, and 071–101
after independent QC found that an earlier generic merge repeated role
rationales instead of stating a replacement. The coordinator reviewed the
drafts and clarified rows 35, 37, and 75:

- Row 35 is an optional, separately owned disposition-work interval; it cannot
  include boarding, bed search, transport or departure delay.
- Row 37 permits an explicitly configured aggregate **local** Macro transit
  interval and a route-derived Micro interval, without adding both; external
  transfer/transport remains a separate hospital-boundary input.
- Row 75 uses `hospital_boundary.admissiondemand` only for finite external
  competing demand. ED-generated admission requests arise once from patient
  disposition and do not create another ED arrival stream.

These are proposed interface replacements, not accepted keys, API signatures,
empirical values or distributions. The source role recommendations remain in
`../p0.2-panel/selected/`, and the copied candidate preserves reviewer,
session, base commit, evidence locator and rationale for each row. The
coordinator disposition CSV remains provisional while the binding contract,
row-level matrix update, and final P0.2 join are reviewed.
