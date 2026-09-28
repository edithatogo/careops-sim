import copy
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import skill_eval


def row(case, arm, *, expected="ready_for_review", actual=None, bad=False,
        evaluable=True, reason=None, repeat=None, mode="serial", size=12,
        corrections=0, candidate="fixture-review", agent=None):
    return {
        "case_id": case, "arm": arm, "candidate_skill": candidate,
        "dispatch_mode": mode, "expected_disposition": expected if evaluable else None,
        "actual_disposition": actual or expected, "known_bad": bad if evaluable else None,
        "evaluable": evaluable, "exclusion_reason": reason if not evaluable else None,
        "repeat_of": repeat, "agent_id": agent or f"{case}-{arm}",
        "model_route": "gpt-6-luna", "reasoning_effort": "low",
        "output_chars": size, "correction_turns": corrections,
    }


def fixture():
    return {"schema_version": 1, "results": [
        row("good", "baseline"), row("good", "skill"),
        row("bad", "baseline", expected="hold_for_evidence", bad=True, actual="ready_for_review"),
        row("bad", "skill", expected="hold_for_evidence", bad=True),
        row("repeat", "baseline", repeat="good", mode="parallel"),
        row("repeat", "skill", repeat="good", mode="parallel"),
        row("ambiguous", "baseline", evaluable=False, reason="prompt ambiguous"),
        row("ambiguous", "skill", evaluable=False, reason="prompt ambiguous"),
    ]}


class SkillEvalTests(unittest.TestCase):
    def test_metrics_pair_repeat_exclusion_dispatch_and_missing_observations(self):
        data = fixture()
        data["results"][0]["output_chars"] = None
        data["results"][1]["correction_turns"] = None
        result = skill_eval.summarize(skill_eval.validate(data))
        self.assertEqual(result["independent_case_count"], 3)
        self.assertEqual(result["repeated_case_count"], 1)
        self.assertEqual(result["excluded_case_count"], 1)
        self.assertEqual(result["arms"]["baseline"]["acceptance_accuracy"], {"numerator": 1, "denominator": 2, "rate": .5})
        self.assertEqual(result["arms"]["skill"]["acceptance_accuracy"]["rate"], 1)
        self.assertEqual(result["paired_case_correctness"], {"numerator": 1, "denominator": 2, "rate": .5})
        self.assertEqual(result["arms"]["baseline"]["known_bad_accepted_per_known_bad_response"]["denominator"], 1)
        self.assertEqual(result["arms"]["baseline"]["known_bad_accepted_per_all_accepted"]["numerator"], 1)
        self.assertEqual(result["dispatch_modes"]["parallel"]["evaluable_responses"], 0)
        self.assertIsNone(result["dispatch_modes"]["parallel"]["acceptance_accuracy"]["rate"])
        self.assertEqual(result["by_arm_dispatch_mode"]["skill"]["parallel"]["responses"], 0)
        self.assertIsNone(result["arms"]["baseline"]["output_chars"]["aggregate"])
        self.assertIsNone(result["arms"]["skill"]["correction_turns"]["aggregate"])
        self.assertEqual(result["excluded_cases"][0]["reason"], "prompt ambiguous")
        self.assertEqual(result["repeated_cases"][0]["repeat_of"], "good")
        candidate = result["by_candidate_skill"]["fixture-review"]
        self.assertEqual(candidate["independent_case_count"], 3)
        self.assertEqual(candidate["paired_case_correctness"], {"numerator": 1, "denominator": 2, "rate": .5})
        self.assertEqual(candidate["dispatch_modes"]["serial"]["arms"]["baseline"]["acceptance_accuracy"]["denominator"], 2)
        self.assertEqual(candidate["dispatch_modes"]["parallel"]["arms"]["baseline"]["responses"], 0)

    def test_metrics_are_separated_by_candidate_skill(self):
        results = fixture()["results"][:2]
        results += [
            row("other", "baseline", expected="revise", actual="hold_for_evidence", candidate="calibration-review"),
            row("other", "skill", expected="revise", actual="revise", candidate="calibration-review"),
        ]
        summary = skill_eval.summarize(skill_eval.validate({"schema_version": 1, "results": results}))
        self.assertEqual(summary["by_candidate_skill"]["fixture-review"]["arms"]["baseline"]["acceptance_accuracy"]["rate"], 1)
        self.assertEqual(summary["by_candidate_skill"]["calibration-review"]["arms"]["baseline"]["acceptance_accuracy"]["rate"], 0)
        self.assertEqual(summary["by_candidate_skill"]["calibration-review"]["arms"]["skill"]["acceptance_accuracy"]["rate"], 1)

    def test_zero_bad_and_zero_acceptance_denominators_are_null(self):
        data = {"schema_version": 1, "results": [
            row("bad", "baseline", expected="hold_for_evidence", bad=True),
            row("bad", "skill", expected="hold_for_evidence", bad=True),
        ]}
        metrics = skill_eval.summarize(skill_eval.validate(data))["arms"]["baseline"]
        self.assertEqual(metrics["known_bad_accepted_per_known_bad_response"]["rate"], 0)
        self.assertIsNone(metrics["known_bad_accepted_per_all_accepted"]["rate"])
        self.assertEqual(metrics["acceptance_accuracy"]["rate"], 1)

    def test_all_excluded_has_null_accuracy_and_pair_denominators(self):
        results = [row("x", arm, evaluable=False, reason="ambiguous") for arm in ("baseline", "skill")]
        summary = skill_eval.summarize(skill_eval.validate({"schema_version": 1, "results": results}))
        self.assertIsNone(summary["overall"]["acceptance_accuracy"]["rate"])
        self.assertIsNone(summary["paired_case_correctness"]["rate"])
        self.assertEqual(summary["excluded_case_count"], 1)

    def test_invalid_pairs_and_fields_fail_closed(self):
        data = fixture()
        cases = []
        d = copy.deepcopy(data); d["results"].pop(); cases.append(d)
        d = copy.deepcopy(data); d["results"][1]["dispatch_mode"] = "parallel"; cases.append(d)
        d = copy.deepcopy(data); d["results"][0]["output_chars"] = True; cases.append(d)
        d = copy.deepcopy(data); d["results"][0]["expected_disposition"] = "accept"; cases.append(d)
        d = copy.deepcopy(data); d["results"][4]["repeat_of"] = "repeat"; d["results"][5]["repeat_of"] = "repeat"; cases.append(d)
        d = copy.deepcopy(data); d["results"][0]["known_bad"] = None; cases.append(d)
        d = copy.deepcopy(data); d["results"][6]["exclusion_reason"] = ""; cases.append(d)
        d = copy.deepcopy(data); d["results"][0]["arm"] = []; cases.append(d)
        d = copy.deepcopy(data); d["results"][0]["actual_disposition"] = []; cases.append(d)
        d = copy.deepcopy(data); d["results"][0]["expected_disposition"] = []; cases.append(d)
        d = copy.deepcopy(data); d["results"][4]["known_bad"] = True; cases.append(d)
        for item in cases:
            with self.subTest(item=item):
                with self.assertRaises(skill_eval.InputError):
                    skill_eval.validate(item)

    def test_cli_emits_json_and_actionable_nonzero_error(self):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "input.json"
            path.write_text(json.dumps(fixture()), encoding="utf-8")
            ok = subprocess.run([sys.executable, str(Path(skill_eval.__file__)), str(path)], capture_output=True, text=True)
            self.assertEqual(ok.returncode, 0, ok.stderr)
            self.assertEqual(json.loads(ok.stdout)["independent_case_count"], 3)
            path.write_text('{"schema_version":1,"results":[]}', encoding="utf-8")
            bad = subprocess.run([sys.executable, str(Path(skill_eval.__file__)), str(path)], capture_output=True, text=True)
            self.assertNotEqual(bad.returncode, 0)
            self.assertIn("non-empty array", bad.stderr)


if __name__ == "__main__":
    unittest.main()
