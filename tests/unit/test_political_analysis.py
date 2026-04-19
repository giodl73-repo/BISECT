"""
Unit tests for political analysis functionality.

Tests partisan lean calculation, vote aggregation, and district-level
political metrics.
"""

import pytest
import pandas as pd
import numpy as np
from pathlib import Path
import sys

# Add project root to path
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))


class TestPoliticalDataGeneration:
    """Test political analysis data generation."""

    def test_generate_political_analysis(self, mock_tracts_medium, mock_districts_medium):
        """Test political analysis generation for districts."""
        from tests.mocks.mock_analysis import generate_mock_political_analysis

        political = generate_mock_political_analysis(mock_districts_medium, state='alabama')

        # Check structure
        assert len(political) == 7, "Should have 7 districts"
        assert 'district' in political.columns
        assert 'dem_votes' in political.columns
        assert 'rep_votes' in political.columns
        assert 'dem_share' in political.columns
        assert 'rep_share' in political.columns
        assert 'margin' in political.columns
        assert 'winner' in political.columns

    def test_political_vote_shares_sum_to_one(self, mock_tracts_medium, mock_districts_medium):
        """Test that D and R vote shares sum to 1.0."""
        from tests.mocks.mock_analysis import generate_mock_political_analysis

        political = generate_mock_political_analysis(mock_districts_medium, state='alabama')

        # Check shares sum to ~1.0
        share_sums = political['dem_share'] + political['rep_share']
        assert (abs(share_sums - 1.0) < 0.01).all(), "D + R shares must sum to 1.0"

    def test_political_winner_matches_votes(self, mock_tracts_medium, mock_districts_medium):
        """Test that winner matches vote totals."""
        from tests.mocks.mock_analysis import generate_mock_political_analysis

        political = generate_mock_political_analysis(mock_districts_medium, state='alabama')

        for _, row in political.iterrows():
            if row['dem_votes'] > row['rep_votes']:
                assert row['winner'] == 'D', f"District {row['district']}: D has more votes"
            else:
                assert row['winner'] == 'R', f"District {row['district']}: R has more votes"

    def test_political_margin_calculation(self, mock_tracts_medium, mock_districts_medium):
        """Test that margin is calculated correctly."""
        from tests.mocks.mock_analysis import generate_mock_political_analysis

        political = generate_mock_political_analysis(mock_districts_medium, state='alabama')

        for _, row in political.iterrows():
            expected_margin = row['dem_share'] - row['rep_share']
            assert abs(row['margin'] - expected_margin) < 0.001, \
                f"Margin mismatch in district {row['district']}"


class TestPartisanLeanDistribution:
    """Test partisan lean calculations and distributions."""

    def test_red_state_lean(self):
        """Test that red states have R advantage."""
        from tests.mocks.mock_tracts import generate_mock_tracts
        from tests.mocks.mock_districts import generate_mock_districts
        from tests.mocks.mock_analysis import generate_mock_political_analysis

        tracts = generate_mock_tracts(num_tracts=200, state='alabama')  # Red state
        districts = generate_mock_districts(tracts, num_districts=7)
        political = generate_mock_political_analysis(districts, state='alabama')

        # Red state should have more R seats
        r_seats = (political['winner'] == 'R').sum()
        d_seats = (political['winner'] == 'D').sum()
        assert r_seats >= d_seats, "Red state should have more R seats"

        # Mean D share should be < 50%
        mean_d_share = political['dem_share'].mean()
        assert mean_d_share < 0.50, f"Red state mean D share {mean_d_share:.2%} should be < 50%"

    def test_blue_state_lean(self):
        """Test that blue states have D advantage."""
        from tests.mocks.mock_tracts import generate_mock_tracts
        from tests.mocks.mock_districts import generate_mock_districts
        from tests.mocks.mock_analysis import generate_mock_political_analysis

        tracts = generate_mock_tracts(num_tracts=50, state='vermont')  # Blue state
        districts = generate_mock_districts(tracts, num_districts=1)
        political = generate_mock_political_analysis(districts, state='vermont')

        # Blue state should favor D
        mean_d_share = political['dem_share'].mean()
        assert mean_d_share > 0.50, f"Blue state mean D share {mean_d_share:.2%} should be > 50%"


class TestSeatCalculation:
    """Test seat count calculations."""

    def test_total_seats_equals_districts(self, mock_tracts_medium, mock_districts_medium):
        """Test that D + R seats equals total districts."""
        from tests.mocks.mock_analysis import generate_mock_political_analysis

        political = generate_mock_political_analysis(mock_districts_medium, state='alabama')

        d_seats = (political['winner'] == 'D').sum()
        r_seats = (political['winner'] == 'R').sum()
        total_districts = len(political)

        assert d_seats + r_seats == total_districts, "All districts must have winner"

    def test_competitive_margin_identification(self, mock_tracts_medium, mock_districts_medium):
        """Test identification of competitive districts."""
        from tests.mocks.mock_analysis import generate_mock_political_analysis

        political = generate_mock_political_analysis(mock_districts_medium, state='alabama')

        # Competitive: margin < 5%
        competitive = political[abs(political['margin']) < 0.05]

        # Should have district numbers
        assert 'district' in competitive.columns
        assert all(competitive['district'] >= 1)
        assert all(competitive['district'] <= len(political))


class TestElectionDataAggregation:
    """Test vote aggregation from tract to district level."""

    def test_aggregate_votes_to_districts(self):
        """Test aggregating tract-level votes to district level."""
        # Create mock tract-level vote data
        tract_votes = pd.DataFrame({
            'GEOID': ['01001000100', '01001000200', '01001000300', '01001000400'],
            'district': [1, 1, 2, 2],
            'dem_votes': [1000, 1500, 1200, 1300],
            'rep_votes': [800, 700, 1400, 1100],
        })

        # Aggregate to district level
        district_votes = tract_votes.groupby('district').agg({
            'dem_votes': 'sum',
            'rep_votes': 'sum',
        }).reset_index()

        # Calculate shares
        district_votes['total_votes'] = district_votes['dem_votes'] + district_votes['rep_votes']
        district_votes['dem_share'] = district_votes['dem_votes'] / district_votes['total_votes']
        district_votes['rep_share'] = district_votes['rep_votes'] / district_votes['total_votes']

        # Check District 1
        d1 = district_votes[district_votes['district'] == 1].iloc[0]
        assert d1['dem_votes'] == 2500  # 1000 + 1500
        assert d1['rep_votes'] == 1500  # 800 + 700
        assert abs(d1['dem_share'] - (2500 / 4000)) < 0.001

        # Check District 2
        d2 = district_votes[district_votes['district'] == 2].iloc[0]
        assert d2['dem_votes'] == 2500  # 1200 + 1300
        assert d2['rep_votes'] == 2500  # 1400 + 1100
        assert abs(d2['dem_share'] - 0.5) < 0.001


class TestPoliticalMetrics:
    """Test political metric calculations."""

    def test_efficiency_gap_calculation(self):
        """Test efficiency gap calculation."""
        # Mock district results
        results = pd.DataFrame({
            'district': [1, 2, 3],
            'dem_votes': [6000, 4000, 4500],
            'rep_votes': [4000, 6000, 5500],
        })

        results['total_votes'] = results['dem_votes'] + results['rep_votes']
        results['winner'] = results['dem_votes'] > results['rep_votes']

        # Calculate wasted votes
        def wasted_votes(row):
            winning_votes = max(row['dem_votes'], row['rep_votes'])
            losing_votes = min(row['dem_votes'], row['rep_votes'])
            threshold = row['total_votes'] / 2

            if row['dem_votes'] > row['rep_votes']:
                # D won - D wasted: votes above threshold, R wasted: all votes
                d_wasted = winning_votes - threshold
                r_wasted = losing_votes
            else:
                # R won
                r_wasted = winning_votes - threshold
                d_wasted = losing_votes

            return d_wasted, r_wasted

        results[['dem_wasted', 'rep_wasted']] = results.apply(
            wasted_votes, axis=1, result_type='expand'
        )

        # Efficiency gap = (R_wasted - D_wasted) / total_votes
        total_votes = results['total_votes'].sum()
        total_d_wasted = results['dem_wasted'].sum()
        total_r_wasted = results['rep_wasted'].sum()
        efficiency_gap = (total_r_wasted - total_d_wasted) / total_votes

        # Check that calculation produces reasonable result
        assert -1.0 <= efficiency_gap <= 1.0, "Efficiency gap must be in [-1, 1]"

    def test_mean_median_difference(self, mock_tracts_medium, mock_districts_medium):
        """Test mean-median difference calculation."""
        from tests.mocks.mock_analysis import generate_mock_political_analysis

        political = generate_mock_political_analysis(mock_districts_medium, state='alabama')

        mean_dem_share = political['dem_share'].mean()
        median_dem_share = political['dem_share'].median()

        # Mean-median difference
        mm_diff = mean_dem_share - median_dem_share

        # Should be reasonable value
        assert -0.2 <= mm_diff <= 0.2, "Mean-median diff should be reasonable"


class TestPoliticalDataValidation:
    """Test validation of political analysis data."""

    def test_validate_political_columns(self, mock_tracts_medium, mock_districts_medium):
        """Test that all required columns are present."""
        from tests.mocks.mock_analysis import generate_mock_political_analysis

        political = generate_mock_political_analysis(mock_districts_medium, state='alabama')

        required_cols = ['district', 'dem_votes', 'rep_votes', 'dem_share', 'rep_share', 'margin', 'winner']
        for col in required_cols:
            assert col in political.columns, f"Missing required column: {col}"

    def test_validate_political_ranges(self, mock_tracts_medium, mock_districts_medium):
        """Test that values are in valid ranges."""
        from tests.mocks.mock_analysis import generate_mock_political_analysis

        political = generate_mock_political_analysis(mock_districts_medium, state='alabama')

        # Shares should be in [0, 1]
        assert (political['dem_share'] >= 0.0).all()
        assert (political['dem_share'] <= 1.0).all()
        assert (political['rep_share'] >= 0.0).all()
        assert (political['rep_share'] <= 1.0).all()

        # Margin should be in [-1, 1]
        assert (political['margin'] >= -1.0).all()
        assert (political['margin'] <= 1.0).all()

        # Winner should be D or R
        assert political['winner'].isin(['D', 'R']).all()


# Pytest markers
pytestmark = [
    pytest.mark.unit,
    pytest.mark.political,
]
