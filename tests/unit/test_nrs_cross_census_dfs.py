from pathlib import Path
import sys


PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT / "scripts" / "research"))

from run_nrs_cross_census_dfs import census_state_inputs


def test_state_inputs_are_year_scoped() -> None:
    state_dir = PROJECT_ROOT / "runs/nrs-v0.3/neutral-analysis/national-2010/states/ri"
    inputs = census_state_inputs(
        2010,
        state_dir,
        PROJECT_ROOT / "target/release/bisect.exe",
        PROJECT_ROOT / "target/release/bisect-ops.exe",
    )

    assert inputs["year"] == 2010
    assert inputs["state"] == "RI"
    assert inputs["context"].as_posix().endswith(
        "data/2010/certified/ri_blocks_2010.rctx"
    )
