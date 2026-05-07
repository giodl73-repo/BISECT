"""
L2 tests for redist_py.adjacency_to_bin / adjacency_from_bin serialization.
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

# Linear graph: 0-1-2
ADJ = [[1], [0, 2], [1]]
EW = {(0, 1): 500.5, (1, 2): 300.0}
N = 3
E = 2


class TestAdjacencySerialize:

    def test_roundtrip_adjacency(self):
        data = redist_py.adjacency_to_bin(ADJ, EW, N, E)
        g = redist_py.adjacency_from_bin(list(data))
        assert g['adjacency'] == ADJ

    def test_roundtrip_edge_weights(self):
        data = redist_py.adjacency_to_bin(ADJ, EW, N, E)
        g = redist_py.adjacency_from_bin(list(data))
        ew = g['edge_weights']
        assert abs(ew[(0, 1)] - 500.5) < 1e-9
        assert abs(ew[(1, 2)] - 300.0) < 1e-9

    def test_roundtrip_vertex_edge_counts(self):
        data = redist_py.adjacency_to_bin(ADJ, EW, N, E)
        g = redist_py.adjacency_from_bin(list(data))
        assert g['n_vertices'] == N
        assert g['n_edges'] == E

    def test_output_is_bytes(self):
        data = redist_py.adjacency_to_bin(ADJ, EW, N, E)
        assert isinstance(data, bytes)

    def test_magic_header(self):
        data = redist_py.adjacency_to_bin(ADJ, EW, N, E)
        assert data[:4] == b'RADJ'

    def test_invalid_magic_raises(self):
        data = bytearray(redist_py.adjacency_to_bin(ADJ, EW, N, E))
        data[0] = ord('X')  # corrupt magic
        with pytest.raises(ValueError, match='magic'):
            redist_py.adjacency_from_bin(list(data))

    def test_truncated_raises(self):
        data = redist_py.adjacency_to_bin(ADJ, EW, N, E)
        with pytest.raises(ValueError):
            redist_py.adjacency_from_bin(list(data[:8]))

    def test_empty_graph_roundtrip(self):
        data = redist_py.adjacency_to_bin([[], []], {}, 2, 0)
        g = redist_py.adjacency_from_bin(list(data))
        assert g['n_vertices'] == 2
        assert g['n_edges'] == 0
        assert g['edge_weights'] == {}

    def test_edge_weights_canonical_order_preserved(self):
        data = redist_py.adjacency_to_bin(ADJ, EW, N, E)
        g = redist_py.adjacency_from_bin(list(data))
        for u, v in g['edge_weights']:
            assert u < v, f"edge ({u},{v}) not canonical after roundtrip"

    def test_large_graph_roundtrip(self):
        n = 50
        adj = [[((i-1)%n), ((i+1)%n)] for i in range(n)]
        ew = {(i, (i+1)%n): float(100+i) for i in range(n) if i < (i+1)%n}
        n_e = len(ew)
        data = redist_py.adjacency_to_bin(adj, ew, n, n_e)
        g = redist_py.adjacency_from_bin(list(data))
        assert g['n_vertices'] == n
        assert len(g['edge_weights']) == n_e
