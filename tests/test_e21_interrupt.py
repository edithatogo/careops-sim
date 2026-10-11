"""Behavior-only E2.1 interruption/handover fixtures; no simulator execution."""

import copy
import json
import unittest
from pathlib import Path

FIXTURE = Path(__file__).parents[1] / "conductor/design/ed/e2-interrupt.json"
POSITIVE_IDS = {
    "interrupt.suspend_resume", "interrupt.resource_consent_absent",
    "interrupt.incumbent_consent_absent", "interrupt.incoming_consent_absent",
    "interrupt.equal_urgency", "interrupt.completion_tie",
    "interrupt.break_precedence", "interrupt.shift_precedence",
    "interrupt.priority_extremes", "interrupt.handover_immediate",
    "interrupt.handover_wait", "interrupt.handover_completion_tie",
    "interrupt.handover_not_active", "interrupt.horizon_suspended",
    "interrupt.horizon_handover", "interrupt.permutation",
}
NEGATIVE_IDS = {
    "interrupt.invalid." + name for name in (
        "unknown_episode", "unknown_incumbent", "unknown_replacement",
        "wrong_incumbent", "duplicate_handover", "handover_after_horizon",
        "unsupported_strategy", "handover_consent_absent",
    )
}

# Independently hand-derived, literal fixture oracle. Each timeline is
# (logical episode, staff, observed start, observed end). These literals are
# intentionally separate from the JSON so a copied or drifting fixture fails.
HAND_ORACLES = {
    "interrupt.suspend_resume": {
        "timeline": [("low", "a", 1, 4), ("urgent", "a", 4, 6), ("low", "a", 6, 13)],
        "final": {"low": ("completed", 0), "urgent": ("completed", 0)},
        "waits": [("low", 4, 6, "suspended_remainder", None)],
        "consent": "allowed", "handovers": {}, "noops": [],
    },
    "interrupt.resource_consent_absent": {
        "timeline": [("ep", "a", 1, 11), ("urgent", "a", 11, 13)],
        "final": {"ep": ("completed", 0), "urgent": ("completed", 0)},
        "waits": [("urgent", 4, 11, "queued", None)],
        "consent": "resource_consent_denied", "handovers": {}, "noops": [],
    },
    "interrupt.incumbent_consent_absent": {
        "timeline": [("ep", "a", 1, 11), ("urgent", "a", 11, 13)],
        "final": {"ep": ("completed", 0), "urgent": ("completed", 0)},
        "waits": [("urgent", 4, 11, "queued", None)],
        "consent": "incumbent_consent_denied", "handovers": {}, "noops": [],
    },
    "interrupt.incoming_consent_absent": {
        "timeline": [("ep", "a", 1, 11), ("urgent", "a", 11, 13)],
        "final": {"ep": ("completed", 0), "urgent": ("completed", 0)},
        "waits": [("urgent", 4, 11, "queued", None)],
        "consent": "incoming_consent_denied", "handovers": {}, "noops": [],
    },
    "interrupt.equal_urgency": {
        "timeline": [("ep", "a", 1, 11), ("urgent", "a", 11, 13)],
        "final": {"ep": ("completed", 0), "urgent": ("completed", 0)},
        "waits": [("urgent", 4, 11, "queued", None)],
        "consent": "equal_urgency", "handovers": {}, "noops": [],
    },
    "interrupt.completion_tie": {
        "timeline": [("low", "a", 0, 3), ("urgent", "a", 3, 5)],
        "final": {"low": ("completed", 0), "urgent": ("completed", 0)},
        "waits": [], "consent": "allowed", "handovers": {}, "noops": [],
    },
    "interrupt.break_precedence": {
        "timeline": [("low", "a", 1, 5), ("urgent", "a", 5, 7), ("low", "a", 7, 13)],
        "final": {"low": ("completed", 0), "urgent": ("completed", 0)},
        "waits": [("urgent", 3, 5, "queued", None), ("low", 5, 7, "suspended_remainder", None)],
        "consent": "allowed", "handovers": {}, "noops": [],
    },
    "interrupt.shift_precedence": {
        "timeline": [("low", "a", 1, 5), ("urgent", "a", 5, 7), ("low", "a", 7, 13)],
        "final": {"low": ("completed", 0), "urgent": ("completed", 0)},
        "waits": [("urgent", 3, 5, "queued", None), ("low", 5, 7, "suspended_remainder", None)],
        "consent": "allowed", "handovers": {}, "noops": [],
    },
    "interrupt.priority_extremes": {
        "timeline": [("max", "a", 0, 1), ("zero", "a", 1, 2), ("min", "a", 2, 3)],
        "final": {"max": ("completed", 0), "zero": ("completed", 0), "min": ("completed", 0)},
        "waits": [("zero", 0, 1, "queued", None), ("min", 0, 2, "queued", None)],
        "consent": "not_applicable", "handovers": {}, "noops": [],
    },
    "interrupt.handover_immediate": {
        "timeline": [("ep", "a", 1, 3), ("ep", "b", 3, 6)],
        "final": {"ep": ("completed", 0)}, "waits": [],
        "consent": "not_applicable", "handovers": {"ep": (3, "applied")}, "noops": [],
    },
    "interrupt.handover_wait": {
        "timeline": [("ep", "a", 1, 3), ("busy", "b", 0, 5), ("ep", "b", 5, 8)],
        "final": {"ep": ("completed", 0), "busy": ("completed", 0)},
        "waits": [("ep", 3, 5, "handover_continuation", "b")],
        "consent": "not_applicable", "handovers": {"ep": (3, "applied")}, "noops": [],
    },
    "interrupt.handover_completion_tie": {
        "timeline": [("ep", "a", 1, 3)], "final": {"ep": ("completed", 0)},
        "waits": [], "consent": "not_applicable", "handovers": {"ep": (3, "completed_before_handover")},
        "noops": ["completed_before_handover"],
    },
    "interrupt.handover_not_active": {
        "timeline": [("ep", "a", 5, 10)], "final": {"ep": ("completed", 0)},
        "waits": [], "consent": "not_applicable", "handovers": {"ep": (3, "not_active")},
        "noops": ["not_active"],
    },
    "interrupt.horizon_suspended": {
        "timeline": [("low", "a", 1, 4), ("urgent", "a", 4, 6)],
        "final": {"low": ("suspended", 7), "urgent": ("active_at_horizon", 3)},
        "waits": [("low", 4, 6, "suspended_remainder", None)],
        "consent": "allowed", "handovers": {}, "noops": [],
    },
    "interrupt.horizon_handover": {
        "timeline": [("ep", "a", 1, 3), ("ep", "b", 3, 3)],
        "final": {"ep": ("active_at_horizon", 3)}, "waits": [],
        "consent": "not_applicable", "handovers": {"ep": (3, "applied")}, "noops": [],
    },
    "interrupt.permutation": {
        "timeline": [("ep", "a", 1, 3), ("ep2", "c", 1, 2), ("ep2", "d", 2, 5), ("ep", "b", 3, 6)],
        "final": {"ep": ("completed", 0), "ep2": ("completed", 0)},
        "waits": [], "consent": "not_applicable",
        "handovers": {"ep": (3, "applied"), "ep2": (2, "applied")}, "noops": [],
    },
}


def identifier(value):
    return isinstance(value, str) and value.isascii() and 1 <= len(value) <= 64


def tick(value, maximum=(1 << 64) - 1):
    return isinstance(value, int) and not isinstance(value, bool) and 0 <= value <= maximum


def setpath(value, path, replacement):
    parts = path.replace("[", ".").replace("]", "").split(".")
    target = value
    for part in parts[:-1]:
        target = target[int(part)] if part.isdigit() else target[part]
    final = parts[-1]
    target[int(final) if final.isdigit() else final] = replacement


def overlaps(left, right):
    return max(left[0], right[0]) < min(left[1], right[1])


def differing_paths(left, right, prefix=""):
    if isinstance(left, dict) and isinstance(right, dict):
        paths = []
        for key in sorted(set(left) | set(right)):
            path = f"{prefix}.{key}" if prefix else key
            if key not in left or key not in right:
                paths.append(path)
            else:
                paths.extend(differing_paths(left[key], right[key], path))
        return paths
    if isinstance(left, list) and isinstance(right, list):
        paths = []
        for index in range(max(len(left), len(right))):
            path = f"{prefix}[{index}]"
            if index >= len(left) or index >= len(right):
                paths.append(path)
            else:
                paths.extend(differing_paths(left[index], right[index], path))
        return paths
    return [] if left == right else [prefix]


def normalized_inputs(inputs):
    """Canonicalize only array order; preserve every meaningful field."""
    result = copy.deepcopy(inputs)
    result["staff"].sort(key=lambda x: x["staff_id"].encode("ascii"))
    result["episodes"].sort(key=lambda x: x["episode_id"].encode("ascii"))
    result["urgent_requests"].sort(key=lambda x: (x["ready_tick"], x["episode_id"].encode("ascii")))
    result["events"].sort(key=lambda x: (x["episode_id"].encode("ascii"), x["tick"]))
    return result


def validate_handover_inputs(inputs, horizon):
    """Contract-level validator for the bounded subset represented by fixtures."""
    if not tick(horizon) or horizon == 0:
        return ("horizon", "horizon")
    if inputs.get("strategy") != "Suspend":
        return ("unsupported_strategy", "strategy")
    if inputs.get("in_flight_policy") != "DrainActive":
        return ("in_flight_policy", "in_flight_policy")
    staff = inputs.get("staff")
    episodes = inputs.get("episodes")
    requests = inputs.get("urgent_requests")
    events = inputs.get("events")
    if not all(isinstance(x, list) for x in (staff, episodes, requests, events)):
        return ("collection", "inputs")
    if len(staff) > 1024 or len(episodes) > 100000 or len(requests) > 100000:
        return ("count_limit", "inputs")

    if any(not isinstance(item, dict) for item in staff + episodes + requests + events):
        return ("row_type", "inputs")

    staff_by_id = {}
    for index, person in enumerate(staff):
        path = f"staff[{index}]"
        sid = person.get("staff_id")
        if not identifier(sid):
            return ("identifier", path + ".staff_id")
        if sid in staff_by_id:
            return ("duplicate_id", path + ".staff_id")
        staff_by_id[sid] = person
        if not identifier(person.get("assigned_zone")):
            return ("identifier", path + ".assigned_zone")
        skills = person.get("skills")
        if not isinstance(skills, list) or len(skills) > 32 or any(not identifier(x) for x in skills):
            return ("skill_count", path + ".skills")
        if skills != sorted(set(skills), key=lambda x: x.encode("ascii")):
            return ("skills_not_sorted_unique", path + ".skills")
        shifts, breaks = person.get("shifts"), person.get("breaks")
        if not isinstance(shifts, list) or not isinstance(breaks, list) or len(shifts) + len(breaks) > 1024:
            return ("calendar_count", path)
        for label, intervals in (("shifts", shifts), ("breaks", breaks)):
            normalized = []
            for i, interval in enumerate(intervals):
                p = f"{path}.{label}[{i}]"
                if not isinstance(interval, list) or len(interval) != 2 or not all(tick(x) for x in interval):
                    return ("calendar_interval", p)
                if interval[0] >= interval[1]:
                    return ("calendar_interval", p)
                normalized.append(tuple(interval))
            normalized.sort()
            if any(a[1] > b[0] for a, b in zip(normalized, normalized[1:])):
                return ("calendar_overlap", f"{path}.{label}")
        for i, (start, end) in enumerate(breaks):
            if not any(lo <= start < end <= hi for lo, hi in shifts):
                return ("break_containment", f"{path}.breaks[{i}]")

    episodes_by_id = {}
    for index, episode in enumerate(episodes):
        path = f"episodes[{index}]"
        eid = episode.get("episode_id")
        if not identifier(eid):
            return ("identifier", path + ".episode_id")
        if eid in episodes_by_id:
            return ("duplicate_id", path + ".episode_id")
        episodes_by_id[eid] = episode
        if episode.get("initial_staff_id") not in staff_by_id:
            return ("unknown_staff", path + ".initial_staff_id")
        if not tick(episode.get("ready_tick")) or episode["ready_tick"] >= horizon:
            return ("ready_tick", path + ".ready_tick")
        if not tick(episode.get("original_work_ticks")) or episode["original_work_ticks"] == 0:
            return ("work_ticks", path + ".original_work_ticks")
        urgency = episode.get("urgency")
        if not isinstance(urgency, int) or isinstance(urgency, bool) or not -(1 << 31) <= urgency < (1 << 31):
            return ("urgency", path + ".urgency")
        if not identifier(episode.get("patient_id")) or not identifier(episode.get("assigned_zone")) or not identifier(episode.get("sample_lineage")):
            return ("identifier", path)
        staff_count = episode.get("required_staff_count")
        if type(staff_count) is not int or staff_count != 1:
            return ("multi_person_unsupported", path + ".required_staff_count")
        required = episode.get("required_skills")
        if not isinstance(required, list) or len(required) > 32 or any(not identifier(x) for x in required):
            return ("skill_count", path + ".required_skills")
        if required != sorted(set(required), key=lambda x: x.encode("ascii")):
            return ("skills_not_sorted_unique", path + ".required_skills")
        assigned = staff_by_id[episode["initial_staff_id"]]
        if assigned["assigned_zone"] != episode["assigned_zone"] or not set(required).issubset(assigned["skills"]):
            return ("initial_assignment_ineligible", path + ".initial_staff_id")
        for consent in ("incumbent_consent", "resource_consent"):
            if not isinstance(episode.get(consent), bool):
                return ("consent_type", path + "." + consent)

    request_ids = set()
    for index, request in enumerate(requests):
        path = f"urgent_requests[{index}]"
        request_id = request.get("request_id")
        if not identifier(request_id):
            return ("identifier", path + ".request_id")
        if request_id in request_ids:
            return ("duplicate_id", path + ".request_id")
        request_ids.add(request_id)
        eid = request.get("episode_id")
        if eid not in episodes_by_id:
            return ("unknown_episode", path + ".episode_id")
        episode = episodes_by_id[eid]
        if not tick(request.get("ready_tick")) or request["ready_tick"] >= horizon:
            return ("ready_tick", path + ".ready_tick")
        if request["ready_tick"] != episode["ready_tick"] or request.get("urgency") != episode["urgency"]:
            return ("request_episode_mismatch", path)
        urgency = request.get("urgency")
        if not isinstance(urgency, int) or isinstance(urgency, bool) or not -(1 << 31) <= urgency < (1 << 31):
            return ("urgency", path + ".urgency")
        if request.get("patient_id") != episode["patient_id"]:
            return ("request_episode_mismatch", path + ".patient_id")
        work_ticks = request.get("work_ticks")
        if not tick(work_ticks) or work_ticks == 0:
            return ("work_ticks", path + ".work_ticks")
        if work_ticks != episode["original_work_ticks"]:
            return ("request_episode_mismatch", path + ".work_ticks")
        if not isinstance(request.get("incoming_consent"), bool):
            return ("consent_type", path + ".incoming_consent")
        if request.get("assigned_zone") != episode["assigned_zone"] or request.get("required_skills") != episode["required_skills"]:
            return ("request_eligibility_mismatch", path)
        staff_count = request.get("required_staff_count")
        if type(staff_count) is not int or staff_count != 1:
            return ("multi_person_unsupported", path + ".required_staff_count")

    if len(events) > len(episodes):
        return ("duplicate_handover", f"events[{len(episodes)}]")
    seen_handover = set()
    for index, event in enumerate(events):
        path = f"events[{index}]"
        eid = event.get("episode_id")
        if eid not in episodes_by_id:
            return ("unknown_episode", path + ".episode_id")
        episode = episodes_by_id[eid]
        incumbent = event.get("incumbent_staff_id")
        replacement = event.get("replacement_staff_id")
        if incumbent not in staff_by_id:
            return ("unknown_staff", path + ".incumbent_staff_id")
        if replacement not in staff_by_id:
            return ("unknown_staff", path + ".replacement_staff_id")
        if incumbent != episode["initial_staff_id"]:
            return ("wrong_incumbent", path + ".incumbent_staff_id")
        if eid in seen_handover:
            return ("duplicate_handover", path)
        seen_handover.add(eid)
        if not tick(event.get("tick")) or event["tick"] > horizon:
            return ("handover_tick", path + ".tick")
        person = staff_by_id[replacement]
        if person["assigned_zone"] != episode["assigned_zone"] or not set(episode["required_skills"]).issubset(person["skills"]):
            return ("replacement_ineligible", path + ".replacement_staff_id")
        if event.get("replacement_eligible_zone") is not True or event.get("replacement_eligible_skills") is not True:
            return ("replacement_eligibility_mismatch", path)
        if not (episode["incumbent_consent"] and episode["resource_consent"]):
            return ("handover_consent", path)

    return None



class InterruptFixtureTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.data = json.loads(FIXTURE.read_text())
        cls.cases = {case["case_id"]: case for case in cls.data["cases"]}

    def test_exact_inventory_and_native_binding_debt(self):
        self.assertEqual(set(self.cases), POSITIVE_IDS | NEGATIVE_IDS)
        self.assertEqual(len(self.data["cases"]), 24)
        self.assertEqual(len(self.cases), 24)
        self.assertEqual(self.data["case_ids"], [case["case_id"] for case in self.data["cases"]])
        self.assertEqual(self.data["native_binding"], {"status": "unbound", "target": "E2.2"})
        for case in self.cases.values():
            self.assertEqual(case["native_binding"], self.data["native_binding"])

    def test_every_positive_matches_literal_hand_oracle(self):
        self.assertEqual(set(HAND_ORACLES), POSITIVE_IDS)
        for case_id, oracle in HAND_ORACLES.items():
            with self.subTest(case=case_id):
                case = self.cases[case_id]
                expected = case["expected"]
                timeline = [(x["episode_id"], x["staff_id"], x["start_tick"], x["end_tick"])
                            for x in expected["work_intervals"]]
                final = {x["episode_id"]: (x["state"], x["remaining_work_ticks"])
                         for x in expected["final_states"]}
                waits = [(x["episode_id"], x["start_tick"], x["end_tick"], x["reason"], x["staff_id"])
                         for x in expected["wait_intervals"]]
                handovers = {(x["episode_id"], x["tick"]): x["outcome"]
                             for x in expected["handovers"]}
                self.assertEqual(timeline, oracle["timeline"])
                self.assertEqual(final, oracle["final"])
                self.assertEqual(waits, oracle["waits"])
                self.assertEqual(expected["consent_outcome"], oracle["consent"])
                literal_handovers = {(episode, tick): outcome
                                     for episode, (tick, outcome) in oracle["handovers"].items()}
                self.assertEqual(handovers, literal_handovers)
                self.assertEqual(expected["handover_noops"], oracle["noops"])
                self.assertEqual(expected["travel_intervals"], [])

    def test_positive_inputs_validate_and_bind_all_references(self):
        for case_id in POSITIVE_IDS:
            with self.subTest(case=case_id):
                case = self.cases[case_id]
                self.assertIsNone(validate_handover_inputs(case["inputs"], case["horizon"]))
                episodes = {x["episode_id"]: x for x in case["inputs"]["episodes"]}
                staff = {x["staff_id"]: x for x in case["inputs"]["staff"]}
                for event in case["inputs"]["events"]:
                    episode = episodes[event["episode_id"]]
                    replacement = staff[event["replacement_staff_id"]]
                    self.assertEqual(replacement["assigned_zone"], episode["assigned_zone"])
                    self.assertTrue(set(episode["required_skills"]).issubset(replacement["skills"]))
                segments = {x["segment_id"]: x for x in case["expected"]["work_intervals"]}
                for handover in case["expected"]["handovers"]:
                    if handover["outcome"] != "applied":
                        continue
                    grant = segments[handover["new_segment_id"]]
                    replacement = staff[grant["staff_id"]]
                    self.assertGreaterEqual(grant["start_tick"], handover["tick"])
                    self.assertTrue(any(lo <= grant["start_tick"] < hi for lo, hi in replacement["shifts"]))
                    self.assertFalse(any(lo <= grant["start_tick"] < hi for lo, hi in replacement["breaks"]))
                for segment in case["expected"]["work_intervals"]:
                    episode = episodes[segment["episode_id"]]
                    person = staff[segment["staff_id"]]
                    self.assertEqual(person["assigned_zone"], episode["assigned_zone"])
                    self.assertTrue(set(episode["required_skills"]).issubset(person["skills"]))
                    if segment["useful_work_ticks"] == 0:
                        person = staff[segment["staff_id"]]
                        start = segment["start_tick"]
                        self.assertTrue(any(lo <= start < hi for lo, hi in person["shifts"]))
                        self.assertFalse(any(lo <= start < hi for lo, hi in person["breaks"]))

    def test_validator_rejects_bad_references_ticks_calendars_and_consent(self):
        control = self.cases["interrupt.handover_immediate"]
        self.assertIsNone(validate_handover_inputs(control["inputs"], control["horizon"]))
        invalid = []

        value = copy.deepcopy(control["inputs"])
        value["episodes"][0]["initial_staff_id"] = "missing"
        invalid.append((value, "unknown_staff", "episodes[0].initial_staff_id"))

        value = copy.deepcopy(control["inputs"])
        value["events"][0]["episode_id"] = "missing"
        invalid.append((value, "unknown_episode", "events[0].episode_id"))

        value = copy.deepcopy(control["inputs"])
        value["events"][0]["replacement_staff_id"] = "missing"
        invalid.append((value, "unknown_staff", "events[0].replacement_staff_id"))

        value = copy.deepcopy(control["inputs"])
        value["episodes"][0]["ready_tick"] = -1
        invalid.append((value, "ready_tick", "episodes[0].ready_tick"))

        value = copy.deepcopy(control["inputs"])
        value["events"][0]["tick"] = True
        invalid.append((value, "handover_tick", "events[0].tick"))

        value = copy.deepcopy(control["inputs"])
        value["events"][0]["tick"] = control["horizon"] + 1
        invalid.append((value, "handover_tick", "events[0].tick"))

        value = copy.deepcopy(control["inputs"])
        value["staff"][0]["shifts"] = [[0, 10], [9, 20]]
        invalid.append((value, "calendar_overlap", "staff[0].shifts"))

        value = copy.deepcopy(control["inputs"])
        value["staff"][0]["shifts"] = [[10, 20], [0, 10]]
        self.assertIsNone(validate_handover_inputs(value, control["horizon"]))

        value = copy.deepcopy(control["inputs"])
        value["episodes"][0]["incumbent_consent"] = 1
        invalid.append((value, "consent_type", "episodes[0].incumbent_consent"))

        value = copy.deepcopy(control["inputs"])
        value["staff"][1]["assigned_zone"] = "other"
        invalid.append((value, "replacement_ineligible", "events[0].replacement_staff_id"))

        request_control = self.cases["interrupt.suspend_resume"]
        request_inputs = request_control["inputs"]
        self.assertIsNone(validate_handover_inputs(request_inputs, request_control["horizon"]))

        same_namespace_name = copy.deepcopy(request_inputs)
        same_namespace_name["urgent_requests"][0]["request_id"] = "urgent"
        self.assertIsNone(validate_handover_inputs(same_namespace_name, request_control["horizon"]))

        duplicate_request = copy.deepcopy(request_inputs)
        self.assertIsNone(validate_handover_inputs(duplicate_request, request_control["horizon"]))
        duplicate_request["urgent_requests"].append(
            copy.deepcopy(duplicate_request["urgent_requests"][0]))
        invalid.append((duplicate_request, "duplicate_id", "urgent_requests[1].request_id"))

        duplicate_episode = copy.deepcopy(control["inputs"])
        duplicate_episode["episodes"].append(copy.deepcopy(duplicate_episode["episodes"][0]))
        invalid.append((duplicate_episode, "duplicate_id", "episodes[1].episode_id"))

        for bad_count in (True, 1.0):
            value = copy.deepcopy(control["inputs"])
            self.assertIsNone(validate_handover_inputs(value, control["horizon"]))
            value["episodes"][0]["required_staff_count"] = bad_count
            invalid.append((value, "multi_person_unsupported", "episodes[0].required_staff_count"))

            value = copy.deepcopy(request_inputs)
            self.assertIsNone(validate_handover_inputs(value, request_control["horizon"]))
            value["urgent_requests"][0]["required_staff_count"] = bad_count
            invalid.append((value, "multi_person_unsupported", "urgent_requests[0].required_staff_count"))

        for bad_work in (True, 1.0, 0):
            value = copy.deepcopy(request_inputs)
            self.assertIsNone(validate_handover_inputs(value, request_control["horizon"]))
            value["urgent_requests"][0]["work_ticks"] = bad_work
            invalid.append((value, "work_ticks", "urgent_requests[0].work_ticks"))

        for inputs, category, path in invalid:
            with self.subTest(category=category, path=path):
                self.assertEqual(validate_handover_inputs(inputs, control["horizon"]),
                                 (category, path))

    def test_work_claim_history_and_segment_remainders_conserve_each_sample(self):
        for case_id in POSITIVE_IDS:
            with self.subTest(case=case_id):
                case = self.cases[case_id]
                segments = case["expected"]["work_intervals"]
                by_episode = {x["episode_id"]: x for x in case["inputs"]["episodes"]}
                for index, left in enumerate(segments):
                    self.assertLessEqual(0, left["start_tick"])
                    self.assertLessEqual(left["start_tick"], left["end_tick"])
                    self.assertLessEqual(left["end_tick"], case["horizon"])
                    self.assertEqual(left["useful_work_ticks"], left["end_tick"] - left["start_tick"])
                    for right in segments[index + 1:]:
                        if left["staff_id"] == right["staff_id"]:
                            self.assertFalse(overlaps((left["start_tick"], left["end_tick"]),
                                                      (right["start_tick"], right["end_tick"])))
                for episode_id, episode in by_episode.items():
                    own = [x for x in segments if x["episode_id"] == episode_id]
                    remaining = episode["original_work_ticks"]
                    for segment in own:
                        self.assertEqual(segment["sample_lineage"], episode["sample_lineage"])
                        self.assertEqual(segment["remaining_work_at_end"], remaining - segment["useful_work_ticks"])
                        remaining = segment["remaining_work_at_end"]
                    final = next(x for x in case["expected"]["final_states"] if x["episode_id"] == episode_id)
                    self.assertEqual(remaining, final["remaining_work_ticks"])
                    self.assertEqual(sum(x["useful_work_ticks"] for x in own) + remaining,
                                     episode["original_work_ticks"])

    def test_capacity_calendar_consent_and_priority_edges(self):
        for case_id in POSITIVE_IDS:
            case = self.cases[case_id]
            people = {x["staff_id"]: x for x in case["inputs"]["staff"]}
            for segment in case["expected"]["work_intervals"]:
                person = people[segment["staff_id"]]
                self.assertTrue(any(lo <= segment["start_tick"] < hi for lo, hi in person["shifts"]))
                self.assertFalse(any(lo <= segment["start_tick"] < hi for lo, hi in person["breaks"]))
        for suffix in ("resource_consent_absent", "incumbent_consent_absent"):
            case = self.cases["interrupt." + suffix]
            incumbent = next(x for x in case["inputs"]["episodes"] if x["episode_id"] == "ep")
            key = "resource_consent" if suffix.startswith("resource") else "incumbent_consent"
            self.assertFalse(incumbent[key])
            self.assertEqual(case["expected"]["interruptions"], [])
        incoming = self.cases["interrupt.incoming_consent_absent"]
        self.assertFalse(incoming["inputs"]["urgent_requests"][0]["incoming_consent"])
        self.assertEqual(incoming["expected"]["interruptions"], [])
        priority = self.cases["interrupt.priority_extremes"]
        mapping = priority["inputs"]["priority_mapping"]
        self.assertEqual([mapping[x]["flow_priority"] for x in ("i32_min", "zero", "i32_max")],
                         [2147483647, -1, -2147483648])
        self.assertEqual([x["episode_id"] for x in priority["expected"]["work_intervals"]],
                         ["max", "zero", "min"])

    def test_suspend_is_not_release_and_waits_have_no_claim(self):
        for case_id in ("interrupt.suspend_resume", "interrupt.break_precedence",
                        "interrupt.shift_precedence", "interrupt.horizon_suspended"):
            case = self.cases[case_id]
            for event in case["expected"]["interruptions"]:
                self.assertEqual(event["claim_action"], "Suspend")
                self.assertFalse(event["explicit_release"])
            for segment in case["expected"]["work_intervals"]:
                if segment["claim_state"] == "suspended":
                    self.assertEqual(segment["claim_history"][-1], "Suspended")
                if segment["segment_id"].endswith("-2"):
                    self.assertEqual(segment["claim_history"][0], "Resumed")
        for case in self.cases.values():
            if case["kind"] != "positive":
                continue
            for waiting in case["expected"]["wait_intervals"]:
                self.assertFalse(waiting["holds_staff_claim"])
                self.assertLess(waiting["start_tick"], waiting["end_tick"])
        break_case = self.cases["interrupt.break_precedence"]
        shift_case = self.cases["interrupt.shift_precedence"]
        self.assertEqual(break_case["inputs"]["staff"][0]["breaks"], [[3, 5]])
        self.assertEqual(shift_case["inputs"]["staff"][0]["shifts"], [[0, 3], [5, 20]])
        for case in (break_case, shift_case):
            observation = case["expected"]["calendar_observations"][0]
            self.assertEqual(observation["interval"], [3, 5])
            self.assertEqual(observation["overlap_with_low_ticks"], 2)
            self.assertEqual(observation["scheduled_unavailable_work_ticks"], 2)
        self.assertEqual(break_case["expected"]["calendar_observations"][0]["break_overlap_ticks"], 2)
        self.assertEqual(shift_case["expected"]["calendar_observations"][0]["shift_overtime_ticks"], 2)

    def test_handover_release_noops_and_permutation(self):
        for case_id in ("interrupt.handover_immediate", "interrupt.handover_wait",
                        "interrupt.horizon_handover", "interrupt.permutation"):
            case = self.cases[case_id]
            for handover in case["expected"]["handovers"]:
                self.assertEqual(handover["old_claim_action"], "Release")
                old = [x for x in case["expected"]["work_intervals"]
                       if x["episode_id"] == handover["episode_id"]
                       and x["staff_id"] == handover["incumbent_staff_id"]]
                self.assertEqual(len(old), 1)
                self.assertEqual(old[0]["claim_history"].count("Released"), 1)
        for suffix, outcome in (("handover_completion_tie", "completed_before_handover"),
                                ("handover_not_active", "not_active")):
            case = self.cases["interrupt." + suffix]
            self.assertEqual(case["expected"]["handover_noops"], [outcome])
            self.assertFalse(any(x["staff_id"] == "b" for x in case["expected"]["work_intervals"]))
        case = self.cases["interrupt.permutation"]
        self.assertEqual(normalized_inputs(case["inputs"]), normalized_inputs(case["permuted_input"]))
        handovers = {(x["episode_id"], x["tick"]): x["outcome"] for x in case["expected"]["handovers"]}
        literal = {(episode, tick): outcome for episode, (tick, outcome)
                   in HAND_ORACLES["interrupt.permutation"]["handovers"].items()}
        self.assertEqual(handovers, literal)

    def test_negative_controls_are_valid_single_mutations_with_exact_repairs(self):
        for case_id in NEGATIVE_IDS:
            with self.subTest(case=case_id):
                case = self.cases[case_id]
                baseline = copy.deepcopy(case["baseline"])
                changed = copy.deepcopy(baseline)
                mutation = case["mutation"]
                if mutation.get("operation") == "duplicate":
                    changed["events"].append(copy.deepcopy(changed["events"][0]))
                else:
                    setpath(changed, mutation["path"], mutation["value"])
                self.assertIsNone(validate_handover_inputs(baseline, case["baseline_horizon"]))
                error = validate_handover_inputs(changed, case["baseline_horizon"])
                self.assertIsNotNone(error)
                self.assertEqual(error[0], case["expected_rejection"]["category"])
                self.assertEqual(error[1], case["expected_rejection"]["path"])
                repaired = copy.deepcopy(changed)
                repair = case["targeted_repair"]
                if repair.get("operation") == "remove":
                    repaired["events"].pop()
                else:
                    setpath(repaired, repair["path"], repair["value"])
                expected_change = (case["expected_rejection"]["path"]
                                   if mutation.get("operation") == "duplicate"
                                   else mutation["path"])
                self.assertEqual(differing_paths(baseline, changed), [expected_change])
                self.assertEqual(repaired, baseline)
                self.assertIsNone(validate_handover_inputs(repaired, case["baseline_horizon"]))
