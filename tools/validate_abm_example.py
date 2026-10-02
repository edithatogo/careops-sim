#!/usr/bin/env python3
"""Check the P2.3 synthetic graph and Macro/Micro accounting fixture only."""

from __future__ import annotations

import argparse
import json
import math
from pathlib import Path
from typing import Any


def _reachable(graph: dict[str, Any], start: str, goal: str) -> bool:
    adjacency: dict[str, list[str]] = {node: [] for node in graph["nodes"]}
    for edge in graph["edges"]:
        if edge["accessible"]:
            adjacency[edge["from"]].append(edge["to"])
    pending = [start]
    visited: set[str] = set()
    while pending:
        node = pending.pop()
        if node == goal:
            return True
        if node in visited:
            continue
        visited.add(node)
        pending.extend(adjacency.get(node, []))
    return False


def validate(example: dict[str, Any]) -> None:
    if example.get("status") != "synthetic_test_only":
        raise ValueError("fixture must be explicitly synthetic_test_only")
    graph = example.get("graph")
    if not isinstance(graph, dict) or graph.get("directed") is not True or graph.get("distance_unit") != "m":
        raise ValueError("Micro graph must be directed and declare metres")
    edge_by_id: dict[str, dict[str, Any]] = {}
    for edge in graph.get("edges", []):
        if edge.get("edge_id") in edge_by_id:
            raise ValueError("duplicate graph edge ID")
        edge_by_id[edge["edge_id"]] = edge
        distance, speed = edge.get("distance_m"), edge.get("speed_m_per_s")
        if not isinstance(distance, (int, float)) or not math.isfinite(distance) or distance <= 0:
            raise ValueError("edge distance must be finite positive metres")
        if not isinstance(speed, (int, float)) or not math.isfinite(speed) or speed <= 0:
            raise ValueError("edge speed must be finite positive metres per second")
        if edge.get("from") not in graph["nodes"] or edge.get("to") not in graph["nodes"]:
            raise ValueError("edge endpoint is not a declared node")

    micro = example.get("micro_case", {})
    route = micro.get("route", {})
    if route.get("from") not in graph["nodes"] or route.get("to") not in graph["nodes"]:
        raise ValueError("route endpoints must be declared graph nodes")
    current = route.get("from")
    travel_seconds = 0.0
    for edge_id in route.get("edge_ids", []):
        edge = edge_by_id.get(edge_id)
        if edge is None or not edge.get("accessible"):
            raise ValueError("route uses missing or inaccessible edge")
        if edge["from"] != current:
            raise ValueError("route edges are not contiguous in declared direction")
        current = edge["to"]
        travel_seconds += edge["distance_m"] / edge["speed_m_per_s"]
    if current != route.get("to") or not _reachable(graph, route["from"], route["to"]):
        raise ValueError("route endpoints are not connected by accessible directed edges")
    if not math.isclose(travel_seconds, micro.get("expected_travel_seconds"), rel_tol=0, abs_tol=1e-9):
        raise ValueError("distance/speed calculation differs from hand-worked seconds")

    for pair in micro.get("unreachable_checks", []):
        if _reachable(graph, pair["from"], pair["to"]):
            raise ValueError("a declared unreachable endpoint pair is reachable")
    intervals = micro.get("intervals_seconds", {})
    required_intervals = {"queue_wait", "active_work", "travel", "interruption_pause"}
    if set(intervals) != required_intervals:
        raise ValueError("queue wait, active work, travel and interruption pause must be separate")
    if any(not isinstance(value, (int, float)) or not math.isfinite(value) or value < 0 for value in intervals.values()):
        raise ValueError("interval durations must be finite nonnegative seconds")
    if not math.isclose(intervals["travel"], travel_seconds, rel_tol=0, abs_tol=1e-9):
        raise ValueError("travel interval must equal the sum of edge distance/speed durations")
    micro_elapsed = sum(intervals.values())
    expected = micro.get("expected_elapsed_seconds")
    if not math.isclose(micro_elapsed, expected, rel_tol=0, abs_tol=1e-9):
        raise ValueError("Micro intervals do not sum to the hand-worked elapsed duration")

    macro = example.get("macro_case", {})
    if macro.get("graph", "missing") is not None or macro.get("transit_policy") != "not_modeled":
        raise ValueError("Macro case must run without a spatial graph")
    if not math.isclose(macro.get("elapsed_seconds", math.nan), micro_elapsed, rel_tol=0, abs_tol=1e-9):
        raise ValueError("Macro and Micro examples must describe the same elapsed window")
    if example.get("cross_mode_rule") != "compare_same_elapsed_window; never_add_macro_elapsed_to_its_micro_decomposition":
        raise ValueError("Macro elapsed time must be compared with, not added to, its Micro decomposition")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("path", type=Path, nargs="?", default=Path("model-inputs/ed/abm/p23-synthetic-accounting.json"))
    args = parser.parse_args()
    try:
        validate(json.loads(args.path.read_text(encoding="utf-8")))
    except (OSError, UnicodeError, json.JSONDecodeError, KeyError, TypeError, ValueError) as exc:
        parser.error(str(exc))
    print("synthetic graph and Macro/Micro accounting checks passed; no ED empirical value established")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
