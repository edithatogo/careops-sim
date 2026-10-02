import hashlib
import json
import math
import random
import struct
import sys
import unittest
from pathlib import Path


FIXTURE_PATH = (
    Path(__file__).resolve().parents[1]
    / "model-inputs/ed/calibration/p31-known-distribution-fixtures.json"
)


def generated_log_values(seed, n, mu, sigma):
    """Generate Box-Muller normals with CPython's documented random stream."""
    rng = random.Random(seed)
    standard_normals = []
    while len(standard_normals) < n:
        u1 = rng.random()
        u2 = rng.random()
        radius = math.sqrt(-2.0 * math.log(u1))
        angle = 2.0 * math.pi * u2
        standard_normals.append(radius * math.cos(angle))
        standard_normals.append(radius * math.sin(angle))
    return [mu + sigma * z for z in standard_normals[:n]]


def digest_log_values(values):
    encoded = b"".join(struct.pack(">d", value) for value in values)
    return hashlib.sha256(encoded).hexdigest()


class KnownDistributionFixtureTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.fixture = json.loads(FIXTURE_PATH.read_text())
        cls.case = cls.fixture["case"]
        cls.truth = cls.case["truth"]
        self_generator = cls.case["generator"]
        cls.log_values = generated_log_values(
            self_generator["seed"],
            cls.case["sample_size"],
            cls.truth["mu_log_duration"],
            cls.truth["sigma_log_duration"],
        )

    def test_fixture_provenance_and_scope_are_synthetic_only(self):
        provenance = self.fixture["provenance"]
        self.assertEqual(self.fixture["schema_version"], 1)
        self.assertEqual(provenance["class"], "synthetic_only")
        self.assertFalse(provenance["empirical_data_used"])
        self.assertFalse(provenance["promoted_to_ed_parameter_or_default"])
        self.assertFalse(provenance["distribution_selection_claim"])
        self.assertEqual(self.case["reference_oracle"]["status"], "synthetic_fixture_only_not_empirical_fit")
        self.assertIn("P3.2", " ".join(self.fixture["limits"]))

    def test_generator_contract_and_same_runtime_repeatability(self):
        generator = self.case["generator"]
        self.assertEqual(generator["random_stream"], "random.Random MT19937 random()")
        self.assertEqual(generator["normal_transform"], "Box-Muller; consume u1,u2 pairs; cosine output then sine output")
        self.assertEqual(generator["precision"], "binary64")
        self.assertTrue(generator["runtime"].startswith("CPython "))
        repeated = generated_log_values(
            generator["seed"], self.case["sample_size"],
            self.truth["mu_log_duration"], self.truth["sigma_log_duration"],
        )
        digest = digest_log_values(self.log_values)
        self.assertEqual(repeated, self.log_values)
        # libm implementations can differ across runtime/platform versions.
        # The stored digest is an exact reference only on its recorded runtime.
        if generator["runtime"] == f"CPython {sys.version.split()[0]}":
            self.assertEqual(digest, generator["log_values_sha256_big_endian_binary64"])

    def test_reference_recovery_oracle_and_prefrozen_limits(self):
        oracle = self.case["reference_oracle"]
        n = len(self.log_values)
        mu_hat = sum(self.log_values) / n
        sigma_hat = math.sqrt(sum((x - mu_hat) ** 2 for x in self.log_values) / n)
        self.assertEqual(oracle["mu_hat"], "mean(log(duration))")
        self.assertEqual(oracle["sigma_hat"], "sqrt(sum((log(duration)-mu_hat)^2)/n)")
        self.assertAlmostEqual(oracle["absolute_error_limits"]["mu"], 3 * 0.6 / math.sqrt(4096), places=12)
        sigma_three_se = 3 * 0.6 / math.sqrt(2 * 4096)
        # The packet froze a slightly stricter sigma limit; don't
        # widen it after observing the generated sample.
        self.assertEqual(oracle["absolute_error_limits"]["sigma"], 0.019882)
        self.assertGreater(sigma_three_se, oracle["absolute_error_limits"]["sigma"])
        self.assertIn("0.019887378220871645", oracle["limit_basis"])
        self.assertLessEqual(abs(mu_hat - self.truth["mu_log_duration"]), oracle["absolute_error_limits"]["mu"])
        self.assertLessEqual(abs(sigma_hat - self.truth["sigma_log_duration"]), oracle["absolute_error_limits"]["sigma"])
        self.assertIn("not empirical", oracle["limit_basis"].lower())

    def test_positive_unbounded_support_and_no_tail_clipping(self):
        support = self.case["support"]
        durations = [math.exp(x) for x in self.log_values]
        self.assertFalse(support["lower_bound_inclusive"])
        self.assertEqual(support["duration_lower_bound"], 0)
        self.assertIsNone(support["duration_upper_bound"])
        self.assertTrue(support["upper_bound_unbounded"])
        self.assertEqual(support["truncation"], "none")
        self.assertEqual(support["clipping"], "none")
        self.assertTrue(all(math.isfinite(t) and t > 0 for t in durations))
        for item in support["boundary_cases"]:
            expected = "inside_support" if item["duration"] > 0 else "outside_support"
            self.assertEqual(item["expected"], expected)
        self.assertEqual(len(support["boundary_cases"]), 4)
        tail = self.case["tail_oracle"]
        self.assertEqual(tail["expected_count_min_inclusive"], 21)
        self.assertEqual(tail["expected_count_max_inclusive"], 61)
        self.assertIn("Binomial(n=4096, p=0.01)", tail["count_range_basis"])
        self.assertIn("not an inferential", tail["count_range_basis"])
        count = sum((x - self.truth["mu_log_duration"]) / self.truth["sigma_log_duration"] > tail["strict_cutoff"] for x in self.log_values)
        self.assertGreaterEqual(count, tail["expected_count_min_inclusive"])
        self.assertLessEqual(count, tail["expected_count_max_inclusive"])

    def test_sparse_states_are_fixture_only_not_a_real_fit_threshold(self):
        states = {item["sample_size"]: item["classification"] for item in self.fixture["sparse_support_states"]}
        self.assertEqual(states, {
            0: "insufficient_data_no_fit",
            1: "insufficient_data_no_fit",
            29: "insufficient_data_no_fit",
            30: "fixture_eligible_only",
        })
        self.assertIn("fixture-only", " ".join(self.fixture["limits"]))


if __name__ == "__main__":
    unittest.main()
