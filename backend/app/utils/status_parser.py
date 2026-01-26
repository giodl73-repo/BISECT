"""
STATUS protocol parser for pipeline execution.

Parses STATUS messages from pipeline stdout to track progress.
"""
import re
from typing import Optional, Dict, Any, List
from dataclasses import dataclass
from enum import Enum


class StatusMessageType(str, Enum):
    """Types of STATUS messages."""
    YEAR = "year"
    WORKER = "worker"
    NATION = "nation"
    UNKNOWN = "unknown"


@dataclass
class StatusMessage:
    """Parsed STATUS message."""
    type: StatusMessageType
    year: Optional[str] = None
    worker_id: Optional[int] = None
    state_name: Optional[str] = None
    states_completed: Optional[int] = None
    states_total: Optional[int] = None
    stage_current: Optional[int] = None
    stage_total: Optional[int] = None
    stage_name: Optional[str] = None
    raw_line: str = ""


class StatusParser:
    """
    Parse STATUS protocol messages from pipeline output.

    Supports multiple message formats:
    - STATUS:YEAR:2020:COMPLETE:24/50
    - STATUS:WORKER:2020:1:STATE:12/50:california:STAGE:3/7:district_maps
    - STATUS:NATION:2020:POSTPROCESS:3/9
    """

    # STATUS:YEAR:2020:COMPLETE:24/50
    YEAR_PATTERN = re.compile(r"STATUS:YEAR:(\d{4}):(\w+):(\d+)/(\d+)")

    # STATUS:WORKER:2020:1:STATE:12/50:california:STAGE:3/7:district_maps
    WORKER_PATTERN = re.compile(
        r"STATUS:WORKER:(\d{4}):(\d+):STATE:(\d+)/(\d+):(\w+):STAGE:(\d+)/(\d+):(.+)"
    )

    # STATUS:NATION:2020:POSTPROCESS:3/9
    NATION_PATTERN = re.compile(r"STATUS:NATION:(\d{4}):(\w+):(\d+)/(\d+)")

    def parse_line(self, line: str) -> Optional[StatusMessage]:
        """
        Parse a single STATUS line.

        Args:
            line: Line from pipeline stdout

        Returns:
            Parsed StatusMessage or None if not a STATUS line
        """
        line = line.strip()

        if not line.startswith("STATUS:"):
            return None

        # Try year pattern
        match = self.YEAR_PATTERN.match(line)
        if match:
            year, status, completed, total = match.groups()
            return StatusMessage(
                type=StatusMessageType.YEAR,
                year=year,
                states_completed=int(completed),
                states_total=int(total),
                raw_line=line,
            )

        # Try worker pattern
        match = self.WORKER_PATTERN.match(line)
        if match:
            year, worker_id, state_num, state_total, state_name, stage_current, stage_total, stage_name = match.groups()
            return StatusMessage(
                type=StatusMessageType.WORKER,
                year=year,
                worker_id=int(worker_id),
                state_name=state_name,
                states_completed=int(state_num),
                states_total=int(state_total),
                stage_current=int(stage_current),
                stage_total=int(stage_total),
                stage_name=stage_name,
                raw_line=line,
            )

        # Try nation pattern
        match = self.NATION_PATTERN.match(line)
        if match:
            year, stage_name, task_current, task_total = match.groups()
            return StatusMessage(
                type=StatusMessageType.NATION,
                year=year,
                stage_name=stage_name,
                stage_current=int(task_current),
                stage_total=int(task_total),
                raw_line=line,
            )

        # Unknown STATUS format
        return StatusMessage(
            type=StatusMessageType.UNKNOWN,
            raw_line=line,
        )

    def aggregate_progress(self, messages: List[StatusMessage]) -> Dict[str, Any]:
        """
        Aggregate multiple STATUS messages into progress structure.

        Args:
            messages: List of parsed StatusMessage objects

        Returns:
            Progress dictionary with year-level and overall progress
        """
        progress = {
            "years": {},
            "overall_progress": 0.0,
        }

        # Group messages by year
        year_messages = {}
        for msg in messages:
            if msg.year:
                if msg.year not in year_messages:
                    year_messages[msg.year] = []
                year_messages[msg.year].append(msg)

        # Calculate progress for each year
        for year, msgs in year_messages.items():
            year_progress = self._calculate_year_progress(msgs)
            progress["years"][year] = year_progress

        # Calculate overall progress
        if progress["years"]:
            total_progress = sum(y.get("progress", 0.0) for y in progress["years"].values())
            progress["overall_progress"] = total_progress / len(progress["years"])

        return progress

    def _calculate_year_progress(self, messages: List[StatusMessage]) -> Dict[str, Any]:
        """Calculate progress for a single year from messages."""
        # Find latest year-level message
        year_msg = None
        for msg in reversed(messages):
            if msg.type == StatusMessageType.YEAR:
                year_msg = msg
                break

        # Find worker messages
        worker_messages = [m for m in messages if m.type == StatusMessageType.WORKER]

        # Build year progress
        year_progress = {
            "states_completed": 0,
            "states_total": 50,
            "progress": 0.0,
            "workers": {},
        }

        if year_msg:
            year_progress["states_completed"] = year_msg.states_completed or 0
            year_progress["states_total"] = year_msg.states_total or 50
            if year_progress["states_total"] > 0:
                year_progress["progress"] = year_progress["states_completed"] / year_progress["states_total"]

        # Add worker details
        for msg in worker_messages:
            if msg.worker_id is not None:
                year_progress["workers"][str(msg.worker_id)] = {
                    "state": msg.state_name,
                    "stage": f"{msg.stage_current}/{msg.stage_total}",
                    "stage_name": msg.stage_name,
                }

        return year_progress
