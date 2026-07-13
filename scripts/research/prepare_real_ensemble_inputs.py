#!/usr/bin/env python3
"""Prepare hash-bound 2016/2020 election arrays for real ensemble traces."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

import pandas as pd


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load_votes(path: Path, state_fips: str, dem_col: str, rep_col: str) -> dict[str, tuple[float, float]]:
    frame = pd.read_csv(
        path,
        usecols=["tract_GEOID", "tract_state_fp", dem_col, rep_col],
        dtype={"tract_GEOID": str, "tract_state_fp": str},
        low_memory=False,
    )
    frame["tract_state_fp"] = frame["tract_state_fp"].str.zfill(2)
    frame = frame[frame["tract_state_fp"] == state_fips]
    frame[dem_col] = pd.to_numeric(frame[dem_col], errors="coerce").fillna(0.0)
    frame[rep_col] = pd.to_numeric(frame[rep_col], errors="coerce").fillna(0.0)
    grouped = frame.groupby("tract_GEOID", as_index=False)[[dem_col, rep_col]].sum()
    return {
        str(row.tract_GEOID).zfill(11): (float(getattr(row, dem_col)), float(getattr(row, rep_col)))
        for row in grouped.itertuples(index=False)
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--state", required=True)
    parser.add_argument("--state-fips", required=True)
    parser.add_argument("--geoids", type=Path, required=True)
    parser.add_argument("--election-2016", type=Path, required=True)
    parser.add_argument("--election-2020", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    raw_geoids = json.loads(args.geoids.read_text(encoding="utf-8"))
    count = len(raw_geoids)
    geoids = [str(raw_geoids[str(index)]).zfill(11) for index in range(count)]
    votes_2016 = load_votes(args.election_2016, args.state_fips, "G16PREDCLI", "G16PRERTRU")
    votes_2020 = load_votes(args.election_2020, args.state_fips, "G20PREDBID", "G20PRERTRU")

    missing_2016 = [geoid for geoid in geoids if geoid not in votes_2016]
    missing_2020 = [geoid for geoid in geoids if geoid not in votes_2020]
    output = {
        "election_input_version": "g-real-election-input v1",
        "state": args.state.upper(),
        "year": 2020,
        "geoid_count": count,
        "sources": {
            "2016": {
                "path": args.election_2016.as_posix(),
                "sha256": sha256(args.election_2016),
                "democratic_column": "G16PREDCLI",
                "republican_column": "G16PRERTRU",
            },
            "2020": {
                "path": args.election_2020.as_posix(),
                "sha256": sha256(args.election_2020),
                "democratic_column": "G20PREDBID",
                "republican_column": "G20PRERTRU",
            },
            "geoids": {
                "path": args.geoids.as_posix(),
                "sha256": sha256(args.geoids),
            },
        },
        "unmatched_geoids": {
            "2016": len(missing_2016),
            "2020": len(missing_2020),
        },
        "democratic_2016": [votes_2016.get(geoid, (0.0, 0.0))[0] for geoid in geoids],
        "republican_2016": [votes_2016.get(geoid, (0.0, 0.0))[1] for geoid in geoids],
        "democratic_2020": [votes_2020.get(geoid, (0.0, 0.0))[0] for geoid in geoids],
        "republican_2020": [votes_2020.get(geoid, (0.0, 0.0))[1] for geoid in geoids],
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(output, separators=(",", ":")), encoding="utf-8")


if __name__ == "__main__":
    main()
