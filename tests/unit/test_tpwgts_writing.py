"""
Unit tests for tpwgts file writing in METIS.

Tests edge cases that might cause "Invalid partition weight" errors.
"""

import pytest
import numpy as np
from pathlib import Path
import tempfile
from apportionment.partition.metis_executable import partition_graph_with_executable


class TestTpwgtsWriting:
    """Test tpwgts file writing with various edge cases."""

    def test_simple_2way_split(self):
        """Test basic 2-way split with balanced weights."""
        # Simple graph: 4 nodes in a line
        adjacency = [[1], [0, 2], [1, 3], [2]]
        vertex_weights = np.array([[1, 1], [1, 1], [1, 1], [1, 1]])

        target_weights = [
            [0.5, 0.5],
            [0.5, 0.5]
        ]

        # Should not raise
        result = partition_graph_with_executable(
            adjacency=adjacency,
            vertex_weights=vertex_weights,
            nparts=2,
            target_weights=target_weights,
            ufactor=1.005,
            debug=True
        )

        assert len(result) == 4
        assert all(r in [0, 1] for r in result)

    def test_unbalanced_2way_split(self):
        """Test 2-way split with unbalanced weights (one side gets more)."""
        adjacency = [[1], [0, 2], [2, 3], [2]]
        vertex_weights = np.array([[1, 1], [1, 1], [1, 1], [1, 1]])

        target_weights = [
            [0.75, 0.8],  # Left gets more
            [0.25, 0.2]   # Right gets less
        ]

        result = partition_graph_with_executable(
            adjacency=adjacency,
            vertex_weights=vertex_weights,
            nparts=2,
            target_weights=target_weights,
            ufactor=1.005,
            debug=True
        )

        assert len(result) == 4

    def test_extreme_unbalanced_split(self):
        """Test extreme unbalance (90/10 split)."""
        adjacency = [[1], [0, 2], [1, 3], [2]]
        vertex_weights = np.array([[1, 1], [1, 1], [1, 1], [1, 1]])

        target_weights = [
            [0.9, 0.95],
            [0.1, 0.05]
        ]

        result = partition_graph_with_executable(
            adjacency=adjacency,
            vertex_weights=vertex_weights,
            nparts=2,
            target_weights=target_weights,
            ufactor=1.005,
            ubvec=[1.005, 1000.0],
            debug=True
        )

        assert len(result) == 4

    def test_edge_case_one_side_all_minority(self):
        """Test when one side should get all minority population."""
        adjacency = [[1], [0, 2], [1, 3], [2]]
        vertex_weights = np.array([[1, 1], [1, 1], [1, 1], [1, 1]])

        # Left side gets all minority
        target_weights = [
            [0.5, 1.0],  # 50% pop, 100% minority
            [0.5, 0.0]   # 50% pop, 0% minority
        ]

        # This might fail - test if it does
        with pytest.raises(RuntimeError, match="Invalid partition weight"):
            partition_graph_with_executable(
                adjacency=adjacency,
                vertex_weights=vertex_weights,
                nparts=2,
                target_weights=target_weights,
                ufactor=1.005,
                debug=True
            )

    def test_normalized_weights_sum_to_one(self):
        """Test that normalized weights sum to 1.0 for each constraint."""
        adjacency = [[1], [0, 2], [1, 3], [2]]
        vertex_weights = np.array([[1, 1], [1, 1], [1, 1], [1, 1]])

        # Unnormalized weights
        target_weights = [
            [3, 5],
            [7, 2]
        ]

        result = partition_graph_with_executable(
            adjacency=adjacency,
            vertex_weights=vertex_weights,
            nparts=2,
            target_weights=target_weights,
            ufactor=1.005,
            debug=True
        )

        assert len(result) == 4

    def test_floating_point_precision(self):
        """Test weights with floating point precision issues."""
        adjacency = [[1], [0, 2], [1, 3], [2]]
        vertex_weights = np.array([[1, 1], [1, 1], [1, 1], [1, 1]])

        # Weights that might not sum to exactly 1.0
        target_weights = [
            [0.333333, 0.666667],
            [0.666667, 0.333333]
        ]

        result = partition_graph_with_executable(
            adjacency=adjacency,
            vertex_weights=vertex_weights,
            nparts=2,
            target_weights=target_weights,
            ufactor=1.005,
            debug=True
        )

        assert len(result) == 4

    def test_nway_7_partitions(self):
        """Test n-way partitioning with 7 partitions."""
        # Create a larger graph
        adjacency = [[1], [0, 2], [1, 3], [2, 4], [3, 5], [4, 6], [5]]
        vertex_weights = np.array([[1, 1], [1, 1], [1, 1], [1, 1], [1, 1], [1, 1], [1, 1]])

        # 7 partitions with varying weights
        target_weights = [
            [1/7, 0.25],  # Partition 0: high minority
            [1/7, 0.25],  # Partition 1: high minority
            [1/7, 0.10],  # Partitions 2-6: low minority
            [1/7, 0.10],
            [1/7, 0.10],
            [1/7, 0.10],
            [1/7, 0.10],
        ]

        result = partition_graph_with_executable(
            adjacency=adjacency,
            vertex_weights=vertex_weights,
            nparts=7,
            target_weights=target_weights,
            ufactor=1.005,
            debug=True
        )

        assert len(result) == 7
        assert all(r in range(7) for r in result)


if __name__ == '__main__':
    pytest.main([__file__, '-v'])
