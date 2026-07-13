#!/usr/bin/env python3
"""Build hostile certified-recursive tree fixtures and root hash manifest."""

from __future__ import annotations

import copy
import hashlib
import json
import shutil
from pathlib import Path


ROOT = Path("docs/examples/certified-recursive")
POSITIVE = ROOT / "path8-k4"
NEGATIVE = ROOT / "negative-corpus"


def canonical_hash(value: object) -> str:
    encoded = json.dumps(
        value, ensure_ascii=False, separators=(",", ":"), sort_keys=True
    ).encode("utf-8")
    return f"sha256:{hashlib.sha256(encoded).hexdigest()}"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def refresh_tree_id(tree: dict) -> None:
    tree["tree_id"] = canonical_hash(
        {
            "schema_version": tree["schema_version"],
            "root_unit_universe_hash": tree["root_unit_universe_hash"],
            "k": tree["k"],
            "nodes": tree["nodes"],
            "leaves": tree["leaves"],
        }
    )


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


def refresh_leaf_id(leaf: dict) -> None:
    leaf["leaf_id"] = canonical_hash(
        {
            "node_path": leaf["node_path"],
            "parent_certificate_id": leaf["parent_certificate_id"],
            "unit_universe_hash": leaf["unit_universe_hash"],
            "unit_ids": leaf["unit_ids"],
            "district_index": leaf["district_index"],
        }
    )


def write_case(name: str, tree: dict, expected_error: str, description: str) -> None:
    case = NEGATIVE / name
    case.mkdir(parents=True, exist_ok=True)
    (case / "certified-bisection-tree.json").write_text(
        json.dumps(tree, indent=2) + "\n", encoding="utf-8"
    )
    (case / "expected.json").write_text(
        json.dumps(
            {
                "schema_version": "certified-recursive-negative-fixture-v1",
                "case": name,
                "expected_error": expected_error,
                "description": description,
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )


def refresh_manifest() -> None:
    files = [
        {
            "path": path.relative_to(ROOT).as_posix(),
            "sha256": sha256(path),
        }
        for path in sorted(ROOT.rglob("*"))
        if path.is_file() and path.name != "manifest.json"
    ]
    manifest = {
        "schema_version": "certified-recursive-fixture-manifest-v1",
        "package_id": "certified-recursive-bounded-fixtures",
        "status": "active",
        "files": files,
        "builder_path": "scripts/research/build_certified_recursive_fixtures.py",
        "verifier_path": "crates/bisect-ilp/examples/certified_recursive.rs",
        "verification_commands": [
            "python scripts/research/verify_certified_recursive_fixtures.py",
            "cargo test -p bisect-ilp --test certified_recursive_negative_corpus -- --test-threads=1",
            "cargo run -p bisect-ilp --example certified_recursive -- verify-package docs/examples/certified-recursive/path8-k4/output",
        ],
        "claim_boundary": (
            "Bounded synthetic recursive certification only; not unrestricted-map "
            "optimality, production proof solving, or block-scale readiness."
        ),
    }
    (ROOT / "manifest.json").write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
    )


def main() -> None:
    if NEGATIVE.exists():
        shutil.rmtree(NEGATIVE)
    original = load(POSITIVE / "output/certified-bisection-tree.json")

    tree = copy.deepcopy(original)
    tree["tree_id"] = "sha256:" + ("0" * 64)
    write_case(
        "tree-id-tamper",
        tree,
        "tree-id-mismatch",
        "Changes the tree ID without changing canonical tree content.",
    )

    tree = copy.deepcopy(original)
    tree["leaves"].pop()
    refresh_tree_id(tree)
    write_case(
        "missing-leaf",
        tree,
        "leaf-set-mismatch",
        "Removes one certified district leaf and recomputes the outer tree ID.",
    )

    tree = copy.deepcopy(original)
    tree["nodes"][1], tree["nodes"][2] = tree["nodes"][2], tree["nodes"][1]
    refresh_tree_id(tree)
    write_case(
        "node-order-tamper",
        tree,
        "node-schedule-mismatch",
        "Reorders valid split nodes away from canonical BisectionTree BFS order.",
    )

    tree = copy.deepcopy(original)
    tree["leaves"][0]["unit_ids"][1] = tree["leaves"][1]["unit_ids"][0]
    tree["leaves"][0]["unit_universe_hash"] = canonical_hash(
        {"unit_ids": tree["leaves"][0]["unit_ids"]}
    )
    refresh_leaf_id(tree["leaves"][0])
    refresh_tree_id(tree)
    write_case(
        "leaf-universe-tamper",
        tree,
        "leaf-mismatch",
        "Substitutes a unit from another district while refreshing leaf and tree IDs.",
    )

    tree = copy.deepcopy(original)
    tree["nodes"][0]["certificate"]["result"]["objective"]["primary"][
        "weighted_boundary_cut"
    ] += 1
    refresh_certificate_id(tree["nodes"][0]["certificate"])
    refresh_tree_id(tree)
    write_case(
        "false-root-optimum",
        tree,
        "split-result-mismatch",
        "Changes the certified root objective and refreshes certificate and tree IDs.",
    )

    refresh_manifest()
    print("Certified recursive fixture build: PASS")


if __name__ == "__main__":
    main()
