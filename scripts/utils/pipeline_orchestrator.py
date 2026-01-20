"""
Unified orchestrator for multi-year, multi-stage redistricting pipeline.

This module provides a centralized approach to managing the three-stage pipeline:
1. Census data processing (parse + merge + adjacency)
2. State redistricting (recursive bisection for all states)
3. National post-processing (aggregation + visualization)

Each stage is a subprocess monitored via STATUS protocol. The orchestrator:
- Spawns processes with line-buffered stdout
- Monitors via daemon threads with readline loops
- Parses STATUS messages and updates coordinator
- Tracks completion and handles errors
- Manages phase transitions automatically

Usage:
    orchestrator = PipelineOrchestrator(
        coordinator=coordinator,
        display_lock=display_lock,
        years=['2020', '2010', '2000'],
        output_dirs={'2020': Path(...), ...}
    )

    # Configure stages
    orchestrator.add_stage('census', census_command_builder, census_handlers)
    orchestrator.add_stage('states', states_command_builder, state_handlers)
    orchestrator.add_stage('nation', nation_command_builder, nation_handlers)

    # Run pipeline
    results = orchestrator.run_pipeline(
        stages=['census', 'states', 'nation'],
        skip_stages_if_complete=True
    )
"""

import sys
import subprocess
import threading
import time
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Callable, Optional, Any, Tuple

from scripts.utils.progress_coordinator import parse_status_message


class ProcessMonitor:
    """
    Low-level subprocess monitor using STATUS protocol.

    Spawns a subprocess, creates a daemon thread to read stdout line-by-line,
    parses STATUS messages, and routes them to handlers.
    """

    def __init__(
        self,
        command: List[str],
        message_handlers: Dict[str, Callable],
        display_lock: threading.Lock,
        env: Optional[Dict[str, str]] = None
    ):
        """
        Initialize process monitor.

        Args:
            command: Command list for subprocess.Popen
            message_handlers: Dict[msg_type, callback(data)]
            display_lock: Threading lock for coordinator updates
            env: Optional environment variables
        """
        self.command = command
        self.message_handlers = message_handlers
        self.display_lock = display_lock
        self.env = env
        self.proc = None
        self.thread = None
        self.returncode = None

    def start(self) -> subprocess.Popen:
        """
        Start subprocess and monitoring thread.

        Returns:
            Popen object for the subprocess
        """
        # Launch process
        self.proc = subprocess.Popen(
            self.command,
            stdout=subprocess.PIPE,
            stderr=sys.stderr,
            text=True,
            bufsize=1,  # Line buffering
            env=self.env
        )

        # Start monitoring thread
        self.thread = threading.Thread(
            target=self._monitor_loop,
            daemon=True
        )
        self.thread.start()

        return self.proc

    def _monitor_loop(self):
        """Thread target: readline loop + message parsing + routing."""
        try:
            # Read stdout until process exits
            while True:
                line = self.proc.stdout.readline()
                if not line:
                    # Check if process has exited
                    if self.proc.poll() is not None:
                        break
                    continue

                line = line.strip()
                if not line:
                    continue

                # Echo ALL STATUS messages to stdout for parent process monitoring
                if line.startswith('STATUS:'):
                    sys.stdout.write(f"{line}\n")
                    sys.stdout.flush()

                # Parse and route STATUS message to handlers
                msg_type, data = parse_status_message(line)
                if msg_type and msg_type in self.message_handlers:
                    with self.display_lock:
                        self.message_handlers[msg_type](data)

            # Process has exited
            if self.proc.returncode is None:
                self.proc.wait()
            self.returncode = self.proc.returncode

        except Exception as e:
            sys.stderr.write(f"[ERROR] Process monitor thread died: {e}\n")
            sys.stderr.flush()

    def wait(self, timeout: Optional[float] = None) -> int:
        """
        Wait for process and thread to complete.

        Args:
            timeout: Optional timeout for thread join

        Returns:
            Process return code
        """
        if self.thread:
            self.thread.join(timeout=timeout)
        if self.proc and self.proc.returncode is None:
            self.proc.wait()
        return self.proc.returncode if self.proc else -1

    def poll(self) -> Optional[int]:
        """Check if process has completed. Returns returncode or None."""
        if self.proc:
            return self.proc.poll()
        return None


class PipelineOrchestrator:
    """
    High-level orchestrator for multi-year, multi-stage pipeline.

    Manages year-based pipeline where each year progresses through:
    census → states → nation

    Key features:
    - Sequential stage execution (all years must complete stage before next)
    - Marker file tracking to skip completed stages
    - Automatic phase transitions
    - Thread-safe coordinator updates
    - Unified error handling
    """

    def __init__(
        self,
        coordinator,
        display_lock: threading.Lock,
        years: List[str],
        output_dirs: Dict[str, Path]
    ):
        """
        Initialize pipeline orchestrator.

        Args:
            coordinator: ProgressCoordinator instance
            display_lock: Threading lock for display updates
            years: List of years to process (e.g., ['2020', '2010'])
            output_dirs: Dict mapping year -> output directory path
        """
        self.coordinator = coordinator
        self.display_lock = display_lock
        self.years = years
        self.output_dirs = output_dirs

        # Stage configuration: {stage_name: (command_builder, handlers, completion_callback)}
        self.stages: Dict[str, Tuple[Callable, Dict, Optional[Callable]]] = {}

        # Track year phases: {year: 'census'|'states'|'nation'|'complete'|'failed'}
        self.year_phase: Dict[str, str] = {}

        # Active monitors: {year: ProcessMonitor}
        self.monitors: Dict[str, ProcessMonitor] = {}

    def add_stage(
        self,
        stage_name: str,
        command_builder: Callable[[str], List[str]],
        message_handlers: Dict[str, Callable],
        completion_callback: Optional[Callable[[str, int], None]] = None
    ):
        """
        Register a pipeline stage.

        Args:
            stage_name: Stage identifier ('census', 'states', 'nation')
            command_builder: Function(year) -> command_list
            message_handlers: Dict[msg_type, callback(data)]
            completion_callback: Optional callback(year, returncode)
        """
        self.stages[stage_name] = (command_builder, message_handlers, completion_callback)

    def run_pipeline(
        self,
        stages: List[str],
        skip_stages_if_complete: bool = True,
        reset: bool = False,
        poll_interval: float = 0.5,
        display_update_callback: Optional[Callable] = None
    ) -> Dict[str, Dict[str, Any]]:
        """
        Run pipeline stages sequentially for all years.

        For each stage:
        1. Check if stage complete (marker files) - skip if requested
        2. Spawn subprocesses for all years needing the stage
        3. Monitor via STATUS protocol
        4. Wait for all years to complete stage
        5. Handle errors, update phase tracking
        6. Create marker files
        7. Proceed to next stage

        Args:
            stages: List of stage names to run ['census', 'states', 'nation']
            skip_stages_if_complete: Skip stages with marker files
            reset: Ignore marker files and rerun all stages
            poll_interval: Display update frequency (seconds)
            display_update_callback: Optional callback to refresh display

        Returns:
            Dict[year, Dict[stage, {'success': bool, 'returncode': int, 'error': str}]]
        """
        results = {year: {} for year in self.years}

        # Initialize year phases
        for year in self.years:
            self.year_phase[year] = stages[0] if stages else 'complete'

        # Process each stage sequentially
        for stage_name in stages:
            if stage_name not in self.stages:
                sys.stderr.write(f"[ERROR] Unknown stage: {stage_name}\n")
                sys.stderr.flush()
                continue

            # Check which years need this stage
            years_needing_stage = []
            for year in self.years:
                # Skip if year already failed
                if self.year_phase.get(year) == 'failed':
                    results[year][stage_name] = {
                        'success': False,
                        'returncode': -1,
                        'error': 'Previous stage failed'
                    }
                    continue

                # Check marker file
                if skip_stages_if_complete and not reset:
                    if self._check_stage_complete(year, stage_name):
                        # Stage already complete, skip
                        results[year][stage_name] = {
                            'success': True,
                            'returncode': 0,
                            'error': None
                        }
                        self._update_phase_for_skip(year, stage_name)
                        continue

                years_needing_stage.append(year)

            # If no years need this stage, continue to next
            if not years_needing_stage:
                continue

            # Spawn processes for years needing this stage
            self._spawn_stage_processes(stage_name, years_needing_stage)

            # Wait for all years to complete this stage
            stage_results = self._wait_for_stage_completion(
                stage_name,
                years_needing_stage,
                poll_interval,
                display_update_callback
            )

            # Process results and update phases
            for year in years_needing_stage:
                result = stage_results[year]
                results[year][stage_name] = result

                if result['success']:
                    # Create marker file
                    self._mark_stage_complete(year, stage_name)

                    # Call completion callback if provided
                    command_builder, handlers, completion_callback = self.stages[stage_name]
                    if completion_callback:
                        with self.display_lock:
                            completion_callback(year, result['returncode'])

                    # Update phase for next stage
                    self._update_phase_for_success(year, stage_name, stages)
                else:
                    # Mark year as failed
                    self.year_phase[year] = 'failed'

        return results

    def _spawn_stage_processes(self, stage_name: str, years: List[str]):
        """
        Spawn subprocess for each year for the given stage.

        Args:
            stage_name: Stage to spawn processes for
            years: Years to process
        """
        command_builder, message_handlers, _ = self.stages[stage_name]

        for year in years:
            # Build command for this year
            command = command_builder(year)

            # Create environment with subprocess flag
            import os
            env = os.environ.copy()
            env['MULTI_YEAR_SUBPROCESS'] = '1'
            env['TQDM_POSITION'] = '999'  # For deeply nested children

            # Create and start monitor
            monitor = ProcessMonitor(
                command=command,
                message_handlers=message_handlers,
                display_lock=self.display_lock,
                env=env
            )
            monitor.start()
            self.monitors[year] = monitor

    def _wait_for_stage_completion(
        self,
        stage_name: str,
        years: List[str],
        poll_interval: float,
        display_update_callback: Optional[Callable]
    ) -> Dict[str, Dict[str, Any]]:
        """
        Wait for all year processes to complete the current stage.

        Args:
            stage_name: Stage being processed
            years: Years to wait for
            poll_interval: Display update frequency
            display_update_callback: Optional callback to refresh display

        Returns:
            Dict[year, {'success': bool, 'returncode': int, 'error': str}]
        """
        results = {}
        active_years = set(years)

        # Wait for all processes to complete
        while active_years:
            # Update display if callback provided
            if display_update_callback:
                with self.display_lock:
                    display_update_callback()

            # Check which years are still active
            still_active = set()
            for year in active_years:
                monitor = self.monitors.get(year)
                if monitor and monitor.poll() is None:
                    still_active.add(year)
                else:
                    # Process completed
                    if monitor:
                        returncode = monitor.wait(timeout=2)
                        success = (returncode == 0)
                        results[year] = {
                            'success': success,
                            'returncode': returncode,
                            'error': None if success else f"{stage_name.capitalize()} processing failed with exit code {returncode}"
                        }
                    else:
                        results[year] = {
                            'success': False,
                            'returncode': -1,
                            'error': 'Monitor not created'
                        }

            active_years = still_active
            time.sleep(poll_interval)

        # Final display update
        if display_update_callback:
            with self.display_lock:
                display_update_callback()
            print()  # Blank line after display

        return results

    def _check_stage_complete(self, year: str, stage_name: str) -> bool:
        """
        Check if stage already complete (marker file exists).

        Args:
            year: Year to check
            stage_name: Stage to check ('census', 'states', 'nation')

        Returns:
            True if marker file exists
        """
        marker_file = self.output_dirs[year] / f'.{stage_name}_complete'
        return marker_file.exists()

    def _mark_stage_complete(self, year: str, stage_name: str):
        """
        Create marker file for completed stage.

        Args:
            year: Year that completed stage
            stage_name: Stage that was completed
        """
        marker_file = self.output_dirs[year] / f'.{stage_name}_complete'
        marker_file.parent.mkdir(parents=True, exist_ok=True)
        marker_file.write_text(
            f"{stage_name.capitalize()} processing completed: {datetime.now().isoformat()}\n"
        )

    def _update_phase_for_skip(self, year: str, stage_name: str):
        """
        Update coordinator when stage is skipped (already complete).

        Args:
            year: Year being processed
            stage_name: Stage being skipped
        """
        with self.display_lock:
            if stage_name == 'census':
                self.coordinator.census_complete(year)
            elif stage_name == 'states':
                self.coordinator.update_year_progress(year, 50, 50)
            elif stage_name == 'nation':
                # Get task count from coordinator
                progress = self.coordinator.year_progress.get(year, {})
                total_tasks = progress.get('total', 9)
                self.coordinator.update_year_postprocess(year, total_tasks, total_tasks)

    def _update_phase_for_success(self, year: str, stage_name: str, all_stages: List[str]):
        """
        Update phase after successful stage completion.

        Args:
            year: Year being processed
            stage_name: Stage that was completed
            all_stages: All stages in pipeline
        """
        # Determine next stage
        try:
            current_idx = all_stages.index(stage_name)
            if current_idx + 1 < len(all_stages):
                self.year_phase[year] = all_stages[current_idx + 1]
            else:
                self.year_phase[year] = 'complete'
        except ValueError:
            self.year_phase[year] = 'complete'

        # Update coordinator based on stage completed
        with self.display_lock:
            if stage_name == 'census':
                self.coordinator.census_complete(year)
            elif stage_name == 'states':
                self.coordinator.update_year_progress(year, 50, 50)
            elif stage_name == 'nation':
                # Get task count from coordinator
                progress = self.coordinator.year_progress.get(year, {})
                total_tasks = progress.get('total', 9)
                self.coordinator.update_year_postprocess(year, total_tasks, total_tasks)
