"""
Tests for STATUS protocol parser.

Tests parsing of different STATUS message formats.
"""
import pytest
from app.utils.status_parser import StatusParser, StatusMessageType


def test_parse_year_message():
    """Test parsing STATUS:YEAR message."""
    parser = StatusParser()
    line = "STATUS:YEAR:2020:COMPLETE:24/50"

    msg = parser.parse_line(line)

    assert msg is not None
    assert msg.type == StatusMessageType.YEAR
    assert msg.year == "2020"
    assert msg.states_completed == 24
    assert msg.states_total == 50


def test_parse_worker_message():
    """Test parsing STATUS:WORKER message."""
    parser = StatusParser()
    line = "STATUS:WORKER:2020:1:STATE:12/50:california:STAGE:3/7:district_maps"

    msg = parser.parse_line(line)

    assert msg is not None
    assert msg.type == StatusMessageType.WORKER
    assert msg.year == "2020"
    assert msg.worker_id == 1
    assert msg.states_completed == 12
    assert msg.states_total == 50
    assert msg.state_name == "california"
    assert msg.stage_current == 3
    assert msg.stage_total == 7
    assert msg.stage_name == "district_maps"


def test_parse_nation_message():
    """Test parsing STATUS:NATION message."""
    parser = StatusParser()
    line = "STATUS:NATION:2020:POSTPROCESS:3/9"

    msg = parser.parse_line(line)

    assert msg is not None
    assert msg.type == StatusMessageType.NATION
    assert msg.year == "2020"
    assert msg.stage_name == "POSTPROCESS"
    assert msg.stage_current == 3
    assert msg.stage_total == 9


def test_parse_non_status_line():
    """Test that non-STATUS lines return None."""
    parser = StatusParser()
    line = "Some regular output line"

    msg = parser.parse_line(line)

    assert msg is None


def test_parse_invalid_status_line():
    """Test parsing invalid STATUS line."""
    parser = StatusParser()
    line = "STATUS:UNKNOWN:FORMAT"

    msg = parser.parse_line(line)

    assert msg is not None
    assert msg.type == StatusMessageType.UNKNOWN
    assert msg.raw_line == line


def test_aggregate_progress_single_year():
    """Test aggregating progress for a single year."""
    parser = StatusParser()

    messages = [
        parser.parse_line("STATUS:YEAR:2020:COMPLETE:10/50"),
        parser.parse_line("STATUS:WORKER:2020:0:STATE:5/50:california:STAGE:3/7:district_maps"),
        parser.parse_line("STATUS:WORKER:2020:1:STATE:6/50:texas:STAGE:2/7:adjacency"),
    ]

    progress = parser.aggregate_progress([m for m in messages if m])

    assert "2020" in progress["years"]
    assert progress["years"]["2020"]["states_completed"] == 10
    assert progress["years"]["2020"]["states_total"] == 50
    assert progress["years"]["2020"]["progress"] == 0.2
    assert len(progress["years"]["2020"]["workers"]) == 2


def test_aggregate_progress_multi_year():
    """Test aggregating progress for multiple years."""
    parser = StatusParser()

    messages = [
        parser.parse_line("STATUS:YEAR:2020:COMPLETE:25/50"),
        parser.parse_line("STATUS:YEAR:2010:COMPLETE:10/50"),
    ]

    progress = parser.aggregate_progress([m for m in messages if m])

    assert len(progress["years"]) == 2
    assert progress["years"]["2020"]["progress"] == 0.5
    assert progress["years"]["2010"]["progress"] == 0.2
    assert progress["overall_progress"] == pytest.approx(0.35)  # (0.5 + 0.2) / 2


def test_aggregate_progress_empty():
    """Test aggregating with no messages."""
    parser = StatusParser()

    progress = parser.aggregate_progress([])

    assert progress["years"] == {}
    assert progress["overall_progress"] == 0.0


def test_parse_multiple_worker_updates():
    """Test parsing multiple updates from same worker."""
    parser = StatusParser()

    messages = [
        parser.parse_line("STATUS:WORKER:2020:0:STATE:1/50:alabama:STAGE:1/7:redistricting"),
        parser.parse_line("STATUS:WORKER:2020:0:STATE:1/50:alabama:STAGE:3/7:district_maps"),
    ]

    progress = parser.aggregate_progress([m for m in messages if m])

    # Should use latest worker status
    assert progress["years"]["2020"]["workers"]["0"]["stage"] == "3/7"
    assert progress["years"]["2020"]["workers"]["0"]["stage_name"] == "district_maps"
