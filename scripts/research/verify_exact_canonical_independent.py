#!/usr/bin/env python3
"""Independent Python verifier for bounded Exact Canonical artifacts."""

from __future__ import annotations

import argparse
import hashlib
import json
from collections import deque
from pathlib import Path
from typing import Any


INSTANCE_SCHEMA = "exact-canonical-instance-v1"
CERTIFICATE_SCHEMA = "exact-canonical-certificate-v1"
PROOF_SCHEMA = "exact-canonical-proof-v1"
MODEL_ID = "exact-canonical-k2-exhaustive-v1"
ENUMERATION_ORDER = (
    "unit-0-fixed-label-0; nonzero masks ascending through 2^(n-1)-1"
)
VERIFIER_ID = "python-exact-canonical-independent-v1"
MAX_UNITS = 24
I64_MAX = (1 << 63) - 1
U64_MAX = (1 << 64) - 1


class VerificationError(Exception):
    def __init__(self, code: str, message: str):
        super().__init__(message)
        self.code = code


def fail(code: str, message: str) -> None:
    raise VerificationError(code, message)


def canonical_hash(value: Any) -> str:
    encoded = json.dumps(
        value, ensure_ascii=False, separators=(",", ":"), sort_keys=True
    ).encode("utf-8")
    return f"sha256:{hashlib.sha256(encoded).hexdigest()}"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_json(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        fail("invalid-json-shape", f"{path} must contain a JSON object")
    return value


def require_int(value: Any, name: str, minimum: int, maximum: int) -> int:
    if type(value) is not int or not minimum <= value <= maximum:
        fail("invalid-instance", f"{name} must be an integer in [{minimum}, {maximum}]")
    return value


def normalize_instance(raw: dict[str, Any]) -> dict[str, Any]:
    if raw.get("schema_version") != INSTANCE_SCHEMA:
        fail("instance-schema", "unsupported exact instance schema")
    if raw.get("model_id") != MODEL_ID:
        fail("model-mismatch", "unsupported exact model")
    unit_ids = raw.get("unit_ids")
    populations = raw.get("populations")
    edges = raw.get("edges")
    if not isinstance(unit_ids, list) or not all(
        isinstance(unit_id, str) and unit_id.strip() for unit_id in unit_ids
    ):
        fail("invalid-instance", "unit_ids must be nonempty strings")
    if not 2 <= len(unit_ids) <= MAX_UNITS:
        fail("invalid-instance", "unit count is outside the bounded verifier range")
    if unit_ids != sorted(unit_ids) or len(set(unit_ids)) != len(unit_ids):
        fail("invalid-instance", "unit_ids must be unique and canonically sorted")
    if not isinstance(populations, list) or len(populations) != len(unit_ids):
        fail("invalid-instance", "population vector length mismatch")
    normalized_populations = [
        require_int(value, f"populations[{index}]", 0, I64_MAX)
        for index, value in enumerate(populations)
    ]
    total_population = sum(normalized_populations)
    if total_population > I64_MAX or total_population * 2 > I64_MAX:
        fail("numeric-overflow", "population objective exceeds the i64 reference range")
    if not isinstance(edges, list):
        fail("invalid-instance", "edges must be an array")
    normalized_edges = []
    seen_edges: set[tuple[int, int]] = set()
    total_edge_weight = 0
    for index, edge in enumerate(edges):
        if not isinstance(edge, dict):
            fail("invalid-instance", f"edges[{index}] must be an object")
        left = require_int(edge.get("left"), f"edges[{index}].left", 0, len(unit_ids) - 1)
        right = require_int(
            edge.get("right"), f"edges[{index}].right", 0, len(unit_ids) - 1
        )
        weight = require_int(edge.get("weight"), f"edges[{index}].weight", 1, U64_MAX)
        if left >= right or (left, right) in seen_edges:
            fail("invalid-instance", f"invalid or duplicate edge ({left}, {right})")
        seen_edges.add((left, right))
        total_edge_weight += weight
        if total_edge_weight > U64_MAX:
            fail("numeric-overflow", "edge-weight total exceeds u64")
        normalized_edges.append({"left": left, "right": right, "weight": weight})
    if type(raw.get("k")) is not int or raw.get("k") != 2:
        fail("unsupported-district-count", "the bounded verifier requires k=2")
    return {
        "schema_version": INSTANCE_SCHEMA,
        "model_id": MODEL_ID,
        "unit_ids": unit_ids,
        "populations": normalized_populations,
        "edges": normalized_edges,
        "k": 2,
    }


def build_adjacency(instance: dict[str, Any]) -> list[list[int]]:
    adjacency = [[] for _ in instance["unit_ids"]]
    for edge in instance["edges"]:
        adjacency[edge["left"]].append(edge["right"])
        adjacency[edge["right"]].append(edge["left"])
    return adjacency


def connected(assignment: list[int], adjacency: list[list[int]]) -> bool:
    for district in (0, 1):
        units = [index for index, label in enumerate(assignment) if label == district]
        if not units:
            return False
        allowed = set(units)
        visited = {units[0]}
        queue = deque([units[0]])
        while queue:
            unit = queue.popleft()
            for neighbor in adjacency[unit]:
                if neighbor in allowed and neighbor not in visited:
                    visited.add(neighbor)
                    queue.append(neighbor)
        if visited != allowed:
            return False
    return True


def objective(instance: dict[str, Any], assignment: list[int]) -> tuple[int, int, int]:
    district_populations = [0, 0]
    for population, district in zip(instance["populations"], assignment, strict=True):
        district_populations[district] += population
    total_population = sum(district_populations)
    deviations = [
        abs(2 * district_population - total_population)
        for district_population in district_populations
    ]
    cut = sum(
        edge["weight"]
        for edge in instance["edges"]
        if assignment[edge["left"]] != assignment[edge["right"]]
    )
    return max(deviations), sum(deviations), cut


def enumerate_instance(instance: dict[str, Any]) -> dict[str, Any]:
    unit_count = len(instance["unit_ids"])
    candidate_count = (1 << (unit_count - 1)) - 1
    adjacency = build_adjacency(instance)
    transcript = hashlib.sha256(b"EXACT_CANONICAL_TRANSCRIPT_V1\0")
    feasible_count = 0
    tie_count = 0
    best_primary: tuple[int, int, int] | None = None
    best_assignment: list[int] | None = None
    for mask in range(1, candidate_count + 1):
        assignment = [0] + [
            (mask >> (unit - 1)) & 1 for unit in range(1, unit_count)
        ]
        transcript.update(mask.to_bytes(8, "little"))
        if not connected(assignment, adjacency):
            transcript.update(b"\x00")
            continue
        feasible_count += 1
        primary = objective(instance, assignment)
        transcript.update(b"\x01")
        for value in primary:
            transcript.update(value.to_bytes(8, "little"))
        transcript.update(bytes(assignment))
        if best_primary is None or primary < best_primary:
            best_primary = primary
            best_assignment = assignment
            tie_count = 1
        elif primary == best_primary:
            tie_count += 1
            if best_assignment is None or assignment < best_assignment:
                best_assignment = assignment
    primary_document = (
        None
        if best_primary is None
        else {
            "max_population_deviation_scaled": best_primary[0],
            "total_population_deviation_scaled": best_primary[1],
            "weighted_boundary_cut": best_primary[2],
        }
    )
    return {
        "candidate_count": candidate_count,
        "feasible_count": feasible_count,
        "primary_objective_ties": tie_count,
        "lower_bound": primary_document,
        "canonical_assignment": best_assignment,
        "search_commitment": f"sha256:{transcript.hexdigest()}",
    }


def proof_projection(proof: dict[str, Any]) -> dict[str, Any]:
    return {
        "schema_version": proof.get("schema_version"),
        "instance_hash": proof.get("instance_hash"),
        "model_id": proof.get("model_id"),
        "enumeration_order": proof.get("enumeration_order"),
        "fixed_label_unit": proof.get("fixed_label_unit"),
        "candidate_count": proof.get("candidate_count"),
        "feasible_count": proof.get("feasible_count"),
        "primary_objective_ties": proof.get("primary_objective_ties"),
        "lower_bound": proof.get("lower_bound"),
        "canonical_assignment": proof.get("canonical_assignment"),
        "search_commitment": proof.get("search_commitment"),
    }


def certificate_projection(certificate: dict[str, Any]) -> dict[str, Any]:
    return {
        "schema_version": certificate.get("schema_version"),
        "instance_hash": certificate.get("instance_hash"),
        "model_id": certificate.get("model_id"),
        "result": certificate.get("result"),
        "proof": certificate.get("proof"),
    }


def verify_artifacts(
    raw_instance: dict[str, Any],
    certificate: dict[str, Any],
    proof: dict[str, Any],
) -> None:
    instance = normalize_instance(raw_instance)
    instance_hash = canonical_hash(instance)
    if certificate.get("schema_version") != CERTIFICATE_SCHEMA:
        fail("certificate-schema", "unsupported exact certificate schema")
    if certificate.get("model_id") != MODEL_ID:
        fail("model-mismatch", "certificate model mismatch")
    if certificate.get("instance_hash") != instance_hash:
        fail("instance-hash-mismatch", "certificate instance hash mismatch")
    if certificate.get("certificate_id") != canonical_hash(
        certificate_projection(certificate)
    ):
        fail("certificate-id-mismatch", "certificate ID does not match its content")
    if proof.get("schema_version") != PROOF_SCHEMA:
        fail("proof-schema", "unsupported proof transcript schema")
    if proof.get("model_id") != MODEL_ID or proof.get("instance_hash") != instance_hash:
        fail("transcript-mismatch", "proof identity does not match the instance")
    if proof.get("transcript_id") != canonical_hash(proof_projection(proof)):
        fail("proof-id-mismatch", "proof transcript ID does not match its content")
    certificate_proof = certificate.get("proof")
    if not isinstance(certificate_proof, dict):
        fail("proof-mismatch", "certificate proof must be an object")
    if certificate_proof.get("transcript_id") != proof.get("transcript_id"):
        fail("transcript-mismatch", "certificate does not bind the submitted proof")

    search = enumerate_instance(instance)
    expected_proof = {
        "schema_version": PROOF_SCHEMA,
        "transcript_id": "",
        "instance_hash": instance_hash,
        "model_id": MODEL_ID,
        "enumeration_order": ENUMERATION_ORDER,
        "fixed_label_unit": 0,
        **search,
    }
    expected_proof["transcript_id"] = canonical_hash(proof_projection(expected_proof))
    expected_result = (
        {"result": "infeasible"}
        if search["canonical_assignment"] is None
        else {
            "result": "optimal",
            "assignment": search["canonical_assignment"],
            "objective": {
                "primary": search["lower_bound"],
                "canonical_assignment": search["canonical_assignment"],
            },
        }
    )
    expected_certificate_proof = {
        "proof_kind": "label-fixed-exhaustive-enumeration",
        "fixed_label_unit": 0,
        "enumerated_assignments": search["candidate_count"],
        "feasible_assignments": search["feasible_count"],
        "primary_objective_ties": search["primary_objective_ties"],
        "lower_bound": search["lower_bound"],
        "transcript_id": expected_proof["transcript_id"],
    }
    if certificate.get("result") != expected_result:
        fail("result-mismatch", "certificate result differs from exact enumeration")
    if certificate_proof != expected_certificate_proof:
        fail("proof-mismatch", "certificate proof statistics differ from enumeration")
    if proof != expected_proof:
        fail("transcript-mismatch", "proof transcript differs from exact enumeration")


def artifact_paths(root: Path) -> tuple[Path, Path, Path]:
    return (
        root / "exact-canonical-instance.json",
        root / "exact-canonical-certificate.json",
        root / "exact-canonical-proof.json",
    )


def verify_paths(instance_path: Path, certificate_path: Path, proof_path: Path) -> None:
    verify_artifacts(
        load_json(instance_path),
        load_json(certificate_path),
        load_json(proof_path),
    )


def verify_corpus(root: Path) -> dict[str, Any]:
    positives = {
        "path4-optimal": root / "path4-optimal/output",
        "three-islands-infeasible": root / "three-islands-infeasible/output",
    }
    positive_results = []
    for name, case_root in positives.items():
        paths = artifact_paths(case_root)
        verify_paths(*paths)
        positive_results.append(
            {
                "case": name,
                "result": "accepted",
                "artifact_sha256": [sha256(path) for path in paths],
            }
        )

    negative_results = []
    corpus = root / "negative-corpus"
    for case_root in sorted(path for path in corpus.iterdir() if path.is_dir()):
        expected = load_json(case_root / "expected.json")
        paths = artifact_paths(case_root)
        try:
            verify_paths(*paths)
        except VerificationError as error:
            if error.code != expected.get("expected_error"):
                fail(
                    "unexpected-rejection",
                    f"{case_root.name}: expected {expected.get('expected_error')}, got {error.code}",
                )
            negative_results.append(
                {
                    "case": case_root.name,
                    "result": "rejected",
                    "error": error.code,
                    "artifact_sha256": [sha256(path) for path in paths],
                }
            )
        else:
            fail("unexpected-acceptance", f"{case_root.name} unexpectedly verified")
    return {
        "schema_version": "exact-canonical-independent-verifier-report-v1",
        "verifier_id": VERIFIER_ID,
        "verifier_path": "scripts/research/verify_exact_canonical_independent.py",
        "verifier_sha256": sha256(Path(__file__)),
        "positive_cases": positive_results,
        "negative_cases": negative_results,
        "claim_boundary": (
            "Independent Python exhaustive verification of bounded E0 JSON artifacts; "
            "not national-scale exactness or production-solver validation."
        ),
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)
    verify_parser = subparsers.add_parser("verify")
    verify_parser.add_argument("instance", type=Path)
    verify_parser.add_argument("certificate", type=Path)
    verify_parser.add_argument("proof", type=Path)
    corpus_parser = subparsers.add_parser("corpus")
    corpus_parser.add_argument(
        "root", type=Path, nargs="?", default=Path("docs/examples/exact-canonical")
    )
    corpus_parser.add_argument("--report", type=Path)
    args = parser.parse_args()
    try:
        if args.command == "verify":
            verify_paths(args.instance, args.certificate, args.proof)
            print("Independent Exact Canonical verification: PASS")
        else:
            report = verify_corpus(args.root)
            if args.report:
                args.report.write_text(
                    json.dumps(report, indent=2) + "\n", encoding="utf-8"
                )
            print(
                "Independent Exact Canonical corpus verification: PASS "
                f"({len(report['positive_cases'])} accepted, "
                f"{len(report['negative_cases'])} rejected)"
            )
    except VerificationError as error:
        raise SystemExit(f"{error.code}: {error}") from error


if __name__ == "__main__":
    main()
