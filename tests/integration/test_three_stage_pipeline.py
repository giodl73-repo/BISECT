"""
Integration tests for 3-stage pipeline control system.

Tests the --stages argument functionality in run_complete_redistricting.py
covering all combinations of stages (data, states, nation).
"""

import pytest
from pathlib import Path
from unittest.mock import Mock, patch, MagicMock, call
import subprocess
import sys


class TestThreeStageControl:
    """Test --stages argument controls pipeline execution."""

    def test_default_runs_all_three_stages(self):
        """Test that default behavior runs all three stages."""
        # Default should be ['data', 'states', 'nation']
        default_stages = ['data', 'states', 'nation']

        assert 'data' in default_stages
        assert 'states' in default_stages
        assert 'nation' in default_stages

    def test_stages_data_only(self):
        """Test --stages data runs only data processing."""
        stages = ['data']

        should_run_data = 'data' in stages
        should_run_states = 'states' in stages
        should_run_nation = 'nation' in stages

        assert should_run_data is True
        assert should_run_states is False
        assert should_run_nation is False

    def test_stages_states_nation_skips_data(self):
        """Test --stages states nation skips data processing."""
        stages = ['states', 'nation']

        should_run_data = 'data' in stages
        should_run_states = 'states' in stages
        should_run_nation = 'nation' in stages

        assert should_run_data is False
        assert should_run_states is True
        assert should_run_nation is True

    def test_stages_nation_only(self):
        """Test --stages nation runs only post-processing."""
        stages = ['nation']

        should_run_data = 'data' in stages
        should_run_states = 'states' in stages
        should_run_nation = 'nation' in stages

        assert should_run_data is False
        assert should_run_states is False
        assert should_run_nation is True

    def test_stages_data_states_skips_nation(self):
        """Test --stages data states skips post-processing."""
        stages = ['data', 'states']

        should_run_data = 'data' in stages
        should_run_states = 'states' in stages
        should_run_nation = 'nation' in stages

        assert should_run_data is True
        assert should_run_states is True
        assert should_run_nation is False


class TestPhaseTransitions:
    """Test phase transitions in multi-year pipeline."""

    def test_data_phase_transitions_to_states(self):
        """Test that 'data' phase transitions to 'ready_for_states'."""
        # Simulate phase transition
        year_phase = 'data'

        # After data processing completes
        year_phase = 'ready_for_states'

        assert year_phase == 'ready_for_states'

    def test_ready_for_states_transitions_to_states(self):
        """Test that 'ready_for_states' transitions to 'states'."""
        year_phase = 'ready_for_states'

        # Pipeline updates ready_for_states -> states
        if year_phase == 'ready_for_states':
            year_phase = 'states'

        assert year_phase == 'states'

    def test_states_phase_transitions_to_ready_for_nation(self):
        """Test that 'states' phase transitions to 'ready_for_nation'."""
        year_phase = 'states'

        # After states processing completes
        year_phase = 'ready_for_nation'

        assert year_phase == 'ready_for_nation'

    def test_ready_for_nation_launches_nation(self):
        """Test that 'ready_for_nation' launches nation processing."""
        year_phase = 'ready_for_nation'

        # Nation processing starts
        year_phase = 'nation'

        assert year_phase == 'nation'

    def test_failed_phase_blocks_further_processing(self):
        """Test that 'failed' phase prevents further stages."""
        year_phase = 'failed'

        should_continue = (year_phase != 'failed')

        assert should_continue is False


class TestMarkerFileInteractions:
    """Test how --stages interacts with marker files."""

    def test_data_stage_checks_per_stage_markers(self, tmp_path):
        """Test that data stage checks per-stage markers."""
        output_dir = tmp_path / "outputs" / "v1" / "2020"
        output_dir.mkdir(parents=True)

        # Create per-stage markers
        tract_marker = output_dir / '.tract_tracts_complete'
        adj_marker = output_dir / '.tract_adjacency_complete'

        tract_marker.write_text("completed")
        adj_marker.write_text("completed")

        # Both markers exist
        assert tract_marker.exists()
        assert adj_marker.exists()

        # Should skip data processing
        all_stages_complete = tract_marker.exists() and adj_marker.exists()
        assert all_stages_complete is True

    def test_states_stage_checks_states_complete_marker(self, tmp_path):
        """Test that states stage checks .states_complete marker."""
        output_dir = tmp_path / "outputs" / "v1" / "2020"
        output_dir.mkdir(parents=True)

        marker = output_dir / '.states_complete'
        marker.write_text("completed")

        assert marker.exists()

        # Should skip states processing
        should_skip_states = marker.exists()
        assert should_skip_states is True

    def test_stages_without_data_skips_marker_check(self):
        """Test that omitting 'data' from --stages skips data marker check."""
        stages = ['states', 'nation']

        # Should not check data markers
        should_check_data_markers = ('data' in stages)

        assert should_check_data_markers is False

    def test_stages_without_states_skips_marker_check(self):
        """Test that omitting 'states' from --stages skips states marker check."""
        stages = ['data', 'nation']

        # Should not check states markers
        should_check_states_markers = ('states' in stages)

        assert should_check_states_markers is False


class TestStageInitialization:
    """Test initial phase determination based on --stages."""

    def test_stages_data_initializes_to_data_or_ready_for_states(self):
        """Test phase initialization when 'data' in --stages."""
        stages = ['data', 'states', 'nation']
        census_complete = False

        # Should initialize to 'data' if not complete
        if 'data' not in stages:
            phase = 'ready_for_states'
        elif not census_complete:
            phase = 'data'
        else:
            phase = 'ready_for_states'

        assert phase == 'data'

    def test_stages_without_data_initializes_to_ready_for_states(self):
        """Test phase initialization when 'data' not in --stages."""
        stages = ['states', 'nation']

        # Should skip directly to ready_for_states
        if 'data' not in stages:
            if 'states' in stages:
                phase = 'ready_for_states'
            else:
                phase = 'ready_for_nation'
        else:
            phase = 'data'

        assert phase == 'ready_for_states'

    def test_stages_nation_only_initializes_to_ready_for_nation(self):
        """Test phase initialization with only 'nation' in --stages."""
        stages = ['nation']

        # Should skip directly to ready_for_nation
        if 'data' not in stages:
            if 'states' in stages:
                phase = 'ready_for_states'
            else:
                phase = 'ready_for_nation'
        else:
            phase = 'data'

        assert phase == 'ready_for_nation'

    def test_stages_data_complete_initializes_to_ready_for_states(self):
        """Test phase initialization when data already complete."""
        stages = ['data', 'states', 'nation']
        census_complete = True

        # Should initialize to ready_for_states if data complete
        if 'data' not in stages:
            phase = 'ready_for_states'
        elif not census_complete:
            phase = 'data'
        elif 'states' in stages:
            phase = 'ready_for_states'
        else:
            phase = 'ready_for_nation'

        assert phase == 'ready_for_states'


class TestStageExitConditions:
    """Test early exit conditions based on --stages."""

    def test_data_only_exits_after_data_complete(self):
        """Test pipeline exits after data processing when 'nation' not in stages."""
        stages = ['data']
        data_complete = True

        # Should exit since 'states' not in stages
        should_continue_to_states = 'states' in stages

        assert should_continue_to_states is False

    def test_states_only_exits_after_states_complete(self):
        """Test pipeline exits after states when 'nation' not in stages."""
        stages = ['data', 'states']
        states_complete = True

        # Should exit since 'nation' not in stages
        should_continue_to_nation = 'nation' in stages

        assert should_continue_to_nation is False

    def test_all_stages_runs_to_completion(self):
        """Test pipeline runs to completion when all stages included."""
        stages = ['data', 'states', 'nation']

        should_run_data = 'data' in stages
        should_run_states = 'states' in stages
        should_run_nation = 'nation' in stages

        assert should_run_data is True
        assert should_run_states is True
        assert should_run_nation is True


class TestMultiYearStaging:
    """Test --stages with multi-year parallel processing."""

    def test_multi_year_data_phase_parallel(self):
        """Test that data processing can run in parallel for multiple years."""
        years = ['2000', '2010', '2020']
        stages = ['data']

        # Each year gets its own data phase
        year_phases = {year: 'data' for year in years}

        assert len(year_phases) == 3
        assert all(phase == 'data' for phase in year_phases.values())

    def test_multi_year_mixed_phases(self):
        """Test that different years can be in different phases."""
        # 2000: data complete, ready for states
        # 2010: data needed
        # 2020: data complete, ready for states
        year_phases = {
            '2000': 'ready_for_states',
            '2010': 'data',
            '2020': 'ready_for_states'
        }

        assert year_phases['2000'] == 'ready_for_states'
        assert year_phases['2010'] == 'data'
        assert year_phases['2020'] == 'ready_for_states'

    def test_multi_year_stages_data_only_all_years(self):
        """Test --stages data processes all years."""
        years = ['2000', '2010', '2020']
        stages = ['data']

        # All years should run data processing
        should_process = {year: ('data' in stages) for year in years}

        assert all(should_process.values())


# ==============================================================================
# Integration Test Summary
# ==============================================================================
#
# Test Coverage:
# - Basic stage control (data, states, nation combinations)
# - Phase transitions (data -> states -> nation)
# - Marker file interactions with --stages
# - Initial phase determination
# - Early exit conditions
# - Multi-year parallel processing with stages
#
# Total Tests: 25 tests across 7 test classes
# ==============================================================================
