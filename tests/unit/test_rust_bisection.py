"""
L2 tests for bisect_py.BisectionTree — bisection split schedule via PyO3.

Verifies:
- Split schedule matches Python's _calculate_max_depth and split logic
- ufactor values per depth match recursive_bisection.py
- Level-order node counts match the tree structure
"""

import os
import pytest

RUST_AVAILABLE = os.environ.get('BISECT_NO_RUST', '0') != '1'
try:
    import bisect_py
    BISECT_PY_IMPORTABLE = True
except ImportError:
    BISECT_PY_IMPORTABLE = False

pytestmark = pytest.mark.skipif(
    not RUST_AVAILABLE or not BISECT_PY_IMPORTABLE,
    reason='bisect_py not available'
)


def python_max_depth(k: int) -> int:
    """Mirror _calculate_max_depth from recursive_bisection.py."""
    if k == 1:
        return 0
    depth = 0
    current = 1
    while current < k:
        current *= 2
        depth += 1
    return depth


def python_ufactor(depth: int) -> float:
    """Mirror the depth-based ufactor from recursive_bisection.py."""
    if depth == 1:
        return 1.001
    elif depth == 2:
        return 1.002
    elif depth == 3:
        return 1.003
    return 1.005


class TestBisectionTreeMaxDepth:

    @pytest.mark.parametrize('k', [1, 2, 3, 4, 7, 8, 14, 52])
    def test_max_depth_matches_python(self, k):
        rust = bisect_py.bisection_max_depth(k)
        py = python_max_depth(k)
        assert rust == py, f"k={k}: Rust={rust}, Python={py}"

    def test_max_depth_via_tree(self):
        for k in [2, 3, 4, 7, 8, 14, 52]:
            tree = bisect_py.BisectionTree.from_k(k)
            assert tree.max_depth() == python_max_depth(k), f"k={k}"


class TestBisectionTreeStructure:

    @pytest.mark.parametrize('k', [2, 3, 4, 7, 8, 14, 52])
    def test_total_splits_equals_k_minus_1(self, k):
        tree = bisect_py.BisectionTree.from_k(k)
        assert tree.total_splits() == k - 1, f"k={k}"

    def test_k2_single_split(self):
        tree = bisect_py.BisectionTree.from_k(2)
        nodes = tree.nodes_at_depth(0)
        assert len(nodes) == 1
        k, k_left, k_right, depth, path = nodes[0]
        assert k == 2
        assert k_left == 1
        assert k_right == 1

    def test_k7_depth0_split(self):
        """k=7: root splits into 3 and 4."""
        tree = bisect_py.BisectionTree.from_k(7)
        d0 = tree.nodes_at_depth(0)
        assert len(d0) == 1
        k, k_left, k_right, _, _ = d0[0]
        assert k == 7
        assert k_left + k_right == 7
        assert min(k_left, k_right) == 3
        assert max(k_left, k_right) == 4

    def test_k7_three_levels(self):
        tree = bisect_py.BisectionTree.from_k(7)
        # Depth 0: 1 node (k=7)
        # Depth 1: 2 nodes (k=3, k=4)
        # Depth 2: 3 nodes (k=2 from 3→1+2, k=2 from 4→2+2, k=2 from 4→2+2)
        assert len(tree.nodes_at_depth(0)) == 1
        assert len(tree.nodes_at_depth(1)) == 2
        assert len(tree.nodes_at_depth(2)) == 3

    def test_k52_structure(self):
        tree = bisect_py.BisectionTree.from_k(52)
        assert tree.max_depth() == 6
        assert tree.total_splits() == 51

    def test_k_left_plus_k_right_equals_k_for_all_nodes(self):
        for k in [3, 7, 14, 52]:
            tree = bisect_py.BisectionTree.from_k(k)
            for depth in range(tree.max_depth()):
                for node in tree.nodes_at_depth(depth):
                    nk, k_left, k_right, nd, path = node
                    assert k_left + k_right == nk, \
                        f"k={k} depth={nd}: {k_left}+{k_right}!={nk}"

    def test_splits_per_depth_sums_to_total(self):
        for k in [2, 7, 14, 52]:
            tree = bisect_py.BisectionTree.from_k(k)
            assert sum(tree.splits_per_depth()) == tree.total_splits()

    def test_paths_are_binary_strings(self):
        tree = bisect_py.BisectionTree.from_k(7)
        for depth in range(tree.max_depth()):
            for node in tree.nodes_at_depth(depth):
                _, _, _, _, path = node
                assert all(c in '01' for c in path), f"non-binary path: {path!r}"

    def test_invalid_k_raises(self):
        with pytest.raises((ValueError, Exception)):
            bisect_py.BisectionTree.from_k(0)


class TestUfactorParity:

    @pytest.mark.parametrize('depth', [1, 2, 3, 4, 5, 10, 99])
    def test_ufactor_matches_python(self, depth):
        rust = bisect_py.bisection_ufactor(depth)
        py = python_ufactor(depth)
        assert abs(rust - py) < 1e-9, f"depth={depth}: Rust={rust}, Python={py}"

    def test_ufactor_tightens_at_early_depths(self):
        """Shallower splits must have tighter tolerance than deeper ones."""
        assert bisect_py.bisection_ufactor(1) < bisect_py.bisection_ufactor(4)
        assert bisect_py.bisection_ufactor(2) < bisect_py.bisection_ufactor(4)
        assert bisect_py.bisection_ufactor(3) < bisect_py.bisection_ufactor(4)
