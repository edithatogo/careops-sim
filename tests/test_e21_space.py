"""Specification checks for E2.1 space fixtures; never runs a simulator."""
import copy
import hashlib
import json
import re
import unittest
from pathlib import Path

FIXTURE=Path(__file__).parents[1]/"conductor/design/ed/e2-space.json"
NATIVE={"status":"unbound","target":"E2.2"}
POSITIVE={
 "space.cleaning_blocks_reuse","space.cleaner_wait_dirty","space.cleaning_calendar","space.cleaning_horizon",
 "space.reservation_blocks","space.reservation_expiry_tie","space.reservation_consumption","space.reservation_wrong_claimant",
 "space.reservation_conflict","space.reservation_occupied","space.reservation_dirty","space.diagnostic_serial",
 "space.diagnostic_unavailable","space.diagnostic_staff_wait","space.boarding_retention","space.boarding_care",
 "space.unused_offer","space.withdrawn_offer","space.transfer_horizon","space.permutation",
}
NEGATIVE={
 "space.invalid.duplicate_space_id","space.invalid.count_mismatch","space.invalid.zero_static_capacity",
 "space.invalid.cleaning_nonpositive_work","space.invalid.unstaffed_cleaning_policy","space.invalid.duplicate_reservation_id",
 "space.invalid.reservation_nonpositive_interval","space.invalid.reservation_unknown_space","space.invalid.reservation_unknown_patient",
 "space.invalid.reservation_start_after_horizon","space.invalid.atomic_staff_space","space.invalid.multi_role_work",
}

# Supplementary drift pins for reviewed expected objects; these hashes are not
# independent behavioral oracles. TIMELINE_ORACLE below provides those assertions.
EXPECTED_SHA256={
 "space.cleaning_blocks_reuse":"16d210a0933b7f6b83e915a57d3e96863e552fc938a8592225a504dc1358b29d",
 "space.cleaner_wait_dirty":"b32f2f8ad9a93a799eda54a76755f8759d7368ac365bd83725475979719938ba",
 "space.cleaning_calendar":"02d6482b6f5028e0d830c12f323985fe38aec549e3b6f35313a81117084cbfb3",
 "space.cleaning_horizon":"984ece7969d70efe19088af3629f73a707ad7c62b4aff3c60aaaa5da36ceb34a",
 "space.reservation_blocks":"3965633390043b091e9c8ce63fd3a874113c27208b8fde60a316ba382a658c1e",
 "space.reservation_expiry_tie":"c2bb5da370a35c7bf02fc100c96a62f2cad2822af631fc1efba590fbf5c676ae",
 "space.reservation_consumption":"cf943051f7e118ce390cd5b803cfa4feae251e8bd2543b1f1e28ee94181efdd8",
 "space.reservation_wrong_claimant":"81c8732fb41b07de4cebfb242aa7f06f7e27def08612eb1ec2a9c5471079f8b6",
 "space.reservation_conflict":"6f6797727a6230005720272ee475b4b779943ea978c3b9728e49ed7e9d7a3b2f",
 "space.reservation_occupied":"aa7314a236e98bf69b0ec90f7977447dd10d46b5c771914bab6608cb925f76ac",
 "space.reservation_dirty":"25854cf410989eefb815e56a8de08a70272adf9581271ceac25f405f9ffb1cb2",
 "space.diagnostic_serial":"5aa735c6075da82b988f4883c005c61765fae1fa4ded10ddfd4cd1ecdfea942d",
 "space.diagnostic_unavailable":"7956667bb83e94517e403f8955fb133e480183b419d125685eae81b754f8d9be",
 "space.diagnostic_staff_wait":"7d7b96a6cdeaf49277e2343aa0eb1860358706d838b4a75b922f4a8282e2469f",
 "space.boarding_retention":"969cbb06a8c17d24de60965dbc838d87824f8368e183b08c84951103d1bfa412",
 "space.boarding_care":"121edcc2e5e97e6f548704fc8e913a40b4b8320ec839e3a4002357a24fd9020b",
 "space.unused_offer":"fd7c9acfba681a885689a802e9e5c1affb8a35ef27b28c9237d3baa6a9da3041",
 "space.withdrawn_offer":"92fbac1de512de9f7b8e67462147c3a5cab4ca5a959731b0d36f84d350fa5d13",
 "space.transfer_horizon":"4778cf272e86693d72270b669e46dc6fe09e4fd416f817c39ba4070aaef1d218",
 "space.permutation":"cf0526c020baea1c4e30c21b21a5ba27f4a0a493d46f9144868ae1c898295c62",
}
HAND_WITNESS={
 "space.cleaning_blocks_reuse":"A[1,3); clean[3,5); B[5,H); no dirty interval is available to B.",
 "space.cleaner_wait_dirty":"other cleaner work[3,5); dirty[3,5); clean[5,7); B[7,H).",
 "space.cleaning_calendar":"cleaner unavailable[3,5); dirty[3,5); clean[5,7); B[7,H).",
 "space.cleaning_horizon":"clean completes H=5; B is granted at5 with zero observed occupancy and remains unfinished.",
 "space.reservation_blocks":"B reservation[2,6) expires unused; A acquires6; B waits7-H.",
 "space.reservation_expiry_tie":"expiry5 precedes grants; A acquires5; B waits5-H.",
 "space.reservation_consumption":"B consumes[2,6) at3; departs7; expiry6 is stale; clean7-9.",
 "space.reservation_wrong_claimant":"A is rejected at2; B consumes3; expiry6 is stale; A waits2-H.",
 "space.reservation_conflict":"rA wins ID tie and A consumes2; rB rejected; stale expiry6; B waits2-H.",
 "space.reservation_occupied":"A occupies1-5; rejected reservation does not retry; B waits through clean5-7 and acquires7.",
 "space.reservation_dirty":"dirty3-5 with no cleaner claim; rejected at4/6; clean5-7; B acquires7.",
 "space.diagnostic_serial":"diag.A room3-5, diag.B room5-7; B room wait3-5; ED spaces retained.",
 "space.diagnostic_unavailable":"room granted3 and held through H10; staff wait has no claim.",
 "space.diagnostic_staff_wait":"room3-7; other staff work1-5; diagnostic staff wait3-5 then work5-7.",
 "space.boarding_retention":"decisions/offers4; transfers A4-6/B6-8; physical departures6/8; cleaning6-8/8-10.",
 "space.boarding_care":"decision4; care6-8; offer7 allocates8; transfer8-10; departure10; clean10-12.",
 "space.unused_offer":"compatible A occupies1-H; no admission decision; offer2-6 unused; no transfer.",
 "space.withdrawn_offer":"boarder decision4; offer withdrawn4; no allocation; A occupies through H.",
 "space.transfer_horizon":"decision/offer4; configured transfer4-8, observed2 at H6; occupancy remains and no terminal LOS.",
 "space.permutation":"rA/rZ consumed by A/Z at1; Z occupies room.z; B waits6-H; reverse input arrays preserve canonical history.",
}

# Independent, readable hand calculations. Columns are complete observed states,
# work claims, patient occupancy, and waits. Empty text means no rows, not unchecked.
# Room a/b/z abbreviate the fixed room.a/room.b/room.z fixture identities.
TIMELINE_ORACLE = {
 "cleaning_blocks_reuse": (
  "a ready 0 1|a occupied 1 3|a cleaning 3 5|a occupied 5 10",
  "clean.room.a cleaner.1 3 5", "A a 1 3|B a 5 10", "space B 1 5"),
 "cleaner_wait_dirty": (
  "a ready 0 1|a occupied 1 3|a dirty_waiting 3 5|a cleaning 5 7|a occupied 7 10",
  "stock.cleaning_cart cleaner.1 3 5|clean.room.a cleaner.1 5 7", "A a 1 3|B a 7 10",
  "space B 1 7|staff clean.room.a 3 5|cleaner room.a 3 5"),
 "cleaning_calendar": (
  "a ready 0 1|a occupied 1 3|a dirty_waiting 3 5|a cleaning 5 7|a occupied 7 10",
  "clean.room.a cleaner.1 5 7", "A a 1 3|B a 7 10", "space B 3 7|staff clean.room.a 3 5"),
 "cleaning_horizon": (
  "a ready 0 1|a occupied 1 3|a cleaning 3 5",
  "clean.room.a cleaner.1 3 5", "A a 1 3", "space B 3 5"),
 "reservation_blocks": ("a ready 0 2|a reserved 2 6|a occupied 6 10", "", "A a 6 10", "space A 2 6|space B 7 10"),
 "reservation_expiry_tie": ("a ready 0 2|a reserved 2 5|a occupied 5 10", "", "A a 5 10", "space A 2 5|space B 5 10"),
 "reservation_consumption": ("a ready 0 2|a reserved 2 3|a occupied 3 7|a cleaning 7 9|a ready 9 10", "clean.room.a cleaner.1 7 9", "B a 3 7", ""),
 "reservation_wrong_claimant": ("a ready 0 2|a reserved 2 3|a occupied 3 10", "", "B a 3 10", "space A 2 10"),
 "reservation_conflict": ("a ready 0 2|a reserved 2 2|a occupied 2 10", "", "A a 2 10", "space B 2 10"),
 "reservation_occupied": ("a ready 0 1|a occupied 1 5|a dirty_waiting 5 5|a cleaning 5 7|a occupied 7 10", "clean.room.a cleaner.1 5 7", "A a 1 5|B a 7 10", "space B 2 7"),
 "reservation_dirty": (
  "a ready 0 1|a occupied 1 3|a dirty_waiting 3 5|a cleaning 5 7|a occupied 7 10",
  "clean.room.a cleaner.1 5 7", "A a 1 3|B a 7 10", "space B 3 7|staff clean.room.a 3 5|cleaner room.a 3 5"),
 "diagnostic_serial": ("a ready 0 1|a occupied 1 10|b ready 0 1|b occupied 1 10", "staff.diag.A worker.1 3 5|staff.diag.B worker.1 5 7", "A a 1 10|B b 1 10", "room diag.B 3 5"),
 "diagnostic_unavailable": ("a ready 0 1|a occupied 1 10", "", "A a 1 10", "staff staff.diag.A 3 10"),
 "diagnostic_staff_wait": ("a ready 0 1|a occupied 1 10", "busy.other worker.1 1 5|staff.diag.A worker.1 5 7", "A a 1 10", "staff staff.diag.A 3 5"),
 "boarding_retention": ("a ready 0 1|a occupied 1 6|a cleaning 6 8|a ready 8 10|b ready 0 1|b occupied 1 8|b cleaning 8 10", "tA transfer.1 4 6|tB transfer.1 6 8|clean.A cleaner.1 6 8|clean.B cleaner.1 8 10", "A a 1 6|B b 1 8", "staff tB 4 6"),
 "boarding_care": ("a ready 0 1|a occupied 1 10|a cleaning 10 12", "care.A care.1 6 8|tA transfer.1 8 10|clean.A cleaner.1 10 12", "A a 1 10", ""),
 "unused_offer": ("a ready 0 1|a occupied 1 10", "", "A a 1 10", ""),
 "withdrawn_offer": ("a ready 0 1|a occupied 1 10", "", "A a 1 10", ""),
 "transfer_horizon": ("a ready 0 1|a occupied 1 6", "tA transfer.1 4 8", "A a 1 6", ""),
 "permutation": ("a reserved 0 1|a occupied 1 10|z reserved 0 1|z occupied 1 10", "", "A a 1 10|Z z 1 10", "space B 6 10"),
}
RESERVATION_ORACLE = {
 "reservation_blocks": "rB accepted 2|rB expired 6",
 "reservation_expiry_tie": "rB accepted 2|rB expired 5",
 "reservation_consumption": "rB accepted 2|rB consumed 3|rB stale_expiry_noop 6",
 "reservation_wrong_claimant": "rB accepted 2|rB wrong_claimant_rejected 2|rB consumed 3|rB stale_expiry_noop 6",
 "reservation_conflict": "rA accepted 2|rB already_reserved 2|rA consumed 2|rA stale_expiry_noop 6",
 "reservation_occupied": "rOcc occupied 2",
 "reservation_dirty": "rDirty dirty 4|rClean cleaning 6",
 "permutation": "rA accepted 0|rZ accepted 0|rA consumed 1|rZ consumed 1|rA stale_expiry_noop 2|rZ stale_expiry_noop 2",
}
DIAGNOSTIC_ORACLE = {
 "diagnostic_serial": "diag.A diag.1 3 5|diag.B diag.1 5 7",
 "diagnostic_unavailable": "diag.A diag.1 3 10",
 "diagnostic_staff_wait": "diag.A diag.1 3 7",
}
# Arrived, discharged, transferred, unfinished, ready spaces at H.
CENSUS_ORACLE = {
 "cleaning_blocks_reuse":(2,1,0,1,0), "cleaner_wait_dirty":(2,1,0,1,0),
 "cleaning_calendar":(2,1,0,1,0), "cleaning_horizon":(2,1,0,1,0),
 "reservation_blocks":(2,0,0,2,0), "reservation_expiry_tie":(2,0,0,2,0),
 "reservation_consumption":(1,1,0,0,1), "reservation_wrong_claimant":(2,0,0,2,0),
 "reservation_conflict":(2,0,0,2,0), "reservation_occupied":(2,1,0,1,0),
 "reservation_dirty":(2,1,0,1,0), "diagnostic_serial":(2,0,0,2,0),
 "diagnostic_unavailable":(1,0,0,1,0), "diagnostic_staff_wait":(1,0,0,1,0),
 "boarding_retention":(2,0,2,0,2), "boarding_care":(1,0,1,0,1),
 "unused_offer":(1,0,0,1,0), "withdrawn_offer":(1,0,0,1,0),
 "transfer_horizon":(1,0,0,1,0), "permutation":(3,0,0,3,0),
}

def literal_rows(text):
    return sorted(text.split("|")) if text else []

def assert_literal_timeline(test,case):
    suffix=case["case_id"].removeprefix("space.");e=case["expected"]
    rows=lambda values: sorted(" ".join(map(str,row)) for row in values)
    short=lambda sid:sid.removeprefix("room.")
    actual=(
      rows((short(r["space_id"]),r["state"],r["start_tick"],r["end_tick"]) for r in e["space_lifecycle"]),
      rows((r["request_id"],r["resource_id"],r["start_tick"],r["configured_end_tick"]) for r in e["work_claims"]),
      rows((r["patient_id"],short(r["space_id"]),r["start_tick"],r["end_tick"]) for r in e["patient_space_intervals"]),
      rows((kind,r[key],r["start_tick"],r["end_tick"]) for kind,key in (("space","patient_id"),("staff","request_id"),("cleaner","space_id"),("room","request_id")) for r in e.get(kind+"_waits",[])),
    )
    test.assertEqual(actual,tuple(literal_rows(v) for v in TIMELINE_ORACLE[suffix]))
    reservation=rows((r["reservation_id"],r.get("reason",r["outcome"]),r.get("tick",r.get("interval",[None])[0])) for r in e["reservation_outcomes"])
    test.assertEqual(reservation,literal_rows(RESERVATION_ORACLE.get(suffix,"")))
    diagnostic=rows((r["request_id"],r["room_id"],r["start_tick"],r["end_tick"]) for r in e["diagnostic_room_intervals"])
    test.assertEqual(diagnostic,literal_rows(DIAGNOSTIC_ORACLE.get(suffix,"")))
    test.assertEqual(tuple(e["accounting"].get(k,0) for k in ("arrived_patients","discharged_patients","transferred_patients","unfinished_patients","ready_spaces_at_horizon")),CENSUS_ORACLE[suffix])
    if suffix=="cleaning_horizon":
        test.assertEqual(e["patient_space_grants"],[{"patient_id":"B","space_id":"room.a","tick":5,"observed_occupancy_ticks":0,"active_at_horizon":True}])
        test.assertTrue(e["accounting"]["cleaner_completed_at_horizon"])
        test.assertEqual(e["accounting"]["cleaner_claims_at_horizon"],0)
    if suffix=="boarding_retention":
        test.assertEqual([(r["patient_id"],r["decision_tick"],r["offer_id"],r["offer_consumed_tick"],r["transfer_interval"],r["physical_departure_tick"]) for r in e["boarding_events"]],[("A",4,"oA",4,[4,6],6),("B",4,"oB",4,[6,8],8)])
    if suffix=="boarding_care":
        r=e["boarding_events"][0]
        test.assertEqual((r["decision_tick"],r["care_interval"],r["offer_tick"],r["offer_allocation_tick"],r["transfer_interval"],r["physical_departure_tick"]),(4,[6,8],7,8,[8,10],10))
        test.assertTrue(e["accounting"]["cleaner_completed_at_horizon"])
    if suffix=="unused_offer":
        test.assertEqual(e["boarding_events"],[{"patient_id":"A","offer_id":"unused","state":"unused_expired","expiry_tick":6,"transfer_started":False}])
    if suffix=="withdrawn_offer":
        r=e["boarding_events"][0];test.assertEqual((r["state"],r["withdrawn_tick"],r["allocated"]),("withdrawn",4,False))
    if suffix=="transfer_horizon":
        r=e["boarding_events"][0];test.assertEqual((r["decision_tick"],r["offer_consumed_tick"],r["transfer_interval"],r["observed_transfer_ticks"],r["physical_departure_tick"]),(4,4,[4,8],2,None))
        test.assertIsNone(e["accounting"]["terminal_outcome"]);test.assertIsNone(e["accounting"]["completed_los"])

MISSING=object()
def path_value(value,path):
    for word,index in re.findall(r"([^.\[\]]+)|\[(\d+)\]",path):
        key=int(index) if index else word
        if isinstance(value,dict) and key in value:value=value[key]
        elif isinstance(value,list) and isinstance(key,int) and key<len(value):value=value[key]
        else:return MISSING
    return value

def strict_int(v,lo=0,hi=(1<<64)-1): return isinstance(v,int) and not isinstance(v,bool) and lo<=v<=hi
def identifier(v): return isinstance(v,str) and v.isascii() and 1<=len(v)<=64
def iid(v): return v.encode("ascii")

def valid_input(envelope):
    """Full bounded contract validator; no scheduling or event simulation."""
    def bad(cat,path): return (False,cat,path)
    if not isinstance(envelope,dict) or not strict_int(envelope.get("horizon"),1): return bad("horizon","horizon")
    h=envelope["horizon"]
    if envelope.get("in_flight_policy")!="DrainActive": return bad("in_flight_policy","in_flight_policy")
    x=envelope.get("inputs")
    if not isinstance(x,dict): return bad("inputs","inputs")
    collection_names=("spaces","patients","physical_departures","reservations","diagnostic_rooms","diagnostic_requests","offers","care_requests","transfers","staff","work_requests")
    if any(not isinstance(x.get(k),list) for k in collection_names): return bad("collection","inputs")
    spaces=x["spaces"]; patients=x["patients"]; reservations=x["reservations"]; staff=x["staff"]; requests=x["work_requests"]
    for name,rows,limit in (("spaces",spaces,1024),("patients",patients,100000),("reservations",reservations,100000),("diagnostic_rooms",x["diagnostic_rooms"],1024),("diagnostic_requests",x["diagnostic_requests"],100000),("offers",x["offers"],100000),("care_requests",x["care_requests"],100000),("transfers",x["transfers"],100000),("staff",staff,1024),("work_requests",requests,100000)):
        if len(rows)>limit:return bad(name+"_count","inputs."+name)
    def unique(rows,key,path,category="duplicate_id"):
        seen=set()
        for n,row in enumerate(rows):
            if not isinstance(row,dict):return bad("row",f"inputs.{path}[{n}]")
            v=row.get(key)
            if not identifier(v):return bad("identifier",f"inputs.{path}[{n}].{key}")
            if v in seen:return bad(category,f"inputs.{path}[{n}].{key}")
            seen.add(v)
        return None
    namespaces=((spaces,"space_id","spaces","duplicate_id"),(patients,"patient_id","patients","duplicate_id"),(reservations,"reservation_id","reservations","duplicate_reservation_id"),(x["diagnostic_rooms"],"room_id","diagnostic_rooms","duplicate_id"),(x["diagnostic_requests"],"request_id","diagnostic_requests","duplicate_id"),(x["offers"],"offer_id","offers","duplicate_id"),(x["care_requests"],"request_id","care_requests","duplicate_id"),(x["transfers"],"transfer_id","transfers","duplicate_id"),(staff,"staff_id","staff","duplicate_id"),(requests,"request_id","work_requests","duplicate_id"))
    for rows,key,path,cat in namespaces:
        result=unique(rows,key,path,cat)
        if result:return result
    profile=x.get("space_capacity_profile")
    if not isinstance(profile,dict):return bad("capacity","inputs.space_capacity_profile")
    for name in ("physical","open","staffed","named_spaces"):
        if name=="physical" and profile.get(name)==0:return bad("zero_static_capacity","inputs.space_capacity_profile.physical")
        if not strict_int(profile.get(name),1,1024):return bad("capacity","inputs.space_capacity_profile."+name)
    if profile["named_spaces"]!=len(spaces):return bad("count_mismatch","inputs.space_capacity_profile.named_spaces")
    if not spaces or any(profile[k]!=len(spaces) for k in ("physical","open","staffed")):
        return bad("count_mismatch","inputs.space_capacity_profile")
    sidset={s["space_id"] for s in spaces}; pids={p["patient_id"] for p in patients}; staffs={s["staff_id"]:s for s in staff}; reqs={r["request_id"]:r for r in requests}; rooms={r["room_id"]:r for r in x["diagnostic_rooms"]}
    for n,p in enumerate(patients):
        if not strict_int(p.get("arrival_tick")) or p["arrival_tick"]>=h:return bad("arrival_tick",f"inputs.patients[{n}].arrival_tick")
    for n,s in enumerate(spaces):
        if not identifier(s.get("assigned_zone")):return bad("identifier",f"inputs.spaces[{n}].assigned_zone")
        if s.get("initial_state") not in {"ready","dirty_waiting"}:return bad("space_state",f"inputs.spaces[{n}].initial_state")
        compatible=s.get("compatible_patient_ids")
        if not isinstance(compatible,list) or any(not identifier(p) or p not in pids for p in compatible) or len(compatible)!=len(set(compatible)):
            return bad("unknown_patient",f"inputs.spaces[{n}].compatible_patient_ids")
    # All calendar input is validated in normalized order; file order is immaterial.
    for n,s in enumerate(staff):
        if isinstance(s.get("shifts"),list) and isinstance(s.get("breaks"),list) and len(s["shifts"])+len(s["breaks"])>1024:return bad("calendar_count",f"inputs.staff[{n}]")
        if not identifier(s.get("assigned_zone")):return bad("identifier",f"inputs.staff[{n}].assigned_zone")
        skills=s.get("skills")
        if not isinstance(skills,list) or len(skills)>32 or any(not identifier(v) for v in skills) or skills!=sorted(set(skills),key=iid):return bad("skills_not_sorted_unique",f"inputs.staff[{n}].skills")
        for kind in ("shifts","breaks"):
            rows=s.get(kind)
            if not isinstance(rows,list) or len(rows)>1024:return bad("calendar",f"inputs.staff[{n}].{kind}")
            spans=[]
            for j,q in enumerate(rows):
                if not isinstance(q,list) or len(q)!=2 or not strict_int(q[0]) or not strict_int(q[1]) or q[0]>=q[1]:return bad("calendar_interval",f"inputs.staff[{n}].{kind}[{j}]")
                spans.append((q[0],q[1]))
            spans.sort()
            if any(a[1]>b[0] for a,b in zip(spans,spans[1:])):return bad("calendar_overlap",f"inputs.staff[{n}].{kind}")
        for j,(a,b) in enumerate(s.get("breaks",[])):
            if not any(lo<=a<b<=hi for lo,hi in s.get("shifts",[])):return bad("break_containment",f"inputs.staff[{n}].breaks[{j}]")
    for n,r in enumerate(requests):
        path=f"inputs.work_requests[{n}]"
        if r.get("subject_kind") not in {"space","patient","non_patient"}:return bad("subject_kind",path+".subject_kind")
        if r["subject_kind"]=="space":
            if r.get("space_id") not in sidset or "patient_id" in r:return bad("unknown_space",path+".space_id")
        elif r["subject_kind"]=="patient":
            if r.get("patient_id") not in pids or "space_id" in r:return bad("unknown_patient",path+".patient_id")
        elif "patient_id" in r or "space_id" in r:return bad("subject_kind",path)
        if not identifier(r.get("assigned_zone")):return bad("identifier",path+".assigned_zone")
        reqskills=r.get("required_skills")
        if not isinstance(reqskills,list) or len(reqskills)>32 or any(not identifier(v) for v in reqskills) or reqskills!=sorted(set(reqskills),key=iid):return bad("skills_not_sorted_unique",path+".required_skills")
        if not strict_int(r.get("ready_tick")) or r["ready_tick"]>=h:return bad("ready_tick",path+".ready_tick")
        if not strict_int(r.get("work_ticks"),1):return bad("work_ticks",path+".work_ticks")
        u=r.get("urgency")
        if not strict_int(u,-(1<<31),(1<<31)-1):return bad("urgency",path+".urgency")
        if not strict_int(r.get("required_staff_count"),1,1):return bad("multi_role_work",path+".required_staff_count")
        if "atomic_resources" in r:return bad("atomic_staff_space",path+".atomic_resources")
    # Static cleaning config and its per-space work requests are joined to physical departures.
    policy=x.get("cleaning_policy")
    if not isinstance(policy,dict) or policy.get("mode") not in {"none","staffed_work"}:return bad("unstaffed_cleaning_policy","inputs.cleaning_policy.mode")
    if policy["mode"]=="none":
        if set(policy)!={"mode"}:return bad("cleaning_policy","inputs.cleaning_policy")
        if x["physical_departures"] or any(r["subject_kind"]=="space" for r in requests):return bad("unstaffed_cleaning_policy","inputs.cleaning_policy.mode")
    else:
        ticks=policy.get("cleaning_work_ticks")
        if not strict_int(ticks,1):return bad("cleaning_nonpositive_work","inputs.cleaning_policy.cleaning_work_ticks")
        cleaner=staffs.get(policy.get("cleaner_staff_id"))
        if cleaner is None:return bad("unknown_staff","inputs.cleaning_policy.cleaner_staff_id")
        zone=policy.get("assigned_zone"); skills=policy.get("required_skills")
        if not identifier(zone):return bad("cleaner_eligibility","inputs.cleaning_policy.assigned_zone")
        if not isinstance(skills,list) or len(skills)>32 or any(not identifier(v) for v in skills) or skills!=sorted(set(skills),key=iid):return bad("cleaner_eligibility","inputs.cleaning_policy.required_skills")
        deps=x["physical_departures"]
        for n,d in enumerate(deps):
            if not isinstance(d,dict):return bad("row",f"inputs.physical_departures[{n}]")
            if d.get("patient_id") not in pids or d.get("space_id") not in sidset:return bad("unknown_reference",f"inputs.physical_departures[{n}]")
            if not strict_int(d.get("tick")) or d["tick"]>h:return bad("tick",f"inputs.physical_departures[{n}].tick")
            tasks=[r for r in requests if r["subject_kind"]=="space" and r.get("space_id")==d["space_id"]]
            if len(tasks)!=1:return bad("cleaning_request_join",f"inputs.physical_departures[{n}].space_id")
            task=tasks[0]
            if task["ready_tick"]!=d["tick"] or task["work_ticks"]!=ticks or task["assigned_zone"]!=zone or task["required_skills"]!=skills:return bad("cleaning_request_join",f"inputs.work_requests.{task['request_id']}")
        space_tasks=[r for r in requests if r["subject_kind"]=="space"]
        if len(space_tasks)!=len(deps):return bad("cleaning_request_join","inputs.work_requests")
        if policy.get("cleaner_request_id") is not None and (len(space_tasks)!=1 or policy["cleaner_request_id"]!=space_tasks[0]["request_id"]):return bad("cleaning_request_join","inputs.cleaning_policy.cleaner_request_id")
    for n,r in enumerate(reservations):
        path=f"inputs.reservations[{n}]"; a,b=r.get("start_tick"),r.get("expiry_tick")
        if not strict_int(a) or not strict_int(b) or a>=b:return bad("reservation_interval",path+".expiry_tick")
        if a>h:return bad("reservation_start_after_horizon",path+".start_tick")
        if r.get("space_id") not in sidset:return bad("unknown_space",path+".space_id")
        if r.get("patient_id") not in pids:return bad("unknown_patient",path+".patient_id")
        if r["patient_id"] not in next(s["compatible_patient_ids"] for s in spaces if s["space_id"]==r["space_id"]):return bad("incompatible_patient",path+".patient_id")
    for n,r in enumerate(x["diagnostic_rooms"]):
        if not identifier(r.get("assigned_zone")) or not strict_int(r.get("capacity"),1,1024):return bad("diagnostic_room",f"inputs.diagnostic_rooms[{n}]")
    for n,r in enumerate(x["diagnostic_requests"]):
        path=f"inputs.diagnostic_requests[{n}]"
        if r.get("room_id") not in rooms:return bad("unknown_room",path+".room_id")
        if r.get("patient_id") not in pids:return bad("unknown_patient",path+".patient_id")
        wr=reqs.get(r.get("staff_request_id"))
        if wr is None or wr.get("subject_kind")!="patient" or wr.get("patient_id")!=r["patient_id"]:return bad("diagnostic_staff_join",path+".staff_request_id")
    for n,o in enumerate(x["offers"]):
        path=f"inputs.offers[{n}]"; ids=o.get("patient_ids")
        if not isinstance(ids,list) or any(p not in pids for p in ids) or len(ids)!=len(set(ids)):return bad("unknown_patient",path+".patient_ids")
        a,b=o.get("start_tick"),o.get("expiry_tick")
        if not strict_int(a) or not strict_int(b) or a>=b:return bad("offer_interval",path)
        if a>h:return bad("offer_start_after_horizon",path+".start_tick")
        if "withdrawn_tick" in o and (not strict_int(o["withdrawn_tick"]) or not a<=o["withdrawn_tick"]<b):return bad("offer_withdrawal",path+".withdrawn_tick")
    seen_departures=set()
    for n,t in enumerate(x["physical_departures"]):
        if not isinstance(t,dict):return bad("row",f"inputs.physical_departures[{n}]")
        if t.get("patient_id") in seen_departures:return bad("duplicate_departure",f"inputs.physical_departures[{n}]")
        seen_departures.add(t.get("patient_id"))
        if t.get("patient_id") not in pids or t.get("space_id") not in sidset:return bad("unknown_reference",f"inputs.physical_departures[{n}]")
        if t.get("kind") not in {"discharged","physical_transfer"}:return bad("departure_kind",f"inputs.physical_departures[{n}].kind")
        if not strict_int(t.get("tick")) or t["tick"]>h:return bad("tick",f"inputs.physical_departures[{n}].tick")
    for n,r in enumerate(x["care_requests"]):
        if r.get("patient_id") not in pids or r.get("request_id") not in reqs:return bad("care_request_join",f"inputs.care_requests[{n}]")
    for n,t in enumerate(x["transfers"]):
        if t.get("patient_id") not in pids or t.get("transfer_id") not in reqs:return bad("transfer_request_join",f"inputs.transfers[{n}]")
    decisions=x.get("admission_decisions",[])
    if not isinstance(decisions,list) or len(decisions)>100000:return bad("admission_decision","inputs.admission_decisions")
    for n,a in enumerate(decisions):
        if not isinstance(a,dict):return bad("row",f"inputs.admission_decisions[{n}]")
        if a.get("status") not in {"admitted","boarded"}:return bad("admission_decision",f"inputs.admission_decisions[{n}].status")
        if a.get("patient_id") not in pids or not strict_int(a.get("tick")) or a["tick"]>h:return bad("admission_decision",f"inputs.admission_decisions[{n}]")
    arrivals={p["patient_id"]:p["arrival_tick"] for p in patients}
    for n,t in enumerate(x["physical_departures"]):
        if t["tick"]<arrivals[t["patient_id"]]:return bad("departure_before_arrival",f"inputs.physical_departures[{n}].tick")
    for n,a in enumerate(decisions):
        if a["tick"]<arrivals[a["patient_id"]]:return bad("admission_before_arrival",f"inputs.admission_decisions[{n}].tick")
    for family,key in (("diagnostic_requests","staff_request_id"),("care_requests","request_id"),("transfers","transfer_id")):
        for n,row in enumerate(x[family]):
            wr=reqs[row[key]]
            if not strict_int(row.get("ready_tick"),0,h-1) or not strict_int(row.get("work_ticks"),1):return bad("work_request_join",f"inputs.{family}[{n}]")
            if any(row.get(k)!=wr.get(k) for k in ("patient_id","ready_tick","work_ticks")) or wr["subject_kind"]!="patient":return bad("work_request_join",f"inputs.{family}[{n}]")
    return True,None,None


def _flatten(value,prefix=""):
    out={}
    if isinstance(value,dict):
        for k,v in value.items(): out.update(_flatten(v,f"{prefix}.{k}" if prefix else k))
    elif isinstance(value,list):
        for i,v in enumerate(value): out.update(_flatten(v,f"{prefix}[{i}]"))
    else: out[prefix]=value
    return out

def changed_paths(left,right,prefix=""):
    # An added/removed collection is one field mutation, not one mutation per element.
    if isinstance(left,dict) and isinstance(right,dict):
        out=set()
        for k in set(left)|set(right):
            path=f"{prefix}.{k}" if prefix else k
            if k not in left or k not in right: out.add(path)
            else: out|=changed_paths(left[k],right[k],path)
        return out
    if isinstance(left,list) and isinstance(right,list):
        if len(left)!=len(right): return {prefix}
        out=set()
        for i,(a,b) in enumerate(zip(left,right)): out|=changed_paths(a,b,f"{prefix}[{i}]")
        return out
    return set() if left==right else {prefix}

def _overlap(a,b,c,d): return max(a,c)<min(b,d)

def _eligible(person,request,tick):
    return person["assigned_zone"]==request["assigned_zone"] and set(request["required_skills"])<=set(person["skills"]) and any(a<=tick<b for a,b in person["shifts"]) and not any(a<=tick<b for a,b in person["breaks"])

def check_expected(case):
    """Check the literal timeline and conservation; never derives a dispatch sequence."""
    h=case["horizon"]; inp=case["inputs"]; e=case["expected"]; spaces={s["space_id"]:s for s in inp["spaces"]}; staff={s["staff_id"]:s for s in inp["staff"]}; reqs={r["request_id"]:r for r in inp["work_requests"]}
    # Each lifecycle is a complete partition of observation [0,H); zero-width endpoint transitions are separate.
    for sid in spaces:
        rows=sorted((r["start_tick"],r["end_tick"],r["state"]) for r in e["space_lifecycle"] if r["space_id"]==sid and r["end_tick"]>r["start_tick"])
        if not rows or rows[0][0]!=0 or rows[-1][1]!=h or any(a[1]!=b[0] for a,b in zip(rows,rows[1:])): raise AssertionError(f"lifecycle does not partition {sid} over [0,H)")
        if any(state not in {"ready","reserved","occupied","dirty_waiting","cleaning"} for _,_,state in rows): raise AssertionError("unknown lifecycle state")
    for r in e["space_lifecycle"]:
        if r["space_id"] not in spaces or not (0<=r["start_tick"]<=r["end_tick"]<=h): raise AssertionError("invalid lifecycle row")
    occ=e["patient_space_intervals"]
    arrivals={p["patient_id"]:p["arrival_tick"] for p in inp["patients"]}
    for r in occ:
        if r["patient_id"] not in arrivals or r["start_tick"]<arrivals[r["patient_id"]]:raise AssertionError("occupancy before arrival")
        if r["patient_id"] not in {p["patient_id"] for p in inp["patients"]} or r["space_id"] not in spaces or not 0<=r["start_tick"]<r["end_tick"]<=h: raise AssertionError("invalid patient-space interval")
        if r["release_reason"] not in {"physical_departure","physical_transfer","censored_at_horizon"}: raise AssertionError("unknown occupancy end")
        rows=[q for q in e["space_lifecycle"] if q["space_id"]==r["space_id"] and q["state"]=="occupied" and q["start_tick"]<=r["start_tick"] and r["end_tick"]<=q["end_tick"]]
        if not rows: raise AssertionError("occupancy interval lacks a matching occupied lifecycle row")
    for patient in {p["patient_id"] for p in inp["patients"]}:
        rows=[o for o in occ if o["patient_id"]==patient]
        if any(_overlap(a["start_tick"],a["end_tick"],b["start_tick"],b["end_tick"]) for i,a in enumerate(rows) for b in rows[i+1:]):raise AssertionError("patient occupies two spaces")
    for row in e["space_lifecycle"]:
        if row["state"]=="occupied" and row["end_tick"]>row["start_tick"]:
            matched=sorted((o["start_tick"],o["end_tick"]) for o in occ if o["space_id"]==row["space_id"] and row["start_tick"]<=o["start_tick"]<o["end_tick"]<=row["end_tick"])
            if not matched or matched[0][0]!=row["start_tick"] or matched[-1][1]!=row["end_tick"] or any(a[1]!=b[0] for a,b in zip(matched,matched[1:])):raise AssertionError("occupied lifecycle lacks occupants")
    # Per-space and aggregate capacity sweeps use only literal intervals.
    for sid in spaces:
        rows=[(r["start_tick"],r["end_tick"],r["patient_id"]) for r in occ if r["space_id"]==sid]
        if any(_overlap(a,b,c,d) for i,(a,b,_) in enumerate(rows) for c,d,_ in rows[i+1:]): raise AssertionError(f"double occupancy of {sid}")
    points=sorted({q for r in occ for q in (r["start_tick"],r["end_tick"])})
    maximum=0
    for tick in points:
        maximum=max(maximum,sum(1 for r in occ if r["start_tick"]<=tick<r["end_tick"]))
    for r in e.get("patient_space_grants",[]):
        if r["patient_id"] not in arrivals or r["tick"]<arrivals[r["patient_id"]]:raise AssertionError("horizon grant before arrival")
        if r["tick"]!=h or r["observed_occupancy_ticks"]!=0 or not r.get("active_at_horizon"): raise AssertionError("horizon grant must be zero-observed and active")
    maximum=max(maximum,len(e.get("patient_space_grants",[])))
    if maximum>inp["space_capacity_profile"]["staffed"]: raise AssertionError("aggregate staffed-space capacity exceeded")
    # Claims exactly join configured work, named staff, subject, skill/zone eligibility and duration.
    claims=e["work_claims"]
    if len({c["request_id"] for c in claims})!=len(claims):raise AssertionError("duplicate claim for nonpreemptive space work")
    for c in claims:
        if not strict_int(c.get("start_tick"),0,h) or not strict_int(c.get("configured_end_tick"),1,(1<<128)-1) or not strict_int(c.get("observed_work_ticks")):raise AssertionError("invalid claim time")
        r=reqs.get(c["request_id"]); person=staff.get(c["resource_id"])
        if r is None or person is None: raise AssertionError("work claim has unknown request/staff")
        if c["start_tick"]<r["ready_tick"] or c["configured_end_tick"]-c["start_tick"]!=r["work_ticks"]: raise AssertionError("claim does not match request readiness/duration")
        if c["configured_end_tick"]>(1<<128)-1 or not _eligible(person,r,c["start_tick"]): raise AssertionError("claim staff was not eligible at actual grant")
        if c["subject_kind"]!=r["subject_kind"]: raise AssertionError("claim subject kind mismatch")
        for key in ("patient_id","space_id"):
            if c.get(key)!=r.get(key): raise AssertionError(f"claim {key} mismatch")
        if c["observed_work_ticks"]!=max(0,min(h,c["configured_end_tick"])-c["start_tick"]): raise AssertionError("configured vs observed work mismatch")
    for sid in staff:
        rows=sorted((c["start_tick"],c["configured_end_tick"]) for c in claims if c["resource_id"]==sid)
        if any(a[1]>b[0] for a,b in zip(rows,rows[1:])): raise AssertionError(f"staff claim overlap {sid}")
    # Waiting rows hold no resource; successful requests begin exactly when their wait ends.
    for w in e.get("space_waits",[]):
        if w.get("holds_space_claim") is not False or not 0<=w["start_tick"]<w["end_tick"]<=h: raise AssertionError("invalid space wait")
        if any(r["patient_id"]==w["patient_id"] and _overlap(w["start_tick"],w["end_tick"],r["start_tick"],r["end_tick"]) for r in occ): raise AssertionError("space wait overlaps ED occupancy")
    for kind,key,claim_field,flag in (("staff_waits","request_id","request_id","holds_staff_claim"),("cleaner_waits","space_id","space_id","holds_cleaner_claim")):
        for w in e.get(kind,[]):
            if w.get(flag) is not False or not 0<=w["start_tick"]<w["end_tick"]<=h: raise AssertionError(f"invalid {kind} row")
            matching=[c for c in claims if c.get(claim_field)==w[key]]
            if matching and min(c["start_tick"] for c in matching)<w["end_tick"]: raise AssertionError(f"claim held while waiting: {kind}")
    for w in e.get("room_waits",[]):
        if w.get("holds_room_claim") is not False: raise AssertionError("room wait holds a room")
        if any(r["request_id"]==w["request_id"] and r["start_tick"]<w["end_tick"] for r in e["diagnostic_room_intervals"]): raise AssertionError("room claimed before wait ended")
    # Diagnostic rooms have independent capacity and references.
    capacities={r["room_id"]:r["capacity"] for r in inp["diagnostic_rooms"]}
    diagnostic_requests={r["request_id"]:r for r in inp["diagnostic_requests"]}
    for interval in e["diagnostic_room_intervals"]:
        request=diagnostic_requests.get(interval["request_id"])
        if request is None or request["room_id"]!=interval["room_id"] or interval["room_id"] not in capacities: raise AssertionError("diagnostic room interval lacks request/room join")
        if not strict_int(interval.get("start_tick"),request["ready_tick"],h) or not strict_int(interval.get("end_tick"),interval["start_tick"],h):raise AssertionError("invalid diagnostic room interval")
        claim=next((c for c in claims if c["request_id"]==request["staff_request_id"]),None)
        if claim is None:
            if not interval.get("held_through_horizon") or interval["end_tick"]!=h:raise AssertionError("unserved diagnostic room must be held at horizon")
            if not any(w["request_id"]==request["staff_request_id"] and w["start_tick"]==interval["start_tick"] and w["end_tick"]==h for w in e.get("staff_waits",[])):raise AssertionError("unserved diagnostic lacks explicit staff wait")
        elif not (interval["start_tick"]<=claim["start_tick"] and interval["end_tick"]==min(h,claim["configured_end_tick"])):raise AssertionError("diagnostic room must span staff wait and work")
        if not any(o["patient_id"]==request["patient_id"] and o["start_tick"]<=interval["start_tick"] and o["end_tick"]>=interval["end_tick"] for o in occ):raise AssertionError("diagnostic patient lost ED space")
    for rid,cap in capacities.items():
        rows=[r for r in e["diagnostic_room_intervals"] if r["room_id"]==rid]
        for t in sorted({q for r in rows for q in (r["start_tick"],r["end_tick"])}):
            if sum(r["start_tick"]<=t<r["end_tick"] for r in rows)>cap: raise AssertionError("diagnostic room capacity exceeded")
    # Horizon states and patient conservation include exact-H transitions/grants.
    endpoint={r["space_id"]:r["state"] for r in e.get("space_state_at_horizon",[])}
    endpoint.update({r["space_id"]:"occupied" for r in e.get("patient_space_grants",[])})
    state_h={}
    for sid in spaces:
        if sid in endpoint: state_h[sid]=endpoint[sid]
        else:
            last=next((r for r in reversed(e["space_lifecycle"]) if r["space_id"]==sid and r["end_tick"]==h and r["end_tick"]>r["start_tick"]),None)
            if last is None: raise AssertionError("missing horizon lifecycle state")
            state_h[sid]=last["state"]
    ready=sum(v=="ready" for v in state_h.values())
    a=e["accounting"]
    departures=inp["physical_departures"]
    if e.get("physical_departures",[])!=departures: raise AssertionError("expected departures differ from input")
    for departure in departures:
        if not any(o["patient_id"]==departure["patient_id"] and o["space_id"]==departure["space_id"] and o["end_tick"]==departure["tick"] and o["release_reason"]!="censored_at_horizon" for o in occ):raise AssertionError("departure lacks actual occupancy release")
    terminal={r["patient_id"] for r in departures}
    arrived={r["patient_id"] for r in inp["patients"]}
    unfinished=arrived-terminal
    occupying={r["patient_id"] for r in occ if r["release_reason"]=="censored_at_horizon" and r["end_tick"]==h}
    occupying|={r["patient_id"] for r in e.get("patient_space_grants",[])}
    held=[(r["patient_id"],r["space_id"]) for r in occ if r["release_reason"]=="censored_at_horizon" and r["end_tick"]==h]+[(r["patient_id"],r["space_id"]) for r in e.get("patient_space_grants",[])]
    if len({p for p,_ in held})!=len(held) or len({s for _,s in held})!=len(held):raise AssertionError("duplicate horizon occupancy")
    for patient,sid in held:
        if sid not in spaces or patient not in spaces[sid]["compatible_patient_ids"]:raise AssertionError("invalid horizon space grant")
    maximum=max(maximum,len(held))
    if len(arrived)!=len(departures)+len(unfinished):raise AssertionError("patient conservation failed")
    counts={"arrived_patients":len(arrived),"discharged_patients":sum(r["kind"]=="discharged" for r in departures),"transferred_patients":sum(r["kind"]=="physical_transfer" for r in departures),"unfinished_patients":len(unfinished),"unfinished_patient_ids":sorted(unfinished,key=iid),"patients_occupying_at_horizon":sorted(occupying,key=iid),"ready_spaces_at_horizon":ready,"max_concurrent_occupants":maximum}
    for k,v in counts.items():
        actual=a.get(k,0 if isinstance(v,int) else [])
        if (set(actual) if isinstance(v,list) else actual)!=(set(v) if isinstance(v,list) else v): raise AssertionError(f"accounting {k}: {actual!r} != {v!r}")
    # Declarative joins between supplied outcomes and configured attempts/offers.
    attempts={r["reservation_id"]:r for r in inp["reservations"]}
    for row in e["reservation_outcomes"]:
        attempt=attempts.get(row["reservation_id"])
        if attempt is None:raise AssertionError("reservation outcome has no attempt")
        kind=row["outcome"];tick=row.get("tick",row.get("interval",[None])[0])
        if not strict_int(tick,0,h):raise AssertionError("invalid reservation outcome tick")
        if kind=="accepted" and row["interval"]!=[attempt["start_tick"],attempt["expiry_tick"]]:raise AssertionError("reservation acceptance changed configured interval")
        if kind in {"expired","stale_expiry_noop"} and tick!=attempt["expiry_tick"]:raise AssertionError("reservation expiry changed")
        if kind=="consumed":
            if not attempt["start_tick"]<=tick<attempt["expiry_tick"]:raise AssertionError("consumption outside reservation")
            if not any(o["patient_id"]==attempt["patient_id"] and o["space_id"]==attempt["space_id"] and o["start_tick"]==tick for o in occ):raise AssertionError("consumption lacks owner occupancy")
        if kind=="wrong_claimant_rejected" and row.get("patient_id")==attempt["patient_id"]:raise AssertionError("owner is not wrong claimant")
    for rid in attempts:
        if sum(r["reservation_id"]==rid and r["outcome"]=="consumed" for r in e["reservation_outcomes"])>1:raise AssertionError("reservation consumed twice")
    offers={o["offer_id"]:o for o in inp["offers"]};consumed=set()
    decisions={r["patient_id"]:r["tick"] for r in inp.get("admission_decisions",[])}
    for row in e["boarding_events"]:
        offer=offers.get(row.get("offer_id"))
        if offer is None:raise AssertionError("boarding event lacks offer")
        patient=row.get("patient_id")
        if patient is not None and patient not in offer["patient_ids"]:raise AssertionError("incompatible offer")
        allocation=row.get("offer_consumed_tick",row.get("offer_allocation_tick"))
        if allocation is not None:
            if offer["offer_id"] in consumed:raise AssertionError("offer consumed twice")
            consumed.add(offer["offer_id"])
            if not offer["start_tick"]<=allocation<offer["expiry_tick"] or ("withdrawn_tick" in offer and allocation>=offer["withdrawn_tick"]):raise AssertionError("offer allocated outside availability")
            if patient not in decisions or decisions[patient]>allocation:raise AssertionError("offer allocated before admission")
            care_ids={r["request_id"] for r in inp["care_requests"] if r["patient_id"]==patient}
            if any(c["request_id"] in care_ids and c["start_tick"]<=allocation<c["configured_end_tick"] for c in claims):raise AssertionError("offer allocated during care")
            start,end=row["transfer_interval"]
            if start<allocation:raise AssertionError("transfer before allocation")
            transfer_ids={t["transfer_id"] for t in inp["transfers"] if t["patient_id"]==patient}
            if not any(c["request_id"] in transfer_ids and (c["start_tick"],c["configured_end_tick"])==(start,end) for c in claims):raise AssertionError("transfer lacks work claim")
            if row["physical_departure_tick"]!=(end if end<=h else None):raise AssertionError("transfer departure disagrees with completion")
        if row.get("state")=="unused_expired" and (row["expiry_tick"]!=offer["expiry_tick"] or row["transfer_started"]):raise AssertionError("invalid unused offer outcome")
        if row.get("state")=="withdrawn" and (row["withdrawn_tick"]!=offer.get("withdrawn_tick") or row["allocated"]):raise AssertionError("invalid withdrawn offer outcome")
    for rid in reqs:
        if not any(c["request_id"]==rid for c in claims) and not any(w["request_id"]==rid and w["end_tick"]==h for w in e.get("staff_waits",[])):raise AssertionError("configured work silently omitted")


def _expected_digest(expected):
    return hashlib.sha256(json.dumps(expected,sort_keys=True,separators=(",",":"),ensure_ascii=True).encode()).hexdigest()

class SpaceFixtureSpecTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.data=json.loads(FIXTURE.read_text()); cls.cases={c["case_id"]:c for c in cls.data["cases"]}

    def test_exact_inventory_and_unbound_bindings(self):
        self.assertEqual(set(self.cases),POSITIVE|NEGATIVE); self.assertEqual(len(self.data["case_ids"]),32)
        self.assertEqual(self.data["case_ids"],[c["case_id"] for c in self.data["cases"]]); self.assertEqual(self.data["native_binding"],NATIVE)
        for c in self.data["cases"]: self.assertEqual(c["native_binding"],NATIVE)

    def test_all_positive_literal_histories_and_invariants(self):
        for cid in sorted(POSITIVE):
            with self.subTest(case=cid):
                c=self.cases[cid]
                self.assertEqual(_expected_digest(c["expected"]),EXPECTED_SHA256[cid],HAND_WITNESS[cid])
                self.assertEqual(valid_input({"horizon":c["horizon"],"in_flight_policy":c["in_flight_policy"],"inputs":c["inputs"]}),(True,None,None))
                assert_literal_timeline(self,c)
                check_expected(c)
                self.assertTrue(c["notes"], HAND_WITNESS[cid])
                if "permuted_input" in c:
                    self.assertNotEqual(c["inputs"],c["permuted_input"])
                    self.assertEqual(set(c["inputs"]),set(c["permuted_input"]))
                    for key,value in c["inputs"].items():
                        if isinstance(value,list): self.assertEqual(c["permuted_input"][key],list(reversed(value)),key)
                    self.assertEqual(valid_input({"horizon":c["horizon"],"in_flight_policy":c["in_flight_policy"],"inputs":c["permuted_input"]}),(True,None,None))
                    # Canonical comparison covers dictionary/scalar fields as well as array order.
                    canonical=lambda value: json.dumps(value,sort_keys=True,separators=(",",":"),ensure_ascii=True)
                    for key,value in c["inputs"].items():
                        if isinstance(value,list): self.assertEqual(sorted(value,key=canonical),sorted(c["permuted_input"][key],key=canonical),key)
                        else: self.assertEqual(value,c["permuted_input"][key],key)

    def test_negative_baseline_invalid_exact_mutation_and_repair(self):
        for cid in sorted(NEGATIVE):
            with self.subTest(case=cid):
                c=self.cases[cid]; b=c["valid_baseline"]; inv=c["invalid_input"]; repair=c["repair_control"]
                self.assertEqual(valid_input(b),(True,None,None)); self.assertEqual(repair,b)
                ok,cat,path=valid_input(inv); self.assertFalse(ok); self.assertEqual((cat,path),(c["diagnostic_category"],c["offending_path"]))
                self.assertEqual(changed_paths(b,inv),{c["mutation_path"]})
                before=path_value(b,c["mutation_path"]); after=path_value(inv,c["mutation_path"])
                if c["expected_before"] is None: self.assertIs(before,MISSING)
                else: self.assertEqual(before,c["expected_before"])
                self.assertEqual(after,c["expected_after"])
                self.assertIsInstance(c["mutation_group"],str); self.assertTrue(c["mutation_group"])
                self.assertEqual((c["horizon"],c["in_flight_policy"]),(b["horizon"],b["in_flight_policy"]))

    def test_semantic_mutations_fail_without_hash_pins(self):
        mutations=[
          ("space.cleaning_blocks_reuse",lambda c:c["expected"]["space_lifecycle"][1].update(end_tick=4)),
          ("space.cleaner_wait_dirty",lambda c:c["expected"]["work_claims"][1].update(start_tick=3)),
          ("space.cleaning_horizon",lambda c:c["expected"]["patient_space_grants"][0].update(patient_id="unknown")),
          ("space.reservation_blocks",lambda c:c["expected"]["space_waits"][0].update(end_tick=7)),
          ("space.reservation_consumption",lambda c:c["expected"]["reservation_outcomes"][1].update(tick=6)),
          ("space.diagnostic_staff_wait",lambda c:c["expected"]["diagnostic_room_intervals"][0].update(end_tick=6)),
          ("space.boarding_retention",lambda c:c["expected"]["boarding_events"][1].update(offer_id="oA")),
          ("space.boarding_care",lambda c:c["expected"]["boarding_events"][0].update(offer_allocation_tick=7)),
          ("space.transfer_horizon",lambda c:c["expected"]["boarding_events"][0].update(physical_departure_tick=6)),
          ("space.permutation",lambda c:c["expected"]["accounting"].update(ready_spaces_at_horizon=1)),
        ]
        for cid,mutate in mutations:
            with self.subTest(case=cid):
                c=copy.deepcopy(self.cases[cid]);mutate(c)
                with self.assertRaises(AssertionError):check_expected(c)
                with self.assertRaises(AssertionError):assert_literal_timeline(self,c)

    def test_input_arrival_and_repeated_scalar_mutations(self):
        c=copy.deepcopy(self.cases["space.cleaning_blocks_reuse"])
        c["inputs"]["patients"][0]["arrival_tick"]=2
        with self.assertRaisesRegex(AssertionError,"before arrival"):check_expected(c)
        for cid,family,key in (("space.diagnostic_serial","diagnostic_requests","staff_request_id"),("space.boarding_care","care_requests","request_id"),("space.boarding_retention","transfers","transfer_id")):
            for field in ("ready_tick","work_ticks"):
                c=copy.deepcopy(self.cases[cid]);row=c["inputs"][family][0]
                row[field]=float(row[field])
                self.assertFalse(valid_input({"horizon":c["horizon"],"in_flight_policy":c["in_flight_policy"],"inputs":c["inputs"]})[0])
            c=copy.deepcopy(self.cases[cid]);row=c["inputs"][family][0]
            wr=next(r for r in c["inputs"]["work_requests"] if r["request_id"]==row[key])
            wr["ready_tick"]=1;row["ready_tick"]=True
            self.assertFalse(valid_input({"horizon":c["horizon"],"in_flight_policy":c["in_flight_policy"],"inputs":c["inputs"]})[0])

    def test_strict_integer_skill_and_calendar_rules(self):
        sample=copy.deepcopy(self.cases["space.cleaning_blocks_reuse"])
        env={"horizon":sample["horizon"],"in_flight_policy":sample["in_flight_policy"],"inputs":sample["inputs"]}
        for bad in (True,1.0):
            q=copy.deepcopy(env);q["horizon"]=bad;self.assertFalse(valid_input(q)[0])
        q=copy.deepcopy(env);q["inputs"]["work_requests"][0]["required_staff_count"]=True;self.assertFalse(valid_input(q)[0])
        q=copy.deepcopy(env);q["inputs"]["staff"][0]["skills"]=["z","a"];self.assertFalse(valid_input(q)[0])
        # Calendar source order is normalized before overlap checks.
        q=copy.deepcopy(env);q["inputs"]["staff"][0]["shifts"]=[[5,10],[0,4]];self.assertTrue(valid_input(q)[0])
        q=copy.deepcopy(env);q["inputs"]["staff"][0]["breaks"]=[[9,11]];self.assertFalse(valid_input(q)[0])
        q=copy.deepcopy(env);q["inputs"]["work_requests"][0]["urgency"]=1<<31;self.assertFalse(valid_input(q)[0])

if __name__=="__main__": unittest.main()
