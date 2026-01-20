"""
Run Manager - Manage pipeline subprocess execution and progress tracking.

Spawns run_complete_redistricting.py as subprocess, parses STATUS protocol
messages, and writes progress to JSON file for frontend polling.
"""

import subprocess
import sys
import os
import json
import threading
import time
from pathlib import Path
from datetime import datetime, timedelta
from typing import Dict, Any, Optional

# Add project root to path
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))

from scripts.utils.status_protocol import parse_status_message

# All 50 US states (for tracking skipped census stages)
ALL_STATES = [
    'AL', 'AK', 'AZ', 'AR', 'CA', 'CO', 'CT', 'DE', 'FL', 'GA',
    'HI', 'ID', 'IL', 'IN', 'IA', 'KS', 'KY', 'LA', 'ME', 'MD',
    'MA', 'MI', 'MN', 'MS', 'MO', 'MT', 'NE', 'NV', 'NH', 'NJ',
    'NM', 'NY', 'NC', 'ND', 'OH', 'OK', 'OR', 'PA', 'RI', 'SC',
    'SD', 'TN', 'TX', 'UT', 'VT', 'VA', 'WA', 'WV', 'WI', 'WY'
]


class RunManager:
    """Manages pipeline run execution and progress tracking."""

    def _ensure_all_states_tracked(self, stage_tasks: list, stage_num: int, stage_name: str) -> int:
        """
        Ensure all 50 US states are tracked in a stage's tasks.

        Args:
            stage_tasks: List of task entries (will be modified in place)
            stage_num: Stage number (1-3 for census, 1-7 for redistricting)
            stage_name: Stage name for logging

        Returns:
            Number of states added
        """
        existing_states = {task['state'].upper() for task in stage_tasks}  # Normalize to uppercase
        missing_states = [s for s in ALL_STATES if s not in existing_states]

        if missing_states:
            print(f"[ASSERT] Stage '{stage_name}' missing {len(missing_states)} states - adding them", flush=True)
            for state_code in missing_states:
                stage_tasks.append({
                    'state': state_code,
                    'worker_id': -1,  # Indicates auto-filled/skipped
                    'stage': stage_num,
                    'completed_at': datetime.now().isoformat()
                })
            print(f"[ASSERT] Added: {', '.join(missing_states)}", flush=True)
            return len(missing_states)
        else:
            print(f"[ASSERT] Stage '{stage_name}' has all 50 states tracked ✓", flush=True)
            return 0

    def _normalize_state_code(self, state_name: str) -> str:
        """
        Normalize state name to uppercase 2-letter code.

        Args:
            state_name: State name (can be "CA", "california", "California", etc.)

        Returns:
            Uppercase 2-letter state code
        """
        state_name = state_name.strip().upper()

        # Already a 2-letter code
        if len(state_name) == 2:
            return state_name

        # Map full names to codes
        state_map = {
            'ALABAMA': 'AL', 'ALASKA': 'AK', 'ARIZONA': 'AZ', 'ARKANSAS': 'AR', 'CALIFORNIA': 'CA',
            'COLORADO': 'CO', 'CONNECTICUT': 'CT', 'DELAWARE': 'DE', 'FLORIDA': 'FL', 'GEORGIA': 'GA',
            'HAWAII': 'HI', 'IDAHO': 'ID', 'ILLINOIS': 'IL', 'INDIANA': 'IN', 'IOWA': 'IA',
            'KANSAS': 'KS', 'KENTUCKY': 'KY', 'LOUISIANA': 'LA', 'MAINE': 'ME', 'MARYLAND': 'MD',
            'MASSACHUSETTS': 'MA', 'MICHIGAN': 'MI', 'MINNESOTA': 'MN', 'MISSISSIPPI': 'MS', 'MISSOURI': 'MO',
            'MONTANA': 'MT', 'NEBRASKA': 'NE', 'NEVADA': 'NV', 'NEW_HAMPSHIRE': 'NH', 'NEW_JERSEY': 'NJ',
            'NEW_MEXICO': 'NM', 'NEW_YORK': 'NY', 'NORTH_CAROLINA': 'NC', 'NORTH_DAKOTA': 'ND', 'OHIO': 'OH',
            'OKLAHOMA': 'OK', 'OREGON': 'OR', 'PENNSYLVANIA': 'PA', 'RHODE_ISLAND': 'RI', 'SOUTH_CAROLINA': 'SC',
            'SOUTH_DAKOTA': 'SD', 'TENNESSEE': 'TN', 'TEXAS': 'TX', 'UTAH': 'UT', 'VERMONT': 'VT',
            'VIRGINIA': 'VA', 'WASHINGTON': 'WA', 'WEST_VIRGINIA': 'WV', 'WISCONSIN': 'WI', 'WYOMING': 'WY'
        }

        return state_map.get(state_name, state_name[:2])  # Fallback to first 2 chars

    def __init__(self, progress_file: Path):
        """
        Initialize run manager.

        Args:
            progress_file: Path to JSON file for progress storage
        """
        self.progress_file = progress_file
        self.active_run = None
        self.process = None
        self.lock = threading.Lock()

        # Load existing progress
        self._load_progress()

        # Clean up any stale runs from previous server instance
        self._cleanup_stale_runs()

    def _cleanup_stale_runs(self):
        """Clean up stale active runs from previous server instance."""
        if self.active_run:
            run_id = self.active_run.get('run_id', 'unknown')
            print(f"[INFO] Found stale active run: {run_id}", flush=True)
            print(f"[INFO] Moving stale run to history (server restarted)", flush=True)

            # Mark as cancelled/failed since server was restarted
            self.active_run['end_time'] = datetime.now().isoformat()
            self.active_run['status'] = 'cancelled'
            self.active_run['error'] = 'Server restarted - run was terminated'

            # Delete output files for cancelled run
            version = self.active_run.get('version', 'unknown')
            self._delete_run_outputs(version)

            # Move to history
            self.history.append(self.active_run)
            self.active_run = None

            # Save cleaned progress
            self._save_progress()
            print(f"[INFO] Stale run cleaned up and moved to history", flush=True)

        # Also clean up any cancelled runs in history
        cancelled_runs = [run for run in self.history if run.get('status') == 'cancelled']
        if cancelled_runs:
            print(f"[INFO] Found {len(cancelled_runs)} cancelled runs in history", flush=True)
            for run in cancelled_runs:
                version = run.get('version', 'unknown')
                run_id = run.get('run_id', 'unknown')
                print(f"[INFO] Cleaning up cancelled run: {run_id} (version: {version})", flush=True)
                self._delete_run_outputs(version)

    def _load_progress(self):
        """Load progress from JSON file."""
        if self.progress_file.exists():
            try:
                with open(self.progress_file, 'r') as f:
                    data = json.load(f)
                    self.active_run = data.get('active_run')
                    self.history = data.get('history', [])

                    # Ensure current_stage and completed_stages exist for all years (migration for old runs)
                    if self.active_run and 'years' in self.active_run:
                        print(f"[ASSERT] Loaded progress file - validating data integrity", flush=True)
                        for year, year_data in self.active_run['years'].items():
                            if 'current_stage' not in year_data:
                                year_data['current_stage'] = None
                            if 'completed_stages' not in year_data:
                                year_data['completed_stages'] = []

                            # Validate completed stages have all states when appropriate
                            for stage in year_data.get('completed_stages', []):
                                if stage.get('tasks'):
                                    state_count = len(stage['tasks'])
                                    if state_count < 50:
                                        print(f"[ASSERT] WARNING: {year} stage '{stage['name']}' only has {state_count}/50 states", flush=True)
                                    elif state_count == 50:
                                        print(f"[ASSERT] {year} stage '{stage['name']}' has all 50 states ✓", flush=True)

                            # Validate current stage
                            if year_data.get('current_stage') and year_data['current_stage'].get('tasks'):
                                stage = year_data['current_stage']
                                state_count = len(stage['tasks'])
                                print(f"[ASSERT] {year} current stage '{stage['name']}' has {state_count} states tracked", flush=True)
            except Exception as e:
                print(f"[ERROR] Failed to load progress: {e}", flush=True)
                self.active_run = None
                self.history = []
        else:
            self.active_run = None
            self.history = []

    def _save_progress(self):
        """Save progress to JSON file."""
        print(f"[DEBUG] _save_progress() called", flush=True)
        with self.lock:
            try:
                data = {
                    'active_run': self.active_run,
                    'history': self.history[-50:]  # Keep last 50 runs
                }
                if self.active_run is not None:
                    print(f"[DEBUG] About to write JSON, active_run has {len(self.active_run.get('workers', {}))} workers", flush=True)
                else:
                    print(f"[DEBUG] About to write JSON, active_run is None", flush=True)
                # Write atomically
                temp_file = self.progress_file.with_suffix('.tmp')
                with open(temp_file, 'w') as f:
                    json.dump(data, f, indent=2)
                temp_file.replace(self.progress_file)
                print(f"[DEBUG] JSON file written successfully", flush=True)
            except Exception as e:
                print(f"[ERROR] Failed to save progress: {e}", flush=True)
                import traceback
                print(f"[ERROR] Traceback: {traceback.format_exc()}", flush=True)

    def _delete_run_outputs(self, version: str):
        """
        Delete output files for a run version.

        Args:
            version: Version string (e.g., 'v1', 'test')
        """
        import shutil

        outputs_dir = project_root / 'outputs' / version
        if outputs_dir.exists():
            try:
                print(f"[INFO] Deleting outputs directory: {outputs_dir}", flush=True)
                shutil.rmtree(outputs_dir)
                print(f"[INFO] Successfully deleted outputs for version: {version}", flush=True)
            except Exception as e:
                print(f"[ERROR] Failed to delete outputs for {version}: {e}", flush=True)
        else:
            print(f"[INFO] No outputs directory found for version: {version}", flush=True)

    def suggest_version_name(self) -> str:
        """
        Suggest the next available version name based on existing outputs.

        Scans outputs/ directory for V{N} directories and suggests V{N+1}.
        Ignores non-V{N} version names (like 'test', 'dev', 'prod').

        Returns:
            Suggested version name (e.g., 'V1', 'V2', 'V3')
        """
        import re

        outputs_dir = project_root / 'outputs'

        # Find all V{N} or v{N} directories (case-insensitive)
        version_numbers = []
        if outputs_dir.exists():
            for item in outputs_dir.iterdir():
                if item.is_dir():
                    match = re.match(r'^[Vv](\d+)$', item.name, re.IGNORECASE)
                    if match:
                        version_numbers.append(int(match.group(1)))

        if not version_numbers:
            # No V{N} versions found - suggest V1
            return 'V1'

        # Return next available number (uppercase V)
        max_version = max(version_numbers)
        return f'V{max_version + 1}'

    def start_run(self, config: Dict[str, Any]) -> tuple[bool, str]:
        """
        Start a new pipeline run.

        Args:
            config: Run configuration
                - version: str
                - years: list[str]
                - states: list[str] (optional)
                - workers: int
                - dpi: int
                - partition_mode: str

        Returns:
            Tuple of (success, message_or_run_id)
        """
        with self.lock:
            if self.active_run and self.active_run.get('status') == 'running':
                return (False, "A run is already in progress")

        # Generate run ID
        run_id = f"run_{datetime.now().strftime('%Y%m%d_%H%M%S')}"

        # Initialize progress structure
        self.active_run = {
            'run_id': run_id,
            'version': config['version'],
            'config': config,
            'start_time': datetime.now().isoformat(),
            'end_time': None,
            'status': 'running',
            'years': {year: {
                'completed': 0,
                'total': 50,
                'status': 'pending',
                'current_stage': None,  # Current stage with tasks: {'name': 'Merging geometries', 'tasks': [...]}
                'completed_stages': []  # Completed stages with tasks: [{'name': 'Parsing', 'tasks': [...], 'completed_at': ...}]
            } for year in config['years']},
            'workers': {},
            'census_progress': {},
            'error': None
        }

        self._save_progress()

        # Build command - use Poetry venv Python explicitly
        scripts_dir = project_root / 'scripts' / 'pipeline'

        # Find Poetry venv Python
        poetry_python = Path.home() / 'AppData' / 'Local' / 'Packages' / 'PythonSoftwareFoundation.Python.3.13_qbz5n2kfra8p0' / 'LocalCache' / 'Roaming' / 'pypoetry' / 'venv' / 'Scripts' / 'python.exe'
        python_exe = str(poetry_python) if poetry_python.exists() else sys.executable
        print(f"[DEBUG] Using Python: {python_exe} (exists: {Path(python_exe).exists()})", flush=True)

        cmd = [
            python_exe,
            str(scripts_dir / 'run_complete_redistricting.py'),
            '--version', config['version'],
            '--dpi', str(config.get('dpi', 150)),
            '--partition-mode', config.get('partition_mode', 'edge-weighted')
        ]

        # Add year (note: singular --year, not --years)
        # The script supports "all" to run all three years in parallel
        print(f"[DEBUG] config['years'] = {config['years']}, length = {len(config['years'])}", flush=True)
        if config['years']:
            if len(config['years']) == 3:
                # All three years selected - use "all"
                print(f"[DEBUG] Using --year all (3 years selected)", flush=True)
                cmd.extend(['--year', 'all'])
            elif len(config['years']) == 1:
                # Single year
                cmd.extend(['--year', config['years'][0]])
            else:
                # Multiple but not all - just use first for now
                # TODO: Run multiple separate processes for partial year selection
                cmd.extend(['--year', config['years'][0]])
                print(f"[WARN] Multiple years selected but not all three. Running {config['years'][0]} only.", flush=True)

        # Add stages if specified
        if config.get('stages'):
            cmd.append('--stages')
            cmd.extend(config['stages'])

        # Add states if specified
        if config.get('states'):
            cmd.append('-st')
            cmd.extend(config['states'])

        # Add workers
        if config.get('workers'):
            cmd.extend(['--workers', str(config['workers'])])

        # Start subprocess in background thread
        print(f"[DEBUG] Starting background thread for run {run_id}", flush=True)
        thread = threading.Thread(target=self._run_subprocess, args=(cmd, run_id))
        thread.daemon = True
        thread.start()
        print(f"[DEBUG] Background thread started", flush=True)

        return (True, run_id)

    def _run_subprocess(self, cmd: list, run_id: str):
        """
        Run subprocess and parse STATUS messages.

        Args:
            cmd: Command to execute
            run_id: Run ID
        """
        print(f"[DEBUG] Starting subprocess for run {run_id}", flush=True)
        print(f"[DEBUG] Command: {' '.join(cmd)}", flush=True)
        print(f"[DEBUG] Working directory: {Path.cwd()}", flush=True)

        try:
            # Spawn subprocess with unbuffered output
            env = os.environ.copy()
            env['PYTHONUNBUFFERED'] = '1'  # Force unbuffered output

            self.process = subprocess.Popen(
                cmd,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                bufsize=1,
                env=env
            )

            print(f"[DEBUG] Subprocess spawned with PID: {self.process.pid}", flush=True)

            # Parse STATUS messages and capture stderr in parallel
            import threading
            stderr_lines = []

            def read_stderr():
                for line in self.process.stderr:
                    stderr_lines.append(line)
                    print(f"[DEBUG] STDERR: {line.strip()}", flush=True)

            stderr_thread = threading.Thread(target=read_stderr)
            stderr_thread.daemon = True
            stderr_thread.start()

            # Start progress poller thread
            stop_polling = threading.Event()
            def poll_progress():
                while not stop_polling.is_set():
                    self._poll_states_complete(run_id)
                    time.sleep(2)  # Poll every 2 seconds

            progress_thread = threading.Thread(target=poll_progress)
            progress_thread.daemon = True
            progress_thread.start()

            line_count = 0
            for line in self.process.stdout:
                line_count += 1
                if line_count <= 10:  # Log first 10 lines
                    print(f"[DEBUG] STDOUT line {line_count}: {line.strip()[:100]}", flush=True)
                elif line_count % 100 == 0:  # Log every 100th line
                    print(f"[DEBUG] STDOUT line {line_count}: {line.strip()[:100]}", flush=True)

                # Parse worker status lines like: "+- Worker 1: [04/50] Arizona | Stage 1/7: Redistricting"
                stripped = line.strip()
                if 'Worker' in stripped and 'Stage' in stripped:
                    print(f"[DEBUG] Parsing worker line: {stripped[:150]}", flush=True)
                    self._parse_worker_line(stripped, run_id)

                # Try to parse as STATUS message
                if stripped.startswith('STATUS:'):
                    print(f"[DEBUG] Found STATUS line {line_count}: {stripped[:200]}", flush=True)

                self._handle_status_message(stripped, run_id)

            # Wait for completion
            returncode = self.process.wait()
            stop_polling.set()  # Stop progress polling
            progress_thread.join(timeout=1)
            stderr_thread.join(timeout=1)

            print(f"[DEBUG] Process completed with return code: {returncode}", flush=True)

            stderr_output = ''.join(stderr_lines)
            if stderr_output:
                print(f"[DEBUG] Full STDERR output ({len(stderr_output)} chars): {stderr_output[:1000]}", flush=True)

            # Mark as complete
            print(f"[DEBUG] About to acquire lock for cleanup", flush=True)
            with self.lock:
                print(f"[DEBUG] Lock acquired. active_run exists: {self.active_run is not None}", flush=True)
                if self.active_run:
                    print(f"[DEBUG] active_run ID: {self.active_run.get('run_id')}, expected: {run_id}", flush=True)

                if self.active_run and self.active_run['run_id'] == run_id:
                    print(f"[DEBUG] Match confirmed, updating status", flush=True)
                    self.active_run['end_time'] = datetime.now().isoformat()
                    self.active_run['status'] = 'complete' if returncode == 0 else 'failed'
                    if returncode != 0:
                        error_msg = f"Process exited with code {returncode}"
                        if stderr_output:
                            error_msg += f"\nSTDERR: {stderr_output[:200]}"
                        self.active_run['error'] = error_msg
                        print(f"[DEBUG] Run failed: {error_msg}", flush=True)

                    # Move to history
                    print(f"[DEBUG] Moving to history", flush=True)
                    self.history.append(self.active_run)
                    self.active_run = None
                    try:
                        print(f"[DEBUG] Calling _save_progress()", flush=True)
                        self._save_progress()
                        print(f"[DEBUG] _save_progress() completed", flush=True)
                    except Exception as e:
                        print(f"[DEBUG] ERROR in _save_progress(): {e}", flush=True)
                        import traceback
                        print(f"[DEBUG] Traceback: {traceback.format_exc()}", flush=True)
                    print(f"[DEBUG] Run {run_id} moved to history", flush=True)
                else:
                    print(f"[DEBUG] Condition failed - not moving to history", flush=True)

        except Exception as e:
            print(f"[DEBUG] Exception in subprocess: {e}", flush=True)
            import traceback
            print(f"[DEBUG] Traceback: {traceback.format_exc()}", flush=True)

            with self.lock:
                if self.active_run and self.active_run['run_id'] == run_id:
                    self.active_run['end_time'] = datetime.now().isoformat()
                    self.active_run['status'] = 'failed'
                    self.active_run['error'] = str(e)
                    self.history.append(self.active_run)
                    self.active_run = None
                    self._save_progress()

    def _parse_worker_line(self, line: str, run_id: str):
        """
        Parse worker status line from hierarchical display.

        Format: "+- Worker 1 [2020]: [04/50] Arizona         | Stage 1/7: Redistricting"

        Args:
            line: Worker status line
            run_id: Run ID
        """
        try:
            import re

            # Extract worker number and year (new format includes year in brackets)
            worker_match = re.search(r'Worker (\d+) \[(\d+)\]:', line)
            if not worker_match:
                print(f"[DEBUG] No worker match in line", flush=True)
                return
            worker_id = int(worker_match.group(1))
            year = worker_match.group(2)  # Extract year from brackets
            print(f"[DEBUG] Found worker_id: {worker_id}, year: {year}", flush=True)

            # Extract state info [04/50] Arizona
            state_match = re.search(r'\[(\d+)/(\d+)\]\s+(\w+)', line)
            if not state_match:
                return
            state_num = int(state_match.group(1))
            total_states = int(state_match.group(2))
            state_name = state_match.group(3).lower()

            # Extract stage info Stage 1/7: Redistricting
            stage_match = re.search(r'Stage (\d+)/(\d+):\s*(.+)', line)
            if not stage_match:
                return
            stage_num = int(stage_match.group(1))
            stage_total = int(stage_match.group(2))
            stage_desc = stage_match.group(3).strip()

            # Verify active run
            if not self.active_run or self.active_run['run_id'] != run_id:
                return

            # Update worker info (year now extracted from worker line)
            with self.lock:
                if self.active_run and self.active_run['run_id'] == run_id:
                    worker_key = f"{year}_{worker_id}"
                    worker_data = {
                        'year': year,
                        'worker_id': worker_id,
                        'state': state_name,
                        'state_num': f"{state_num}/{total_states}",
                        'stage': f"{stage_num}/{stage_total}",
                        'stage_desc': stage_desc
                    }
                    self.active_run['workers'][worker_key] = worker_data
                    print(f"[DEBUG] Added worker to progress: {worker_key} = {worker_data}", flush=True)

            # Save progress OUTSIDE the lock to avoid deadlock
            self._save_progress()
            print(f"[DEBUG] Progress saved with {len(self.active_run.get('workers', {}))} workers", flush=True)

        except Exception as e:
            print(f"[DEBUG] Error parsing worker line: {e}", flush=True)

    def _poll_states_complete(self, run_id: str):
        """
        Poll the .states_complete file to update progress.

        Args:
            run_id: Run ID
        """
        if not self.active_run or self.active_run['run_id'] != run_id:
            return

        # Find the .states_complete file
        version = self.active_run['config']['version']
        years = self.active_run['config']['years']

        for year in years:
            states_file = project_root / 'outputs' / version / str(year) / '.states_complete'
            if states_file.exists():
                try:
                    with open(states_file, 'r') as f:
                        completed_states = [line.strip() for line in f if line.strip()]

                    with self.lock:
                        if self.active_run and self.active_run['run_id'] == run_id:
                            prev_count = self.active_run['years'][year]['completed']
                            self.active_run['years'][year]['completed'] = len(completed_states)
                            self.active_run['years'][year]['status'] = 'running'

                            # Track completed redistricting states in current_stage
                            # Create or update "Redistricting" stage
                            if self.active_run['years'][year]['current_stage'] is None or \
                               self.active_run['years'][year]['current_stage'].get('type') == 'census':
                                # Create redistricting stage
                                self.active_run['years'][year]['current_stage'] = {
                                    'name': 'Redistricting',
                                    'type': 'redistricting',
                                    'tasks': []
                                }
                                print(f"[DEBUG] Created Redistricting stage for {year}", flush=True)

                            # Add newly completed states to tasks (normalize all state names)
                            current_stage = self.active_run['years'][year]['current_stage']
                            if current_stage and current_stage['type'] == 'redistricting':
                                existing_states = {task['state'].upper() for task in current_stage['tasks']}

                                # Normalize state names from .states_complete file
                                normalized_completed = [self._normalize_state_code(s) for s in completed_states]
                                new_states = [s for s in normalized_completed if s not in existing_states]

                                if new_states:
                                    print(f"[ASSERT] Adding {len(new_states)} newly completed redistricting states for {year}: {', '.join(new_states)}", flush=True)
                                    for state_code in new_states:
                                        current_stage['tasks'].append({
                                            'state': state_code,
                                            'stage': 7,  # Mark as final redistricting stage
                                            'completed_at': datetime.now().isoformat()
                                        })

                                # Assert all 50 states present when complete
                                if len(normalized_completed) >= 50:
                                    added = self._ensure_all_states_tracked(current_stage['tasks'], 7, 'Redistricting')
                                    if added > 0:
                                        print(f"[ASSERT] Backfilled {added} missing redistricting states for {year}", flush=True)

                            self._save_progress()
                            print(f"[DEBUG] Progress update: {year} = {len(completed_states)}/50 states (+{len(completed_states) - prev_count} since last poll)", flush=True)
                except Exception as e:
                    print(f"[DEBUG] Error polling progress: {e}", flush=True)

    def _handle_status_message(self, line: str, run_id: str):
        """
        Parse and handle STATUS protocol message.

        Args:
            line: Status message line
            run_id: Run ID
        """
        msg_type, data = parse_status_message(line)
        if not msg_type:
            # Check if line contains CENSUS to see if parsing failed
            if 'CENSUS' in line:
                print(f"[DEBUG] Failed to parse CENSUS line: {line}", flush=True)
            return

        print(f"[DEBUG] STATUS message parsed: type={msg_type}, data={data}", flush=True)

        with self.lock:
            if not self.active_run or self.active_run['run_id'] != run_id:
                print(f"[DEBUG] Ignoring STATUS - no active run or ID mismatch", flush=True)
                return

            # Update progress based on message type
            if msg_type == 'YEAR':
                # STATUS:YEAR:2020:COMPLETE:24/50
                year = data['year']
                prev_completed = self.active_run['years'][year].get('completed', 0)
                self.active_run['years'][year]['completed'] = data['completed']
                self.active_run['years'][year]['total'] = data['total']
                self.active_run['years'][year]['status'] = 'running'
                print(f"[DEBUG] Updated YEAR progress: {year} = {data['completed']}/{data['total']}", flush=True)

                # Track when states complete
                if prev_completed < data['total'] and data['completed'] >= data['total']:
                    # Ensure current redistricting stage has all 50 states before marking complete
                    if self.active_run['years'][year]['current_stage'] and \
                       self.active_run['years'][year]['current_stage']['type'] == 'redistricting':
                        current_stage = self.active_run['years'][year]['current_stage']
                        added = self._ensure_all_states_tracked(current_stage['tasks'], 7, 'Redistricting')
                        if added > 0:
                            print(f"[ASSERT] Backfilled {added} states before marking {year} redistricting complete", flush=True)

                        # Move current redistricting stage to completed
                        self.active_run['years'][year]['completed_stages'].append({
                            'name': current_stage['name'],
                            'type': current_stage['type'],
                            'tasks': current_stage['tasks'],
                            'completed_at': datetime.now().isoformat()
                        })
                        self.active_run['years'][year]['current_stage'] = None
                        print(f"[DEBUG] Moved Redistricting stage to completed_stages for {year} with {len(current_stage['tasks'])} states", flush=True)

                    completion_entry = {
                        'stage': 'States',
                        'name': 'State Redistricting',
                        'completed_at': datetime.now().isoformat(),
                        'status': 'complete'
                    }
                    self.active_run['years'][year]['completed_stages'].append(completion_entry)
                    print(f"[DEBUG] Added States stage to completed_stages for {year}: {completion_entry}", flush=True)
                    print(f"[ASSERT] Year {year} state processing complete - all 50 states done ✓", flush=True)

            elif msg_type == 'YEAR_POSTPROCESS':
                # STATUS:YEAR:2020:POSTPROCESS:3/9
                year = data['year']
                if year in self.active_run['years']:
                    prev_postprocess = self.active_run['years'][year].get('postprocess_completed', 0)
                    self.active_run['years'][year]['status'] = 'postprocessing'
                    self.active_run['years'][year]['postprocess_completed'] = data['completed']
                    self.active_run['years'][year]['postprocess_total'] = data['total']

                    # Track when national processing completes
                    if prev_postprocess < data['total'] and data['completed'] >= data['total']:
                        completion_entry = {
                            'stage': 'Nation',
                            'name': 'National Post-processing',
                            'completed_at': datetime.now().isoformat(),
                            'status': 'complete'
                        }
                        self.active_run['years'][year]['completed_stages'].append(completion_entry)
                        print(f"[DEBUG] Added Nation stage to completed_stages for {year}: {completion_entry}", flush=True)
                        print(f"[DEBUG] National processing complete for {year}", flush=True)

                print(f"[DEBUG] Updated YEAR postprocess: {year} = {data['completed']}/{data['total']}", flush=True)

            elif msg_type == 'WORKER':
                # STATUS:WORKER:2020:1:STATE:12/50:california:STAGE:3/7:political_visualization
                year = data['year']
                worker_id = data['worker_id']
                worker_key = f"{year}_{worker_id}"
                self.active_run['workers'][worker_key] = {
                    'year': year,
                    'worker_id': worker_id,
                    'state': data['state_name'],
                    'state_num': data['state_num'],
                    'stage': f"{data['stage']}/{data['stage_total']}",
                    'stage_desc': data['stage_desc'],
                    'last_update': datetime.now().isoformat()
                }
                print(f"[DEBUG] Updated WORKER: {worker_key} = {data['state_name']} stage {data['stage']}/{data['stage_total']}", flush=True)

                # Track redistricting states in current_stage as workers progress through stages
                state_name = data['state_name']
                stage_num = data['stage']
                normalized_state = self._normalize_state_code(state_name)

                # Ensure we have a redistricting current_stage
                if self.active_run['years'][year]['current_stage'] is None or \
                   self.active_run['years'][year]['current_stage'].get('type') == 'census':
                    print(f"[ASSERT] Creating redistricting current_stage for {year}", flush=True)
                    self.active_run['years'][year]['current_stage'] = {
                        'name': 'Redistricting',
                        'type': 'redistricting',
                        'tasks': []
                    }

                # Check if state already tracked
                current_stage = self.active_run['years'][year]['current_stage']
                existing_task = None
                for task in current_stage['tasks']:
                    if task['state'].upper() == normalized_state:
                        existing_task = task
                        break

                if existing_task:
                    # Update existing task with new stage (if progressed)
                    old_stage = existing_task.get('stage', 0)
                    if stage_num > old_stage:
                        existing_task['stage'] = stage_num
                        existing_task['worker_id'] = worker_id
                        existing_task['last_update'] = datetime.now().isoformat()
                        print(f"[ASSERT] Updated {normalized_state} redistricting progress: stage {old_stage} -> {stage_num}", flush=True)
                else:
                    # Add new state to tracking
                    current_stage['tasks'].append({
                        'state': normalized_state,
                        'stage': stage_num,
                        'worker_id': worker_id,
                        'last_update': datetime.now().isoformat()
                    })
                    print(f"[ASSERT] Added {normalized_state} to redistricting tracking at stage {stage_num}", flush=True)

                # Clean up stale workers (no update in 60 seconds)
                now = datetime.now()
                stale_workers = []
                for wkey, worker in self.active_run['workers'].items():
                    if 'last_update' in worker:
                        last_update = datetime.fromisoformat(worker['last_update'])
                        if (now - last_update).total_seconds() > 60:
                            stale_workers.append(wkey)
                for wkey in stale_workers:
                    del self.active_run['workers'][wkey]
                    print(f"[DEBUG] Removed stale worker: {wkey}", flush=True)

            elif msg_type == 'CENSUS_STAGE':
                # STATUS:CENSUS:2020:STAGE:1/3:Parsing PL 94-171:5/50
                year = data['year']
                stage_name = data['stage_name']

                # Update census progress tracking
                self.active_run['census_progress'][year] = {
                    'stage': stage_name,
                    'completed': data['completed'],
                    'total': data['total'],
                    'active': True
                }

                # Set or update current stage for this year
                if self.active_run['years'][year]['current_stage'] is None or \
                   self.active_run['years'][year]['current_stage']['name'] != stage_name:
                    # New stage starting - save previous stage as completed if it exists
                    if self.active_run['years'][year]['current_stage'] is not None:
                        prev_stage = self.active_run['years'][year]['current_stage']
                        self.active_run['years'][year]['completed_stages'].append({
                            'name': prev_stage['name'],
                            'type': 'census',
                            'tasks': prev_stage['tasks'],
                            'completed_at': datetime.now().isoformat()
                        })
                        print(f"[DEBUG] Moved '{prev_stage['name']}' to completed_stages for {year}", flush=True)

                    # Start new current stage
                    self.active_run['years'][year]['current_stage'] = {
                        'name': stage_name,
                        'type': 'census',
                        'tasks': []
                    }
                    print(f"[DEBUG] Started new current_stage for {year}: {stage_name}", flush=True)

                # If stage shows completed >= total (all states done, possibly skipped), ensure all states are tracked
                if data['completed'] >= data['total']:
                    current_stage = self.active_run['years'][year]['current_stage']
                    if current_stage and current_stage['name'] == stage_name:
                        # Determine stage number from stage name
                        if 'Parsing' in stage_name:
                            stage_num = 1
                        elif 'Merging' in stage_name:
                            stage_num = 2
                        elif 'adjacency' in stage_name.lower():
                            stage_num = 3
                        else:
                            stage_num = 3  # Default

                        # Use assertion function to ensure all 50 states tracked
                        added = self._ensure_all_states_tracked(current_stage['tasks'], stage_num, stage_name)
                        if added > 0:
                            print(f"[ASSERT] Stage '{stage_name}' completed with {len(current_stage['tasks'])} states total", flush=True)

                print(f"[DEBUG] Updated CENSUS_STAGE: {year} = {stage_name} {data['completed']}/{data['total']}", flush=True)

            elif msg_type == 'CENSUS_STAGE_PROGRESS':
                # STATUS:CENSUS:2020:WORKER:0:STATE:5/50:CA:COMPLETE (worker completed a state)
                # This updates the overall completed count
                year = data['year']
                state_code = data.get('state_code', '??')
                worker_id = data.get('worker_id', 0)
                print(f"[DEBUG] *** RECEIVED CENSUS_STAGE_PROGRESS: year={year}, state={state_code}, worker={worker_id}, completed={data['completed']}/{data['total']}", flush=True)

                # Create entry if it doesn't exist
                if year not in self.active_run['census_progress']:
                    self.active_run['census_progress'][year] = {
                        'stage': 'Processing',
                        'completed': 0,
                        'total': data['total'],
                        'active': True
                    }
                    print(f"[DEBUG] Created new census_progress entry for {year}", flush=True)

                # Update completed count
                prev_completed = self.active_run['census_progress'][year]['completed']
                self.active_run['census_progress'][year]['completed'] = data['completed']
                print(f"[DEBUG] Updated CENSUS_STAGE_PROGRESS: {year} = {prev_completed} -> {data['completed']}/{data['total']}, state={state_code}, worker={worker_id}", flush=True)

                # Add task to current stage with stage number
                if self.active_run['years'][year]['current_stage'] is not None:
                    current_stage = self.active_run['years'][year]['current_stage']
                    # Parse stage number from stage name
                    stage_num = None
                    if 'Parsing' in current_stage['name']:
                        stage_num = 1
                    elif 'Merging' in current_stage['name']:
                        stage_num = 2
                    elif 'adjacency' in current_stage['name'].lower():
                        stage_num = 3

                    task_entry = {
                        'state': state_code,
                        'worker_id': worker_id,
                        'stage': stage_num,
                        'completed_at': datetime.now().isoformat()
                    }
                    self.active_run['years'][year]['current_stage']['tasks'].append(task_entry)
                    print(f"[DEBUG] Added task to current_stage for {year}: {state_code} (stage {stage_num})", flush=True)

                # If all states complete, move current stage to completed
                if data['completed'] >= data['total']:
                    if self.active_run['years'][year]['current_stage'] is not None:
                        current = self.active_run['years'][year]['current_stage']

                        # Ensure all 50 states are tracked before moving to completed
                        stage_num = None
                        if 'Parsing' in current['name']:
                            stage_num = 1
                        elif 'Merging' in current['name']:
                            stage_num = 2
                        elif 'adjacency' in current['name'].lower():
                            stage_num = 3

                        if stage_num:
                            added = self._ensure_all_states_tracked(current['tasks'], stage_num, current['name'])
                            if added > 0:
                                print(f"[ASSERT] Backfilled {added} states before completing stage '{current['name']}'", flush=True)

                        self.active_run['years'][year]['completed_stages'].append({
                            'name': current['name'],
                            'type': current['type'],
                            'tasks': current['tasks'],
                            'completed_at': datetime.now().isoformat()
                        })
                        self.active_run['years'][year]['current_stage'] = None
                        print(f"[DEBUG] Moved current stage '{current['name']}' to completed_stages for {year} with {len(current['tasks'])} states", flush=True)

                    # Remove all census workers for this year
                    workers_to_remove = [k for k, v in self.active_run['workers'].items()
                                       if v.get('type') == 'census' and v.get('year') == year]
                    for worker_key in workers_to_remove:
                        del self.active_run['workers'][worker_key]
                    # Remove census progress entry
                    del self.active_run['census_progress'][year]
                    print(f"[DEBUG] Census complete for {year}, removed workers and progress", flush=True)

            elif msg_type == 'CENSUS_WORKER':
                # STATUS:CENSUS:2020:WORKER:0:STATE:5/50:CA:STAGE:1/3:Parsing
                year = data['year']
                worker_id = data['worker_id']
                worker_key = f"census_{year}_{worker_id}"
                self.active_run['workers'][worker_key] = {
                    'year': year,
                    'worker_id': worker_id,
                    'type': 'census',
                    'state': data['state_name'],
                    'last_update': datetime.now().isoformat(),
                    'state_num': data['state_num'],
                    'stage': f"{data['stage_num']}/{data['stage_total']}",
                    'stage_desc': data['stage_name']
                }

        # Save progress OUTSIDE lock to avoid deadlock
        self._save_progress()

    def cancel_run(self, run_id: str) -> tuple[bool, str]:
        """
        Cancel an active run.

        Args:
            run_id: Run ID to cancel

        Returns:
            Tuple of (success, message)
        """
        with self.lock:
            if not self.active_run or self.active_run['run_id'] != run_id:
                return (False, "Run not found or not active")

            if self.process:
                try:
                    self.process.terminate()
                    self.process.wait(timeout=5)
                except Exception:
                    self.process.kill()

            # Delete output files for cancelled run
            version = self.active_run.get('version', 'unknown')
            self._delete_run_outputs(version)

            self.active_run['end_time'] = datetime.now().isoformat()
            self.active_run['status'] = 'cancelled'
            self.history.append(self.active_run)
            self.active_run = None
            self._save_progress()

            return (True, "Run cancelled - outputs deleted")

    def get_active_run(self) -> Optional[Dict[str, Any]]:
        """Get active run progress."""
        with self.lock:
            return self.active_run

    def get_history(self) -> list[Dict[str, Any]]:
        """Get run history."""
        with self.lock:
            return self.history[-50:]  # Last 50 runs
