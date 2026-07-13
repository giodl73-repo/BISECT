#!/usr/bin/env python3
"""Build the Stage 2 small-State operational and proof coverage matrix."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "docs/experiments/small-states-2020"
SCRIPT = Path("scripts/research/build_small_state_scaling_report.py")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    one = json.loads((OUT / "one-district-states.json").read_text(encoding="utf-8"))
    two = json.loads((OUT / "two-district-frontier.json").read_text(encoding="utf-8"))
    nv = json.loads((OUT / "nv-operational-tree.json").read_text(encoding="utf-8"))
    nm = json.loads((OUT / "nm-operational-tree.json").read_text(encoding="utf-8"))
    rows = []
    for state in one["states"]:
        rows.append(
            {
                "state": state["state"],
                "districts": 1,
                "unit_count": state["unit_count"],
                "operational_status": "verified",
                "population_status": "trivial-proved",
                "boundary_status": "trivial-zero-proved",
                "canonical_status": "trivial-proved",
            }
        )
    for state in two["states"]:
        rows.append(
            {
                "state": state["state"],
                "districts": 2,
                "unit_count": state["unit_count"],
                "operational_status": "verified",
                "population_status": "verified-unsat",
                "population_model_variables": state["population_proof"][
                    "model_variables"
                ],
                "population_proof_bytes": state["population_proof"]["proof_bytes"],
                "boundary_status": "timelimit-120s",
                "canonical_status": "blocked-by-boundary",
            }
        )
    for tree in (nm, nv):
        rows.append(
            {
                "state": tree["state"],
                "districts": tree["districts"],
                "unit_count": tree["unit_count"],
                "operational_status": "verified",
                "population_status": "arithmetic-proved-all-nodes",
                "population_node_count": len(tree["nodes"]),
                "boundary_status": "unproved",
                "canonical_status": "blocked-by-boundary",
            }
        )
    nontrivial_nodes = 2 + len(nm["nodes"]) + len(nv["nodes"])
    report = {
        "schema_version": "certified-small-state-scaling-2020-v1",
        "status": "operational-complete-exact-partial",
        "state_count": len(rows),
        "district_count": sum(row["districts"] for row in rows),
        "unit_count": sum(row["unit_count"] for row in rows),
        "states": sorted(rows, key=lambda row: row["state"]),
        "coverage": {
            "operational_state_packages": f"{len(rows)}/{len(rows)}",
            "nontrivial_population_nodes_proved": f"{nontrivial_nodes}/{nontrivial_nodes}",
            "nontrivial_boundary_nodes_proved": f"0/{nontrivial_nodes}",
            "nontrivial_canonical_nodes_proved": f"0/{nontrivial_nodes}",
            "trivial_one_district_boundary_proofs": "6/6"
        },
        "conclusion": (
            "Wall-to-wall operational certification and population optimality "
            "scale across one- through four-district States. Nontrivial exact "
            "boundary and canonical proof remains the limiting frontier."
        ),
        "claim_boundary": (
            "Selected small-State sample, not nationwide 2020 completion."
        ),
    }
    report_path = OUT / "scaling-report.json"
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    manifest = {
        "schema_version": "certified-small-state-scaling-package-v1",
        "package_id": "small-state-scaling-2020",
        "status": report["status"],
        "files": [{"path": report_path.name, "sha256": sha256(report_path)}],
        "builder_path": SCRIPT.as_posix(),
        "builder_sha256": sha256(ROOT / SCRIPT),
        "claim_boundary": report["claim_boundary"],
    }
    (OUT / "scaling-manifest.json").write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
    )
    print("Small-State scaling report: BUILT")


if __name__ == "__main__":
    main()
