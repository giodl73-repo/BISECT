#!/usr/bin/env python3
"""
Unit tests for STATUS protocol module.

Tests StatusReporter (child-side message generation) and StatusReader (parent-side
message parsing) for all message types.
"""

import os
import sys
import io
from unittest.mock import patch
import pytest

# Add project root to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..'))

from scripts.utils.status_protocol import StatusReporter, StatusReader, parse_status_message


class TestStatusReporter:
    """Tests for StatusReporter class (child-side message generation)."""

    def test_init_standalone_default(self):
        """Test initialization in standalone mode (default)."""
        reporter = StatusReporter()
        assert reporter.position == -1
        assert reporter.is_standalone is True
        assert reporter.send_status is False

    def test_init_child_mode(self):
        """Test initialization in child mode (TQDM_POSITION set)."""
        with patch.dict(os.environ, {'TQDM_POSITION': '0'}):
            reporter = StatusReporter()
            assert reporter.position == 0
            assert reporter.is_standalone is False
            assert reporter.send_status is True

    def test_init_explicit_position(self):
        """Test initialization with explicit position override."""
        reporter = StatusReporter(position=5)
        assert reporter.position == 5
        assert reporter.is_standalone is False
        assert reporter.send_status is True

    def test_report_standalone(self, capsys):
        """Test basic report in standalone mode."""
        reporter = StatusReporter(position=-1)
        reporter.report("Processing california")

        captured = capsys.readouterr()
        assert captured.out == "Processing california\n"
        assert "STATUS:" not in captured.out

    def test_report_child_mode(self, capsys):
        """Test basic report in child mode."""
        reporter = StatusReporter(position=0)
        reporter.report("Processing california")

        captured = capsys.readouterr()
        assert captured.out == "STATUS:0:Processing california\n"

    def test_report_census_stage_standalone(self, capsys):
        """Test census stage report in standalone mode."""
        reporter = StatusReporter(position=-1)
        reporter.report_census_stage("2020", "Parsing PL 94-171", 5, 50)

        captured = capsys.readouterr()
        assert "[2020] Parsing PL 94-171: 5/50\n" == captured.out

    def test_report_census_stage_child_mode(self, capsys):
        """Test census stage report in child mode."""
        reporter = StatusReporter(position=0)
        reporter.report_census_stage("2020", "Parsing PL 94-171", 5, 50)

        captured = capsys.readouterr()
        assert captured.out == "STATUS:CENSUS:2020:STAGE:1/3:Parsing PL 94-171:5/50\n"

    def test_report_census_worker_standalone(self, capsys):
        """Test census worker report in standalone mode."""
        reporter = StatusReporter(position=-1)
        reporter.report_census_worker("2020", 0, 5, "CA", 1, 3, "Parsing PL 94-171")

        captured = capsys.readouterr()
        assert "Worker 0: [5/50] CA - Parsing PL 94-171\n" == captured.out

    def test_report_census_worker_child_mode(self, capsys):
        """Test census worker report in child mode."""
        reporter = StatusReporter(position=0)
        reporter.report_census_worker("2020", 0, 5, "CA", 1, 3, "Parsing PL 94-171")

        captured = capsys.readouterr()
        assert captured.out == "STATUS:CENSUS:2020:WORKER:0:STATE:5/50:CA:STAGE:1/3:Parsing PL 94-171\n"

    def test_report_census_worker_complete_standalone(self, capsys):
        """Test census worker completion in standalone mode."""
        reporter = StatusReporter(position=-1)
        reporter.report_census_worker_complete("2020", 0, 5, "CA", 50)

        captured = capsys.readouterr()
        assert "Worker 0: [5/50] CA complete\n" == captured.out

    def test_report_census_worker_complete_child_mode(self, capsys):
        """Test census worker completion in child mode."""
        reporter = StatusReporter(position=0)
        reporter.report_census_worker_complete("2020", 0, 5, "CA", 50)

        captured = capsys.readouterr()
        assert captured.out == "STATUS:CENSUS:2020:WORKER:0:STATE:5/50:CA:COMPLETE\n"

    def test_report_year_complete_standalone(self, capsys):
        """Test year completion report in standalone mode."""
        reporter = StatusReporter(position=-1)
        reporter.report_year_complete("2020", 24, 50)

        captured = capsys.readouterr()
        assert "[2020] 24/50 states complete\n" == captured.out

    def test_report_year_complete_child_mode(self, capsys):
        """Test year completion report in child mode."""
        reporter = StatusReporter(position=0)
        reporter.report_year_complete("2020", 24, 50)

        captured = capsys.readouterr()
        assert captured.out == "STATUS:YEAR:2020:COMPLETE:24/50\n"

    def test_report_year_postprocess_standalone(self, capsys):
        """Test year post-processing in standalone mode."""
        reporter = StatusReporter(position=-1)
        reporter.report_year_postprocess("2020", 3, 9)

        captured = capsys.readouterr()
        assert "[2020] Post-processing: 3/9 tasks\n" == captured.out

    def test_report_year_postprocess_child_mode(self, capsys):
        """Test year post-processing in child mode."""
        reporter = StatusReporter(position=0)
        reporter.report_year_postprocess("2020", 3, 9)

        captured = capsys.readouterr()
        assert captured.out == "STATUS:YEAR:2020:POSTPROCESS:3/9\n"

    def test_report_worker_state_standalone(self, capsys):
        """Test worker state report in standalone mode."""
        reporter = StatusReporter(position=-1)
        reporter.report_worker_state("2020", 1, 12, "california", 3, 7, "political_visualization")

        captured = capsys.readouterr()
        assert "Worker 1: [12/50] california - Stage 3/7: political_visualization\n" == captured.out

    def test_report_worker_state_child_mode(self, capsys):
        """Test worker state report in child mode."""
        reporter = StatusReporter(position=0)
        reporter.report_worker_state("2020", 1, 12, "california", 3, 7, "political_visualization")

        captured = capsys.readouterr()
        assert captured.out == "STATUS:WORKER:2020:1:STATE:12/50:california:STAGE:3/7:political_visualization\n"

    def test_report_worker_task_standalone(self, capsys):
        """Test worker task report in standalone mode."""
        reporter = StatusReporter(position=-1)
        reporter.report_worker_task("2020", 1, 3, 9, "National_district_map")

        captured = capsys.readouterr()
        assert "Worker 1: Task 3/9 - National_district_map\n" == captured.out

    def test_report_worker_task_child_mode(self, capsys):
        """Test worker task report in child mode."""
        reporter = StatusReporter(position=0)
        reporter.report_worker_task("2020", 1, 3, 9, "National_district_map")

        captured = capsys.readouterr()
        assert captured.out == "STATUS:WORKER:2020:1:TASK:3/9:National_district_map\n"

    def test_report_redistricting_stage_standalone(self, capsys):
        """Test redistricting stage report in standalone mode (batch mode)."""
        reporter = StatusReporter(position=-1)
        reporter.report_redistricting_stage("2020", "metis", 25, 50)

        captured = capsys.readouterr()
        assert "[2020] metis: 25/50 states\n" == captured.out

    def test_report_redistricting_stage_child_mode(self, capsys):
        """Test redistricting stage report in child mode (batch mode)."""
        reporter = StatusReporter(position=0)
        reporter.report_redistricting_stage("2020", "metis", 25, 50)

        captured = capsys.readouterr()
        assert captured.out == "STATUS:REDISTRICTING:2020:STAGE:metis:25/50\n"


class TestParseStatusMessage:
    """Tests for parse_status_message function."""

    def test_parse_not_status_message(self):
        """Test parsing non-STATUS line."""
        msg_type, data = parse_status_message("Regular output line")
        assert msg_type is None
        assert data is None

    def test_parse_malformed_status(self):
        """Test parsing malformed STATUS message."""
        msg_type, data = parse_status_message("STATUS:")
        assert msg_type is None
        assert data is None

    def test_parse_year_complete(self):
        """Test parsing YEAR complete message."""
        msg_type, data = parse_status_message("STATUS:YEAR:2020:COMPLETE:24/50")

        assert msg_type == 'YEAR'
        assert data == {
            'year': '2020',
            'completed': 24,
            'total': 50
        }

    def test_parse_year_postprocess(self):
        """Test parsing YEAR post-processing message."""
        msg_type, data = parse_status_message("STATUS:YEAR:2020:POSTPROCESS:3/9")

        assert msg_type == 'YEAR_POSTPROCESS'
        assert data == {
            'year': '2020',
            'completed': 3,
            'total': 9
        }

    def test_parse_census_stage(self):
        """Test parsing CENSUS stage message."""
        msg_type, data = parse_status_message("STATUS:CENSUS:2020:STAGE:1/3:Parsing PL 94-171:5/50")

        assert msg_type == 'CENSUS_STAGE'
        assert data == {
            'year': '2020',
            'stage_name': 'Parsing PL 94-171',
            'completed': 5,
            'total': 50
        }

    def test_parse_census_worker(self):
        """Test parsing CENSUS worker message."""
        msg_type, data = parse_status_message(
            "STATUS:CENSUS:2020:WORKER:0:STATE:5/50:CA:STAGE:1/3:Parsing PL 94-171"
        )

        assert msg_type == 'CENSUS_WORKER'
        assert data == {
            'year': '2020',
            'worker_id': 0,
            'state_num': 5,
            'state_name': 'CA',
            'stage_num': 1,
            'stage_total': 3,
            'stage_name': 'Parsing PL 94-171'
        }

    def test_parse_census_worker_complete(self):
        """Test parsing CENSUS worker complete message."""
        msg_type, data = parse_status_message("STATUS:CENSUS:2020:WORKER:0:STATE:5/50:CA:COMPLETE")

        assert msg_type == 'CENSUS_STAGE_PROGRESS'
        assert data == {
            'year': '2020',
            'completed': 5,
            'total': 50
        }

    def test_parse_worker_state(self):
        """Test parsing WORKER state message."""
        msg_type, data = parse_status_message(
            "STATUS:WORKER:2020:1:STATE:12/50:california:STAGE:3/7:political_visualization"
        )

        assert msg_type == 'WORKER'
        assert data == {
            'year': '2020',
            'worker_id': 1,
            'state_num': 12,
            'state_name': 'california',
            'stage': 3,
            'stage_total': 7,
            'stage_desc': 'political_visualization'
        }

    def test_parse_worker_task(self):
        """Test parsing WORKER task message."""
        msg_type, data = parse_status_message("STATUS:WORKER:2020:1:TASK:3/9:National_district_map")

        assert msg_type == 'WORKER_TASK'
        assert data == {
            'year': '2020',
            'worker_id': 1,
            'task_index': 3,
            'task_total': 9,
            'task_name': 'National_district_map'
        }

    def test_parse_redistricting_stage(self):
        """Test parsing REDISTRICTING_STAGE message (batch mode)."""
        msg_type, data = parse_status_message("STATUS:REDISTRICTING:2020:STAGE:metis:25/50")

        assert msg_type == 'REDISTRICTING_STAGE'
        assert data == {
            'year': '2020',
            'stage_name': 'metis',
            'completed': 25,
            'total': 50
        }

    def test_parse_redistricting_stage_summary(self):
        """Test parsing REDISTRICTING_STAGE message for summary stage."""
        msg_type, data = parse_status_message("STATUS:REDISTRICTING:2020:STAGE:summary:50/50")

        assert msg_type == 'REDISTRICTING_STAGE'
        assert data == {
            'year': '2020',
            'stage_name': 'summary',
            'completed': 50,
            'total': 50
        }


class TestStatusReader:
    """Tests for StatusReader class (parent-side message parsing)."""

    def test_iter_status_messages_single_message(self):
        """Test iterating over single STATUS message."""
        stream = io.StringIO("STATUS:YEAR:2020:COMPLETE:24/50\n")
        reader = StatusReader(stream)

        messages = list(reader.iter_status_messages())
        assert len(messages) == 1

        msg_type, data = messages[0]
        assert msg_type == 'YEAR'
        assert data['year'] == '2020'
        assert data['completed'] == 24

    def test_iter_status_messages_multiple_messages(self):
        """Test iterating over multiple STATUS messages."""
        stream = io.StringIO(
            "STATUS:YEAR:2020:COMPLETE:24/50\n"
            "STATUS:CENSUS:2020:STAGE:1/3:Parsing:5/50\n"
        )
        reader = StatusReader(stream)

        messages = list(reader.iter_status_messages())
        assert len(messages) == 2

        assert messages[0][0] == 'YEAR'
        assert messages[1][0] == 'CENSUS_STAGE'

    def test_iter_status_messages_mixed_output(self):
        """Test iterating with mixed STATUS and non-STATUS lines."""
        stream = io.StringIO(
            "Regular output line\n"
            "STATUS:YEAR:2020:COMPLETE:24/50\n"
            "Another regular line\n"
        )
        reader = StatusReader(stream, forward_non_status=False)

        messages = list(reader.iter_status_messages())
        assert len(messages) == 1
        assert messages[0][0] == 'YEAR'

    def test_iter_status_messages_forward_non_status(self, capsys):
        """Test forwarding non-STATUS lines."""
        stream = io.StringIO(
            "Regular output line\n"
            "STATUS:YEAR:2020:COMPLETE:24/50\n"
        )
        reader = StatusReader(stream, forward_non_status=True)

        messages = list(reader.iter_status_messages())

        captured = capsys.readouterr()
        assert "Regular output line" in captured.out
        assert len(messages) == 1

    def test_iter_status_messages_empty_stream(self):
        """Test iterating over empty stream."""
        stream = io.StringIO("")
        reader = StatusReader(stream)

        messages = list(reader.iter_status_messages())
        assert len(messages) == 0

    def test_iter_status_messages_malformed_status(self):
        """Test handling malformed STATUS messages."""
        stream = io.StringIO(
            "STATUS:INVALID\n"
            "STATUS:YEAR:2020:COMPLETE:24/50\n"
        )
        reader = StatusReader(stream)

        messages = list(reader.iter_status_messages())
        # Malformed message is skipped, only valid message is returned
        assert len(messages) == 1
        assert messages[0][0] == 'YEAR'


class TestIntegration:
    """Integration tests combining reporter and reader."""

    def test_reporter_to_reader_basic(self):
        """Test basic message flow from reporter to reader."""
        # Capture reporter output
        output = io.StringIO()
        with patch('sys.stdout', output):
            reporter = StatusReporter(position=0)
            reporter.report("Processing california")

        # Parse with reader
        output.seek(0)
        reader = StatusReader(output)
        messages = list(reader.iter_status_messages())

        # Note: Basic report messages don't have a specific type in parse_status_message
        # They would be handled by the parent but not parsed into structured data
        assert len(messages) == 0  # Basic messages aren't structured

    def test_reporter_to_reader_census(self):
        """Test CENSUS message flow from reporter to reader."""
        # Capture reporter output
        output = io.StringIO()
        with patch('sys.stdout', output):
            reporter = StatusReporter(position=0)
            reporter.report_census_worker("2020", 0, 5, "CA", 1, 3, "Parsing PL 94-171")

        # Parse with reader
        output.seek(0)
        reader = StatusReader(output)
        messages = list(reader.iter_status_messages())

        assert len(messages) == 1
        msg_type, data = messages[0]
        assert msg_type == 'CENSUS_WORKER'
        assert data['year'] == '2020'
        assert data['state_name'] == 'CA'

    def test_reporter_to_reader_year(self):
        """Test YEAR message flow from reporter to reader."""
        # Capture reporter output
        output = io.StringIO()
        with patch('sys.stdout', output):
            reporter = StatusReporter(position=0)
            reporter.report_year_complete("2020", 24, 50)

        # Parse with reader
        output.seek(0)
        reader = StatusReader(output)
        messages = list(reader.iter_status_messages())

        assert len(messages) == 1
        msg_type, data = messages[0]
        assert msg_type == 'YEAR'
        assert data['completed'] == 24
        assert data['total'] == 50

    def test_reporter_to_reader_worker(self):
        """Test WORKER message flow from reporter to reader."""
        # Capture reporter output
        output = io.StringIO()
        with patch('sys.stdout', output):
            reporter = StatusReporter(position=0)
            reporter.report_worker_state("2020", 1, 12, "california", 3, 7, "political_visualization")

        # Parse with reader
        output.seek(0)
        reader = StatusReader(output)
        messages = list(reader.iter_status_messages())

        assert len(messages) == 1
        msg_type, data = messages[0]
        assert msg_type == 'WORKER'
        assert data['worker_id'] == 1
        assert data['state_name'] == 'california'
        assert data['stage'] == 3


if __name__ == '__main__':
    pytest.main([__file__, '-v'])
