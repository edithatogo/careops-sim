# E2.1 space fixture crosswalk

These are 32 synthetic E2.1 specifications. The 20 positive expected histories
are hand-authored and pinned in the focused checker; its independent structured literals
cover states, claims, occupancy, waits, reservations, diagnostics and horizon
counts, with explicit boarding/offer clocks. Supplementary hashes pin remaining
reviewed expected fields. Generic checks validate references,
strict input types, calendars, capacity, lifecycle partitions, work claims,
waiting, diagnostic rooms and horizon conservation. No simulator or native
binding is executed; every case remains `unbound` targeting E2.2.

| Cases | Contract coverage | Evidence boundary |
| --- | --- | --- |
| `space.cleaning_blocks_reuse`, `space.cleaner_wait_dirty`, `space.cleaning_calendar`, `space.cleaning_horizon` | Departure, dirty wait, cleaner queue/calendar, completion and H grant | Synthetic timelines only; cleaning requires a named eligible work claim |
| `space.reservation_blocks`, `space.reservation_expiry_tie`, `space.reservation_consumption`, `space.reservation_wrong_claimant`, `space.reservation_conflict`, `space.reservation_occupied`, `space.reservation_dirty` | Start ordering, owner-only consumption, stale expiry, deterministic rejection and no retry | Rejections are valid outcomes; malformed configuration is covered by negative cases |
| `space.diagnostic_serial`, `space.diagnostic_unavailable`, `space.diagnostic_staff_wait` | Room-before-staff order, capacity-one room, independent room/staff waits and held room | Fixed fixture approximation; no atomic staff-plus-space acquisition |
| `space.boarding_retention`, `space.boarding_care`, `space.unused_offer`, `space.withdrawn_offer`, `space.transfer_horizon` | Offer use/expiry/withdrawal, recurrent care exclusion, transfer work and physical departure | Admission decisions/offers do not release ED occupancy; unfinished transfer has no terminal LOS |
| `space.permutation` | Reversed patient, space, staff, offer and reservation arrays with canonical witness | Input-order invariance is still a native E2.2 integration gate |
| `space.invalid.duplicate_space_id`, `space.invalid.count_mismatch`, `space.invalid.zero_static_capacity`, `space.invalid.cleaning_nonpositive_work`, `space.invalid.unstaffed_cleaning_policy` | Identity, static counts and supported cleaning policy | Dynamic unavailability is not invalid static zero capacity |
| `space.invalid.duplicate_reservation_id`, `space.invalid.reservation_nonpositive_interval`, `space.invalid.reservation_unknown_space`, `space.invalid.reservation_unknown_patient`, `space.invalid.reservation_start_after_horizon` | Reservation identity, interval and reference validation | Start is in `[0,H]`; expiry may exceed H |
| `space.invalid.atomic_staff_space`, `space.invalid.multi_role_work` | Explicit unsupported acquisition and work shapes | No silent substitution or invented native API |

## Report 29 crosswalk

| Report 29 findings | Disposition | Follow-on boundary |
| --- | --- | --- |
| ST001, ST002: consumed-once and unused/expired offers | Compatible with these fixtures | Offer/runtime joins remain unbound until E2.2 |
| ST003, ST004: ED occupancy through physical departure and recurrent care claims | Compatible with boarding retention/care witnesses | Integrated native lifecycle remains E2.2 |
| ST005, ST006: ambulance arrival, triage, handover, offload, crew release and receiver-staff sensitivity | Explicitly deferred | Composed ambulance work belongs to E2.3 |
| ST007, ST008: multi-person sedation/resuscitation teams | Explicitly unsupported here | Requires a reviewed, non-overlapping team model before native support |
| ST009: finite external offer streams do not identify causal ward/fleet/community effects | Explicit limitation | No causal downstream effect is inferred by these fixtures |

Every negative has a complete valid baseline, one recorded field mutation, a
category and path, and a repair structurally equal to baseline. Specification
checks are not native runtime, clinical, operational, stable-release or
acceptance evidence.
