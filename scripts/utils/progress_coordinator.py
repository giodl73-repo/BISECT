#!/usr/bin/env python3
"""
Progress coordinator for hierarchical multi-year pipeline visualization.

Manages progress display showing:
- Top-level bars: Census year overall progress
- Worker bars: Individual worker activity

Simplified version for initial implementation.
"""

import sys
import threading
from collections import defaultdict
from .terminal_utils import (
    get_terminal_size, is_wide_terminal,
    create_progress_bar, format_tree_connector,
    abbreviate_state_name, format_stage_description
)


class ProgressCoordinator:
    """
    Coordinates hierarchical progress display for multi-year pipeline.

    Displays:
        [2020] ████████████████░░░░ 24/50 states complete
          ├─ Worker 1: [12/50] California      | Stage 3.4/7: Political visualization
          └─ Worker 2: [12/50] Florida         | Stage 2.5/7: District maps
    """

    def __init__(self, years=['2020', '2010', '2000'], workers_per_year=[2, 2, 2]):
        """
        Initialize progress coordinator.

        Args:
            years: List of census years
            workers_per_year: Number of workers for each year
        """
        self.years = years
        self.workers_per_year = {year: count for year, count in zip(years, workers_per_year)}

        # Track state
        self.year_progress = {year: {'completed': 0, 'total': 50, 'phase': 'states', 'phase_desc': ''} for year in years}
        self.worker_status = {}  # {(year, worker_id): status_dict}

        self.lock = threading.Lock()
        self.wide_terminal = is_wide_terminal(120)

    def update_year_progress(self, year, completed, total=50):
        """
        Update top-level year progress.

        Args:
            year: Census year ('2020', '2010', '2000')
            completed: Number of states completed
            total: Total states (default: 50)
        """
        with self.lock:
            self.year_progress[year]['completed'] = completed
            self.year_progress[year]['total'] = total

    def update_year_phase(self, year, phase_desc):
        """
        Update year to post-processing phase.

        Args:
            year: Census year
            phase_desc: Phase description (e.g., "Phase 2/3: Visualization")
        """
        with self.lock:
            self.year_progress[year]['phase'] = 'postprocess'
            self.year_progress[year]['phase_desc'] = phase_desc

    def update_worker_status(self, year, worker_id, state_num, state_name, stage, stage_total, stage_desc):
        """
        Update worker-level status for state processing.

        Args:
            year: Census year
            worker_id: Worker ID (0, 1, 2, ...)
            state_num: State number this worker is on
            state_name: State name (e.g., 'california')
            stage: Current stage number
            stage_total: Total stages
            stage_desc: Stage description
        """
        with self.lock:
            self.worker_status[(year, worker_id)] = {
                'type': 'state',
                'state_num': state_num,
                'state_name': state_name,
                'stage': stage,
                'stage_total': stage_total,
                'stage_desc': stage_desc
            }

    def update_worker_task(self, year, worker_id, task_index, task_total, task_name):
        """
        Update worker-level status for post-processing task.

        Args:
            year: Census year
            worker_id: Worker ID (0, 1, 2, ...)
            task_index: Task number (1, 2, 3, ...)
            task_total: Total tasks
            task_name: Task name (e.g., "National_district_map")
        """
        with self.lock:
            self.worker_status[(year, worker_id)] = {
                'type': 'task',
                'task_index': task_index,
                'task_total': task_total,
                'task_name': task_name
            }

    def update_year_postprocess(self, year, completed, total):
        """
        Update year-level post-processing progress.

        Args:
            year: Census year
            completed: Number of tasks completed
            total: Total tasks
        """
        with self.lock:
            self.year_progress[year]['phase'] = 'postprocess'
            self.year_progress[year]['completed'] = completed
            self.year_progress[year]['total'] = total
            self.year_progress[year]['phase_desc'] = f'{completed}/{total} tasks'

    def render(self):
        """
        Render the current progress display.

        Returns:
            String with formatted progress display
        """
        with self.lock:
            lines = []

            for year in self.years:
                # Top bar
                progress = self.year_progress[year]
                completed = progress['completed']
                total = progress['total']
                phase = progress.get('phase', 'states')
                phase_desc = progress.get('phase_desc', '')

                bar = create_progress_bar(completed, total, width=20)

                if phase == 'postprocess':
                    # Post-processing phase - show tasks not states
                    if completed >= total:
                        year_line = f"[{year}] {bar} {phase_desc} COMPLETE"
                    else:
                        year_line = f"[{year}] {bar} {phase_desc}"
                else:
                    # State processing phase
                    if completed >= total:
                        year_line = f"[{year}] {bar} {completed}/{total} states COMPLETE"
                    else:
                        year_line = f"[{year}] {bar} {completed}/{total} states complete"

                lines.append(year_line)

                # Worker bars
                num_workers = self.workers_per_year[year]
                year_complete = (completed >= total)

                for worker_id in range(num_workers):
                    worker_key = (year, worker_id)
                    is_last = (worker_id == num_workers - 1)
                    connector = format_tree_connector(is_last, wide_terminal=self.wide_terminal)

                    # If year is complete, show all workers as Idle
                    if year_complete:
                        worker_line = f"{connector}Worker {worker_id + 1}: Idle"
                        lines.append(worker_line)
                    elif worker_key in self.worker_status:
                        status = self.worker_status[worker_key]
                        status_type = status.get('type', 'state')

                        if status_type == 'state':
                            # State processing worker
                            # Format state name
                            state_name = status['state_name']
                            if not self.wide_terminal:
                                state_name = abbreviate_state_name(state_name, max_length=10)

                            # Format stage description (replace underscores, title case)
                            stage_desc = format_stage_description(status['stage_desc'], wide_terminal=self.wide_terminal)

                            # Worker line
                            worker_line = (
                                f"{connector}Worker {worker_id + 1}: "
                                f"[{status['state_num']:02d}/50] {state_name:15s} | "
                                f"Stage {status['stage']}/{status['stage_total']}: {stage_desc}"
                            )
                            lines.append(worker_line)

                        elif status_type == 'task':
                            # Post-processing task worker
                            task_name = status['task_name'].replace('_', ' ')
                            task_index = status['task_index']
                            task_total = status['task_total']

                            worker_line = (
                                f"{connector}Worker {worker_id + 1}: "
                                f"[Task {task_index}/{task_total}] {task_name}"
                            )
                            lines.append(worker_line)
                    else:
                        # Worker not yet started
                        worker_line = f"{connector}Worker {worker_id + 1}: Idle"
                        lines.append(worker_line)

                # Blank line between years
                if year != self.years[-1]:
                    lines.append("")

            return "\n".join(lines)

    def print_status(self):
        """
        Print the current status to console.

        Clears previous output and reprints (simple version).
        """
        status_text = self.render()
        print(f"\n{status_text}\n", flush=True)


def parse_status_message(line):
    """
    Parse STATUS messages from child processes.

    Four types:
    1. STATUS:YEAR:2020:COMPLETE:24/50 (state progress)
    2. STATUS:WORKER:2020:1:STATE:12/50:california:STAGE:3/7:political_visualization (state worker)
    3. STATUS:YEAR:2020:POSTPROCESS:PHASE:2/3:Visualization (post-processing phase)
    4. STATUS:WORKER:2020:1:TASK:3/6:National_district_map:PROGRESS:75/100 (post-processing worker)

    Returns:
        Tuple of (message_type, data_dict) or (None, None) if not a STATUS message

    Examples:
        parse_status_message("STATUS:YEAR:2020:COMPLETE:24/50")
        -> ('YEAR', {'year': '2020', 'completed': 24, 'total': 50})

        parse_status_message("STATUS:YEAR:2020:POSTPROCESS:PHASE:2/3:Visualization")
        -> ('YEAR_PHASE', {'year': '2020', 'phase': '2/3', 'phase_desc': 'Visualization'})

        parse_status_message("STATUS:WORKER:2020:1:TASK:3/6:National_map:PROGRESS:75/100")
        -> ('WORKER_TASK', {'year': '2020', 'worker_id': 1, 'task_index': 3, ...})
    """
    if not line.startswith("STATUS:"):
        return (None, None)

    parts = line.split(":")

    if len(parts) < 3:
        return (None, None)

    msg_type = parts[1]

    if msg_type == "YEAR":
        year = parts[2]

        # Check if this is post-processing progress
        if len(parts) >= 5 and parts[3] == "POSTPROCESS":
            # STATUS:YEAR:2020:POSTPROCESS:3/9
            progress_str = parts[4]  # "3/9"
            if '/' in progress_str:
                completed, total = progress_str.split('/')
                return ('YEAR_POSTPROCESS', {
                    'year': year,
                    'completed': int(completed),
                    'total': int(total)
                })

        # Otherwise it's a regular state completion message
        # STATUS:YEAR:2020:COMPLETE:24/50
        if len(parts) >= 5 and parts[3] == "COMPLETE":
            complete_str = parts[4]  # "24/50"
            if '/' in complete_str:
                completed, total = complete_str.split('/')
                return ('YEAR', {
                    'year': year,
                    'completed': int(completed),
                    'total': int(total)
                })

    elif msg_type == "WORKER":
        year = parts[2]
        worker_id = int(parts[3])
        work_type = parts[4]  # "STATE" or "TASK"

        if work_type == "STATE":
            # STATUS:WORKER:2020:1:STATE:12/50:california:STAGE:3/7:political_visualization
            if len(parts) >= 10:
                state_str = parts[5]  # "12/50"
                state_name = parts[6]
                stage_str = parts[8]  # "3/7"
                stage_desc = parts[9]

                if '/' in state_str and '/' in stage_str:
                    state_num, _ = state_str.split('/')
                    stage, stage_total = stage_str.split('/')

                    return ('WORKER', {
                        'year': year,
                        'worker_id': worker_id,
                        'state_num': int(state_num),
                        'state_name': state_name,
                        'stage': int(stage),
                        'stage_total': int(stage_total),
                        'stage_desc': stage_desc
                    })

        elif work_type == "TASK":
            # STATUS:WORKER:2020:1:TASK:3/9:National_district_map
            if len(parts) >= 7:
                task_str = parts[5]  # "3/9"
                task_name = parts[6]

                if '/' in task_str:
                    task_index, task_total = task_str.split('/')

                    return ('WORKER_TASK', {
                        'year': year,
                        'worker_id': worker_id,
                        'task_index': int(task_index),
                        'task_total': int(task_total),
                        'task_name': task_name
                    })

    return (None, None)
