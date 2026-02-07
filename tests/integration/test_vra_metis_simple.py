"""
Integration test for VRA multi-constraint METIS partitioning.

Tests if METIS can concentrate minorities using multi-constraint partitioning
with a simple, controlled test case.
"""

import pytest
import numpy as np
from src.apportionment.partition.metis_wrapper import partition_graph
from src.apportionment.partition.vra_targets import create_vra_target_tree


class TestVRAMETISSimple:
    """Test METIS multi-constraint with simple controlled cases."""

    def test_simple_linear_case_2_districts(self):
        """
        Test simplest case: 10 tracts in a line, 2 districts.

        Tract layout (minority %):
        [80% 80% 80% 80% 80% 20% 20% 20% 20% 20%]
         |---- High minority ----|---- Low minority ----|

        Goal: Concentrate high-minority tracts into District 1

        Expected: District 1 gets tracts 0-4 (80% minority)
                  District 2 gets tracts 5-9 (20% minority)
        """
        # Create linear adjacency: each tract connects to neighbors
        adjacency = [
            [1],        # 0 connects to 1
            [0, 2],     # 1 connects to 0, 2
            [1, 3],     # etc.
            [2, 4],
            [3, 5],
            [4, 6],
            [5, 7],
            [6, 8],
            [7, 9],
            [8],        # 9 connects to 8
        ]

        # Create 2D vertex weights: [population, minority_vap]
        # High minority tracts: 1000 pop, 800 minority (80%)
        # Low minority tracts: 1000 pop, 200 minority (20%)
        vertex_weights = np.array([
            [1000, 800],  # Tract 0: 80% minority
            [1000, 800],  # Tract 1: 80% minority
            [1000, 800],  # Tract 2: 80% minority
            [1000, 800],  # Tract 3: 80% minority
            [1000, 800],  # Tract 4: 80% minority
            [1000, 200],  # Tract 5: 20% minority
            [1000, 200],  # Tract 6: 20% minority
            [1000, 200],  # Tract 7: 20% minority
            [1000, 200],  # Tract 8: 20% minority
            [1000, 200],  # Tract 9: 20% minority
        ])

        # Overall: 50% minority, want 1 MM district at 80%
        # Calculate VRA target weights
        vra_tree = create_vra_target_tree(
            num_districts=2,
            target_mm_districts=1,
            state_minority_pct=0.50,
            mm_target_pct=0.80
        )

        # Get root targets
        target_weights = vra_tree['target_weights']

        print(f"\nVRA Target Weights:")
        print(f"  Left (MM district):  pop={target_weights[0][0]:.3f}, minority={target_weights[0][1]:.3f}")
        print(f"  Right (non-MM dist): pop={target_weights[1][0]:.3f}, minority={target_weights[1][1]:.3f}")

        # Partition with multi-constraint
        parts = partition_graph(
            adjacency=adjacency,
            vertex_weights=vertex_weights,
            nparts=2,
            target_weights=target_weights,
            multi_constraint=True,
            debug=True
        )

        # Analyze results
        partition_0_tracts = [i for i, p in enumerate(parts) if p == 0]
        partition_1_tracts = [i for i, p in enumerate(parts) if p == 1]

        # Calculate minority percentages
        def calc_minority_pct(tracts):
            total_pop = sum(vertex_weights[i, 0] for i in tracts)
            total_minority = sum(vertex_weights[i, 1] for i in tracts)
            return total_minority / total_pop if total_pop > 0 else 0

        minority_pct_0 = calc_minority_pct(partition_0_tracts)
        minority_pct_1 = calc_minority_pct(partition_1_tracts)

        print(f"\nResults:")
        print(f"  Partition 0: tracts {partition_0_tracts}, {minority_pct_0*100:.1f}% minority")
        print(f"  Partition 1: tracts {partition_1_tracts}, {minority_pct_1*100:.1f}% minority")

        # Check if one partition is MM (>50%)
        has_mm_district = minority_pct_0 > 0.5 or minority_pct_1 > 0.5

        print(f"\nMM District Created: {has_mm_district}")

        # The test passes if we create at least one MM district
        # Ideally, one should be ~80% and one should be ~20%
        assert has_mm_district, "METIS failed to create MM district in simple case"

        # Check if high concentration was achieved
        max_minority = max(minority_pct_0, minority_pct_1)
        print(f"  Highest minority concentration: {max_minority*100:.1f}%")

        # Ideally should be close to 80%
        assert max_minority > 0.65, f"METIS only achieved {max_minority*100:.1f}% minority (target: 80%)"

    def test_simple_without_target_weights(self):
        """
        Baseline test: same layout without VRA target weights.

        This should produce balanced districts (~50% minority each).
        """
        adjacency = [
            [1], [0, 2], [1, 3], [2, 4], [3, 5],
            [4, 6], [5, 7], [6, 8], [7, 9], [8]
        ]

        vertex_weights = np.array([
            [1000, 800], [1000, 800], [1000, 800], [1000, 800], [1000, 800],
            [1000, 200], [1000, 200], [1000, 200], [1000, 200], [1000, 200],
        ])

        # NO target weights (baseline)
        parts = partition_graph(
            adjacency=adjacency,
            vertex_weights=vertex_weights,
            nparts=2,
            multi_constraint=False,  # Use single-constraint
            debug=True
        )

        # Analyze results
        partition_0_tracts = [i for i, p in enumerate(parts) if p == 0]
        partition_1_tracts = [i for i, p in enumerate(parts) if p == 1]

        def calc_minority_pct(tracts):
            total_pop = sum(vertex_weights[i, 0] for i in tracts)
            total_minority = sum(vertex_weights[i, 1] for i in tracts)
            return total_minority / total_pop if total_pop > 0 else 0

        minority_pct_0 = calc_minority_pct(partition_0_tracts)
        minority_pct_1 = calc_minority_pct(partition_1_tracts)

        print(f"\nBaseline (no VRA):")
        print(f"  Partition 0: {minority_pct_0*100:.1f}% minority")
        print(f"  Partition 1: {minority_pct_1*100:.1f}% minority")

        # Without VRA, should be relatively balanced
        # (METIS will split at the boundary between high/low minority tracts)

    def test_strongly_separated_case(self):
        """
        Test case with strongly separated minority populations.

        Layout: Two clusters connected by one edge
        Cluster A: 5 tracts, 80% minority, fully connected
        Cluster B: 5 tracts, 20% minority, fully connected
        Bridge: single edge between cluster A and B
        """
        # Create clustered adjacency
        adjacency = [
            # Cluster A (high minority): fully connected mesh
            [1, 2, 3, 4],     # 0
            [0, 2, 3, 4],     # 1
            [0, 1, 3, 4],     # 2
            [0, 1, 2, 4],     # 3
            [0, 1, 2, 3, 5],  # 4 - bridge to cluster B
            # Cluster B (low minority): fully connected mesh
            [4, 6, 7, 8, 9],  # 5 - bridge from cluster A
            [5, 7, 8, 9],     # 6
            [5, 6, 8, 9],     # 7
            [5, 6, 7, 9],     # 8
            [5, 6, 7, 8],     # 9
        ]

        vertex_weights = np.array([
            [1000, 800], [1000, 800], [1000, 800], [1000, 800], [1000, 800],
            [1000, 200], [1000, 200], [1000, 200], [1000, 200], [1000, 200],
        ])

        vra_tree = create_vra_target_tree(2, 1, 0.50, 0.80)
        target_weights = vra_tree['target_weights']

        parts = partition_graph(
            adjacency=adjacency,
            vertex_weights=vertex_weights,
            nparts=2,
            target_weights=target_weights,
            multi_constraint=True,
            debug=True
        )

        # Analyze
        partition_0_tracts = [i for i, p in enumerate(parts) if p == 0]
        partition_1_tracts = [i for i, p in enumerate(parts) if p == 1]

        def calc_minority_pct(tracts):
            total_pop = sum(vertex_weights[i, 0] for i in tracts)
            total_minority = sum(vertex_weights[i, 1] for i in tracts)
            return total_minority / total_pop if total_pop > 0 else 0

        minority_pct_0 = calc_minority_pct(partition_0_tracts)
        minority_pct_1 = calc_minority_pct(partition_1_tracts)

        print(f"\nStrongly Separated Case:")
        print(f"  Partition 0: tracts {partition_0_tracts}, {minority_pct_0*100:.1f}% minority")
        print(f"  Partition 1: tracts {partition_1_tracts}, {minority_pct_1*100:.1f}% minority")

        # This case should DEFINITELY work - natural cut separates clusters
        has_mm_district = minority_pct_0 > 0.5 or minority_pct_1 > 0.5
        assert has_mm_district, "METIS failed even with strongly separated clusters!"

        # Should achieve high concentration
        max_minority = max(minority_pct_0, minority_pct_1)
        assert max_minority > 0.70, f"Only achieved {max_minority*100:.1f}% with separated clusters"
