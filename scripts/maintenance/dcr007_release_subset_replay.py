#!/usr/bin/env python3
"""Capture a DCR-007 release-subset replay evidence record."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


DATA_PREFIXES = ("data/", "data\\")


def run_command(command: list[str], cwd: Path) -> dict[str, object]:
    started = datetime.now(timezone.utc)
    completed = subprocess.run(
        command,
        cwd=cwd,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    ended = datetime.now(timezone.utc)
    return {
        "command": command,
        "exit_code": completed.returncode,
        "started_at": started.isoformat(),
        "ended_at": ended.isoformat(),
        "stdout": completed.stdout,
        "stderr": completed.stderr,
    }


def git_lines(repo: Path, *args: str) -> list[str]:
    result = subprocess.run(
        ["git", *args],
        cwd=repo,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(result.stderr.strip() or f"git {' '.join(args)} failed")
    return [line for line in result.stdout.splitlines() if line.strip()]


def file_sha256(path: Path) -> str | None:
    if not path.is_file():
        return None
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def hash_tree(path: Path, repo: Path) -> list[dict[str, str]]:
    if not path.exists():
        return []
    hashes: list[dict[str, str]] = []
    for child in sorted(p for p in path.rglob("*") if p.is_file()):
        digest = file_sha256(child)
        if digest is not None:
            hashes.append(
                {
                    "path": child.relative_to(repo).as_posix(),
                    "sha256": digest,
                }
            )
    return hashes


def source_status(repo: Path) -> list[str]:
    return git_lines(repo, "status", "--porcelain=v1", "--untracked-files=all")


def status_path(entry: str) -> str:
    return entry[3:].strip()


def non_data_status_entries(entries: list[str]) -> list[str]:
    return [
        entry
        for entry in entries
        if not status_path(entry).startswith(DATA_PREFIXES)
    ]


def default_binary_path(repo: Path) -> Path:
    executable = "bisect.exe" if os.name == "nt" else "bisect"
    return repo / "target" / "debug" / executable


def command_line(command: list[str]) -> str:
    return " ".join(command)


def build_replay_commands(args: argparse.Namespace, binary: Path) -> list[list[str]]:
    commands: list[list[str]] = []
    if not args.no_cargo_build:
        commands.append(["cargo", "build", "-p", "bisect-cli", "--bin", "bisect"])

    base = [str(binary)]
    build_command = [
        *base,
        "build",
        args.label,
        "--year",
        str(args.year),
        "--states",
        args.states,
        "--workers",
        str(args.workers),
        "--no-interactive",
    ]
    if args.force:
        build_command.append("--force")
    commands.append(build_command)
    commands.append(
        [
            *base,
            "label-analyze",
            args.label,
            "--year",
            str(args.year),
            "--types",
            args.analysis_types,
        ]
    )
    commands.append(
        [
            *base,
            "label-report",
            args.label,
            "--year",
            str(args.year),
            "--format",
            *args.report_format,
        ]
    )
    commands.append(
        [
            *base,
            "label-verify",
            args.label,
            "--year",
            str(args.year),
        ]
    )
    return commands


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Run or preflight the DCR-007 release-subset reproducibility replay "
            "and write a JSON evidence record."
        )
    )
    parser.add_argument("--label", default="official_proposal")
    parser.add_argument("--year", default="2020")
    parser.add_argument("--states", default="VT")
    parser.add_argument("--workers", type=int, default=1)
    parser.add_argument("--analysis-types", default="all")
    parser.add_argument("--report-format", nargs="+", default=["html"])
    parser.add_argument("--binary", type=Path)
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("reports/vtrace/dcr007_release_subset_replay.json"),
    )
    parser.add_argument(
        "--force",
        action=argparse.BooleanOptionalAction,
        default=True,
        help="pass --force to the build step",
    )
    parser.add_argument(
        "--no-cargo-build",
        action="store_true",
        help="use the existing bisect binary instead of rebuilding it first",
    )
    parser.add_argument(
        "--preflight-only",
        action="store_true",
        help="record environment and planned commands without running the replay",
    )
    parser.add_argument(
        "--allow-dirty-data",
        action="store_true",
        help=(
            "allow data/ changes in the clean-source preflight. This is not an "
            "L2 clean replay claim."
        ),
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    repo = Path(
        subprocess.run(
            ["git", "rev-parse", "--show-toplevel"],
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=True,
        ).stdout.strip()
    )
    binary = (args.binary or default_binary_path(repo)).resolve()
    output = (repo / args.output).resolve()
    config = repo / "configs" / f"{args.label}.yml"
    data_manifest = repo / "data" / "manifest.json"
    status_entries = source_status(repo)
    blocking_status = (
        non_data_status_entries(status_entries)
        if args.allow_dirty_data
        else status_entries
    )
    commands = build_replay_commands(args, binary)

    record: dict[str, object] = {
        "schema": "BISECT-DCR007-REPLAY-v1",
        "created_at": datetime.now(timezone.utc).isoformat(),
        "scope": {
            "class": "release-subset",
            "label": args.label,
            "year": str(args.year),
            "states": args.states,
            "workers": args.workers,
            "analysis_types": args.analysis_types,
            "report_format": args.report_format,
        },
        "environment": {
            "os": platform.platform(),
            "python": platform.python_version(),
            "machine": platform.machine(),
            "processor": platform.processor(),
            "rustc": shutil.which("rustc"),
            "cargo": shutil.which("cargo"),
        },
        "source": {
            "commit": git_lines(repo, "rev-parse", "HEAD")[0],
            "status": status_entries,
            "status_policy": (
                "data_changes_allowed_for_preflight_only"
                if args.allow_dirty_data
                else "fully_clean_required"
            ),
            "clean_for_replay": not blocking_status,
            "blocking_status": blocking_status,
        },
        "inputs": {
            "config_path": config.relative_to(repo).as_posix(),
            "config_sha256": file_sha256(config),
            "data_manifest_path": data_manifest.relative_to(repo).as_posix(),
            "data_manifest_sha256": file_sha256(data_manifest),
        },
        "binary": {
            "path": str(binary),
            "sha256_before": file_sha256(binary),
        },
        "planned_commands": [command_line(command) for command in commands],
        "preflight_only": args.preflight_only,
        "commands": [],
        "artifact_hashes": [],
        "result": "preflight_pass",
    }

    output.parent.mkdir(parents=True, exist_ok=True)

    if blocking_status:
        record["result"] = "blocked_dirty_source"
        output.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
        print(f"blocked: dirty source entries prevent clean replay; wrote {output}")
        return 2

    if not args.preflight_only:
        command_results = []
        for command in commands:
            result = run_command(command, repo)
            command_results.append(result)
            if result["exit_code"] != 0:
                record["result"] = "command_failed"
                break
        record["commands"] = command_results
        record["binary"]["sha256_after"] = file_sha256(binary)  # type: ignore[index]

        artifact_roots = [
            repo / "runs" / args.label / str(args.year),
            repo / "analysis" / args.label / str(args.year),
            repo / "reports" / args.label / str(args.year),
        ]
        artifact_hashes: list[dict[str, str]] = []
        for root in artifact_roots:
            artifact_hashes.extend(hash_tree(root, repo))
        record["artifact_hashes"] = artifact_hashes
        if record["result"] == "preflight_pass":
            record["result"] = "replay_pass"

    output.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    print(f"wrote {output}")
    return 0 if record["result"] in {"preflight_pass", "replay_pass"} else 1


if __name__ == "__main__":
    sys.exit(main())
