#!/usr/bin/env python3
"""
Unified STATUS protocol for parent-child process communication.

This module provides a standardized way for child processes to report progress
to parent processes, and for parent processes to read and parse those messages.

STATUS Message Formats:
-----------------------

1. Basic: STATUS:{pos}:{message}
   Example: STATUS:0:Processing california

2. CENSUS: STATUS:CENSUS:{year}:WORKER:{worker_id}:STATE:{num}/{total}:{state}:STAGE:{stage_num}/{stage_total}:{stage_name}
   Example: STATUS:CENSUS:2020:WORKER:0:STATE:5/50:CA:STAGE:1/3:Parsing PL 94-171

3. YEAR: STATUS:YEAR:{year}:COMPLETE:{completed}/{total}
   Example: STATUS:YEAR:2020:COMPLETE:24/50

4. WORKER: STATUS:WORKER:{year}:{worker_id}:STATE:{num}/{total}:{state}:STAGE:{stage}/{total}:{desc}
   Example: STATUS:WORKER:2020:1:STATE:12/50:california:STAGE:3/7:political_visualization

Environment Variables:
---------------------
TQDM_POSITION: Controls reporting mode
  -1 (default): Standalone mode - print all output normally
  0-50: Child process mode - use STATUS protocol
  999: Deeply nested child - suppress verbose output

Usage Examples:
--------------

Child Process (generating STATUS messages):
    from scripts.utils.status_protocol import StatusReporter

    reporter = StatusReporter()
    reporter.report("Processing state CA")
    reporter.report_census_worker(year="2020", worker_id=0, state_num=5,
                                   state_code="CA", stage_num=1, stage_total=3,
                                   stage_name="Parsing PL 94-171")

Parent Process (reading STATUS messages):
    from scripts.utils.status_protocol import StatusReader

    proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, text=True)
    reader = StatusReader(proc.stdout)

    for msg_type, data in reader.iter_status_messages():
        if msg_type == 'CENSUS_WORKER':
            print(f"Worker {data['worker_id']} processing {data['state_name']}")
"""

import os
import sys
from typing import Optional, Tuple, Dict, Any, Iterator, TextIO


class StatusReporter:
    """
    Generate STATUS protocol messages for child processes.

    Automatically detects whether running as a child process (via TQDM_POSITION
    environment variable) and emits STATUS messages accordingly.

    Usage:
        reporter = StatusReporter()
        reporter.report("Processing state CA")
        reporter.report_census_worker(year="2020", worker_id=0, ...)
    """

    def __init__(self, position: Optional[int] = None):
        """
        Initialize StatusReporter.

        Args:
            position: Override TQDM_POSITION (default: read from environment)
        """
        if position is not None:
            self.position = position
        else:
            self.position = int(os.environ.get('TQDM_POSITION', '-1'))

        self.is_standalone = (self.position < 0)
        self.send_status = not self.is_standalone

    def report(self, message: str):
        """
        Report a basic progress message.

        Args:
            message: Progress message to report

        Example:
            reporter.report("Processing california")
            # Emits: STATUS:0:Processing california
        """
        if self.send_status:
            print(f"STATUS:{self.position}:{message}", flush=True)
        else:
            print(message, flush=True)

    def report_census_stage(self, year: str, stage_name: str, completed: int, total: int):
        """
        Report census data processing stage progress.

        Args:
            year: Census year (e.g., "2020")
            stage_name: Stage description (e.g., "Parsing PL 94-171")
            completed: Number of states completed
            total: Total number of states

        Example:
            reporter.report_census_stage("2020", "Parsing PL 94-171", 5, 50)
            # Emits: STATUS:CENSUS:2020:STAGE:1/3:Parsing PL 94-171:5/50
        """
        if self.send_status:
            print(f"STATUS:CENSUS:{year}:STAGE:1/3:{stage_name}:{completed}/{total}", flush=True)
        else:
            print(f"[{year}] {stage_name}: {completed}/{total}", flush=True)

    def report_census_worker(self, year: str, worker_id: int, state_num: int,
                            state_code: str, stage_num: int, stage_total: int,
                            stage_name: str):
        """
        Report census data processing worker status.

        Args:
            year: Census year
            worker_id: Worker ID (0, 1, 2, ...)
            state_num: State number this worker is on
            state_code: State code (e.g., 'CA', 'TX')
            stage_num: Current stage number (1, 2, 3)
            stage_total: Total stages (usually 3)
            stage_name: Stage description

        Example:
            reporter.report_census_worker("2020", 0, 5, "CA", 1, 3, "Parsing PL 94-171")
            # Emits: STATUS:CENSUS:2020:WORKER:0:STATE:5/50:CA:STAGE:1/3:Parsing PL 94-171
        """
        if self.send_status:
            print(f"STATUS:CENSUS:{year}:WORKER:{worker_id}:STATE:{state_num}/50:{state_code}:STAGE:{stage_num}/{stage_total}:{stage_name}", flush=True)
        else:
            print(f"Worker {worker_id}: [{state_num}/50] {state_code} - {stage_name}", flush=True)

    def report_census_worker_complete(self, year: str, worker_id: int, state_num: int,
                                     state_code: str, total: int):
        """
        Report census data processing worker completion for one state.

        Args:
            year: Census year
            worker_id: Worker ID
            state_num: State number completed
            state_code: State code
            total: Total states

        Example:
            reporter.report_census_worker_complete("2020", 0, 5, "CA", 50)
            # Emits: STATUS:CENSUS:2020:WORKER:0:STATE:5/50:CA:COMPLETE
        """
        if self.send_status:
            print(f"STATUS:CENSUS:{year}:WORKER:{worker_id}:STATE:{state_num}/{total}:{state_code}:COMPLETE", flush=True)
        else:
            print(f"Worker {worker_id}: [{state_num}/{total}] {state_code} complete", flush=True)

    def report_year_complete(self, year: str, completed: int, total: int):
        """
        Report year-level state completion progress.

        Args:
            year: Census year
            completed: Number of states completed
            total: Total states

        Example:
            reporter.report_year_complete("2020", 24, 50)
            # Emits: STATUS:YEAR:2020:COMPLETE:24/50
        """
        if self.send_status:
            print(f"STATUS:YEAR:{year}:COMPLETE:{completed}/{total}", flush=True)
        else:
            print(f"[{year}] {completed}/{total} states complete", flush=True)

    def report_year_postprocess(self, year: str, completed: int, total: int):
        """
        Report year-level post-processing progress.

        Args:
            year: Census year
            completed: Number of tasks completed
            total: Total tasks

        Example:
            reporter.report_year_postprocess("2020", 3, 9)
            # Emits: STATUS:YEAR:2020:POSTPROCESS:3/9
        """
        if self.send_status:
            print(f"STATUS:YEAR:{year}:POSTPROCESS:{completed}/{total}", flush=True)
        else:
            print(f"[{year}] Post-processing: {completed}/{total} tasks", flush=True)

    def report_worker_state(self, year: str, worker_id: int, state_num: int,
                           state_name: str, stage: int, stage_total: int,
                           stage_desc: str):
        """
        Report worker-level state processing status.

        Args:
            year: Census year
            worker_id: Worker ID
            state_num: State number this worker is on
            state_name: State name (e.g., 'california')
            stage: Current stage number
            stage_total: Total stages
            stage_desc: Stage description (e.g., 'political_visualization')

        Example:
            reporter.report_worker_state("2020", 1, 12, "california", 3, 7, "political_visualization")
            # Emits: STATUS:WORKER:2020:1:STATE:12/50:california:STAGE:3/7:political_visualization
        """
        if self.send_status:
            print(f"STATUS:WORKER:{year}:{worker_id}:STATE:{state_num}/50:{state_name}:STAGE:{stage}/{stage_total}:{stage_desc}", flush=True)
        else:
            print(f"Worker {worker_id}: [{state_num}/50] {state_name} - Stage {stage}/{stage_total}: {stage_desc}", flush=True)

    def report_worker_task(self, year: str, worker_id: int, task_index: int,
                          task_total: int, task_name: str):
        """
        Report worker-level post-processing task status.

        Args:
            year: Census year
            worker_id: Worker ID
            task_index: Task number (1, 2, 3, ...)
            task_total: Total tasks
            task_name: Task name (e.g., 'National_district_map')

        Example:
            reporter.report_worker_task("2020", 1, 3, 9, "National_district_map")
            # Emits: STATUS:WORKER:2020:1:TASK:3/9:National_district_map
        """
        if self.send_status:
            print(f"STATUS:WORKER:{year}:{worker_id}:TASK:{task_index}/{task_total}:{task_name}", flush=True)
        else:
            print(f"Worker {worker_id}: Task {task_index}/{task_total} - {task_name}", flush=True)


class StatusReader:
    """
    Read and parse STATUS protocol messages from child processes.

    Provides an iterator interface for reading STATUS messages from a subprocess
    stdout stream. Non-STATUS lines are optionally forwarded or discarded.

    Usage:
        proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, text=True)
        reader = StatusReader(proc.stdout)

        for msg_type, data in reader.iter_status_messages():
            if msg_type == 'CENSUS_WORKER':
                print(f"Processing {data['state_name']}")
    """

    def __init__(self, stream: TextIO, forward_non_status: bool = False):
        """
        Initialize StatusReader.

        Args:
            stream: Text stream to read from (usually subprocess.stdout)
            forward_non_status: Whether to print non-STATUS lines (default: False)
        """
        self.stream = stream
        self.forward_non_status = forward_non_status

    def iter_status_messages(self) -> Iterator[Tuple[Optional[str], Optional[Dict[str, Any]]]]:
        """
        Iterate over STATUS messages from the stream.

        Yields:
            Tuple of (message_type, data_dict)
            - message_type: One of 'YEAR', 'CENSUS_STAGE', 'CENSUS_WORKER', 'WORKER', etc.
            - data_dict: Parsed message data

        Non-STATUS lines are forwarded if forward_non_status=True.
        """
        for line in self.stream:
            line = line.rstrip('\n\r')

            if line.startswith('STATUS:'):
                msg_type, data = parse_status_message(line)
                if msg_type is not None:
                    yield (msg_type, data)
            elif self.forward_non_status:
                print(line, flush=True)


def parse_status_message(line: str) -> Tuple[Optional[str], Optional[Dict[str, Any]]]:
    """
    Parse STATUS messages from child processes.

    Supports four message types:
    1. STATUS:YEAR:2020:COMPLETE:24/50 (state progress)
    2. STATUS:WORKER:2020:1:STATE:12/50:california:STAGE:3/7:political_visualization
    3. STATUS:YEAR:2020:POSTPROCESS:PHASE:2/3:Visualization (post-processing phase)
    4. STATUS:WORKER:2020:1:TASK:3/6:National_district_map (post-processing worker)
    5. STATUS:CENSUS:{year}:STAGE:{stage_num}/{total}:{stage_name}:{completed}/{total_states}
    6. STATUS:CENSUS:{year}:WORKER:{worker_id}:STATE:{num}/{total}:{state}:STAGE:{stage_num}/{total}:{stage_name}

    Args:
        line: Raw STATUS message line

    Returns:
        Tuple of (message_type, data_dict) or (None, None) if not a STATUS message

    Examples:
        >>> parse_status_message("STATUS:YEAR:2020:COMPLETE:24/50")
        ('YEAR', {'year': '2020', 'completed': 24, 'total': 50})

        >>> parse_status_message("STATUS:CENSUS:2020:WORKER:0:STATE:5/50:CA:STAGE:1/3:Parsing")
        ('CENSUS_WORKER', {'year': '2020', 'worker_id': 0, 'state_num': 5, ...})
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

    elif msg_type == "CENSUS":
        # STATUS:CENSUS:{year}:STAGE:{stage_num}/{total}:{stage_name}:{completed}/{total_states}
        # STATUS:CENSUS:{year}:WORKER:{worker_id}:STATE:{state_num}/{total}:{state_code}:STAGE:{stage_num}/{total}:{stage_name}
        year = parts[2]
        submsg_type = parts[3]

        if submsg_type == "STAGE":
            # STATUS:CENSUS:2020:STAGE:1/3:Parsing PL 94-171:0/50
            if len(parts) >= 7:
                stage_str = parts[4]  # "1/3"
                stage_name = parts[5]
                progress_str = parts[6]  # "0/50"

                if '/' in progress_str:
                    completed, total = progress_str.split('/')
                    return ('CENSUS_STAGE', {
                        'year': year,
                        'stage_name': stage_name,
                        'completed': int(completed),
                        'total': int(total)
                    })

        elif submsg_type == "WORKER":
            # STATUS:CENSUS:2020:WORKER:0:STATE:1/50:CA:STAGE:1/3:Parsing PL 94-171
            # STATUS:CENSUS:2020:WORKER:0:STATE:5/50:CA:COMPLETE
            worker_id = int(parts[4])

            # Check for COMPLETE message (state finished)
            if len(parts) >= 9 and parts[5] == "STATE" and parts[8] == "COMPLETE":
                state_str = parts[6]  # "5/50"
                state_code = parts[7]

                if '/' in state_str:
                    completed, total = state_str.split('/')
                    return ('CENSUS_STAGE_PROGRESS', {
                        'year': year,
                        'worker_id': worker_id,
                        'state_code': state_code,
                        'completed': int(completed),
                        'total': int(total)
                    })

            # Check for in-progress message (state working)
            elif len(parts) >= 11 and parts[5] == "STATE" and parts[8] == "STAGE":
                state_str = parts[6]  # "1/50"
                state_code = parts[7]
                stage_str = parts[9]  # "1/3"
                stage_name = parts[10]

                if '/' in state_str and '/' in stage_str:
                    state_num, _ = state_str.split('/')
                    stage_num, stage_total = stage_str.split('/')

                    return ('CENSUS_WORKER', {
                        'year': year,
                        'worker_id': worker_id,
                        'state_num': int(state_num),
                        'state_name': state_code,
                        'stage_num': int(stage_num),
                        'stage_total': int(stage_total),
                        'stage_name': stage_name
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
