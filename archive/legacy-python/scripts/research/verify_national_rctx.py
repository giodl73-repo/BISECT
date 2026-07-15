#!/usr/bin/env python3
"""Verify all 50 connected 2020 State block RCTX files."""

from __future__ import annotations

import hashlib
import json
from collections import deque
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "docs/experiments/nationwide-2020"
SCRIPT = Path("scripts/research/verify_national_rctx.py")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def canonical_hash(value: object) -> str:
    encoded = json.dumps(
        value, ensure_ascii=False, separators=(",", ":"), sort_keys=True
    ).encode("utf-8")
    return f"sha256:{hashlib.sha256(encoded).hexdigest()}"


def main() -> None:
    inventory = json.loads((OUT / "inventory.json").read_text(encoding="utf-8"))
    rows = []
    for state in sorted(row["state"] for row in inventory["states"]):
        path = ROOT / f"data/2020/certified/{state.lower()}_blocks_2020.rctx"
        context = json.loads(path.read_text(encoding="utf-8"))
        projection = {
            key: context[key]
            for key in ("units", "graph", "populations", "source_hashes")
        }
        if context["context_hash"] != canonical_hash(projection):
            raise SystemExit(f"{state} context hash mismatch")
        adjacency = context["graph"]["adjacency"]
        seen = {0}
        queue = deque([0])
        while queue:
            unit = queue.popleft()
            for edge in adjacency[unit]:
                neighbor = edge["to"]
                if neighbor not in seen:
                    seen.add(neighbor)
                    queue.append(neighbor)
        if len(seen) != len(adjacency):
            raise SystemExit(f"{state} context is disconnected")
        edge_count = sum(len(neighbors) for neighbors in adjacency) // 2
        bridge_count = (
            sum(
                edge.get("kind") == "bridge"
                for neighbors in adjacency
                for edge in neighbors
            )
            // 2
        )
        rows.append(
            {
                "state": state,
                "unit_count": len(context["units"]["unit_ids"]),
                "population_total": sum(context["populations"]),
                "edge_count": edge_count,
                "bridge_edge_count": bridge_count,
                "rctx_bytes": path.stat().st_size,
                "rctx_sha256": sha256(path),
                "context_hash": context["context_hash"],
                "status": "verified",
            }
        )
        print(f"{state}: verified")
    report = {
        "schema_version": "certified-national-rctx-verification-v1",
        "status": "verified",
        "state_count": len(rows),
        "unit_count": sum(row["unit_count"] for row in rows),
        "population_total": sum(row["population_total"] for row in rows),
        "edge_count": sum(row["edge_count"] for row in rows),
        "bridge_edge_count": sum(row["bridge_edge_count"] for row in rows),
        "rctx_bytes": sum(row["rctx_bytes"] for row in rows),
        "states": rows,
        "claim_boundary": "All 50 local 2020 block contexts are hash-valid and connected; no district assignments are claimed.",
    }
    report_path = OUT / "rctx-verification.json"
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    manifest = {
        "schema_version": "certified-national-rctx-verification-package-v1",
        "package_id": "nationwide-2020-rctx-verification",
        "status": report["status"],
        "files": [{"path": report_path.name, "sha256": sha256(report_path)}],
        "verifier_path": SCRIPT.as_posix(),
        "verifier_sha256": sha256(ROOT / SCRIPT),
        "claim_boundary": report["claim_boundary"],
    }
    (OUT / "rctx-manifest.json").write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
    )
    print(
        f"National RCTX verification: {len(rows)} States, "
        f"{report['unit_count']} blocks, {report['bridge_edge_count']} bridges"
    )


if __name__ == "__main__":
    main()
