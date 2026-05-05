"""
Unit tests for VRA target weight calculation (vra_targets.py)

Tests the bottom-up target calculation algorithm for concentrating
minority populations in recursive bisection.

NOTE: Skipped — imports src.apportionment.partition.vra_targets which was
archived on 2026-04-29 (archive/python-pipeline-final/).  VRA targets are
now computed in Rust (redist-analysis::vra).
"""
import pytest
pytestmark = pytest.mark.skip(reason="apportionment.partition archived 2026-04-29")
"""

import pytest
from src.apportionment.partition.vra_targets import (
    calculate_final_district_targets,
    build_binary_tree_structure,
    calculate_node_targets,
    create_vra_target_tree,
    get_target_weights_for_node
)


class TestCalculateFinalDistrictTargets:
    """Test final district target calculation."""

    def test_alabama_2_mm_districts(self):
        """Test Alabama configuration: 7 districts, 2 MM, 36.9% minority."""
        targets = calculate_final_district_targets(
            num_districts=7,
            target_mm_districts=2,
            state_minority_pct=0.369,
            mm_target_pct=0.60
        )

        assert len(targets) == 7

        # First 2 districts should be MM at 60%
        assert targets[0]['minority_pct_of_district'] == pytest.approx(0.60)
        assert targets[1]['minority_pct_of_district'] == pytest.approx(0.60)

        # Remaining 5 districts should have lower minority %
        for i in range(2, 7):
            assert targets[i]['minority_pct_of_district'] < 0.50

        # Total minority should sum to state percentage
        total_minority = sum(t['minority_fraction_of_state'] for t in targets)
        assert total_minority == pytest.approx(0.369, abs=0.001)

        # All districts should have equal population
        for t in targets:
            assert t['pop_fraction'] == pytest.approx(1.0 / 7)

    def test_single_mm_district(self):
        """Test configuration with only 1 MM district."""
        targets = calculate_final_district_targets(
            num_districts=5,
            target_mm_districts=1,
            state_minority_pct=0.30,
            mm_target_pct=0.55
        )

        assert len(targets) == 5
        assert targets[0]['minority_pct_of_district'] == pytest.approx(0.55)

        # Check conservation of minority population
        total_minority = sum(t['minority_fraction_of_state'] for t in targets)
        assert total_minority == pytest.approx(0.30, abs=0.001)

    def test_no_mm_districts(self):
        """Test configuration with 0 MM districts (baseline mode)."""
        targets = calculate_final_district_targets(
            num_districts=5,
            target_mm_districts=0,
            state_minority_pct=0.20,
            mm_target_pct=0.60
        )

        assert len(targets) == 5

        # All districts should have equal minority percentage
        for t in targets:
            assert t['minority_pct_of_district'] == pytest.approx(0.20)

    def test_all_mm_districts(self):
        """Test edge case: all districts are MM."""
        targets = calculate_final_district_targets(
            num_districts=3,
            target_mm_districts=3,
            state_minority_pct=0.75,
            mm_target_pct=0.60
        )

        assert len(targets) == 3

        # All districts should be 60% minority (can't use all 75% at 60% each)
        for t in targets:
            assert t['minority_pct_of_district'] == pytest.approx(0.60)


class TestBuildBinaryTreeStructure:
    """Test binary tree structure building."""

    def test_7_districts(self):
        """Test Alabama tree: 7 -> [3, 4]."""
        tree = build_binary_tree_structure(7)

        assert tree['districts'] == 7
        assert tree['depth'] == 0
        assert tree['is_leaf'] is False
        assert tree['left_count'] == 3
        assert tree['right_count'] == 4

        # Check left subtree: 3 -> [1, 2]
        left = tree['left']
        assert left['districts'] == 3
        assert left['left_count'] == 1
        assert left['right_count'] == 2

        # Check right subtree: 4 -> [2, 2]
        right = tree['right']
        assert right['districts'] == 4
        assert right['left_count'] == 2
        assert right['right_count'] == 2

    def test_single_district(self):
        """Test leaf node: 1 district."""
        tree = build_binary_tree_structure(1)

        assert tree['districts'] == 1
        assert tree['is_leaf'] is True
        assert 'left' not in tree
        assert 'right' not in tree

    def test_power_of_2(self):
        """Test balanced tree: 8 districts."""
        tree = build_binary_tree_structure(8)

        assert tree['districts'] == 8
        assert tree['left_count'] == 4
        assert tree['right_count'] == 4

        # Both subtrees should be balanced
        assert tree['left']['left_count'] == 2
        assert tree['left']['right_count'] == 2
        assert tree['right']['left_count'] == 2
        assert tree['right']['right_count'] == 2

    def test_binary_paths(self):
        """Test binary path encoding."""
        tree = build_binary_tree_structure(4)

        # Root has empty path
        assert tree['path'] == ""

        # First level: "0" and "1"
        assert tree['left']['path'] == "0"
        assert tree['right']['path'] == "1"

        # Second level: "00", "01", "10", "11"
        assert tree['left']['left']['path'] == "00"
        assert tree['left']['right']['path'] == "01"
        assert tree['right']['left']['path'] == "10"
        assert tree['right']['right']['path'] == "11"


class TestCalculateNodeTargets:
    """Test bottom-up target calculation."""

    def test_alabama_root_targets(self):
        """Test that root split concentrates minorities on left."""
        # Create tree
        district_targets = calculate_final_district_targets(7, 2, 0.369, 0.60)
        tree = build_binary_tree_structure(7)

        # Assign districts (MM districts go left)
        from src.apportionment.partition.vra_targets import assign_districts_to_tree
        tree = assign_districts_to_tree(tree, district_targets, mm_clustering='left')

        # Calculate targets
        tree = calculate_node_targets(tree, 0.369)

        # Root should concentrate minorities on left
        root_targets = tree['target_weights']
        left_pop, left_minority = root_targets[0]
        right_pop, right_minority = root_targets[1]

        # Left gets 3/7 districts but more than 3/7 of minorities
        assert left_pop == pytest.approx(3.0 / 7.0)
        assert left_minority > left_pop  # More minority than proportional

        # Right gets 4/7 districts but less than 4/7 of minorities
        assert right_pop == pytest.approx(4.0 / 7.0)
        assert right_minority < right_pop  # Less minority than proportional

        # Total should sum to 1.0
        assert left_pop + right_pop == pytest.approx(1.0)
        assert left_minority + right_minority == pytest.approx(1.0)

    def test_balanced_distribution(self):
        """Test that non-VRA mode produces balanced targets."""
        # No MM districts: all districts have equal minority %
        district_targets = calculate_final_district_targets(4, 0, 0.25, 0.60)
        tree = build_binary_tree_structure(4)

        from src.apportionment.partition.vra_targets import assign_districts_to_tree
        tree = assign_districts_to_tree(tree, district_targets)
        tree = calculate_node_targets(tree, 0.25)

        # Root should have proportional targets
        root_targets = tree['target_weights']
        left_pop, left_minority = root_targets[0]
        right_pop, right_minority = root_targets[1]

        # Both should be 0.5 (balanced)
        assert left_pop == pytest.approx(0.5)
        assert right_pop == pytest.approx(0.5)
        assert left_minority == pytest.approx(0.5)
        assert right_minority == pytest.approx(0.5)


class TestCreateVRATargetTree:
    """Test complete VRA target tree creation (integration)."""

    def test_alabama_complete_tree(self):
        """Test complete Alabama VRA target tree."""
        tree = create_vra_target_tree(
            num_districts=7,
            target_mm_districts=2,
            state_minority_pct=0.369,
            mm_target_pct=0.60
        )

        # Check root targets
        assert 'target_weights' in tree
        root_targets = tree['target_weights']

        # Left branch (3 districts, 2 MM) should get more minority
        left_pop, left_minority = root_targets[0]
        right_pop, right_minority = root_targets[1]

        assert left_pop == pytest.approx(3.0 / 7.0, abs=0.01)
        assert left_minority > 0.5  # More than half of minorities go left
        assert right_minority < 0.5  # Less than half go right

        # Check tree structure
        assert tree['districts'] == 7
        assert tree['left_count'] == 3
        assert tree['right_count'] == 4

    def test_simple_case_2_districts_1_mm(self):
        """Test simplest case: 2 districts, 1 MM."""
        tree = create_vra_target_tree(
            num_districts=2,
            target_mm_districts=1,
            state_minority_pct=0.40,
            mm_target_pct=0.60
        )

        root_targets = tree['target_weights']
        left_pop, left_minority = root_targets[0]
        right_pop, right_minority = root_targets[1]

        # Both get 50% population
        assert left_pop == pytest.approx(0.5)
        assert right_pop == pytest.approx(0.5)

        # Left (MM district) gets more minority
        assert left_minority > right_minority

        # Left should get 60% * 50% = 30% of state minorities
        # Right should get 40% - 30% = 10% of state minorities
        # So split is 30% / 10% = 75% / 25%
        assert left_minority == pytest.approx(0.75, abs=0.01)
        assert right_minority == pytest.approx(0.25, abs=0.01)


class TestGetTargetWeightsForNode:
    """Test target weight lookup by node path."""

    def test_root_node_lookup(self):
        """Test looking up root node (7 districts, path='')."""
        tree = create_vra_target_tree(7, 2, 0.369, 0.60)

        targets = get_target_weights_for_node(tree, 7, path="")

        assert targets is not None
        assert len(targets) == 2  # Left and right
        assert len(targets[0]) == 2  # [pop, minority]
        assert len(targets[1]) == 2

    def test_left_subtree_lookup(self):
        """Test looking up left subtree (3 districts, path='0')."""
        tree = create_vra_target_tree(7, 2, 0.369, 0.60)

        targets = get_target_weights_for_node(tree, 3, path="0")

        assert targets is not None
        # Left subtree has 2 MM districts, so left child (1 dist) and right child (2 dist)
        # Left child (1 dist) should be MM, so gets higher minority target

    def test_leaf_node_has_no_targets(self):
        """Test that leaf nodes return district targets, not split targets."""
        tree = create_vra_target_tree(7, 2, 0.369, 0.60)

        # Leaf node (1 district) has no split targets
        targets = get_target_weights_for_node(tree, 1, path="00")

        # Should return None or leaf-specific format (depending on implementation)
        # Leaf nodes don't split further, so no target_weights for splitting


class TestVRATargetConsistency:
    """Test mathematical consistency of VRA targets."""

    def test_minority_conservation(self):
        """Test that total minority population is conserved."""
        for num_districts in [3, 5, 7, 9, 13]:
            for target_mm in range(min(3, num_districts)):
                targets = calculate_final_district_targets(
                    num_districts=num_districts,
                    target_mm_districts=target_mm,
                    state_minority_pct=0.35,
                    mm_target_pct=0.60
                )

                total_minority = sum(t['minority_fraction_of_state'] for t in targets)
                assert total_minority == pytest.approx(0.35, abs=0.001), \
                    f"Failed for {num_districts} districts, {target_mm} MM"

    def test_population_balance(self):
        """Test that population is evenly distributed."""
        targets = calculate_final_district_targets(7, 2, 0.369, 0.60)

        for t in targets:
            assert t['pop_fraction'] == pytest.approx(1.0 / 7, abs=0.001)

    def test_target_weights_sum_to_one(self):
        """Test that target weights at each node sum to 1.0."""
        tree = create_vra_target_tree(7, 2, 0.369, 0.60)

        def check_node(node):
            if node.get('is_leaf', False):
                return

            weights = node.get('target_weights')
            if weights is not None:
                left_pop, left_minority = weights[0]
                right_pop, right_minority = weights[1]

                # Population targets should sum to 1.0
                assert left_pop + right_pop == pytest.approx(1.0)

                # Minority targets should sum to 1.0
                assert left_minority + right_minority == pytest.approx(1.0)

            # Recurse
            if 'left' in node:
                check_node(node['left'])
            if 'right' in node:
                check_node(node['right'])

        check_node(tree)
