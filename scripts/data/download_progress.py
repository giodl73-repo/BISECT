#!/usr/bin/env python3
"""
Progress coordinator for hierarchical multi-year download visualization.

Manages progress display showing:
- Top-level bars: Year overall progress
- Worker bars: Individual worker activity

Adapted from scripts/utils/progress_coordinator.py for download workflows.

Created: 2026-01-18 (Enhancement #48)
"""

import sys
import threading
from collections import defaultdict
from scripts.utils.terminal_utils import (
    get_terminal_size, is_wide_terminal,
    create_progress_bar, format_tree_connector,
    abbreviate_state_name, format_stage_description
)


class DownloadCoordinator:
    """
    Coordinates hierarchical progress display for multi-year downloads.

    Displays:
        [2020] ████████████████░░░░ 24/50 states complete
          ├─ Worker 1: [12/50] California      | Step 2/4: Downloading CSV
          └─ Worker 2: [13/50] Texas           | Step 3/4: Extracting ZIP
    """

    def __init__(self, download_type='data', years=['2020', '2010', '2000'], workers_per_year=[2, 2, 2]):
        """
        Initialize download progress coordinator.

        Args:
            download_type: Type of data being downloaded ('tracts', 'demographics', etc.)
            years: List of census years
            workers_per_year: Number of workers for each year
        """
        self.download_type = download_type
        self.years = years
        self.workers_per_year = {year: count for year, count in zip(years, workers_per_year)}

        # Track state
        self.year_progress = {year: {'completed': 0, 'total': 50, 'phase': 'downloading'} for year in years}
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

    def update_worker_status(self, year, worker_id, state_num, state_name, step, step_total, step_desc):
        """
        Update worker-level status for state downloading.

        Args:
            year: Census year
            worker_id: Worker ID (0, 1, 2, ...)
            state_num: State number this worker is on
            state_name: State name (e.g., 'california')
            step: Current step number (1=checking, 2=downloading, 3=extracting, 4=validating)
            step_total: Total steps (typically 4)
            step_desc: Step description (e.g., 'downloading_csv', 'extracting_zip')
        """
        with self.lock:
            self.worker_status[(year, worker_id)] = {
                'state_num': state_num,
                'state_name': state_name,
                'step': step,
                'step_total': step_total,
                'step_desc': step_desc
            }

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

                bar = create_progress_bar(completed, total, width=20)

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

                        # Format state name
                        state_name = status['state_name']
                        if not self.wide_terminal:
                            state_name = abbreviate_state_name(state_name, max_length=10)

                        # Format step description (replace underscores, title case)
                        step_desc = format_stage_description(status['step_desc'], wide_terminal=self.wide_terminal)

                        # Worker line
                        worker_line = (
                            f"{connector}Worker {worker_id + 1}: "
                            f"[{status['state_num']:02d}/50] {state_name:15s} | "
                            f"Step {status['step']}/{status['step_total']}: {step_desc}"
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


def parse_download_status(line):
    """
    Parse STATUS messages from download worker processes.

    Three types:
    1. STATUS:DOWNLOAD:2020:COMPLETE:24/50 (state progress)
    2. STATUS:WORKER:2020:1:STATE:12/50:california:STEP:3/4:extracting_zip (worker status)
    3. STATUS:DOWNLOAD:2020:ERROR:california:Connection timeout (error message)

    Returns:
        Tuple of (message_type, data_dict) or (None, None) if not a STATUS message

    Examples:
        parse_download_status("STATUS:DOWNLOAD:2020:COMPLETE:24/50")
        -> ('YEAR', {'year': '2020', 'completed': 24, 'total': 50})

        parse_download_status("STATUS:WORKER:2020:1:STATE:12/50:california:STEP:3/4:extracting")
        -> ('WORKER', {'year': '2020', 'worker_id': 1, 'state_num': 12, ...})
    """
    if not line.startswith("STATUS:"):
        return (None, None)

    parts = line.split(":")

    if len(parts) < 3:
        return (None, None)

    msg_type = parts[1]

    if msg_type == "DOWNLOAD":
        year = parts[2]

        # STATUS:DOWNLOAD:2020:COMPLETE:24/50
        if len(parts) >= 5 and parts[3] == "COMPLETE":
            complete_str = parts[4]  # "24/50"
            if '/' in complete_str:
                completed, total = complete_str.split('/')
                return ('YEAR', {
                    'year': year,
                    'completed': int(completed),
                    'total': int(total)
                })

        # STATUS:DOWNLOAD:2020:ERROR:california:Connection timeout
        elif len(parts) >= 6 and parts[3] == "ERROR":
            state_name = parts[4]
            error_msg = ':'.join(parts[5:])  # Rejoin in case error has colons
            return ('ERROR', {
                'year': year,
                'state_name': state_name,
                'error': error_msg
            })

    elif msg_type == "WORKER":
        year = parts[2]
        worker_id = int(parts[3])
        work_type = parts[4]  # "STATE"

        if work_type == "STATE":
            # STATUS:WORKER:2020:1:STATE:12/50:california:STEP:3/4:extracting_zip
            if len(parts) >= 10:
                state_str = parts[5]  # "12/50"
                state_name = parts[6]
                step_str = parts[8]  # "3/4"
                step_desc = parts[9]

                if '/' in state_str and '/' in step_str:
                    state_num, _ = state_str.split('/')
                    step, step_total = step_str.split('/')

                    return ('WORKER', {
                        'year': year,
                        'worker_id': worker_id,
                        'state_num': int(state_num),
                        'state_name': state_name,
                        'step': int(step),
                        'step_total': int(step_total),
                        'step_desc': step_desc
                    })

    return (None, None)
