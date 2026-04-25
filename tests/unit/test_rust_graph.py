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
        assignments = {0: 0, 1: 0, 2: 1, 3: 1, 4: 1}
        p = redist_py.Partition.from_dict(assignments)
        assert p.to_dict() == assignments

    def test_population_balance_uneven(self):
        import numpy as np
        # D0: 1000+1200=2200, D1: 900+1100+800=2800, ideal=2500, dev=300/2500=0.12
        assignments = {0: 0, 1: 0, 2: 1, 3: 1, 4: 1}
        p = redist_py.Partition.from_dict(assignments)
        weights = np.array(SMALL_CSR['vertex_weights'], dtype='int64')
        dev = p.population_balance(weights, n_districts=2)
        assert abs(dev - 0.12) < 0.001

    def test_population_balance_even(self):
        import numpy as np
        # Perfect split: D0=2500, D1=2500
        equal = {0: 0, 1: 0, 2: 1, 3: 1, 4: 1}
        p = redist_py.Partition.from_dict(equal)
        weights = np.array([1250, 1250, 1250, 1250, 1000], dtype='int64')
        # D0: 2500, D1: 3500 — not perfect, but test a truly even case:
        p2 = redist_py.Partition.from_dict({0: 0, 1: 0, 2: 0, 3: 1, 4: 1})
        weights2 = np.array([1000, 1000, 500, 1000, 1500], dtype='int64')
        # D0=2500, D1=2500
        dev = p2.population_balance(weights2, n_districts=2)
        assert dev < 1e-9

    def test_assert_balanced_fails(self):
        import numpy as np
        # D0: 2000, D1: 3000 → 20% deviation → fails ±0.5%
        p = redist_py.Partition.from_dict({0: 0, 1: 0, 2: 1, 3: 1, 4: 1})
        equal_weights = np.array([1000, 1000, 1000, 1000, 1000], dtype='int64')
        with pytest.raises(ValueError):
            p.assert_balanced(equal_weights, n_districts=2, tolerance=0.005)

    def test_assert_balanced_passes(self):
        import numpy as np
        # D0=2500, D1=2500 → perfectly balanced
        p = redist_py.Partition.from_dict({0: 0, 1: 0, 2: 0, 3: 1, 4: 1})
        weights = np.array([1000, 1000, 500, 1000, 1500], dtype='int64')
        # Should not raise
        p.assert_balanced(weights, n_districts=2, tolerance=0.005)

    def test_assert_balanced_default_tolerance(self):
        import numpy as np
        # 2% deviation: passes ±5%, fails ±0.5% (default)
        p = redist_py.Partition.from_dict({0: 0, 1: 0, 2: 1, 3: 1})
        weights = np.array([1200, 1250, 1250, 1300], dtype='int64')
        # D0=2450, D1=2550, dev=50/2500=2%
        with pytest.raises(ValueError):
            p.assert_balanced(weights, n_districts=2)  # default tolerance=0.005
        # But passes with 5% tolerance
        p.assert_balanced(weights, n_districts=2, tolerance=0.05)


class TestVraEdgeWeights:
    """L2 tests for build_vra_edge_weights — the adaptive boost formula via PyO3."""

    def test_minority_minority_edge_boosted(self):
        import numpy as np
        # Tracts 0,1 are minority (>40%); tract 2 is not
        minority_fracs = np.array([0.60, 0.55, 0.15], dtype='float64')
        edges = [(0, 1), (1, 2)]
        weights = redist_py.build_vra_edge_weights(edges, minority_fracs, threshold=0.40)
        assert (0, 1) in weights, "minority-minority edge must be boosted"
        assert (1, 2) not in weights, "mixed edge must not be boosted"

    def test_alpha_formula_correctness(self):
        import numpy as np
        # f_minority = 2/3 → α = max(3.0, 10*(1 - 0.7*(2/3))) ≈ 5.333
        minority_fracs = np.array([0.60, 0.55, 0.10], dtype='float64')
        edges = [(0, 1)]
        weights = redist_py.build_vra_edge_weights(edges, minority_fracs)
        alpha = weights[(0, 1)]
        expected = max(3.0, 10.0 * (1.0 - 0.7 * (2.0 / 3.0)))
        assert abs(alpha - expected) < 1e-9, f"alpha={alpha:.6f}, expected={expected:.6f}"

    def test_floor_at_high_minority_density(self):
        import numpy as np
        # All tracts minority → f=1.0 → α = max(3.0, 10*(1-0.7)) = 3.0
        minority_fracs = np.array([0.70] * 20, dtype='float64')
        edges = [(i, i + 1) for i in range(19)]
        weights = redist_py.build_vra_edge_weights(edges, minority_fracs)
        for alpha in weights.values():
            assert abs(alpha - 3.0) < 1e-9, f"floor should be 3.0, got {alpha}"

    def test_no_minority_tracts_returns_empty(self):
        import numpy as np
        minority_fracs = np.array([0.10, 0.15, 0.20], dtype='float64')
        edges = [(0, 1), (1, 2)]
        weights = redist_py.build_vra_edge_weights(edges, minority_fracs)
        assert len(weights) == 0

    def test_default_threshold_is_040(self):
        import numpy as np
        # Tract 0: 0.41 (just above 40%), tract 1: 0.39 (just below)
        minority_fracs = np.array([0.41, 0.39], dtype='float64')
        edges = [(0, 1)]
        # With default threshold=0.40: only tract 0 qualifies → mixed edge → not boosted
        weights_default = redist_py.build_vra_edge_weights(edges, minority_fracs)
        assert (0, 1) not in weights_default

        # With threshold=0.38: both qualify → boosted
        weights_low = redist_py.build_vra_edge_weights(edges, minority_fracs, threshold=0.38)
        assert (0, 1) in weights_low

    def test_matches_alabama_paper_value(self):
        """Adaptive formula for Alabama-like state (~22% minority tracts)."""
        import numpy as np
        n = 100
        minority_fracs = np.array(
            [0.55 if i < 22 else 0.20 for i in range(n)], dtype='float64'
        )
        edges = [(0, 1)]  # both minority tracts
        weights = redist_py.build_vra_edge_weights(edges, minority_fracs)
        alpha = weights[(0, 1)]
        # f=0.22 → α = max(3.0, 10*(1-0.7*0.22)) = max(3.0, 8.46) = 8.46
        assert abs(alpha - 8.46) < 0.01, f"Alabama alpha should be ~8.46, got {alpha:.4f}"
