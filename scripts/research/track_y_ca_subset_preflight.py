#!/usr/bin/env python3
"""Track Y California subset preflight.

This does not run BISECT. It verifies that the local checkout has enough
California tract graph material to start Pulse 05 and prints the exact next
commands when it does not.
"""

from __future__ import annotations

import argparse
import contextlib
import io
import json
import os
import struct
import sys
import tempfile
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Check prerequisites for Track Y CA subset research runs."
    )
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument("--year", type=int, default=2020)
    parser.add_argument("--state", default="CA")
    parser.add_argument("--adjacency-override", type=Path)
    parser.add_argument("--write-json", type=Path)
    parser.add_argument(
        "--allow-blocked",
        action="store_true",
        help="Return success after writing a blocked diagnostic preflight.",
    )
    parser.add_argument("--self-test", action="store_true")
    return parser.parse_args()


def load_manifest(repo_root: Path) -> dict[str, object]:
    manifest_path = Path(os.environ["BISECT_MANIFEST"]) if "BISECT_MANIFEST" in os.environ else None
    if manifest_path is None:
        manifest_path = repo_root / "data" / "manifest.json"
    if not manifest_path.is_absolute():
        manifest_path = repo_root / manifest_path
    with manifest_path.open("r", encoding="utf-8") as fh:
        manifest = json.load(fh)
    manifest["_source_path"] = str(manifest_path)
    return manifest


def candidate_adj_paths(
    repo_root: Path,
    manifest: dict[str, object],
    year: int,
    state: str,
) -> list[Path]:
    lower = state.lower()
    upper = state.upper()
    local_outputs_dir = Path(str(manifest.get("local_outputs_dir", "outputs")))
    if not local_outputs_dir.is_absolute():
        local_outputs_dir = repo_root / local_outputs_dir
    local_data_dir = Path(str(manifest.get("local_data_dir", "data")))
    if not local_data_dir.is_absolute():
        local_data_dir = repo_root / local_data_dir
    return [
        local_outputs_dir / "data" / str(year) / "adjacency" / f"{lower}.adj.bin",
        local_outputs_dir
        / "data"
        / str(year)
        / "adjacency"
        / f"{upper}.adj.bin",
        local_outputs_dir
        / "data"
        / str(year)
        / "adjacency"
        / f"{lower}_adjacency_{year}.adj.bin",
        local_outputs_dir
        / "data"
        / str(year)
        / "adjacency"
        / f"{upper}_adjacency_{year}.adj.bin",
        local_data_dir / str(year) / "adjacency" / f"{lower}.adj.bin",
        local_data_dir / str(year) / "adjacency" / f"{upper}.adj.bin",
        local_outputs_dir
        / "V3"
        / "data"
        / str(year)
        / "adjacency"
        / f"{lower}_adjacency_{year}.pkl",
        local_outputs_dir
        / "V3"
        / "data"
        / str(year)
        / "adjacency"
        / f"{upper}_adjacency_{year}.pkl",
    ]


def read_adj_header(path: Path) -> dict[str, int | str]:
    with path.open("rb") as fh:
        header = fh.read(16)
    if len(header) != 16:
        raise ValueError(f"{path} is shorter than a RADJ header")
    magic, version, n_vertices, n_edges = struct.unpack("<4sIII", header)
    if magic != b"RADJ":
        raise ValueError(f"{path} has magic {magic!r}, expected b'RADJ'")
    if version != 2:
        raise ValueError(f"{path} has RADJ version {version}, expected 2")
    return {
        "path": str(path),
        "version": version,
        "n_vertices": n_vertices,
        "n_edges": n_edges,
    }


def build_preflight_result(
    repo_root: Path,
    year: int,
    state: str,
    write_json: Path | None,
    adjacency_override: Path | None = None,
) -> dict[str, object]:
    manifest = load_manifest(repo_root)
    releases = manifest.get("releases", {})
    data_inputs_release = (
        releases.get("data_inputs") if isinstance(releases, dict) else None
    )
    github_repo = str(manifest.get("github_repo", ""))
    candidates = candidate_adj_paths(repo_root, manifest, year, state)
    if adjacency_override is not None:
        override = adjacency_override
        if not override.is_absolute():
            override = repo_root / override
        candidates = [override] + candidates
    existing = [path for path in candidates if path.exists()]
    previous: dict[str, object] = {}
    if write_json and write_json.exists():
        try:
            previous = json.loads(write_json.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            previous = {}

    result: dict[str, object] = {
        "schema": "bisect.track_y.ca_subset_preflight.v1",
        "state": state.upper(),
        "year": year,
        "repo_root": str(repo_root),
        "manifest": {
            "source_path": manifest["_source_path"],
            "github_repo": github_repo,
            "data_inputs_release": data_inputs_release,
            "local_data_dir": manifest.get("local_data_dir"),
            "local_outputs_dir": manifest.get("local_outputs_dir"),
        },
        "candidate_paths": [str(path) for path in candidates],
        "adjacency_override": str(adjacency_override) if adjacency_override else None,
        "ready": False,
        "adjacency": None,
        "release_check_command": (
            f"gh release view {data_inputs_release} --repo {github_repo}"
            if github_repo and data_inputs_release
            else None
        ),
        "next_command": (
            f"cargo run -p bisect-cli -- fetch --year {year} "
            f"--states {state.upper()} --type adjacency --release"
        ),
    }
    if "fetch_attempt" in previous:
        result["fetch_attempt"] = previous["fetch_attempt"]

    if existing:
        try:
            if existing[0].suffix == ".pkl":
                result["adjacency"] = {
                    "path": str(existing[0]),
                    "format": "pkl",
                    "note": "usable fallback; RADJ .adj.bin remains preferred",
                }
            else:
                result["adjacency"] = read_adj_header(existing[0])
            result["ready"] = True
            result["next_step"] = "build_subset_fixture"
        except ValueError as exc:
            result["error"] = str(exc)
            result["next_step"] = "repair_or_replace_adjacency_artifact"
    else:
        result["error"] = "No California adjacency artifact found."
        result["next_step"] = "fetch_or_supply_adjacency_artifact"

    return result


def self_test() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        repo_root = Path(tmp)
        manifest_dir = repo_root / "data"
        manifest_dir.mkdir(parents=True)
        (manifest_dir / "manifest.json").write_text(
            json.dumps(
                {
                    "version": "1",
                    "github_repo": "example/repo",
                    "releases": {"data_inputs": "data-inputs-test"},
                    "local_data_dir": "data",
                    "local_outputs_dir": "outputs",
                    "states": {"CA": {"name": "California"}},
                }
            ),
            encoding="utf-8",
        )

        result = build_preflight_result(repo_root, 2020, "CA", None)
        assert result["ready"] is False
        assert result["manifest"]["github_repo"] == "example/repo"
        assert result["manifest"]["data_inputs_release"] == "data-inputs-test"
        assert result["release_check_command"] == (
            "gh release view data-inputs-test --repo example/repo"
        )
        assert result["next_command"] == (
            "cargo run -p bisect-cli -- fetch --year 2020 "
            "--states CA --type adjacency --release"
        )
        assert result["next_step"] == "fetch_or_supply_adjacency_artifact"
        assert len(result["candidate_paths"]) == 8
        assert result["error"] == "No California adjacency artifact found."

        override = repo_root / "local" / "ca.adj.bin"
        override.parent.mkdir()
        payload = bytearray()
        payload.extend(struct.pack("<4sIII", b"RADJ", 2, 2, 1))
        payload.extend(struct.pack("<q", 100))
        payload.extend(struct.pack("<q", 200))
        payload.extend(struct.pack("<I", 1))
        payload.extend(struct.pack("<I", 1))
        payload.extend(struct.pack("<I", 1))
        payload.extend(struct.pack("<I", 0))
        payload.extend(struct.pack("<I", 1))
        payload.extend(struct.pack("<IId", 0, 1, 3.0))
        override.write_bytes(payload)

        ready = build_preflight_result(repo_root, 2020, "CA", None, override)
        assert ready["ready"] is True
        assert ready["adjacency"]["path"] == str(override)
        assert ready["adjacency"]["n_vertices"] == 2
        assert ready["adjacency_override"] == str(override)
        assert ready["next_step"] == "build_subset_fixture"

        original_argv = sys.argv
        try:
            out_path = repo_root / "preflight.json"
            sys.argv = [
                "track_y_ca_subset_preflight.py",
                "--repo-root",
                str(repo_root),
                "--write-json",
                str(out_path),
                "--allow-blocked",
            ]
            with contextlib.redirect_stdout(io.StringIO()):
                assert main() == 0
            written = json.loads(out_path.read_text(encoding="utf-8"))
            assert written["ready"] is False
            assert written["next_step"] == "fetch_or_supply_adjacency_artifact"
        finally:
            sys.argv = original_argv


def main() -> int:
    args = parse_args()
    if args.self_test:
        self_test()
        print("[OK] track_y_ca_subset_preflight self-test passed")
        return 0

    repo_root = args.repo_root.resolve()
    result = build_preflight_result(
        repo_root,
        args.year,
        args.state,
        args.write_json,
        args.adjacency_override,
    )
    text = json.dumps(result, indent=2, sort_keys=True)
    print(text)
    if args.write_json:
        args.write_json.parent.mkdir(parents=True, exist_ok=True)
        args.write_json.write_text(text + "\n", encoding="utf-8")

    return 0 if result["ready"] or args.allow_blocked else 2


if __name__ == "__main__":
    raise SystemExit(main())
