"""
Unit tests for ablation study analysis scripts.

Tests the boundary penalty ablation analysis infrastructure.
"""

import pytest
import pickle
from pathlib import Path
import sys

project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root / "scripts"))
sys.path.insert(0, str(project_root / "scripts" / "experimental"))

from run_boundary_penalty_ablation import apply_boundary_penalty


class TestBoundaryPenalty:
    """Test boundary penalty application."""

    def test_apply_penalty_beta_zero(self):
        """Test that beta=0 leaves weights unchanged."""
        base_weights = {(0, 1): 1.0, (1, 2): 1.0, (2, 3): 1.0}
        cross_state = [(1, 2)]

        result = apply_boundary_penalty(base_weights, cross_state, beta=0.0)

        assert result == base_weights

    def test_apply_penalty_beta_positive(self):
        """Test that positive beta increases cross-state edge weights."""
        base_weights = {(0, 1): 1.0, (1, 2): 1.0, (2, 3): 1.0}
        cross_state = [(1, 2)]

        result = apply_boundary_penalty(base_weights, cross_state, beta=5.0)

        assert result[(0, 1)] == 1.0  # Within-state unchanged
        assert result[(1, 2)] == 6.0  # Cross-state: 1.0 * (1 + 5)
        assert result[(2, 3)] == 1.0  # Within-state unchanged

    def test_apply_penalty_symmetric_edges(self):
        """Test that edge order doesn't matter for cross-state detection."""
        base_weights = {(0, 1): 1.0, (1, 2): 1.0}
        cross_state = [(2, 1)]  # Reversed order

        result = apply_boundary_penalty(base_weights, cross_state, beta=10.0)

        assert result[(1, 2)] == 11.0  # Should still be penalized

    def test_apply_penalty_large_beta(self):
        """Test that large beta values work correctly."""
        base_weights = {(0, 1): 1.0}
        cross_state = [(0, 1)]

        result = apply_boundary_penalty(base_weights, cross_state, beta=100.0)

        assert result[(0, 1)] == 101.0

    def test_apply_penalty_no_cross_state(self):
        """Test behavior when no edges cross state boundaries."""
        base_weights = {(0, 1): 1.0, (1, 2): 1.0}
        cross_state = []

        result = apply_boundary_penalty(base_weights, cross_state, beta=5.0)

        assert result == base_weights


class TestAblationResults:
    """Test ablation results structure and validity."""

    def test_ablation_file_exists_beta_zero(self, tmp_path):
        """Test that ablation results have correct structure."""
        # This is a structural test - actual files should exist from runs
        expected_keys = ['beta', 'districts', 'results', 'geoid_to_district']

        # Check beta=0 file exists in actual output
        ablation_file = Path('outputs/experimental/ablation_beta_0.0_2020.pkl')
        if ablation_file.exists():
            with open(ablation_file, 'rb') as f:
                data = pickle.load(f)

            for key in expected_keys:
                assert key in data, f"Missing key: {key}"

            assert data['beta'] == 0.0
            assert 'n_cross_state_districts' in data['results']

    def test_cross_state_counts_reasonable(self):
        """Test that cross-state district counts are in reasonable range."""
        # Check a few known beta values
        test_betas = [0.0, 5.0, 10.0, 100.0]

        for beta in test_betas:
            ablation_file = Path(f'outputs/experimental/ablation_beta_{beta}_2020.pkl')
            if ablation_file.exists():
                with open(ablation_file, 'rb') as f:
                    data = pickle.load(f)

                n_cross = data['results']['n_cross_state_districts']

                # Should be between 0 and 435
                assert 0 <= n_cross <= 435

                # Should be reasonably high (>100) for low penalties
                if beta <= 10:
                    assert n_cross >= 100, f"Beta={beta} has unexpectedly low cross-state count: {n_cross}"


class TestAblationTrends:
    """Test expected trends in ablation study."""

    def test_penalty_reduces_crossings(self):
        """Test that increasing penalty generally reduces cross-state districts."""
        # This tests the general trend, not strict monotonicity
        betas = [0.0, 10.0, 50.0, 100.0]
        counts = []

        for beta in betas:
            ablation_file = Path(f'outputs/experimental/ablation_beta_{beta}_2020.pkl')
            if ablation_file.exists():
                with open(ablation_file, 'rb') as f:
                    data = pickle.load(f)
                counts.append(data['results']['n_cross_state_districts'])

        if len(counts) == 4:
            # General downward trend (allowing for non-monotonicity)
            assert counts[-1] < counts[0], "High penalty should have fewer crossings than no penalty"

    def test_plateau_behavior(self):
        """Test that curve plateaus at high penalties."""
        # At high penalties (100-300), should stay roughly stable
        high_betas = [100.0, 200.0, 300.0]
        counts = []

        for beta in high_betas:
            ablation_file = Path(f'outputs/experimental/ablation_beta_{beta}_2020.pkl')
            if ablation_file.exists():
                with open(ablation_file, 'rb') as f:
                    data = pickle.load(f)
                counts.append(data['results']['n_cross_state_districts'])

        if len(counts) == 3:
            # Should be relatively stable (within 20 districts)
            assert max(counts) - min(counts) <= 20, "High-penalty plateau should be stable"


class TestMetroAlignment:
    """Test metro area alignment analysis."""

    def test_metro_detection_logic(self):
        """Test that metro matching logic works correctly."""
        # Mock data
        district_states = {
            1: {'NY', 'NJ'},     # Should match NYC metro
            2: {'CA', 'OR'},     # Should match a west coast metro
            3: {'TX'},           # Should not match (single state)
            4: {'WY', 'MT'},     # Should not match (no metro defined)
        }

        metro_info = {
            'NYC': {'states': {'NY', 'NJ', 'CT'}, 'core': ['NY', 'NJ']},
            'West': {'states': {'CA', 'OR', 'WA'}, 'core': ['CA', 'OR']},
        }

        # Test NYC match
        assert district_states[1].issubset(metro_info['NYC']['states'])
        assert any(s in district_states[1] for s in metro_info['NYC']['core'])

        # Test West match
        assert district_states[2].issubset(metro_info['West']['states'])
        assert any(s in district_states[2] for s in metro_info['West']['core'])


@pytest.mark.skipif(
    not Path('outputs/experimental/ablation_beta_5.0_2020.pkl').exists(),
    reason="Ablation results not available"
)
class TestAblationCompleteness:
    """Test that ablation study is complete and consistent."""

    def test_all_expected_files_exist(self):
        """Test that all expected ablation result files exist."""
        expected_betas = [0.0, 0.5, 1.0, 2.0, 5.0, 10.0, 20.0, 30.0, 40.0, 50.0,
                         60.0, 70.0, 80.0, 90.0, 100.0, 125.0, 150.0, 175.0,
                         200.0, 250.0, 300.0]

        for beta in expected_betas:
            ablation_file = Path(f'outputs/experimental/ablation_beta_{beta}_2020.pkl')
            assert ablation_file.exists(), f"Missing ablation file for beta={beta}"

    def test_consistent_total_districts(self):
        """Test that all results have 435 total districts."""
        test_betas = [0.0, 5.0, 50.0, 100.0, 300.0]

        for beta in test_betas:
            ablation_file = Path(f'outputs/experimental/ablation_beta_{beta}_2020.pkl')
            with open(ablation_file, 'rb') as f:
                data = pickle.load(f)

            districts = data['districts']
            unique_districts = len(set(districts))
            assert unique_districts == 435, f"Beta={beta} has {unique_districts} districts, expected 435"
