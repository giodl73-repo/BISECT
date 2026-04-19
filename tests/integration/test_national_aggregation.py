"""
Integration tests for national aggregation.

Tests aggregation of state-level results to national level, including
seat counts, demographic summaries, and compactness statistics.
"""

import pytest
import pandas as pd
import numpy as np
from pathlib import Path
import sys
import tempfile

# Add project root to path
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))


class TestNationalSeatAggregation:
    """Test aggregation of seat counts to national level."""

    def test_aggregate_seats_from_multiple_states(self):
        """Test aggregating D/R seat counts from multiple states."""
        from tests.mocks.mock_tracts import generate_mock_tracts
        from tests.mocks.mock_districts import generate_mock_districts
        from tests.mocks.mock_analysis import generate_mock_political_analysis

        # Generate data for 3 states with different political leans
        states_data = []

        # Red state (Alabama - 7 districts)
        tracts_al = generate_mock_tracts(num_tracts=200, state='alabama', seed=42)
        districts_al = generate_mock_districts(tracts_al, num_districts=7, seed=42)
        political_al = generate_mock_political_analysis(districts_al, state='alabama', seed=42)
        political_al['state'] = 'AL'
        states_data.append(political_al)

        # Blue state (Vermont - 1 district)
        tracts_vt = generate_mock_tracts(num_tracts=50, state='vermont', seed=42)
        districts_vt = generate_mock_districts(tracts_vt, num_districts=1, seed=42)
        political_vt = generate_mock_political_analysis(districts_vt, state='vermont', seed=42)
        political_vt['state'] = 'VT'
        states_data.append(political_vt)

        # Swing state (Wyoming - 1 district, treated as neutral)
        tracts_wy = generate_mock_tracts(num_tracts=50, state='wyoming', seed=42)
        districts_wy = generate_mock_districts(tracts_wy, num_districts=1, seed=42)
        political_wy = generate_mock_political_analysis(districts_wy, state='wyoming', seed=42)
        political_wy['state'] = 'WY'
        states_data.append(political_wy)

        # Combine all states
        national_results = pd.concat(states_data, ignore_index=True)

        # Calculate national seat counts
        d_seats = (national_results['winner'] == 'D').sum()
        r_seats = (national_results['winner'] == 'R').sum()
        total_seats = len(national_results)

        assert d_seats + r_seats == total_seats, "All seats should have winner"
        assert total_seats == 9, "Should have 9 total districts (7+1+1)"

    def test_aggregate_vote_totals_national(self):
        """Test aggregating vote totals to national level."""
        from tests.mocks.mock_tracts import generate_mock_tracts
        from tests.mocks.mock_districts import generate_mock_districts
        from tests.mocks.mock_analysis import generate_mock_political_analysis

        # Generate data for multiple states
        states_data = []
        state_configs = [
            ('alabama', 200, 7),
            ('california', 500, 52),
            ('texas', 400, 38),
        ]

        for state, num_tracts, num_districts in state_configs:
            tracts = generate_mock_tracts(num_tracts=num_tracts, state=state, seed=42)
            districts = generate_mock_districts(tracts, num_districts=num_districts, seed=42)
            political = generate_mock_political_analysis(districts, state=state, seed=42)
            political['state'] = state.upper()[:2]
            states_data.append(political)

        # Combine
        national_results = pd.concat(states_data, ignore_index=True)

        # Calculate national vote totals
        national_dem_votes = national_results['dem_votes'].sum()
        national_rep_votes = national_results['rep_votes'].sum()
        total_votes = national_dem_votes + national_rep_votes

        # Calculate national shares
        national_dem_share = national_dem_votes / total_votes
        national_rep_share = national_rep_votes / total_votes

        assert 0 <= national_dem_share <= 1
        assert 0 <= national_rep_share <= 1
        assert abs(national_dem_share + national_rep_share - 1.0) < 0.01


class TestNationalDemographicAggregation:
    """Test aggregation of demographics to national level."""

    def test_aggregate_demographics_weighted(self):
        """Test population-weighted demographic aggregation."""
        from tests.mocks.mock_tracts import generate_mock_tracts
        from tests.mocks.mock_districts import generate_mock_districts
        from tests.mocks.mock_analysis import generate_mock_demographic_analysis

        # Generate data for multiple states
        states_data = []
        state_configs = [
            ('alabama', 200, 7),
            ('california', 500, 52),
            ('texas', 400, 38),
        ]

        for state, num_tracts, num_districts in state_configs:
            tracts = generate_mock_tracts(num_tracts=num_tracts, state=state, seed=42)
            districts = generate_mock_districts(tracts, num_districts=num_districts, seed=42)
            demographics = generate_mock_demographic_analysis(districts, seed=42)
            demographics['state'] = state.upper()[:2]
            states_data.append(demographics)

        # Combine
        national_demographics = pd.concat(states_data, ignore_index=True)

        # Calculate national statistics (population-weighted)
        total_population = national_demographics['population'].sum()

        national_stats = {}
        for col in ['white', 'black', 'hispanic', 'asian', 'other']:
            weighted_sum = (national_demographics[col] * national_demographics['population']).sum()
            national_stats[col] = weighted_sum / total_population

        # Check that shares sum to ~1.0
        total_share = sum(national_stats.values())
        assert abs(total_share - 1.0) < 0.01, "Demographic shares should sum to 1.0"


class TestNationalCompactnessStatistics:
    """Test national compactness statistics."""

    def test_aggregate_compactness_national(self):
        """Test aggregating compactness metrics to national level."""
        from tests.mocks.mock_tracts import generate_mock_tracts
        from tests.mocks.mock_districts import generate_mock_districts
        from tests.mocks.mock_analysis import generate_mock_compactness_analysis

        # Generate data for multiple states
        states_data = []
        state_configs = [
            ('alabama', 200, 7),
            ('california', 500, 52),
            ('texas', 400, 38),
        ]

        for state, num_tracts, num_districts in state_configs:
            tracts = generate_mock_tracts(num_tracts=num_tracts, state=state, seed=42)
            districts = generate_mock_districts(tracts, num_districts=num_districts, seed=42)
            compactness = generate_mock_compactness_analysis(districts, seed=42)
            compactness['state'] = state.upper()[:2]
            states_data.append(compactness)

        # Combine
        national_compactness = pd.concat(states_data, ignore_index=True)

        # Calculate national statistics
        national_stats = {
            'total_districts': len(national_compactness),
            'mean_polsby_popper': national_compactness['polsby_popper'].mean(),
            'median_polsby_popper': national_compactness['polsby_popper'].median(),
            'std_polsby_popper': national_compactness['polsby_popper'].std(),
            'mean_reock': national_compactness['reock'].mean(),
            'median_reock': national_compactness['reock'].median(),
        }

        # Validate
        assert national_stats['total_districts'] == 97  # 7 + 52 + 38
        assert 0 <= national_stats['mean_polsby_popper'] <= 1
        assert 0 <= national_stats['median_polsby_popper'] <= 1
        assert national_stats['std_polsby_popper'] >= 0


class TestNationalMapGeneration:
    """Test generation of national maps."""

    def test_generate_national_district_map(self, tmp_output_dir):
        """Test generating national district map."""
        from tests.mocks.mock_maps import generate_mock_national_map

        output_file = tmp_output_dir / 'national_districts_2020.png'

        generate_mock_national_map(
            output_file,
            map_type='districts',
            width=1600,
            height=1000
        )

        assert output_file.exists(), "National map should be generated"

        # Verify is valid PNG
        from PIL import Image
        img = Image.open(output_file)
        assert img.format == 'PNG'
        assert img.size == (1600, 1000)

    def test_generate_national_political_map(self, tmp_output_dir):
        """Test generating national political map."""
        from tests.mocks.mock_maps import generate_mock_national_map

        output_file = tmp_output_dir / 'national_political_2020.png'

        generate_mock_national_map(
            output_file,
            map_type='political',
            width=1600,
            height=1000
        )

        assert output_file.exists()


class TestCrossStateComparisons:
    """Test comparisons across states."""

    def test_rank_states_by_compactness(self):
        """Test ranking states by mean compactness."""
        from tests.mocks.mock_tracts import generate_mock_tracts
        from tests.mocks.mock_districts import generate_mock_districts
        from tests.mocks.mock_analysis import generate_mock_compactness_analysis

        # Generate data for multiple states
        states_compactness = []
        state_configs = [
            ('alabama', 200, 7, 'AL'),
            ('california', 500, 52, 'CA'),
            ('texas', 400, 38, 'TX'),
            ('vermont', 50, 1, 'VT'),
            ('wyoming', 50, 1, 'WY'),
        ]

        for state, num_tracts, num_districts, code in state_configs:
            tracts = generate_mock_tracts(num_tracts=num_tracts, state=state, seed=42)
            districts = generate_mock_districts(tracts, num_districts=num_districts, seed=42)
            compactness = generate_mock_compactness_analysis(districts, seed=42)

            # Calculate state mean
            state_mean_pp = compactness['polsby_popper'].mean()
            states_compactness.append({
                'state': code,
                'mean_polsby_popper': state_mean_pp,
                'num_districts': num_districts,
            })

        # Create DataFrame and rank
        states_df = pd.DataFrame(states_compactness)
        states_df = states_df.sort_values('mean_polsby_popper', ascending=False)

        # Check that ranking works
        assert len(states_df) == 5
        assert states_df.iloc[0]['mean_polsby_popper'] >= states_df.iloc[-1]['mean_polsby_popper']


class TestNational435Districts:
    """Test full 435-district national dataset."""

    @pytest.mark.slow
    def test_generate_full_435_districts(self):
        """Test generating complete 435-district dataset."""
        from tests.mocks.mock_tracts import generate_mock_tracts
        from tests.mocks.mock_districts import generate_mock_districts

        # Simplified state configs (just a few representative states)
        state_configs = [
            ('alabama', 200, 7),
            ('california', 500, 52),
            ('texas', 400, 38),
            ('vermont', 50, 1),
        ]

        all_districts = []
        total_expected_districts = sum(config[2] for config in state_configs)

        for state, num_tracts, num_districts in state_configs:
            tracts = generate_mock_tracts(num_tracts=num_tracts, state=state, seed=42)
            districts = generate_mock_districts(tracts, num_districts=num_districts, seed=42)
            districts['state'] = state
            all_districts.append(districts)

        # Combine all districts
        national_districts = pd.concat(all_districts, ignore_index=True)

        # Count districts per state
        districts_per_state = national_districts.groupby('state')['district'].nunique()

        # Validate
        total_districts = districts_per_state.sum()
        assert total_districts == total_expected_districts, \
            f"Should have {total_expected_districts} districts, got {total_districts}"


class TestDataConsistency:
    """Test data consistency across aggregation."""

    def test_population_conservation(self):
        """Test that population is conserved through aggregation."""
        from tests.mocks.mock_tracts import generate_mock_tracts
        from tests.mocks.mock_districts import generate_mock_districts

        # Generate data for multiple states
        state_populations = []
        state_configs = [
            ('alabama', 200, 7),
            ('california', 500, 52),
            ('texas', 400, 38),
        ]

        total_tract_population = 0
        total_district_population = 0

        for state, num_tracts, num_districts in state_configs:
            tracts = generate_mock_tracts(num_tracts=num_tracts, state=state, seed=42)
            districts = generate_mock_districts(tracts, num_districts=num_districts, seed=42)

            tract_pop = tracts['population'].sum()
            district_pop = districts['population'].sum()

            total_tract_population += tract_pop
            total_district_population += district_pop

            # Population should match within state
            assert tract_pop == district_pop, \
                f"{state}: Tract population {tract_pop} != District population {district_pop}"

        # Population should match nationally
        assert total_tract_population == total_district_population, \
            "National population should be conserved"


class TestSummaryCSVGeneration:
    """Test generation of summary CSV files."""

    def test_generate_national_summary_csv(self, tmp_output_dir):
        """Test generating national summary CSV."""
        from tests.mocks.mock_tracts import generate_mock_tracts
        from tests.mocks.mock_districts import generate_mock_districts, generate_mock_district_summary

        # Generate data for multiple states
        all_summaries = []
        state_configs = [
            ('alabama', 200, 7, 'AL'),
            ('california', 500, 52, 'CA'),
        ]

        for state, num_tracts, num_districts, code in state_configs:
            tracts = generate_mock_tracts(num_tracts=num_tracts, state=state, seed=42)
            districts = generate_mock_districts(tracts, num_districts=num_districts, seed=42)
            summary = generate_mock_district_summary(districts, num_districts=num_districts)
            summary['state'] = code
            all_summaries.append(summary)

        # Combine and save
        national_summary = pd.concat(all_summaries, ignore_index=True)
        output_file = tmp_output_dir / 'national_summary.csv'
        national_summary.to_csv(output_file, index=False)

        # Verify file exists and is valid CSV
        assert output_file.exists()

        # Read back and validate
        loaded = pd.read_csv(output_file)
        assert len(loaded) == 59  # 7 + 52
        assert 'state' in loaded.columns
        assert 'district' in loaded.columns
        assert 'population' in loaded.columns


# Pytest markers
pytestmark = [
    pytest.mark.integration,
]
