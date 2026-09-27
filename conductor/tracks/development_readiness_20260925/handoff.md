# Handoff: Development readiness and hardened delivery

Status: in_progress; full implementation/acceptance remains outstanding.
Read [the plan](plan.md), [module readiness](../../module-readiness.md) and
[development audit](../../evidence/development-audit.md) before starting.

Owners: 13/20/27/30/44. Respect milestone dependencies, source pins and the existing
Kairos contract/compatibility rules. Keep source changes in their owning repo.
No later clinical-domain tracks have been introduced.

Next: D0 is closed with source-backed receipts and passing local planning checks.
D0.3's support-profile review does not claim Kairos maintainer approval; D1.2
still requires owner review for MSRV/toolchain compatibility, and candidate
platforms remain unqualified. Start D1.1: test missing/wrong tools and establish
clean macOS ARM/Linux bootstrap evidence. Do not count planned gates or upstream
status labels as actual readiness. Remote creation follows D2.
