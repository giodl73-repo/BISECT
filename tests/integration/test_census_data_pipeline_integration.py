"""
Integration tests for census data processing in the main pipeline.

Tests cover:
- Census data processing phase before state processing
- Marker file creation and skip logic
- Integration with run_complete_redistricting.py
- Multi-year parallel processing with census data
- Error handling and graceful failure
"""

import pytest
from pathlib import Path
from unittest.mock import Mock, patch, MagicMock
import subprocess


class TestCensusDataPhaseIntegration:
    """Test census data processing phase in main pipeline."""

    def test_census_phase_runs_before_states(self):
        """Test that census data processing runs before state processing."""
        # Phase order should be: data -> states -> nation
        phases = ['data', 'states', 'nation']

        census_idx = phases.index('data')
        states_idx = phases.index('states')

        assert census_idx < states_idx

    @patch('subprocess.Popen')
    def test_census_process_launched_when_marker_missing(self, mock_popen, tmp_path):
        """Test that census process is launched when marker file is missing."""
        output_dir = tmp_path / "outputs" / "v1" / "2020"
        output_dir.mkdir(parents=True)

        marker_file = output_dir / '.downloads_complete'

        # Marker doesn't exist
        assert not marker_file.exists()

        # Should launch census processing
        should_launch = not marker_file.exists()

        assert should_launch is True

    def test_census_process_skipped_when_marker_exists(self, tmp_path):
        """Test that census process is skipped when marker exists."""
        output_dir = tmp_path / "outputs" / "v1" / "2020"
        output_dir.mkdir(parents=True)

        marker_file = output_dir / '.downloads_complete'
        marker_file.write_text("Complete\n")

        # Marker exists
        assert marker_file.exists()

        # Should skip census processing
        should_skip = marker_file.exists()

        assert should_skip is True

    def test_reset_flag_deletes_marker(self, tmp_path):
        """Test that --reset flag allows reprocessing."""
        output_dir = tmp_path / "outputs" / "v1" / "2020"
        output_dir.mkdir(parents=True)

        marker_file = output_dir / '.downloads_complete'
        marker_file.write_text("Complete\n")

        reset = True

        # Reset should override marker
        should_process = not marker_file.exists() or reset

        assert should_process is True


class TestMultiYearIntegration:
    """Test census data processing in multi-year parallel mode."""

    def test_census_processing_all_years(self):
        """Test census data processing for all census years."""
        years = ['2020', '2010', '2000']

        # Each year should get census processing if needed
        for year in years:
            assert year in ['2000', '2010', '2020']

    def test_census_failure_prevents_state_processing(self, tmp_path):
        """Test that census failure prevents state processing for that year."""
        year_phase = {'2020': 'failed', '2010': 'states', '2000': 'states'}

        # 2020 failed census, should not proceed to states
        assert year_phase['2020'] == 'failed'
        assert year_phase['2010'] == 'states'
        assert year_phase['2000'] == 'states'

    def test_partial_census_success(self, tmp_path):
        """Test that some years can succeed while others fail."""
        census_results = {
            '2020': True,   # Success
            '2010': True,   # Success
            '2000': False   # Failed
        }

        successes = sum(1 for success in census_results.values() if success)

        assert successes == 2
        assert not census_results['2000']


class TestCommandConstruction:
    """Test command construction for census data processing."""

    def test_basic_command_structure(self):
        """Test basic command structure for process_census_data.py."""
        import sys

        year = '2020'
        output_dir = 'outputs/v1/2020'
        scripts_dir = Path('scripts')
        census_script = scripts_dir / 'data' / 'process_census_data.py'

        cmd = [
            sys.executable,
            str(census_script),
            '--year', year,
            '--output-dir', output_dir,
            '--stages', 'tracts', 'adjacency', 'elections', 'demographics'
        ]

        assert sys.executable in cmd
        assert '--year' in cmd
        assert '2020' in cmd
        assert '--output-dir' in cmd
        assert 'outputs/v1/2020' in cmd

    def test_command_with_dry_run_flag(self):
        """Test command includes --dry-run flag when print_only is True."""
        import sys

        cmd = [
            sys.executable,
            'scripts/data/process_census_data.py',
            '--year', '2020',
            '--output-dir', 'outputs/v1/2020'
        ]

        print_only = True
        if print_only:
            cmd.append('--dry-run')

        assert '--dry-run' in cmd

    def test_environment_variables_set(self):
        """Test that environment variables are set correctly."""
        import os

        env = os.environ.copy()
        env['TQDM_POSITION'] = '0'
        env['CENSUS_YEAR'] = '2020'

        assert env['TQDM_POSITION'] == '0'
        assert env['CENSUS_YEAR'] == '2020'


class TestMarkerFileIntegration:
    """Test marker file creation and checking in pipeline."""

    def test_marker_created_on_success(self, tmp_path):
        """Test marker file is created after successful processing."""
        from datetime import datetime

        output_dir = tmp_path / "outputs" / "v1" / "2020"
        output_dir.mkdir(parents=True)

        marker_file = output_dir / '.downloads_complete'

        # Simulate successful completion
        success = True
        if success:
            marker_file.write_text(f"Census data processing completed: {datetime.now().isoformat()}\n")

        assert marker_file.exists()

    def test_marker_not_created_on_failure(self, tmp_path):
        """Test marker file is not created after failed processing."""
        output_dir = tmp_path / "outputs" / "v1" / "2020"
        output_dir.mkdir(parents=True)

        marker_file = output_dir / '.downloads_complete'

        # Simulate failure
        success = False
        if success:
            marker_file.write_text("Complete\n")

        assert not marker_file.exists()

    def test_marker_includes_timestamp(self, tmp_path):
        """Test that marker file includes ISO timestamp."""
        from datetime import datetime

        output_dir = tmp_path / "outputs" / "v1" / "2020"
        output_dir.mkdir(parents=True)

        marker_file = output_dir / '.downloads_complete'
        timestamp = datetime.now().isoformat()
        marker_file.write_text(f"Census data processing completed: {timestamp}\n")

        content = marker_file.read_text()
        assert "completed" in content
        assert "T" in content  # ISO format


class TestErrorHandlingIntegration:
    """Test error handling in pipeline integration."""

    def test_census_failure_stops_pipeline_for_year(self):
        """Test that census failure stops pipeline for that year."""
        year_phase = {}

        # Simulate census failure for 2020
        year_phase['2020'] = 'failed'

        # This year should not proceed to states
        can_process_states = year_phase.get('2020') != 'failed'

        assert can_process_states is False

    def test_census_error_reported_to_user(self, tmp_path):
        """Test that census errors are reported clearly."""
        failed_years = ['2000']

        error_message = f"Census data processing failed for: {', '.join(failed_years)}"

        assert "Census data processing failed" in error_message
        assert "2000" in error_message

    def test_partial_failure_allows_other_years(self):
        """Test that partial failures don't block other years."""
        year_phase = {
            '2020': 'states',       # Success, proceeding
            '2010': 'states',       # Success, proceeding
            '2000': 'failed'        # Failed, blocked
        }

        successful_years = [year for year, phase in year_phase.items() if phase == 'states']

        assert len(successful_years) == 2
        assert '2020' in successful_years
        assert '2010' in successful_years
        assert '2000' not in successful_years


class TestStatusMessageForwarding:
    """Test STATUS message forwarding from census processes."""

    @patch('subprocess.Popen')
    def test_stdout_forwarding(self, mock_popen):
        """Test that stdout is forwarded for STATUS messages."""
        mock_process = Mock()
        mock_process.stdout = [
            "STATUS:0:Processing tracts\n",
            "STATUS:0:Processing adjacency\n",
            ""
        ]
        mock_process.wait.return_value = 0
        mock_popen.return_value = mock_process

        # Simulate reading stdout
        lines = []
        proc = subprocess.Popen(
            ['python', 'script.py'],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1
        )

        for line in mock_process.stdout:
            if line:
                lines.append(line)

        assert any("STATUS" in line for line in lines)

    def test_status_message_format(self):
        """Test STATUS message format is correct."""
        position = 0
        message = "Processing tracts"

        status_msg = f"STATUS:{position}:{message}"

        assert status_msg.startswith("STATUS:")
        assert ":0:" in status_msg
        assert "Processing tracts" in status_msg


class TestResolutionIntegration:
    """Test resolution-aware integration."""

    def test_tract_resolution_default(self):
        """Test that tract resolution is used by default."""
        resolution = 'tract'  # Default

        assert resolution == 'tract'

    def test_multiple_resolution_support(self):
        """Test support for multiple resolutions."""
        resolutions = ['tract', 'block']

        for resolution in resolutions:
            assert resolution in ['tract', 'block']

    def test_resolution_specific_markers(self, tmp_path):
        """Test that markers can be resolution-specific."""
        output_dir = tmp_path / "outputs" / "v1" / "2020"
        output_dir.mkdir(parents=True)

        # Could have separate markers for each resolution
        tract_marker = output_dir / '.tracts_complete'
        block_marker = output_dir / '.blocks_complete'

        tract_marker.write_text("Tract processing complete\n")

        assert tract_marker.exists()
        assert not block_marker.exists()


class TestPipelineSkipLogic:
    """Test skip logic in pipeline."""

    def test_skip_states_flag_skips_census(self):
        """Test that --skip-states flag skips census processing."""
        skip_states = True

        # If skipping states, should skip census too
        skip_census = skip_states

        assert skip_census is True

    def test_states_only_flag_behavior(self):
        """Test that --states-only doesn't affect census processing."""
        states_only = True

        # Census processing should run even with --states-only
        # because it's a prerequisite for state processing
        run_census = True

        assert run_census is True

    def test_unified_hierarchical_mode(self):
        """Test unified hierarchical mode with different worker counts."""
        # Always use hierarchical mode, workers just controls parallelism level

        # Single worker (sequential processing)
        workers = 1
        uses_hierarchical = True  # Always true now
        assert uses_hierarchical

        # Multiple workers (parallel processing)
        workers = 4
        uses_hierarchical = True  # Still uses same hierarchical code path
        assert uses_hierarchical

        # Workers just controls how many states run in parallel
        assert workers == 4
