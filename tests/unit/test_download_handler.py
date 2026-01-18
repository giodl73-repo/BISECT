#!/usr/bin/env python3
"""
Unit tests for download handler utilities.

Tests: scripts/data/download_handler.py
Created: 2026-01-18 (Enhancement #48)
"""

import pytest
from pathlib import Path
from unittest.mock import Mock, patch, MagicMock
import requests
from scripts.data.download_handler import (
    download_with_retry,
    download_with_fallback,
    check_existing_file,
    cleanup_on_failure,
    adaptive_rate_limit
)


class TestDownloadWithRetry:
    """Test download_with_retry function."""

    @patch('requests.get')
    def test_successful_download(self, mock_get, tmp_path):
        """Test successful download on first attempt."""
        # Mock successful response
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.headers = {'content-length': '100'}
        mock_response.iter_content = lambda chunk_size: [b'data']
        mock_get.return_value = mock_response

        output_path = tmp_path / 'test.dat'
        success = download_with_retry('https://example.com/file', output_path)

        assert success is True
        assert output_path.exists()
        mock_get.assert_called_once()

    @patch('requests.get')
    def test_retry_on_429(self, mock_get, tmp_path):
        """Test retry logic on 429 rate limit."""
        # First call returns 429, second succeeds
        mock_response_429 = Mock()
        mock_response_429.status_code = 429

        mock_response_200 = Mock()
        mock_response_200.status_code = 200
        mock_response_200.headers = {}
        mock_response_200.content = b'data'

        mock_get.side_effect = [mock_response_429, mock_response_200]

        output_path = tmp_path / 'test.dat'
        success = download_with_retry(
            'https://example.com/file',
            output_path,
            base_delay=0.01  # Fast for testing
        )

        assert success is True
        assert mock_get.call_count == 2

    @patch('requests.get')
    def test_max_retries_exceeded(self, mock_get, tmp_path):
        """Test failure after max retries."""
        # Always return 429
        mock_response = Mock()
        mock_response.status_code = 429
        mock_get.return_value = mock_response

        output_path = tmp_path / 'test.dat'
        success = download_with_retry(
            'https://example.com/file',
            output_path,
            max_retries=3,
            base_delay=0.01
        )

        assert success is False
        assert mock_get.call_count == 3

    @patch('requests.get')
    def test_cleanup_on_failure(self, mock_get, tmp_path):
        """Test that partial downloads are cleaned up on failure."""
        # Create a file first
        output_path = tmp_path / 'test.dat'
        output_path.write_text('partial')

        # Mock failure
        mock_get.side_effect = requests.exceptions.Timeout()

        success = download_with_retry(
            'https://example.com/file',
            output_path,
            max_retries=1,
            base_delay=0.01
        )

        assert success is False
        # File should be cleaned up
        assert not output_path.exists()

    @patch('requests.get')
    def test_progress_callback(self, mock_get, tmp_path):
        """Test that progress callback is called."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.headers = {}
        mock_response.content = b'data'
        mock_get.return_value = mock_response

        messages = []
        def callback(msg):
            messages.append(msg)

        output_path = tmp_path / 'test.dat'
        success = download_with_retry(
            'https://example.com/file',
            output_path,
            progress_callback=callback
        )

        assert success is True
        assert len(messages) > 0
        assert any('Downloaded' in msg for msg in messages)

    @patch('requests.get')
    def test_creates_output_directory(self, mock_get, tmp_path):
        """Test that output directory is created if missing."""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.headers = {}
        mock_response.content = b'data'
        mock_get.return_value = mock_response

        output_path = tmp_path / 'nested' / 'dir' / 'test.dat'
        success = download_with_retry('https://example.com/file', output_path)

        assert success is True
        assert output_path.parent.exists()
        assert output_path.exists()


class TestDownloadWithFallback:
    """Test download_with_fallback function."""

    @patch('scripts.data.download_handler.download_with_retry')
    def test_first_url_succeeds(self, mock_download, tmp_path):
        """Test that first URL succeeds."""
        mock_download.return_value = True

        urls = [
            'https://primary.com/file',
            'https://mirror.com/file'
        ]
        output_path = tmp_path / 'test.dat'

        success = download_with_fallback(urls, output_path)

        assert success is True
        mock_download.assert_called_once()  # Only tried first URL

    @patch('scripts.data.download_handler.download_with_retry')
    def test_falls_back_to_second_url(self, mock_download, tmp_path):
        """Test that second URL is tried after first fails."""
        # First fails, second succeeds
        mock_download.side_effect = [False, True]

        urls = [
            'https://primary.com/file',
            'https://mirror.com/file'
        ]
        output_path = tmp_path / 'test.dat'

        success = download_with_fallback(urls, output_path)

        assert success is True
        assert mock_download.call_count == 2

    @patch('scripts.data.download_handler.download_with_retry')
    def test_all_urls_fail(self, mock_download, tmp_path):
        """Test that failure is returned when all URLs fail."""
        mock_download.return_value = False

        urls = [
            'https://primary.com/file',
            'https://mirror1.com/file',
            'https://mirror2.com/file'
        ]
        output_path = tmp_path / 'test.dat'

        success = download_with_fallback(urls, output_path)

        assert success is False
        assert mock_download.call_count == 3


class TestCheckExistingFile:
    """Test check_existing_file function."""

    def test_file_exists_no_force(self, tmp_path):
        """Test that existing file returns True (skip download)."""
        file_path = tmp_path / 'existing.dat'
        file_path.write_text('data')

        should_skip = check_existing_file(file_path, force=False)

        assert should_skip is True

    def test_file_exists_with_force(self, tmp_path):
        """Test that force flag causes redownload."""
        file_path = tmp_path / 'existing.dat'
        file_path.write_text('data')

        should_skip = check_existing_file(file_path, force=True)

        assert should_skip is False

    def test_file_not_exists(self, tmp_path):
        """Test that missing file returns False (download needed)."""
        file_path = tmp_path / 'missing.dat'

        should_skip = check_existing_file(file_path, force=False)

        assert should_skip is False

    def test_progress_callback(self, tmp_path):
        """Test that callback is called for skip."""
        file_path = tmp_path / 'existing.dat'
        file_path.write_text('data')

        messages = []
        def callback(msg):
            messages.append(msg)

        should_skip = check_existing_file(file_path, progress_callback=callback)

        assert should_skip is True
        assert len(messages) == 1
        assert '[SKIP]' in messages[0]


class TestCleanupOnFailure:
    """Test cleanup_on_failure function."""

    def test_deletes_existing_file(self, tmp_path):
        """Test that existing file is deleted."""
        file_path = tmp_path / 'partial.dat'
        file_path.write_text('partial data')

        cleanup_on_failure(file_path)

        assert not file_path.exists()

    def test_handles_missing_file(self, tmp_path):
        """Test that missing file doesn't cause error."""
        file_path = tmp_path / 'missing.dat'

        # Should not raise
        cleanup_on_failure(file_path)

        assert not file_path.exists()


class TestAdaptiveRateLimit:
    """Test adaptive_rate_limit function."""

    def test_no_rate_limit_headers(self):
        """Test default delay when no headers present."""
        headers = {}
        delay = adaptive_rate_limit(headers, base_delay=1.0)

        assert delay == 1.0

    def test_low_remaining_requests(self):
        """Test increased delay when few requests remaining."""
        headers = {'X-RateLimit-Remaining': '5'}
        delay = adaptive_rate_limit(headers, base_delay=1.0)

        assert delay == 2.0  # Should double

    def test_moderate_remaining_requests(self):
        """Test moderate delay when some requests remaining."""
        headers = {'X-RateLimit-Remaining': '25'}
        delay = adaptive_rate_limit(headers, base_delay=1.0)

        assert delay == 1.5  # Should increase by 1.5x

    def test_high_remaining_requests(self):
        """Test no increase when many requests remaining."""
        headers = {'X-RateLimit-Remaining': '100'}
        delay = adaptive_rate_limit(headers, base_delay=1.0)

        assert delay == 1.0  # No increase

    def test_invalid_header_value(self):
        """Test that invalid header values don't crash."""
        headers = {'X-RateLimit-Remaining': 'invalid'}
        delay = adaptive_rate_limit(headers, base_delay=1.0)

        assert delay == 1.0  # Fall back to base delay
