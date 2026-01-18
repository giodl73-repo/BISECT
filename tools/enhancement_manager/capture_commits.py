#!/usr/bin/env python3
"""
Capture Commits for Enhancement

Automatically captures recent git commits for a specific enhancement
and updates the enhancement file with commit metadata.

Usage:
    python capture_commits.py <enhancement_id> [--dry-run] [--verbose]
    python capture_commits.py 48
    python capture_commits.py 48 --commit abc123def456  # Add specific commit
"""

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Dict, List, Optional

# Import git_analyzer and update_enhancements
from git_analyzer import GitAnalyzer


class CommitCapture:
    """Captures and updates commits for a specific enhancement."""

    def __init__(self, enhancement_id: int, verbose: bool = False):
        """
        Initialize commit capture.

        Args:
            enhancement_id: Enhancement ID to capture commits for
            verbose: Enable verbose output
        """
        self.enhancement_id = enhancement_id
        self.verbose = verbose
        self.analyzer = GitAnalyzer(verbose=verbose)

        # Paths
        self.base_path = Path(__file__).parent.parent.parent / 'context' / 'enhancements'
        self.enhancement_file = self.find_enhancement_file()

    def log(self, message: str) -> None:
        """Print message if verbose mode enabled."""
        if self.verbose:
            print(f"[capture_commits] {message}", file=sys.stderr)

    def find_enhancement_file(self) -> Optional[Path]:
        """
        Find the enhancement file by ID.

        Returns:
            Path to enhancement file, or None if not found
        """
        # Search in main enhancements directory
        pattern = f'{self.enhancement_id:02d}_*.md'
        files = list(self.base_path.glob(pattern))
        if files:
            return files[0]

        # Search in active directory
        active_dir = self.base_path / 'active'
        if active_dir.exists():
            files = list(active_dir.glob(pattern))
            if files:
                return files[0]

        return None

    def get_current_commits(self) -> List[str]:
        """
        Get commits already recorded in the enhancement file.

        Returns:
            List of commit SHAs
        """
        if not self.enhancement_file or not self.enhancement_file.exists():
            return []

        content = self.enhancement_file.read_text(encoding='utf-8')

        # Look for **Commits**: line
        match = re.search(r'\*\*Commits\*\*:\s*(.+)', content)
        if not match:
            return []

        commits_text = match.group(1).strip()

        # Extract SHAs from markdown links: [sha](url)
        commit_links = re.findall(r'\[([a-f0-9]+)\]', commits_text)
        return commit_links

    def get_new_commits(self, specific_sha: Optional[str] = None) -> Dict:
        """
        Get new commits for this enhancement not yet recorded.

        Args:
            specific_sha: If provided, only check for this specific commit

        Returns:
            Dict with commit data from enhancement_commits.json format
        """
        # Get all commits for this enhancement from git
        enhancement_commits = self.analyzer.analyze()
        commits_data = enhancement_commits.get(self.enhancement_id, [])

        if not commits_data:
            self.log(f"No commits found in git history for Enhancement {self.enhancement_id}")
            return {'commits': [], 'total_lines_changed': 0, 'total_files_modified': 0}

        # Get currently recorded commits
        current_shas = self.get_current_commits()
        self.log(f"Current commits in file: {current_shas}")

        # Filter to only new commits
        if specific_sha:
            # Check if specific SHA matches any commit
            new_commits = [c for c in commits_data if c.sha.startswith(specific_sha)]
            if not new_commits:
                self.log(f"Specific SHA {specific_sha} not found for this enhancement")
                return {'commits': [], 'total_lines_changed': 0, 'total_files_modified': 0}
        else:
            # Get all commits not already recorded
            new_commits = [c for c in commits_data if c.sha[:7] not in current_shas]

        if not new_commits:
            self.log("No new commits to add")
            return {'commits': [], 'total_lines_changed': 0, 'total_files_modified': 0}

        # Calculate metrics
        metrics = self.analyzer.aggregate_metrics(new_commits)

        return {
            'commits': [
                {
                    'sha': c.sha,
                    'short_sha': c.sha[:7],
                    'message': c.message,
                    'date': c.date,
                    'lines_added': c.lines_added,
                    'lines_deleted': c.lines_deleted,
                    'files_modified': c.files_modified,
                    'github_url': self.analyzer.generate_github_url(c.sha)
                }
                for c in new_commits
            ],
            'total_lines_changed': metrics['total_lines'],
            'total_lines_added': metrics['lines_added'],
            'total_lines_deleted': metrics['lines_deleted'],
            'total_files_modified': metrics['total_files']
        }

    def update_enhancement_file(self, new_commits_data: Dict, dry_run: bool = False) -> bool:
        """
        Update enhancement file with new commits.

        Args:
            new_commits_data: Dict with new commits and metrics
            dry_run: If True, don't write changes

        Returns:
            True if updated, False otherwise
        """
        if not self.enhancement_file or not self.enhancement_file.exists():
            print(f"Error: Enhancement file not found for Enhancement {self.enhancement_id}", file=sys.stderr)
            return False

        new_commits = new_commits_data['commits']
        if not new_commits:
            print(f"No new commits to add for Enhancement {self.enhancement_id}")
            return False

        # Read current file
        content = self.enhancement_file.read_text(encoding='utf-8')

        # Get current commits and size
        current_shas = self.get_current_commits()
        current_size_match = re.search(r'\*\*Size\*\*:\s*(.+)', content)

        # Build new commit links
        all_commit_links = []

        # Add existing commits first
        for sha in current_shas:
            url = f"https://github.com/giodl_microsoft/redistricting/commit/{sha}"
            all_commit_links.append(f"[{sha}]({url})")

        # Add new commits
        for commit in new_commits:
            all_commit_links.append(f"[{commit['short_sha']}]({commit['github_url']})")

        new_commits_line = ', '.join(all_commit_links)

        # Calculate new total size (add to existing)
        current_total_lines = 0
        current_total_files = 0

        if current_size_match:
            size_text = current_size_match.group(1)
            lines_match = re.search(r'([\d,]+)\s+lines?\s+changed', size_text)
            files_match = re.search(r'\((\d+)\s+files?\)', size_text)
            if lines_match:
                current_total_lines = int(lines_match.group(1).replace(',', ''))
            if files_match:
                current_total_files = int(files_match.group(1))

        new_total_lines = current_total_lines + new_commits_data['total_lines_changed']
        new_total_files = current_total_files + new_commits_data['total_files_modified']

        # Determine size category
        if new_total_lines < 100:
            category = "XS"
        elif new_total_lines < 500:
            category = "S"
        elif new_total_lines < 1500:
            category = "M"
        elif new_total_lines < 5000:
            category = "L"
        else:
            category = "XL"

        new_size_line = f"{category} - {new_total_lines:,} lines changed ({new_total_files} files)"

        # Update content
        content = re.sub(
            r'\*\*Commits\*\*:\s*(.+)',
            f'**Commits**: {new_commits_line}',
            content
        )
        content = re.sub(
            r'\*\*Size\*\*:\s*(.+)',
            f'**Size**: {new_size_line}',
            content
        )

        # Write or preview
        if dry_run:
            print(f"\n[DRY RUN] Would update {self.enhancement_file.name}:")
            print(f"  New commits: {len(new_commits)}")
            for commit in new_commits:
                print(f"    - {commit['short_sha']}: {commit['message']}")
            print(f"  New size: {new_size_line}")
            print(f"\nNew Commits line:")
            print(f"  **Commits**: {new_commits_line}")
        else:
            self.enhancement_file.write_text(content, encoding='utf-8', newline='\n')
            print(f"Updated {self.enhancement_file.name}")
            print(f"  Added {len(new_commits)} new commit(s)")
            print(f"  New size: {new_size_line}")

        return True

    def capture(self, specific_sha: Optional[str] = None, dry_run: bool = False) -> bool:
        """
        Capture and update commits for this enhancement.

        Args:
            specific_sha: Optional specific commit SHA to add
            dry_run: If True, preview changes without writing

        Returns:
            True if successful, False otherwise
        """
        if not self.enhancement_file:
            print(f"Error: Enhancement {self.enhancement_id} not found", file=sys.stderr)
            return False

        self.log(f"Enhancement file: {self.enhancement_file}")

        # Get new commits
        new_commits_data = self.get_new_commits(specific_sha)

        if not new_commits_data['commits']:
            print(f"No new commits to add for Enhancement {self.enhancement_id}")
            return True

        # Update file
        return self.update_enhancement_file(new_commits_data, dry_run)


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description='Capture git commits for an enhancement and update the enhancement file'
    )
    parser.add_argument(
        'enhancement_id',
        type=int,
        help='Enhancement ID (e.g., 48)'
    )
    parser.add_argument(
        '--commit',
        type=str,
        help='Specific commit SHA to add (can be short SHA like abc123)'
    )
    parser.add_argument(
        '-n', '--dry-run',
        action='store_true',
        help='Preview changes without writing files'
    )
    parser.add_argument(
        '-v', '--verbose',
        action='store_true',
        help='Enable verbose output'
    )

    args = parser.parse_args()

    try:
        capturer = CommitCapture(args.enhancement_id, verbose=args.verbose)
        success = capturer.capture(specific_sha=args.commit, dry_run=args.dry_run)
        return 0 if success else 1

    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        if args.verbose:
            import traceback
            traceback.print_exc()
        return 1


if __name__ == '__main__':
    sys.exit(main())
