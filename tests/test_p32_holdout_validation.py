import hashlib
import json
import math
import random
import struct
import unittest
from pathlib import Path
from statistics import NormalDist


ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "model-inputs/ed/calibration/p31-known-distribution-fixtures.json"
CANDIDATES = ROOT / "model-inputs/ed/calibration/p32-candidate-comparison.synthetic.json"
OUTPUT = ROOT / "model-inputs/ed/calibration/p32-holdout-validation.synthetic.json"
QUANTILES = (0.5, 0.9, 0.95, 0.99)
CANDIDATE_ORDER = ("parametric_lognormal_mle", "empirical_resampling_baseline")


def generated_durations(fixture):
    case = fixture["case"]
    truth = case["truth"]
    rng = random.Random(case["generator"]["seed"])
    logs = []
    while len(logs) < case["sample_size"]:
        u1, u2 = rng.random(), rng.random()
        radius = math.sqrt(-2.0 * math.log(u1))
        angle = 2.0 * math.pi * u2
        logs.extend((truth["mu_log_duration"] + truth["sigma_log_duration"] * radius * math.cos(angle),
                     truth["mu_log_duration"] + truth["sigma_log_duration"] * radius * math.sin(angle)))
    return [math.exp(value) for value in logs[:case["sample_size"]]]


def frozen_split(n=4096, seed=20261003):
    indices = list(range(n))
    random.Random(seed).shuffle(indices)
    return {"train": indices[:3072], "selection": indices[3072:3584], "locked_test": indices[3584:]}


def digest_indices(indices):
    return hashlib.sha256(json.dumps(indices, separators=(",", ":")).encode("ascii")).hexdigest()


def digest_durations(values):
    return hashlib.sha256(b"".join(struct.pack(">d", x) for x in values)).hexdigest()


def quantile(values, p):
    ordered = sorted(values)
    position = (len(ordered) - 1) * p
    lo, hi = math.floor(position), math.ceil(position)
    return ordered[lo] if lo == hi else ordered[lo] + (position - lo) * (ordered[hi] - ordered[lo])


def score(predicted, observed):
    return sum(abs(predicted[f"p{int(p * 100)}"] - quantile(observed, p)) for p in QUANTILES)


def predicted_quantiles(candidate, training_values):
    if candidate == "parametric_lognormal_mle":
        fit = CANDIDATE_RECEIPT["candidates"][candidate]
        mu, sigma = fit["mu_log_duration"], fit["sigma_log_duration"]
        return {f"p{int(p * 100)}": math.exp(mu + sigma * NormalDist().inv_cdf(p)) for p in QUANTILES}
    return {f"p{int(p * 100)}": quantile(training_values, p) for p in QUANTILES}


def selection_scores(selection_durations, training_values):
    return {candidate: score(predicted_quantiles(candidate, training_values), selection_durations)
            for candidate in CANDIDATE_ORDER}


def select_candidate(scores):
    return min(CANDIDATE_ORDER, key=lambda name: (scores[name], CANDIDATE_ORDER.index(name)))


def locked_test_diagnostics(selected, test_durations, training_values):
    if selected not in CANDIDATE_ORDER:
        raise ValueError("selected candidate must be frozen before locked-test evaluation")
    diagnostics = {name: score(predicted_quantiles(name, training_values), test_durations) for name in CANDIDATE_ORDER}
    selected_score = diagnostics[selected]
    rng = random.Random(20261005)
    bootstrap = [score(predicted_quantiles(selected, training_values),
                       [test_durations[rng.randrange(len(test_durations))] for _ in test_durations])
                 for _ in range(1000)]
    bootstrap.sort()
    return diagnostics, selected_score, {
        "confidence_level": 0.95,
        "method": "percentile bootstrap of locked-test rows; selected candidate predictions held fixed",
        "replicates": 1000,
        "seed": 20261005,
        "scope": "conditional sampling uncertainty for this synthetic test block only; excludes parameter, data, and ED uncertainty",
        "lower": quantile(bootstrap, 0.025),
        "upper": quantile(bootstrap, 0.975),
    }


def build_output():
    global CANDIDATE_RECEIPT
    fixture = json.loads(FIXTURE.read_text())
    CANDIDATE_RECEIPT = json.loads(CANDIDATES.read_text())
    durations = generated_durations(fixture)
    split = frozen_split(len(durations))
    train = [durations[i] for i in split["train"]]
    selection = [durations[i] for i in split["selection"]]
    # Candidate parameter estimates and empirical quantiles were frozen on train rows.
    selected_scores = selection_scores(selection, train)
    selected = select_candidate(selected_scores)
    # This is the sole test opening point; selection and predictions are already frozen.
    locked_test = [durations[i] for i in split["locked_test"]]
    test_scores, selected_test_score, interval = locked_test_diagnostics(selected, locked_test, train)
    return {
        "schema_version": 1,
        "fixture_id": "P3.2.synthetic-lognormal-heldout-validation.v1",
        "provenance": {"class": "synthetic_only", "empirical_data_used": False,
                       "ed_parameter_or_default": False, "candidate_promotion": False,
                       "limitations": ["No empirical ED data were used.", "The selection score and split are synthetic design choices, not universal criteria.", "No result is transferable to an operating ED or a real parameter choice."]},
        "lineage": {"accepted_fixture": fixture["fixture_id"],
                    "accepted_candidate_fixture": CANDIDATE_RECEIPT["fixture_id"],
                    "generator_seed": fixture["case"]["generator"]["seed"],
                    "split_seed": 20261003,
                    "candidate_artifact_sha256": hashlib.sha256(CANDIDATES.read_bytes()).hexdigest(),
                    "report_12_method_lineage": {
                        "source": "conductor/research/supplied/20260927/report-12.md.txt:20-30",
                        "stages": {
                            "data_audit": "Classify fields by role and record units, provenance, censor/event state, missingness, and temporal availability.",
                            "primitive_fitting": "Fit primitive distributions from directly observed intervals; compare prespecified parametric and empirical options when supported.",
                            "validation_selection": "Use a later block to select prespecified candidates by unclamped behavior, with adequacy depending on targets, tolerances, and data.",
                            "locked_test": "Freeze model and comparison choices, open the test block once, and report without retuning.",
                        },
                        "uncertainty_distinction": "Report-12 distinguishes repeated-run simulation stochasticity from input-model/parameter uncertainty assessed by resampling/refitting input data and rerunning.",
                        "packet_design_boundary": "The random split, quantile score, and test-block bootstrap specified here are this packet's synthetic design choices; report-12 motivates stage separation and uncertainty distinction but does not prescribe these exact methods.",
                    }},
        "split": {name: {"size": len(indices), "index_membership_sha256_json_ascii": digest_indices(indices)}
                  for name, indices in split.items()},
        "training": {"rows_used_for_candidate_fits": len(train),
                     "index_membership_sha256_json_ascii": digest_indices(split["train"]),
                     "duration_digest_sha256_big_endian_binary64": digest_durations(train)},
        "selection": {"rows": len(selection), "candidate_scores_sum_absolute_quantile_errors": selected_scores,
                      "quantile_probabilities": list(QUANTILES), "quantile_method": "linear interpolation",
                      "selection_rule": "minimum sum absolute difference from observed selection p50,p90,p95,p99; tie order: parametric_lognormal_mle then empirical_resampling_baseline",
                      "selected_candidate": selected,
                      "design_choice": "Predeclared synthetic selection rule; not prescribed by report-12 and not an ED family-selection criterion."},
        "locked_test": {"opened_once_after_selection": True, "rows": len(locked_test),
                        "index_membership_sha256_json_ascii": digest_indices(split["locked_test"]),
                        "candidate_scores_sum_absolute_quantile_errors_descriptive_only": test_scores,
                        "selected_candidate": selected, "selected_score": selected_test_score,
                        "selection_changed_after_test": False,
                        "uncertainty": interval},
        "units": "unchanged synthetic fixture duration units; no ED unit mapping asserted",
    }


CANDIDATE_RECEIPT = {}


class SyntheticHoldoutValidationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.fixture = json.loads(FIXTURE.read_text())
        cls.candidates = json.loads(CANDIDATES.read_text())
        cls.output = json.loads(OUTPUT.read_text())
        cls.durations = generated_durations(cls.fixture)
        cls.split = frozen_split(len(cls.durations))
        global CANDIDATE_RECEIPT
        CANDIDATE_RECEIPT = cls.candidates

    def test_frozen_generator_and_split_integrity(self):
        self.assertEqual([len(self.split[k]) for k in ("train", "selection", "locked_test")], [3072, 512, 512])
        self.assertEqual(len(set(sum(self.split.values(), []))), 4096)
        for name, indices in self.split.items():
            self.assertEqual(self.output["split"][name]["index_membership_sha256_json_ascii"], digest_indices(indices))
        train = [self.durations[i] for i in self.split["train"]]
        self.assertEqual(self.output["training"]["duration_digest_sha256_big_endian_binary64"], digest_durations(train))

    def test_selection_rule_uses_selection_rows_and_freezes_id(self):
        selection = [self.durations[i] for i in self.split["selection"]]
        train = [self.durations[i] for i in self.split["train"]]
        scores = selection_scores(selection, train)
        selected = select_candidate(scores)
        self.assertEqual(scores, self.output["selection"]["candidate_scores_sum_absolute_quantile_errors"])
        self.assertEqual(selected, self.output["selection"]["selected_candidate"])
        self.assertEqual(selected, self.output["locked_test"]["selected_candidate"])
        self.assertFalse(self.output["locked_test"]["selection_changed_after_test"])

    def test_changing_locked_test_values_cannot_change_selected_id(self):
        train_indices = self.split["train"]
        selection_indices = self.split["selection"]
        locked_indices = self.split["locked_test"]
        train = [self.durations[i] for i in train_indices]
        selection = [self.durations[i] for i in selection_indices]
        selected_before = select_candidate(selection_scores(selection, train))
        changed_durations = list(self.durations)
        for index in locked_indices:
            changed_durations[index] *= 1000.0
        changed_train = [changed_durations[i] for i in train_indices]
        changed_selection = [changed_durations[i] for i in selection_indices]
        selected_after = select_candidate(selection_scores(changed_selection, changed_train))
        self.assertEqual(selected_before, selected_after)
        self.assertEqual(selected_after, self.output["selection"]["selected_candidate"])

    def test_test_scores_are_once_descriptive_and_bootstrap_repeats(self):
        chosen = self.output["selection"]["selected_candidate"]
        locked = [self.durations[i] for i in self.split["locked_test"]]
        train = [self.durations[i] for i in self.split["train"]]
        first = locked_test_diagnostics(chosen, locked, train)
        second = locked_test_diagnostics(chosen, locked, train)
        self.assertEqual(first, second)
        test = self.output["locked_test"]
        self.assertTrue(test["opened_once_after_selection"])
        self.assertEqual(first[0], test["candidate_scores_sum_absolute_quantile_errors_descriptive_only"])
        self.assertEqual(first[2], test["uncertainty"])
        self.assertEqual(test["uncertainty"]["replicates"], 1000)
        self.assertIn("synthetic test block only", test["uncertainty"]["scope"])

    def test_synthetic_scope_and_no_transfer_claim(self):
        provenance = self.output["provenance"]
        self.assertEqual(provenance["class"], "synthetic_only")
        self.assertFalse(provenance["empirical_data_used"])
        self.assertFalse(provenance["ed_parameter_or_default"])
        self.assertFalse(provenance["candidate_promotion"])
        self.assertTrue(all(word in " ".join(provenance["limitations"]).lower()
                            for word in ("no empirical ed data", "universal criteria", "transferable")))
        method = self.output["lineage"]["report_12_method_lineage"]
        self.assertIn("report-12.md.txt:20-30", method["source"])
        self.assertEqual(set(method["stages"]), {"data_audit", "primitive_fitting", "validation_selection", "locked_test"})
        self.assertIn("simulation stochasticity", method["uncertainty_distinction"])
        self.assertIn("packet's synthetic design choices", method["packet_design_boundary"])


if __name__ == "__main__":
    unittest.main()
