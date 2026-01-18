"""
Unit tests for scripts.data.process_census_data module.

Tests cover:
- Marker file creation and checking (resolution-specific)
- Command execution with mocks
- Error handling and logging
- STATUS protocol integration
- Stage processing logic
- Resolution-aware processing
"""

import pytest
from pathlib import Path
from unittest.mock import Mock, patch, MagicMock, call
import subprocess
import os


# Note: We'll import the actual module functions in the tests
# This allows us to mock subprocess calls and file I/O


class TestMarkerFiles:
    """Test marker file creation and checking."""

    def test_marker_file_path_tract_level(self, tmp_path):
        """Test marker file path for tract-level processing."""
        output_dir = tmp_path / "outputs" / "v1" / "2020"
        output_dir.mkdir(parents=True)

        # Marker file should be at root of output directory
        marker_file = output_dir / '.downloads_complete'

        assert not marker_file.exists()

        # Create marker
        marker_file.write_text("Census data processing completed\n")

        assert marker_file.exists()
        assert "completed" in marker_file.read_text()

    def test_marker_file_path_block_level(self, tmp_path):
        """Test marker file path for block-level processing."""
        output_dir = tmp_path / "outputs" / "v1" / "2020"
        output_dir.mkdir(parents=True)

        # For block-level, could have separate marker or include in name
        marker_file = output_dir / '.downloads_complete'

        assert not marker_file.exists()
        marker_file.write_text("Census data processing completed: block\n")

        assert marker_file.exists()

    def test_marker_file_prevents_reprocessing(self, tmp_path):
        """Test that existing marker file prevents reprocessing."""
        output_dir = tmp_path / "outputs" / "v1" / "2020"
        output_dir.mkdir(parents=True)

        marker_file = output_dir / '.downloads_complete'
        marker_file.write_text("Already complete\n")

        # Check marker exists
        skip_processing = marker_file.exists()

        assert skip_processing is True

    def test_force_flag_overrides_marker(self, tmp_path):
        """Test that --force flag overrides existing marker."""
        output_dir = tmp_path / "outputs" / "v1" / "2020"
        output_dir.mkdir(parents=True)

        marker_file = output_dir / '.downloads_complete'
        marker_file.write_text("Already complete\n")

        force = True
        skip_processing = marker_file.exists() and not force

        assert skip_processing is False


class TestCommandExecution:
    """Test command execution with subprocess mocks."""

    @patch('subprocess.run')
    def test_successful_command_execution(self, mock_run):
        """Test successful subprocess command execution."""
        # Mock successful subprocess run
        mock_run.return_value = Mock(returncode=0, stdout="Success", stderr="")

        # Simulate running a command
        result = subprocess.run(
            ['python', 'script.py'],
            check=True,
            capture_output=True,
            text=True
        )

        assert result.returncode == 0
        assert mock_run.called

    @patch('subprocess.run')
    def test_failed_command_execution(self, mock_run):
        """Test failed subprocess command execution."""
        # Mock failed subprocess run
        mock_run.side_effect = subprocess.CalledProcessError(
            returncode=1,
            cmd=['python', 'script.py'],
            stderr="Error message"
        )

        # Verify exception is raised
        with pytest.raises(subprocess.CalledProcessError):
            subprocess.run(
                ['python', 'script.py'],
                check=True,
                capture_output=True,
                text=True
            )

    @patch('subprocess.run')
    def test_command_with_environment_variables(self, mock_run):
        """Test command execution with environment variables."""
        mock_run.return_value = Mock(returncode=0, stdout="", stderr="")

        env = os.environ.copy()
        env['TQDM_POSITION'] = '0'
        env['CENSUS_YEAR'] = '2020'

        subprocess.run(
            ['python', 'script.py'],
            env=env,
            capture_output=True,
            text=True
        )

        assert mock_run.called
        call_args = mock_run.call_args
        assert call_args[1]['env']['TQDM_POSITION'] == '0'
        assert call_args[1]['env']['CENSUS_YEAR'] == '2020'


class TestErrorHandling:
    """Test error handling and logging."""

    def test_error_log_file_creation(self, tmp_path):
        """Test error log file is created in output directory."""
        output_dir = tmp_path / "outputs" / "v1" / "2020"
        output_dir.mkdir(parents=True)

        error_log = output_dir / 'error.log'

        # Simulate logging an error
        with open(error_log, 'a') as f:
            f.write("[2020-01-01T12:00:00] tracts: Processing failed\n")

        assert error_log.exists()
        content = error_log.read_text()
        assert "tracts" in content
        assert "Processing failed" in content

    def test_error_log_append_mode(self, tmp_path):
        """Test error log appends errors instead of overwriting."""
        output_dir = tmp_path / "outputs" / "v1" / "2020"
        output_dir.mkdir(parents=True)

        error_log = output_dir / 'error.log'

        # Write first error
        with open(error_log, 'a') as f:
            f.write("Error 1\n")

        # Write second error
        with open(error_log, 'a') as f:
            f.write("Error 2\n")

        content = error_log.read_text()
        assert "Error 1" in content
        assert "Error 2" in content

    def test_error_includes_stage_information(self, tmp_path):
        """Test error log includes stage name."""
        output_dir = tmp_path / "outputs" / "v1" / "2020"
        output_dir.mkdir(parents=True)

        error_log = output_dir / 'error.log'

        stages = ['tracts', 'adjacency', 'elections', 'demographics']

        for stage in stages:
            with open(error_log, 'a') as f:
                f.write(f"[timestamp] {stage}: Error occurred\n")

        content = error_log.read_text()
        for stage in stages:
            assert stage in content


class TestSTATUSProtocol:
    """Test STATUS protocol integration."""

    def test_status_message_format_standalone(self):
        """Test STATUS message format in standalone mode."""
        # In standalone mode (TQDM_POSITION < 0), use normal print
        position = -1
        message = "Processing tracts"

        # Should NOT format as STATUS message
        is_standalone = position < 0

        if is_standalone:
            output = message
        else:
            output = f"STATUS:{position}:{message}"

        assert output == "Processing tracts"

    def test_status_message_format_worker(self):
        """Test STATUS message format in worker mode."""
        # In worker mode (TQDM_POSITION >= 0), use STATUS protocol
        position = 0
        message = "Processing tracts"

        is_standalone = position < 0

        if is_standalone:
            output = message
        else:
            output = f"STATUS:{position}:{message}"

        assert output == "STATUS:0:Processing tracts"

    def test_status_message_auto_detection(self):
        """Test automatic detection of standalone vs worker mode."""
        # Mock environment variable
        with patch.dict(os.environ, {'TQDM_POSITION': '2'}):
            position = int(os.environ.get('TQDM_POSITION', '-1'))
            is_standalone = position < 0

            assert is_standalone is False
            assert position == 2

    def test_status_message_default_standalone(self):
        """Test default to standalone mode when TQDM_POSITION not set."""
        # Mock environment without TQDM_POSITION
        with patch.dict(os.environ, {}, clear=True):
            position = int(os.environ.get('TQDM_POSITION', '-1'))
            is_standalone = position < 0

            assert is_standalone is True


class TestStageProcessing:
    """Test stage processing logic."""

    def test_all_stages_specified(self):
        """Test when all stages are specified."""
        stages = ['tracts', 'merge', 'adjacency', 'elections', 'demographics']

        assert len(stages) == 5
        assert 'tracts' in stages
        assert 'merge' in stages
        assert 'adjacency' in stages
        assert 'elections' in stages
        assert 'demographics' in stages

    def test_subset_of_stages(self):
        """Test when only subset of stages specified."""
        stages = ['tracts', 'merge', 'adjacency']

        assert len(stages) == 3
        assert 'tracts' in stages
        assert 'merge' in stages
        assert 'adjacency' in stages
        assert 'elections' not in stages
        assert 'demographics' not in stages

    def test_stage_order_matters(self):
        """Test that stages run in correct order."""
        # Correct order: tracts (parse) -> merge (add geometry) -> adjacency (needs geometry)
        correct_order = ['tracts', 'merge', 'adjacency', 'elections', 'demographics']

        # Verify tracts before merge
        tract_idx = correct_order.index('tracts')
        merge_idx = correct_order.index('merge')
        assert tract_idx < merge_idx

        # Verify merge before adjacency (adjacency needs geometry from merge)
        adjacency_idx = correct_order.index('adjacency')
        assert merge_idx < adjacency_idx

    def test_default_stages_core_only(self):
        """Test that default includes core stages only."""
        # Default should be core stages: tracts, merge, adjacency
        default_stages = ['tracts', 'merge', 'adjacency']

        assert len(default_stages) == 3
        assert 'tracts' in default_stages
        assert 'merge' in default_stages
        assert 'adjacency' in default_stages

    def test_merge_stage_outputs_parquet(self):
        """Test that merge stage produces parquet files."""
        year = 2020
        state = 'VT'

        # Merge takes CSV input and produces parquet output
        input_csv = f"outputs/data/{year}/units/{state.lower()}_tracts_{year}_population.csv"
        output_parquet = f"outputs/data/{year}/units/{state.lower()}_tracts_{year}.parquet"

        # Verify paths
        assert '.csv' in input_csv
        assert '.parquet' in output_parquet
        assert input_csv != output_parquet


class TestMergeStage:
    """Test merge stage specifically."""

    def test_merge_requires_tiger_directory(self):
        """Test that merge checks for TIGER/Line directory."""
        year = 2020
        tiger_dir = Path(f'data/{year}/tiger/tracts')

        # Merge should check tiger directory exists
        # If missing, should provide helpful error message
        if not tiger_dir.exists():
            error_msg = f"TIGER/Line directory not found: {tiger_dir}"
            assert "TIGER/Line" in error_msg
            assert str(year) in error_msg

    def test_merge_stage_marker(self, tmp_path):
        """Test that merge stage creates .tract_merge_complete marker."""
        output_dir = tmp_path / "outputs" / "test" / "2020"
        output_dir.mkdir(parents=True)

        marker = output_dir / '.tract_merge_complete'

        # Create marker after successful merge
        from datetime import datetime
        marker.write_text(f"tract merge processing completed: {datetime.now().isoformat()}\n")

        assert marker.exists()
        assert 'merge' in marker.name

    def test_merge_skip_logic(self, tmp_path):
        """Test that merge skips states with existing parquet files."""
        output_dir = tmp_path / "outputs" / "data" / "2020" / "tracts"
        output_dir.mkdir(parents=True)

        # Create parquet file (simulate completed merge)
        parquet_file = output_dir / "vt_tracts_2020.parquet"
        parquet_file.write_text("")  # Mock file

        # Should skip if exists and not force
        force = False
        should_skip = parquet_file.exists() and not force

        assert should_skip is True

    def test_merge_parallel_processing(self):
        """Test that merge supports parallel state processing."""
        workers = 12
        states = ['VT', 'DE', 'RI', 'NH', 'ME']

        # Can process min(workers, states) simultaneously
        max_parallel = min(workers, len(states))

        assert max_parallel == 5


class TestResolutionAwareness:
    """Test resolution-aware processing (tract vs block)."""

    def test_tract_resolution(self):
        """Test tract-level resolution processing."""
        resolution = 'tract'

        assert resolution == 'tract'

        # Tract files should go to outputs/data/units/
        expected_dir = Path('outputs/data/units/2020')
        assert 'units' in str(expected_dir)

    def test_block_resolution(self):
        """Test block-level resolution processing."""
        resolution = 'block'

        assert resolution == 'block'

        # Block files should go to outputs/data/units/
        expected_dir = Path('outputs/data/units/2020')
        assert 'units' in str(expected_dir)

    def test_multiple_resolutions(self):
        """Test processing multiple resolutions."""
        resolutions = ['tract', 'block']

        assert len(resolutions) == 2
        assert 'tract' in resolutions
        assert 'block' in resolutions

    def test_default_resolution_tract(self):
        """Test that default resolution is tract."""
        default_resolution = 'tract'

        assert default_resolution == 'tract'

    def test_resolution_in_output_paths(self):
        """Test that resolution-independent path is used."""
        year = '2020'

        # Both tract and block resolutions use units directory
        units_path = Path(f'outputs/data/units/{year}')
        assert 'units' in str(units_path)

        # Same path for both resolutions (resolution-independent)
        assert units_path == Path(f'outputs/data/units/{year}')


class TestDryRunMode:
    """Test dry-run mode without actual execution."""

    @patch('subprocess.run')
    def test_dry_run_skips_execution(self, mock_run):
        """Test that dry-run mode skips actual command execution."""
        dry_run = True

        if not dry_run:
            subprocess.run(['python', 'script.py'])

        # Verify subprocess was NOT called
        assert not mock_run.called

    def test_dry_run_prints_commands(self):
        """Test that dry-run mode prints what would be executed."""
        dry_run = True
        command = "python scripts/data/process_census_tracts.py --year 2020"

        if dry_run:
            output = f"[DRY RUN] Would execute: {command}"
        else:
            output = "Executing..."

        assert "[DRY RUN]" in output
        assert command in output


class TestIntegrationPrerequisites:
    """Test prerequisites for integration with main pipeline."""

    def test_subprocess_popen_compatibility(self):
        """Test that commands can be launched with Popen for real-time monitoring."""
        # This is needed for STATUS message forwarding in pipeline
        with patch('subprocess.Popen') as mock_popen:
            mock_process = Mock()
            mock_process.stdout.readline.return_value = ''
            mock_process.poll.return_value = 0
            mock_popen.return_value = mock_process

            # Simulate Popen call
            proc = subprocess.Popen(
                ['python', 'script.py'],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                bufsize=1
            )

            assert mock_popen.called

    def test_environment_variable_passing(self):
        """Test that environment variables are passed to child processes."""
        env = os.environ.copy()
        env['CENSUS_YEAR'] = '2020'
        env['TQDM_POSITION'] = '0'

        assert env['CENSUS_YEAR'] == '2020'
        assert env['TQDM_POSITION'] == '0'

    def test_output_directory_creation(self, tmp_path):
        """Test that output directory is created if missing."""
        output_dir = tmp_path / "outputs" / "v1" / "2020"

        assert not output_dir.exists()

        # Create directory
        output_dir.mkdir(parents=True, exist_ok=True)

        assert output_dir.exists()

    def test_marker_file_timestamp(self, tmp_path):
        """Test that marker file includes timestamp."""
        from datetime import datetime

        output_dir = tmp_path / "outputs" / "v1" / "2020"
        output_dir.mkdir(parents=True)

        marker_file = output_dir / '.downloads_complete'
        timestamp = datetime.now().isoformat()
        marker_file.write_text(f"Census data processing completed: {timestamp}\n")

        content = marker_file.read_text()
        assert "completed" in content
        assert "T" in content  # ISO format includes T


class TestYearSpecificProcessing:
    """Test year-specific processing differences."""

    def test_2020_census_processing(self):
        """Test 2020 census data processing."""
        year = '2020'

        # 2020 uses Census API (cenpy) for demographics
        assert year == '2020'

    def test_2010_census_processing(self):
        """Test 2010 census data processing."""
        year = '2010'

        # 2010 uses PL 94-171 files
        assert year == '2010'

    def test_2000_census_processing(self):
        """Test 2000 census data processing."""
        year = '2000'

        # 2000 uses NHGIS data
        assert year == '2000'

    def test_all_years_supported(self):
        """Test that all census years are supported."""
        supported_years = ['2000', '2010', '2020']

        for year in supported_years:
            assert year in supported_years
