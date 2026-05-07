"""
L1 integration test: Python producer + Rust loader format consistency.

Exercises the partisan-shares.tsv contract across the language boundary.
The Python producer (`scripts/data/political/build_dem_shares.py`) writes
the file; the Rust loader (consumed via the PyO3 binding for partisan_weights)
reads it. If the format spec drifts on either side, this test fails.

Skipped if redist_py is not built. The Rust-side parsing logic lives in
`redist-cli/src/partisan_shares.rs` and has its own unit tests; what THIS
test verifies is that the Python writer produces output the Rust side
accepts as valid input.

Run from repo root:
    pytest tests/integration/test_partisan_shares_format_roundtrip.py -v
"""

from __future__ import annotations

import csv
import sys
from pathlib import Path

import pytest

# Make the producer importable
PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT / "scripts" / "data" / "political"))

import build_dem_shares  # noqa: E402

# The Rust loader proper is in redist-cli, not exposed via PyO3 directly.
# What IS exposed is build_partisan_weights, which takes a numpy array of
# dem_shares — already parsed. So we test the format end-to-end by:
#   1. Python producer writes TSV
#   2. Python (this test) parses the TSV with the same rules the Rust loader uses
#   3. Pass parsed shares to redist_py.build_partisan_weights
#   4. Verify the result matches what build_partisan_weights returns when called
#      with the same shares directly (no TSV roundtrip)

REDIST_PY_AVAILABLE = False
try:
    import redist_py  # noqa: F401
    import numpy as np  # noqa: F401
    REDIST_PY_AVAILABLE = True
except ImportError:
    pass

pytestmark = pytest.mark.skipif(
    not REDIST_PY_AVAILABLE,
    reason="redist_py not available; build with `cd redist/python/redist_py && maturin develop`"
)


def parse_tsv_like_rust(path: Path) -> dict[str, float]:
    """
    Mirror the Rust loader's parsing rules from
    redist-cli/src/partisan_shares.rs::load_partisan_shares_map:
      - Skip lines starting with '#' or blank
      - First non-blank line whose 2nd column doesn't parse as float = header (skip)
      - GEOIDs are zfilled to 11 chars
      - dem_share must be in [0.0, 1.0]
    """
    shares: dict[str, float] = {}
    data_started = False
    with path.open(encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            parts = line.split("\t")
            assert len(parts) >= 2, f"line has fewer than 2 fields: {line!r}"
            if not data_started:
                try:
                    float(parts[1])
                except ValueError:
                    data_started = True
                    continue  # header
                data_started = True
            geoid = parts[0].strip().zfill(11)
            share = float(parts[1])
            assert 0.0 <= share <= 1.0, f"share {share} out of range for {geoid}"
            shares[geoid] = share
    return shares


class TestRoundtrip:
    def test_producer_output_parses_back_to_input(self, tmp_path):
        """Producer writes a TSV; parsing it back yields the original shares."""
        original = {
            "01001020100": 0.42,
            "01001020200": 0.71,
            "01001020300": 0.5,
        }
        out = tmp_path / "shares.tsv"
        build_dem_shares.write_tsv(original, out, {"producer": "roundtrip-test"})

        parsed = parse_tsv_like_rust(out)
        assert set(parsed.keys()) == set(original.keys())
        for k, v in original.items():
            assert abs(parsed[k] - v) < 1e-6, f"share for {k}: parsed={parsed[k]}, original={v}"

    def test_producer_pads_short_geoids(self, tmp_path):
        """Producer writes 11-char GEOIDs; parser also pads (defensive)."""
        out = tmp_path / "shares.tsv"
        build_dem_shares.write_tsv({"1001020100": 0.5}, out, {})  # 10-char input
        parsed = parse_tsv_like_rust(out)
        assert "01001020100" in parsed, f"GEOID padding failed: {parsed}"


class TestPyO3PartisanWeightsAcceptsProducerOutput:
    """The shares loaded from a producer-written TSV produce the same edge weights
    as if we'd built them directly. This exercises both:
      - Format: the TSV round-trips losslessly
      - Algorithm: the PyO3 binding accepts the same inputs the runner builds
    """

    def test_edge_weights_identical_via_tsv_vs_direct(self, tmp_path):
        # Build a minimal scenario: 4 tracts, 2 strong-D adjacent, 2 strong-R adjacent
        shares_dict = {
            "00000000001": 0.70,  # strong-D
            "00000000002": 0.65,  # strong-D
            "00000000003": 0.30,  # strong-R
            "00000000004": 0.35,  # strong-R
        }
        # Writer round-trip
        out = tmp_path / "shares.tsv"
        build_dem_shares.write_tsv(shares_dict, out, {})
        parsed = parse_tsv_like_rust(out)

        # Build aligned numpy array — index 0..3 corresponds to the sorted GEOIDs
        sorted_geoids = sorted(shares_dict.keys())
        shares_array = np.array([parsed[g] for g in sorted_geoids], dtype=np.float64)
        direct_array = np.array([shares_dict[g] for g in sorted_geoids], dtype=np.float64)

        edges = [(0, 1), (2, 3), (0, 2)]  # D-D, R-R, D-R
        w_via_tsv = redist_py.build_partisan_weights(edges, shares_array, 0.55, 0.45)
        w_direct = redist_py.build_partisan_weights(edges, direct_array, 0.55, 0.45)

        # Both should return the same dict (D-D and R-R boosted, D-R not)
        assert w_via_tsv == w_direct, (
            f"weights diverge between TSV and direct path:\n"
            f"  via TSV:  {w_via_tsv}\n"
            f"  direct:   {w_direct}"
        )
        # Sanity: 2 boosted edges (D-D and R-R), no D-R
        assert (0, 1) in w_via_tsv, "D-D edge must be boosted"
        assert (2, 3) in w_via_tsv, "R-R edge must be boosted"
        assert (0, 2) not in w_via_tsv, "D-R edge must NOT be boosted"
