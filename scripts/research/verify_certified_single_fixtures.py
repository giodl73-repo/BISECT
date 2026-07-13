#!/usr/bin/env python3
"""Independent verifier for committed single-district certificate fixtures."""

from __future__ import annotations

import hashlib
import json
from collections import deque
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
PACKAGE = ROOT / "docs/examples/certified-single-district"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def value_hash(value: object) -> str:
    encoded = json.dumps(value, separators=(",", ":"), ensure_ascii=False).encode()
    return f"sha256:{hashlib.sha256(encoded).hexdigest()}"


def verify(instance: dict, certificate: dict) -> None:
    unit_ids = instance["unit_ids"]
    if not unit_ids or unit_ids != sorted(set(unit_ids)):
        raise ValueError("noncanonical unit ids")
    if len(unit_ids) != len(instance["populations"]):
        raise ValueError("population length")
    if any(population < 0 for population in instance["populations"]):
        raise ValueError("negative population")
    adjacency = [[] for _ in unit_ids]
    seen_edges = set()
    for edge in instance["edges"]:
        pair = (edge["left"], edge["right"])
        if (
            pair in seen_edges
            or pair[0] >= pair[1]
            or pair[1] >= len(unit_ids)
            or edge["weight"] <= 0
        ):
            raise ValueError("invalid edge")
        seen_edges.add(pair)
        adjacency[pair[0]].append(pair[1])
        adjacency[pair[1]].append(pair[0])
    reached = {0}
    queue = deque([0])
    while queue:
        unit = queue.popleft()
        for neighbor in adjacency[unit]:
            if neighbor not in reached:
                reached.add(neighbor)
                queue.append(neighbor)
    if len(reached) != len(unit_ids):
        raise ValueError("disconnected graph")

    instance_hash = value_hash(instance)
    if certificate["instance_hash"] != instance_hash:
        raise ValueError("instance hash")
    assignment = certificate["assignment"]
    if len(assignment) != len(unit_ids) or any(label != 0 for label in assignment):
        raise ValueError("assignment")
    projection = {
        "schema_version": certificate["schema_version"],
        "instance_hash": certificate["instance_hash"],
        "unit_universe_hash": certificate["unit_universe_hash"],
        "assignment": assignment,
        "unit_count": certificate["unit_count"],
        "population_total": certificate["population_total"],
        "weighted_boundary_cut": certificate["weighted_boundary_cut"],
        "connected": certificate["connected"],
        "proof_kind": certificate["proof_kind"],
    }
    if certificate["certificate_id"] != value_hash(projection):
        raise ValueError("certificate id")
    if (
        certificate["unit_count"] != len(unit_ids)
        or certificate["population_total"] != sum(instance["populations"])
        or certificate["weighted_boundary_cut"] != 0
        or certificate["connected"] is not True
    ):
        raise ValueError("summary")


def main() -> None:
    manifest = json.loads((PACKAGE / "manifest.json").read_text(encoding="utf-8"))
    for relative, expected in manifest["files"].items():
        if sha256(PACKAGE / relative) != expected:
            raise SystemExit(f"single-district fixture hash mismatch: {relative}")
    positive = PACKAGE / "grid3x3"
    instance = json.loads(
        (positive / "single-district-instance.json").read_text(encoding="utf-8")
    )
    certificate = json.loads(
        (positive / "single-district-certificate.json").read_text(encoding="utf-8")
    )
    verify(instance, certificate)
    negative_cases = [
        (
            instance,
            json.loads(
                (PACKAGE / "negative/omitted-assignment.json").read_text(
                    encoding="utf-8"
                )
            ),
        ),
        (
            instance,
            json.loads(
                (PACKAGE / "negative/nonzero-assignment.json").read_text(
                    encoding="utf-8"
                )
            ),
        ),
        (
            json.loads(
                (PACKAGE / "negative/disconnected-instance.json").read_text(
                    encoding="utf-8"
                )
            ),
            certificate,
        ),
    ]
    for negative_instance, negative_certificate in negative_cases:
        try:
            verify(negative_instance, negative_certificate)
        except ValueError:
            continue
        raise SystemExit("single-district hostile fixture was accepted")
    print("Certified single-district independent verification: PASS")


if __name__ == "__main__":
    main()
