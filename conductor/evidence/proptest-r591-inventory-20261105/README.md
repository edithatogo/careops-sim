# Proptest R59.1 inventory

Published Kairos PR305 is merged at418b6bc; its core manifest and two Track96 test files match tested2e8782b byte-for-byte. Historical owner acceptance reports51 tests. The current runtime074 pin remains unchanged; this verifies a historical prerequisite, not a new upstream implementation or parent adoption.

The parent lock currently resolves proptest1.8.0 through the implicit Kairos core workspace member. Neither CareOps core nor ED declares a direct edge, and the parent has no property runner for its healthcare invariants. Existing conformance and fuzz tests remain protected.

The accepted qualification scope is a core-only dev edge, with independent capacity, identity and checkpoint properties. Capacity uses widened arithmetic/tuple references; identity uses independent logical sets/counts plus canonical outcomes; rollback requires valid-prefix execution before a later failure; recovery also checks independent lifecycle results. Generation must support full u64 ranges, without an exhaustive-coverage claim. Reproducible seeds, bounded shrinking and regression persistence are required.

No production code is proposed for deletion. The value question is whether these required test capabilities improve robustness enough to justify dependency, support and execution costs. Exact1.12.0/std remains a candidate until R59.2–R59.3 qualification; all R60/R61 gates remain unchecked. New-format durable resume remains the existing Track70 responsibility.

[Inventory and source bindings](inventory.json) record the reviewed scope and limits.
