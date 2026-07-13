from __future__ import annotations

import importlib.util
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts/research/analyze_certified_metis_consensus.py"
SPEC = importlib.util.spec_from_file_location("certified_metis_consensus", MODULE_PATH)
assert SPEC and SPEC.loader
CONSENSUS = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CONSENSUS)


def test_consensus_separates_stable_and_disagreement_units() -> None:
    labels, disagreement = CONSENSUS.consensus(
        [[0, 0, 1, 1], [0, 1, 1, 1], [0, 0, 1, 0]]
    )
    assert labels == [0, None, 1, None]
    assert disagreement == [1, 3]


def test_consensus_rejects_mismatched_assignment_lengths() -> None:
    try:
        CONSENSUS.consensus([[0, 1], [0]])
    except ValueError as error:
        assert str(error) == "assignment lengths differ"
    else:
        raise AssertionError("mismatched assignments were accepted")


def test_component_sizes_are_sorted() -> None:
    adjacency = [[1], [0, 2], [1], [4], [3]]
    assert CONSENSUS.component_sizes(adjacency, [0, 1, 2, 4]) == [3, 1]
