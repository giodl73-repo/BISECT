from __future__ import annotations

import copy
import importlib.util
from pathlib import Path

import pytest


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts/research/verify_exact_canonical_independent.py"
SPEC = importlib.util.spec_from_file_location("exact_independent_verifier", MODULE_PATH)
assert SPEC and SPEC.loader
VERIFIER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(VERIFIER)


def test_independent_verifier_accepts_positive_and_rejects_negative_corpus() -> None:
    report = VERIFIER.verify_corpus(ROOT / "docs/examples/exact-canonical")
    assert [row["case"] for row in report["positive_cases"]] == [
        "path4-optimal",
        "three-islands-infeasible",
    ]
    assert len(report["negative_cases"]) == 5


def test_independent_verifier_rejects_proof_commitment_tamper() -> None:
    case = ROOT / "docs/examples/exact-canonical/path4-optimal/output"
    instance = VERIFIER.load_json(case / "exact-canonical-instance.json")
    certificate = VERIFIER.load_json(case / "exact-canonical-certificate.json")
    proof = VERIFIER.load_json(case / "exact-canonical-proof.json")
    tampered = copy.deepcopy(proof)
    tampered["search_commitment"] = "sha256:" + ("0" * 64)
    tampered["transcript_id"] = VERIFIER.canonical_hash(
        VERIFIER.proof_projection(tampered)
    )
    with pytest.raises(VERIFIER.VerificationError) as error:
        VERIFIER.verify_artifacts(instance, certificate, tampered)
    assert error.value.code == "transcript-mismatch"
