import json
from pathlib import Path
import sys


PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT / "scripts" / "research"))

from run_nrs_cross_census_dfs import census_state_inputs


def test_state_inputs_are_year_scoped(tmp_path: Path) -> None:
    state_dir = tmp_path / "national-2010/states/ri"
    package = state_dir / "package"
    package.mkdir(parents=True)
    (package / "baseline-tree.json").write_text(
        json.dumps({"districts": 2}), encoding="utf-8"
    )
    inputs = census_state_inputs(
        2010,
        state_dir,
        PROJECT_ROOT / "target/release/bisect.exe",
        PROJECT_ROOT / "target/release/bisect-ops.exe",
    )

    assert inputs["year"] == 2010
    assert inputs["state"] == "RI"
    assert inputs["districts"] == 2
    assert inputs["context"].as_posix().endswith(
        "data/2010/certified/ri_blocks_2010.rctx"
    )
