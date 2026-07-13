#!/usr/bin/env python3
"""Build real RPLAN/RCTX baseline artifacts for the G.1-G.3 package."""

from __future__ import annotations

import argparse
import hashlib
import json
import pickle
from datetime import datetime, timezone
from pathlib import Path


def canonical_bytes(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--state", required=True)
    parser.add_argument("--year", type=int, default=2020)
    parser.add_argument("--geoids", type=Path, required=True)
    parser.add_argument("--adjacency", type=Path, required=True)
    parser.add_argument("--assignments", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()

    raw_geoids = json.loads(args.geoids.read_text(encoding="utf-8"))
    unit_ids = [str(raw_geoids[str(index)]).zfill(11) for index in range(len(raw_geoids))]
    graph_data = pickle.loads(args.adjacency.read_bytes())
    assignments_map = {
        int(index): int(district)
        for index, district in json.loads(args.assignments.read_text(encoding="utf-8")).items()
    }
    assignment = [assignments_map[index] - 1 for index in range(len(unit_ids))]
    k = max(assignment) + 1
    source_id = f"bisect-adjacency-{args.state.lower()}-{args.year}"
    universe_projection = {
        "unit_kind": "tract",
        "state": args.state.upper(),
        "year": args.year,
        "canonical_order": "explicit-unit-ids",
        "unit_ids": unit_ids,
        "source_id": source_id,
    }
    units = {
        **universe_projection,
        "unit_universe_hash": f"sha256:{sha256_bytes(canonical_bytes(universe_projection))}",
    }
    created_at = datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")
    source_hashes = {
        "adjacency-pickle": f"sha256:{sha256_file(args.adjacency)}",
        "geoid-index": f"sha256:{sha256_file(args.geoids)}",
        "baseline-assignments": f"sha256:{sha256_file(args.assignments)}",
    }
    plan = {
        "rplan_version": "0.2",
        "plan": {
            "schema_version": "district-plan-v1",
            "units": units,
            "assignment": assignment,
            "k": k,
            "display_labels": [str(index) for index in range(1, k + 1)],
            "allow_empty_districts": False,
        },
        "metadata": {
            "label": "nrs_reference_v0_1",
            "jurisdiction": args.state.upper(),
            "chamber": "congressional",
            "created_at": created_at,
            "description": "Tract-level NRS v0.1 geographic benchmark used to initialize real ensemble evidence.",
        },
        "provenance": {
            "producer": {
                "crate": "bisect-cli",
                "method": "standard-bisect-geographic-single",
                "method_crate": "bisect-runner",
            },
            "source_hashes": source_hashes,
            "conversion_lineage": [
                {
                    "workflow": "scripts/research/build_real_ensemble_contexts.py",
                    "claim_boundary": "tract-level research benchmark; not block-level NRS conformance",
                }
            ],
        },
        "geometry": None,
        "extensions": {},
    }
    adjacency = [
        [{"to": int(neighbor), "kind": "boundary"} for neighbor in neighbors]
        for neighbors in graph_data["adjacency"]
    ]
    context_projection = {
        "units": units,
        "graph": {"edge_semantics": "undirected", "adjacency": adjacency},
        "populations": [int(value) for value in graph_data["vertex_weights"]],
        "source_hashes": source_hashes,
    }
    context = {
        "rctx_version": "0.1",
        "context_hash": f"sha256:{sha256_bytes(canonical_bytes(context_projection))}",
        **context_projection,
    }
    args.output_dir.mkdir(parents=True, exist_ok=True)
    (args.output_dir / "baseline.rplan").write_text(
        json.dumps(plan, indent=2), encoding="utf-8"
    )
    (args.output_dir / "context.rctx").write_text(
        json.dumps(context, indent=2), encoding="utf-8"
    )


if __name__ == "__main__":
    main()
