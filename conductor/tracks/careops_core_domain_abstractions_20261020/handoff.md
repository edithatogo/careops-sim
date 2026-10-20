# Handoff: Multi-domain core simulation abstractions and egress boundaries

Status: Proposed.

Prerequisites: `repo_reconciliation_kairos_upgrade_20261020:R3` complete and accepted.

Upon completion of phases A0–A6, `careops-core` provides the reusable, domain-neutral clinical engine. `careops-ed` is successfully decoupled, and the birthing suite delivery (`birthing_suite_delivery_20261020`) can immediately consume `careops-core` to construct the birthing simulation.
