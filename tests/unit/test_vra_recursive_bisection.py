"""
Unit tests for VRA mode in recursive bisection (recursive_bisection.py)

Tests VRA target weight lookup and multi-constraint integration.

NOTE: Skipped — imports src.apportionment.partition.recursive_bisection which
was archived on 2026-04-29 (archive/python-pipeline-final/).  VRA mode is now
implemented in Rust (redist-cli --partition-mode metis-vra / vra-section).
"""
import pytest
pytestmark = pytest.mark.skip(reason="apportionment.partition archived 2026-04-29")

import pytest
import numpy as np
from src.apportionment.partition.recursive_bisection import RecursiveBisection, PartitionNode
from src.apportionment.partition.vra_targets import create_vra_target_tree


class TestPartitionNode:
    """Test PartitionNode with VRA mode."""

    def test_node_creation_standard_mode(self):
        """Test creating node in standard mode."""
        blocks = {0, 1, 2, 3, 4}
        node = PartitionNode(blocks, depth=1, name="CA0", state_code="CA")

        assert len(node.block_indices) == 5
        assert node.depth == 1
        assert node.name == "CA0"
        assert node.state_code == "CA"
        assert node.left_child is None
        assert node.right_child is None

    def test_node_binary_path_extraction(self):
        """Test binary path extraction from node names."""
        # Test various node naming patterns
        test_cases = [
            ("AL", ""),
            ("AL0", "0"),
            ("AL1", "1"),
            ("AL01", "01"),
            ("AL101", "101"),
            ("CA00", "00"),
        ]

        for name, expected_path in test_cases:
            # Extract path like the algorithm does
            path = ''.join(c for c in name if c in '01')
            assert path == expected_path, f"Failed for node name: {name}"


class TestVRATargetWeightLookup:
    """Test VRA target weight lookup in recursive bisection."""

    def test_get_vra_targets_root_node(self):
        """Test getting VRA targets for root node."""
        # Create simple adjacency: 10 blocks in a line
        adjacency = [[i+1] if i < 9 else [] for i in range(10)]
        for i in range(1, 10):
            adjacency[i].append(i-1)

        # Create 2D vertex weights: [population, minority_vap]
        vertex_weights = np.array([
            [1000, 600],  # 60% minority
            [1000, 600],
            [1000, 200],  # 20% minority
            [1000, 200],
            [1000, 200],
            [1000, 200],
            [1000, 200],
            [1000, 200],
            [1000, 200],
            [1000, 200],
        ])

        # Overall: 30% minority, want 1 MM district at 60%
        vra_tree = create_vra_target_tree(
            num_districts=2,
            target_mm_districts=1,
            state_minority_pct=0.30,
            mm_target_pct=0.60
        )

        partitioner = RecursiveBisection(
            adjacency=adjacency,
            vertex_weights=vertex_weights,
            num_districts=2,
            state_code="TEST",
            vra_mode=True,
            vra_target_weights=vra_tree
        )

        # Get targets for root node (2 districts)
        root_node = PartitionNode(
            block_indices=set(range(10)),
            depth=0,
            name="TEST",
            state_code="TEST"
        )
        root_node.target_districts = 2

        targets = partitioner._get_vra_target_weights_for_node(root_node)

        assert targets is not None
        assert len(targets) == 2  # Left and right
        assert len(targets[0]) == 2  # [pop, minority]
        assert len(targets[1]) == 2

        # Left should get more minority (it's the MM district)
        left_pop, left_minority = targets[0]
        right_pop, right_minority = targets[1]

        assert left_pop == pytest.approx(0.5)  # Equal population split
        assert right_pop == pytest.approx(0.5)
        assert left_minority > right_minority  # More minority goes left

    def test_get_vra_targets_no_vra_mode(self):
        """Test that standard mode returns None for VRA targets."""
        adjacency = [[i+1] if i < 5 else [] for i in range(5)]
        vertex_weights = np.array([1000, 1000, 1000, 1000, 1000])

        partitioner = RecursiveBisection(
            adjacency=adjacency,
            vertex_weights=vertex_weights,
            num_districts=2,
            state_code="TEST",
            vra_mode=False,
            vra_target_weights=None
        )

        node = PartitionNode(set(range(5)), 0, "TEST", "TEST")
        node.target_districts = 2

        targets = partitioner._get_vra_target_weights_for_node(node)

        assert targets is None

    def test_get_vra_targets_leaf_node(self):
        """Test that leaf nodes (1 district) return None."""
        adjacency = [[i+1] if i < 5 else [] for i in range(5)]
        vertex_weights = np.array([[1000, 600]] * 5)

        vra_tree = create_vra_target_tree(2, 1, 0.30, 0.60)

        partitioner = RecursiveBisection(
            adjacency=adjacency,
            vertex_weights=vertex_weights,
            num_districts=2,
            state_code="TEST",
            vra_mode=True,
            vra_target_weights=vra_tree
        )

        # Leaf node with 1 district
        leaf_node = PartitionNode(set(range(2)), 2, "TEST01", "TEST")
        leaf_node.target_districts = 1

        targets = partitioner._get_vra_target_weights_for_node(leaf_node)

        # Leaf nodes don't split further, so no targets
        assert targets is None


class TestVRAPopulationCalculation:
    """Test population calculation in VRA mode with 2D weights."""

    def test_population_sum_vra_mode(self):
        """Test that population is calculated from first column in VRA mode."""
        adjacency = [[i+1] if i < 5 else [] for i in range(5)]
        for i in range(1, 5):
            adjacency[i].append(i-1)

        # 2D weights: [population, minority_vap]
        vertex_weights = np.array([
            [1000, 600],
            [1100, 550],
            [900, 450],
            [1050, 525],
            [950, 475],
        ])

        vra_tree = create_vra_target_tree(2, 1, 0.30, 0.60)

        partitioner = RecursiveBisection(
            adjacency=adjacency,
            vertex_weights=vertex_weights,
            num_districts=2,
            state_code="TEST",
            vra_mode=True,
            vra_target_weights=vra_tree
        )

        # Total population should be sum of first column
        expected_pop = 1000 + 1100 + 900 + 1050 + 950  # 5000
        assert partitioner.total_population == expected_pop

    def test_population_sum_standard_mode(self):
        """Test that population is calculated from 1D array in standard mode."""
        adjacency = [[i+1] if i < 5 else [] for i in range(5)]
        vertex_weights = np.array([1000, 1100, 900, 1050, 950])

        partitioner = RecursiveBisection(
            adjacency=adjacency,
            vertex_weights=vertex_weights,
            num_districts=2,
            state_code="TEST",
            vra_mode=False
        )

        assert partitioner.total_population == 5000


class TestVRAModeIntegration:
    """Integration tests for VRA mode."""

    def test_vra_mode_initialization(self):
        """Test that VRA mode initializes correctly."""
        adjacency = [[i+1] if i < 10 else [] for i in range(10)]
        vertex_weights = np.array([[1000, 500]] * 10)
        vra_tree = create_vra_target_tree(3, 1, 0.30, 0.60)

        partitioner = RecursiveBisection(
            adjacency=adjacency,
            vertex_weights=vertex_weights,
            num_districts=3,
            state_code="TEST",
            vra_mode=True,
            vra_target_weights=vra_tree
        )

        assert partitioner.vra_mode is True
        assert partitioner.vra_target_tree is not None
        assert partitioner.vra_target_tree == vra_tree

    def test_standard_mode_initialization(self):
        """Test that standard mode works without VRA tree."""
        adjacency = [[i+1] if i < 10 else [] for i in range(10)]
        vertex_weights = np.array([1000] * 10)

        partitioner = RecursiveBisection(
            adjacency=adjacency,
            vertex_weights=vertex_weights,
            num_districts=3,
            state_code="TEST",
            vra_mode=False
        )

        assert partitioner.vra_mode is False
        assert partitioner.vra_target_tree is None

    def test_worker_task_includes_vra_params(self):
        """Test that parallel worker tasks include VRA parameters."""
        adjacency = [[i+1] if i < 20 else [] for i in range(20)]
        for i in range(1, 20):
            adjacency[i].append(i-1)

        vertex_weights = np.array([[1000, 400]] * 20)
        vra_tree = create_vra_target_tree(5, 2, 0.30, 0.60)

        partitioner = RecursiveBisection(
            adjacency=adjacency,
            vertex_weights=vertex_weights,
            num_districts=5,
            state_code="TEST",
            vra_mode=True,
            vra_target_weights=vra_tree
        )

        # Create a mock split task
        node = PartitionNode(set(range(20)), 0, "TEST", "TEST")
        node.target_districts = 5

        # Check that task dict would include VRA params
        # (We can't easily test _split_nodes_parallel without actually running it,
        # but we can verify the attributes exist)
        assert hasattr(partitioner, 'vra_mode')
        assert hasattr(partitioner, 'vra_target_tree')
        assert partitioner.vra_mode is True
        assert partitioner.vra_target_tree is not None


class TestVRAErrorHandling:
    """Test error handling in VRA mode."""

    def test_vra_mode_with_1d_weights_raises_error(self):
        """Test that VRA mode requires 2D vertex weights."""
        adjacency = [[i+1] if i < 5 else [] for i in range(5)]
        vertex_weights = np.array([1000] * 5)  # 1D weights
        vra_tree = create_vra_target_tree(2, 1, 0.30, 0.60)

        # VRA mode with 1D weights should raise IndexError
        with pytest.raises(IndexError):
            partitioner = RecursiveBisection(
                adjacency=adjacency,
                vertex_weights=vertex_weights,
                num_districts=2,
                state_code="TEST",
                vra_mode=True,
                vra_target_weights=vra_tree
            )

    def test_standard_mode_with_2d_weights(self):
        """Test that standard mode can handle 2D weights (uses first column)."""
        adjacency = [[i+1] if i < 5 else [] for i in range(5)]
        vertex_weights = np.array([[1000, 600]] * 5)  # 2D weights

        # Standard mode with 2D weights
        # The algorithm should handle this gracefully
        partitioner = RecursiveBisection(
            adjacency=adjacency,
            vertex_weights=vertex_weights,
            num_districts=2,
            state_code="TEST",
            vra_mode=False
        )

        # In standard mode with 2D weights, the behavior depends on implementation
        # Current implementation expects 1D in standard mode
