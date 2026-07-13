#!/usr/bin/env python3
"""Build the NRS external challenge-bundle manifest."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    root = Path(".").resolve()
    bundle = root / "docs/external/nrs-v0.1-challenge-bundle"
    paths = [
        "docs/external/nrs-v0.1-challenge-bundle/README.md",
        "docs/external/nrs-v0.1-challenge-bundle/replicate-reference.ps1",
        "docs/external/nrs-v0.1-challenge-bundle/replication-record-template.md",
        "docs/external/nrs-v0.1-challenge-bundle/replication-record-2026-07-10-agent.md",
        "docs/external/nrs-v0.1-challenge-bundle/final-bundle-validation.md",
        "docs/external/nrs-v0.1-challenge-bundle/reviewer-criteria.md",
        "docs/external/nrs-v0.1-challenge-bundle/issue-and-response-process.md",
        "docs/external/nrs-v0.1-challenge-bundle/adoption-matrix.md",
        "docs/external/nrs-v0.1-challenge-bundle/claims-map.md",
        "docs/specs/2026-07-09-national-redistricting-standard-v0.1.md",
        "docs/specs/2026-07-10-exact-canonical-benchmark-north-star.md",
        "docs/concepts/certified-recursive-bisection.md",
        "docs/file-formats/exact-canonical-certificates.md",
        "docs/file-formats/certified-recursive-bisection.md",
        "docs/file-formats/certified-proof-backend.md",
        "docs/file-formats/certified-single-district.md",
        "docs/examples/exact-canonical/manifest.json",
        "docs/examples/exact-canonical/independent-verifier-report.json",
        "docs/examples/certified-recursive/README.md",
        "docs/examples/certified-recursive/manifest.json",
        "docs/examples/certified-recursive/path8-k4/output/certified-bisection-tree.json",
        "docs/examples/certified-proof-backend/README.md",
        "docs/examples/certified-proof-backend/path8-root/manifest.json",
        "docs/examples/certified-comparison/README.md",
        "docs/examples/certified-comparison/path8-root/manifest.json",
        "docs/examples/proof-toolchain-smoke/manifest.json",
        "docs/examples/ri-proof-frontier/manifest.json",
        "docs/examples/certified-single-district/manifest.json",
        "docs/experiments/exact-canonical/manifest.json",
        "docs/experiments/exact-canonical/README.md",
        "docs/experiments/exact-canonical/ri-2020-block-frontier.json",
        "docs/experiments/certified-recursive/README.md",
        "docs/experiments/certified-recursive/manifest.json",
        "docs/experiments/certified-recursive/ri-2020-root-frontier.json",
        "docs/experiments/scalable-certified/manifest.json",
        "docs/experiments/scalable-certified/ri-discovery-frontier.json",
        "docs/experiments/scalable-certified/README.md",
        "docs/experiments/scalable-certified/model-manifest.json",
        "docs/experiments/scalable-certified/ri-model-frontier.json",
        "docs/experiments/scalable-certified/solver-frontier.json",
        "docs/experiments/scalable-certified/regional-decomposition-frontier.json",
        "docs/experiments/small-states-2020/manifest.json",
        "docs/experiments/small-states-2020/one-district-states.json",
        "docs/experiments/small-states-2020/two-district-manifest.json",
        "docs/experiments/small-states-2020/two-district-frontier.json",
        "docs/experiments/small-states-2020/nv-tree-manifest.json",
        "docs/experiments/small-states-2020/nv-operational-tree.json",
        "docs/experiments/small-states-2020/nm-tree-manifest.json",
        "docs/experiments/small-states-2020/nm-operational-tree.json",
        "docs/experiments/small-states-2020/scaling-manifest.json",
        "docs/experiments/small-states-2020/scaling-report.json",
        "docs/experiments/nationwide-2020/manifest.json",
        "docs/experiments/nationwide-2020/inventory.json",
        "docs/experiments/nationwide-2020/rctx-manifest.json",
        "docs/experiments/nationwide-2020/rctx-verification.json",
        "docs/experiments/scalable-certified/FRONTIER-REVIEW.md",
        "docs/examples/exact-canonical/path4-optimal/output/exact-canonical-certificate.json",
        "docs/examples/exact-canonical/path4-optimal/output/exact-canonical-proof.json",
        "docs/examples/exact-canonical/three-islands-infeasible/output/exact-canonical-certificate.json",
        "docs/examples/exact-canonical/three-islands-infeasible/output/exact-canonical-proof.json",
        "scripts/research/verify_exact_canonical_independent.py",
        "scripts/research/verify_certified_recursive_fixtures.py",
        "scripts/research/verify_certified_proof_backend.py",
        "scripts/research/verify_certified_vs_metis.py",
        "scripts/research/verify_proof_toolchain_smoke.py",
        "scripts/research/verify_ri_proof_frontier.py",
        "scripts/research/analyze_small_state_exact_frontier.py",
        "scripts/research/build_ri_block_rctx.py",
        "scripts/research/analyze_ri_certified_discovery.py",
        "scripts/research/analyze_ri_model_package.py",
        "docs/legal/MODEL_FEDERAL_STATUTE.md",
        "docs/legal/NRS_TECHNICAL_SCHEDULE_V0.1.md",
        "docs/legal/NRS_EVALUATION_SCHEDULE_V0.1.md",
        "docs/legal/STATUTE_ONE_PAGER.md",
        "docs/fixtures/nrs-reference-v0.1/reference_manifest.json",
        "docs/fixtures/nrs-reference-v0.1/runtime-overlay.patch",
        "docs/fixtures/nrs-reference-v0.1/config.yml",
        "docs/examples/g-ensemble-evidence-packages/G.1-G.3+real-2020/manifest.json",
        "docs/examples/g-ensemble-evidence-packages/G.1-G.3+real-2020/analysis.json",
        "docs/papers/A.0+synthesis-metapaper.pdf",
        "docs/papers/A.5+policy-brief.pdf",
        "docs/papers/U.21+certified-recursive-bisection.pdf",
        "docs/papers/B.02+one-federal-law.pdf",
    ]
    files = [
        {
            "path": path,
            "sha256": sha256(root / path),
        }
        for path in paths
    ]
    manifest = {
        "schema_version": "nrs-challenge-bundle v1",
        "package_id": "nrs-v0.1-challenge-bundle",
        "status": "internal-external-review-candidate",
        "base_source_commit": "d61a7136d60c27ecdd451067a1c08a063581820f",
        "files": files,
        "verification_commands": [
            "python scripts/research/verify_nrs_challenge_bundle.py",
            "python scripts/research/verify_real_ensemble_package.py docs/examples/g-ensemble-evidence-packages/G.1-G.3+real-2020",
            "python scripts/research/verify_exact_canonical_fixtures.py",
        ],
        "claim_boundary": "Supports independent challenge and internal non-author replication; not human external validation, public release readiness, legal certification, or exact optimality.",
    }
    (bundle / "manifest.json").write_text(json.dumps(manifest, indent=2), encoding="utf-8")


if __name__ == "__main__":
    main()
