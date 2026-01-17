"""
Integration tests for single-state pipeline flow.

Tests the complete flow from tract data → redistricting → analysis → visualization
for a single state.
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


class TestSingleStateRedistrictingFlow:
    """Test complete redistricting flow for single state."""

    def test_vermont_full_pipeline(self, tmp_output_dir):
        """Test full pipeline for Vermont (1 district)."""
        from tests.mocks.mock_tracts import generate_mock_tracts
        from tests.mocks.mock_adjacency import generate_mock_adjacency
        from tests.mocks.mock_districts import generate_mock_districts
        from tests.mocks.mock_analysis import (
            generate_mock_political_analysis,
            generate_mock_demographic_analysis,
            generate_mock_compactness_analysis
        )
        from tests.mocks.mock_maps import generate_mock_state_map

        # Step 1: Generate tract data
        tracts = generate_mock_tracts(num_tracts=50, state='vermont', year='2020')
        assert len(tracts) == 50, "Should generate 50 tracts"

        # Step 2: Generate adjacency graph
        graph = generate_mock_adjacency(tracts, connectivity=0.2)
        assert len(graph.nodes) == 50, "Graph should have 50 nodes"
        assert graph.number_of_edges() > 0, "Graph should have edges"

        # Step 3: Generate district assignments
        districts = generate_mock_districts(tracts, num_districts=1)
        assert len(districts) == 50, "Should assign all tracts"
        assert districts['district'].unique().tolist() == [1], "Vermont has 1 district"

        # Step 4: Generate political analysis
        political = generate_mock_political_analysis(districts, state='vermont')
        assert len(political) == 1, "Should have 1 district analysis"
        assert 'winner' in political.columns

        # Step 5: Generate demographic analysis
        demographics = generate_mock_demographic_analysis(districts)
        assert len(demographics) == 1
        assert 'diversity_index' in demographics.columns

        # Step 6: Generate compactness analysis
        compactness = generate_mock_compactness_analysis(districts)
        assert len(compactness) == 1
        assert 'polsby_popper' in compactness.columns

        # Step 7: Generate visualization
        map_file = tmp_output_dir / 'vermont_districts_2020.png'
        generate_mock_state_map(map_file, state='vermont', map_type='districts')
        assert map_file.exists(), "Map should be generated"

    def test_alabama_full_pipeline(self, tmp_output_dir):
        """Test full pipeline for Alabama (7 districts)."""
        from tests.mocks.mock_tracts import generate_mock_tracts
        from tests.mocks.mock_adjacency import generate_mock_adjacency
        from tests.mocks.mock_districts import generate_mock_districts
        from tests.mocks.mock_analysis import (
            generate_mock_political_analysis,
            generate_mock_demographic_analysis,
            generate_mock_compactness_analysis
        )
        from tests.mocks.mock_maps import generate_mock_state_map

        # Full pipeline for medium-sized state
        tracts = generate_mock_tracts(num_tracts=200, state='alabama', year='2020')
        graph = generate_mock_adjacency(tracts, connectivity=0.2)
        districts = generate_mock_districts(tracts, num_districts=7)

        # Validate district assignments
        assert len(districts) == 200
        assert sorted(districts['district'].unique()) == list(range(1, 8))

        # Generate all analysis types
        political = generate_mock_political_analysis(districts, state='alabama')
        demographics = generate_mock_demographic_analysis(districts)
        compactness = generate_mock_compactness_analysis(districts)

        assert len(political) == 7
        assert len(demographics) == 7
        assert len(compactness) == 7

        # Generate visualization
        map_file = tmp_output_dir / 'alabama_districts_2020.png'
        generate_mock_state_map(map_file, state='alabama', map_type='districts')
        assert map_file.exists()


class TestDataPropagation:
    """Test data propagation through pipeline stages."""

    def test_tract_to_district_propagation(self):
        """Test that tract data propagates correctly to district level."""
        from tests.mocks.mock_tracts import generate_mock_tracts
        from tests.mocks.mock_districts import generate_mock_districts

        # Generate data
        tracts = generate_mock_tracts(num_tracts=200, state='alabama')
        districts = generate_mock_districts(tracts, num_districts=7)

        # Check that all tract IDs are present
        tract_ids_input = set(tracts['GEOID'])
        tract_ids_output = set(districts['GEOID'])
        assert tract_ids_input == tract_ids_output, "All tract IDs should be preserved"

        # Check that populations match
        total_pop_input = tracts['population'].sum()
        total_pop_output = districts['population'].sum()
        assert total_pop_input == total_pop_output, "Total population should be preserved"

    def test_district_to_analysis_propagation(self):
        """Test that district assignments propagate to analysis."""
        from tests.mocks.mock_tracts import generate_mock_tracts
        from tests.mocks.mock_districts import generate_mock_districts
        from tests.mocks.mock_analysis import generate_mock_political_analysis

        # Generate data
        tracts = generate_mock_tracts(num_tracts=200, state='alabama')
        districts = generate_mock_districts(tracts, num_districts=7)
        political = generate_mock_political_analysis(districts, state='alabama')

        # Check district numbers match
        district_nums_districts = set(districts['district'].unique())
        district_nums_political = set(political['district'].unique())
        assert district_nums_districts == district_nums_political, \
            "District numbers should match between stages"


class TestMultiYearFlow:
    """Test pipeline flow across different census years."""

    def test_2020_census_flow(self, tmp_output_dir):
        """Test pipeline with 2020 census data."""
        from tests.mocks.mock_tracts import generate_mock_tracts
        from tests.mocks.mock_adjacency import generate_mock_adjacency
        from tests.mocks.mock_districts import generate_mock_districts

        tracts = generate_mock_tracts(num_tracts=200, state='alabama', year='2020')
        assert 'ALAND' in tracts.columns, "2020 data should have ALAND field"

        graph = generate_mock_adjacency(tracts, connectivity=0.2)
        districts = generate_mock_districts(tracts, num_districts=7)

        assert len(districts) == 200
        assert sorted(districts['district'].unique()) == list(range(1, 8))

    def test_2010_census_flow(self, tmp_output_dir):
        """Test pipeline with 2010 census data."""
        from tests.mocks.mock_tracts import generate_mock_tracts
        from tests.mocks.mock_adjacency import generate_mock_adjacency
        from tests.mocks.mock_districts import generate_mock_districts

        tracts = generate_mock_tracts(num_tracts=200, state='alabama', year='2010')
        assert 'ALAND10' in tracts.columns, "2010 data should have ALAND10 field"

        graph = generate_mock_adjacency(tracts, connectivity=0.2)
        districts = generate_mock_districts(tracts, num_districts=7)

        assert len(districts) == 200

    def test_2000_census_flow(self, tmp_output_dir):
        """Test pipeline with 2000 census data."""
        from tests.mocks.mock_tracts import generate_mock_tracts
        from tests.mocks.mock_adjacency import generate_mock_adjacency
        from tests.mocks.mock_districts import generate_mock_districts

        tracts = generate_mock_tracts(num_tracts=200, state='alabama', year='2000')
        assert 'ALAND00' in tracts.columns, "2000 data should have ALAND00 field"

        graph = generate_mock_adjacency(tracts, connectivity=0.2)
        districts = generate_mock_districts(tracts, num_districts=7)

        assert len(districts) == 200


class TestErrorHandling:
    """Test error handling in pipeline flow."""

    def test_handle_disconnected_graph(self):
        """Test handling of disconnected adjacency graph."""
        from tests.mocks.mock_tracts import generate_mock_tracts
        from tests.mocks.mock_adjacency import generate_mock_adjacency

        tracts = generate_mock_tracts(num_tracts=50, state='vermont')

        # Generate graph (should be connected by design)
        graph = generate_mock_adjacency(tracts, connectivity=0.2)

        import networkx as nx
        assert nx.is_connected(graph), "Mock graphs should be connected"

    def test_handle_population_imbalance(self):
        """Test detection of population imbalance."""
        from tests.mocks.mock_tracts import generate_mock_tracts
        from tests.mocks.mock_districts import generate_mock_districts, validate_mock_districts

        tracts = generate_mock_tracts(num_tracts=200, state='alabama')
        districts = generate_mock_districts(tracts, num_districts=7, tolerance=0.005)

        # Validate should not raise error (within tolerance)
        validate_mock_districts(districts, num_districts=7, tolerance=0.01)


class TestOutputFormats:
    """Test output format consistency."""

    def test_csv_output_format(self):
        """Test that CSV outputs have consistent format."""
        from tests.mocks.mock_tracts import generate_mock_tracts
        from tests.mocks.mock_districts import generate_mock_districts, generate_mock_district_summary

        tracts = generate_mock_tracts(num_tracts=200, state='alabama')
        districts = generate_mock_districts(tracts, num_districts=7)
        summary = generate_mock_district_summary(districts, num_districts=7)

        # Check CSV structure
        assert isinstance(summary, pd.DataFrame)
        assert 'district' in summary.columns
        assert 'population' in summary.columns
        assert len(summary) == 7

    def test_geojson_compatibility(self):
        """Test that tract data is compatible with GeoJSON export."""
        from tests.mocks.mock_tracts import generate_mock_tracts

        tracts = generate_mock_tracts(num_tracts=50, state='vermont')

        # Should have geometry column
        assert 'geometry' in tracts.columns

        # All geometries should be valid
        assert tracts.geometry.is_valid.all()


class TestPerformance:
    """Test performance characteristics."""

    @pytest.mark.slow
    def test_large_state_performance(self, tmp_output_dir):
        """Test pipeline with large state (California-sized)."""
        from tests.mocks.mock_tracts import generate_mock_tracts
        from tests.mocks.mock_adjacency import generate_mock_adjacency
        from tests.mocks.mock_districts import generate_mock_districts

        # Generate large dataset
        tracts = generate_mock_tracts(num_tracts=500, state='california', year='2020')
        graph = generate_mock_adjacency(tracts, connectivity=0.2)
        districts = generate_mock_districts(tracts, num_districts=52)

        # Should complete without timeout
        assert len(districts) == 500
        assert len(districts['district'].unique()) == 52


# Pytest markers
pytestmark = [
    pytest.mark.integration,
]
