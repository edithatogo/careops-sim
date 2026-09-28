#!/usr/bin/env python3
"""Score matched, synthetic skill-evaluation responses without modifying inputs."""

import json
import sys
from collections import Counter, defaultdict
from pathlib import Path


DISPOSITIONS = {"ready_for_review", "revise", "hold_for_evidence"}
ARMS = {"baseline", "skill"}
MODES = {"serial", "parallel"}
COMMON = (
    "candidate_skill", "dispatch_mode", "expected_disposition", "known_bad",
    "evaluable", "exclusion_reason", "repeat_of", "model_route",
    "reasoning_effort",
)
FIELDS = {
    "case_id", "arm", "candidate_skill", "dispatch_mode",
    "expected_disposition", "actual_disposition", "known_bad", "evaluable",
    "exclusion_reason", "repeat_of", "agent_id", "model_route",
    "reasoning_effort", "output_chars", "correction_turns",
}


class InputError(ValueError):
    pass


def fail(message):
    raise InputError(message)


def validate(data):
    if not isinstance(data, dict) or set(data) != {"schema_version", "results"}:
        fail("input must be an object containing only schema_version and results")
    if type(data["schema_version"]) is not int or data["schema_version"] != 1:
        fail("schema_version must be integer 1")
    rows = data["results"]
    if not isinstance(rows, list) or not rows:
        fail("results must be a non-empty array")
    pairs = defaultdict(dict)
    for i, row in enumerate(rows):
        where = f"results[{i}]"
        if not isinstance(row, dict) or set(row) != FIELDS:
            fail(f"{where} must contain exactly the documented result fields")
        for key in ("case_id", "candidate_skill", "agent_id", "model_route", "reasoning_effort"):
            if not isinstance(row[key], str) or not row[key].strip():
                fail(f"{where}.{key} must be a non-empty string")
        if not isinstance(row["arm"], str) or row["arm"] not in ARMS:
            fail(f"{where}.arm must be baseline or skill")
        if not isinstance(row["dispatch_mode"], str) or row["dispatch_mode"] not in MODES:
            fail(f"{where}.dispatch_mode must be serial or parallel")
        if not isinstance(row["actual_disposition"], str) or row["actual_disposition"] not in DISPOSITIONS:
            fail(f"{where}.actual_disposition is invalid")
        if type(row["evaluable"]) is not bool:
            fail(f"{where}.evaluable must be boolean")
        if row["evaluable"]:
            if not isinstance(row["expected_disposition"], str) or row["expected_disposition"] not in DISPOSITIONS:
                fail(f"{where}.expected_disposition must be a valid disposition when evaluable")
            if type(row["known_bad"]) is not bool:
                fail(f"{where}.known_bad must be boolean when evaluable")
            if row["exclusion_reason"] is not None:
                fail(f"{where}.exclusion_reason must be null when evaluable")
        else:
            if row["expected_disposition"] is not None or row["known_bad"] is not None:
                fail(f"{where} excluded case requires null expected_disposition and known_bad")
            if not isinstance(row["exclusion_reason"], str) or not row["exclusion_reason"].strip():
                fail(f"{where}.exclusion_reason must explain excluded cases")
        if row["repeat_of"] is not None and (not isinstance(row["repeat_of"], str) or not row["repeat_of"].strip()):
            fail(f"{where}.repeat_of must be null or a non-empty case_id")
        for key in ("output_chars", "correction_turns"):
            value = row[key]
            if value is not None and (type(value) is not int or value < 0):
                fail(f"{where}.{key} must be a nonnegative integer or null")
        if row["arm"] in pairs[row["case_id"]]:
            fail(f"case {row['case_id']!r} has duplicate {row['arm']} response")
        pairs[row["case_id"]][row["arm"]] = row
    for case_id, pair in pairs.items():
        if set(pair) != ARMS:
            fail(f"case {case_id!r} must have exactly one baseline and one skill response")
        base, skill = pair["baseline"], pair["skill"]
        for field in COMMON:
            if base[field] != skill[field]:
                fail(f"case {case_id!r} has mismatched paired metadata: {field}")
        if base["candidate_skill"] == "":
            fail(f"case {case_id!r} candidate_skill is empty")
    independents = {cid for cid, p in pairs.items() if p["baseline"]["repeat_of"] is None}
    for cid, pair in pairs.items():
        target = pair["baseline"]["repeat_of"]
        if target is None:
            continue
        if target not in independents:
            fail(f"repeat case {cid!r} must refer to an existing independent case")
        target_pair = pairs[target]
        for field in ("candidate_skill", "model_route", "reasoning_effort", "evaluable"):
            if pair["baseline"][field] != target_pair["baseline"][field]:
                fail(f"repeat case {cid!r} {field} differs from {target!r}")
        if pair["baseline"]["evaluable"]:
            for field in ("expected_disposition", "known_bad"):
                if pair["baseline"][field] != target_pair["baseline"][field]:
                    fail(f"repeat case {cid!r} {field} differs from {target!r}")
    return pairs


def ratio(n, d):
    return {"numerator": n, "denominator": d, "rate": n / d if d else None}


def pair_correctness(pairs):
    scored = [p for p in pairs if p["baseline"]["evaluable"]]
    both_correct = sum(
        all(p[arm]["actual_disposition"] == p[arm]["expected_disposition"] for arm in ARMS)
        for p in scored
    )
    return ratio(both_correct, len(scored))


def arm_metrics(rows):
    eval_rows = [r for r in rows if r["evaluable"]]
    accepted = [r for r in eval_rows if r["actual_disposition"] == "ready_for_review"]
    bad = [r for r in eval_rows if r["known_bad"]]
    bad_accepted = [r for r in bad if r["actual_disposition"] == "ready_for_review"]
    correct = sum(r["actual_disposition"] == r["expected_disposition"] for r in eval_rows)
    result = {
        "responses": len(rows), "evaluable_responses": len(eval_rows),
        "excluded_responses": len(rows) - len(eval_rows),
        "acceptance_accuracy": ratio(correct, len(eval_rows)),
        "known_bad_accepted_per_known_bad_response": ratio(len(bad_accepted), len(bad)),
        "known_bad_accepted_per_all_accepted": ratio(len(bad_accepted), len(accepted)),
        "accepted_responses": len(accepted),
    }
    for key in ("output_chars", "correction_turns"):
        observed = [r[key] for r in rows if r[key] is not None]
        complete = len(observed) == len(rows)
        result[key] = {
            "available": len(observed), "total": len(rows),
            "aggregate": sum(observed) if complete else None,
            "mean": (sum(observed) / len(observed)) if complete and observed else (None if not complete else None),
        }
    return result


def summarize(pairs):
    cases = list(pairs.items())
    independent = [(cid, p) for cid, p in cases if p["baseline"]["repeat_of"] is None]
    repeated = [(cid, p) for cid, p in cases if p["baseline"]["repeat_of"] is not None]
    evaluable = [(cid, p) for cid, p in independent if p["baseline"]["evaluable"]]
    excluded = [(cid, p) for cid, p in independent if not p["baseline"]["evaluable"]]
    all_rows = [r for _, p in cases for r in p.values()]
    summary = {
        "schema_version": 1,
        "candidate_skills": sorted({r["candidate_skill"] for r in all_rows}),
        "independent_case_count": len(independent),
        "repeated_case_count": len(repeated),
        "excluded_case_count": len(excluded),
        "excluded_cases": [{"case_id": cid, "reason": p["baseline"]["exclusion_reason"]} for cid, p in excluded],
        "response_count": len(all_rows),
        "arms": {}, "dispatch_modes": {},
        "paired_case_correctness": pair_correctness([p for _, p in evaluable]),
        "overall": arm_metrics([r for _, p in evaluable for r in p.values()]),
    }
    for arm in sorted(ARMS):
        summary["arms"][arm] = arm_metrics([p[arm] for _, p in independent])
    for mode in sorted(MODES):
        selected = [p[arm] for _, p in independent for arm in ARMS if p[arm]["dispatch_mode"] == mode]
        summary["dispatch_modes"][mode] = arm_metrics(selected)
    summary["repeated_cases"] = [{
        "case_id": cid, "repeat_of": p["baseline"]["repeat_of"],
        "dispatch_mode": p["baseline"]["dispatch_mode"],
        "arms": {arm: {"actual_disposition": p[arm]["actual_disposition"], "agent_id": p[arm]["agent_id"]} for arm in sorted(ARMS)},
    } for cid, p in repeated]
    summary["by_arm_dispatch_mode"] = {
        arm: {mode: arm_metrics([p[arm] for _, p in independent if p[arm]["dispatch_mode"] == mode]) for mode in sorted(MODES)}
        for arm in sorted(ARMS)
    }
    candidate_skills = sorted({p["baseline"]["candidate_skill"] for _, p in cases})
    summary["by_candidate_skill"] = {}
    for candidate in candidate_skills:
        selected = [(cid, p) for cid, p in independent if p["baseline"]["candidate_skill"] == candidate]
        candidate_result = {
            "independent_case_count": len(selected),
            "repeated_case_count": sum(p["baseline"]["candidate_skill"] == candidate for _, p in repeated),
            "paired_case_correctness": pair_correctness([p for _, p in selected]),
            "arms": {
                arm: arm_metrics([p[arm] for _, p in selected]) for arm in sorted(ARMS)
            },
            "dispatch_modes": {},
        }
        for mode in sorted(MODES):
            mode_pairs = [(cid, p) for cid, p in selected if p["baseline"]["dispatch_mode"] == mode]
            candidate_result["dispatch_modes"][mode] = {
                "paired_case_correctness": pair_correctness([p for _, p in mode_pairs]),
                "arms": {
                    arm: arm_metrics([p[arm] for _, p in mode_pairs]) for arm in sorted(ARMS)
                },
            }
        summary["by_candidate_skill"][candidate] = candidate_result
    return summary


def main(argv):
    if len(argv) != 2:
        print("usage: python3 tools/skill_eval.py INPUT.json", file=sys.stderr)
        return 2
    try:
        data = json.loads(Path(argv[1]).read_text(encoding="utf-8"))
        result = summarize(validate(data))
    except (OSError, json.JSONDecodeError, InputError) as exc:
        print(f"skill_eval: {exc}", file=sys.stderr)
        return 2
    json.dump(result, sys.stdout, indent=2, sort_keys=True)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
