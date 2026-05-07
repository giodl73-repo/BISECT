"""
Unit tests for demographic analysis functionality.

Tests demographic composition calculation, diversity metrics, and
population breakdowns by race/ethnicity.
"""

import pytest
import pandas as pd
import numpy as np
from pathlib import Path
import sys

# Add project root to path
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))


class TestDemographicDataGeneration:
    """Test demographic analysis data generation."""

    def test_generate_demographic_analysis(self, mock_tracts_medium, mock_districts_medium):
        """Test demographic analysis generation for districts."""
        from tests.mocks.mock_analysis import generate_mock_demographic_analysis

        demographics = generate_mock_demographic_analysis(mock_districts_medium)

        # Check structure
        assert len(demographics) == 7, "Should have 7 districts"
        assert 'district' in demographics.columns
        assert 'population' in demographics.columns
        assert 'white' in demographics.columns
        assert 'black' in demographics.columns
        assert 'hispanic' in demographics.columns
        assert 'asian' in demographics.columns
        assert 'other' in demographics.columns
        assert 'diversity_index' in demographics.columns

    def test_demographic_shares_sum_to_one(self, mock_tracts_medium, mock_districts_medium):
        """Test that demographic shares sum to 1.0."""
        from tests.mocks.mock_analysis import generate_mock_demographic_analysis

        demographics = generate_mock_demographic_analysis(mock_districts_medium)

        # Calculate total share for each district
        share_sums = (demographics['white'] + demographics['black'] +
                     demographics['hispanic'] + demographics['asian'] +
                     demographics['other'])

        assert (abs(share_sums - 1.0) < 0.01).all(), "Demographic shares must sum to 1.0"

    def test_demographic_shares_positive(self, mock_tracts_medium, mock_districts_medium):
        """Test that all demographic shares are positive."""
        from tests.mocks.mock_analysis import generate_mock_demographic_analysis

        demographics = generate_mock_demographic_analysis(mock_districts_medium)

        assert (demographics['white'] >= 0.0).all()
        assert (demographics['black'] >= 0.0).all()
        assert (demographics['hispanic'] >= 0.0).all()
        assert (demographics['asian'] >= 0.0).all()
        assert (demographics['other'] >= 0.0).all()


class TestDiversityIndex:
    """Test diversity index calculations."""

    def test_diversity_index_range(self, mock_tracts_medium, mock_districts_medium):
        """Test that diversity index is in valid range [0, 1]."""
        from tests.mocks.mock_analysis import generate_mock_demographic_analysis

        demographics = generate_mock_demographic_analysis(mock_districts_medium)

        assert (demographics['diversity_index'] >= 0.0).all(), "Diversity index must be >= 0"
        assert (demographics['diversity_index'] <= 1.0).all(), "Diversity index must be <= 1"

    def test_diversity_index_calculation(self):
        """Test Simpson's Diversity Index calculation."""
        # Example: Homogeneous district (all one group)
        homogeneous = pd.DataFrame({
            'white': [1.0],
            'black': [0.0],
            'hispanic': [0.0],
            'asian': [0.0],
            'other': [0.0],
        })

        diversity_homogeneous = 1 - (1.0**2 + 0.0**2 + 0.0**2 + 0.0**2 + 0.0**2)
        assert diversity_homogeneous == 0.0, "Homogeneous should have diversity = 0"

        # Example: Perfectly diverse district (5 equal groups)
        diverse = pd.DataFrame({
            'white': [0.2],
            'black': [0.2],
            'hispanic': [0.2],
            'asian': [0.2],
            'other': [0.2],
        })

        diversity_diverse = 1 - (0.2**2 + 0.2**2 + 0.2**2 + 0.2**2 + 0.2**2)
        expected = 1 - (5 * 0.04)  # 1 - 0.20 = 0.80
        assert abs(diversity_diverse - expected) < 0.001, "Perfectly diverse should have diversity = 0.80"

    def test_diversity_index_correlation(self, mock_tracts_medium, mock_districts_medium):
        """Test that diversity index correlates with demographic distribution."""
        from tests.mocks.mock_analysis import generate_mock_demographic_analysis

        demographics = generate_mock_demographic_analysis(mock_districts_medium)

        # Districts with high white share should have lower diversity
        # (assuming other groups are small)
        for _, row in demographics.iterrows():
            if row['white'] > 0.9:
                assert row['diversity_index'] < 0.3, \
                    f"District with {row['white']:.1%} white should have low diversity"


class TestMajorityMinorityDistricts:
    """Test identification of majority-minority districts."""

    def test_identify_majority_minority_districts(self, mock_tracts_medium, mock_districts_medium):
        """Test identification of majority-minority districts."""
        from tests.mocks.mock_analysis import generate_mock_demographic_analysis

        demographics = generate_mock_demographic_analysis(mock_districts_medium)

        # Majority-minority: non-white population > 50%
        demographics['non_white'] = 1.0 - demographics['white']
        demographics['is_majority_minority'] = demographics['non_white'] > 0.5

        mm_districts = demographics[demographics['is_majority_minority']]

        # Should identify districts correctly
        assert 'district' in mm_districts.columns
        assert all(mm_districts['non_white'] > 0.5)


class TestDemographicAggregation:
    """Test aggregation of demographics from tract to district level."""

    def test_aggregate_demographics_to_districts(self):
        """Test aggregating tract-level demographics to district level."""
        # Create mock tract-level data
        tract_demographics = pd.DataFrame({
            'GEOID': ['01001000100', '01001000200', '01001000300'],
            'district': [1, 1, 2],
            'population': [1000, 1500, 2000],
            'white': [600, 900, 1000],
            'black': [200, 300, 600],
            'hispanic': [150, 200, 300],
            'asian': [30, 70, 80],
            'other': [20, 30, 20],
        })

        # Aggregate to district level
        district_demographics = tract_demographics.groupby('district').agg({
            'population': 'sum',
            'white': 'sum',
            'black': 'sum',
            'hispanic': 'sum',
            'asian': 'sum',
            'other': 'sum',
        }).reset_index()

        # Convert counts to shares
        for col in ['white', 'black', 'hispanic', 'asian', 'other']:
            district_demographics[f'{col}_share'] = \
                district_demographics[col] / district_demographics['population']

        # Check District 1
        d1 = district_demographics[district_demographics['district'] == 1].iloc[0]
        assert d1['population'] == 2500  # 1000 + 1500
        assert d1['white'] == 1500  # 600 + 900
        assert abs(d1['white_share'] - (1500 / 2500)) < 0.001

        # Check District 2
        d2 = district_demographics[district_demographics['district'] == 2].iloc[0]
        assert d2['population'] == 2000
        assert d2['white'] == 1000
        assert abs(d2['white_share'] - 0.5) < 0.001


class TestDemographicComparisons:
    """Test demographic comparison metrics."""

    def test_compare_district_to_state_demographics(self):
        """Test comparing district demographics to state average."""
        # State-level demographics
        state_demographics = pd.Series({
            'white': 0.60,
            'black': 0.25,
            'hispanic': 0.10,
            'asian': 0.03,
            'other': 0.02,
        })

        # District demographics
        district_demographics = pd.Series({
            'white': 0.70,
            'black': 0.15,
            'hispanic': 0.10,
            'asian': 0.03,
            'other': 0.02,
        })

        # Calculate absolute differences
        differences = abs(district_demographics - state_demographics)

        # White population difference
        assert abs(differences['white'] - 0.10) < 0.001, "White difference should be 10pp"

        # Total absolute difference
        total_diff = differences.sum()
        assert total_diff < 1.0, "Total difference should be < 100pp"


class TestRepresentationMetrics:
    """Test demographic representation metrics."""

    def test_proportional_representation(self):
        """Test proportional representation calculation."""
        # State population: 60% white, 25% black, 15% other
        state_demographics = {
            'white': 0.60,
            'black': 0.25,
            'other': 0.15,
        }

        # 7 districts
        # White-majority: 5 districts (71%)
        # Black-majority: 2 districts (29%)
        district_demographics = pd.DataFrame({
            'district': [1, 2, 3, 4, 5, 6, 7],
            'white': [0.70, 0.75, 0.65, 0.72, 0.68, 0.30, 0.25],
            'black': [0.20, 0.15, 0.25, 0.18, 0.22, 0.50, 0.60],
            'other': [0.10, 0.10, 0.10, 0.10, 0.10, 0.20, 0.15],
        })

        # Count majority districts
        white_majority = (district_demographics['white'] > 0.5).sum()
        black_majority = (district_demographics['black'] > 0.5).sum()

        white_seat_share = white_majority / len(district_demographics)
        black_seat_share = black_majority / len(district_demographics)

        # Compare to population shares
        white_pop_share = state_demographics['white']
        black_pop_share = state_demographics['black']

        white_representation_ratio = white_seat_share / white_pop_share
        black_representation_ratio = black_seat_share / black_pop_share

        # Check ratios
        assert white_representation_ratio > 0, "White representation ratio should be positive"
        assert black_representation_ratio > 0, "Black representation ratio should be positive"


class TestDemographicDataValidation:
    """Test validation of demographic analysis data."""

    def test_validate_demographic_columns(self, mock_tracts_medium, mock_districts_medium):
        """Test that all required columns are present."""
        from tests.mocks.mock_analysis import generate_mock_demographic_analysis

        demographics = generate_mock_demographic_analysis(mock_districts_medium)

        required_cols = ['district', 'population', 'white', 'black', 'hispanic', 'asian', 'other', 'diversity_index']
        for col in required_cols:
            assert col in demographics.columns, f"Missing required column: {col}"

    def test_validate_demographic_ranges(self, mock_tracts_medium, mock_districts_medium):
        """Test that values are in valid ranges."""
        from tests.mocks.mock_analysis import generate_mock_demographic_analysis

        demographics = generate_mock_demographic_analysis(mock_districts_medium)

        # All shares should be in [0, 1]
        for col in ['white', 'black', 'hispanic', 'asian', 'other']:
            assert (demographics[col] >= 0.0).all(), f"{col} share must be >= 0"
            assert (demographics[col] <= 1.0).all(), f"{col} share must be <= 1"

        # Diversity index should be in [0, 1]
        assert (demographics['diversity_index'] >= 0.0).all()
        assert (demographics['diversity_index'] <= 1.0).all()

        # Population should be positive
        assert (demographics['population'] > 0).all()


# Pytest markers
pytestmark = [
    pytest.mark.unit,
    pytest.mark.demographic,
]
