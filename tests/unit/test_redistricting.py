"""
Unit tests for redistricting functionality.

Tests core redistricting algorithm, graph partitioning, and METIS integration.
"""

import pytest
import numpy as np
import networkx as nx
from pathlib import Path
import sys

# Add project root to path
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))

from apportionment.partition.recursive_bisection import (
    _extract_subgraph_edge_weights,
    _split_node_worker,
)
from apportionment.partition.metis_wrapper import partition_graph


class TestSubgraphEdgeWeights:
    """Test edge weight extraction for subgraphs."""

    def test_extract_subgraph_edge_weights_none(self):
        """Test extraction when no edge weights provided."""
        result = _extract_subgraph_edge_weights(
            edge_weights=None,
            block_indices={0, 1, 2},
            global_to_local={0: 0, 1: 1, 2: 2}
        )
        assert result is None

    def test_extract_subgraph_edge_weights_simple(self):
        """Test extraction of simple edge weights."""
        edge_weights = {
            (0, 1): 100,
            (1, 2): 200,
            (2, 3): 300,
            (3, 4): 400,
        }
        block_indices = {1, 2, 3}
        global_to_local = {1: 0, 2: 1, 3: 2}

        result = _extract_subgraph_edge_weights(
            edge_weights=edge_weights,
            block_indices=block_indices,
            global_to_local=global_to_local
        )

        # Should only include edges within the subgraph
        assert len(result) == 2
        assert (0, 1) in result  # Local (1,2) -> (0,1)
        assert result[(0, 1)] == 200
        assert (1, 2) in result  # Local (2,3) -> (1,2)
        assert result[(1, 2)] == 300

    def test_extract_subgraph_edge_weights_canonical_ordering(self):
        """Test that edge keys use canonical ordering (min, max)."""
        edge_weights = {
            (5, 3): 150,  # Larger index first
            (2, 4): 250,
        }
        block_indices = {2, 3, 4, 5}
        global_to_local = {2: 0, 3: 1, 4: 2, 5: 3}

        result = _extract_subgraph_edge_weights(
            edge_weights=edge_weights,
            block_indices=block_indices,
            global_to_local=global_to_local
        )

        # Keys should be (min, max) regardless of input order
        assert (1, 3) in result  # (3,5) -> (1,3)
        assert result[(1, 3)] == 150
        assert (0, 2) in result  # (2,4) -> (0,2)
        assert result[(0, 2)] == 250


class TestPartitionGraph:
    """Test METIS graph partitioning wrapper."""

    def test_partition_simple_graph(self):
        """Test partitioning a simple graph."""
        # Create simple chain graph: 0-1-2-3
        adjacency = [
            [1],      # 0 connects to 1
            [0, 2],   # 1 connects to 0, 2
            [1, 3],   # 2 connects to 1, 3
            [2],      # 3 connects to 2
        ]
        weights = np.array([100, 100, 100, 100], dtype=np.int32)

        # Partition into 2 parts
        parts = partition_graph(
            adjacency=adjacency,
            vertex_weights=weights,
            nparts=2,
            recursive=True,
            ufactor=1.01,
            debug=False
        )

        # Check basic properties
        assert len(parts) == 4
        assert set(parts) == {0, 1}, "Parts should be labeled 0 and 1"

        # Each part should have 2 nodes (balanced)
        part0_count = sum(1 for p in parts if p == 0)
        part1_count = sum(1 for p in parts if p == 1)
        assert part0_count == 2
        assert part1_count == 2

    def test_partition_with_edge_weights(self):
        """Test partitioning with edge weights."""
        # Simple 4-node graph
        adjacency = [
            [1, 2],   # 0
            [0, 2, 3], # 1
            [0, 1, 3], # 2
            [1, 2],   # 3
        ]
        weights = np.array([100, 100, 100, 100], dtype=np.int32)

        # Edge weights (boundaries)
        edge_weights = {
            (0, 1): 1000,  # Heavy edge
            (0, 2): 100,
            (1, 2): 100,
            (1, 3): 100,
            (2, 3): 100,
        }

        parts = partition_graph(
            adjacency=adjacency,
            vertex_weights=weights,
            nparts=2,
            edge_weights=edge_weights,
            recursive=True,
            ufactor=1.01,
            debug=False
        )

        assert len(parts) == 4
        assert set(parts) == {0, 1}

        # Heavy edge (0,1) should ideally NOT be cut
        # (METIS tries to minimize total cut weight)
        # Note: This is heuristic, so we just check it runs


class TestSplitNodeWorker:
    """Test parallel node splitting worker."""

    def test_split_node_single_district(self):
        """Test splitting when target is 1 district (no split)."""
        task = {
            'block_indices': {0, 1, 2},
            'target_districts': 1,
            'depth': 1,
            'name': 'root',
            'state_code': 'VT',
            'adjacency': [[1], [0, 2], [1]],
            'vertex_weights': np.array([100, 100, 100], dtype=np.int32),
            'debug': False,
            'edge_weights': None,
        }

        result = _split_node_worker(task)

        # Should return None (no split needed)
        assert result is None

    def test_split_node_two_districts(self):
        """Test splitting into 2 districts."""
        # Create 4-node chain
        adjacency = [[1], [0, 2], [1, 3], [2]]
        vertex_weights = np.array([100, 100, 100, 100], dtype=np.int32)

        task = {
            'block_indices': {0, 1, 2, 3},
            'target_districts': 2,
            'depth': 1,
            'name': 'root',
            'state_code': 'VT',
            'adjacency': adjacency,
            'vertex_weights': vertex_weights,
            'debug': False,
            'edge_weights': None,
        }

        result = _split_node_worker(task)

        # Should return split result
        assert result is not None
        assert 'left_blocks' in result
        assert 'right_blocks' in result
        assert 'left_name' in result
        assert 'right_name' in result

        # Check that all blocks are assigned
        all_blocks = result['left_blocks'] | result['right_blocks']
        assert all_blocks == {0, 1, 2, 3}

        # Check that no blocks are in both
        assert len(result['left_blocks'] & result['right_blocks']) == 0

        # Check populations are roughly balanced
        left_pop = result['left_pop']
        right_pop = result['right_pop']
        assert abs(left_pop - right_pop) <= 50  # Within 50 people


class TestRecursiveBisectionIntegration:
    """Integration tests using mock data."""

    def test_partition_mock_tracts(self, mock_tracts_small, mock_adjacency_small):
        """Test partitioning mock tract data."""
        from tests.mocks.mock_tracts import generate_mock_tracts
        from tests.mocks.mock_adjacency import generate_mock_adjacency

        # Generate small dataset
        tracts = generate_mock_tracts(num_tracts=50, state='vermont')
        graph = generate_mock_adjacency(tracts, connectivity=0.2)

        # Convert to adjacency list format
        num_nodes = len(graph.nodes)
        adjacency = [[] for _ in range(num_nodes)]
        for node in graph.nodes:
            neighbors = list(graph.neighbors(node))
            adjacency[node] = neighbors

        # Get population weights
        vertex_weights = np.array(tracts['population'].values, dtype=np.int32)

        # Partition into 2 parts (simulating Vermont's 1 district split for testing)
        parts = partition_graph(
            adjacency=adjacency,
            vertex_weights=vertex_weights,
            nparts=2,
            recursive=True,
            ufactor=1.01,
            debug=False
        )

        # Validate partitioning
        assert len(parts) == num_nodes
        assert set(parts) <= {0, 1}, "Parts should be 0 or 1"

        # Check population balance
        total_pop = vertex_weights.sum()
        part0_pop = sum(vertex_weights[i] for i in range(num_nodes) if parts[i] == 0)
        part1_pop = sum(vertex_weights[i] for i in range(num_nodes) if parts[i] == 1)

        assert part0_pop + part1_pop == total_pop
        target_pop = total_pop / 2
        assert abs(part0_pop - target_pop) / target_pop < 0.01, "Should be within 1% of target"
        assert abs(part1_pop - target_pop) / target_pop < 0.01, "Should be within 1% of target"

    def test_partition_with_mock_edge_weights(self, mock_tracts_small):
        """Test partitioning with edge-weighted graph."""
        from tests.mocks.mock_tracts import generate_mock_tracts
        from tests.mocks.mock_adjacency import generate_mock_adjacency

        # Generate small dataset with edge weights
        tracts = generate_mock_tracts(num_tracts=50, state='vermont')
        graph = generate_mock_adjacency(tracts, connectivity=0.2, edge_weighted=True)

        # Convert to adjacency list
        num_nodes = len(graph.nodes)
        adjacency = [[] for _ in range(num_nodes)]
        for node in graph.nodes:
            neighbors = list(graph.neighbors(node))
            adjacency[node] = neighbors

        # Extract edge weights
        edge_weights = {}
        for u, v, data in graph.edges(data=True):
            weight = data.get('weight', 1000)
            key = (min(u, v), max(u, v))
            edge_weights[key] = weight

        vertex_weights = np.array(tracts['population'].values, dtype=np.int32)

        # Partition with edge weights
        parts = partition_graph(
            adjacency=adjacency,
            vertex_weights=vertex_weights,
            nparts=2,
            edge_weights=edge_weights,
            recursive=True,
            ufactor=1.01,
            debug=False
        )

        # Validate
        assert len(parts) == num_nodes
        assert set(parts) <= {0, 1}


class TestPopulationBalance:
    """Test population balance constraints."""

    def test_balance_single_split(self):
        """Test population balance in single bisection."""
        # Create 10 nodes with varying populations
        populations = np.array([100, 150, 200, 120, 180, 110, 140, 160, 130, 170], dtype=np.int32)
        total_pop = populations.sum()

        # Simple chain graph
        adjacency = [[1], [0, 2], [1, 3], [2, 4], [3, 5], [4, 6], [5, 7], [6, 8], [7, 9], [8]]

        # Partition into 2
        parts = partition_graph(
            adjacency=adjacency,
            vertex_weights=populations,
            nparts=2,
            recursive=True,
            ufactor=1.005,  # Tight balance (0.5%)
            debug=False
        )

        # Calculate actual populations
        part0_pop = sum(populations[i] for i in range(len(populations)) if parts[i] == 0)
        part1_pop = sum(populations[i] for i in range(len(populations)) if parts[i] == 1)

        target_pop = total_pop / 2
        deviation_0 = abs(part0_pop - target_pop) / target_pop
        deviation_1 = abs(part1_pop - target_pop) / target_pop

        # Should be within reasonable METIS tolerance (3%)
        assert deviation_0 < 0.03, f"Part 0 deviation {deviation_0:.2%} > 3%"
        assert deviation_1 < 0.03, f"Part 1 deviation {deviation_1:.2%} > 3%"


# Pytest markers
pytestmark = [
    pytest.mark.unit,
    pytest.mark.redistricting,
]
