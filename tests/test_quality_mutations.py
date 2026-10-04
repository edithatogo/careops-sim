import copy
from contextlib import redirect_stderr
import hashlib
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from tools.quality_mutations import check_outcomes, main


SOURCE_PATH = Path(__file__).resolve().parents[1] / "crates/careops-ed/src/lib.rs"
SOURCE_BYTES = SOURCE_PATH.read_bytes()
BASELINE_UNVIABLES = (
    "crates/careops-ed/src/lib.rs:239:5: replace parse_scenario_json -> Result<ScenarioConfig, ScenarioError> with Ok(Default::default())",
    "crates/careops-ed/src/lib.rs:635:5: replace invalid -> Result<T, ScenarioError> with Ok(Default::default())",
    "crates/careops-ed/src/lib.rs:653:5: replace run_scenario -> Result<RunSummary, SimulationError> with Ok(Default::default())",
)
EQUIVALENT_MISSES = (
    (735, 60, "crates/careops-ed/src/lib.rs:735:60: replace || with && in run_scenario"),
    (798, 43, "crates/careops-ed/src/lib.rs:798:43: replace || with && in run_scenario"),
)


def phase(name, outcome):
    return {
        "argv": ["cargo", "test"] + (["--no-run"] if name == "Build" else []) + ["--verbose", "--package=careops-ed@0.1.0"],
        "duration": 0.25,
        "phase": name,
        "process_status": outcome,
    }


def mutant_row(name, summary, *, line, column=1, genre="BinaryOperator", replacement="!", phase_results=None):
    mutant = {
        "file": "crates/careops-ed/src/lib.rs",
        "function": {
            "function_name": "run_scenario",
            "return_type": "-> Result<RunSummary, SimulationError>",
            "span": {"start": {"line": 1, "column": 1}, "end": {"line": 900, "column": 1}},
        },
        "genre": genre,
        "name": name,
        "package": "careops-ed",
        "replacement": replacement,
        "span": {"start": {"line": line, "column": column},
                 "end": {"line": line, "column": column + (2 if replacement == "&&" else 1)}},
    }
    return {
        "diff_path": "diff/mutant-" + str(line) + "-" + str(column) + ".diff",
        "log_path": "log/mutant-" + str(line) + "-" + str(column) + ".log",
        "phase_results": phase_results,
        "scenario": {"Mutant": mutant},
        "summary": summary,
    }


def valid_outcomes():
    rows = [{
        "diff_path": None,
        "log_path": "log/baseline.log",
        "phase_results": [phase("Build", "Success"), phase("Test", "Success")],
        "scenario": "Baseline",
        "summary": "Success",
    }]
    for index in range(110):
        line = 10 + index
        rows.append(mutant_row(
            f"crates/careops-ed/src/lib.rs:{line}:1: replace ! with true in run_scenario",
            "CaughtMutant", line=line,
            phase_results=[phase("Build", "Success"), phase("Test", {"Failure": 101})],
        ))
    for line, column, name in EQUIVALENT_MISSES:
        rows.append(mutant_row(
            name, "MissedMutant", line=line, column=column,
            genre="BinaryOperator", replacement="&&",
            phase_results=[phase("Build", "Success"), phase("Test", "Success")],
        ))
    for index, name in enumerate(BASELINE_UNVIABLES):
        line = (239, 635, 653)[index]
        rows.append(mutant_row(
            name, "Unviable", line=line, column=5,
            genre="FnValue", replacement="Ok(Default::default())",
            phase_results=[phase("Build", {"Failure": 101})],
        ))
    return {
        "cargo_mutants_version": "27.1.0",
        "caught": 110,
        "end_time": "2026-10-05T10:02:00Z",
        "missed": 2,
        "outcomes": rows,
        "start_time": "2026-10-05T10:00:00Z",
        "success": 0,
        "timeout": 0,
        "total_mutants": 115,
        "unviable": 3,
    }


def catalog_bytes(document):
    records = []
    for row in document["outcomes"]:
        if isinstance(row.get("scenario"), dict) and set(row["scenario"]) == {"Mutant"}:
            records.append({"diff": "synthetic diff", **row["scenario"]["Mutant"]})
    return json.dumps(records, sort_keys=True).encode("utf-8")


def validate(document, source_bytes=SOURCE_BYTES, catalog=None, expected_catalog_sha256=None, expected_cargo="cargo"):
    catalog = catalog_bytes(document) if catalog is None else catalog
    expected = expected_catalog_sha256 or hashlib.sha256(catalog).hexdigest()
    return check_outcomes(document, source_bytes, catalog, expected_catalog_sha256=expected, expected_cargo=expected_cargo)


class MutationGateTests(unittest.TestCase):
    def test_accepts_full_bound_baseline_and_exact_equivalence_boundaries(self):
        result = validate(valid_outcomes())
        self.assertEqual(result["source_sha256"], "4f7917f3976d84b86a1b2b6837065c803d75b29d6b4f594abbbb7ceaedc57380")
        self.assertEqual(result["total_mutants"], 115)
        self.assertEqual(result["caught"], 110)
        self.assertEqual(result["missed"], 2)
        self.assertEqual(result["unviable"], 3)
        self.assertEqual(result["timeout"], 0)
        self.assertEqual(len(result["equivalent_misses"]), 2)
        self.assertNotIn("score", result)

    def test_rejects_substituted_phase_commands(self):
        for argv in (["false"], ["cargo", "test", "--package=another"],
                     ["cargo", "test", "--no-run", "--verbose", "--package=careops-ed@0.1.0", "--locked", "--locked"]):
            document = valid_outcomes()
            document["outcomes"][0]["phase_results"][0]["argv"] = argv
            with self.subTest(argv=argv), self.assertRaises(ValueError):
                validate(document)

    def test_accepts_single_locked_argument_in_recorded_commands(self):
        document = valid_outcomes()
        for row in document["outcomes"]:
            for item in row["phase_results"]:
                item["argv"].append("--locked")
        self.assertEqual(validate(document)["total_mutants"], 115)

    def test_exact_runtime_resolved_cargo_path_must_be_supplied(self):
        resolved = "/home/runner/.rustup/toolchains/1.99.0-x86_64-unknown-linux-gnu/bin/cargo"
        document = valid_outcomes()
        for row in document["outcomes"]:
            for item in row["phase_results"]:
                item["argv"][0] = resolved
                item["argv"].append("--locked")
        with self.assertRaises(ValueError):
            validate(document)
        self.assertEqual(validate(document, expected_cargo=resolved)["total_mutants"], 115)
        with self.assertRaises(ValueError):
            validate(document, expected_cargo=resolved.replace("/home/runner", "/other"))

    def test_invalid_or_noncanonical_cargo_identity_is_rejected(self):
        for executable in ("false", "bin/cargo", "/opt/bin/cargo",
                           "/home/runner/.rustup/toolchains/1.98.1-x86_64-unknown-linux-gnu/bin/cargo"):
            with self.subTest(executable=executable), self.assertRaises(ValueError):
                validate(valid_outcomes(), expected_cargo=executable)

    def test_rejects_changed_source(self):
        with self.assertRaises(ValueError):
            validate(valid_outcomes(), source_bytes=SOURCE_BYTES + b"\n")

    def test_rejects_wrong_version_and_missing_baseline(self):
        document = valid_outcomes()
        document["cargo_mutants_version"] = "27.0.0"
        with self.assertRaises(ValueError):
            validate(document)
        document = valid_outcomes()
        document["outcomes"].pop(0)
        with self.assertRaises(ValueError):
            validate(document)

    def test_rejects_baseline_failure_or_duplicate_baseline(self):
        document = valid_outcomes()
        document["outcomes"][0]["phase_results"][1]["process_status"] = {"Failure": 101}
        with self.assertRaises(ValueError):
            validate(document)
        document = valid_outcomes()
        document["outcomes"].append(copy.deepcopy(document["outcomes"][0]))
        with self.assertRaises(ValueError):
            validate(document)

    def test_rejects_missing_or_duplicate_mutants(self):
        document = valid_outcomes()
        document["outcomes"].pop()
        document["total_mutants"] -= 1
        document["unviable"] -= 1
        with self.assertRaises(ValueError):
            validate(document)
        document = valid_outcomes()
        document["outcomes"][2] = copy.deepcopy(document["outcomes"][1])
        with self.assertRaises(ValueError):
            validate(document)

    def test_rejects_wrong_package_or_source_file(self):
        for key, value in (("package", "other"), ("file", "crates/careops-ed/src/main.rs")):
            document = valid_outcomes()
            document["outcomes"][1]["scenario"]["Mutant"][key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                validate(document)

    def test_rejects_any_non_disposition_missed_mutant(self):
        bad_mutants = (
            {"line": 735, "column": 61, "genre": "BinaryOperator", "replacement": "&&", "name": "replace || with &&"},
            {"line": 735, "column": 60, "genre": "Other", "replacement": "&&", "name": "replace || with &&"},
            {"line": 735, "column": 60, "genre": "BinaryOperator", "replacement": "||", "name": "replace || with &&"},
            {"line": 735, "column": 60, "genre": "BinaryOperator", "replacement": "&&", "name": "replace && with ||"},
            {"line": 735, "column": 60, "genre": "BinaryOperator", "replacement": "&&", "name": "unbound replace || with &&"},
        )
        for bad in bad_mutants:
            document = valid_outcomes()
            row = document["outcomes"][111]
            mutant = row["scenario"]["Mutant"]
            mutant["span"]["start"] = {"line": bad["line"], "column": bad["column"]}
            mutant["span"]["end"] = {"line": bad["line"], "column": bad["column"] + 2}
            mutant["genre"] = bad["genre"]
            mutant["replacement"] = bad["replacement"]
            mutant["name"] = bad["name"]
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                validate(document)

    def test_rejects_new_or_excess_unviable_mutants(self):
        document = valid_outcomes()
        document["outcomes"][-1]["scenario"]["Mutant"]["name"] = "new unviable mutant"
        with self.assertRaises(ValueError):
            validate(document)
        document = valid_outcomes()
        extra = copy.deepcopy(document["outcomes"][-1])
        extra["scenario"]["Mutant"]["name"] = "fourth unviable mutant"
        extra["scenario"]["Mutant"]["span"]["start"]["line"] = 700
        document["outcomes"].append(extra)
        document["total_mutants"] += 1
        document["unviable"] += 1
        with self.assertRaises(ValueError):
            validate(document)

    def test_rejects_counter_bypass_timeout_and_boolean_counts(self):
        changes = (
            ("total_mutants", 114), ("caught", 109), ("missed", 3),
            ("unviable", 2), ("timeout", 1), ("success", True),
            ("caught", True),
        )
        for key, value in changes:
            document = valid_outcomes()
            document[key] = value
            with self.subTest(key=key, value=value), self.assertRaises(ValueError):
                validate(document)

    def test_rejects_skipped_unfinished_and_unexpected_summaries(self):
        for summary in ("Skipped", "NotRun", "Unknown"):
            document = valid_outcomes()
            document["outcomes"][1]["summary"] = summary
            with self.subTest(summary=summary), self.assertRaises(ValueError):
                validate(document)

    def test_rejects_incoherent_mutant_phase_results(self):
        cases = (
            (1, [phase("Build", "Success"), phase("Test", "Success")]),
            (111, [phase("Build", "Success"), phase("Test", {"Failure": 101})]),
            (113, [phase("Build", "Success"), phase("Test", "Success")]),
        )
        for row_index, phases in cases:
            document = valid_outcomes()
            document["outcomes"][row_index]["phase_results"] = phases
            with self.subTest(row_index=row_index), self.assertRaises(ValueError):
                validate(document)

    def test_rejects_unrepresentable_phase_duration(self):
        document = valid_outcomes()
        document["outcomes"][0]["phase_results"][0]["duration"] = 10 ** 1000
        with self.assertRaises(ValueError):
            validate(document)

    def test_cli_validates_json_and_writes_report_or_stderr_failure(self):
        with tempfile.TemporaryDirectory() as directory:
            outcomes_path = Path(directory) / "outcomes.json"
            document = valid_outcomes()
            catalog = catalog_bytes(document)
            outcomes_path.write_text(json.dumps(document), encoding="utf-8")
            (Path(directory) / "mutants.json").write_bytes(catalog)
            with patch("tools.quality_mutations.CATALOG_SHA256", hashlib.sha256(catalog).hexdigest()):
                self.assertEqual(main(["--outcomes", str(outcomes_path), "--source", str(SOURCE_PATH)]), 0)
            outcomes_path.write_text("{\"timeout\": 0, \"timeout\": 1}", encoding="utf-8")
            stderr = io.StringIO()
            with redirect_stderr(stderr):
                result = main(["--outcomes", str(outcomes_path), "--source", str(SOURCE_PATH)])
            self.assertEqual(result, 1)
            self.assertIn("duplicate JSON key", stderr.getvalue())

    def test_rejects_mutant_catalog_hash_and_outcome_catalog_substitution(self):
        document = valid_outcomes()
        catalog = catalog_bytes(document)
        with self.assertRaises(ValueError):
            check_outcomes(document, SOURCE_BYTES, catalog, expected_catalog_sha256="0" * 64)
        altered = copy.deepcopy(document)
        altered["outcomes"][1]["scenario"]["Mutant"]["replacement"] = "different"
        with self.assertRaises(ValueError):
            validate(document, catalog=catalog_bytes(altered))


if __name__ == "__main__":
    unittest.main()
