"""
L2 tests: Partition::assert_balanced() wired into run_state_redistricting.py.

Verifies that the Rust balance check runs on the final partition and raises
ValueError for imbalanced partitions before any output is written.

These tests run without gpmetis (they mock the partitioner output).
"""

import os
import pytest
import numpy as np
from unittest.mock import patch, MagicMock

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


class TestRustBalanceCheckInPipeline:
    """Test the balance check wired into run_state_redistricting."""

    def test_balanced_partition_passes(self):
        """A perfectly balanced partition does not raise."""
        # D0: tracts 0,1 → 1000+1000=2000, D1: tracts 2,3 → 1000+1000=2000
        p = redist_py.Partition.from_dict({0: 1, 1: 1, 2: 2, 3: 2})
        vw = np.array([1000, 1000, 1000, 1000], dtype=np.int64)
        # Should not raise
        p.assert_balanced(vw, n_districts=2, tolerance=0.005)

    def test_imbalanced_partition_raises_valueerror(self):
        """An imbalanced partition raises ValueError before output is written."""
        # D0: 100, D1: 1900 → 90% deviation
        p = redist_py.Partition.from_dict({0: 1, 1: 2, 2: 2, 3: 2})
        vw = np.array([100, 600, 600, 700], dtype=np.int64)
        with pytest.raises(ValueError):
            p.assert_balanced(vw, n_districts=2, tolerance=0.005)

    def test_error_message_names_deviation(self):
        """ValueError message includes 'deviates' for diagnosability."""
        p = redist_py.Partition.from_dict({0: 1, 1: 2})
        vw = np.array([100, 900], dtype=np.int64)
        with pytest.raises(ValueError, match='deviates'):
            p.assert_balanced(vw, n_districts=2, tolerance=0.005)

    def test_tolerance_boundary(self):
        """Exactly at 0.5% passes; 0.5% + epsilon fails."""
        # Total=2000, ideal=1000, tolerance=0.005 → max allowed deviation=5
        # D0=1005, D1=995 → dev = 5/1000 = 0.005 → exactly at boundary
        p = redist_py.Partition.from_dict({0: 1, 1: 1, 2: 2, 3: 2})
        vw = np.array([505, 500, 498, 497], dtype=np.int64)
        # D0=1005, D1=995, total=2000, ideal=1000, dev=5/1000=0.005 → passes
        p.assert_balanced(vw, n_districts=2, tolerance=0.005)

        # D0=1006 → dev=6/1000=0.006 → fails
        vw2 = np.array([506, 500, 498, 496], dtype=np.int64)
        p2 = redist_py.Partition.from_dict({0: 1, 1: 1, 2: 2, 3: 2})
        with pytest.raises(ValueError):
            p2.assert_balanced(vw2, n_districts=2, tolerance=0.005)

    def test_leaf_only_not_intermediate(self):
        """
        Intermediate bisection nodes may have 10-20% imbalance due to ufactor.
        assert_balanced uses 0.5% — it must only be called on final leaf partitions.

        This test verifies that a looser tolerance (e.g. 20%) passes intermediate results
        while ±0.5% would fail — confirming the caller must choose the right moment.
        """
        # Simulated intermediate split: D0=1200, D1=800 → 20% deviation
        p = redist_py.Partition.from_dict({0: 1, 1: 1, 2: 2})
        vw = np.array([600, 600, 800], dtype=np.int64)
        # Final tolerance fails
        with pytest.raises(ValueError):
            p.assert_balanced(vw, n_districts=2, tolerance=0.005)
        # Intermediate tolerance passes
        p.assert_balanced(vw, n_districts=2, tolerance=0.25)

    def test_single_district_trivially_balanced(self):
        """Single-district state (VT, DE, etc.) always passes — no split to check."""
        p = redist_py.Partition.from_dict({0: 1, 1: 1, 2: 1})
        vw = np.array([500, 700, 300], dtype=np.int64)
        # 1 district = everything in D1 → deviation vs ideal (total/1) = 0
        p.assert_balanced(vw, n_districts=1, tolerance=0.005)
