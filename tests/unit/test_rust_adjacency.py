"""
L2 tests for redist_py.build_adjacency — parallel spatial adjacency builder.

Tests with synthetic projected polygons (EPSG:5070-like coordinates in metres).
All polygons use coordinates in the millions to clearly look projected.
"""

import os
import struct
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

BASE = 1_000_000.0  # clearly projected coords (metres)
SZ = 1000.0         # 1km squares


def make_square_wkb(x0: float, y0: float, size: float) -> bytes:
    """Create WKB-encoded square polygon in projected coordinates."""
    coords = [
        (x0, y0), (x0 + size, y0), (x0 + size, y0 + size),
        (x0, y0 + size), (x0, y0)
    ]
    buf = bytearray()
    buf += b'\x01'                          # little-endian
    buf += struct.pack('<I', 3)             # type=Polygon
    buf += struct.pack('<I', 1)             # 1 ring
    buf += struct.pack('<I', len(coords))   # n_points
    for x, y in coords:
        buf += struct.pack('<d', x)
        buf += struct.pack('<d', y)
    return bytes(buf)


# Linear chain: p0-p1-p2
P0 = make_square_wkb(BASE,           BASE, SZ)
P1 = make_square_wkb(BASE + SZ,      BASE, SZ)
P2 = make_square_wkb(BASE + SZ*2,    BASE, SZ)
P_FAR = make_square_wkb(BASE + 10*SZ, BASE, SZ)  # not adjacent to anything


class TestBuildAdjacencyBasic:

    def test_linear_chain_n_edges(self):
        g = redist_py.build_adjacency([P0, P1, P2])
        assert g['n_edges'] == 2
        assert g['n_vertices'] == 3

    def test_linear_chain_adjacency(self):
        g = redist_py.build_adjacency([P0, P1, P2])
        adj = g['adjacency']
        assert 1 in adj[0]  # p0 → p1
        assert 0 in adj[1]  # p1 → p0
        assert 2 in adj[1]  # p1 → p2
        assert 1 in adj[2]  # p2 → p1
        assert 2 not in adj[0]  # p0 not directly adjacent to p2

    def test_adjacency_is_symmetric(self):
        g = redist_py.build_adjacency([P0, P1, P2])
        adj = g['adjacency']
        for i, nbrs in enumerate(adj):
            for j in nbrs:
                assert i in adj[j], f"{j} should have {i} as neighbor"

    def test_edge_weights_exist(self):
        g = redist_py.build_adjacency([P0, P1, P2])
        ew = g['edge_weights']
        assert (0, 1) in ew
        assert (1, 2) in ew
        # All weights should be positive
        for w in ew.values():
            assert w > 0

    def test_edge_weights_canonical_order(self):
        g = redist_py.build_adjacency([P0, P1, P2])
        for (u, v) in g['edge_weights']:
            assert u < v, f"Edge key ({u},{v}) not in canonical order"

    def test_isolated_polygon_no_neighbors(self):
        g = redist_py.build_adjacency([P0, P_FAR])
        adj = g['adjacency']
        assert adj[0] == []
        assert adj[1] == []
        assert g['n_edges'] == 0

    def test_neighbor_lists_sorted(self):
        # Central polygon adjacent to left and right
        center = make_square_wkb(BASE + SZ, BASE, SZ)
        left   = make_square_wkb(BASE,       BASE, SZ)
        right  = make_square_wkb(BASE + SZ*2, BASE, SZ)
        g = redist_py.build_adjacency([center, left, right])
        adj = g['adjacency']
        for nbrs in adj:
            assert nbrs == sorted(nbrs), "neighbor lists must be sorted"

    def test_min_boundary_length_default_10m(self):
        """Corner-touching squares share ~0.1m — filtered at default 10m."""
        diag = make_square_wkb(BASE + SZ, BASE + SZ, SZ)  # diagonal from P0
        g = redist_py.build_adjacency([P0, diag])
        # Corner contact → filtered by min_boundary_length=10
        assert g['n_edges'] == 0

    def test_min_boundary_length_override(self):
        """Setting min_boundary_length=0 includes corner contacts."""
        diag = make_square_wkb(BASE + SZ, BASE + SZ, SZ)
        g = redist_py.build_adjacency([P0, diag], min_boundary_length=0.0)
        assert g['n_edges'] == 1


class TestBuildAdjacencyValidation:

    def test_unprojected_coords_raise(self):
        """WGS84-like degree coordinates should be rejected."""
        def degree_square_wkb(lon0, lat0, size=0.1):
            coords = [
                (lon0, lat0), (lon0+size, lat0), (lon0+size, lat0+size),
                (lon0, lat0+size), (lon0, lat0)
            ]
            buf = bytearray()
            buf += b'\x01'
            buf += struct.pack('<I', 3)
            buf += struct.pack('<I', 1)
            buf += struct.pack('<I', len(coords))
            for x, y in coords:
                buf += struct.pack('<d', x)
                buf += struct.pack('<d', y)
            return bytes(buf)

        wkb0 = degree_square_wkb(-72.5, 44.0)
        wkb1 = degree_square_wkb(-72.4, 44.0)
        with pytest.raises(ValueError, match='projected'):
            redist_py.build_adjacency([wkb0, wkb1])

    def test_return_type_is_dict(self):
        g = redist_py.build_adjacency([P0, P1])
        assert isinstance(g, dict)
        assert 'adjacency' in g
        assert 'edge_weights' in g
        assert 'n_vertices' in g
        assert 'n_edges' in g

    def test_single_polygon(self):
        g = redist_py.build_adjacency([P0])
        assert g['n_vertices'] == 1
        assert g['n_edges'] == 0
        assert g['adjacency'] == [[]]
