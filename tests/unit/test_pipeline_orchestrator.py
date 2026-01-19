"""
Unit tests for pipeline orchestrator module.

Tests both ProcessMonitor and PipelineOrchestrator classes.
"""

import pytest
import threading
import subprocess
import sys
from pathlib import Path
from unittest.mock import Mock, MagicMock, patch, call
from io import StringIO

# Add project root to path
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))

from scripts.utils.pipeline_orchestrator import ProcessMonitor, PipelineOrchestrator


class TestProcessMonitor:
    """Tests for ProcessMonitor class."""

    def test_init(self):
        """Test ProcessMonitor initialization."""
        command = ['python', '-c', 'print("test")']
        handlers = {'TEST': lambda data: None}
        lock = threading.Lock()

        monitor = ProcessMonitor(command, handlers, lock)

        assert monitor.command == command
        assert monitor.message_handlers == handlers
        assert monitor.display_lock == lock
        assert monitor.env is None
        assert monitor.proc is None
        assert monitor.thread is None
        assert monitor.returncode is None

    @patch('subprocess.Popen')
    def test_start_creates_process_and_thread(self, mock_popen):
        """Test that start() creates subprocess and monitoring thread."""
        command = ['python', '-c', 'print("test")']
        handlers = {}
        lock = threading.Lock()

        # Create mock process
        mock_proc = Mock()
        mock_proc.stdout = StringIO("test output\n")
        mock_proc.poll.return_value = None
        mock_popen.return_value = mock_proc

        monitor = ProcessMonitor(command, handlers, lock)
        proc = monitor.start()

        # Verify Popen was called correctly
        mock_popen.assert_called_once_with(
            command,
            stdout=subprocess.PIPE,
            stderr=sys.stderr,
            text=True,
            bufsize=1,
            env=None
        )

        # Verify process and thread created
        assert proc == mock_proc
        assert monitor.proc == mock_proc
        assert monitor.thread is not None
        assert monitor.thread.daemon is True

    @patch('subprocess.Popen')
    @patch('scripts.utils.pipeline_orchestrator.parse_status_message')
    def test_monitor_loop_parses_status_messages(self, mock_parse, mock_popen):
        """Test that monitor loop parses and routes STATUS messages."""
        command = ['python', '-c', 'print("test")']
        lock = threading.Lock()

        # Create handler mock
        handler_mock = Mock()
        handlers = {'TEST': handler_mock}

        # Create mock process with status output
        mock_proc = Mock()
        mock_proc.stdout = StringIO("STATUS:TEST:data\n")
        mock_proc.poll.side_effect = [None, 0]  # First call returns None, second returns 0
        mock_proc.returncode = 0
        mock_popen.return_value = mock_proc

        # Configure parse_status_message to return test data
        mock_parse.return_value = ('TEST', {'value': 'test_data'})

        monitor = ProcessMonitor(command, handlers, lock)
        monitor.start()

        # Wait for thread to complete
        monitor.wait(timeout=1)

        # Verify handler was called with parsed data
        handler_mock.assert_called_once_with({'value': 'test_data'})

    @patch('subprocess.Popen')
    def test_poll_returns_process_status(self, mock_popen):
        """Test that poll() returns process status."""
        command = ['python', '-c', 'print("test")']
        handlers = {}
        lock = threading.Lock()

        mock_proc = Mock()
        mock_proc.stdout = StringIO("")
        mock_proc.poll.return_value = None
        mock_popen.return_value = mock_proc

        monitor = ProcessMonitor(command, handlers, lock)
        monitor.start()

        assert monitor.poll() is None

        # Change poll to return exit code
        mock_proc.poll.return_value = 0
        assert monitor.poll() == 0

    @patch('subprocess.Popen')
    def test_wait_joins_thread(self, mock_popen):
        """Test that wait() waits for thread and process."""
        command = ['python', '-c', 'print("test")']
        handlers = {}
        lock = threading.Lock()

        mock_proc = Mock()
        mock_proc.stdout = StringIO("")
        mock_proc.poll.return_value = 0
        mock_proc.returncode = 0
        mock_popen.return_value = mock_proc

        monitor = ProcessMonitor(command, handlers, lock)
        monitor.start()

        returncode = monitor.wait(timeout=1)

        assert returncode == 0
        # Thread should have finished
        assert not monitor.thread.is_alive()


class TestPipelineOrchestrator:
    """Tests for PipelineOrchestrator class."""

    def test_init(self):
        """Test PipelineOrchestrator initialization."""
        coordinator = Mock()
        lock = threading.Lock()
        years = ['2020', '2010']
        output_dirs = {'2020': Path('/out/2020'), '2010': Path('/out/2010')}

        orchestrator = PipelineOrchestrator(coordinator, lock, years, output_dirs)

        assert orchestrator.coordinator == coordinator
        assert orchestrator.display_lock == lock
        assert orchestrator.years == years
        assert orchestrator.output_dirs == output_dirs
        assert orchestrator.stages == {}
        assert orchestrator.year_phase == {}
        assert orchestrator.monitors == {}

    def test_add_stage(self):
        """Test adding a stage to the orchestrator."""
        coordinator = Mock()
        lock = threading.Lock()
        years = ['2020']
        output_dirs = {'2020': Path('/out/2020')}

        orchestrator = PipelineOrchestrator(coordinator, lock, years, output_dirs)

        def command_builder(year):
            return ['echo', year]

        handlers = {'TEST': lambda data: None}
        completion_callback = Mock()

        orchestrator.add_stage('test_stage', command_builder, handlers, completion_callback)

        assert 'test_stage' in orchestrator.stages
        stored_builder, stored_handlers, stored_callback = orchestrator.stages['test_stage']
        assert stored_builder == command_builder
        assert stored_handlers == handlers
        assert stored_callback == completion_callback

    def test_check_stage_complete(self):
        """Test checking if stage is complete via marker file."""
        coordinator = Mock()
        lock = threading.Lock()
        years = ['2020']
        output_dirs = {'2020': Path('/tmp/test_output')}

        orchestrator = PipelineOrchestrator(coordinator, lock, years, output_dirs)

        # No marker file exists
        assert not orchestrator._check_stage_complete('2020', 'census')

    def test_mark_stage_complete(self, tmp_path):
        """Test creating stage completion marker file."""
        coordinator = Mock()
        lock = threading.Lock()
        years = ['2020']
        output_dir = tmp_path / '2020'
        output_dir.mkdir()
        output_dirs = {'2020': output_dir}

        orchestrator = PipelineOrchestrator(coordinator, lock, years, output_dirs)

        orchestrator._mark_stage_complete('2020', 'census')

        marker_file = output_dir / '.census_complete'
        assert marker_file.exists()
        content = marker_file.read_text()
        assert 'Census processing completed' in content

    def test_update_phase_for_skip_census(self):
        """Test updating coordinator when census stage is skipped."""
        coordinator = Mock()
        lock = threading.Lock()
        years = ['2020']
        output_dirs = {'2020': Path('/out/2020')}

        orchestrator = PipelineOrchestrator(coordinator, lock, years, output_dirs)

        orchestrator._update_phase_for_skip('2020', 'census')

        coordinator.census_complete.assert_called_once_with('2020')

    def test_update_phase_for_skip_states(self):
        """Test updating coordinator when states stage is skipped."""
        coordinator = Mock()
        lock = threading.Lock()
        years = ['2020']
        output_dirs = {'2020': Path('/out/2020')}

        orchestrator = PipelineOrchestrator(coordinator, lock, years, output_dirs)

        orchestrator._update_phase_for_skip('2020', 'states')

        coordinator.update_year_progress.assert_called_once_with('2020', 50, 50)

    def test_update_phase_for_skip_nation(self):
        """Test updating coordinator when nation stage is skipped."""
        coordinator = Mock()
        coordinator.year_progress = {'2020': {'total': 9}}
        lock = threading.Lock()
        years = ['2020']
        output_dirs = {'2020': Path('/out/2020')}

        orchestrator = PipelineOrchestrator(coordinator, lock, years, output_dirs)

        orchestrator._update_phase_for_skip('2020', 'nation')

        coordinator.update_year_postprocess.assert_called_once_with('2020', 9, 9)

    def test_update_phase_for_success(self):
        """Test updating phase after successful stage completion."""
        coordinator = Mock()
        lock = threading.Lock()
        years = ['2020']
        output_dirs = {'2020': Path('/out/2020')}

        orchestrator = PipelineOrchestrator(coordinator, lock, years, output_dirs)
        orchestrator.year_phase = {'2020': 'census'}

        all_stages = ['census', 'states', 'nation']
        orchestrator._update_phase_for_success('2020', 'census', all_stages)

        # Should transition to states
        assert orchestrator.year_phase['2020'] == 'states'
        coordinator.census_complete.assert_called_once_with('2020')

    @patch('scripts.utils.pipeline_orchestrator.ProcessMonitor')
    def test_spawn_stage_processes(self, mock_monitor_class):
        """Test spawning processes for a stage."""
        coordinator = Mock()
        lock = threading.Lock()
        years = ['2020', '2010']
        output_dirs = {'2020': Path('/out/2020'), '2010': Path('/out/2010')}

        orchestrator = PipelineOrchestrator(coordinator, lock, years, output_dirs)

        def command_builder(year):
            return ['echo', year]

        handlers = {}
        orchestrator.add_stage('test_stage', command_builder, handlers)

        # Create mock monitors
        mock_monitor_2020 = Mock()
        mock_monitor_2010 = Mock()
        mock_monitor_class.side_effect = [mock_monitor_2020, mock_monitor_2010]

        orchestrator._spawn_stage_processes('test_stage', years)

        # Verify monitors created and started
        assert mock_monitor_class.call_count == 2
        mock_monitor_2020.start.assert_called_once()
        mock_monitor_2010.start.assert_called_once()

        # Verify monitors stored
        assert orchestrator.monitors['2020'] == mock_monitor_2020
        assert orchestrator.monitors['2010'] == mock_monitor_2010

    def test_wait_for_stage_completion(self):
        """Test waiting for stage completion."""
        coordinator = Mock()
        lock = threading.Lock()
        years = ['2020']
        output_dirs = {'2020': Path('/out/2020')}

        orchestrator = PipelineOrchestrator(coordinator, lock, years, output_dirs)

        # Create mock monitor that completes immediately
        mock_monitor = Mock()
        mock_monitor.poll.return_value = 0  # Already completed
        mock_monitor.wait.return_value = 0
        orchestrator.monitors = {'2020': mock_monitor}

        results = orchestrator._wait_for_stage_completion(
            'test_stage',
            years,
            poll_interval=0.01,
            display_update_callback=None
        )

        assert '2020' in results
        assert results['2020']['success'] is True
        assert results['2020']['returncode'] == 0

    @patch('scripts.utils.pipeline_orchestrator.ProcessMonitor')
    def test_run_pipeline_single_stage(self, mock_monitor_class, tmp_path):
        """Test running pipeline with single stage."""
        coordinator = Mock()
        lock = threading.Lock()
        years = ['2020']
        output_dir = tmp_path / '2020'
        output_dir.mkdir()
        output_dirs = {'2020': output_dir}

        orchestrator = PipelineOrchestrator(coordinator, lock, years, output_dirs)

        def command_builder(year):
            return ['echo', year]

        handlers = {}
        orchestrator.add_stage('test_stage', command_builder, handlers)

        # Mock monitor that succeeds
        mock_monitor = Mock()
        mock_monitor.poll.return_value = 0
        mock_monitor.wait.return_value = 0
        mock_monitor_class.return_value = mock_monitor

        results = orchestrator.run_pipeline(
            stages=['test_stage'],
            skip_stages_if_complete=False,
            poll_interval=0.01
        )

        # Verify results
        assert '2020' in results
        assert 'test_stage' in results['2020']
        assert results['2020']['test_stage']['success'] is True

        # Verify marker file created
        marker_file = output_dir / '.test_stage_complete'
        assert marker_file.exists()

    @patch('scripts.utils.pipeline_orchestrator.ProcessMonitor')
    def test_run_pipeline_skip_complete_stage(self, mock_monitor_class, tmp_path):
        """Test pipeline skips stage when marker file exists."""
        coordinator = Mock()
        lock = threading.Lock()
        years = ['2020']
        output_dir = tmp_path / '2020'
        output_dir.mkdir()
        output_dirs = {'2020': output_dir}

        # Create marker file
        marker_file = output_dir / '.test_stage_complete'
        marker_file.write_text('Already complete\n')

        orchestrator = PipelineOrchestrator(coordinator, lock, years, output_dirs)

        def command_builder(year):
            return ['echo', year]

        handlers = {}
        orchestrator.add_stage('test_stage', command_builder, handlers)

        results = orchestrator.run_pipeline(
            stages=['test_stage'],
            skip_stages_if_complete=True,
            poll_interval=0.01
        )

        # Verify stage was skipped
        assert '2020' in results
        assert 'test_stage' in results['2020']
        assert results['2020']['test_stage']['success'] is True

        # Verify no process was spawned
        mock_monitor_class.assert_not_called()

    @patch('scripts.utils.pipeline_orchestrator.ProcessMonitor')
    def test_run_pipeline_failed_stage(self, mock_monitor_class, tmp_path):
        """Test pipeline handles failed stage."""
        coordinator = Mock()
        lock = threading.Lock()
        years = ['2020']
        output_dir = tmp_path / '2020'
        output_dir.mkdir()
        output_dirs = {'2020': output_dir}

        orchestrator = PipelineOrchestrator(coordinator, lock, years, output_dirs)

        def command_builder(year):
            return ['false']  # Command that fails

        handlers = {}
        orchestrator.add_stage('failing_stage', command_builder, handlers)

        # Mock monitor that fails
        mock_monitor = Mock()
        mock_monitor.poll.return_value = 1  # Non-zero exit code
        mock_monitor.wait.return_value = 1
        mock_monitor_class.return_value = mock_monitor

        results = orchestrator.run_pipeline(
            stages=['failing_stage'],
            skip_stages_if_complete=False,
            poll_interval=0.01
        )

        # Verify failure recorded
        assert '2020' in results
        assert 'failing_stage' in results['2020']
        assert results['2020']['failing_stage']['success'] is False
        assert results['2020']['failing_stage']['returncode'] == 1

        # Verify marker file NOT created
        marker_file = output_dir / '.failing_stage_complete'
        assert not marker_file.exists()

    @patch('scripts.utils.pipeline_orchestrator.ProcessMonitor')
    def test_run_pipeline_sequential_stages(self, mock_monitor_class, tmp_path):
        """Test pipeline runs stages sequentially."""
        coordinator = Mock()
        lock = threading.Lock()
        years = ['2020']
        output_dir = tmp_path / '2020'
        output_dir.mkdir()
        output_dirs = {'2020': output_dir}

        orchestrator = PipelineOrchestrator(coordinator, lock, years, output_dirs)

        def command_builder(year):
            return ['echo', year]

        handlers = {}
        orchestrator.add_stage('stage1', command_builder, handlers)
        orchestrator.add_stage('stage2', command_builder, handlers)

        # Mock monitors that succeed
        mock_monitor1 = Mock()
        mock_monitor1.poll.return_value = 0
        mock_monitor1.wait.return_value = 0

        mock_monitor2 = Mock()
        mock_monitor2.poll.return_value = 0
        mock_monitor2.wait.return_value = 0

        mock_monitor_class.side_effect = [mock_monitor1, mock_monitor2]

        results = orchestrator.run_pipeline(
            stages=['stage1', 'stage2'],
            skip_stages_if_complete=False,
            poll_interval=0.01
        )

        # Verify both stages completed
        assert 'stage1' in results['2020']
        assert 'stage2' in results['2020']
        assert results['2020']['stage1']['success'] is True
        assert results['2020']['stage2']['success'] is True

        # Verify markers created for both
        assert (output_dir / '.stage1_complete').exists()
        assert (output_dir / '.stage2_complete').exists()


if __name__ == '__main__':
    pytest.main([__file__, '-v'])
