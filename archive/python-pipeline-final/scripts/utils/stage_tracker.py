#!/usr/bin/env python3
"""
Stage tracking utility for the redistricting pipeline.

Tracks which pipeline stages have completed successfully using marker files.
Enables error recovery by skipping completed stages on retry.

Marker files are created as: outputs/{version}/{year}/.stage_{stage_name}

Usage:
    tracker = StageTracker(output_dir=Path('outputs/v1/2020'))

    # Check if stage completed
    if tracker.is_stage_complete('national_post_processing'):
        print("Skipping already-completed stage")
        return

    # Mark stage as complete
    tracker.mark_stage_complete('national_post_processing')

    # Clear all markers (for --reset flag)
    tracker.clear_all()
"""

from pathlib import Path
from datetime import datetime
from typing import List, Optional


class StageTracker:
    """Tracks pipeline stage completion using marker files."""

    def __init__(self, output_dir: Path):
        """
        Initialize stage tracker.

        Args:
            output_dir: Output directory for this year (e.g., outputs/v1/2020)
        """
        self.output_dir = Path(output_dir)

        # Ensure directory exists
        self.output_dir.mkdir(parents=True, exist_ok=True)

    def _get_marker_path(self, stage_name: str) -> Path:
        """
        Get path to marker file for a stage.

        Args:
            stage_name: Name of the stage (e.g., 'national_post_processing')

        Returns:
            Path to marker file
        """
        return self.output_dir / f'.stage_{stage_name}'

    def is_stage_complete(self, stage_name: str) -> bool:
        """
        Check if a stage has completed successfully.

        Args:
            stage_name: Name of the stage

        Returns:
            True if stage is complete, False otherwise
        """
        marker_path = self._get_marker_path(stage_name)
        return marker_path.exists()

    def mark_stage_complete(self, stage_name: str, metadata: Optional[dict] = None):
        """
        Mark a stage as complete.

        Creates a marker file with timestamp and optional metadata.

        Args:
            stage_name: Name of the stage
            metadata: Optional dict with metadata to write to marker file
        """
        marker_path = self._get_marker_path(stage_name)

        # Write marker file with timestamp
        timestamp = datetime.now().strftime('%Y-%m-%d %H:%M:%S')
        content = f"Stage: {stage_name}\nCompleted: {timestamp}\n"

        if metadata:
            content += "\nMetadata:\n"
            for key, value in metadata.items():
                content += f"  {key}: {value}\n"

        marker_path.write_text(content, encoding='utf-8')

    def get_completed_stages(self) -> List[str]:
        """
        Get list of all completed stages.

        Returns:
            List of stage names that have completed
        """
        marker_files = self.output_dir.glob('.stage_*')
        return [f.name.replace('.stage_', '') for f in marker_files]

    def clear_stage(self, stage_name: str):
        """
        Clear completion marker for a specific stage.

        Args:
            stage_name: Name of the stage
        """
        marker_path = self._get_marker_path(stage_name)
        if marker_path.exists():
            marker_path.unlink()

    def clear_all(self):
        """Clear all stage completion markers."""
        marker_files = self.output_dir.glob('.stage_*')
        for marker_file in marker_files:
            marker_file.unlink()

    def get_stage_metadata(self, stage_name: str) -> Optional[dict]:
        """
        Get metadata for a completed stage.

        Args:
            stage_name: Name of the stage

        Returns:
            Dict with metadata if stage is complete, None otherwise
        """
        marker_path = self._get_marker_path(stage_name)
        if not marker_path.exists():
            return None

        try:
            content = marker_path.read_text(encoding='utf-8')
            metadata = {}

            # Parse simple key: value format
            in_metadata_section = False
            for line in content.split('\n'):
                line = line.strip()
                if not line:
                    continue

                if line == 'Metadata:':
                    in_metadata_section = True
                    continue

                if in_metadata_section and ':' in line:
                    key, value = line.split(':', 1)
                    metadata[key.strip()] = value.strip()
                elif ':' in line and not in_metadata_section:
                    # Parse top-level metadata (Stage, Completed)
                    key, value = line.split(':', 1)
                    metadata[key.strip()] = value.strip()

            return metadata
        except Exception:
            return None


def get_stage_tracker(output_dir: Path) -> StageTracker:
    """
    Factory function to create a StageTracker instance.

    Args:
        output_dir: Output directory for this year

    Returns:
        StageTracker instance
    """
    return StageTracker(output_dir)
