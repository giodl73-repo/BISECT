"""
Unit tests for aggregation functionality.

Tests CSV aggregation, state-to-national rollups, summary statistics,
and data consolidation.
"""

import pytest
import pandas as pd
import numpy as np
from pathlib import Path
import sys

# Add project root to path
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))


class TestDistrictSummaryAggregation:
    """Test aggregation of district summaries."""

    def test_aggregate_state_summaries_to_national(self):
        """Test aggregating state-level summaries to national level."""
        # Mock state-level data
        alabama = pd.DataFrame({
            'state': ['AL'] * 7,
            'district': list(range(1, 8)),
            'population': [700000] * 7,
            'polsby_popper': [0.25, 0.30, 0.28, 0.32, 0.27, 0.29, 0.31],
        })

        california = pd.DataFrame({
            'state': ['CA'] * 52,
            'district': list(range(1, 53)),
            'population': [750000] * 52,
            'polsby_popper': np.random.uniform(0.20, 0.40, 52),
        })

        # Combine
        all_states = pd.concat([alabama, california], ignore_index=True)

        # Calculate national statistics
        national_stats = {
            'total_districts': len(all_states),
            'mean_population': all_states['population'].mean(),
            'median_population': all_states['population'].median(),
            'mean_polsby_popper': all_states['polsby_popper'].mean(),
            'median_polsby_popper': all_states['polsby_popper'].median(),
        }

        # Validate
        assert national_stats['total_districts'] == 59  # 7 + 52
        assert 700000 <= national_stats['mean_population'] <= 750000

    def test_group_by_state(self):
        """Test grouping districts by state."""
        # National data
        data = pd.DataFrame({
            'state': ['AL', 'AL', 'CA', 'CA', 'CA', 'TX', 'TX'],
            'district': [1, 2, 1, 2, 3, 1, 2],
            'population': [700000, 710000, 750000, 740000, 760000, 720000, 730000],
        })

        # Group by state
        by_state = data.groupby('state').agg({
            'district': 'count',
            'population': ['mean', 'sum'],
        }).reset_index()

        by_state.columns = ['state', 'num_districts', 'mean_population', 'total_population']

        # Check Alabama
        al = by_state[by_state['state'] == 'AL'].iloc[0]
        assert al['num_districts'] == 2
        assert al['mean_population'] == 705000

        # Check California
        ca = by_state[by_state['state'] == 'CA'].iloc[0]
        assert ca['num_districts'] == 3
        assert ca['total_population'] == 2250000  # 750k + 740k + 760k


class TestPoliticalAggregation:
    """Test aggregation of political data."""

    def test_aggregate_seat_counts(self):
        """Test aggregating seat counts across states."""
        # State-level results
        state_results = pd.DataFrame({
            'state': ['AL', 'AL', 'AL', 'CA', 'CA', 'CA', 'TX', 'TX'],
            'district': [1, 2, 3, 1, 2, 3, 1, 2],
            'winner': ['R', 'R', 'D', 'D', 'D', 'D', 'R', 'R'],
        })

        # Aggregate by party
        national_seats = state_results['winner'].value_counts()

        d_seats = national_seats.get('D', 0)
        r_seats = national_seats.get('R', 0)

        assert d_seats == 4  # 1 AL + 3 CA
        assert r_seats == 4  # 2 AL + 2 TX

    def test_aggregate_vote_totals(self):
        """Test aggregating vote totals to national level."""
        # State-level vote data
        state_votes = pd.DataFrame({
            'state': ['AL', 'CA', 'TX'],
            'dem_votes': [1000000, 5000000, 3000000],
            'rep_votes': [1500000, 3000000, 4000000],
        })

        # National totals
        national_dem = state_votes['dem_votes'].sum()
        national_rep = state_votes['rep_votes'].sum()

        assert national_dem == 9000000  # 1M + 5M + 3M
        assert national_rep == 8500000  # 1.5M + 3M + 4M

        # National share
        total_votes = national_dem + national_rep
        dem_share = national_dem / total_votes
        rep_share = national_rep / total_votes

        assert abs(dem_share - (9.0 / 17.5)) < 0.001
        assert abs(rep_share - (8.5 / 17.5)) < 0.001


class TestDemographicAggregation:
    """Test aggregation of demographic data."""

    def test_aggregate_demographics_weighted(self):
        """Test population-weighted demographic aggregation."""
        # State-level demographics
        state_demographics = pd.DataFrame({
            'state': ['AL', 'CA', 'TX'],
            'population': [5000000, 39000000, 29000000],
            'white': [0.65, 0.40, 0.45],
            'black': [0.27, 0.06, 0.12],
            'hispanic': [0.05, 0.39, 0.39],
            'asian': [0.02, 0.14, 0.03],
            'other': [0.01, 0.01, 0.01],
        })

        # Calculate national demographics (population-weighted)
        total_population = state_demographics['population'].sum()

        national_demographics = {}
        for col in ['white', 'black', 'hispanic', 'asian', 'other']:
            weighted_sum = (state_demographics[col] * state_demographics['population']).sum()
            national_demographics[col] = weighted_sum / total_population

        # Check that shares sum to ~1.0
        total_share = sum(national_demographics.values())
        assert abs(total_share - 1.0) < 0.01

        # Check that calculation is weighted (not simple mean)
        # CA and TX are much larger, so should dominate
        assert national_demographics['hispanic'] > 0.25, \
            "National Hispanic share should reflect CA/TX weight"


class TestCompactnessAggregation:
    """Test aggregation of compactness metrics."""

    def test_aggregate_compactness_statistics(self, mock_tracts_medium, mock_districts_medium):
        """Test calculating aggregate compactness statistics."""
        from tests.mocks.mock_analysis import generate_mock_compactness_analysis

        # Generate compactness data for multiple "states"
        states_data = []
        for state in ['alabama', 'california', 'texas']:
            compactness = generate_mock_compactness_analysis(mock_districts_medium)
            compactness['state'] = state
            states_data.append(compactness)

        all_compactness = pd.concat(states_data, ignore_index=True)

        # Calculate national statistics
        national_stats = {
            'mean_pp': all_compactness['polsby_popper'].mean(),
            'median_pp': all_compactness['polsby_popper'].median(),
            'std_pp': all_compactness['polsby_popper'].std(),
            'min_pp': all_compactness['polsby_popper'].min(),
            'max_pp': all_compactness['polsby_popper'].max(),
        }

        # Validate
        assert 0 <= national_stats['mean_pp'] <= 1
        assert 0 <= national_stats['median_pp'] <= 1
        assert national_stats['std_pp'] >= 0
        assert national_stats['min_pp'] <= national_stats['median_pp'] <= national_stats['max_pp']


class TestCSVMerging:
    """Test CSV file merging operations."""

    def test_merge_district_summaries(self):
        """Test merging multiple district summary CSVs."""
        # Create mock CSVs for different states
        al_summary = pd.DataFrame({
            'state': ['AL'] * 3,
            'district': [1, 2, 3],
            'population': [700000, 710000, 705000],
        })

        ca_summary = pd.DataFrame({
            'state': ['CA'] * 3,
            'district': [1, 2, 3],
            'population': [750000, 745000, 755000],
        })

        # Merge
        national_summary = pd.concat([al_summary, ca_summary], ignore_index=True)

        # Validate
        assert len(national_summary) == 6
        assert list(national_summary['state'].unique()) == ['AL', 'CA']

    def test_merge_with_missing_columns(self):
        """Test merging CSVs with missing optional columns."""
        # CSV 1 has all columns
        csv1 = pd.DataFrame({
            'state': ['AL'],
            'district': [1],
            'population': [700000],
            'polsby_popper': [0.30],
        })

        # CSV 2 missing optional column
        csv2 = pd.DataFrame({
            'state': ['CA'],
            'district': [1],
            'population': [750000],
        })

        # Merge (outer join to keep all columns)
        merged = pd.concat([csv1, csv2], ignore_index=True)

        # Check that missing values are NaN
        assert pd.isna(merged.loc[1, 'polsby_popper'])


class TestRankingAndSorting:
    """Test ranking and sorting of aggregated data."""

    def test_rank_states_by_compactness(self):
        """Test ranking states by mean compactness."""
        # State compactness data
        state_compactness = pd.DataFrame({
            'state': ['AL', 'CA', 'TX', 'NY', 'FL'],
            'mean_polsby_popper': [0.28, 0.32, 0.25, 0.30, 0.27],
        })

        # Rank by compactness (descending)
        ranked = state_compactness.sort_values('mean_polsby_popper', ascending=False)

        # Check top state
        top_state = ranked.iloc[0]
        assert top_state['state'] == 'CA'
        assert top_state['mean_polsby_popper'] == 0.32

        # Check bottom state
        bottom_state = ranked.iloc[-1]
        assert bottom_state['state'] == 'TX'
        assert bottom_state['mean_polsby_popper'] == 0.25

    def test_rank_districts_within_states(self):
        """Test ranking districts within each state."""
        # Multi-state district data
        data = pd.DataFrame({
            'state': ['AL', 'AL', 'AL', 'CA', 'CA', 'CA'],
            'district': [1, 2, 3, 1, 2, 3],
            'polsby_popper': [0.25, 0.32, 0.28, 0.35, 0.30, 0.38],
        })

        # Rank within each state
        data['rank_within_state'] = data.groupby('state')['polsby_popper'].rank(ascending=False)

        # Check Alabama ranks
        al_data = data[data['state'] == 'AL']
        assert al_data[al_data['district'] == 2]['rank_within_state'].values[0] == 1  # Highest PP
        assert al_data[al_data['district'] == 1]['rank_within_state'].values[0] == 3  # Lowest PP


class TestPercentileCalculations:
    """Test percentile calculations across aggregated data."""

    def test_calculate_percentiles(self):
        """Test calculating percentiles for national data."""
        # National compactness data
        compactness_values = np.array([0.15, 0.20, 0.25, 0.28, 0.30, 0.32, 0.35, 0.40, 0.42, 0.45])

        # Calculate percentiles
        p25 = np.percentile(compactness_values, 25)
        p50 = np.percentile(compactness_values, 50)  # Median
        p75 = np.percentile(compactness_values, 75)

        # Validate
        assert p25 < p50 < p75
        assert 0.20 <= p25 <= 0.30
        assert 0.28 <= p50 <= 0.35
        assert 0.35 <= p75 <= 0.42


class TestTimeSeriesAggregation:
    """Test aggregation across multiple census years."""

    def test_aggregate_across_years(self):
        """Test aggregating metrics across census years."""
        # Data for multiple years
        year_2000 = pd.DataFrame({
            'year': [2000] * 3,
            'state': ['AL', 'CA', 'TX'],
            'mean_polsby_popper': [0.20, 0.25, 0.22],
        })

        year_2010 = pd.DataFrame({
            'year': [2010] * 3,
            'state': ['AL', 'CA', 'TX'],
            'mean_polsby_popper': [0.22, 0.28, 0.24],
        })

        year_2020 = pd.DataFrame({
            'year': [2020] * 3,
            'state': ['AL', 'CA', 'TX'],
            'mean_polsby_popper': [0.28, 0.32, 0.27],
        })

        # Combine
        all_years = pd.concat([year_2000, year_2010, year_2020], ignore_index=True)

        # Calculate improvement over time
        improvements = all_years.groupby('state').apply(
            lambda x: (x['mean_polsby_popper'].iloc[-1] - x['mean_polsby_popper'].iloc[0]) /
                     x['mean_polsby_popper'].iloc[0]
        )

        # All states should show improvement
        assert (improvements > 0).all(), "All states should show compactness improvement"


class TestDataQualityChecks:
    """Test data quality checks during aggregation."""

    def test_detect_duplicate_districts(self):
        """Test detecting duplicate district entries."""
        # Data with duplicates
        data = pd.DataFrame({
            'state': ['AL', 'AL', 'AL', 'AL'],
            'district': [1, 2, 2, 3],  # District 2 appears twice
        })

        # Check for duplicates
        duplicates = data[data.duplicated(subset=['state', 'district'], keep=False)]

        assert len(duplicates) == 2, "Should detect 2 duplicate entries"
        assert all(duplicates['district'] == 2), "Duplicates should be district 2"

    def test_detect_missing_states(self):
        """Test detecting missing states in national data."""
        # National data missing some states
        data = pd.DataFrame({
            'state': ['AL', 'CA', 'TX'],  # Only 3 states
            'num_districts': [7, 52, 38],
        })

        expected_states = 50
        actual_states = len(data)

        assert actual_states < expected_states, "Should detect missing states"
        missing_count = expected_states - actual_states
        assert missing_count == 47, f"Should be missing {missing_count} states"


# Pytest markers
pytestmark = [
    pytest.mark.unit,
    pytest.mark.aggregation,
]
