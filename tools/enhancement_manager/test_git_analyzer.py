#!/usr/bin/env python3
"""Unit tests for git_analyzer.py"""

import json
import unittest
from pathlib import Path
from unittest.mock import Mock, patch, MagicMock

from git_analyzer import GitAnalyzer, CommitInfo


class TestEnhancementNumberExtraction(unittest.TestCase):
    """Test extraction of enhancement numbers from commit messages."""

    def setUp(self):
        self.analyzer = GitAnalyzer(verbose=False)

    def test_basic_pattern(self):
        """Test 'Enhancement N: Title' pattern."""
        message = "Enhancement 39: Add Error Logging System"
        numbers = self.analyzer.extract_enhancement_numbers(message)
        self.assertEqual(numbers, [39])

    def test_phase_pattern(self):
        """Test 'Enhancement N Phase M' pattern."""
        message = "Enhancement 47 Phase 2: Parse and Merge Census Data"
        numbers = self.analyzer.extract_enhancement_numbers(message)
        self.assertEqual(numbers, [47])

    def test_mark_completed_pattern(self):
        """Test 'Mark Enhancement N' pattern."""
        message = "Mark Enhancement 46 as Completed"
        numbers = self.analyzer.extract_enhancement_numbers(message)
        self.assertEqual(numbers, [46])

    def test_update_pattern(self):
        """Test 'Update Enhancement N' pattern."""
        message = "Update Enhancement 48 with commit metadata"
        numbers = self.analyzer.extract_enhancement_numbers(message)
        self.assertEqual(numbers, [48])

    def test_batch_proposal_pattern(self):
        """Test 'Add Enhancements N-M' batch pattern."""
        message = "Add Enhancements 42-46: Proposals for Q1"
        numbers = self.analyzer.extract_enhancement_numbers(message)
        self.assertEqual(numbers, [42, 43, 44, 45, 46])

    def test_batch_proposal_singular(self):
        """Test 'Add Enhancement N-M' (singular) batch pattern."""
        message = "Add Enhancement 10-12"
        numbers = self.analyzer.extract_enhancement_numbers(message)
        self.assertEqual(numbers, [10, 11, 12])

    def test_multiple_enhancements_in_message(self):
        """Test message referencing multiple enhancements."""
        message = "Enhancement 35: Web UI\nBuilds on Enhancement 19 patterns"
        numbers = self.analyzer.extract_enhancement_numbers(message)
        self.assertEqual(numbers, [19, 35])

    def test_no_enhancement_reference(self):
        """Test commit with no enhancement reference."""
        message = "Fix typo in README"
        numbers = self.analyzer.extract_enhancement_numbers(message)
        self.assertEqual(numbers, [])

    def test_case_insensitive(self):
        """Test case-insensitive matching."""
        message = "ENHANCEMENT 50: ALL CAPS TITLE"
        numbers = self.analyzer.extract_enhancement_numbers(message)
        self.assertEqual(numbers, [50])

    def test_space_after_number(self):
        """Test 'Enhancement N ' pattern (space, no colon)."""
        message = "Enhancement 39 logging system implementation"
        numbers = self.analyzer.extract_enhancement_numbers(message)
        self.assertEqual(numbers, [39])

    def test_complete_pattern(self):
        """Test 'Complete Enhancement N' pattern."""
        message = "Complete Enhancement 42: Final touches"
        numbers = self.analyzer.extract_enhancement_numbers(message)
        self.assertEqual(numbers, [42])

    def test_fix_pattern(self):
        """Test 'Fix Enhancement N' pattern."""
        message = "Fix Enhancement 35: Resolve UI bug"
        numbers = self.analyzer.extract_enhancement_numbers(message)
        self.assertEqual(numbers, [35])


class TestCommitInfo(unittest.TestCase):
    """Test CommitInfo dataclass."""

    def test_total_lines_calculation(self):
        """Test total_lines property."""
        commit = CommitInfo(
            sha='abc123',
            message='Test commit',
            date='2026-01-18',
            lines_added=100,
            lines_deleted=50,
            files_modified=5
        )
        self.assertEqual(commit.total_lines, 150)

    def test_zero_changes(self):
        """Test commit with no changes."""
        commit = CommitInfo(
            sha='def456',
            message='Empty commit',
            date='2026-01-18',
            lines_added=0,
            lines_deleted=0,
            files_modified=0
        )
        self.assertEqual(commit.total_lines, 0)


class TestGitHubURLGeneration(unittest.TestCase):
    """Test GitHub URL generation."""

    def setUp(self):
        self.analyzer = GitAnalyzer(verbose=False)

    def test_github_url_format(self):
        """Test GitHub commit URL is correctly formatted."""
        sha = 'abc123def456'
        url = self.analyzer.generate_github_url(sha)
        expected = 'https://github.com/giodl_microsoft/redistricting/commit/abc123def456'
        self.assertEqual(url, expected)

    def test_short_sha(self):
        """Test GitHub URL with short SHA."""
        sha = 'abc1234'
        url = self.analyzer.generate_github_url(sha)
        expected = 'https://github.com/giodl_microsoft/redistricting/commit/abc1234'
        self.assertEqual(url, expected)


class TestMetricsAggregation(unittest.TestCase):
    """Test aggregation of size metrics across multiple commits."""

    def setUp(self):
        self.analyzer = GitAnalyzer(verbose=False)

    def test_single_commit_aggregation(self):
        """Test aggregation for single commit."""
        commits = [
            CommitInfo(
                sha='abc123',
                message='Test',
                date='2026-01-18',
                lines_added=100,
                lines_deleted=50,
                files_modified=5
            )
        ]
        metrics = self.analyzer.aggregate_metrics(commits)
        self.assertEqual(metrics['lines_added'], 100)
        self.assertEqual(metrics['lines_deleted'], 50)
        self.assertEqual(metrics['total_lines'], 150)
        self.assertEqual(metrics['total_files'], 5)

    def test_multiple_commits_aggregation(self):
        """Test aggregation across multiple commits."""
        commits = [
            CommitInfo('sha1', 'Msg1', '2026-01-18', 100, 50, 5),
            CommitInfo('sha2', 'Msg2', '2026-01-17', 200, 75, 8),
            CommitInfo('sha3', 'Msg3', '2026-01-16', 50, 25, 3),
        ]
        metrics = self.analyzer.aggregate_metrics(commits)
        self.assertEqual(metrics['lines_added'], 350)
        self.assertEqual(metrics['lines_deleted'], 150)
        self.assertEqual(metrics['total_lines'], 500)
        self.assertEqual(metrics['total_files'], 16)

    def test_empty_commits_list(self):
        """Test aggregation with empty commits list."""
        commits = []
        metrics = self.analyzer.aggregate_metrics(commits)
        self.assertEqual(metrics['lines_added'], 0)
        self.assertEqual(metrics['lines_deleted'], 0)
        self.assertEqual(metrics['total_lines'], 0)
        self.assertEqual(metrics['total_files'], 0)


class TestGitCommands(unittest.TestCase):
    """Test git command execution (mocked)."""

    @patch('git_analyzer.subprocess.run')
    def test_run_git_command_success(self, mock_run):
        """Test successful git command execution."""
        mock_run.return_value = Mock(stdout='output', returncode=0)
        analyzer = GitAnalyzer(verbose=False)
        result = analyzer.run_git_command(['status'])
        self.assertEqual(result, 'output')
        mock_run.assert_called_once()

    @patch('git_analyzer.subprocess.run')
    def test_run_git_command_failure(self, mock_run):
        """Test git command failure handling."""
        mock_run.side_effect = Exception('Git error')
        analyzer = GitAnalyzer(verbose=False)
        with self.assertRaises(Exception):
            analyzer.run_git_command(['invalid'])


class TestCommitSizeParsing(unittest.TestCase):
    """Test parsing of git diff-tree output."""

    @patch.object(GitAnalyzer, 'run_git_command')
    def test_single_file_change(self, mock_git):
        """Test parsing single file change."""
        mock_git.return_value = "100\t50\tsrc/test.py\n"
        analyzer = GitAnalyzer(verbose=False)
        added, deleted, files = analyzer.get_commit_size('abc123')
        self.assertEqual(added, 100)
        self.assertEqual(deleted, 50)
        self.assertEqual(files, 1)

    @patch.object(GitAnalyzer, 'run_git_command')
    def test_multiple_file_changes(self, mock_git):
        """Test parsing multiple file changes."""
        mock_git.return_value = (
            "100\t50\tsrc/test1.py\n"
            "200\t75\tsrc/test2.py\n"
            "50\t25\tdocs/README.md\n"
        )
        analyzer = GitAnalyzer(verbose=False)
        added, deleted, files = analyzer.get_commit_size('abc123')
        self.assertEqual(added, 350)
        self.assertEqual(deleted, 150)
        self.assertEqual(files, 3)

    @patch.object(GitAnalyzer, 'run_git_command')
    def test_binary_file_change(self, mock_git):
        """Test parsing with binary files (marked as '-')."""
        mock_git.return_value = (
            "100\t50\tsrc/test.py\n"
            "-\t-\timage.png\n"
        )
        analyzer = GitAnalyzer(verbose=False)
        added, deleted, files = analyzer.get_commit_size('abc123')
        self.assertEqual(added, 100)
        self.assertEqual(deleted, 50)
        self.assertEqual(files, 2)  # Binary file still counts

    @patch.object(GitAnalyzer, 'run_git_command')
    def test_empty_commit(self, mock_git):
        """Test parsing empty commit (no changes)."""
        mock_git.return_value = ""
        analyzer = GitAnalyzer(verbose=False)
        added, deleted, files = analyzer.get_commit_size('abc123')
        self.assertEqual(added, 0)
        self.assertEqual(deleted, 0)
        self.assertEqual(files, 0)


class TestIntegration(unittest.TestCase):
    """Integration tests (require mocking full git workflow)."""

    @patch.object(GitAnalyzer, 'run_git_command')
    def test_analyze_workflow(self, mock_git):
        """Test complete analyze workflow with mocked git."""
        # Mock git log output
        def mock_command(args):
            if args[0] == 'log':
                return (
                    "abc123|||2026-01-18T10:00:00|||Enhancement 48: Title|||Body text\n\n"
                    "def456|||2026-01-17T09:00:00|||Fix typo|||No enhancement\n\n"
                )
            elif args[0] == 'diff-tree':
                if 'abc123' in args:
                    return "100\t50\ttest.py\n"
                else:
                    return "10\t5\tREADME.md\n"
            return ""

        mock_git.side_effect = mock_command

        analyzer = GitAnalyzer(verbose=False)
        result = analyzer.analyze()

        # Should find Enhancement 48
        self.assertIn(48, result)
        self.assertEqual(len(result[48]), 1)
        self.assertEqual(result[48][0].sha, 'abc123')
        self.assertEqual(result[48][0].lines_added, 100)
        self.assertEqual(result[48][0].lines_deleted, 50)
        self.assertEqual(result[48][0].files_modified, 1)


def run_tests():
    """Run all tests."""
    unittest.main(argv=[''], exit=False, verbosity=2)


if __name__ == '__main__':
    unittest.main()
