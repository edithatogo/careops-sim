import math
import unittest

from tools.quality_metrics import check_coverage


FILE = "crates/careops-ed/src/lib.rs"
POLICY = {"file": FILE, "minimum_line_percent": 80}


def document(*, filename=FILE, count=10, covered=8, percent=80, data_files=None,
             functions=None):
    summary = {"lines": {"count": count, "covered": covered, "percent": percent}}
    if functions is not None:
        summary["functions"] = functions
    files = data_files if data_files is not None else [{"filename": filename, "summary": summary}]
    return {"data": [{"files": files}]}


class CoverageMetricsTests(unittest.TestCase):
    def test_extreme_numeric_inputs_raise_validation_errors(self):
        huge = 10 ** 1000
        for data, policy in ((document(percent=huge), POLICY),
                             (document(count=huge, covered=huge, percent=100), POLICY),
                             (document(), {"file": FILE, "minimum_line_percent": huge})):
            with self.assertRaises(ValueError):
                check_coverage(data, policy)

    def test_accepts_exact_file_suffix_and_returns_counts_and_percent(self):
        result = check_coverage(document(filename="/tmp/build/crates/careops-ed/src/lib.rs"), POLICY)
        self.assertEqual(result["file"], FILE)
        self.assertEqual(result["lines"], {"count": 10, "covered": 8, "percent": 80.0})

    def test_reports_function_coverage_separately(self):
        result = check_coverage(document(functions={"count": 5, "covered": 3, "percent": 60}), POLICY)
        self.assertEqual(result["functions"], {"count": 5, "covered": 3, "percent": 60.0})
        self.assertEqual(result["lines"]["percent"], 80.0)

    def test_rejects_low_counts_with_falsely_high_percent(self):
        with self.assertRaises(ValueError):
            check_coverage(document(count=100, covered=1, percent=99), POLICY)

    def test_rejects_wrong_file_and_empty_data(self):
        for input_document in (
            document(filename="crates/careops-ed/src/other.rs"),
            {"data": []},
            {"data": [{"files": []}]},
        ):
            with self.subTest(input_document=input_document), self.assertRaises(ValueError):
                check_coverage(input_document, POLICY)

    def test_rejects_duplicate_target_rows_even_with_path_separator_alias(self):
        first = {"filename": FILE, "summary": {"lines": {"count": 10, "covered": 8, "percent": 80}}}
        second = {"filename": "crates\\careops-ed\\src\\lib.rs", "summary": {"lines": {"count": 10, "covered": 8, "percent": 80}}}
        with self.assertRaises(ValueError):
            check_coverage({"data": [{"files": [first, second]}]}, POLICY)

    def test_accepts_exact_floor_and_rejects_below_floor(self):
        self.assertEqual(check_coverage(document(count=5, covered=4, percent=80), POLICY)["lines"]["percent"], 80.0)
        with self.assertRaises(ValueError):
            check_coverage(document(count=100, covered=79, percent=79), POLICY)

    def test_rejects_invalid_counts_and_coverage_above_total(self):
        cases = [(0, 0, 0), (10, 0, 0), (10.0, 1, 10), (True, 1, 100),
                 (10, False, 0), (10, -1, -10), (10, 11, 110)]
        for count, covered, percent in cases:
            with self.subTest(count=count, covered=covered), self.assertRaises(ValueError):
                check_coverage(document(count=count, covered=covered, percent=percent), POLICY)

    def test_rejects_nonfinite_and_inconsistent_percent(self):
        for percent in (math.nan, math.inf, -math.inf, 90):
            with self.subTest(percent=percent), self.assertRaises(ValueError):
                check_coverage(document(percent=percent), POLICY)

    def test_rejects_malformed_policy_and_nan_or_invalid_threshold(self):
        for policy in (None, {}, {"file": FILE}, {"file": FILE, "minimum_line_percent": math.nan},
                       {"file": FILE, "minimum_line_percent": True},
                       {"file": FILE, "minimum_line_percent": 79},
                       {"file": FILE, "minimum_line_percent": 101},
                       {"file": "../lib.rs", "minimum_line_percent": 80}):
            with self.subTest(policy=policy), self.assertRaises(ValueError):
                check_coverage(document(), policy)


if __name__ == "__main__":
    unittest.main()
