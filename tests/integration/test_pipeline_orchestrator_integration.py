"""
Integration tests for pipeline orchestrator with real subprocesses.

Tests actual subprocess execution and STATUS message parsing.
"""

import pytest
import sys
import time
import threading
from pathlib import Path
from unittest.mock import Mock

# Add project root to path
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))

from scripts.utils.pipeline_orchestrator import ProcessMonitor, PipelineOrchestrator


class TestProcessMonitorIntegration:
    """Integration tests for ProcessMonitor with real subprocesses."""

    def test_monitor_real_subprocess_success(self):
        """Test monitoring a real subprocess that succeeds."""
        # Create a simple Python command that prints and exits
        command = [
            sys.executable, '-c',
            'import sys; print("Hello from subprocess"); sys.exit(0)'
        ]
        handlers = {}
        lock = threading.Lock()

        monitor = ProcessMonitor(command, handlers, lock)
        proc = monitor.start()

        # Wait for completion
        returncode = monitor.wait(timeout=5)

        assert returncode == 0
        assert not monitor.thread.is_alive()

    def test_monitor_real_subprocess_failure(self):
        """Test monitoring a real subprocess that fails."""
        # Create command that exits with error code
        command = [
            sys.executable, '-c',
            'import sys; sys.exit(1)'
        ]
        handlers = {}
        lock = threading.Lock()

        monitor = ProcessMonitor(command, handlers, lock)
        monitor.start()

        # Wait for completion
        returncode = monitor.wait(timeout=5)

        assert returncode == 1

    def test_monitor_parses_status_messages(self):
        """Test that monitor parses real STATUS messages."""
        # Create subprocess that emits STATUS messages
        script = '''
import sys
print("STATUS:YEAR:2020:COMPLETE:10/50", flush=True)
print("STATUS:YEAR:2020:COMPLETE:20/50", flush=True)
sys.exit(0)
'''
        command = [sys.executable, '-c', script]

        # Track received messages
        messages_received = []
        lock = threading.Lock()

        def year_handler(data):
            messages_received.append(('YEAR', data))

        handlers = {'YEAR': year_handler}

        monitor = ProcessMonitor(command, handlers, lock)
        monitor.start()

        # Wait for completion
        returncode = monitor.wait(timeout=5)

        assert returncode == 0
        assert len(messages_received) == 2
        assert messages_received[0][0] == 'YEAR'
        assert messages_received[0][1]['year'] == '2020'
        assert messages_received[0][1]['completed'] == 10
        assert messages_received[0][1]['total'] == 50

    def test_monitor_handles_mixed_output(self):
        """Test monitor handles mix of STATUS and regular output."""
        script = '''
import sys
print("Regular output line 1")
print("STATUS:YEAR:2020:COMPLETE:5/10", flush=True)
print("Regular output line 2")
sys.exit(0)
'''
        command = [sys.executable, '-c', script]

        messages_received = []
        lock = threading.Lock()

        def year_handler(data):
            messages_received.append(data)

        handlers = {'YEAR': year_handler}

        monitor = ProcessMonitor(command, handlers, lock)
        monitor.start()
        returncode = monitor.wait(timeout=5)

        assert returncode == 0
        # Should have parsed the STATUS message
        assert len(messages_received) == 1
        assert messages_received[0]['completed'] == 5


class TestPipelineOrchestratorIntegration:
    """Integration tests for PipelineOrchestrator with real subprocesses."""

    def test_orchestrator_single_stage_success(self, tmp_path):
        """Test orchestrator with single successful stage."""
        # Create mock coordinator
        coordinator = Mock()
        lock = threading.Lock()
        years = ['2020']

        output_dir = tmp_path / '2020'
        output_dir.mkdir()
        output_dirs = {'2020': output_dir}

        orchestrator = PipelineOrchestrator(coordinator, lock, years, output_dirs)

        # Define a simple successful stage
        def command_builder(year):
            return [
                sys.executable, '-c',
                f'import sys; print("Processing {year}"); sys.exit(0)'
            ]

        handlers = {}
        orchestrator.add_stage('test_stage', command_builder, handlers)

        # Run pipeline
        results = orchestrator.run_pipeline(
            stages=['test_stage'],
            skip_stages_if_complete=False,
            poll_interval=0.1
        )

        # Verify success
        assert results['2020']['test_stage']['success'] is True
        assert results['2020']['test_stage']['returncode'] == 0

        # Verify marker created
        marker = output_dir / '.test_stage_complete'
        assert marker.exists()

    def test_orchestrator_single_stage_failure(self, tmp_path):
        """Test orchestrator handles stage failure."""
        coordinator = Mock()
        lock = threading.Lock()
        years = ['2020']

        output_dir = tmp_path / '2020'
        output_dir.mkdir()
        output_dirs = {'2020': output_dir}

        orchestrator = PipelineOrchestrator(coordinator, lock, years, output_dirs)

        # Define failing stage
        def command_builder(year):
            return [
                sys.executable, '-c',
                'import sys; sys.exit(1)'
            ]

        handlers = {}
        orchestrator.add_stage('failing_stage', command_builder, handlers)

        # Run pipeline
        results = orchestrator.run_pipeline(
            stages=['failing_stage'],
            skip_stages_if_complete=False,
            poll_interval=0.1
        )

        # Verify failure
        assert results['2020']['failing_stage']['success'] is False
        assert results['2020']['failing_stage']['returncode'] == 1

        # Verify no marker created
        marker = output_dir / '.failing_stage_complete'
        assert not marker.exists()

    def test_orchestrator_multiple_years(self, tmp_path):
        """Test orchestrator processing multiple years."""
        coordinator = Mock()
        lock = threading.Lock()
        years = ['2020', '2010']

        # Create output directories
        output_dirs = {}
        for year in years:
            year_dir = tmp_path / year
            year_dir.mkdir()
            output_dirs[year] = year_dir

        orchestrator = PipelineOrchestrator(coordinator, lock, years, output_dirs)

        # Define stage
        def command_builder(year):
            return [
                sys.executable, '-c',
                f'import sys; print("Processing year {year}"); sys.exit(0)'
            ]

        handlers = {}
        orchestrator.add_stage('test_stage', command_builder, handlers)

        # Run pipeline
        results = orchestrator.run_pipeline(
            stages=['test_stage'],
            skip_stages_if_complete=False,
            poll_interval=0.1
        )

        # Verify both years succeeded
        assert results['2020']['test_stage']['success'] is True
        assert results['2010']['test_stage']['success'] is True

        # Verify markers created for both
        assert (output_dirs['2020'] / '.test_stage_complete').exists()
        assert (output_dirs['2010'] / '.test_stage_complete').exists()

    def test_orchestrator_sequential_stages(self, tmp_path):
        """Test orchestrator runs stages sequentially."""
        coordinator = Mock()
        lock = threading.Lock()
        years = ['2020']

        output_dir = tmp_path / '2020'
        output_dir.mkdir()
        output_dirs = {'2020': output_dir}

        orchestrator = PipelineOrchestrator(coordinator, lock, years, output_dirs)

        # Define two stages
        def stage1_command(year):
            # Stage 1 creates a file
            return [
                sys.executable, '-c',
                f'from pathlib import Path; '
                f'Path(r"{output_dir / "stage1_done.txt"}").write_text("done"); '
            ]

        def stage2_command(year):
            # Stage 2 checks that stage1 file exists
            return [
                sys.executable, '-c',
                f'from pathlib import Path; '
                f'assert Path(r"{output_dir / "stage1_done.txt"}").exists(); '
            ]

        handlers = {}
        orchestrator.add_stage('stage1', stage1_command, handlers)
        orchestrator.add_stage('stage2', stage2_command, handlers)

        # Run pipeline
        results = orchestrator.run_pipeline(
            stages=['stage1', 'stage2'],
            skip_stages_if_complete=False,
            poll_interval=0.1
        )

        # Verify both stages succeeded
        assert results['2020']['stage1']['success'] is True
        assert results['2020']['stage2']['success'] is True

        # Stage 2 success implies stage 1 ran first
        assert (output_dir / 'stage1_done.txt').exists()

    def test_orchestrator_skips_complete_stage(self, tmp_path):
        """Test orchestrator skips stage with existing marker."""
        coordinator = Mock()
        lock = threading.Lock()
        years = ['2020']

        output_dir = tmp_path / '2020'
        output_dir.mkdir()
        output_dirs = {'2020': output_dir}

        # Create marker file
        marker = output_dir / '.test_stage_complete'
        marker.write_text('Already complete\n')

        orchestrator = PipelineOrchestrator(coordinator, lock, years, output_dirs)

        # Define stage that would fail if executed
        def command_builder(year):
            return [sys.executable, '-c', 'import sys; sys.exit(1)']

        handlers = {}
        orchestrator.add_stage('test_stage', command_builder, handlers)

        # Run pipeline with skip enabled
        results = orchestrator.run_pipeline(
            stages=['test_stage'],
            skip_stages_if_complete=True,
            poll_interval=0.1
        )

        # Verify stage was skipped (success without executing)
        assert results['2020']['test_stage']['success'] is True
        assert results['2020']['test_stage']['returncode'] == 0

    def test_orchestrator_reset_ignores_marker(self, tmp_path):
        """Test orchestrator ignores marker when reset=True."""
        coordinator = Mock()
        lock = threading.Lock()
        years = ['2020']

        output_dir = tmp_path / '2020'
        output_dir.mkdir()
        output_dirs = {'2020': output_dir}

        # Create marker file
        marker = output_dir / '.test_stage_complete'
        marker.write_text('Old completion\n')
        old_content = marker.read_text()

        orchestrator = PipelineOrchestrator(coordinator, lock, years, output_dirs)

        # Define stage that succeeds
        def command_builder(year):
            return [sys.executable, '-c', 'import sys; sys.exit(0)']

        handlers = {}
        orchestrator.add_stage('test_stage', command_builder, handlers)

        # Run pipeline with reset=True
        results = orchestrator.run_pipeline(
            stages=['test_stage'],
            skip_stages_if_complete=True,  # This would skip...
            reset=True,  # But reset overrides it
            poll_interval=0.1
        )

        # Verify stage ran (marker file updated)
        assert results['2020']['test_stage']['success'] is True
        new_content = marker.read_text()
        assert new_content != old_content  # Marker was updated

    def test_orchestrator_with_status_messages(self, tmp_path):
        """Test orchestrator receives and routes STATUS messages."""
        coordinator = Mock()
        lock = threading.Lock()
        years = ['2020']

        output_dir = tmp_path / '2020'
        output_dir.mkdir()
        output_dirs = {'2020': output_dir}

        orchestrator = PipelineOrchestrator(coordinator, lock, years, output_dirs)

        # Track handler calls
        handler_calls = []

        def test_handler(data):
            handler_calls.append(data)

        # Define stage that emits STATUS messages
        def command_builder(year):
            script = '''
import sys
print("STATUS:YEAR:2020:COMPLETE:10/50", flush=True)
print("STATUS:YEAR:2020:COMPLETE:20/50", flush=True)
print("STATUS:YEAR:2020:COMPLETE:50/50", flush=True)
sys.exit(0)
'''
            return [sys.executable, '-c', script]

        handlers = {'YEAR': test_handler}
        orchestrator.add_stage('test_stage', command_builder, handlers)

        # Run pipeline
        results = orchestrator.run_pipeline(
            stages=['test_stage'],
            skip_stages_if_complete=False,
            poll_interval=0.1
        )

        # Verify success
        assert results['2020']['test_stage']['success'] is True

        # Verify handler received messages
        assert len(handler_calls) >= 1  # At least one STATUS message received

    def test_orchestrator_completion_callback(self, tmp_path):
        """Test orchestrator calls completion callback."""
        coordinator = Mock()
        lock = threading.Lock()
        years = ['2020']

        output_dir = tmp_path / '2020'
        output_dir.mkdir()
        output_dirs = {'2020': output_dir}

        orchestrator = PipelineOrchestrator(coordinator, lock, years, output_dirs)

        # Track callback calls
        callback_calls = []

        def completion_callback(year, returncode):
            callback_calls.append((year, returncode))

        # Define stage
        def command_builder(year):
            return [sys.executable, '-c', 'import sys; sys.exit(0)']

        handlers = {}
        orchestrator.add_stage('test_stage', command_builder, handlers, completion_callback)

        # Run pipeline
        results = orchestrator.run_pipeline(
            stages=['test_stage'],
            skip_stages_if_complete=False,
            poll_interval=0.1
        )

        # Verify callback was called
        assert len(callback_calls) == 1
        assert callback_calls[0] == ('2020', 0)


if __name__ == '__main__':
    pytest.main([__file__, '-v'])
