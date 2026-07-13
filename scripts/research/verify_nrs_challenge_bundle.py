#!/usr/bin/env python3
"""Verify hashes and cross-package claim anchors in the NRS challenge bundle."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    root = Path(".").resolve()
    manifest_path = root / "docs/external/nrs-v0.1-challenge-bundle/manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if manifest["schema_version"] != "nrs-challenge-bundle v1":
        raise SystemExit("unsupported challenge-bundle schema")
    for row in manifest["files"]:
        path = root / row["path"]
        if not path.is_file():
            raise SystemExit(f"missing bundle artifact: {row['path']}")
        actual = sha256(path)
        if actual != row["sha256"]:
            raise SystemExit(f"hash mismatch: {row['path']}")

    overlay_text = (
        root / "docs/fixtures/nrs-reference-v0.1/runtime-overlay.patch"
    ).read_text(encoding="utf-8")
    expected_overlay_files = {
        "Cargo.lock",
        "crates/bisect-cli/src/algo_config.rs",
        "crates/bisect-cli/src/build_cmd.rs",
        "crates/bisect-cli/src/fetch.rs",
        "crates/bisect-cli/src/output.rs",
        "rust-toolchain.toml",
    }
    actual_overlay_files = {
        line.removeprefix("diff --git a/").split(" b/", 1)[0]
        for line in overlay_text.splitlines()
        if line.startswith("diff --git a/")
    }
    if actual_overlay_files != expected_overlay_files:
        raise SystemExit(
            f"runtime overlay file set drift: {sorted(actual_overlay_files)}"
        )
    for required_text in [
        "pub seed: Option<u64>",
        "Rhode_Island",
        "BTreeMap<String, usize>",
        'channel = "1.95.0"',
    ]:
        if required_text not in overlay_text:
            raise SystemExit(f"runtime overlay missing required change: {required_text}")

    reference = json.loads(
        (root / "docs/fixtures/nrs-reference-v0.1/reference_manifest.json").read_text()
    )
    if reference["expected_output"]["canonical_assignment_sha256"] != (
        "6cd96b33ac8fdae2d8e5e4b7bc9674358311eed62becbe624e6913d1507b4822"
    ):
        raise SystemExit("reference assignment identity drift")
    ensemble = json.loads(
        (
            root
            / "docs/examples/g-ensemble-evidence-packages/G.1-G.3+real-2020/manifest.json"
        ).read_text()
    )
    if ensemble["status"] != "active" or set(ensemble["papers"]) != {"G.1", "G.2", "G.3"}:
        raise SystemExit("ensemble evidence posture drift")
    exact = json.loads(
        (root / "docs/examples/exact-canonical/manifest.json").read_text()
    )
    if exact["status"] != "active" or exact["schema_version"] != (
        "exact-canonical-fixture-manifest-v1"
    ):
        raise SystemExit("exact canonical fixture posture drift")
    independent = json.loads(
        (
            root
            / "docs/examples/exact-canonical/independent-verifier-report.json"
        ).read_text()
    )
    if independent["verifier_id"] != "python-exact-canonical-independent-v1":
        raise SystemExit("independent exact verifier identity drift")
    if len(independent["positive_cases"]) != 2 or len(
        independent["negative_cases"]
    ) != 5:
        raise SystemExit("independent exact verifier corpus drift")
    frontier = json.loads(
        (
            root / "docs/experiments/exact-canonical/ri-2020-block-frontier.json"
        ).read_text()
    )
    if frontier["status"] != "blocked" or frontier["observed_instance"][
        "tiger_block_rows"
    ] != 25_649:
        raise SystemExit("small-State exact frontier drift")
    if frontier["exact_reference"]["candidate_decimal_digits"] != 7_721:
        raise SystemExit("small-State exact search-scale drift")
    recursive = json.loads(
        (root / "docs/examples/certified-recursive/manifest.json").read_text()
    )
    if recursive["status"] != "active" or recursive["schema_version"] != (
        "certified-recursive-fixture-manifest-v1"
    ):
        raise SystemExit("certified recursive fixture posture drift")
    proof_backend = json.loads(
        (
            root / "docs/examples/certified-proof-backend/path8-root/manifest.json"
        ).read_text()
    )
    if proof_backend["proof_generator_status"] != (
        "smoke-verified-proof-not-bundled"
    ):
        raise SystemExit("certified proof backend posture drift")
    comparison = json.loads(
        (
            root / "docs/examples/certified-comparison/path8-root/comparison.json"
        ).read_text()
    )
    if comparison["rows"][1]["objective"] != comparison["rows"][0]["objective"]:
        raise SystemExit("certified versus METIS comparison drift")
    proof_smoke = json.loads(
        (root / "docs/examples/proof-toolchain-smoke/provenance.json").read_text()
    )
    if proof_smoke["status"] != "verified":
        raise SystemExit("proof toolchain smoke posture drift")
    ri_proof = json.loads(
        (root / "docs/examples/ri-proof-frontier/provenance.json").read_text()
    )
    if ri_proof["population_stage"]["status"] != "verified-unsat":
        raise SystemExit("RI population proof posture drift")
    ri_frontier = json.loads(
        (
            root
            / "docs/experiments/certified-recursive/ri-2020-root-frontier.json"
        ).read_text()
    )
    if ri_frontier["graph"]["unit_count"] != 25_649 or ri_frontier["graph"][
        "final_component_count"
    ] != 1:
        raise SystemExit("RI certified root frontier drift")
    discovery = json.loads(
        (
            root / "docs/experiments/scalable-certified/ri-discovery-frontier.json"
        ).read_text()
    )
    if discovery["candidate"]["max_population_deviation_scaled"] != 1:
        raise SystemExit("RI certified discovery frontier drift")
    models = json.loads(
        (root / "docs/experiments/scalable-certified/ri-model-frontier.json").read_text()
    )
    if models["population_stage"]["status"] != "verified-unsat":
        raise SystemExit("RI compact proof model frontier drift")
    print("NRS challenge bundle verification: PASS")


if __name__ == "__main__":
    main()
