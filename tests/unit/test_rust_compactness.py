"""
L2 tests for redist_py compactness metrics and VRA analysis.

CRITICAL: all WKB polygons must be in projected coordinates (metres).
These tests use coordinates in the millions (EPSG:5070-like).

Verifies:
- Polsby-Popper formula matches Python exactly: 4π×A/P²
- Reock uses centroid + max boundary distance (matching Python approximation)
- VRA analysis matches Python vra_utils.py:analyze_mm_districts()
- 50% MM threshold is inclusive (>= 0.50)
"""

import os
import struct
import math
import pytest

RUST_AVAILABLE = os.environ.get('REDIST_NO_RUST', '0') != '1'
try:
    import redist_py
    REDIST_PY_IMPORTABLE = True
except ImportError:
    REDIST_PY_IMPORTABLE = False

pytestmark = pytest.mark.skipif(
    not RUST_AVAILABLE or not REDIST_PY_IMPORTABLE,
    reason='redist_py not available'
)

BASE = 1_000_000.0  # projected coord base (clearly not degrees)


def make_wkb_polygon(coords: list[tuple[float, float]]) -> bytes:
    """Create WKB polygon from a list of (x, y) coords. Closes automatically."""
    if coords[0] != coords[-1]:
        coords = coords + [coords[0]]
    buf = bytearray()
    buf += b'\x01'                          # little-endian
    buf += struct.pack('<I', 3)             # type = Polygon
    buf += struct.pack('<I', 1)             # 1 ring
    buf += struct.pack('<I', len(coords))
    for x, y in coords:
        buf += struct.pack('<d', x)
        buf += struct.pack('<d', y)
    return bytes(buf)


def square_wkb(x0, y0, side):
    return make_wkb_polygon([
        (x0, y0), (x0+side, y0), (x0+side, y0+side), (x0, y0+side)
    ])


def rect_wkb(x0, y0, w, h):
    return make_wkb_polygon([
        (x0, y0), (x0+w, y0), (x0+w, y0+h), (x0, y0+h)
    ])


# ---------------------------------------------------------------------------
# Polsby-Popper
# ---------------------------------------------------------------------------

class TestPolsbyPopper:

    def test_square_pp_is_pi_over_4(self):
        """1km square: PP = π/4 ≈ 0.7854."""
        wkb = square_wkb(BASE, BASE, 1000.0)
        pp, perimeter = redist_py.compute_polsby_popper(wkb)
        expected = math.pi / 4
        assert abs(pp - expected) < 1e-6, f"square PP={pp:.6f}, expected π/4={expected:.6f}"

    def test_square_perimeter(self):
        wkb = square_wkb(BASE, BASE, 1000.0)
        _, perimeter = redist_py.compute_polsby_popper(wkb)
        assert abs(perimeter - 4000.0) < 0.01, f"perimeter={perimeter}, expected 4000m"

    def test_rectangle_2x1_matches_python(self):
        """2000×500m rectangle: PP = 4π×1e6 / 5000² ≈ 0.503."""
        wkb = rect_wkb(BASE, BASE, 2000.0, 500.0)
        pp, perimeter = redist_py.compute_polsby_popper(wkb)
        area = 2000.0 * 500.0
        p = 2 * (2000.0 + 500.0)
        expected = (4 * math.pi * area) / (p ** 2)
        assert abs(pp - expected) < 1e-9, f"PP={pp:.9f}, expected={expected:.9f}"

    def test_pp_capped_at_1(self):
        wkb = square_wkb(BASE, BASE, 1000.0)
        pp, _ = redist_py.compute_polsby_popper(wkb)
        assert pp <= 1.0

    def test_pp_positive(self):
        wkb = square_wkb(BASE, BASE, 1000.0)
        pp, _ = redist_py.compute_polsby_popper(wkb)
        assert pp > 0.0

    def test_elongated_rectangle_lower_pp(self):
        """5:1 rectangle has lower PP than 2:1."""
        wkb_2x1 = rect_wkb(BASE, BASE, 2000.0, 1000.0)
        wkb_5x1 = rect_wkb(BASE, BASE, 5000.0, 1000.0)
        pp_2x1, _ = redist_py.compute_polsby_popper(wkb_2x1)
        pp_5x1, _ = redist_py.compute_polsby_popper(wkb_5x1)
        assert pp_5x1 < pp_2x1, "elongated rectangle must have lower PP"


# ---------------------------------------------------------------------------
# Reock
# ---------------------------------------------------------------------------

class TestReock:

    def test_square_reock_is_2_over_pi(self):
        """Square: Reock = 2/π ≈ 0.6366."""
        wkb = square_wkb(BASE, BASE, 1000.0)
        r = redist_py.compute_reock(wkb)
        expected = 2.0 / math.pi
        assert abs(r - expected) < 0.001, f"Reock={r:.6f}, expected 2/π={expected:.6f}"

    def test_reock_capped_at_1(self):
        wkb = square_wkb(BASE, BASE, 1000.0)
        r = redist_py.compute_reock(wkb)
        assert r <= 1.0

    def test_reock_positive(self):
        wkb = square_wkb(BASE, BASE, 1000.0)
        r = redist_py.compute_reock(wkb)
        assert r > 0.0


# ---------------------------------------------------------------------------
# All metrics
# ---------------------------------------------------------------------------

class TestAllCompactness:

    def test_all_metrics_returns_dict(self):
        wkb = square_wkb(BASE, BASE, 1000.0)
        m = redist_py.compute_all_compactness(1, wkb)
        assert isinstance(m, dict)
        for key in ['district', 'polsby_popper', 'reock', 'convex_hull_ratio',
                    'perimeter_m', 'area_m2']:
            assert key in m, f"Missing key: {key}"

    def test_district_id_preserved(self):
        wkb = square_wkb(BASE, BASE, 1000.0)
        m = redist_py.compute_all_compactness(42, wkb)
        assert m['district'] == 42

    def test_area_m2_correct(self):
        wkb = square_wkb(BASE, BASE, 1000.0)
        m = redist_py.compute_all_compactness(1, wkb)
        assert abs(m['area_m2'] - 1_000_000.0) < 0.01  # 1km² = 1e6 m²

    def test_convex_hull_ratio_square_is_1(self):
        wkb = square_wkb(BASE, BASE, 1000.0)
        m = redist_py.compute_all_compactness(1, wkb)
        assert abs(m['convex_hull_ratio'] - 1.0) < 1e-6

    def test_all_values_in_range(self):
        wkb = square_wkb(BASE, BASE, 1000.0)
        m = redist_py.compute_all_compactness(1, wkb)
        for metric in ['polsby_popper', 'reock', 'convex_hull_ratio']:
            assert 0.0 <= m[metric] <= 1.0, f"{metric}={m[metric]} out of [0,1]"


# ---------------------------------------------------------------------------
# VRA Analysis
# ---------------------------------------------------------------------------

class TestVraAnalysis:

    def _run(self, assignments, pct_minority_per_tract, n_tracts=6, threshold=0.50):
        total_pops = [1000] * n_tracts
        minority_pops = [p * 1000 for p in pct_minority_per_tract]
        return redist_py.compute_vra_analysis(
            assignments, total_pops, minority_pops, [0.0]*n_tracts, [0.0]*n_tracts, threshold
        )

    def test_mm_district_detected(self):
        # Tracts 0-2 (60%) → district 1; tracts 3-5 (20%) → district 2
        assignments = {i: 1 if i < 3 else 2 for i in range(6)}
        pcts = [0.60]*3 + [0.20]*3
        result = self._run(assignments, pcts)
        assert result['mm_count'] == 1
        assert 1 in result['mm_districts']
        assert 2 not in result['mm_districts']

    def test_no_mm_districts(self):
        assignments = {i: i+1 for i in range(4)}
        result = self._run(assignments, [0.30]*4, n_tracts=4)
        assert result['mm_count'] == 0

    def test_threshold_inclusive_at_50pct(self):
        # Exactly 50% → must be MM
        assignments = {0: 1}
        result = redist_py.compute_vra_analysis(
            {0: 1}, [1000], [500.0], [400.0], [100.0], 0.50
        )
        assert result['mm_count'] == 1, "50% must count as MM (>= threshold)"

    def test_just_below_threshold_not_mm(self):
        result = redist_py.compute_vra_analysis(
            {0: 1}, [10000], [4999.0], [0.0], [0.0], 0.50
        )
        assert result['mm_count'] == 0, "49.99% should not be MM"

    def test_returns_all_district_info(self):
        assignments = {i: i+1 for i in range(3)}
        result = redist_py.compute_vra_analysis(
            assignments, [1000]*3, [600.0, 300.0, 200.0], [0.0]*3, [0.0]*3, 0.50
        )
        assert len(result['districts']) == 3
        districts_by_id = {d['district']: d for d in result['districts']}
        assert districts_by_id[1]['is_mm'] is True
        assert districts_by_id[2]['is_mm'] is False
        assert abs(districts_by_id[1]['pct_minority'] - 0.60) < 1e-9

    def test_districts_sorted_by_id(self):
        assignments = {0: 3, 1: 1, 2: 2}
        result = redist_py.compute_vra_analysis(
            assignments, [1000]*3, [300.0]*3, [0.0]*3, [0.0]*3, 0.50
        )
        ids = [d['district'] for d in result['districts']]
        assert ids == sorted(ids), "districts must be sorted"
