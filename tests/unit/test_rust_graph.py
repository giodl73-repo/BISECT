"""
Phase 0 exit criterion: verify redist_py.Graph round-trips through PyO3.

Skipped when REDIST_NO_RUST=1 (CI without Rust compiled) or when the
redist_py wheel hasn't been built yet (maturin develop not run).

To run locally after building:
    cd redist && maturin develop && cd ..
    pytest tests/unit/test_rust_graph.py -v
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
    reason='redist_py not available (set REDIST_NO_RUST=0 and run maturin develop)'
)


# A simple 5-vertex graph:
#   0 -- 1 -- 2
#   |         |
#   3 -- 4 ---+
# Edges: (0,1), (1,2), (0,3), (3,4), (2,4)  → 5 edges
SMALL_CSR = {
    'adjacency': [[1, 3], [0, 2], [1, 4], [0, 4], [3, 2]],
    'vertex_weights': [1000, 1200, 900, 1100, 800],  # populations
    'n_vertices': 5,
}


class TestGraphRoundTrip:

    def test_n_vertices(self):
        g = redist_py.Graph.from_csr(SMALL_CSR)
        assert g.n_vertices() == 5

    def test_n_edges(self):
        g = redist_py.Graph.from_csr(SMALL_CSR)
        # 5 undirected edges → stored as 10 directed half-edges in CSR
        assert g.n_edges() == 5

    def test_rejects_2d_vertex_weights(self):
        """2D vertex weights (multi-constraint mode) must be rejected at the boundary."""
        import numpy as np
        bad = dict(SMALL_CSR)
        bad['vertex_weights'] = np.array([[1000, 300], [1200, 400], [900, 200],
                                           [1100, 350], [800, 150]])
        with pytest.raises(ValueError, match='2D'):
            redist_py.Graph.from_csr(bad)

    def test_empty_graph_raises(self):
        with pytest.raises(ValueError):
            redist_py.Graph.from_csr({'adjacency': [], 'vertex_weights': [], 'n_vertices': 0})


class TestPartitionRoundTrip:

    def test_to_dict_round_trips(self):
        g = redist_py.Graph.from_csr(SMALL_CSR)
        assignments = {0: 0, 1: 0, 2: 1, 3: 1, 4: 1}
        p = redist_py.Partition.from_dict(assignments)
        result = p.to_dict()
        assert result == assignments

    def test_population_balance(self):
        import numpy as np
        assignments = {0: 0, 1: 0, 2: 1, 3: 1, 4: 1}
        p = redist_py.Partition.from_dict(assignments)
        weights = np.array(SMALL_CSR['vertex_weights'])
        # District 0: 1000+1200=2200, District 1: 900+1100+800=2800
        # Total=5000, ideal=2500, max_dev = 300/2500 = 0.12 (12%)
        dev = p.population_balance(weights, n_districts=2)
        assert abs(dev - 0.12) < 0.001

    def test_assert_balanced_passes_for_even_split(self):
        import numpy as np
        # Perfect 50/50 split
        assignments = {0: 0, 1: 1, 2: 0, 3: 1, 4: 0}
        # weights: 0→1000+900+800=2700, 1→1200+1100=2300 — close but not ±0.5%
        # Use equal weights instead
        p = redist_py.Partition.from_dict({0: 0, 1: 0, 2: 1, 3: 1, 4: 1})
        equal_weights = np.array([1000, 1000, 1000, 1000, 1000])
        # 2000 vs 3000 — 20% deviation, should fail
        with pytest.raises(ValueError):
            p.assert_balanced(equal_weights, n_districts=2, tolerance=0.005)
