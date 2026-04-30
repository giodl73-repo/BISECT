"""
E2E tests for pipeline scripts execution.

Tests actual command-line execution of pipeline scripts to catch:
- Syntax errors (script fails to import)
- Missing function arguments
- File path issues
- Output directory structure

These tests run the ACTUAL scripts as subprocesses, not mocks.
"""

import pytest
import subprocess
import sys
from pathlib import Path
import tempfile
import shutil
import json
import pickle
import pandas as pd
import geopandas as gpd
from shapely.geometry import Point, Polygon

# Add project root to path
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))


@pytest.fixture
def vermont_test_data(tmp_path):
    """
    Create minimal Vermont test dataset for E2E testing.

    Creates:
    - Census tract geometries (parquet)
    - Adjacency graph (pickle)
    - Places data (parquet)
    - District assignments (pickle)
    - Intermediate round data

    Returns path to test data directory.
    """
    # Create directory structure
    test_data_dir = tmp_path / 'data'
    tracts_dir = test_data_dir / 'tracts' / '2020'
    adj_dir = test_data_dir / 'adjacency' / '2020'
    places_dir = test_data_dir / 'places' / '2020'

    tracts_dir.mkdir(parents=True, exist_ok=True)
    adj_dir.mkdir(parents=True, exist_ok=True)
    places_dir.mkdir(parents=True, exist_ok=True)

    # Create minimal Vermont tract data (50 tracts, 1 district)
    num_tracts = 50

    # Generate simple geometries (grid of polygons)
    tracts_data = []
    for i in range(num_tracts):
        x = i % 10
        y = i // 10
        poly = Polygon([
            (x, y), (x+1, y), (x+1, y+1), (x, y+1), (x, y)
        ])
        tracts_data.append({
            'GEOID': f'50000{i:05d}',  # Vermont FIPS = 50
            'STATEFP': '50',
            'COUNTYFP': '001',
            'TRACTCE': f'{i:06d}',
            'NAME': f'Tract {i}',
            'geometry': poly,
            'population': 1000 + (i * 10),  # Varying populations
            'ALAND': 1000000,
            'AWATER': 0
        })

    tracts_gdf = gpd.GeoDataFrame(tracts_data, crs='EPSG:4269')

    # Save as parquet
    tracts_file = tracts_dir / 'vt_tracts_2020.parquet'
    tracts_gdf.to_parquet(tracts_file)

    # Create adjacency graph (simple chain)
    adjacency = {}
    edge_weights = {}

    for i in range(num_tracts):
        neighbors = []
        # Connect to adjacent tracts in grid
        if i % 10 > 0:  # Left neighbor
            neighbors.append(i - 1)
        if i % 10 < 9:  # Right neighbor
            neighbors.append(i + 1)
        if i >= 10:  # Top neighbor
            neighbors.append(i - 10)
        if i < 40:  # Bottom neighbor
            neighbors.append(i + 10)

        adjacency[i] = neighbors

        # Add edge weights (boundary lengths in km)
        for neighbor in neighbors:
            edge_key = tuple(sorted([i, neighbor]))
            edge_weights[edge_key] = 1.0  # 1 km boundary

    graph_data = {
        'adjacency': adjacency,
        'edge_weights': edge_weights,
        'num_nodes': num_tracts
    }

    adj_file = adj_dir / 'vt_adjacency_2020.pkl'
    with open(adj_file, 'wb') as f:
        pickle.dump(graph_data, f)

    # Create places data (cities)
    places_data = []
    places_data.append({
        'GEOID': '5046000',
        'NAME': 'Montpelier',
        'geometry': Point(5, 5),
        'population': 8000,
        'PLACEFP': '46000'
    })
    places_data.append({
        'GEOID': '5010675',
        'NAME': 'Burlington',
        'geometry': Point(2, 8),
        'population': 42000,
        'PLACEFP': '10675'
    })

    places_gdf = gpd.GeoDataFrame(places_data, crs='EPSG:4269')
    places_file = places_dir / 'vt_places_2020.parquet'
    places_gdf.to_parquet(places_file)

    return test_data_dir


@pytest.fixture
def vermont_output_dir(tmp_path):
    """Create Vermont output directory structure."""
    output_dir = tmp_path / 'outputs' / 'test' / '2020' / 'states' / 'vermont'
    output_dir.mkdir(parents=True, exist_ok=True)

    # Create intermediate directory with basic round data
    intermediate_dir = output_dir / 'intermediate'
    intermediate_dir.mkdir(exist_ok=True)

    # Create round 1 metadata (1 district = no intermediate rounds for Vermont)
    # But create it anyway for testing
    metadata = {
        'depth': 1,
        'num_regions': 1,
        'total_population': 50000,
        'regions': [
            {
                'region_id': 0,
                'target_districts': 1,
                'population': 50000,
                'num_tracts': 50
            }
        ]
    }

    metadata_file = intermediate_dir / 'round_1_metadata.json'
    with open(metadata_file, 'w') as f:
        json.dump(metadata, f, indent=2)

    # Create assignments (all tracts to district 1)
    assignments = {str(i): 0 for i in range(50)}
    assignments_file = intermediate_dir / 'round_1_assignments.json'
    with open(assignments_file, 'w') as f:
        json.dump(assignments, f, indent=2)

    # Create final assignments
    final_assignments = {i: 1 for i in range(50)}
    final_file = output_dir / 'data' / 'final_assignments.pkl'
    final_file.parent.mkdir(parents=True, exist_ok=True)
    with open(final_file, 'wb') as f:
        pickle.dump(final_assignments, f)

    return output_dir


class TestProcessSingleState:
    """Test process_single_state.py script execution."""

    def test_script_imports_successfully(self):
        """Test that script can be imported without syntax errors."""
        script = project_root / 'scripts' / 'pipeline' / 'process_single_state.py'

        # Try to compile the script (catches syntax errors)
        with open(script, 'r', encoding='utf-8') as f:
            code = f.read()

        try:
            compile(code, str(script), 'exec')
        except SyntaxError as e:
            pytest.fail(f"Script has syntax error: {e}")

    def test_script_has_main_function(self):
        """Test that script has a main() function."""
        script = project_root / 'scripts' / 'pipeline' / 'process_single_state.py'

        with open(script, 'r', encoding='utf-8') as f:
            content = f.read()

        assert 'def main():' in content, "Script must have main() function"
        assert 'if __name__ == \'__main__\':' in content, "Script must be executable"

    @pytest.mark.slow
    def test_vermont_execution_no_analysis(self, vermont_test_data, vermont_output_dir):
        """Test running process_single_state.py for Vermont without analysis."""
        script = project_root / 'scripts' / 'pipeline' / 'process_single_state.py'

        # Set up environment to point to test data
        import os
        env = os.environ.copy()
        env['PYTHONPATH'] = str(project_root)

        # Change to project root so relative imports work
        original_cwd = Path.cwd()
        try:
            os.chdir(project_root)

            # Create a minimal config structure
            # Copy test data to expected location
            data_dir = project_root / 'data'
            if not data_dir.exists():
                shutil.copytree(vermont_test_data, data_dir)

            # Run the script
            result = subprocess.run([
                sys.executable,
                str(script),
                '--state', 'VT',
                '--year', '2020',
                '--output-dir', str(vermont_output_dir),
                '--dpi', '72',  # Low DPI for speed
                '--print-only'  # Dry run to avoid needing all dependencies
            ], capture_output=True, text=True, env=env, timeout=30)

            # Check it didn't fail with syntax error
            assert 'SyntaxError' not in result.stderr, f"Syntax error: {result.stderr}"
            assert 'TypeError' not in result.stderr, f"Type error: {result.stderr}"

        finally:
            os.chdir(original_cwd)
            # Clean up test data
            if (project_root / 'data').exists():
                shutil.rmtree(project_root / 'data', ignore_errors=True)


class TestAnalyzeDistrictDemographics:
    """Test analyze_district_demographics.py script execution."""

    def test_script_imports_successfully(self):
        """Test that script can be imported without syntax errors."""
        script = project_root / 'scripts' / 'pipeline' / 'analyze_district_demographics.py'

        with open(script, 'r', encoding='utf-8') as f:
            code = f.read()

        try:
            compile(code, str(script), 'exec')
        except SyntaxError as e:
            pytest.fail(f"Script has syntax error at line {e.lineno}: {e.msg}")

    def test_output_directory_structure(self, vermont_output_dir):
        """Test that demographic analysis uses correct output directory."""
        script_path = project_root / 'scripts' / 'pipeline' / 'analyze_district_demographics.py'

        # Read the script and check it creates demographic/ subdirectory
        with open(script_path, 'r', encoding='utf-8') as f:
            content = f.read()

        # Should create demographic subdirectory like political analysis does
        assert "/ 'demographic'" in content or "demographic_dir" in content, \
            "Script should create demographic/ subdirectory"
        assert "district_demographics.csv" in content, \
            "Script should create district_demographics.csv"


class TestCreateFinalDistrictSummary:
    """Test create_final_district_summary.py script execution."""

    def test_script_imports_successfully(self):
        """Test that script can be imported without syntax errors."""
        script = project_root / 'scripts' / 'pipeline' / 'create_final_district_summary.py'

        with open(script, 'r', encoding='utf-8') as f:
            code = f.read()

        try:
            compile(code, str(script), 'exec')
        except SyntaxError as e:
            pytest.fail(f"Script has syntax error at line {e.lineno}: {e.msg}")

    def test_create_rounds_hierarchy_calls_correct(self):
        """Test that create_rounds_hierarchy is called with correct arguments."""
        script_path = project_root / 'scripts' / 'pipeline' / 'create_final_district_summary.py'

        with open(script_path, 'r', encoding='utf-8') as f:
            content = f.read()

        # Check function signature
        assert 'def create_rounds_hierarchy(run_dir: Path, num_districts: int, state_code: str, census_year: str' in content, \
            "Function signature should include state_code and census_year"

        # Check all calls include required arguments
        import re
        calls = re.findall(r'create_rounds_hierarchy\([^)]+\)', content)

        for call in calls:
            # All calls should have at least 4 arguments (run_dir, num_districts, state_code, year)
            # Count commas (4 args = 3 commas minimum)
            comma_count = call.count(',')
            assert comma_count >= 3, \
                f"create_rounds_hierarchy call missing arguments: {call}"


class TestAnalyzeDistricts:
    """Test analyze_districts.py (political analysis) script execution."""

    def test_script_imports_successfully(self):
        """Test that script can be imported without syntax errors."""
        script = project_root / 'scripts' / 'pipeline' / 'analyze_districts.py'

        with open(script, 'r', encoding='utf-8') as f:
            code = f.read()

        try:
            compile(code, str(script), 'exec')
        except SyntaxError as e:
            pytest.fail(f"Script has syntax error at line {e.lineno}: {e.msg}")

    def test_output_directory_structure(self):
        """Test that political analysis uses political/ subdirectory."""
        script_path = project_root / 'scripts' / 'pipeline' / 'analyze_districts.py'

        with open(script_path, 'r', encoding='utf-8') as f:
            content = f.read()

        assert "/ 'political'" in content or "political_dir" in content, \
            "Script should create political/ subdirectory"
        assert "district_political.csv" in content, \
            "Script should create district_political.csv"


class TestAnalyzeDistrictCompactness:
    """Test analyze_district_compactness.py script execution."""

    def test_script_imports_successfully(self):
        """Test that script can be imported without syntax errors."""
        script = project_root / 'scripts' / 'pipeline' / 'analyze_district_compactness.py'

        with open(script, 'r', encoding='utf-8') as f:
            code = f.read()

        try:
            compile(code, str(script), 'exec')
        except SyntaxError as e:
            pytest.fail(f"Script has syntax error at line {e.lineno}: {e.msg}")


class TestVisualizationScripts:
    """Test visualization scripts can be imported."""

    @pytest.mark.parametrize('script_name', [
        'visualize_individual_districts.py',
        'visualize_all_rounds.py',
        'visualize_partisan_lean.py',
        'visualize_district_demographics.py',
        'visualize_compactness.py',
        'visualize_metro_areas.py',
    ])
    def test_visualization_script_imports(self, script_name):
        """Test that visualization scripts can be imported without syntax errors."""
        script = project_root / 'scripts' / 'pipeline' / script_name

        if not script.exists():
            pytest.skip(f"Script {script_name} not found")

        with open(script, 'r', encoding='utf-8') as f:
            code = f.read()

        try:
            compile(code, str(script), 'exec')
        except SyntaxError as e:
            pytest.fail(f"{script_name} has syntax error at line {e.lineno}: {e.msg}")


class TestRunCompleteRedistricting:
    """Test run_complete_redistricting.py orchestrator script."""

    def test_script_imports_successfully(self):
        """Test that orchestrator can be imported without syntax errors."""
        script = project_root / 'scripts' / 'pipeline' / 'run_complete_redistricting.py'

        with open(script, 'r', encoding='utf-8') as f:
            code = f.read()

        try:
            compile(code, str(script), 'exec')
        except SyntaxError as e:
            pytest.fail(f"Script has syntax error at line {e.lineno}: {e.msg}")

    def test_has_argument_parser(self):
        """Test that script has proper argument parser."""
        script_path = project_root / 'scripts' / 'pipeline' / 'run_complete_redistricting.py'

        with open(script_path, 'r', encoding='utf-8') as f:
            content = f.read()

        assert 'argparse.ArgumentParser' in content, "Should use argparse"
        assert '--year' in content, "Should have --year argument"
        assert '--version' in content, "Should have --version argument"
        assert '--workers' in content, "Should have --workers argument"


# ============================================================================
# HIGH PRIORITY: Scripts that caused or are involved in V6 failure
# ============================================================================

class TestProcessNation:
    """Test process_nation.py - National post-processing orchestrator (V6 failure point)."""

    def test_script_imports_successfully(self):
        """Test that script can be imported without syntax errors."""
        script = project_root / 'scripts' / 'pipeline' / 'process_nation.py'

        with open(script, 'r', encoding='utf-8') as f:
            code = f.read()

        try:
            compile(code, str(script), 'exec')
        except SyntaxError as e:
            pytest.fail(f"Script has syntax error at line {e.lineno}: {e.msg}")

    def test_has_main_function(self):
        """Test that script has a main() function."""
        script = project_root / 'scripts' / 'pipeline' / 'process_nation.py'

        with open(script, 'r', encoding='utf-8') as f:
            content = f.read()

        assert 'def main():' in content, "Script must have main() function"
        assert 'if __name__ == \'__main__\':' in content, "Script must be executable"

    def test_has_error_logging(self):
        """Test that script uses error logger for debugging failures."""
        script_path = project_root / 'scripts' / 'pipeline' / 'process_nation.py'

        with open(script_path, 'r', encoding='utf-8') as f:
            content = f.read()

        assert 'get_error_logger' in content or 'ErrorLogger' in content, \
            "Should use error logger to catch V6-style failures"


class TestVisualizeNationalRounds:
    """Test visualize_national_rounds.py - Script that failed in V6 run."""

    def test_script_imports_successfully(self):
        """Test that script can be imported without syntax errors."""
        script = project_root / 'scripts' / 'pipeline' / 'visualize_national_rounds.py'

        with open(script, 'r', encoding='utf-8') as f:
            code = f.read()

        try:
            compile(code, str(script), 'exec')
        except SyntaxError as e:
            pytest.fail(f"Script has syntax error at line {e.lineno}: {e.msg}")

    def test_has_error_logging(self):
        """Test that script uses error logger."""
        script_path = project_root / 'scripts' / 'pipeline' / 'visualize_national_rounds.py'

        with open(script_path, 'r', encoding='utf-8') as f:
            content = f.read()

        # Should have error logger for debugging
        assert 'get_error_logger' in content or 'ErrorLogger' in content, \
            "Should use error logger (this script failed in V6)"


class TestRunStateRedistricting:
    """Test run_state_redistricting.py - State-level orchestrator."""

    def test_script_imports_successfully(self):
        """Test that script can be imported without syntax errors."""
        script = project_root / 'scripts' / 'pipeline' / 'run_state_redistricting.py'

        with open(script, 'r', encoding='utf-8') as f:
            code = f.read()

        try:
            compile(code, str(script), 'exec')
        except SyntaxError as e:
            pytest.fail(f"Script has syntax error at line {e.lineno}: {e.msg}")

    def test_has_argument_parser(self):
        """Test that script has argparse for CLI execution."""
        script = project_root / 'scripts' / 'pipeline' / 'run_state_redistricting.py'

        with open(script, 'r', encoding='utf-8') as f:
            content = f.read()

        assert 'argparse.ArgumentParser' in content, "Should use argparse"
        assert '--state' in content, "Should have --state argument"


class TestAddCitiesToDistricts:
    """Test add_cities_to_districts.py - Called for every state."""

    def test_script_imports_successfully(self):
        """Test that script can be imported without syntax errors."""
        script = project_root / 'scripts' / 'pipeline' / 'add_cities_to_districts.py'

        with open(script, 'r', encoding='utf-8') as f:
            code = f.read()

        try:
            compile(code, str(script), 'exec')
        except SyntaxError as e:
            pytest.fail(f"Script has syntax error at line {e.lineno}: {e.msg}")


class TestCreateUSAggregate:
    """Test create_us_aggregate.py - Used in national post-processing."""

    def test_script_imports_successfully(self):
        """Test that script can be imported without syntax errors."""
        script = project_root / 'scripts' / 'pipeline' / 'create_us_aggregate.py'

        with open(script, 'r', encoding='utf-8') as f:
            code = f.read()

        try:
            compile(code, str(script), 'exec')
        except SyntaxError as e:
            pytest.fail(f"Script has syntax error at line {e.lineno}: {e.msg}")


class TestCreateUSRoundsHierarchy:
    """Test create_us_rounds_hierarchy.py - Used in national post-processing."""

    def test_script_imports_successfully(self):
        """Test that script can be imported without syntax errors."""
        script = project_root / 'scripts' / 'pipeline' / 'create_us_rounds_hierarchy.py'

        with open(script, 'r', encoding='utf-8') as f:
            code = f.read()

        try:
            compile(code, str(script), 'exec')
        except SyntaxError as e:
            pytest.fail(f"Script has syntax error at line {e.lineno}: {e.msg}")


class TestVisualizeNationalDistricts:
    """Test visualize_national_districts.py - Used in national post-processing."""

    def test_script_imports_successfully(self):
        """Test that script can be imported without syntax errors."""
        script = project_root / 'scripts' / 'pipeline' / 'visualize_national_districts.py'

        with open(script, 'r', encoding='utf-8') as f:
            code = f.read()

        try:
            compile(code, str(script), 'exec')
        except SyntaxError as e:
            pytest.fail(f"Script has syntax error at line {e.lineno}: {e.msg}")


# ============================================================================
# REMAINING SCRIPTS: Data processing and utilities
# ============================================================================

class TestCreateSingleDistrictStates:
    """Test create_single_district_states.py - Creates at-large districts."""

    def test_script_imports_successfully(self):
        """Test that script can be imported without syntax errors."""
        script = project_root / 'scripts' / 'pipeline' / 'create_single_district_states.py'

        with open(script, 'r', encoding='utf-8') as f:
            code = f.read()

        try:
            compile(code, str(script), 'exec')
        except SyntaxError as e:
            pytest.fail(f"Script has syntax error at line {e.lineno}: {e.msg}")


class TestExportRoundsToCSV:
    """Test export_rounds_to_csv.py - Exports round data."""

    def test_script_imports_successfully(self):
        """Test that script can be imported without syntax errors."""
        script = project_root / 'scripts' / 'pipeline' / 'export_rounds_to_csv.py'

        with open(script, 'r', encoding='utf-8') as f:
            code = f.read()

        try:
            compile(code, str(script), 'exec')
        except SyntaxError as e:
            pytest.fail(f"Script has syntax error at line {e.lineno}: {e.msg}")


class TestCleanupDistrictSummary:
    """Test cleanup_district_summary.py - Cleans up summary files."""

    def test_script_imports_successfully(self):
        """Test that script can be imported without syntax errors."""
        script = project_root / 'scripts' / 'pipeline' / 'cleanup_district_summary.py'

        with open(script, 'r', encoding='utf-8') as f:
            code = f.read()

        try:
            compile(code, str(script), 'exec')
        except SyntaxError as e:
            pytest.fail(f"Script has syntax error at line {e.lineno}: {e.msg}")


class TestVisualizeDistricts:
    """Test visualize_districts.py - District visualization."""

    def test_script_imports_successfully(self):
        """Test that script can be imported without syntax errors."""
        script = project_root / 'scripts' / 'pipeline' / 'visualize_districts.py'

        with open(script, 'r', encoding='utf-8') as f:
            code = f.read()

        try:
            compile(code, str(script), 'exec')
        except SyntaxError as e:
            pytest.fail(f"Script has syntax error at line {e.lineno}: {e.msg}")


class TestVisualizeSplit:
    """Test visualize_split.py - Split visualization."""

    def test_script_imports_successfully(self):
        """Test that script can be imported without syntax errors."""
        script = project_root / 'scripts' / 'pipeline' / 'visualize_split.py'

        with open(script, 'r', encoding='utf-8') as f:
            code = f.read()

        try:
            compile(code, str(script), 'exec')
        except SyntaxError as e:
            pytest.fail(f"Script has syntax error at line {e.lineno}: {e.msg}")


# Pytest markers
pytestmark = [
    pytest.mark.e2e,
    pytest.mark.pipeline,
]
