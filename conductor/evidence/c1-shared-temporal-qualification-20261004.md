# C1 shared temporal helper qualification

Kairos development commit `4a90ebce58d8d31655d89a6c7c35680241c0dec8` retains the Q4 ABM foundation and adds a role-preserving shared timestamp normalizer. The required occurrence wrapper keeps its missing/role precedence and unchanged result fields.

Local Rust 1.76 and 1.98.1 Arrow suites each passed 23 tests. Exact-head [owner CI](https://github.com/edithatogo/kairos/actions/runs/37165239657) passed all 11 jobs; both native hosts ran 23 Arrow tests including six new role fixtures. Ten retained artifact ZIPs had verified API digests, CRCs and member hashes. Details are in [the qualified receipt](c1-shared-temporal-qualified-receipts-20261004.json).

The `KnowledgeAvailable` enum variant can break exhaustive matches; acceptance is for experimental development and retains the compatibility/release hold. Full C1 data mapping, UTC codec, exclusion accounting, ordering, occupancy, censoring and clinical acceptance remain open. This parent pin requires its own exact-head hosted gate; earlier billing failures do not satisfy it. No phase checkboxes were changed.
