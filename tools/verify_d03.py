#!/usr/bin/env python3
"""Validate D0.3 profile completeness and pinned-source provenance."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import re
import subprocess


ROOT = Path(__file__).resolve().parents[1]
PROFILE = ROOT / "conductor/evidence/d0.3-ed-native-profile-20260927.json"
INVENTORY = ROOT / "conductor/evidence/d0.2-kairos-capability-inventory-20260927.json"
IDENTITY = ROOT / "conductor/evidence/d0.2-prerequisite-identity-audit-20260927.json"
ALLOWED_STAGES = {"MVP", "Native v1", "Post-v1"}
EXPECTED_MODULES = {
    "MVP": {
        "kairo-ecs-types", "kairo-ecs-core", "kairo-ecs-state", "kairo-ecs-rng",
        "kairo-ecs-des", "kairo-ecs-abm", "kairo-ecs-cli",
    },
    "Native v1": {
        "kairo-ecs-arrow", "kairo-ecs-calibration (proposed)",
        "kairo-ecs-bench", "kairo-ecs-debug",
    },
    "Post-v1": {
        "kairo-ecs-viz", "kairo-ecs-ffi", "kairo-ecs-wasm", "kairo-ecs-uniffi",
        "kairo-ecs-diplomat", "kairo-ecs-cs-bridge", "kairo-ecs-gpu",
        "kairo-ecs-webgpu", "kairo-ecs-pdes", "kairo-ecs-mpi", "kairo-ecs-grpc",
        "kairo-ecs-streaming", "kairo-ecs-ml", "kairo-ecs-fmi",
    },
}
EXPECTED_SOURCE_PATHS = {
    "libs/kairos/conductor/tracks/25-api-design-review-compatibility-governance/spec.md",
    "libs/kairos/conductor/tracks/25-api-design-review-compatibility-governance/api-review-plan.md",
    "libs/kairos/conductor/contracts/versioning-compatibility.md",
    "libs/kairos/conductor/tracks/30-toolchain-version-support-matrix/spec.md",
    "libs/kairos/conductor/toolchain-matrix.md",
    "libs/kairos/conductor/tracks/15-packaging-publishing-delivery/handoff.md",
    "libs/kairos/conductor/tracks/42-package-registry-publication-provenance/handoff.md",
    "conductor/delivery-contract.md",
    "conductor/dependency-policy.md",
    "conductor/decisions/ADR-0001-ed-native-support-profile.md",
    "conductor/module-readiness.md",
    "conductor/evidence/d0.3-execution-receipt-20260928.md",
    "conductor/evidence/d0.2-kairos-capability-inventory-20260927.json",
    "conductor/evidence/d0.2-prerequisite-identity-audit-20260927.json",
    "conductor/tracks/development_readiness_20260925/spec.md",
}
EXPECTED_ACCEPTANCE_MARKERS = {
    "functional_mvp": {
        "repeat seed reproduces canonical outputs",
        "hand-computable route/queue cases agree",
        "patient and resource conservation holds",
        "malformed config fails clearly",
        "the two example scenarios run without private data",
        "controlled comparison showing that the selected bed/staff input actually affects a known fixture",
    },
    "hardened_native_v1": {
        "run and compare two scenarios",
        "export results and explain limitations",
        "clean consumer install",
        "troubleshooting path",
        "support matrix and input/schema migration policy",
        "No empirical accuracy claim follows from synthetic verification",
    },
}


def fail(message: str) -> None:
    raise SystemExit(f"FAIL: {message}")


def read_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        fail(f"cannot read {path.relative_to(ROOT)}: {exc}")


def verify_module_stages(actual: dict, stage_profiles: dict) -> dict[str, int]:
    counts = {stage: sum(row["delivery_stage"] == stage for row in actual.values()) for stage in ALLOWED_STAGES}
    for stage, expected_modules in EXPECTED_MODULES.items():
        classified = {name for name, row in actual.items() if row["delivery_stage"] == stage}
        if classified != expected_modules:
            fail(f"{stage} module allocation drifted: {sorted(classified ^ expected_modules)}")
        profile_key = {"MVP": "functional_mvp", "Native v1": "hardened_native_v1", "Post-v1": "post_v1"}[stage]
        if set(stage_profiles.get(profile_key, {}).get("module_names", [])) != expected_modules:
            fail(f"{stage} narrative profile module list differs from the inventory classification")
    expected_counts = {stage: len(names) for stage, names in EXPECTED_MODULES.items()}
    if counts != expected_counts:
        fail(f"unexpected stage counts: {counts}")
    return counts


def verify_stage_acceptance(stage_profiles: dict, contract_text: str) -> None:
    contract = re.sub(r"\s+", " ", contract_text.lower())
    for stage_key in ("functional_mvp", "hardened_native_v1"):
        stage_profile = stage_profiles.get(stage_key, {})
        markers = stage_profile.get("contract_acceptance_markers", [])
        if {marker.lower() for marker in markers} != {marker.lower() for marker in EXPECTED_ACCEPTANCE_MARKERS[stage_key]}:
            fail(f"{stage_key} acceptance-marker set is incomplete or contains unreviewed markers")
        acceptance = re.sub(r"\s+", " ", stage_profile.get("acceptance", "").lower())
        if any(marker.lower() not in contract or marker.lower() not in acceptance for marker in markers):
            fail(f"{stage_key} acceptance is missing or differs from delivery-contract evidence")


def verify_source_hashes(source_hashes: dict, root: Path) -> None:
    if set(source_hashes) != EXPECTED_SOURCE_PATHS:
        fail("pinned source hash set is incomplete or contains unexpected paths")
    for relative, digest in source_hashes.items():
        path = root / relative
        if not path.is_file() or hashlib.sha256(path.read_bytes()).hexdigest() != digest:
            fail(f"profile source hash drift: {relative}")


def main() -> None:
    profile = read_json(PROFILE)
    inventory = read_json(INVENTORY)
    identity = read_json(IDENTITY)
    if profile.get("schema_version") != 1:
        fail("unsupported profile schema")
    if profile.get("kairos_pin") != identity.get("baseline", {}).get("kairos_pin"):
        fail("profile Kairos pin differs from the D0.2 identity audit")
    pin_result = subprocess.run(
        ["git", "-C", str(ROOT / "libs/kairos"), "rev-parse", "HEAD"],
        check=False, capture_output=True, text=True,
    )
    if pin_result.returncode or pin_result.stdout.strip() != profile.get("kairos_pin"):
        fail("checked-out Kairos submodule differs from the reviewed profile pin")
    base = profile.get("parent_base_commit", "")
    if len(base) != 40 or subprocess.run(
        ["git", "-C", str(ROOT), "cat-file", "-e", f"{base}^{{commit}}"],
        check=False, capture_output=True,
    ).returncode:
        fail("profile parent base commit is not a resolvable exact commit")

    capabilities = profile.get("capabilities")
    baseline = inventory.get("capabilities")
    if not isinstance(capabilities, list) or not isinstance(baseline, list):
        fail("capability inventory is missing")
    expected = {row["module"]: row for row in baseline}
    actual = {}
    for row in capabilities:
        module = row.get("module")
        if not module or module in actual:
            fail(f"missing or duplicate capability module {module!r}")
        if row.get("delivery_stage") not in ALLOWED_STAGES:
            fail(f"invalid delivery stage for {module}")
        if not row.get("stage_rationale") or not row.get("owner"):
            fail(f"missing rationale or owner for {module}")
        # Preserve the audited D0.2 row so source changes force a re-review.
        for key in ("capability", "current_state", "evidence", "owner", "delivery_gate"):
            if row.get(key) != expected.get(module, {}).get(key):
                fail(f"D0.2 source inventory drift for {module}.{key}")
        actual[module] = row
    if set(actual) != set(expected):
        fail("D0.3 profile does not classify every D0.2 module exactly once")

    stage_profiles = profile.get("stage_profiles", {})
    counts = verify_module_stages(actual, stage_profiles)
    verify_stage_acceptance(
        stage_profiles,
        (ROOT / "conductor/delivery-contract.md").read_text(encoding="utf-8"),
    )
    mvp_profile = stage_profiles.get("functional_mvp", {})
    for required in ("arrow/parquet", "cad", "websocket", "wasm", "gpu/metal"):
        if required not in " ".join(mvp_profile.get("not_required", [])).lower():
            fail(f"MVP accidentally requires or omits explicit deferral of {required}")
    v1_profile = stage_profiles.get("hardened_native_v1", {})
    v1_included = (" ".join(v1_profile.get("included", [])) + " " + v1_profile.get("acceptance", "")).lower()
    for required in (
        "repeated experiments and uncertainty summaries", "real arrow/parquet",
        "bounded calibration, macro/micro", "queue/preemption conformance",
        "cancellation, atomic outputs and checkpoint recovery",
        "generic input coverage and usable example profiles",
        "supported-platform quality/security/release evidence", "clean consumer",
    ):
        if required not in v1_included:
            fail(f"native-v1 profile omits required delivery increment: {required}")

    execution = profile.get("execution_profile", {})
    if execution.get("runtime_language") != "Rust" or execution.get("execution_backend") != "local CPU":
        fail("native runtime profile must remain Rust/local CPU")
    if execution.get("consumer_msrv", "").startswith("1."):
        fail("consumer MSRV must remain unresolved until D1.2 evidence")
    if execution.get("target_status", "").find("not qualified") < 0:
        fail("candidate platforms must not be represented as qualified before CI evidence")
    if set(execution.get("candidate_ed_targets", [])) != {"Linux x86_64", "macOS aarch64"}:
        fail("candidate ED platform scope drifted")
    review_status = profile.get("upstream_owner_alignment", {}).get("independent_review", "")
    if "passed on 2026-09-28" not in review_status or "no Kairos maintainer sign-off asserted" not in review_status:
        fail("independent review status or upstream approval boundary is missing")
    publication = profile.get("publication_boundary", {})
    if publication.get("public_package_or_registry_publication", "").find("blocked") < 0:
        fail("global Kairos publication hold was not preserved")
    if "does not waive or change Kairos" not in publication.get("authority_note", ""):
        fail("profile must disclaim authority to waive upstream policy")
    if set(publication.get("upstream_holds", [])) != {"15", "16", "20", "25", "28", "30", "42", "44"}:
        fail("profile does not preserve the full set of Kairos global release holds")

    verify_source_hashes(profile.get("source_hashes_sha256", {}), ROOT)

    adr = (ROOT / "conductor/decisions/ADR-0001-ed-native-support-profile.md").read_text(encoding="utf-8")
    summary = (ROOT / "conductor/module-readiness.md").read_text(encoding="utf-8")
    receipt_path = ROOT / "conductor/evidence/d0.3-execution-receipt-20260928.md"
    receipt = receipt_path.read_text(encoding="utf-8")
    task_plan = (ROOT / "conductor/tracks/development_readiness_20260925/plan.md").read_text(encoding="utf-8")
    if "Status: accepted for CareOps Sim after independent review" not in adr or "no Kairos maintainer approval is recorded" not in adr:
        fail("ADR acceptance or maintainer-approval boundary is missing")
    if "d0.3-ed-native-profile-20260927.json" not in adr or "ADR-0001" not in summary or "no remaining D0.3 scope or verifier findings" not in " ".join(receipt.split()):
        fail("profile and ADR/readiness handoff are not cross-referenced")
    if "D0.3 Define the ED-native support/release profile with 25/30" not in task_plan:
        fail("active plan no longer contains the D0.3 owner/profile acceptance")

    print(f"PASS: {len(actual)} Kairos capabilities classified once; MVP/native-v1/post-v1={counts['MVP']}/{counts['Native v1']}/{counts['Post-v1']}")
    print("PASS: Rust/local-CPU scope, provisional platform/MSRV state, pinned contracts and upstream publication holds verified")


if __name__ == "__main__":
    main()
