#!/usr/bin/env python3
"""
Update Enhancement Files with Commit Metadata

Reads enhancement_commits.json and updates all enhancement files
with commit links and size metrics.

Usage:
    python update_enhancements.py [--dry-run] [--verbose]
"""

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Dict, List, Optional, Tuple


class EnhancementUpdater:
    """Updates enhancement files with commit metadata."""

    def __init__(self, enhancements_dir: Path, commits_file: Path, verbose: bool = False):
        """
        Initialize the updater.

        Args:
            enhancements_dir: Directory containing enhancement files
            commits_file: Path to enhancement_commits.json
            verbose: Enable verbose output
        """
        self.enhancements_dir = enhancements_dir
        self.commits_file = commits_file
        self.verbose = verbose

        # Load commit data
        with open(commits_file, 'r', encoding='utf-8') as f:
            self.commit_data = json.load(f)

        self.log(f"Loaded commit data for {len(self.commit_data)} enhancements")

    def log(self, message: str) -> None:
        """Print message if verbose mode enabled."""
        if self.verbose:
            print(f"[update_enhancements] {message}", file=sys.stderr)

    def find_enhancement_files(self) -> List[Tuple[int, Path]]:
        """
        Find all enhancement markdown files.

        Returns:
            List of (enhancement_id, file_path) tuples
        """
        files = []
        pattern = re.compile(r'^(\d+)_')

        for file_path in self.enhancements_dir.glob('*.md'):
            match = pattern.match(file_path.name)
            if match:
                enhancement_id = int(match.group(1))
                files.append((enhancement_id, file_path))

        # Sort by enhancement ID
        files.sort(key=lambda x: x[0])

        self.log(f"Found {len(files)} enhancement files")
        return files

    def format_commit_links(self, commits: List[Dict]) -> str:
        """
        Format commit information as markdown links.

        Args:
            commits: List of commit dicts from JSON

        Returns:
            Markdown string with commit links
        """
        if not commits:
            return "(Not yet implemented)"

        # Format each commit as [short_sha](github_url)
        links = []
        for commit in commits:
            short_sha = commit['short_sha']
            github_url = commit['github_url']
            links.append(f"[{short_sha}]({github_url})")

        return ', '.join(links)

    def format_size_info(self, data: Dict) -> str:
        """
        Format size information.

        Args:
            data: Enhancement data from JSON

        Returns:
            Human-readable size string
        """
        if not data:
            return "(Not yet implemented)"

        total_lines = data['total_lines_changed']
        total_files = data['total_files_modified']

        # Format with commas for thousands
        lines_str = f"{total_lines:,}"
        files_str = f"{total_files}"

        # Determine size category
        if total_lines < 100:
            category = "XS"
        elif total_lines < 500:
            category = "S"
        elif total_lines < 1500:
            category = "M"
        elif total_lines < 5000:
            category = "L"
        else:
            category = "XL"

        return f"{category} - {lines_str} lines changed ({files_str} files)"

    def has_commit_metadata(self, content: str) -> bool:
        """Check if file already has commit metadata."""
        return '**Commits**:' in content or '**Size**:' in content

    def find_insertion_point(self, lines: List[str]) -> Optional[int]:
        """
        Find the line number where commit metadata should be inserted.

        Looks for the last metadata field (Completed, Created, Priority, etc.)
        and inserts after it.

        Args:
            lines: File content as list of lines

        Returns:
            Line number to insert at, or None if not found
        """
        # Look for metadata fields
        metadata_patterns = [
            r'^\*\*Completed\*\*:',
            r'^\*\*Started\*\*:',
            r'^\*\*Created\*\*:',
            r'^\*\*Estimated Complexity\*\*:',
            r'^\*\*Priority\*\*:',
            r'^\*\*Status\*\*:',
        ]

        compiled = [re.compile(p) for p in metadata_patterns]

        # Find the last metadata field
        last_metadata_line = -1
        for i, line in enumerate(lines):
            for pattern in compiled:
                if pattern.match(line):
                    last_metadata_line = i
                    break

        if last_metadata_line >= 0:
            # Insert after the last metadata field
            return last_metadata_line + 1

        return None

    def update_file(self, enhancement_id: int, file_path: Path, dry_run: bool = False) -> bool:
        """
        Update a single enhancement file with commit metadata.

        Args:
            enhancement_id: Enhancement ID
            file_path: Path to enhancement file
            dry_run: If True, don't write changes

        Returns:
            True if file was updated, False otherwise
        """
        # Read file
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
                lines = content.splitlines(keepends=True)
        except Exception as e:
            print(f"Error reading {file_path}: {e}", file=sys.stderr)
            return False

        # Check if already has metadata
        if self.has_commit_metadata(content):
            self.log(f"Enhancement {enhancement_id}: Already has commit metadata, skipping")
            return False

        # Get commit data
        commit_data = self.commit_data.get(str(enhancement_id))

        # Format metadata
        if commit_data:
            commit_links = self.format_commit_links(commit_data['commits'])
            size_info = self.format_size_info(commit_data)
        else:
            commit_links = "(Not yet implemented)"
            size_info = "(Not yet implemented)"

        # Build metadata lines
        metadata_lines = [
            f"**Commits**: {commit_links}\n",
            f"**Size**: {size_info}\n",
        ]

        # Find insertion point
        insertion_line = self.find_insertion_point(lines)
        if insertion_line is None:
            print(f"Warning: Could not find insertion point for {file_path}", file=sys.stderr)
            return False

        # Insert metadata
        new_lines = lines[:insertion_line] + metadata_lines + lines[insertion_line:]
        new_content = ''.join(new_lines)

        # Write file (if not dry run)
        if not dry_run:
            try:
                with open(file_path, 'w', encoding='utf-8', newline='\n') as f:
                    f.write(new_content)
                self.log(f"Enhancement {enhancement_id}: Updated {file_path}")
            except Exception as e:
                print(f"Error writing {file_path}: {e}", file=sys.stderr)
                return False
        else:
            self.log(f"Enhancement {enhancement_id}: Would update {file_path}")
            if self.verbose:
                print(f"\nWould add:")
                for line in metadata_lines:
                    print(f"  {line.rstrip()}")

        return True

    def update_all(self, dry_run: bool = False) -> Tuple[int, int]:
        """
        Update all enhancement files.

        Args:
            dry_run: If True, don't write changes

        Returns:
            Tuple of (updated_count, skipped_count)
        """
        files = self.find_enhancement_files()

        updated = 0
        skipped = 0

        for enhancement_id, file_path in files:
            if self.update_file(enhancement_id, file_path, dry_run):
                updated += 1
            else:
                skipped += 1

        return updated, skipped


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description='Update enhancement files with commit metadata'
    )
    parser.add_argument(
        '--enhancements-dir',
        type=Path,
        default=Path(__file__).parent.parent.parent / 'context' / 'enhancements',
        help='Directory containing enhancement files'
    )
    parser.add_argument(
        '--commits-file',
        type=Path,
        default=Path(__file__).parent / 'enhancement_commits.json',
        help='Path to enhancement_commits.json'
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

    # Validate paths
    if not args.enhancements_dir.exists():
        print(f"Error: Enhancements directory not found: {args.enhancements_dir}", file=sys.stderr)
        return 1

    if not args.commits_file.exists():
        print(f"Error: Commits file not found: {args.commits_file}", file=sys.stderr)
        return 1

    try:
        updater = EnhancementUpdater(
            enhancements_dir=args.enhancements_dir,
            commits_file=args.commits_file,
            verbose=args.verbose
        )

        updated, skipped = updater.update_all(dry_run=args.dry_run)

        print(f"\n{'[DRY RUN] ' if args.dry_run else ''}Summary:")
        print(f"  Updated: {updated}")
        print(f"  Skipped: {skipped}")
        print(f"  Total: {updated + skipped}")

        return 0

    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        if args.verbose:
            import traceback
            traceback.print_exc()
        return 1


if __name__ == '__main__':
    sys.exit(main())
