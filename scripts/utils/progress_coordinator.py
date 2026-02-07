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
from .status_protocol import parse_status_message


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

        # Track census data processing
        self.census_progress = {year: {'stage': '', 'completed': 0, 'total': 50, 'active': False} for year in years}
        self.census_worker_status = {}  # {(year, worker_id): status_dict}

        # Track redistricting stage completion across all states
        # Maps year -> {stage_name: completed_count}
        self.stage_progress = {year: {} for year in years}
        self.stage_names = [
            'redistricting', 'summary', 'cities', 'round_maps', 'district_maps',
            'metro_area_maps', 'compactness', 'demographics_analysis',
            'demographics_visualization', 'political_analysis', 'political_visualization'
        ]

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

    def update_census_stage(self, year, stage_name, completed, total):
        """
        Update census data processing stage.

        Args:
            year: Census year
            stage_name: Stage name (e.g., "Parsing PL 94-171", "Merging geometries")
            completed: Number of states completed
            total: Total states
        """
        with self.lock:
            self.census_progress[year] = {
                'stage': stage_name,
                'completed': completed,
                'total': total,
                'active': True
            }

    def update_census_worker(self, year, worker_id, state_num, state_name, stage_name):
        """
        Update census data worker status.

        Args:
            year: Census year
            worker_id: Worker ID (0, 1, 2, ...)
            state_num: State number this worker is on
            state_name: State code (e.g., 'CA', 'TX')
            stage_name: Stage name (e.g., "Parsing PL 94-171")
        """
        with self.lock:
            self.census_worker_status[(year, worker_id)] = {
                'state_num': state_num,
                'state_name': state_name,
                'stage_name': stage_name
            }

    def census_complete(self, year):
        """
        Mark census data processing as complete for a year.

        Args:
            year: Census year
        """
        with self.lock:
            self.census_progress[year]['active'] = False
            self.census_worker_status = {k: v for k, v in self.census_worker_status.items() if k[0] != year}

    def update_stage_completion(self, year, stage_name, completed_count):
        """
        Update stage completion count for redistricting pipeline.

        Args:
            year: Census year
            stage_name: Stage name (e.g., 'redistricting', 'summary', 'cities')
            completed_count: Number of states that have completed this stage
        """
        with self.lock:
            self.stage_progress[year][stage_name] = completed_count

    def render(self):
        """
        Render the current progress display.

        Returns:
            String with formatted progress display
        """
        with self.lock:
            lines = []

            for year in self.years:
                # Census data processing (if active)
                census = self.census_progress[year]
                if census['active']:
                    census_completed = census['completed']
                    census_total = census['total']
                    census_stage = census['stage']

                    census_bar = create_progress_bar(census_completed, census_total, width=20)
                    census_line = f"[{year}] {census_bar} {census_stage}: {census_completed}/{census_total} states"
                    lines.append(census_line)

                    # Census worker bars
                    num_workers = self.workers_per_year[year]
                    for worker_id in range(num_workers):
                        worker_key = (year, worker_id)
                        is_last = (worker_id == num_workers - 1)
                        connector = format_tree_connector(is_last, wide_terminal=self.wide_terminal)

                        if worker_key in self.census_worker_status:
                            status = self.census_worker_status[worker_key]
                            state_name = status['state_name']
                            stage_name = status['stage_name']

                            worker_line = (
                                f"{connector}Worker {worker_id + 1} [{year}]: "
                                f"[{status['state_num']:02d}/{census_total}] {state_name:4s} | {stage_name}"
                            )
                            lines.append(worker_line)
                        else:
                            worker_line = f"{connector}Worker {worker_id + 1}: Idle"
                            lines.append(worker_line)

                    lines.append("")  # Blank line after census data

                # Top bar (state processing)
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

                # Stage progress bars (if any stages have been completed)
                if phase == 'states' and self.stage_progress.get(year):
                    stage_lines = []

                    for stage_name in self.stage_names:
                        stage_completed = self.stage_progress[year].get(stage_name, 0)
                        if stage_completed > 0:  # Only show stages that have started
                            stage_bar = create_progress_bar(stage_completed, total, width=15)
                            # Format stage name nicely
                            display_name = stage_name.replace('_', ' ').title()
                            stage_line = f"  {display_name:25s} {stage_completed:2d}/50 {stage_bar}"
                            if stage_completed >= total:
                                stage_line += " [OK]"
                            stage_lines.append(stage_line)

                    if stage_lines:
                        lines.extend(stage_lines)

                # No worker bars during state processing - stage progress is sufficient

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
