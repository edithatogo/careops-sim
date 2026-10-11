"""Specification checks for E2.1 staff/calendar fixtures; no simulator execution."""
import copy
import json
import unittest
from pathlib import Path

FIXTURE = Path(__file__).parents[1] / "conductor/design/ed/e2-staff.json"
EXPECTED_POSITIVE = {
    "staff.empty", "staff.single_work", "staff.urgency", "staff.fifo",
    "staff.request_id_tie", "staff.staff_id_tie", "staff.skill_subset",
    "staff.zone_exact", "staff.ineligible_high_skip", "staff.shift_start_inclusive",
    "staff.shift_end_exclusive", "staff.break_start_exclusive", "staff.break_end_inclusive",
    "staff.drain_shift_end", "staff.drain_break", "staff.completion_at_boundary",
    "staff.horizon_active", "staff.horizon_grant", "staff.calendar_gap", "staff.permutation",
}
EXPECTED_NEGATIVE = {
    "staff.invalid.duplicate_staff_id", "staff.invalid.duplicate_request_id",
    "staff.invalid.malformed_id", "staff.invalid.nonpositive_shift",
    "staff.invalid.overlapping_shifts", "staff.invalid.break_outside_shift",
    "staff.invalid.overlapping_breaks", "staff.invalid.zero_work",
    "staff.invalid.ready_at_horizon", "staff.invalid.zero_horizon",
    "staff.invalid.count_limit", "staff.invalid.multi_person_unsupported",
}


def overlaps(a, b):
    return max(a[0], b[0]) < min(a[1], b[1])


def eligible(person, request, tick):
    return (person["assigned_zone"] == request["assigned_zone"]
            and set(request["required_skills"]).issubset(person["skills"])
            and any(a <= tick < b for a, b in person["shifts"])
            and not any(a <= tick < b for a, b in person["breaks"]))


def hand_oracle(case):
    """Small integer-tick reference sweep for the frozen hand expectations."""
    horizon = case["horizon"]
    people = {x["staff_id"]: x for x in case["inputs"]["staff"]}
    requests = {x["request_id"]: x for x in case["inputs"]["requests"]}
    active = {}
    rows = []
    for tick in range(horizon + 1):
        # Completion/release precedes calendar changes and new grants.
        for sid, (rid, end) in list(active.items()):
            if end == tick:
                del active[sid]
        waiting = [r for r in requests.values()
                   if r["ready_tick"] <= tick
                   and r["request_id"] not in {x["request_id"] for x in rows}]
        waiting.sort(key=lambda r: (-r["urgency"], r["ready_tick"], r["request_id"].encode("ascii")))
        changed = True
        while changed:
            changed = False
            for request in waiting:
                candidates = sorted(sid for sid, person in people.items()
                                    if sid not in active and eligible(person, request, tick))
                if not candidates:
                    continue
                sid = candidates[0]
                end = tick + request["work_ticks"]
                active[sid] = (request["request_id"], end)
                rows.append({"request_id": request["request_id"], "staff_id": sid,
                             "start_tick": tick, "configured_end_tick": end,
                             "observed_work_ticks": max(0, min(end, horizon) - tick),
                             "completed_by_horizon": end <= horizon,
                             "active_at_horizon": tick <= horizon < end})
                waiting.remove(request)
                changed = True
                break
    rows.sort(key=lambda r: (r["start_tick"], r["request_id"].encode("ascii")))
    assigned = {r["request_id"] for r in rows}
    unassigned = sorted(set(requests) - assigned, key=lambda x: x.encode("ascii"))
    staff_rows = {}
    for sid, person in people.items():
        own = [r for r in rows if r["staff_id"] == sid]
        work = [(r["start_tick"], min(r["configured_end_tick"], horizon))
                for r in own if min(r["configured_end_tick"], horizon) > r["start_tick"]]
        # Capacity-one and exclusivity are checked by sweeping the actual clipped intervals.
        for i, left in enumerate(work):
            for right in work[i + 1:]:
                if overlaps(left, right):
                    raise AssertionError(f"overlapping work for {sid}: {left}, {right}")
        busy = sum(b - a for a, b in work)
        break_ticks = 0
        outside_shift_ticks = 0
        for r in own:
            a, b = r["start_tick"], min(r["configured_end_tick"], horizon)
            for lo, hi in person["breaks"]:
                break_ticks += max(0, min(b, hi) - max(a, lo))
            covered = 0
            for lo, hi in person["shifts"]:
                covered += max(0, min(b, hi) - max(a, lo))
            outside_shift_ticks += (b - a) - covered
        staff_rows[sid] = {"configured_work_ticks": sum(r["configured_end_tick"] - r["start_tick"] for r in own),
                           "observed_work_ticks": busy,
                           "remaining_work_ticks_at_horizon": sum(max(0, r["configured_end_tick"] - horizon) for r in own if r["active_at_horizon"]),
                           "completed_count": sum(r["completed_by_horizon"] for r in own),
                           "active_at_horizon_count": sum(r["active_at_horizon"] for r in own),
                           "break_overlap_ticks": break_ticks,
                           "shift_overtime_ticks": outside_shift_ticks,
                           "scheduled_unavailable_work_ticks": break_ticks + outside_shift_ticks}
    return {"assignments": rows, "unassigned_request_ids": unassigned,
            "staff_accounting": staff_rows,
            "total_observed_work_ticks": sum(r["observed_work_ticks"] for r in rows)}


def valid_input(data):
    """Return (valid, category, path), using contract-level validation only."""
    def bad(category, path):
        return False, category, path

    def integer(value, maximum=(1 << 64) - 1):
        return isinstance(value, int) and not isinstance(value, bool) and 0 <= value <= maximum

    def identifier(value):
        return isinstance(value, str) and value.isascii() and 1 <= len(value) <= 64

    horizon = data.get("horizon")
    if not integer(horizon) or horizon == 0:
        return bad("horizon", "horizon")
    inputs = data["inputs"]
    staff = inputs["staff"]
    requests = inputs["requests"]
    if len(staff) > 1024 or len(requests) > 100000:
        return bad("staff_count" if len(staff) > 1024 else "request_count",
                   "inputs.staff" if len(staff) > 1024 else "inputs.requests")

    def unique_ids(collection, key, prefix):
        seen = set()
        for i, item in enumerate(collection):
            value = item.get(key)
            path = f"{prefix}[{i}].{key}"
            if not identifier(value):
                return bad("identifier", path)
            if value in seen:
                return bad("duplicate_id", path)
            seen.add(value)
        return None

    error = unique_ids(staff, "staff_id", "inputs.staff")
    if error:
        return error
    error = unique_ids(requests, "request_id", "inputs.requests")
    if error:
        return error
    for i, request in enumerate(requests):
        if not identifier(request.get("patient_id")):
            return bad("identifier", f"inputs.requests[{i}].patient_id")

    for i, person in enumerate(staff):
        if not identifier(person.get("assigned_zone")):
            return bad("identifier", f"inputs.staff[{i}].assigned_zone")
        skills = person.get("skills")
        if not isinstance(skills, list) or len(skills) > 32:
            return bad("skill_count", f"inputs.staff[{i}].skills")
        if any(not identifier(skill) for skill in skills):
            return bad("identifier", f"inputs.staff[{i}].skills")
        if skills != sorted(set(skills), key=lambda x: x.encode("ascii")):
            return bad("skills_not_sorted_unique", f"inputs.staff[{i}].skills")
        shifts, breaks = person.get("shifts"), person.get("breaks")
        if not isinstance(shifts, list) or not isinstance(breaks, list):
            return bad("calendar", f"inputs.staff[{i}]")
        if len(shifts) + len(breaks) > 1024:
            return bad("calendar_interval_count", f"inputs.staff[{i}]")
        for kind, intervals in (("shifts", shifts), ("breaks", breaks)):
            normalized = []
            for j, interval in enumerate(intervals):
                path = f"inputs.staff[{i}].{kind}[{j}]"
                if (not isinstance(interval, list) or len(interval) != 2
                        or not integer(interval[0]) or not integer(interval[1])):
                    return bad("calendar_interval", path)
                start, end = interval
                if start >= end:
                    return bad("shift_interval" if kind == "shifts" else "break_intervals", path)
                normalized.append((start, end))
            normalized.sort()
            if any(left[1] > right[0] for left, right in zip(normalized, normalized[1:])):
                return bad("shift_intervals" if kind == "shifts" else "break_intervals",
                           f"inputs.staff[{i}].{kind}")
        for j, (start, end) in enumerate(breaks):
            if not any(lo <= start < end <= hi for lo, hi in shifts):
                return bad("break_containment", f"inputs.staff[{i}].breaks[{j}]")

    for i, request in enumerate(requests):
        path = f"inputs.requests[{i}]"
        if not identifier(request.get("assigned_zone")):
            return bad("identifier", f"{path}.assigned_zone")
        skills = request.get("required_skills")
        if not isinstance(skills, list) or len(skills) > 32:
            return bad("skill_count", f"{path}.required_skills")
        if any(not identifier(skill) for skill in skills):
            return bad("identifier", f"{path}.required_skills")
        if skills != sorted(set(skills), key=lambda x: x.encode("ascii")):
            return bad("skills_not_sorted_unique", f"{path}.required_skills")
        ready = request.get("ready_tick")
        if not integer(ready) or ready >= horizon:
            return bad("ready_tick", f"{path}.ready_tick")
        work = request.get("work_ticks")
        if not integer(work) or work == 0:
            return bad("work_ticks", f"{path}.work_ticks")
        urgency = request.get("urgency")
        if not isinstance(urgency, int) or isinstance(urgency, bool) or not -(1 << 31) <= urgency < (1 << 31):
            return bad("urgency", f"{path}.urgency")
        if not integer(request.get("required_staff_count"), maximum=1) or request["required_staff_count"] != 1:
            return bad("required_staff_count", f"{path}.required_staff_count")
    return True, None, None


class StaffFixtureSpecTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.data = json.loads(FIXTURE.read_text())
        cls.cases = {case["case_id"]: case for case in cls.data["cases"]}

    def test_exact_case_inventory_and_unbound_status(self):
        self.assertEqual(set(self.cases), EXPECTED_POSITIVE | EXPECTED_NEGATIVE)
        self.assertEqual(self.data["case_ids"], [c["case_id"] for c in self.data["cases"]])
        self.assertEqual(len(self.data["cases"]), 32)
        self.assertEqual(self.data["native_binding"], {"status": "unbound", "target": "E2.2"})
        for case in self.cases.values():
            self.assertEqual(case["native_binding"], self.data["native_binding"])

    def test_positive_hand_oracles_and_resource_conservation(self):
        for case_id in sorted(EXPECTED_POSITIVE):
            with self.subTest(case=case_id):
                case = self.cases[case_id]
                self.assertEqual(valid_input({"horizon": case["horizon"], "inputs": case["inputs"]}), (True, None, None))
                actual = hand_oracle(case)
                self.assertEqual(case["expected"], actual)
                if case_id == "staff.permutation":
                    permuted = case["permuted_input"]
                    self.assertNotEqual(permuted["inputs"]["staff"][0]["shifts"], case["inputs"]["staff"][0]["shifts"])
                    self.assertNotEqual(permuted["inputs"]["staff"][0]["breaks"], case["inputs"]["staff"][0]["breaks"])
                    self.assertEqual(valid_input(permuted), (True, None, None))
                    self.assertEqual(hand_oracle(case | {"inputs": permuted["inputs"]}), actual)
                expected_ids = {r["request_id"] for r in case["inputs"]["requests"]}
                assigned = {r["request_id"] for r in actual["assignments"]}
                self.assertEqual(assigned | set(actual["unassigned_request_ids"]), expected_ids)
                self.assertFalse(assigned & set(actual["unassigned_request_ids"]))
                self.assertEqual(actual["total_observed_work_ticks"],
                                 sum(x["observed_work_ticks"] for x in actual["assignments"]))

    def test_contract_field_and_cardinality_boundaries(self):
        base = self.cases["staff.single_work"]
        sample = {"horizon": base["horizon"], "inputs": copy.deepcopy(base["inputs"])}

        # Identifiers are any ASCII strings of length 1..64; patient IDs may repeat.
        long_id = copy.deepcopy(sample)
        long_id["inputs"]["staff"][0]["staff_id"] = "x" * 64
        long_id["inputs"]["requests"][0]["patient_id"] = "x" * 64
        self.assertEqual(valid_input(long_id), (True, None, None))
        for value in ("", "x" * 65, "é"):
            changed = copy.deepcopy(sample)
            changed["inputs"]["staff"][0]["staff_id"] = value
            self.assertFalse(valid_input(changed)[0], value)
        repeated_patient = copy.deepcopy(sample)
        repeated_patient["inputs"]["requests"].append(copy.deepcopy(repeated_patient["inputs"]["requests"][0]))
        repeated_patient["inputs"]["requests"][1]["request_id"] = "r2"
        self.assertEqual(valid_input(repeated_patient), (True, None, None))

        # Skill lists must be sorted, unique, and capped at 32.
        skills = copy.deepcopy(sample)
        skills["inputs"]["staff"][0]["skills"] = [f"k{i:02d}" for i in range(32)]
        self.assertEqual(valid_input(skills), (True, None, None))
        for bad_skills in ([f"k{i:02d}" for i in range(33)], ["b", "a"], ["a", "a"]):
            changed = copy.deepcopy(sample)
            changed["inputs"]["staff"][0]["skills"] = bad_skills
            self.assertFalse(valid_input(changed)[0], bad_skills[:3])

        # Calendar interval cap is 1024 combined; input order is not semantic.
        calendar = copy.deepcopy(sample)
        calendar["horizon"] = 2048
        person = calendar["inputs"]["staff"][0]
        person["shifts"] = [[2 * i, 2 * i + 2] for i in range(512)]
        person["breaks"] = [[2 * i, 2 * i + 1] for i in range(512)]
        self.assertEqual(valid_input(calendar), (True, None, None))
        person["shifts"].append([1024, 1026])
        self.assertEqual(valid_input(calendar), (False, "calendar_interval_count", "inputs.staff[0]"))
        reordered = copy.deepcopy(sample)
        reordered["inputs"]["staff"][0]["shifts"] = [[5, 8], [0, 3]]
        reordered["inputs"]["staff"][0]["breaks"] = [[6, 7], [1, 2]]
        self.assertEqual(valid_input(reordered), (True, None, None))

        # Tick quantities are nonnegative u64 integers (never booleans); urgency is i32.
        boundary = copy.deepcopy(sample)
        request = boundary["inputs"]["requests"][0]
        request["urgency"] = -(1 << 31)
        request["work_ticks"] = (1 << 64) - 1
        self.assertEqual(valid_input(boundary), (True, None, None))
        mutations = [
            ("ready_tick", True), ("work_ticks", True), ("work_ticks", 1 << 64),
            ("urgency", 1 << 31), ("urgency", -(1 << 31) - 1),
        ]
        for key, value in mutations:
            changed = copy.deepcopy(sample)
            changed["inputs"]["requests"][0][key] = value
            self.assertFalse(valid_input(changed)[0], (key, value))
        for value in (True, 1.0, 2):
            changed = copy.deepcopy(sample)
            changed["inputs"]["requests"][0]["required_staff_count"] = value
            self.assertEqual(valid_input(changed), (False, "required_staff_count", "inputs.requests[0].required_staff_count"))

    def test_required_boundary_and_priority_witnesses(self):
        def starts(case_id):
            return [(x["request_id"], x["staff_id"], x["start_tick"])
                    for x in self.cases[case_id]["expected"]["assignments"]]
        self.assertEqual(starts("staff.single_work"), [("r1", "s1", 1)])
        self.assertEqual(starts("staff.urgency")[0][0], "high")
        self.assertEqual(starts("staff.fifo")[0][0], "z_earlier")
        self.assertEqual(starts("staff.request_id_tie")[0][0], "a")
        self.assertEqual(starts("staff.staff_id_tie")[0][1], "a")
        self.assertEqual(starts("staff.shift_start_inclusive")[0][2], 3)
        self.assertEqual(starts("staff.shift_end_exclusive"), [])
        self.assertEqual(starts("staff.break_start_exclusive")[0][2], 5)
        self.assertEqual(starts("staff.break_end_inclusive")[0][2], 5)
        self.assertEqual(starts("staff.zone_exact")[0][1], "s2")
        self.assertEqual(starts("staff.skill_subset")[0][1], "s1")
        self.assertEqual(starts("staff.ineligible_high_skip")[0][0], "low")
        self.assertEqual(self.cases["staff.calendar_gap"]["expected"]["assignments"][0]["start_tick"], 5)
        self.assertEqual(starts("staff.completion_at_boundary"), [("first", "s1", 1), ("second", "s1", 5)])

    def test_drain_and_horizon_accounting_distinctions(self):
        shift = self.cases["staff.drain_shift_end"]["expected"]
        self.assertEqual(shift["assignments"][0]["configured_end_tick"], 6)
        self.assertEqual(shift["staff_accounting"]["s1"]["shift_overtime_ticks"], 3)
        self.assertEqual(shift["staff_accounting"]["s1"]["break_overlap_ticks"], 0)
        self.assertEqual(shift["unassigned_request_ids"], ["queued"])
        brk = self.cases["staff.drain_break"]["expected"]["staff_accounting"]["s1"]
        self.assertEqual(brk["break_overlap_ticks"], 2)
        self.assertEqual(brk["shift_overtime_ticks"], 0)
        self.assertEqual(brk["scheduled_unavailable_work_ticks"], 2)
        active = self.cases["staff.horizon_active"]["expected"]["assignments"][0]
        self.assertEqual((active["observed_work_ticks"], active["completed_by_horizon"], active["active_at_horizon"]), (3, False, True))
        grant = self.cases["staff.horizon_grant"]["expected"]["assignments"][0]
        self.assertEqual((grant["start_tick"], grant["observed_work_ticks"], grant["completed_by_horizon"], grant["active_at_horizon"]), (10, 0, False, True))
        self.assertEqual(self.cases["staff.horizon_grant"]["expected"]["staff_accounting"]["s1"]["remaining_work_ticks_at_horizon"], 2)

    def test_negative_baseline_mutation_and_targeted_repair(self):
        for case_id in sorted(EXPECTED_NEGATIVE):
            with self.subTest(case=case_id):
                case = self.cases[case_id]
                baseline = case["valid_baseline"]
                self.assertEqual(valid_input(baseline), (True, None, None))
                invalid = case["invalid_input"]
                ok, category, path = valid_input(invalid)
                self.assertFalse(ok)
                self.assertEqual(category, case["diagnostic_category"])
                self.assertEqual(path, case["offending_path"])
                repair = case["repair_control"]
                self.assertEqual(valid_input(repair), (True, None, None))
                self.assertEqual(repair, baseline)
                self.assertTrue(case["repair_changes_only_mutation_group"])
                if case["mutation_group"] == "horizon":
                    self.assertEqual(invalid["inputs"], baseline["inputs"])
                elif case["mutation_group"] in {"staff_count", "duplicate_id", "identifier", "shift_interval", "shift_intervals", "break_containment", "break_intervals"} and case_id != "staff.invalid.duplicate_request_id":
                    self.assertEqual(invalid["horizon"], baseline["horizon"])
                    self.assertEqual(invalid["inputs"]["requests"], baseline["inputs"]["requests"])
                else:
                    self.assertEqual(invalid["horizon"], baseline["horizon"])
                    self.assertEqual(invalid["inputs"]["staff"], baseline["inputs"]["staff"])
        count = self.cases["staff.invalid.count_limit"]["invalid_input"]["inputs"]["staff"]
        self.assertEqual(len(count), 1025)
        self.assertEqual(len({p["staff_id"] for p in count}), 1025)
        multi = self.cases["staff.invalid.multi_person_unsupported"]
        self.assertEqual(multi["invalid_input"]["inputs"]["requests"][0]["required_staff_count"], 2)
        self.assertEqual(multi["repair_control"]["inputs"]["requests"][0]["required_staff_count"], 1)


if __name__ == "__main__":
    unittest.main()
