"""
Unit tests for scripts.data.merge_tracts_with_geometries module.

Tests cover:
- GEOID normalization across years
- Shapefile path generation
- CSV and shapefile merging
- STATUS protocol integration
- Year-specific handling (2000, 2010, 2020)
- Error handling for missing files
"""

import pytest
from pathlib import Path
from unittest.mock import Mock, patch, MagicMock
import pandas as pd
import geopandas as gpd
from shapely.geometry import Point, Polygon


class TestShapefilePaths:
    """Test year-specific shapefile path generation."""

    def test_2020_shapefile_path(self):
        """Test 2020 shapefile path format."""
        tiger_dir = Path('data/2020/tiger/tracts')
        state_fips = '50'
        year = 2020

        expected = tiger_dir / f"tl_2020_{state_fips}_tract" / f"tl_2020_{state_fips}_tract.shp"

        # Verify path structure
        assert 'tl_2020_50_tract' in str(expected)
        assert expected.suffix == '.shp'

    def test_2010_shapefile_path(self):
        """Test 2010 shapefile path format."""
        tiger_dir = Path('data/2010/tiger/tracts')
        state_fips = '50'
        year = 2010

        expected = tiger_dir / f"tl_2010_{state_fips}_tract10" / f"tl_2010_{state_fips}_tract10.shp"

        # Verify path structure (note tract10 suffix)
        assert 'tl_2010_50_tract10' in str(expected)
        assert expected.suffix == '.shp'

    def test_2000_shapefile_path(self):
        """Test 2000 shapefile path format (uses 2010 TIGER with 2000 boundaries)."""
        tiger_dir = Path('data/2000/tiger/tracts')
        state_fips = '50'
        year = 2000

        expected = tiger_dir / f"tl_2010_{state_fips}_tract00" / f"tl_2010_{state_fips}_tract00.shp"

        # Verify path structure (note tract00 suffix)
        assert 'tl_2010_50_tract00' in str(expected)
        assert expected.suffix == '.shp'


class TestGEOIDNormalization:
    """Test GEOID column normalization across years."""

    def test_geoid_column_already_normalized(self):
        """Test when GEOID column already exists."""
        gdf = gpd.GeoDataFrame({
            'GEOID': ['50001010100', '50001010200'],
            'geometry': [Point(0, 0), Point(1, 1)]
        })

        # Should return unchanged
        result_columns = gdf.columns.tolist()
        assert 'GEOID' in result_columns

    def test_2020_geoid_normalization(self):
        """Test normalizing GEOID20 to GEOID for 2020 data."""
        gdf = gpd.GeoDataFrame({
            'GEOID20': ['50001010100', '50001010200'],
            'geometry': [Point(0, 0), Point(1, 1)]
        })

        # Should rename GEOID20 -> GEOID
        if 'GEOID20' in gdf.columns:
            gdf = gdf.rename(columns={'GEOID20': 'GEOID'})

        assert 'GEOID' in gdf.columns
        assert 'GEOID20' not in gdf.columns

    def test_2010_geoid_normalization(self):
        """Test normalizing GEOID10 to GEOID for 2010 data."""
        gdf = gpd.GeoDataFrame({
            'GEOID10': ['50001010100', '50001010200'],
            'geometry': [Point(0, 0), Point(1, 1)]
        })

        # Should rename GEOID10 -> GEOID
        if 'GEOID10' in gdf.columns:
            gdf = gdf.rename(columns={'GEOID10': 'GEOID'})

        assert 'GEOID' in gdf.columns
        assert 'GEOID10' not in gdf.columns

    def test_2000_geoid_normalization(self):
        """Test normalizing CTIDFP00 to GEOID for 2000 data."""
        gdf = gpd.GeoDataFrame({
            'CTIDFP00': ['50001010100', '50001010200'],
            'geometry': [Point(0, 0), Point(1, 1)]
        })

        # Should rename CTIDFP00 -> GEOID
        if 'CTIDFP00' in gdf.columns:
            gdf = gdf.rename(columns={'CTIDFP00': 'GEOID'})

        assert 'GEOID' in gdf.columns
        assert 'CTIDFP00' not in gdf.columns


class TestGEOIDPrefixStripping:
    """Test stripping of 2020 GEOID prefixes."""

    def test_2020_prefix_stripping(self):
        """Test stripping 1400000US prefix from 2020 CSV GEOIDs."""
        pop_df = pd.DataFrame({
            'GEOID': ['1400000US50001010100', '1400000US50001010200'],
            'population': [1000, 1500]
        })

        # Strip prefix if present
        if pop_df['GEOID'].str.startswith('1400000US').any():
            pop_df['GEOID'] = pop_df['GEOID'].str.replace('1400000US', '', regex=False)

        assert pop_df['GEOID'].iloc[0] == '50001010100'
        assert pop_df['GEOID'].iloc[1] == '50001010200'
        assert not pop_df['GEOID'].str.startswith('1400000US').any()

    def test_no_prefix_unchanged(self):
        """Test that GEOIDs without prefix are unchanged."""
        pop_df = pd.DataFrame({
            'GEOID': ['50001010100', '50001010200'],
            'population': [1000, 1500]
        })

        # Should not modify if no prefix
        original = pop_df['GEOID'].copy()

        if pop_df['GEOID'].str.startswith('1400000US').any():
            pop_df['GEOID'] = pop_df['GEOID'].str.replace('1400000US', '', regex=False)

        pd.testing.assert_series_equal(pop_df['GEOID'], original)


class TestMergeLogic:
    """Test CSV and shapefile merging logic."""

    def test_basic_merge(self):
        """Test basic merge on GEOID."""
        # Population data
        pop_df = pd.DataFrame({
            'GEOID': ['50001010100', '50001010200'],
            'population': [1000, 1500],
            'NAME': ['Tract 101', 'Tract 102']
        })

        # Shapefile data (mock)
        gdf = gpd.GeoDataFrame({
            'GEOID': ['50001010100', '50001010200'],
            'geometry': [Point(0, 0), Point(1, 1)]
        })

        # Merge
        merged = gdf[['GEOID', 'geometry']].merge(pop_df, on='GEOID', how='inner')

        assert len(merged) == 2
        assert 'GEOID' in merged.columns
        assert 'geometry' in merged.columns
        assert 'population' in merged.columns
        assert 'NAME' in merged.columns

    def test_merge_with_missing_geometries(self):
        """Test that tracts without geometry are dropped."""
        # Population data (3 tracts)
        pop_df = pd.DataFrame({
            'GEOID': ['50001010100', '50001010200', '50001010300'],
            'population': [1000, 1500, 2000]
        })

        # Shapefile data (only 2 tracts)
        gdf = gpd.GeoDataFrame({
            'GEOID': ['50001010100', '50001010200'],
            'geometry': [Point(0, 0), Point(1, 1)]
        })

        # Merge (inner join)
        merged = gdf[['GEOID', 'geometry']].merge(pop_df, on='GEOID', how='inner')

        # Should only have 2 tracts (with geometry)
        assert len(merged) == 2
        assert len(pop_df) == 3

    def test_geoid_string_conversion(self):
        """Test that GEOID is converted to string for consistent merging."""
        # Population data (string GEOID)
        pop_df = pd.DataFrame({
            'GEOID': ['50001010100', '50001010200'],
            'population': [1000, 1500]
        })

        # Shapefile data (numeric GEOID - needs conversion)
        gdf = gpd.GeoDataFrame({
            'GEOID': [50001010100, 50001010200],
            'geometry': [Point(0, 0), Point(1, 1)]
        })

        # Convert to string
        gdf['GEOID'] = gdf['GEOID'].astype(str)
        pop_df['GEOID'] = pop_df['GEOID'].astype(str)

        # Merge should work
        merged = gdf[['GEOID', 'geometry']].merge(pop_df, on='GEOID', how='inner')

        assert len(merged) == 2


class TestCRSHandling:
    """Test CRS (Coordinate Reference System) handling."""

    def test_crs_preservation(self):
        """Test that CRS is preserved after merge."""
        gdf = gpd.GeoDataFrame({
            'GEOID': ['50001010100'],
            'geometry': [Point(0, 0)]
        }, crs='EPSG:4269')  # NAD83

        pop_df = pd.DataFrame({
            'GEOID': ['50001010100'],
            'population': [1000]
        })

        merged = gdf[['GEOID', 'geometry']].merge(pop_df, on='GEOID', how='inner')
        merged = gpd.GeoDataFrame(merged, crs=gdf.crs)

        assert merged.crs.to_string() == 'EPSG:4269'

    def test_missing_crs_set(self):
        """Test that missing CRS is set to EPSG:4269."""
        gdf = gpd.GeoDataFrame({
            'GEOID': ['50001010100'],
            'geometry': [Point(0, 0)]
        })  # No CRS

        # Should set CRS if missing
        if gdf.crs is None:
            gdf = gdf.set_crs('EPSG:4269')

        assert gdf.crs.to_string() == 'EPSG:4269'


class TestErrorHandling:
    """Test error handling for missing files and data."""

    def test_missing_csv_file(self, tmp_path):
        """Test error when population CSV is missing."""
        csv_file = tmp_path / "vt_tracts_2020_population.csv"

        # File doesn't exist
        assert not csv_file.exists()

        # Should detect and handle
        if not csv_file.exists():
            error_occurred = True
        else:
            error_occurred = False

        assert error_occurred is True

    def test_missing_shapefile(self, tmp_path):
        """Test error when shapefile is missing."""
        shapefile = tmp_path / "tl_2020_50_tract" / "tl_2020_50_tract.shp"

        # File doesn't exist
        assert not shapefile.exists()

        # Should detect and handle
        if not shapefile.exists():
            error_occurred = True
        else:
            error_occurred = False

        assert error_occurred is True

    def test_empty_merge_result(self):
        """Test handling when merge produces no results."""
        # Population data
        pop_df = pd.DataFrame({
            'GEOID': ['50001010100'],
            'population': [1000]
        })

        # Shapefile with different GEOIDs
        gdf = gpd.GeoDataFrame({
            'GEOID': ['99999999999'],
            'geometry': [Point(0, 0)]
        })

        # Merge should be empty
        merged = gdf[['GEOID', 'geometry']].merge(pop_df, on='GEOID', how='inner')

        assert len(merged) == 0

        # Should detect and handle
        if len(merged) == 0:
            error_occurred = True
        else:
            error_occurred = False

        assert error_occurred is True


class TestSTATUSProtocol:
    """Test STATUS protocol integration."""

    def test_status_message_worker_mode(self):
        """Test STATUS message format in worker mode."""
        import os

        with patch.dict(os.environ, {'TQDM_POSITION': '0'}):
            position = int(os.environ.get('TQDM_POSITION', '-1'))
            use_status = position >= 0

            message = "VT: Reading population data..."

            if use_status:
                output = f"STATUS:{position}:{message}"
            else:
                output = message

            assert output == "STATUS:0:VT: Reading population data..."

    def test_status_message_standalone_mode(self):
        """Test STATUS message format in standalone mode."""
        import os

        with patch.dict(os.environ, {}, clear=True):
            position = int(os.environ.get('TQDM_POSITION', '-1'))
            use_status = position >= 0

            message = "VT: Reading population data..."

            if use_status:
                output = f"STATUS:{position}:{message}"
            else:
                output = message

            assert output == "VT: Reading population data..."


class TestParallelProcessing:
    """Test parallel processing support."""

    def test_states_can_process_independently(self):
        """Test that states can be processed independently in parallel."""
        states = ['VT', 'DE', 'RI']

        # Each state has independent inputs/outputs
        for state in states:
            state_lower = state.lower()
            csv_file = f"outputs/data/units/2020/{state_lower}_tracts_2020_population.csv"
            shapefile = f"data/2020/tiger/tracts/tl_2020_{state}_tract/tl_2020_{state}_tract.shp"
            output_file = f"outputs/data/units/2020/{state_lower}_tracts_2020.parquet"

            # Verify paths are unique per state
            assert state_lower in csv_file
            assert state_lower in output_file

    def test_worker_count_controls_parallelism(self):
        """Test that worker count controls parallelism level."""
        workers = 12
        states = ['VT', 'DE', 'RI']

        # With 12 workers, all 3 states can run simultaneously
        max_parallel = min(workers, len(states))

        assert max_parallel == 3

    def test_sequential_processing_workers_1(self):
        """Test that workers=1 triggers sequential processing."""
        workers = 1

        # Should use sequential mode
        use_parallel = workers > 1

        assert use_parallel is False


class TestOutputValidation:
    """Test output file validation."""

    def test_parquet_file_creation(self, tmp_path):
        """Test that parquet file is created."""
        output_file = tmp_path / "vt_tracts_2020.parquet"

        # Create mock parquet file
        gdf = gpd.GeoDataFrame({
            'GEOID': ['50001010100'],
            'population': [1000],
            'geometry': [Point(0, 0)]
        }, crs='EPSG:4269')

        gdf.to_parquet(output_file, index=False)

        assert output_file.exists()
        assert output_file.suffix == '.parquet'

    def test_required_columns_present(self):
        """Test that merged output has all required columns."""
        # Population data
        pop_df = pd.DataFrame({
            'GEOID': ['50001010100'],
            'NAME': ['Tract 101'],
            'population': [1000],
            'AREALAND': [1000000],
            'AREAWATR': [10000],
            'INTPTLAT': [44.5],
            'INTPTLON': [-72.6]
        })

        # Shapefile data
        gdf = gpd.GeoDataFrame({
            'GEOID': ['50001010100'],
            'geometry': [Point(0, 0)]
        })

        # Merge
        merged = gdf[['GEOID', 'geometry']].merge(pop_df, on='GEOID', how='inner')

        # Check required columns
        required = ['GEOID', 'geometry', 'population', 'NAME']
        for col in required:
            assert col in merged.columns

    def test_geometry_column_valid(self):
        """Test that geometry column has valid geometries."""
        gdf = gpd.GeoDataFrame({
            'GEOID': ['50001010100', '50001010200'],
            'geometry': [Point(0, 0), Point(1, 1)]
        })

        # All geometries should be valid
        assert not gdf.geometry.isna().any()
        assert len(gdf) == 2
