#!/usr/bin/env python3
"""Build the hash manifest for the G.1-G.3 real ensemble package."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("package", type=Path)
    args = parser.parse_args()
    root = args.package.resolve()
    roles = {
        "design.md": ("other", "Frozen experiment design."),
        "deviations.md": ("other", "Eligibility and election-coverage deviations."),
        "software.json": ("other", "Software versions and source/input hashes."),
        "analysis.json": ("metric-output", "Regenerated percentiles, diagnostics, and cross-tool comparisons."),
        "summary.csv": ("diagnostic", "Flat summary of state/tool/metric diagnostics."),
    }
    for state in ["ri", "ia", "nc"]:
        roles.update(
            {
                f"{state}/gerrychain-trace.json": (
                    "external-trace",
                    "Independent GerryChain 0.3.2 ReCom trace.",
                ),
                f"{state}/rust-trace.json": (
                    "other",
                    "Rust ReCom trace using the GerryChain-compatible tree kernel.",
                ),
                f"{state}/election-input.json": (
                    "election-input",
                    "Aligned 2016/2020 presidential tract estimates with coverage counts.",
                ),
                f"{state}/baseline.rplan": (
                    "bisect-plan",
                    "NRS v0.1 tract-level BISECT benchmark in RPLAN v0.2.",
                ),
                f"{state}/context.rctx": (
                    "rctx-context",
                    "Hash-bound graph, population, unit, and source context.",
                ),
                f"{state}/audit-certificate.json": (
                    "other",
                    "Plan-shape, population, and contiguity audit certificate.",
                ),
            }
        )
    files = []
    for path, (role, description) in roles.items():
        full = root / path
        files.append(
            {
                "path": path,
                "sha256": sha256(full),
                "role": role,
                "description": description,
            }
        )
    manifest = {
        "schema_version": "g-ensemble-evidence-manifest v1",
        "package_id": "G.1-G.3+real-2020",
        "status": "active",
        "papers": ["G.1", "G.2", "G.3"],
        "claims": [
            {
                "paper": "G.1",
                "claim": "Three-state cut-fraction percentiles and cross-tool differences are trace-backed with convergence and partition-space diagnostics.",
                "required_roles": [
                    "external-trace",
                    "metric-output",
                    "bisect-plan",
                    "rctx-context",
                    "diagnostic",
                ],
            },
            {
                "paper": "G.2",
                "claim": "Three-state 2020 presidential Democratic-seat percentiles are backed by aligned election inputs, traces, baselines, and diagnostics.",
                "required_roles": [
                    "external-trace",
                    "election-input",
                    "bisect-plan",
                    "rctx-context",
                    "diagnostic",
                ],
            },
            {
                "paper": "G.3",
                "claim": "The package supports tract-graph cut-fraction positions only and explicitly withdraws polygon compactness percentile claims.",
                "required_roles": [
                    "external-trace",
                    "metric-output",
                    "bisect-plan",
                    "rctx-context",
                ],
            },
        ],
        "files": files,
        "verifier_path": "scripts/research/verify_real_ensemble_package.py",
        "verification_commands": [
            "python scripts/research/verify_real_ensemble_package.py docs/examples/g-ensemble-evidence-packages/G.1-G.3+real-2020",
            "cargo test -p bisect-ensemble real_g1_g3_package_validates",
        ],
        "missing_evidence": [],
    }
    (root / "manifest.json").write_text(json.dumps(manifest, indent=2), encoding="utf-8")


if __name__ == "__main__":
    main()
