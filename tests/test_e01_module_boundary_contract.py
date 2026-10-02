"""Integrity checks for the E0.1 logical module-boundary contract only."""

import json
import unittest
from pathlib import Path


CONTRACT_PATH = (
    Path(__file__).resolve().parents[1]
    / "conductor/design/ed/e0.1-module-boundary-contract.json"
)
EXPECTED_OWNERS = {
    "careops_parent",
    "kairos_upstream",
}
EXPECTED_OWNER_LABELS = {
    "careops_parent": "CareOps parent repository owns ED domain, site policy, runner, presentation, and its export artifacts.",
    "kairos_upstream": "Kairos upstream owns reusable simulation engine crates and public engine interfaces.",
}
EXPECTED_MODULES = {
    "careops.ed_domain": "careops_parent",
    "careops.site_policy": "careops_parent",
    "careops.runner": "careops_parent",
    "careops.presentation": "careops_parent",
    "careops.export_artifacts": "careops_parent",
    "kairos.public_api": "kairos_upstream",
    "kairos.reusable_engine": "kairos_upstream",
}
EXPECTED_PERMITTED_EDGES = {
    ("careops.ed_domain", "kairos.public_api", "public_api"),
    ("careops.site_policy", "kairos.public_api", "public_api"),
    ("careops.runner", "careops.ed_domain", "parent_application"),
    ("careops.runner", "careops.site_policy", "parent_application"),
    ("careops.runner", "kairos.public_api", "public_api"),
    ("careops.presentation", "kairos.public_api", "public_api"),
    ("careops.presentation", "careops.export_artifacts", "export_artifact"),
}
EXPECTED_FORBIDDEN_EDGES = {
    ("kairos.reusable_engine", "careops.ed_domain", "ed_policy_dependency"),
    ("kairos.reusable_engine", "careops.site_policy", "site_policy_dependency"),
    ("kairos.reusable_engine", "careops.runner", "application_dependency"),
    ("kairos.reusable_engine", "careops.presentation", "presentation_dependency"),
    ("careops.presentation", "kairos.reusable_engine", "internal_core_dependency"),
}


def edge_keys(edges):
    return {(edge["source"], edge["target"], edge["label"]) for edge in edges}


class ModuleBoundaryContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.contract = json.loads(CONTRACT_PATH.read_text(encoding="utf-8"))

    def test_contract_declares_contract_only_scope_and_e02_binding(self):
        self.assertEqual(self.contract["contract_id"], "e0.1.module_boundary")
        self.assertEqual(self.contract["contract_version"], 1)
        self.assertFalse(self.contract["claim_limits"]["cargo_graph_checked"])
        self.assertFalse(self.contract["claim_limits"]["runtime_api_frozen"])
        self.assertFalse(self.contract["claim_limits"]["crate_existence_asserted"])
        self.assertEqual(self.contract["owner_labels"], EXPECTED_OWNER_LABELS)
        self.assertEqual(self.contract["claim_limits"]["binding_milestone"], "E0.2")
        handoff = self.contract["e0_2_handoff"]
        self.assertTrue(handoff["required"])
        self.assertTrue(handoff["bind_contract_to_actual_cargo_packages"])
        self.assertTrue(handoff["verify_actual_dependency_edges"])
        self.assertTrue(handoff["check_forbidden_edges_against_manifests"])
        self.assertTrue(handoff["freeze_actual_public_api_and_runner_paths"])
        self.assertIn("does not establish", handoff["acceptance_statement"])

    def test_module_ids_are_exact_unique_and_have_expected_owners(self):
        modules = self.contract["modules"]
        ids = [module["id"] for module in modules]
        self.assertEqual(len(ids), len(set(ids)))
        self.assertEqual(set(ids), set(EXPECTED_MODULES))
        self.assertEqual(
            {module["id"]: module["owner"] for module in modules}, EXPECTED_MODULES
        )
        self.assertEqual(
            {module["owner"] for module in modules}, EXPECTED_OWNERS
        )
        self.assertTrue(all(module["role"].strip() for module in modules))

    def test_every_permitted_edge_and_owner_label_is_exact(self):
        edges = self.contract["permitted_edges"]
        self.assertEqual(edge_keys(edges), EXPECTED_PERMITTED_EDGES)
        self.assertEqual(len(edges), len(EXPECTED_PERMITTED_EDGES))
        owners = EXPECTED_MODULES
        for edge in edges:
            with self.subTest(edge=edge):
                self.assertEqual(edge["source_owner"], owners[edge["source"]])
                self.assertEqual(edge["target_owner"], owners[edge["target"]])
                self.assertTrue(edge["reason"].strip())

    def test_every_forbidden_edge_and_owner_label_is_exact(self):
        edges = self.contract["forbidden_edges"]
        self.assertEqual(edge_keys(edges), EXPECTED_FORBIDDEN_EDGES)
        self.assertEqual(len(edges), len(EXPECTED_FORBIDDEN_EDGES))
        owners = EXPECTED_MODULES
        for edge in edges:
            with self.subTest(edge=edge):
                self.assertEqual(edge["source_owner"], owners[edge["source"]])
                self.assertEqual(edge["target_owner"], owners[edge["target"]])
                self.assertTrue(edge["reason"].strip())

    def test_edges_reference_declared_modules_and_use_distinct_sets(self):
        module_ids = set(EXPECTED_MODULES)
        permitted = edge_keys(self.contract["permitted_edges"])
        forbidden = edge_keys(self.contract["forbidden_edges"])
        self.assertTrue(permitted.isdisjoint(forbidden))
        for edge in self.contract["permitted_edges"] + self.contract["forbidden_edges"]:
            self.assertIn(edge["source"], module_ids)
            self.assertIn(edge["target"], module_ids)


if __name__ == "__main__":
    unittest.main()
