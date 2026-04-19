#!/usr/bin/env python3
"""
Parse and summarize error logs from redistricting pipeline runs.

Analyzes error.log files and provides:
- Human-readable summary of errors and warnings
- Categorization by error type
- Suggested fixes based on common patterns
- Statistics across multiple years/versions

Usage:
    # Parse logs for a specific run
    python scripts/utils/parse_error_logs.py --version v1 --year 2020

    # Parse logs for all years in a version
    python scripts/utils/parse_error_logs.py --version v1

    # Parse specific log file
    python scripts/utils/parse_error_logs.py --log-file outputs/v1/2020/error.log
"""

import argparse
import re
from pathlib import Path
from typing import List, Dict, Tuple
from collections import Counter, defaultdict


class ErrorLogEntry:
    """Represents a single error or warning from the log."""

    def __init__(self, timestamp: str, level: str, task_name: str,
                 exception_type: str = None, message: str = None,
                 traceback: str = None, context: Dict = None):
        self.timestamp = timestamp
        self.level = level  # 'ERROR' or 'WARNING'
        self.task_name = task_name
        self.exception_type = exception_type
        self.message = message
        self.traceback = traceback
        self.context = context or {}

    def __repr__(self):
        return f"<ErrorLogEntry {self.level} {self.task_name} {self.exception_type}>"


class ErrorLogParser:
    """Parses error log files and provides analysis."""

    def __init__(self, log_file: Path):
        self.log_file = log_file
        self.entries: List[ErrorLogEntry] = []
        self.error_count = 0
        self.warning_count = 0

    def parse(self):
        """Parse the error log file."""
        if not self.log_file.exists():
            return

        content = self.log_file.read_text(encoding='utf-8', errors='replace')

        # Parse errors
        error_pattern = r'\[([^\]]+)\] ERROR: ([^\n]+)\nException: ([^\n]+)\nMessage: ([^\n]+)'
        for match in re.finditer(error_pattern, content):
            timestamp = match.group(1)
            task_name = match.group(2)
            exception_type = match.group(3)
            message = match.group(4)

            # Extract traceback (lines between "Traceback:" and next section)
            start_pos = match.end()
            traceback_match = re.search(r'Traceback:\n(.*?)\n\n', content[start_pos:], re.DOTALL)
            traceback = traceback_match.group(1) if traceback_match else None

            self.entries.append(ErrorLogEntry(
                timestamp, 'ERROR', task_name,
                exception_type, message, traceback
            ))
            self.error_count += 1

        # Parse warnings
        warning_pattern = r'\[([^\]]+)\] WARNING: ([^\n]+)'
        for match in re.finditer(warning_pattern, content):
            timestamp = match.group(1)
            message = match.group(2)

            self.entries.append(ErrorLogEntry(
                timestamp, 'WARNING', 'general',
                message=message
            ))
            self.warning_count += 1

    def categorize_errors(self) -> Dict[str, List[ErrorLogEntry]]:
        """Categorize errors by type."""
        categories = defaultdict(list)

        for entry in self.entries:
            if entry.level != 'ERROR':
                continue

            # Categorize by exception type
            if entry.exception_type:
                if 'FileNotFoundError' in entry.exception_type or 'No such file' in entry.message:
                    categories['Missing Data'].append(entry)
                elif 'MemoryError' in entry.exception_type or 'memory' in entry.message.lower():
                    categories['Memory Issues'].append(entry)
                elif 'TimeoutExpired' in entry.exception_type or 'timeout' in entry.message.lower():
                    categories['Timeouts'].append(entry)
                elif 'METIS' in entry.message or 'graph' in entry.message.lower():
                    categories['METIS/Graph Issues'].append(entry)
                elif 'IndexError' in entry.exception_type or 'KeyError' in entry.exception_type:
                    categories['Data Structure Issues'].append(entry)
                elif 'UnicodeEncodeError' in entry.exception_type:
                    categories['Unicode/Encoding Issues'].append(entry)
                else:
                    categories['Other Errors'].append(entry)
            else:
                categories['Other Errors'].append(entry)

        return dict(categories)

    def get_suggestions(self, categories: Dict[str, List[ErrorLogEntry]]) -> Dict[str, List[str]]:
        """Generate fix suggestions based on error categories."""
        suggestions = {}

        if 'Missing Data' in categories:
            suggestions['Missing Data'] = [
                "Check if required data files exist in data/ directory",
                "Run census download scripts: python scripts/data/census/download_all_states_tracts.py",
                "Build adjacency graphs: python scripts/data/geography/build_all_adjacency_graphs.py",
                "Verify year-specific paths: data/tracts/{year}/, data/adjacency/{year}/"
            ]

        if 'Memory Issues' in categories:
            suggestions['Memory Issues'] = [
                "Close other applications to free memory",
                "Process states individually with --states parameter",
                "Reduce map DPI: --dpi 100 instead of 300",
                "Use fewer parallel workers: --workers 2"
            ]

        if 'METIS/Graph Issues' in categories:
            suggestions['METIS/Graph Issues'] = [
                "Verify METIS is installed: gpmetis -help",
                "Check graph connectivity: python scripts/data/geography/check_graph_connectivity.py",
                "Rebuild adjacency graphs with --force flag",
                "Check for isolated tracts or island issues"
            ]

        if 'Data Structure Issues' in categories:
            suggestions['Data Structure Issues'] = [
                "Check for incomplete round data files",
                "Verify district assignments exist for all states",
                "Check for missing intermediate files in states/{state}/intermediate/",
                "Ensure .states_complete marker exists before national processing"
            ]

        if 'Unicode/Encoding Issues' in categories:
            suggestions['Unicode/Encoding Issues'] = [
                "Replace Unicode characters with ASCII equivalents",
                "Check for ✓ → [OK], ✗ → [FAIL], → → ->",
                "Verify all print statements use ASCII only on Windows",
                "Set PYTHONIOENCODING=utf-8 in environment"
            ]

        if 'Timeouts' in categories:
            suggestions['Timeouts'] = [
                "Increase timeout limits in subprocess calls",
                "Check for hanging METIS processes",
                "Verify disk space is sufficient",
                "Monitor system resources during run"
            ]

        return suggestions

    def print_summary(self):
        """Print human-readable summary of errors and warnings."""
        print("="*74)
        print(f"ERROR LOG SUMMARY: {self.log_file}")
        print("="*74)
        print(f"Total Errors: {self.error_count}")
        print(f"Total Warnings: {self.warning_count}")
        print()

        if self.error_count == 0 and self.warning_count == 0:
            print("[OK] No errors or warnings found in log file.")
            print("="*74)
            return

        # Categorize and display errors
        if self.error_count > 0:
            print("-" * 74)
            print("ERRORS BY CATEGORY:")
            print("-" * 74)

            categories = self.categorize_errors()
            suggestions = self.get_suggestions(categories)

            for category, entries in sorted(categories.items(), key=lambda x: len(x[1]), reverse=True):
                print(f"\n[{category}] ({len(entries)} error(s)):")
                for entry in entries[:3]:  # Show up to 3 examples per category
                    print(f"  - {entry.task_name}: {entry.exception_type}")
                    if entry.message and len(entry.message) < 80:
                        print(f"    Message: {entry.message}")

                if len(entries) > 3:
                    print(f"  ... and {len(entries) - 3} more")

                # Print suggestions for this category
                if category in suggestions:
                    print(f"\n  Suggested Fixes:")
                    for suggestion in suggestions[category]:
                        print(f"    - {suggestion}")

        # Display warnings summary
        if self.warning_count > 0:
            print()
            print("-" * 74)
            print(f"WARNINGS ({self.warning_count} total):")
            print("-" * 74)

            # Group warnings by message pattern
            warning_messages = [e.message for e in self.entries if e.level == 'WARNING']
            warning_counts = Counter(warning_messages)

            for message, count in warning_counts.most_common(5):
                if count > 1:
                    print(f"  - ({count}x) {message[:70]}")
                else:
                    print(f"  - {message[:70]}")

            if len(warning_counts) > 5:
                print(f"  ... and {len(warning_counts) - 5} more warning types")

        print()
        print("="*74)
        print(f"Full details in: {self.log_file}")
        print("="*74)


def main():
    parser = argparse.ArgumentParser(description='Parse and summarize pipeline error logs')
    parser.add_argument('--version', type=str,
                        help='Version identifier (e.g., v1, test)')
    parser.add_argument('--year', type=str, choices=['2020', '2010', '2000', 'all'],
                        help='Census year (or "all" for all years)')
    parser.add_argument('--log-file', type=Path,
                        help='Specific error.log file to parse')
    parser.add_argument('--output-dir', type=Path, default=Path('outputs'),
                        help='Base output directory (default: outputs/)')

    args = parser.parse_args()

    # Collect log files to parse
    log_files: List[Tuple[Path, str]] = []  # [(log_file, label), ...]

    if args.log_file:
        # Parse specific log file
        if not args.log_file.exists():
            print(f"ERROR: Log file not found: {args.log_file}")
            return 1
        log_files.append((args.log_file, str(args.log_file)))

    elif args.version:
        # Parse logs for version
        version_dir = args.output_dir / args.version

        if not version_dir.exists():
            print(f"ERROR: Version directory not found: {version_dir}")
            return 1

        # Determine which years to parse
        years_to_parse = []
        if args.year == 'all':
            years_to_parse = ['2020', '2010', '2000']
        elif args.year:
            years_to_parse = [args.year]
        else:
            # Auto-detect years
            for year in ['2020', '2010', '2000']:
                year_dir = version_dir / year
                if year_dir.exists():
                    years_to_parse.append(year)

        # Collect log files
        for year in years_to_parse:
            log_file = version_dir / year / 'error.log'
            if log_file.exists():
                log_files.append((log_file, f"{args.version}/{year}"))

        if not log_files:
            print(f"[INFO] No error.log files found in {version_dir}")
            print(f"       (This is normal if no errors occurred)")
            return 0

    else:
        print("ERROR: Must specify either --log-file or --version")
        parser.print_help()
        return 1

    # Parse and summarize each log file
    total_errors = 0
    total_warnings = 0

    for log_file, label in log_files:
        if len(log_files) > 1:
            print(f"\n{'='*74}")
            print(f"Parsing: {label}")
            print(f"{'='*74}\n")

        log_parser = ErrorLogParser(log_file)
        log_parser.parse()
        log_parser.print_summary()

        total_errors += log_parser.error_count
        total_warnings += log_parser.warning_count

    # Overall summary if multiple files
    if len(log_files) > 1:
        print(f"\n{'='*74}")
        print("OVERALL SUMMARY")
        print(f"{'='*74}")
        print(f"Total files parsed: {len(log_files)}")
        print(f"Total errors: {total_errors}")
        print(f"Total warnings: {total_warnings}")
        print(f"{'='*74}")

    return 0


if __name__ == '__main__':
    import sys
    sys.exit(main())
