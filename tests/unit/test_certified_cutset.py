from __future__ import annotations

import importlib.util
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts/research/run_certified_cutset.py"
SPEC = importlib.util.spec_from_file_location("certified_cutset", MODULE_PATH)
assert SPEC and SPEC.loader
CUTSET = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CUTSET)


def path5() -> list[list[int]]:
    return [[1], [0, 2], [1, 3], [2, 4], [3]]


def test_connected_assignment_emits_no_cuts() -> None:
    assert CUTSET.separate(path5(), [0, 0, 0, 1, 1]) == []


def test_disconnected_assignment_emits_component_boundaries() -> None:
    cuts = CUTSET.separate(path5(), [0, 1, 0, 1, 1])
    assert len(cuts) == 4
    assert {
        (cut["district_id"], tuple(cut["component"]), tuple(cut["outside_neighbors"]))
        for cut in cuts
    } == {
        (0, (0,), (1,)),
        (0, (2,), (1, 3)),
        (1, (1,), (0, 2)),
        (1, (3, 4), (2,)),
    }


def test_roundingsat_assignment_parser_reads_unit_prefix() -> None:
    output = "s SATISFIABLE\nv -x1 x2 -x3 x4 x5\n"
    assert CUTSET.parse_assignment(output, 4) == [0, 1, 0, 1]


def test_windows_path_maps_to_wsl_mount() -> None:
    assert CUTSET.wsl_path(Path("C:/tmp/model.opb")).lower() == "/mnt/c/tmp/model.opb"


def test_fixed_core_component_is_not_cut() -> None:
    fixed = [0, None, None, None, 1]
    cuts = CUTSET.separate(path5(), [0, 1, 0, 1, 1], fixed)
    assert {
        (cut["district_id"], tuple(cut["component"])) for cut in cuts
    } == {(0, (2,)), (1, (1,))}
