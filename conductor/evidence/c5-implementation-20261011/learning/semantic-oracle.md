# C5 lesson evidence — 11 October 2026

At integration source d4e1284, all25 CLI/worker tests passed despite the native
empty-anchor registration defect: identical failed attempts still produced equal
worker bundles. Native tests exposed the failure; the coordinator then required
exact known winner c000004 (route2/work3), score0, final mean0 and2 completed final
attempts in scripts/qualify_c5.py. Corrected native execution and those semantic
assertions passed local process qualification at clean source bebc140.

Independent worker c5_cli reviewed exact summary field names and the executable
semantic assertions. The lesson is to pair equality/recovery oracles with a known
successful native result and explicit failure/censor counts; equality alone proves
neither success nor the intended model behavior. MC precision remains separately
indeterminate. This does not qualify a model or expand task authority.
