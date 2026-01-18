#!/usr/bin/env python3
"""
Unit tests for download progress coordinator.

Tests: scripts/data/download_progress.py
Created: 2026-01-18 (Enhancement #48)
"""

import pytest
from scripts.data.download_progress import (
    DownloadCoordinator,
    parse_download_status
)


class TestDownloadCoordinator:
    """Test DownloadCoordinator class."""

    def test_initialization(self):
        """Test coordinator initializes correctly."""
        coordinator = DownloadCoordinator(
            download_type='tracts',
            years=['2020', '2010'],
            workers_per_year=[2, 2]
        )

        assert coordinator.download_type == 'tracts'
        assert coordinator.years == ['2020', '2010']
        assert len(coordinator.year_progress) == 2
        assert coordinator.year_progress['2020']['completed'] == 0
        assert coordinator.year_progress['2020']['total'] == 50

    def test_update_year_progress(self):
        """Test updating year-level progress."""
        coordinator = DownloadCoordinator(years=['2020'])

        coordinator.update_year_progress('2020', completed=10, total=50)

        assert coordinator.year_progress['2020']['completed'] == 10
        assert coordinator.year_progress['2020']['total'] == 50

    def test_update_worker_status(self):
        """Test updating worker-level status."""
        coordinator = DownloadCoordinator(years=['2020'], workers_per_year=[2])

        coordinator.update_worker_status(
            year='2020',
            worker_id=0,
            state_num=5,
            state_name='california',
            step=2,
            step_total=4,
            step_desc='downloading_zip'
        )

        key = ('2020', 0)
        assert key in coordinator.worker_status
        status = coordinator.worker_status[key]
        assert status['state_num'] == 5
        assert status['state_name'] == 'california'
        assert status['step'] == 2
        assert status['step_total'] == 4
        assert status['step_desc'] == 'downloading_zip'

    def test_render_empty(self):
        """Test rendering with no progress."""
        coordinator = DownloadCoordinator(years=['2020'], workers_per_year=[1])

        output = coordinator.render()

        assert '[2020]' in output
        assert '0/50' in output
        assert 'Worker 1' in output

    def test_render_with_progress(self):
        """Test rendering with some progress."""
        coordinator = DownloadCoordinator(years=['2020'], workers_per_year=[2])

        coordinator.update_year_progress('2020', completed=10, total=50)
        coordinator.update_worker_status(
            year='2020',
            worker_id=0,
            state_num=10,
            state_name='texas',
            step=3,
            step_total=4,
            step_desc='extracting_zip'
        )

        output = coordinator.render()

        assert '[2020]' in output
        assert '10/50' in output
        assert 'texas' in output
        assert '3/4' in output

    def test_render_complete(self):
        """Test rendering when download complete."""
        coordinator = DownloadCoordinator(years=['2020'], workers_per_year=[2])

        coordinator.update_year_progress('2020', completed=50, total=50)

        output = coordinator.render()

        assert 'COMPLETE' in output
        assert 'Idle' in output  # Workers should show as idle

    def test_multi_year_rendering(self):
        """Test rendering with multiple years."""
        coordinator = DownloadCoordinator(
            years=['2020', '2010', '2000'],
            workers_per_year=[2, 2, 2]
        )

        coordinator.update_year_progress('2020', 15, 50)
        coordinator.update_year_progress('2010', 30, 50)
        coordinator.update_year_progress('2000', 5, 50)

        output = coordinator.render()

        assert '[2020]' in output
        assert '[2010]' in output
        assert '[2000]' in output
        assert '15/50' in output
        assert '30/50' in output
        assert '5/50' in output


class TestParseDownloadStatus:
    """Test parse_download_status function."""

    def test_parse_year_progress(self):
        """Test parsing year progress message."""
        line = "STATUS:DOWNLOAD:2020:COMPLETE:24/50"

        msg_type, data = parse_download_status(line)

        assert msg_type == 'YEAR'
        assert data['year'] == '2020'
        assert data['completed'] == 24
        assert data['total'] == 50

    def test_parse_worker_status(self):
        """Test parsing worker status message."""
        line = "STATUS:WORKER:2020:1:STATE:12/50:california:STEP:3/4:extracting_zip"

        msg_type, data = parse_download_status(line)

        assert msg_type == 'WORKER'
        assert data['year'] == '2020'
        assert data['worker_id'] == 1
        assert data['state_num'] == 12
        assert data['state_name'] == 'california'
        assert data['step'] == 3
        assert data['step_total'] == 4
        assert data['step_desc'] == 'extracting_zip'

    def test_parse_error_message(self):
        """Test parsing error message."""
        line = "STATUS:DOWNLOAD:2020:ERROR:california:Connection timeout"

        msg_type, data = parse_download_status(line)

        assert msg_type == 'ERROR'
        assert data['year'] == '2020'
        assert data['state_name'] == 'california'
        assert data['error'] == 'Connection timeout'

    def test_parse_error_with_colon_in_message(self):
        """Test parsing error message with colons in error text."""
        line = "STATUS:DOWNLOAD:2020:ERROR:texas:HTTP 429: Rate limit exceeded"

        msg_type, data = parse_download_status(line)

        assert msg_type == 'ERROR'
        assert data['year'] == '2020'
        assert data['state_name'] == 'texas'
        assert data['error'] == 'HTTP 429: Rate limit exceeded'

    def test_parse_non_status_line(self):
        """Test that non-STATUS lines return None."""
        line = "Regular log message"

        msg_type, data = parse_download_status(line)

        assert msg_type is None
        assert data is None

    def test_parse_malformed_status(self):
        """Test that malformed STATUS lines return None."""
        line = "STATUS:INVALID"

        msg_type, data = parse_download_status(line)

        assert msg_type is None
        assert data is None

    def test_parse_incomplete_worker_status(self):
        """Test that incomplete worker status returns None."""
        line = "STATUS:WORKER:2020:1:STATE:12"

        msg_type, data = parse_download_status(line)

        assert msg_type is None
        assert data is None


class TestThreadSafety:
    """Test thread-safe operations."""

    def test_concurrent_updates(self):
        """Test that concurrent updates don't cause crashes."""
        import threading

        coordinator = DownloadCoordinator(years=['2020'], workers_per_year=[4])

        def update_progress():
            for i in range(10):
                coordinator.update_year_progress('2020', i, 50)

        def update_worker(worker_id):
            for i in range(10):
                coordinator.update_worker_status(
                    '2020', worker_id, i, 'state', 1, 4, 'downloading'
                )

        threads = []
        threads.append(threading.Thread(target=update_progress))
        for w in range(4):
            threads.append(threading.Thread(target=update_worker, args=(w,)))

        for t in threads:
            t.start()
        for t in threads:
            t.join()

        # Should not crash, and render should work
        output = coordinator.render()
        assert '[2020]' in output
