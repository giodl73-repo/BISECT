#!/usr/bin/env python3
"""Extend an unresolved operational-tree seed frontier without provenance drift."""

from __future__ import annotations

import argparse
import importlib.util
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = Path("scripts/research/build_operational_recursive_tree_extended.py")
BASE_SCRIPT = Path("scripts/research/build_operational_recursive_tree.py")
SPEC = importlib.util.spec_from_file_location(
    "operational_recursive_tree_base", ROOT / BASE_SCRIPT
)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError("cannot load frozen operational-tree builder")
BASE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BASE)


def rewrite_manifest_provenance(out_dir: Path, max_seed: int) -> None:
    manifest_path = out_dir / "manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    manifest["builder_path"] = SCRIPT.as_posix()
    manifest["builder_sha256"] = BASE.sha256(ROOT / SCRIPT)
    manifest["base_builder_path"] = BASE_SCRIPT.as_posix()
    manifest["base_builder_sha256"] = BASE.sha256(ROOT / BASE_SCRIPT)
    manifest["seed_frontier_max"] = max_seed
    manifest_path.write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
    )


def build(
    bisect: Path,
    context_path: Path,
    out_dir: Path,
    districts: int,
    root_seed: int,
    child_seeds: tuple[int, int],
    max_seed: int,
) -> None:
    if max_seed < 16:
        raise ValueError("extended seed frontier must be at least 16")
    base_run_floor_discovery = BASE.run_floor_discovery

    def run_extended_floor_discovery(
        bisect_path: Path,
        node_context_path: Path,
        node_districts: int,
        node_out_dir: Path,
        preferred_seed: int,
        population_floor: int,
        base_max_seed: int,
    ) -> tuple[dict, int, list[dict]]:
        return base_run_floor_discovery(
            bisect_path,
            node_context_path,
            node_districts,
            node_out_dir,
            preferred_seed,
            population_floor,
            max(base_max_seed, max_seed),
        )

    BASE.run_floor_discovery = run_extended_floor_discovery
    try:
        BASE.build(
            bisect,
            context_path,
            out_dir,
            districts,
            root_seed,
            child_seeds,
        )
    finally:
        BASE.run_floor_discovery = base_run_floor_discovery
    rewrite_manifest_provenance(out_dir, max_seed)


def main() -> None:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)
    build_parser = subparsers.add_parser("build")
    build_parser.add_argument("--bisect", type=Path, required=True)
    build_parser.add_argument("--context", type=Path, required=True)
    build_parser.add_argument("--out-dir", type=Path, required=True)
    build_parser.add_argument("--districts", type=int, required=True)
    build_parser.add_argument("--root-seed", type=int, default=1)
    build_parser.add_argument("--child-seed-0", type=int, default=2)
    build_parser.add_argument("--child-seed-1", type=int, default=3)
    build_parser.add_argument("--max-seed", type=int, default=32)
    verify_parser = subparsers.add_parser("verify")
    verify_parser.add_argument("package", type=Path)
    args = parser.parse_args()
    if args.command == "build":
        build(
            args.bisect.resolve(),
            args.context.resolve(),
            args.out_dir.resolve(),
            args.districts,
            args.root_seed,
            (args.child_seed_0, args.child_seed_1),
            args.max_seed,
        )
    else:
        BASE.verify(args.package.resolve())


if __name__ == "__main__":
    main()
