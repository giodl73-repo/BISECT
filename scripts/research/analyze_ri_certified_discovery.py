#!/usr/bin/env python3
"""Analyze and verify the local Rhode Island certified-discovery package."""

from __future__ import annotations

import argparse
import hashlib
import json
import tempfile
from collections import deque
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = Path("scripts/research/analyze_ri_certified_discovery.py")
LOCAL = Path("data/2020/certified/ri-root-discovery")
SCHEMA = "ri-certified-discovery-frontier-v1"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def text_sha256(path: Path) -> str:
    """Hash UTF-8 text canonically across LF and CRLF checkouts."""
    normalized = (
        path.read_text(encoding="utf-8")
        .replace("\r\n", "\n")
        .replace("\r", "\n")
    )
    return hashlib.sha256(normalized.encode("utf-8")).hexdigest()


def canonical_hash(value: object) -> str:
    encoded = json.dumps(
        value, ensure_ascii=False, separators=(",", ":"), sort_keys=True
    ).encode("utf-8")
    return f"sha256:{hashlib.sha256(encoded).hexdigest()}"


def connected(adjacency: list[list[int]], assignment: list[int], label: int) -> bool:
    units = [index for index, value in enumerate(assignment) if value == label]
    allowed = set(units)
    seen = {units[0]}
    queue = deque([units[0]])
    while queue:
        unit = queue.popleft()
        for neighbor in adjacency[unit]:
            if neighbor in allowed and neighbor not in seen:
                seen.add(neighbor)
                queue.append(neighbor)
    return seen == allowed


def analyze(report_path: Path, manifest_path: Path) -> None:
    package_manifest = json.loads(
        (ROOT / LOCAL / "certified-discovery-manifest.json").read_text(encoding="utf-8")
    )
    instance_path = ROOT / LOCAL / "certified-split-instance.json"
    discovery_path = ROOT / LOCAL / "certified-discovery.json"
    for name, expected in package_manifest["files"].items():
        if sha256(ROOT / LOCAL / name) != expected:
            raise SystemExit(f"local RI discovery hash mismatch: {name}")
    instance = json.loads(instance_path.read_text(encoding="utf-8"))
    discovery = json.loads(discovery_path.read_text(encoding="utf-8"))
    if canonical_hash(instance) != discovery["instance_hash"]:
        raise SystemExit("RI discovery instance identity mismatch")
    discovery_projection = {
        key: discovery[key]
        for key in (
            "schema_version",
            "instance_hash",
            "solver_name",
            "solver_version",
            "method",
            "objective",
        )
    }
    if canonical_hash(discovery_projection) != discovery["discovery_id"]:
        raise SystemExit("RI discovery ID mismatch")

    assignment = discovery["objective"]["canonical_assignment"]
    adjacency = [[] for _ in assignment]
    cut = 0
    for edge in instance["edges"]:
        left, right = edge["left"], edge["right"]
        adjacency[left].append(right)
        adjacency[right].append(left)
        if assignment[left] != assignment[right]:
            cut += edge["weight"]
    populations = [0, 0]
    unit_counts = [0, 0]
    for population, label in zip(instance["populations"], assignment, strict=True):
        populations[label] += population
        unit_counts[label] += 1
    total = sum(populations)
    deviation = abs(2 * populations[0] - total)
    if discovery["objective"]["primary"] != {
        "max_population_deviation_scaled": deviation,
        "total_population_deviation_scaled": 2 * deviation,
        "weighted_boundary_cut": cut,
    }:
        raise SystemExit("RI discovery objective mismatch")
    connected_children = [connected(adjacency, assignment, label) for label in (0, 1)]
    if connected_children != [True, True]:
        raise SystemExit("RI discovery is not connected")

    report = {
        "schema_version": SCHEMA,
        "status": "unproved-incumbent",
        "state": "rhode_island",
        "year": 2020,
        "districts": 2,
        "local_package": {
            "path": LOCAL.as_posix(),
            "committed": False,
            "instance_bytes": instance_path.stat().st_size,
            "instance_sha256": sha256(instance_path),
            "discovery_bytes": discovery_path.stat().st_size,
            "discovery_sha256": sha256(discovery_path),
            "instance_hash": discovery["instance_hash"],
            "discovery_id": discovery["discovery_id"],
        },
        "solver": {
            "name": discovery["solver_name"],
            "version": discovery["solver_version"],
            "method": discovery["method"],
            "seed": package_manifest["seed"],
            "deterministic_replay": True,
        },
        "candidate": {
            "child_populations": populations,
            "child_unit_counts": unit_counts,
            "connected": connected_children,
            "max_population_deviation_scaled": deviation,
            "total_population_deviation_scaled": 2 * deviation,
            "weighted_boundary_cut": cut,
            "theoretical_population_floor": total % 2,
            "population_gap_above_floor": deviation - (total % 2),
        },
        "certification_status": {
            "proof": "not-generated",
            "claim": "deterministic connected incumbent only",
            "next_step": "compile compact population decision and seek a SAT counterexample or UNSAT proof",
        },
        "claim_boundary": (
            "Hash-bound deterministic METIS discovery with exact objective and connectivity "
            "validation; not an optimality proof or certified Rhode Island split."
        ),
    }
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    try:
        manifest_command_path = manifest_path.relative_to(ROOT).as_posix()
    except ValueError:
        manifest_command_path = manifest_path.as_posix()
    manifest = {
        "schema_version": "ri-certified-discovery-frontier-package-v1",
        "package_id": "ri-2020-root-discovery-frontier",
        "status": "unproved-incumbent",
        "files": [{"path": report_path.name, "sha256": sha256(report_path)}],
        "analyzer_path": SCRIPT.as_posix(),
        "analyzer_sha256": text_sha256(ROOT / SCRIPT),
        "verification_commands": [
            (
                "python scripts/research/analyze_ri_certified_discovery.py verify "
                f"{manifest_command_path}"
            ),
            (
                "python scripts/research/analyze_ri_certified_discovery.py verify "
                f"{manifest_command_path} --check-local"
            ),
        ],
        "claim_boundary": report["claim_boundary"],
    }
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print("RI certified discovery analysis: UNPROVED INCUMBENT")


def verify(manifest_path: Path, check_local: bool) -> None:
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if manifest["analyzer_sha256"] != text_sha256(ROOT / manifest["analyzer_path"]):
        raise SystemExit("RI discovery analyzer hash mismatch")
    report_path = manifest_path.parent / manifest["files"][0]["path"]
    if manifest["files"][0]["sha256"] != sha256(report_path):
        raise SystemExit("RI discovery report hash mismatch")
    report = json.loads(report_path.read_text(encoding="utf-8"))
    if report["schema_version"] != SCHEMA or report["status"] != "unproved-incumbent":
        raise SystemExit("RI discovery report posture drift")
    candidate = report["candidate"]
    if sum(candidate["child_populations"]) != 1_097_379:
        raise SystemExit("RI discovery population drift")
    if sum(candidate["child_unit_counts"]) != 25_649:
        raise SystemExit("RI discovery unit-count drift")
    if candidate["connected"] != [True, True]:
        raise SystemExit("RI discovery connectivity drift")
    if check_local:
        with tempfile.TemporaryDirectory() as temporary:
            temporary_root = Path(temporary)
            generated_report = temporary_root / "report.json"
            generated_manifest = temporary_root / "manifest.json"
            analyze(generated_report, generated_manifest)
            rebuilt = json.loads(generated_report.read_text(encoding="utf-8"))
            if rebuilt != report:
                raise SystemExit("RI discovery local replay differs from committed report")
    print("RI certified discovery verification: PASS")


def main() -> None:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)
    analyze_parser = subparsers.add_parser("analyze")
    analyze_parser.add_argument("--report", type=Path, required=True)
    analyze_parser.add_argument("--manifest", type=Path, required=True)
    verify_parser = subparsers.add_parser("verify")
    verify_parser.add_argument("manifest", type=Path)
    verify_parser.add_argument("--check-local", action="store_true")
    args = parser.parse_args()
    if args.command == "analyze":
        analyze(ROOT / args.report, ROOT / args.manifest)
    else:
        verify(ROOT / args.manifest, args.check_local)


if __name__ == "__main__":
    main()
