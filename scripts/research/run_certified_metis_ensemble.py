#!/usr/bin/env python3
"""Screen deterministic METIS seeds before expensive exact refinement."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCHEMA = "certified-metis-ensemble-v1"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def objective_key(row: dict[str, object]) -> tuple[int, int, int, int]:
    if row["status"] != "accepted":
        return (1, 2**63 - 1, 2**63 - 1, 2**63 - 1)
    objective = row["objective"]
    assert isinstance(objective, dict)
    return (
        0,
        int(objective["max_population_deviation_scaled"]),
        int(objective["total_population_deviation_scaled"]),
        int(objective["weighted_boundary_cut"]),
    )


def run_seed(
    bisect: Path,
    context: Path,
    out_root: Path,
    districts: int,
    seed: int,
    refinement: str,
) -> dict[str, object]:
    seed_dir = out_root / f"seed-{seed:04d}"
    seed_dir.mkdir(parents=True, exist_ok=True)
    completed = subprocess.run(
        [
            str(bisect),
            "exact",
            "--context",
            str(context),
            "--districts",
            str(districts),
            "--method",
            "certified-discovery",
            "--out-dir",
            str(seed_dir),
            "--generated-at",
            "2026-07-10T00:00:00Z",
            "--discovery-seed",
            str(seed),
            "--discovery-refinement",
            refinement,
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    (seed_dir / "command.log").write_text(
        completed.stdout + completed.stderr, encoding="utf-8"
    )
    if completed.returncode != 0:
        return {
            "seed": seed,
            "status": "rejected",
            "exit_code": completed.returncode,
            "reason": (completed.stdout + completed.stderr).strip().splitlines()[-1],
        }
    discovery_path = seed_dir / "certified-discovery.json"
    discovery = json.loads(discovery_path.read_text(encoding="utf-8"))
    return {
        "seed": seed,
        "status": "accepted",
        "discovery_sha256": sha256(discovery_path),
        "discovery_id": discovery["discovery_id"],
        "objective": discovery["objective"]["primary"],
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--bisect", type=Path, required=True)
    parser.add_argument("--context", type=Path, required=True)
    parser.add_argument("--out-dir", type=Path, required=True)
    parser.add_argument("--districts", type=int, default=2)
    parser.add_argument("--seed-start", type=int, default=1)
    parser.add_argument("--seed-end", type=int, default=16)
    parser.add_argument(
        "--refinement",
        choices=("metis", "population", "fast", "full"),
        default="population",
    )
    parser.add_argument("--workers", type=int, default=2)
    args = parser.parse_args()

    bisect = args.bisect.resolve()
    context = args.context.resolve()
    out_dir = args.out_dir.resolve()
    out_dir.mkdir(parents=True, exist_ok=True)
    seeds = list(range(args.seed_start, args.seed_end + 1))
    rows: list[dict[str, object]] = []
    with ThreadPoolExecutor(max_workers=args.workers) as executor:
        futures = {
            executor.submit(
                run_seed,
                bisect,
                context,
                out_dir,
                args.districts,
                seed,
                args.refinement,
            ): seed
            for seed in seeds
        }
        for future in as_completed(futures):
            row = future.result()
            rows.append(row)
            print(f"seed {row['seed']}: {row['status']}")
    rows.sort(key=lambda row: int(row["seed"]))
    ranked = sorted(rows, key=objective_key)
    accepted = [row for row in ranked if row["status"] == "accepted"]
    report = {
        "schema_version": SCHEMA,
        "context_sha256": sha256(context),
        "districts": args.districts,
        "refinement": args.refinement,
        "seed_start": args.seed_start,
        "seed_end": args.seed_end,
        "accepted_count": len(accepted),
        "rejected_count": len(rows) - len(accepted),
        "best_seed": accepted[0]["seed"] if accepted else None,
        "best_objective": accepted[0]["objective"] if accepted else None,
        "results": rows,
        "claim_boundary": (
            "Deterministic METIS screening with exact post-validation; not an "
            "optimality proof or permission to contract block units."
        ),
    }
    (out_dir / "ensemble.json").write_text(
        json.dumps(report, indent=2) + "\n", encoding="utf-8"
    )
    print(
        f"Certified METIS ensemble: {len(accepted)} accepted, "
        f"{len(rows) - len(accepted)} rejected, best seed {report['best_seed']}"
    )


if __name__ == "__main__":
    main()
