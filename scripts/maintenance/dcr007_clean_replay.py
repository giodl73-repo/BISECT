#!/usr/bin/env python3
"""Strict launcher for a DCR-007 clean replay evidence run."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path


def git_output(repo: Path | None, *args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["git", *args],
        cwd=repo,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )


def repo_root() -> Path:
    result = git_output(None, "rev-parse", "--show-toplevel")
    if result.returncode != 0:
        raise RuntimeError(result.stderr.strip() or "not inside a git repository")
    return Path(result.stdout.strip())


def dirty_status(repo: Path) -> list[str]:
    result = git_output(repo, "status", "--porcelain=v1", "--untracked-files=all")
    if result.returncode != 0:
        raise RuntimeError(result.stderr.strip() or "git status failed")
    return [line for line in result.stdout.splitlines() if line.strip()]


def main(argv: list[str]) -> int:
    if "--allow-dirty-data" in argv:
        print(
            "blocked: dcr007_clean_replay.py never permits --allow-dirty-data; "
            "use dcr007_release_subset_replay.py for candidate runs",
            file=sys.stderr,
        )
        return 2

    try:
        repo = repo_root()
        status_entries = dirty_status(repo)
    except RuntimeError as error:
        print(f"blocked: {error}", file=sys.stderr)
        return 2

    if status_entries:
        print(
            "blocked: clean DCR-007 replay requires empty git status --short",
            file=sys.stderr,
        )
        for entry in status_entries:
            print(entry, file=sys.stderr)
        return 2

    harness = repo / "scripts" / "maintenance" / "dcr007_release_subset_replay.py"
    command = [sys.executable, str(harness), *argv]
    completed = subprocess.run(command, cwd=repo, check=False)
    return completed.returncode


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
