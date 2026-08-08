from pathlib import Path
import sys

import pytest


PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT / "scripts" / "research"))

from run_nrs_ri_sensitivity import (
    SensitivityError,
    assignment_overlap,
    benchmark_rank,
    derive_seed,
    pack_assignment,
    unpack_assignment,
)


def test_seed_derivation_is_domain_separated_and_stable() -> None:
    result = derive_seed(b'{"a":1}', 1)

    assert result == {
        "diagnostic_index": 1,
        "seed_digest_sha256": (
            "0456e63ded6a3e90bbfba35d09448991"
            "4cf4759c353539600bd8656a1ed46ec7"
        ),
        "seed_u64_little_endian": 10393862557195982340,
        "engine_seed": 1583557600,
    }


def test_seed_derivation_rejects_out_of_range_index() -> None:
    with pytest.raises(SensitivityError, match="outside 1..100"):
        derive_seed(b"{}", 0)


def test_assignment_packing_round_trips_little_bit_first() -> None:
    assignment = [0, 1, 1, 0, 1, 0, 0, 1, 1]

    packed = pack_assignment(assignment)

    assert packed == bytes([0b10010110, 0b00000001])
    assert unpack_assignment(packed, len(assignment)) == assignment


def test_assignment_overlap_ignores_binary_label_flip() -> None:
    benchmark = [0, 0, 1, 1]

    assert assignment_overlap(benchmark, [1, 1, 0, 0]) == (4, 1.0)


def test_benchmark_rank_reports_tie_interval() -> None:
    benchmark = (1, 2, 3, (0, 1))
    keys = [(0, 9, 9, (0, 1)), benchmark, benchmark, (2, 0, 0, (0, 1))]

    assert benchmark_rank(keys, benchmark) == {
        "diagnostic_seeds_better": 1,
        "diagnostic_seeds_tied": 2,
        "rank_min": 2,
        "rank_max": 4,
        "rank_denominator": 5,
    }
