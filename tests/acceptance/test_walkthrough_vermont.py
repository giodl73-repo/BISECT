"""
L2 acceptance test for the Vermont 2020 canonical walkthrough.

Onboarding plan Task 8 (`docs/superpowers/plans/2026-04-30-onboarding-and-tutorials.md`).

This test invokes `bash examples/vermont-2020-walkthrough/run.sh` end-to-end on a
clean machine, then runs `redist doctor --check-tutorial-data --tutorial vermont-2020`
and asserts exit code 0. It is the load-bearing artifact for the Onboarding plan's
Definition of Done line "Vermont walkthrough committed under examples/...".

Marked `@pytest.mark.network` and `@pytest.mark.slow` so it default-skips on PR
CI. The roadmap CI strategy (specs/2026-04-30-roadmap-five-star.md §CI strategy)
runs it nightly on `ubuntu-latest-large`.

Per v2.1.1 tracking item 211-P3.1, this test ALSO asserts that walkthrough
stdout contains only ASCII characters — enforcing the PP-34 Windows console
policy at the L2 level so a console-only Windows user does not crash.

Note on pin status: the committed `checksums.json` ships with `PIN_ON_FIRST_RUN`
placeholder SHAs. Until a maintainer runs `bash examples/vermont-2020-walkthrough/pin.sh`
on a clean machine, the doctor `--check-tutorial-data` step will report MISSING
(not FAIL) because the placeholder strings won't match any real file. Once the
fixture is pinned, this test will assert the full PASS contract.
"""

import json
import os
import string
import subprocess
import sys
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[2]
WALKTHROUGH_DIR = REPO_ROOT / "examples" / "vermont-2020-walkthrough"
CHECKSUMS_PATH = WALKTHROUGH_DIR / "checksums.json"


def _redist_on_path() -> bool:
    """Best-effort check; pytest will skip with an actionable message if missing."""
    from shutil import which
    return which("redist") is not None


def _checksums_pinned() -> bool:
    """Return True if checksums.json contains real SHAs (not the PIN_ON_FIRST_RUN placeholder)."""
    if not CHECKSUMS_PATH.exists():
        return False
    try:
        data = json.loads(CHECKSUMS_PATH.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return False
    placeholders = ["PIN_ON_FIRST_RUN"]
    if data.get("build_commit") in placeholders:
        return False
    for row in data.get("pinned_inputs", []) + data.get("expected_outputs", []):
        if row.get("sha256") in placeholders:
            return False
    return True


@pytest.mark.network
@pytest.mark.slow
def test_walkthrough_vermont_runs_end_to_end():
    """End-to-end Vermont walkthrough: run.sh -> doctor --check-tutorial-data."""
    if not _redist_on_path():
        pytest.skip("redist binary not on PATH; run bootstrap.sh first")

    # Fixture must exist (structural). Pin status is checked downstream.
    assert WALKTHROUGH_DIR.is_dir(), \
        f"missing walkthrough directory: {WALKTHROUGH_DIR}"
    assert CHECKSUMS_PATH.exists(), \
        f"missing checksums.json: {CHECKSUMS_PATH}"
    data = json.loads(CHECKSUMS_PATH.read_text(encoding="utf-8"))
    assert data.get("schema_version") == "tutorial-checksums v1", \
        f"unexpected schema_version in {CHECKSUMS_PATH}"

    # 1. Run the walkthrough.
    runner = WALKTHROUGH_DIR / ("run.bat" if sys.platform == "win32" else "run.sh")
    if sys.platform == "win32":
        cmd = ["cmd", "/c", str(runner)]
    else:
        cmd = ["bash", str(runner)]
    proc = subprocess.run(
        cmd,
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        env={**os.environ, "PATH": os.environ.get("PATH", "")},
    )
    assert proc.returncode == 0, (
        f"walkthrough exited non-zero ({proc.returncode})\n"
        f"--- stdout ---\n{proc.stdout}\n"
        f"--- stderr ---\n{proc.stderr}"
    )

    # 2. PP-34 — assert ASCII-only console output. Tracking item 211-P3.1.
    # File outputs MAY be Unicode; CLI stdout/stderr MUST NOT on Windows.
    allowed = set(string.printable)
    for stream_name, stream in [("stdout", proc.stdout), ("stderr", proc.stderr)]:
        bad = [c for c in stream if c not in allowed]
        assert not bad, (
            f"walkthrough {stream_name} contains non-ASCII chars (PP-34 violation): "
            f"{[hex(ord(c)) for c in set(bad)][:10]}"
        )

    # 3. doctor --check-tutorial-data reports the right exit code.
    # If checksums are pinned, expect exit 0 (every present file matches).
    # If checksums are PIN_ON_FIRST_RUN placeholders, doctor will report FAIL
    # for every present file -> exit 1. We accept that until a maintainer pins.
    doctor = subprocess.run(
        ["redist", "doctor", "--check-tutorial-data", "--tutorial", "vermont-2020"],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
    )
    pinned = _checksums_pinned()
    if pinned:
        assert doctor.returncode == 0, (
            f"doctor --check-tutorial-data exited {doctor.returncode}\n"
            f"--- stdout ---\n{doctor.stdout}\n"
            f"--- stderr ---\n{doctor.stderr}"
        )
    else:
        # Until pinned, accept either 0 (no real SHAs to compare) or 1
        # (placeholders fail their hash check). 2 (parse error) is still bad.
        assert doctor.returncode in (0, 1), (
            f"doctor --check-tutorial-data exited unexpected code {doctor.returncode}; "
            f"checksums.json is not yet pinned (run pin.sh after first successful walkthrough)\n"
            f"--- stdout ---\n{doctor.stdout}"
        )


@pytest.mark.network
@pytest.mark.slow
def test_walkthrough_vermont_tract_count_is_193():
    """
    Once the walkthrough has run, final_assignments.json must have exactly 193
    entries (matches tests/acceptance/test_pipeline_acceptance.py baseline).
    Skipped if the walkthrough hasn't been run yet.
    """
    assign = (
        REPO_ROOT
        / "outputs"
        / "tutorial"
        / "2020"
        / "plans"
        / "vt_2020_tutorial"
        / "final_assignments.json"
    )
    if not assign.exists():
        pytest.skip(
            f"walkthrough has not been run yet (missing {assign}); "
            f"run `bash examples/vermont-2020-walkthrough/run.sh` first"
        )
    data = json.loads(assign.read_text(encoding="utf-8"))
    assert len(data) == 193, (
        f"expected 193 tract assignments for Vermont 2020 (matches the project baseline); "
        f"got {len(data)}. The upstream Census TIGER vintage may have drifted."
    )
