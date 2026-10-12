# Dependency packet preparation lessons

12 October 2026; private candidate for reviewed repository learning.

Observed before source edits or native gates: MVP prepare writes the packet under .artifacts/packets and context under .artifacts/mvp/<packet_id>. Reserve both exact locations and run the scope check before prepare. Do not infer ignored outputs are authorized from their absence in git status. Preserve the original packet on a corrected attempt.

Cargo metadata packages carry name/version/source but do not carry Cargo.lock registry checksums. Verify metadata identities independently, and compare checksums using baseline Cargo.lock read from the exact baseline Git commit against the final lockfile. Resolved dependency node names may normalize hyphens to underscores; identify the target through packages[dep.pkg].name.

Recipe files may exceed harness input snapshot limits. Keep them recipe-hashed by MVP preparation; use the bounded owner contract for the lease input and explicit reviewed context slices. Do not weaken source snapshot limits to accommodate whole recipe catalogs.

Root review caught these issues before any source edits or metadata gates. No native failure or test pass is claimed. A future harness improvement may expose prepare output paths for preflight reservation and add a dependency evidence template; these require separately reviewed ownership and execution evidence.
