"""
L2 tests for redist_py.connect_islands — island component bridging via PyO3.
"""

import os
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

# GEOIDs: SS=50 (Vermont), CCC=001 (county 001), TTTTTT
def geoid(county: str, tract: str) -> str:
    return f"50{county}{tract}"


class TestConnectIslands:

    def test_connected_graph_returns_empty(self):
        adj = [[1], [0, 2], [1]]
        centroids = [(0.0, 0.0), (1.0, 0.0), (2.0, 0.0)]
        geoids = [geoid("001", "000100"), geoid("001", "000200"), geoid("001", "000300")]
        edges = redist_py.connect_islands(adj, centroids, geoids)
        assert edges == []

    def test_isolated_tract_gets_bridged(self):
        # Tract 2 isolated
        adj = [[1], [0], []]
        centroids = [(0.0, 0.0), (1.0, 0.0), (0.5, 5.0)]
        geoids = [geoid("001", "000100"), geoid("001", "000200"), geoid("001", "000300")]
        edges = redist_py.connect_islands(adj, centroids, geoids)
        assert len(edges) == 1
        u, v = edges[0]
        # Tract 2 is isolated; should connect to tract 0 or 1
        assert 2 in (u, v)

    def test_bridge_edges_canonical_order(self):
        adj = [[1], [0], [], []]
        centroids = [(0.0, 0.0), (1.0, 0.0), (0.5, 5.0), (0.5, 6.0)]
        geoids = [geoid("001", "000100"), geoid("001", "000200"),
                  geoid("001", "000300"), geoid("001", "000400")]
        edges = redist_py.connect_islands(adj, centroids, geoids)
        for u, v in edges:
            assert u < v, f"edge ({u},{v}) not canonical (u must be < v)"

    def test_same_county_preferred(self):
        # Main: tracts 0,1 county=001; Island: tract 2 county=001
        # Nearest in county: tract 0 (closer than tract 1)
        adj = [[1], [0], []]
        centroids = [(1_000_000.0, 1_000_000.0),
                     (1_010_000.0, 1_000_000.0),
                     (1_001_000.0, 1_020_000.0)]  # closest to tract 0
        geoids = [geoid("001", "000100"), geoid("001", "000200"), geoid("001", "000300")]
        edges = redist_py.connect_islands(adj, centroids, geoids)
        assert len(edges) == 1
        u, v = edges[0]
        # Should connect 2 → 0 (nearest same-county)
        assert (u, v) == (0, 2)

    def test_cross_county_fallback(self):
        # Main tracts county=001; Island tract county=002 — no same-county match
        adj = [[1], [0], []]
        centroids = [(0.0, 0.0), (1.0, 0.0), (0.5, 5.0)]
        geoids = [geoid("001", "000100"), geoid("001", "000200"), geoid("002", "000100")]
        edges = redist_py.connect_islands(adj, centroids, geoids)
        # Cross-county fallback should still produce a bridge
        assert len(edges) == 1

    def test_return_type_is_list_of_tuples(self):
        adj = [[1], [0], []]
        centroids = [(0.0, 0.0), (1.0, 0.0), (0.5, 5.0)]
        geoids = [geoid("001", "000100"), geoid("001", "000200"), geoid("001", "000300")]
        edges = redist_py.connect_islands(adj, centroids, geoids)
        assert isinstance(edges, list)
        if edges:
            assert isinstance(edges[0], tuple)
            assert len(edges[0]) == 2
