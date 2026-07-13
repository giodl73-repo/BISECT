#!/usr/bin/env python3
"""Build committed adversarial Exact Canonical certificate fixtures."""

from __future__ import annotations

import copy
import hashlib
import json
import shutil
import subprocess
import tempfile
from pathlib import Path


ROOT = Path("docs/examples/exact-canonical")
CORPUS = ROOT / "negative-corpus"


def canonical_hash(value: object) -> str:
    encoded = json.dumps(
        value, ensure_ascii=False, separators=(",", ":"), sort_keys=True
    ).encode("utf-8")
    return f"sha256:{hashlib.sha256(encoded).hexdigest()}"


def refresh_certificate_id(certificate: dict) -> None:
    certificate["certificate_id"] = canonical_hash(
        {
            "schema_version": certificate["schema_version"],
            "instance_hash": certificate["instance_hash"],
            "model_id": certificate["model_id"],
            "result": certificate["result"],
            "proof": certificate["proof"],
        }
    )


def write_case(
    name: str,
    instance: dict,
    certificate: dict,
    proof: dict,
    expected_error: str,
    description: str,
) -> None:
    case = CORPUS / name
    case.mkdir(parents=True, exist_ok=True)
    documents = {
        "exact-canonical-instance.json": instance,
        "exact-canonical-certificate.json": certificate,
        "exact-canonical-proof.json": proof,
        "expected.json": {
            "schema_version": "exact-canonical-negative-fixture-v1",
            "case": name,
            "expected_error": expected_error,
            "description": description,
        },
    }
    for filename, document in documents.items():
        (case / filename).write_text(
            json.dumps(document, indent=2) + "\n", encoding="utf-8"
        )


def load(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def refresh_root_manifest() -> None:
    manifest_path = ROOT / "manifest.json"
    manifest = load(manifest_path)
    manifest["files"] = [
        {
            "path": path.relative_to(ROOT).as_posix(),
            "sha256": sha256(path),
        }
        for path in sorted(ROOT.rglob("*"))
        if path.is_file() and path != manifest_path
    ]
    manifest["builder_path"] = "scripts/research/build_exact_negative_corpus.py"
    manifest["independent_verifier_path"] = (
        "scripts/research/verify_exact_canonical_independent.py"
    )
    manifest["verification_commands"] = [
        "python scripts/research/verify_exact_canonical_fixtures.py",
        "python scripts/research/verify_exact_canonical_independent.py corpus",
        "cargo test -p bisect-ilp canonical -- --test-threads=1",
        "cargo test -p bisect-ilp --test exact_negative_corpus -- --test-threads=1",
        "cargo test -p bisect-cli exact_cmd --lib -- --test-threads=1",
    ]
    manifest_path.write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
    )


def cycle4_artifacts() -> tuple[dict, dict, dict]:
    instance = {
        "schema_version": "exact-canonical-instance-v1",
        "model_id": "exact-canonical-k2-exhaustive-v1",
        "unit_ids": ["u0", "u1", "u2", "u3"],
        "populations": [100, 100, 100, 100],
        "edges": [
            {"left": 0, "right": 1, "weight": 1},
            {"left": 0, "right": 3, "weight": 1},
            {"left": 1, "right": 2, "weight": 1},
            {"left": 2, "right": 3, "weight": 1},
        ],
        "k": 2,
    }
    with tempfile.TemporaryDirectory() as temp:
        temp_path = Path(temp)
        instance_path = temp_path / "instance.json"
        out_dir = temp_path / "output"
        instance_path.write_text(json.dumps(instance, indent=2), encoding="utf-8")
        subprocess.run(
            [
                "cargo",
                "run",
                "-q",
                "-p",
                "bisect-ilp",
                "--example",
                "exact_canonical",
                "--",
                "solve",
                str(instance_path),
                str(out_dir),
            ],
            check=True,
        )
        return (
            instance,
            load(out_dir / "exact-canonical-certificate.json"),
            load(out_dir / "exact-canonical-proof.json"),
        )


def main() -> None:
    if CORPUS.exists():
        shutil.rmtree(CORPUS)

    path_root = ROOT / "path4-optimal/output"
    path_instance = load(path_root / "exact-canonical-instance.json")
    path_certificate = load(path_root / "exact-canonical-certificate.json")
    path_proof = load(path_root / "exact-canonical-proof.json")

    certificate = copy.deepcopy(path_certificate)
    certificate["result"]["assignment"] = [0, 1, 1, 1]
    certificate["result"]["objective"] = {
        "primary": {
            "max_population_deviation_scaled": 200,
            "total_population_deviation_scaled": 400,
            "weighted_boundary_cut": 1,
        },
        "canonical_assignment": [0, 1, 1, 1],
    }
    refresh_certificate_id(certificate)
    write_case(
        "false-optimum",
        path_instance,
        certificate,
        path_proof,
        "result-mismatch",
        "Claims a feasible but population-suboptimal connected assignment as exact.",
    )

    certificate = copy.deepcopy(path_certificate)
    certificate["result"] = {"result": "infeasible"}
    refresh_certificate_id(certificate)
    write_case(
        "false-infeasibility",
        path_instance,
        certificate,
        path_proof,
        "result-mismatch",
        "Claims infeasibility for the feasible path-4 instance.",
    )

    cycle_instance, cycle_certificate, cycle_proof = cycle4_artifacts()
    certificate = copy.deepcopy(cycle_certificate)
    certificate["result"]["assignment"] = [0, 1, 1, 0]
    certificate["result"]["objective"]["canonical_assignment"] = [0, 1, 1, 0]
    refresh_certificate_id(certificate)
    write_case(
        "noncanonical-tie",
        cycle_instance,
        certificate,
        cycle_proof,
        "result-mismatch",
        "Submits a primary-objective tie that loses the canonical assignment tie-break.",
    )

    certificate = copy.deepcopy(path_certificate)
    certificate["result"]["assignment"] = [0, 1, 0, 1]
    certificate["result"]["objective"] = {
        "primary": {
            "max_population_deviation_scaled": 0,
            "total_population_deviation_scaled": 0,
            "weighted_boundary_cut": 3,
        },
        "canonical_assignment": [0, 1, 0, 1],
    }
    refresh_certificate_id(certificate)
    write_case(
        "disconnected-assignment",
        path_instance,
        certificate,
        path_proof,
        "result-mismatch",
        "Submits a population-balanced assignment with disconnected districts.",
    )

    certificate = copy.deepcopy(path_certificate)
    certificate["certificate_id"] = "sha256:" + ("0" * 64)
    write_case(
        "certificate-hash-tamper",
        path_instance,
        certificate,
        path_proof,
        "certificate-id-mismatch",
        "Changes the certificate ID without changing its canonical content.",
    )
    refresh_root_manifest()
    print("Exact Canonical negative corpus build: PASS")


if __name__ == "__main__":
    main()
