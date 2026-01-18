#!/usr/bin/env python3
"""
Git History Analyzer for Enhancement Tracking

Analyzes git commit history to match commits with enhancements
and calculate size metrics (lines changed, files modified).

Usage:
    python git_analyzer.py [--output FILE] [--verbose]
"""

import argparse
import json
import re
import subprocess
import sys
from collections import defaultdict
from dataclasses import dataclass, asdict
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Tuple


@dataclass
class CommitInfo:
    """Information about a single commit."""
    sha: str
    message: str
    date: str
    lines_added: int
    lines_deleted: int
    files_modified: int

    @property
    def total_lines(self) -> int:
        """Total lines changed (added + deleted)."""
        return self.lines_added + self.lines_deleted


class GitAnalyzer:
    """Analyzes git history to extract enhancement-related commits."""

    # Regex patterns for matching enhancement references in commit messages
    ENHANCEMENT_PATTERNS = [
        r'Enhancement (\d+):',                    # Enhancement 39: Title
        r'Enhancement (\d+) Phase',               # Enhancement 47 Phase 2: Title
        r'Mark Enhancement (\d+)',                # Mark Enhancement 46 as Completed
        r'Enhancement (\d+)\s+',                  # Enhancement 39 (space, no colon)
        r'Add Enhancements? (\d+)-(\d+)',        # Add Enhancements 42-46 (batch)
        r'Update Enhancement (\d+)',              # Update Enhancement 48 with...
        r'Complete Enhancement (\d+)',            # Complete Enhancement 42
        r'Fix Enhancement (\d+)',                 # Fix Enhancement 35
    ]

    # GitHub repository info
    GITHUB_OWNER = 'giodl_microsoft'
    GITHUB_REPO = 'redistricting'

    def __init__(self, repo_path: Optional[Path] = None, verbose: bool = False):
        """
        Initialize the analyzer.

        Args:
            repo_path: Path to git repository (default: current directory)
            verbose: Enable verbose output
        """
        self.repo_path = repo_path or Path.cwd()
        self.verbose = verbose
        self.compiled_patterns = [re.compile(p, re.IGNORECASE) for p in self.ENHANCEMENT_PATTERNS]

    def log(self, message: str) -> None:
        """Print message if verbose mode enabled."""
        if self.verbose:
            print(f"[git_analyzer] {message}", file=sys.stderr)

    def run_git_command(self, args: List[str]) -> str:
        """Run a git command and return output."""
        try:
            result = subprocess.run(
                ['git'] + args,
                cwd=self.repo_path,
                capture_output=True,
                encoding='utf-8',
                errors='replace',  # Replace invalid characters instead of crashing
                check=True
            )
            return result.stdout
        except subprocess.CalledProcessError as e:
            raise RuntimeError(f"Git command failed: {e.stderr}") from e

    def extract_enhancement_numbers(self, message: str) -> List[int]:
        """
        Extract enhancement numbers from commit message.

        Handles both single enhancements and batch proposals (e.g., "42-46").

        Args:
            message: Commit message (subject + body)

        Returns:
            List of enhancement numbers found
        """
        numbers = []

        for pattern in self.compiled_patterns:
            matches = pattern.findall(message)
            for match in matches:
                if isinstance(match, tuple):
                    # Batch proposal: "Add Enhancements 42-46"
                    if len(match) == 2 and match[0].isdigit() and match[1].isdigit():
                        start, end = int(match[0]), int(match[1])
                        numbers.extend(range(start, end + 1))
                    else:
                        # Single number in tuple (from other patterns)
                        numbers.append(int(match[0]))
                else:
                    # Single number
                    numbers.append(int(match))

        # Remove duplicates and sort
        return sorted(set(numbers))

    def get_commit_size(self, sha: str) -> Tuple[int, int, int]:
        """
        Get size metrics for a commit.

        Args:
            sha: Commit SHA

        Returns:
            Tuple of (lines_added, lines_deleted, files_modified)
        """
        try:
            # Use git diff-tree to get numstat for the commit
            output = self.run_git_command(['diff-tree', '--numstat', '-r', sha])

            lines_added = 0
            lines_deleted = 0
            files_modified = 0

            for line in output.strip().split('\n'):
                if not line:
                    continue

                parts = line.split('\t')
                if len(parts) < 3:
                    continue

                added_str, deleted_str, filename = parts[0], parts[1], parts[2]

                # Skip binary files (marked with '-')
                if added_str == '-' or deleted_str == '-':
                    files_modified += 1
                    continue

                try:
                    lines_added += int(added_str)
                    lines_deleted += int(deleted_str)
                    files_modified += 1
                except ValueError:
                    self.log(f"Warning: Could not parse numstat for {sha}: {line}")

            return lines_added, lines_deleted, files_modified

        except Exception as e:
            self.log(f"Warning: Could not get size for commit {sha}: {e}")
            return 0, 0, 0

    def get_all_commits(self) -> List[Dict[str, str]]:
        """
        Get all commits from git history.

        Returns:
            List of dicts with 'sha', 'subject', 'body', 'date'
        """
        # Format: SHA|ISO8601_DATE|SUBJECT|BODY (separated by ||| to avoid conflicts)
        log_format = '%H|||%aI|||%s|||%b'
        output = self.run_git_command(['log', '--all', f'--format={log_format}'])

        commits = []
        for entry in output.split('\n\n'):
            if not entry.strip():
                continue

            parts = entry.split('|||')
            if len(parts) < 3:
                continue

            sha = parts[0].strip()
            date = parts[1].strip() if len(parts) > 1 else ''
            subject = parts[2].strip() if len(parts) > 2 else ''
            body = parts[3].strip() if len(parts) > 3 else ''

            # Combine subject and body for pattern matching
            full_message = f"{subject}\n{body}".strip()

            commits.append({
                'sha': sha,
                'subject': subject,
                'body': body,
                'message': full_message,
                'date': date
            })

        return commits

    def analyze(self) -> Dict[int, List[CommitInfo]]:
        """
        Analyze git history and build enhancement-to-commits mapping.

        Returns:
            Dict mapping enhancement ID to list of CommitInfo objects
        """
        self.log("Fetching all commits...")
        commits = self.get_all_commits()
        self.log(f"Found {len(commits)} total commits")

        # Build mapping
        enhancement_commits = defaultdict(list)
        matched_commits = 0

        for commit in commits:
            sha = commit['sha']
            message = commit['message']
            date = commit['date']

            # Extract enhancement numbers
            enhancement_nums = self.extract_enhancement_numbers(message)

            if enhancement_nums:
                matched_commits += 1

                # Get size metrics
                lines_added, lines_deleted, files_modified = self.get_commit_size(sha)

                # Create commit info
                commit_info = CommitInfo(
                    sha=sha,
                    message=commit['subject'],  # Use subject only for display
                    date=date,
                    lines_added=lines_added,
                    lines_deleted=lines_deleted,
                    files_modified=files_modified
                )

                # Add to all matched enhancements
                for num in enhancement_nums:
                    enhancement_commits[num].append(commit_info)
                    self.log(f"Matched Enhancement {num}: {sha[:7]} - {commit['subject']}")

        self.log(f"Matched {matched_commits} commits to {len(enhancement_commits)} enhancements")

        # Sort commits by date (newest first) for each enhancement
        for enhancement_id in enhancement_commits:
            enhancement_commits[enhancement_id].sort(key=lambda c: c.date, reverse=True)

        return dict(enhancement_commits)

    def generate_github_url(self, sha: str) -> str:
        """Generate GitHub commit URL."""
        return f"https://github.com/{self.GITHUB_OWNER}/{self.GITHUB_REPO}/commit/{sha}"

    def aggregate_metrics(self, commits: List[CommitInfo]) -> Dict[str, int]:
        """
        Aggregate size metrics across multiple commits.

        Args:
            commits: List of CommitInfo objects

        Returns:
            Dict with 'total_lines', 'total_files', 'lines_added', 'lines_deleted'
        """
        total_added = sum(c.lines_added for c in commits)
        total_deleted = sum(c.lines_deleted for c in commits)
        total_files = sum(c.files_modified for c in commits)

        return {
            'lines_added': total_added,
            'lines_deleted': total_deleted,
            'total_lines': total_added + total_deleted,
            'total_files': total_files
        }

    def export_to_json(self, output_path: Path) -> None:
        """
        Analyze and export results to JSON file.

        Args:
            output_path: Path to output JSON file
        """
        enhancement_commits = self.analyze()

        # Build output structure
        output = {}
        for enhancement_id, commits in enhancement_commits.items():
            metrics = self.aggregate_metrics(commits)

            output[str(enhancement_id)] = {
                'enhancement_id': enhancement_id,
                'commit_count': len(commits),
                'commits': [
                    {
                        'sha': c.sha,
                        'short_sha': c.sha[:7],
                        'message': c.message,
                        'date': c.date,
                        'lines_added': c.lines_added,
                        'lines_deleted': c.lines_deleted,
                        'files_modified': c.files_modified,
                        'github_url': self.generate_github_url(c.sha)
                    }
                    for c in commits
                ],
                'total_lines_changed': metrics['total_lines'],
                'total_lines_added': metrics['lines_added'],
                'total_lines_deleted': metrics['lines_deleted'],
                'total_files_modified': metrics['total_files']
            }

        # Write to file
        output_path.parent.mkdir(parents=True, exist_ok=True)
        with open(output_path, 'w', encoding='utf-8') as f:
            json.dump(output, f, indent=2)

        print(f"Exported {len(output)} enhancements to {output_path}")

        # Print summary statistics
        total_commits = sum(len(commits) for commits in enhancement_commits.values())
        total_lines = sum(
            sum(c.total_lines for c in commits)
            for commits in enhancement_commits.values()
        )
        total_files = sum(
            sum(c.files_modified for c in commits)
            for commits in enhancement_commits.values()
        )

        print(f"\nSummary:")
        print(f"  Enhancements with commits: {len(enhancement_commits)}")
        print(f"  Total commits: {total_commits}")
        print(f"  Total lines changed: {total_lines:,}")
        print(f"  Total files modified: {total_files:,}")


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description='Analyze git history to extract enhancement commits and size metrics'
    )
    parser.add_argument(
        '-o', '--output',
        type=Path,
        default=Path(__file__).parent / 'enhancement_commits.json',
        help='Output JSON file path (default: enhancement_commits.json)'
    )
    parser.add_argument(
        '-v', '--verbose',
        action='store_true',
        help='Enable verbose output'
    )
    parser.add_argument(
        '--repo',
        type=Path,
        help='Path to git repository (default: current directory)'
    )

    args = parser.parse_args()

    try:
        analyzer = GitAnalyzer(repo_path=args.repo, verbose=args.verbose)
        analyzer.export_to_json(args.output)
        return 0
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(main())
