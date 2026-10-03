"""Synthetic-only P3.2 candidate comparison; no ED observations are used."""

import hashlib
import json
import math
import random
import struct
import unittest
from unittest import mock
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SOURCE_PATH = ROOT / "model-inputs/ed/calibration/p31-known-distribution-fixtures.json"
OUTPUT_PATH = ROOT / "model-inputs/ed/calibration/p32-candidate-comparison.synthetic.json"


CANONICAL_PATH = ROOT / "model-inputs/ed/calibration/p32-canonical-durations.synthetic.json"
CANONICAL_SHA256 = "225b06f6ea0dce6228cd1982454d6604e0e407a9838c3de1f8d985f201ceae31"


def canonical_durations(source):
    raw = CANONICAL_PATH.read_bytes()
    if hashlib.sha256(raw).hexdigest() != CANONICAL_SHA256:
        raise ValueError("canonical duration fixture hash drift")
    frozen = json.loads(raw)
    if frozen["source_sha256"] != hashlib.sha256(SOURCE_PATH.read_bytes()).hexdigest() or frozen["source_fixture"] != source["fixture_id"] or frozen["generator"] != source["case"]["generator"]:
        raise ValueError("canonical duration lineage drift")
    durations = [float.fromhex(value) for value in frozen["duration_hex"]]
    if len(durations) != source["case"]["sample_size"] or any(not math.isfinite(value) or value <= 0 for value in durations):
        raise ValueError("invalid canonical durations")
    if digest_durations(durations) != frozen["binary64_sha256_big_endian"]:
        raise ValueError("canonical binary64 input drift")
    train = [durations[i] for i in frozen_split(len(durations))["train"]]
    if digest_durations(train) != frozen["training_binary64_sha256_big_endian"]:
        raise ValueError("canonical training input drift")
    return durations


def generated_log_values(seed, n, mu, sigma):
    """Match the accepted P3.1 Box-Muller generator contract."""
    rng = random.Random(seed)
    values = []
    while len(values) < n:
        u1, u2 = rng.random(), rng.random()
        radius = math.sqrt(-2.0 * math.log(u1))
        angle = 2.0 * math.pi * u2
        values.extend((mu + sigma * radius * math.cos(angle),
                       mu + sigma * radius * math.sin(angle)))
    return values[:n]


def digest_bytes(data):
    return hashlib.sha256(data).hexdigest()


def digest_indices(indices):
    encoded = json.dumps(indices, separators=(",", ":")).encode("ascii")
    return digest_bytes(encoded)


def digest_durations(values):
    return digest_bytes(b"".join(struct.pack(">d", value) for value in values))


def frozen_split(n=4096, seed=20261003):
    indices = list(range(n))
    random.Random(seed).shuffle(indices)
    return {
        "train": indices[:3072],
        "selection": indices[3072:3584],
        "locked_test": indices[3584:],
    }


def quantile(values, probability):
    ordered = sorted(values)
    position = (len(ordered) - 1) * probability
    lower = math.floor(position)
    upper = math.ceil(position)
    if lower == upper:
        return ordered[lower]
    return ordered[lower] + (position - lower) * (ordered[upper] - ordered[lower])


def make_candidates(training_records, truth_mu, truth_sigma, tail_cutoff,
                    resample_seed=20261004, bootstrap_replicates=1000):
    """Consume only (training index, duration) pairs and return diagnostics."""
    indices = [index for index, _ in training_records]
    durations = [value for _, value in training_records]
    if not durations or any(not math.isfinite(x) or x <= 0 for x in durations):
        raise ValueError("lognormal candidates require finite positive training durations")
    logs = [math.log(x) for x in durations]
    n = len(logs)
    mu_hat = sum(logs) / n
    sigma_hat = math.sqrt(sum((x - mu_hat) ** 2 for x in logs) / n)
    if sigma_hat <= 0:
        raise ValueError("lognormal MLE requires nonzero training variation")
    log_likelihood = sum(
        -math.log(x) - math.log(sigma_hat) - 0.5 * math.log(2.0 * math.pi)
        - ((math.log(x) - mu_hat) ** 2) / (2.0 * sigma_hat**2)
        for x in durations
    )

    rng = random.Random(resample_seed)
    resampled = [durations[rng.randrange(n)] for _ in range(n)]
    empirical_tail_count = sum(
        (math.log(x) - truth_mu) / truth_sigma > tail_cutoff for x in durations
    )
    resampled_tail_count = sum(
        (math.log(x) - truth_mu) / truth_sigma > tail_cutoff for x in resampled
    )
    resampled_medians = []
    for _ in range(bootstrap_replicates):
        sample = [durations[rng.randrange(n)] for _ in range(n)]
        resampled_medians.append(quantile(sample, 0.5))
    resampled_medians.sort()
    return {
        "input_index_digest_sha256_json_ascii": digest_indices(indices),
        "input_duration_digest_sha256_big_endian_binary64": digest_durations(durations),
        "sample_size": n,
        "units": "fixture duration units (unchanged; not an ED unit claim)",
        "parametric_lognormal_mle": {
            "fit_method": "mu=mean(log(x)); sigma=sqrt(sum((log(x)-mu)^2)/n), population divisor n",
            "sample_size": n,
            "mu_log_duration": mu_hat,
            "sigma_log_duration": sigma_hat,
            "support": {"lower": 0, "lower_inclusive": False, "upper": None, "upper_unbounded": True},
            "log_likelihood_training_only": log_likelihood,
            "known_truth_diagnostic": {
                "truth_mu_log_duration": truth_mu,
                "truth_sigma_log_duration": truth_sigma,
                "absolute_mu_error": abs(mu_hat - truth_mu),
                "absolute_sigma_error": abs(sigma_hat - truth_sigma),
                "scope": "synthetic generator truth; diagnostic only, no held-out data",
            },
            "training_quantiles": {f"p{int(p*100)}": quantile(durations, p) for p in (0.5, 0.9, 0.95, 0.99)},
            "training_tail_count_above_known_truth_z_cutoff": empirical_tail_count,
        },
        "empirical_resampling_baseline": {
            "method": "sample with replacement from the exact training empirical distribution",
            "resample_seed": resample_seed,
            "bootstrap_replicates": bootstrap_replicates,
            "support": {"lower_observed": min(durations), "upper_observed": max(durations), "discrete_atoms": n},
            "resample_sample_size": len(resampled),
            "resample_quantiles": {f"p{int(p*100)}": quantile(resampled, p) for p in (0.5, 0.9, 0.95, 0.99)},
            "training_tail_count_above_known_truth_z_cutoff": empirical_tail_count,
            "one_resample_tail_count_above_known_truth_z_cutoff": resampled_tail_count,
            "bootstrap_median_interval_percentiles": {
                "p2_5": quantile(resampled_medians, 0.025),
                "p97_5": quantile(resampled_medians, 0.975),
            },
            "scope": "synthetic resampling diagnostics; not a fitted continuous likelihood",
        },
    }


def build_output():
    source = json.loads(SOURCE_PATH.read_text())
    case = source["case"]
    truth = case["truth"]
    durations = canonical_durations(source)
    split = frozen_split(len(durations))
    # Only this projection of row data enters candidate calculations. The
    # selection and locked-test members are represented by indices/digests only.
    training_records = [(i, durations[i]) for i in split["train"]]
    candidates = make_candidates(
        training_records, truth["mu_log_duration"], truth["sigma_log_duration"],
        case["tail_oracle"]["strict_cutoff"],
    )
    partitions = {
        name: {"size": len(indices), "index_membership_sha256_json_ascii": digest_indices(indices)}
        for name, indices in split.items()
    }
    return {
        "schema_version": 1,
        "fixture_id": "P3.2.synthetic-lognormal-vs-empirical-resampling.v1",
        "provenance": {
            "class": "synthetic_only",
            "empirical_data_used": False,
            "ed_parameter_or_default": False,
            "family_selection_or_winner_claim": False,
            "candidate_promotion": False,
            "limitations": [
                "The generated lognormal truth is known by construction; successful recovery cannot validate an empirical ED family or parameters.",
                "Empirical-distribution support and quantile/bootstrap diagnostics are not comparable likelihood scores to a continuous parametric density.",
                "The parametric training log likelihood is in-sample and is not a validation score.",
                "A finite empirical resample cannot represent the lognormal candidate's unbounded upper support; conversely the parametric family imposes smooth lognormal tail shape.",
                "Censoring, clustering, dependence, nonstationarity, ED unit mapping, and operational applicability are not assessed here.",
            ],
        },
        "lineage": {
            "accepted_input_fixture": source["fixture_id"],
            "input_fixture_sha256": hashlib.sha256(SOURCE_PATH.read_bytes()).hexdigest(),
            "generator_seed": case["generator"]["seed"],
            "generator_contract": case["generator"]["normal_transform"],
            "duration_transform": "duration = exp(log_duration)",
            "report_12": "conductor/research/supplied/20260927/report-12.md.txt; primitive fitting distinguishes parametric and empirical options and requires training/selection/locked-test separation; this output is a synthetic method check only.",
            "integration_note": "conductor/research/ed-research-incorporation-9-12-20260927.md keeps these as synthetic fixtures and requires empirical applicability to remain open.",
        },
        "split_contract": {
            "algorithm": "indices=list(range(4096)); random.Random(20261003).shuffle(indices); first 3072 train, next 512 selection, final 512 locked_test",
            "seed": 20261003,
            "design_class": "predeclared synthetic design choice; report-12 motivates separation but does not prescribe this random split",
            "partitions": partitions,
            "candidate_calculation_access": "training row values only; selection/test row values are not projected into any candidate calculation",
            "reuse_for_p32_holdout": "Reuse this exact index permutation and memberships; freeze any later selection/test access plan before opening those rows.",
        },
        "units": "fixture duration units; same unchanged units for both candidates; no ED mapping asserted",
        "candidates": candidates,
        "metric_comparability": {
            "parametric_log_likelihood": "continuous lognormal density evaluated on training rows only; unit-dependent and in-sample",
            "empirical_resampling": "discrete empirical distribution diagnostics; no continuous log likelihood reported",
            "comparison_rule": "Empirical-distribution diagnostics are not comparable likelihood scores to the continuous parametric density; do not declare a preferred candidate from these synthetic diagnostics.",
        },
        "rejected_or_limited_alternative": {
            "alternative": "Fit either candidate or choose family using selection/test rows in this leaf",
            "disposition": "deferred",
            "rationale": "This packet's candidate leaf is train-only; selection and locked-test members are reserved for the separately bound P3.2 holdout work.",
            "evidence_class": "packet contract and predeclared synthetic design boundary",
        },
    }


class SyntheticCandidateComparisonTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.source = json.loads(SOURCE_PATH.read_text())
        cls.output = json.loads(OUTPUT_PATH.read_text())
        case = cls.source["case"]
        truth = case["truth"]
        cls.durations = canonical_durations(cls.source)
        cls.split = frozen_split(len(cls.durations))

    def test_canonical_duration_source_is_bound_to_accepted_receipts(self):
        frozen = json.loads(CANONICAL_PATH.read_text())
        train = [self.durations[i] for i in self.split["train"]]
        self.assertEqual(frozen["training_binary64_sha256_big_endian"], self.output["candidates"]["input_duration_digest_sha256_big_endian_binary64"])
        self.assertEqual(digest_durations(train), frozen["training_binary64_sha256_big_endian"])
        with mock.patch.object(CANONICAL_PATH.__class__, "read_bytes", return_value=b"{}"):
            with self.assertRaisesRegex(ValueError, "hash drift"):
                canonical_durations(self.source)

    def test_split_is_frozen_disjoint_and_complete(self):
        split = self.split
        self.assertEqual([len(split[k]) for k in ("train", "selection", "locked_test")], [3072, 512, 512])
        self.assertEqual(len(set(split["train"]) | set(split["selection"]) | set(split["locked_test"])), 4096)
        self.assertFalse(set(split["train"]) & set(split["selection"]))
        self.assertFalse(set(split["train"]) & set(split["locked_test"]))
        self.assertEqual(self.output["split_contract"]["partitions"]["train"]["index_membership_sha256_json_ascii"], digest_indices(split["train"]))
        self.assertEqual(self.output["split_contract"]["partitions"]["selection"]["index_membership_sha256_json_ascii"], digest_indices(split["selection"]))
        self.assertEqual(self.output["split_contract"]["partitions"]["locked_test"]["index_membership_sha256_json_ascii"], digest_indices(split["locked_test"]))

    def test_candidate_calculation_receives_training_rows_only(self):
        train = self.split["train"]
        selection = self.split["selection"]
        locked_test = self.split["locked_test"]
        records = [(i, self.durations[i]) for i in train]
        received_indices = [i for i, _ in records]
        self.assertEqual(received_indices, train)
        self.assertFalse(set(received_indices) & set(selection))
        self.assertFalse(set(received_indices) & set(locked_test))
        candidates = make_candidates(records, 2.5, 0.6, 2.326347874)
        self.assertEqual(candidates["input_index_digest_sha256_json_ascii"], digest_indices(train))
        self.assertEqual(candidates["sample_size"], len(train))

    def test_same_units_same_rows_and_repeatable_diagnostics(self):
        records = [(i, self.durations[i]) for i in self.split["train"]]
        one = make_candidates(records, 2.5, 0.6, 2.326347874)
        two = make_candidates(records, 2.5, 0.6, 2.326347874)
        self.assertEqual(one, two)
        self.assertEqual(one["input_duration_digest_sha256_big_endian_binary64"], digest_durations([x for _, x in records]))
        self.assertEqual(one["parametric_lognormal_mle"]["sample_size"], one["empirical_resampling_baseline"]["resample_sample_size"])
        self.assertEqual(one["units"], "fixture duration units (unchanged; not an ED unit claim)")
        self.assertEqual(self.output["units"], "fixture duration units; same unchanged units for both candidates; no ED mapping asserted")
        self.assertEqual(one, self.output["candidates"])

    def test_synthetic_only_and_no_winner_or_holdout_claim(self):
        provenance = self.output["provenance"]
        self.assertEqual(provenance["class"], "synthetic_only")
        self.assertFalse(provenance["empirical_data_used"])
        self.assertFalse(provenance["ed_parameter_or_default"])
        self.assertFalse(provenance["family_selection_or_winner_claim"])
        self.assertFalse(provenance["candidate_promotion"])
        self.assertIn("selection and locked-test members", self.output["rejected_or_limited_alternative"]["rationale"])
        self.assertIn("not comparable likelihood", self.output["metric_comparability"]["comparison_rule"])
        self.assertIn("report-12.md.txt", self.output["lineage"]["report_12"])


if __name__ == "__main__":
    unittest.main()
