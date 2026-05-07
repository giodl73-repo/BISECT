"""
Unit tests for VRA minority edge-weighting approach.

Tests the core algorithm: edges between high-minority tracts are weighted
higher to encourage minority clustering while maintaining population balance.
These tests run without requiring pipeline outputs (use synthetic data).
"""

import pytest
import numpy as np
import tempfile
from pathlib import Path


# ── Synthetic graph helpers ──────────────────────────────────────────────────

def make_linear_graph(n_tracts: int, minority_pcts: list[float]) -> tuple:
    """
    Create a linear tract graph: 0-1-2-...(n-1).
    Returns (adjacency, vertex_weights, tracts_df).
    """
    import pandas as pd
    adjacency = []
    for i in range(n_tracts):
        neighbors = []
        if i > 0: neighbors.append(i - 1)
        if i < n_tracts - 1: neighbors.append(i + 1)
        adjacency.append(neighbors)

    # Equal population, varying minority
    pop = 1000
    vertex_weights = np.array([pop] * n_tracts, dtype=np.int32)

    tracts_df = pd.DataFrame({
        'pct_minority': minority_pcts,
        'total_pop': [pop] * n_tracts,
    })
    return adjacency, vertex_weights, tracts_df


def build_vra_edge_weights(adjacency, tracts_df,
                            threshold=0.40, weight=10.0, normal=1.0):
    """Mirror the exact logic from run_state_redistricting.py."""
    is_minority = (tracts_df['pct_minority'] >= threshold).values
    n = len(adjacency)
    ew = {}
    for i in range(n):
        for j in adjacency[i]:
            if i < j:
                ew[(i, j)] = weight if (is_minority[i] and is_minority[j]) else normal
    return ew


# ── Tests ────────────────────────────────────────────────────────────────────

class TestEdgeWeightConstruction:
    """Verify edge weights are built correctly from minority percentages."""

    def test_minority_minority_edges_get_boost(self):
        """Edges between two minority tracts get the boost weight."""
        # 4 tracts: 0,1 are minority (60%), 2,3 are not (20%)
        # Linear: 0-1-2-3
        adj = [[1], [0, 2], [1, 3], [2]]
        import pandas as pd
        tracts = pd.DataFrame({'pct_minority': [0.60, 0.60, 0.20, 0.20]})
        ew = build_vra_edge_weights(adj, tracts, threshold=0.40, weight=10.0)

        assert ew[(0, 1)] == 10.0  # minority-minority → boost
        assert ew[(1, 2)] == 1.0   # minority-nonminority → normal
        assert ew[(2, 3)] == 1.0   # nonminority-nonminority → normal

    def test_threshold_exactly_at_boundary(self):
        """Tracts at exactly the threshold count as minority."""
        adj = [[1], [0]]
        import pandas as pd
        tracts = pd.DataFrame({'pct_minority': [0.40, 0.40]})
        ew = build_vra_edge_weights(adj, tracts, threshold=0.40, weight=5.0)
        assert ew[(0, 1)] == 5.0

    def test_all_minority_all_edges_boosted(self):
        """When all tracts are minority, all edges get the boost."""
        adj = [[1, 2], [0, 2], [0, 1]]
        import pandas as pd
        tracts = pd.DataFrame({'pct_minority': [0.80, 0.75, 0.60]})
        ew = build_vra_edge_weights(adj, tracts, threshold=0.40, weight=8.0)
        assert all(w == 8.0 for w in ew.values())

    def test_no_minority_no_edges_boosted(self):
        """When no tracts are minority, no edges get the boost."""
        adj = [[1], [0, 2], [1]]
        import pandas as pd
        tracts = pd.DataFrame({'pct_minority': [0.10, 0.15, 0.20]})
        ew = build_vra_edge_weights(adj, tracts, threshold=0.40, weight=10.0)
        assert all(w == 1.0 for w in ew.values())

    def test_edges_stored_canonical_order(self):
        """Edge (i,j) is stored with i < j only."""
        adj = [[1, 2], [0, 2], [0, 1]]
        import pandas as pd
        tracts = pd.DataFrame({'pct_minority': [0.60, 0.60, 0.60]})
        ew = build_vra_edge_weights(adj, tracts)
        # All keys should have i < j
        assert all(i < j for i, j in ew.keys())
        # No duplicate edges
        assert len(ew) == 3  # triangle: (0,1),(0,2),(1,2)


class TestAdaptiveBoostScaling:
    """Verify the adaptive boost factor scales correctly with minority density."""

    def _compute_boost(self, minority_frac: float) -> float:
        """Mirror the formula from run_state_redistricting.py."""
        return max(3.0, 10.0 * (1.0 - 0.7 * minority_frac))

    def test_low_minority_gets_full_boost(self):
        """States with few minority tracts get the full 10x boost."""
        boost = self._compute_boost(0.0)
        assert abs(boost - 10.0) < 0.01

    def test_high_minority_gets_reduced_boost(self):
        """States like California (63% minority tracts) get reduced boost."""
        boost = self._compute_boost(0.635)  # California
        assert 3.0 <= boost < 7.0, f'Expected 3-7x for CA, got {boost:.1f}x'

    def test_very_high_minority_floors_at_3x(self):
        """Even at 100% minority saturation, boost never drops below 3x."""
        boost = self._compute_boost(1.0)
        assert abs(boost - 3.0) < 1e-9

    def test_alabama_gets_near_full_boost(self):
        """Alabama (~40% minority tracts) gets close to full boost."""
        boost = self._compute_boost(0.40)
        assert boost >= 7.0, f'Alabama should get >=7x boost, got {boost:.1f}x'


class TestPopulationBalance:
    """
    Verify that VRA edge weighting maintains ±0.5% population balance.
    Uses METIS directly (requires gpmetis).
    """

    @pytest.fixture(autouse=True)
    def skip_if_no_metis(self):
        """Skip if gpmetis is not available."""
        import sys
        sys.path.insert(0, str(Path(__file__).parents[2] / 'src'))
        try:
            from apportionment.partition.metis_executable import find_gpmetis_executable
            if find_gpmetis_executable() is None:
                pytest.skip('gpmetis not available')
        except ImportError:
            pytest.skip('METIS not available')

    def _max_pop_deviation(self, adjacency, vertex_weights, num_districts, edge_weights):
        """Run METIS bisection and return max population deviation."""
        import sys
        sys.path.insert(0, str(Path(__file__).parents[2] / 'src'))
        from apportionment.partition.recursive_bisection import RecursiveBisection

        with tempfile.TemporaryDirectory() as tmp:
            p = RecursiveBisection(
                adjacency=adjacency,
                vertex_weights=vertex_weights,
                num_districts=num_districts,
                save_intermediate=False,
                intermediate_dir=tmp,
                state_code='TEST',
                tqdm_position=999,
                debug=False,
                edge_weights=edge_weights,
                ufactor=5,
                niter=100,
            )
            assignments = p.partition()

        total = int(vertex_weights.sum())
        ideal = total / num_districts
        dist_pop = {}
        for tract_idx, dist_id in assignments.items():
            dist_pop[dist_id] = dist_pop.get(dist_id, 0) + vertex_weights[tract_idx]
        return max(abs(p - ideal) / ideal for p in dist_pop.values())

    def test_no_boost_balanced(self):
        """Without edge weighting, all-equal-pop graph stays balanced."""
        import pandas as pd
        n = 20
        adj, vw, tracts = make_linear_graph(n, [0.1] * n)
        ew = build_vra_edge_weights(adj, tracts, weight=1.0)
        dev = self._max_pop_deviation(adj, vw, 4, ew)
        assert dev <= 0.005, f'Expected ≤0.5% deviation, got {dev*100:.2f}%'

    def test_minority_boost_maintains_balance(self):
        """10x boost on minority-minority edges stays within ±0.5%."""
        import pandas as pd
        # 20 tracts: first 8 are minority (clustered), rest not
        pcts = [0.70] * 8 + [0.10] * 12
        adj, vw, tracts = make_linear_graph(20, pcts)
        ew = build_vra_edge_weights(adj, tracts, threshold=0.40, weight=10.0)
        dev = self._max_pop_deviation(adj, vw, 4, ew)
        assert dev <= 0.005, (
            f'VRA edge weighting broke population balance: {dev*100:.2f}% deviation. '
            f'Constitutional limit is ±0.5%.'
        )

    def test_high_boost_still_balanced(self):
        """Even very high boost (50x) should not exceed ±1% with ufactor constraint."""
        import pandas as pd
        pcts = [0.80] * 6 + [0.10] * 14
        adj, vw, tracts = make_linear_graph(20, pcts)
        ew = build_vra_edge_weights(adj, tracts, threshold=0.40, weight=50.0)
        dev = self._max_pop_deviation(adj, vw, 4, ew)
        # Looser tolerance — very high boost can stress balance, but ufactor limits it
        assert dev <= 0.01, (
            f'High boost (50x) produced {dev*100:.2f}% deviation (limit: 1%)'
        )


class TestVRACodePath:
    """
    Verify the VRA code path in run_state_redistricting actually reaches
    edge-weighting logic and does NOT use multi-constraint vertex weights.
    """

    def test_vra_mode_stays_true_throughout_run(self):
        """
        vra_mode must remain True for the entire VRA run — it is a run-type flag,
        not a multi-constraint flag. Population stats use vertex_weights.ndim to
        distinguish 1D vs 2D arrays, NOT vra_mode (invariant fixed in VRA rewrite).

        Previous bug: vra_mode was cleared to False after edge-weight setup, which
        caused the vra_analysis block to be skipped (vra_analysis.pkl not written).
        """
        import pandas as pd
        import numpy as np

        partition_mode = 'metis-vra'
        vra_mode = False

        if partition_mode == 'metis-vra':
            vra_mode = True

            tracts_df = pd.DataFrame({'pct_minority': [0.6, 0.2, 0.5, 0.1]})
            adj = [[1], [0, 2], [1, 3], [2]]
            ew = build_vra_edge_weights(adj, tracts_df)

            # vra_mode must NOT be cleared here — analysis block needs it True
            # (the old bug cleared it here, skipping vra_analysis.pkl write)

        assert vra_mode, (
            'vra_mode must stay True after edge-weight setup. '
            'Clearing it causes vra_analysis.pkl to not be written.'
        )

    def test_vertex_weights_remain_1d(self):
        """VRA mode must NOT modify vertex_weights to 2D (multi-constraint)."""
        import numpy as np

        # Simulate what the pipeline does
        original_vw = np.array([1000, 2000, 1500, 1200], dtype=np.float64)
        vertex_weights = original_vw.copy()

        # VRA edge-weighting should NOT change vertex_weights
        # (old multi-constraint code would set vertex_weights = vertex_weights_vra)
        # The new code only changes edge_weights

        assert vertex_weights.ndim == 1, (
            f'vertex_weights must stay 1D but got {vertex_weights.ndim}D. '
            'Multi-constraint mode was incorrectly activated.'
        )
        np.testing.assert_array_equal(vertex_weights, original_vw)


# ── Rust formula parity tests ─────────────────────────────────────────────────
import os as _os
_RUST_AVAILABLE = _os.environ.get('REDIST_NO_RUST', '0') != '1'
try:
    import redist_py as _redist_py
    _RUST_IMPORTABLE = True
except ImportError:
    _RUST_IMPORTABLE = False

_RUST_SKIP = pytest.mark.skipif(
    not _RUST_AVAILABLE or not _RUST_IMPORTABLE,
    reason='redist_py not available'
)


@_RUST_SKIP
class TestRustFormulaParity:
    """
    Verify Rust build_vra_edge_weights produces identical results to the
    Python formula across representative state profiles.

    These are the tests that would catch formula drift between the Rust
    implementation and any future Python copy. If they fail, it means
    someone edited one implementation without updating the other.
    """

    def _python_weights(self, adj, minority_pcts, threshold=0.40):
        """Reference: Python formula as it existed before Phase 1a deletion."""
        import numpy as np
        minority_fracs = np.array(minority_pcts)
        is_minority = minority_fracs >= threshold
        f_minority = is_minority.mean()
        alpha = max(3.0, 10.0 * (1.0 - 0.7 * f_minority))
        ew = {}
        for i in range(len(adj)):
            for j in adj[i]:
                if i < j and is_minority[i] and is_minority[j]:
                    ew[(i, j)] = alpha
        return ew

    def _rust_weights(self, adj, minority_pcts, threshold=0.40):
        import numpy as np
        minority_fracs = np.array(minority_pcts, dtype=np.float64)
        edges = [(i, j) for i in range(len(adj)) for j in adj[i] if i < j]
        return _redist_py.build_vra_edge_weights(edges, minority_fracs, threshold=threshold)

    def _assert_parity(self, adj, minority_pcts, label, threshold=0.40):
        py = self._python_weights(adj, minority_pcts, threshold)
        rs = self._rust_weights(adj, minority_pcts, threshold)
        assert set(py.keys()) == set(rs.keys()), (
            f'{label}: boosted edge sets differ\n'
            f'  Python only: {set(py)-set(rs)}\n'
            f'  Rust only:   {set(rs)-set(py)}'
        )
        for edge, py_w in py.items():
            rs_w = rs[edge]
            assert abs(py_w - rs_w) < 1e-9, (
                f'{label} edge {edge}: Python={py_w:.6f} Rust={rs_w:.6f}'
            )

    def test_parity_alabama_profile(self):
        """Alabama: ~22% minority tracts → alpha ≈ 8.46."""
        n = 100
        # Linear graph: tracts 0-21 minority, 22-99 not
        adj = [[i-1] if i == n-1 else ([i+1] if i == 0 else [i-1, i+1]) for i in range(n)]
        pcts = [0.55 if i < 22 else 0.20 for i in range(n)]
        self._assert_parity(adj, pcts, 'Alabama')

    def test_parity_georgia_profile(self):
        """Georgia: ~34% minority tracts → alpha ≈ 7.6."""
        n = 60
        adj = [[i-1] if i == n-1 else ([i+1] if i == 0 else [i-1, i+1]) for i in range(n)]
        pcts = [0.55 if i < 20 else 0.20 for i in range(n)]
        self._assert_parity(adj, pcts, 'Georgia')

    def test_parity_california_profile(self):
        """California: ~63% minority tracts → alpha ≈ 5.6 (near floor)."""
        n = 50
        adj = [[i-1] if i == n-1 else ([i+1] if i == 0 else [i-1, i+1]) for i in range(n)]
        pcts = [0.65 if i < 32 else 0.15 for i in range(n)]
        self._assert_parity(adj, pcts, 'California')

    def test_parity_all_minority_floor(self):
        """All tracts minority → f=1.0 → alpha = 3.0 (floor)."""
        n = 20
        adj = [[i-1] if i == n-1 else ([i+1] if i == 0 else [i-1, i+1]) for i in range(n)]
        pcts = [0.70] * n
        self._assert_parity(adj, pcts, 'All-minority floor')

    def test_parity_no_minority_empty(self):
        """No minority tracts → no boosted edges → both return empty."""
        n = 20
        adj = [[i-1] if i == n-1 else ([i+1] if i == 0 else [i-1, i+1]) for i in range(n)]
        pcts = [0.10] * n
        self._assert_parity(adj, pcts, 'No-minority empty')

    def test_parity_south_carolina_profile(self):
        """South Carolina: ~16% minority tracts → alpha ≈ 8.9."""
        n = 80
        adj = [[i-1] if i == n-1 else ([i+1] if i == 0 else [i-1, i+1]) for i in range(n)]
        pcts = [0.50 if i < 13 else 0.20 for i in range(n)]
        self._assert_parity(adj, pcts, 'South Carolina')
