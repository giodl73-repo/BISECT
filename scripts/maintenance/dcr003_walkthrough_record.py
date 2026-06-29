#!/usr/bin/env python3
"""Create a DCR-003 external walkthrough observation record template."""

from __future__ import annotations

import argparse
import platform
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


DEFAULT_QUICKSTART = "docs/quickstart/quickstart-special-master.md"
DEFAULT_OUTPUT = "reports/vtrace/dcr003_external_walkthrough_record.txt"


def run_text(repo: Path, *args: str) -> str:
    result = subprocess.run(
        [*args],
        cwd=repo,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(result.stderr.strip() or f"{' '.join(args)} failed")
    return result.stdout.strip()


def repo_root() -> Path:
    result = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(result.stderr.strip() or "not inside a git repository")
    return Path(result.stdout.strip())


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Write a prefilled DCR-003 external walkthrough record. The output is "
            "a template for a real non-author run, not L2 evidence by itself."
        )
    )
    parser.add_argument("--reviewer-role", default="special master")
    parser.add_argument(
        "--reviewer-independence",
        default="non-author confirmation pending",
        help="statement of whether the reviewer authored implementation/docs under review",
    )
    parser.add_argument("--observer", default="observer pending")
    parser.add_argument("--quickstart", default=DEFAULT_QUICKSTART)
    parser.add_argument(
        "--workflow",
        default="read-only verification or documented blocker path",
    )
    parser.add_argument("--label", default="official_proposal")
    parser.add_argument("--year", default="2020")
    parser.add_argument("--states", default="VT")
    parser.add_argument(
        "--data-scope",
        default="real data if provisioned; otherwise document blocker",
    )
    parser.add_argument("--output", type=Path, default=Path(DEFAULT_OUTPUT))
    parser.add_argument(
        "--stdout",
        action="store_true",
        help="print the record instead of writing --output",
    )
    return parser.parse_args()


def render_record(args: argparse.Namespace, repo: Path) -> str:
    quickstart = Path(args.quickstart)
    quickstart_path = repo / quickstart
    if not quickstart_path.is_file():
        raise RuntimeError(f"selected quickstart does not exist: {args.quickstart}")

    commit = run_text(repo, "git", "rev-parse", "HEAD")
    status = run_text(repo, "git", "status", "--short")
    status_text = status if status else "clean"
    environment = (
        f"{platform.platform()}; Python {platform.python_version()}; "
        f"machine={platform.machine() or 'unknown'}"
    )
    data_config_scope = (
        f"label={args.label}; year={args.year}; states={args.states}; "
        f"data={args.data_scope}"
    )

    return f"""DCR-003 external walkthrough record

Date: {datetime.now(timezone.utc).isoformat()}
Observer: {args.observer}
Reviewer role: {args.reviewer_role}
Reviewer independence: {args.reviewer_independence}
Repo commit: {commit}
Working tree status at template creation: {status_text}
Environment: {environment}
Selected quickstart: {quickstart.as_posix()}
Selected workflow: {args.workflow}
Data/config scope: {data_config_scope}

Task results:
1. README claim posture understood? pass / partial / fail
   Notes:
2. First command discoverable? pass / partial / fail
   Notes:
3. Expected outputs discoverable? pass / partial / fail
   Notes:
4. Workflow executed or blocked? pass / blocked / not-run
   Commands or blocker:
5. Failure modes understandable? pass / partial / fail
   Notes:
6. Legal/certification boundary understood? pass / partial / fail
   Notes:
7. Evidence/legal handoff docs found? pass / partial / fail
   Notes:

Friction items:
- Item:
  Class: doc fix / command fix / accepted limitation / environment blocker
  Disposition:

Reviewer summary:

Observer disposition:
pass_l2_candidate / blocked / needs_fixes

Record use:
This template is not DCR-003 L2 evidence until a real non-author operator
completes it and COMMONS/operator-review dispositions every friction item.
"""


def main() -> int:
    args = parse_args()
    try:
        repo = repo_root()
        record = render_record(args, repo)
    except RuntimeError as error:
        print(f"blocked: {error}", file=sys.stderr)
        return 2

    if args.stdout:
        print(record, end="")
        return 0

    output = (repo / args.output).resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(record, encoding="utf-8")
    print(f"wrote {output}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
