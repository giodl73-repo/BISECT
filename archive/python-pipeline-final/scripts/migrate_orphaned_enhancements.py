#!/usr/bin/env python3
"""
Migrate orphaned enhancements (E1-E9) to Wave 01 pulses.

Converts context/enhancements/01-09_*.md files into Wave 01 pulses with:
- V4 UUIDs based on project slug + wave UUID + enhancement slug
- YAML frontmatter
- Renamed to 01+slug.md format
"""

import hashlib
import re
import subprocess
from pathlib import Path
from datetime import datetime

# Paths
PROJECT_ROOT = Path(__file__).parent.parent
ENHANCEMENTS_DIR = PROJECT_ROOT / "context" / "enhancements"
WAVE01_DIR = PROJECT_ROOT / "context" / "waves" / "01+core-algorithm-foundation"
PULSES_DIR = WAVE01_DIR / "pulses"
META_FILE = WAVE01_DIR / "_meta.yaml"

# Wave 01 metadata
WAVE_UUID = "79fae8"
PROJECT_SLUG = "apportionment"

# Enhancement files to convert
ENHANCEMENTS = [
    ("01_compactness_integration.md", "compactness-integration", "Compactness Integration"),
    ("02_seat_totals.md", "seat-totals", "Seat Totals"),
    ("03_national_maps.md", "national-maps", "National Maps"),
    ("04_metro_areas.md", "metro-areas", "Metro Areas"),
    ("05_national_rounds.md", "national-rounds", "National Rounds"),
    ("06_architecture_diagrams.md", "architecture-diagrams", "Architecture Diagrams"),
    ("07_edge_weighted_bisection.md", "edge-weighted-bisection", "Edge-Weighted Bisection"),
    ("08_block_level_data.md", "block-level-data", "Block-Level Data"),
    ("09_per_state_analysis.md", "per-state-analysis", "Per-State Analysis"),
]


def get_git_timestamp(file_path: Path) -> int:
    """Get creation timestamp from git history."""
    try:
        result = subprocess.run(
            ['git', 'log', '--diff-filter=A', '--format=%at', '--', str(file_path)],
            capture_output=True,
            text=True,
            cwd=PROJECT_ROOT
        )

        if result.returncode != 0 or not result.stdout.strip():
            print(f"  Warning: No git history for {file_path.name}, using current time")
            import time
            return int(time.time())

        timestamps = result.stdout.strip().split('\n')
        return int(timestamps[-1])  # Oldest commit
    except Exception as e:
        print(f"  Error getting git timestamp: {e}")
        import time
        return int(time.time())


def generate_v4_uuid(project_slug: str, wave_uuid: str, enhancement_slug: str) -> str:
    """Generate V4 UUID based on project + wave + enhancement slug."""
    # Create deterministic seed from components
    seed = f"{project_slug}:{wave_uuid}:{enhancement_slug}"
    hash_hex = hashlib.sha256(seed.encode()).hexdigest()

    # Format as UUID v4 (8-4-4-4-12)
    return f"{hash_hex[:8]}-{hash_hex[8:12]}-{hash_hex[12:16]}-{hash_hex[16:20]}-{hash_hex[20:32]}"


def extract_metadata(content: str) -> dict:
    """Extract metadata from enhancement content."""
    metadata = {}

    # Extract title (first # line)
    title_match = re.search(r'^#\s+(.+)$', content, re.MULTILINE)
    if title_match:
        metadata['title'] = title_match.group(1)

    # Extract status
    status_match = re.search(r'\*\*Status\*\*:\s*(.+)', content)
    if status_match:
        status_text = status_match.group(1).strip()
        # Clean status (remove emojis and symbols)
        status_text = re.sub(r'[✅📋⏳]', '', status_text).strip()
        if 'COMPLETED' in status_text:
            metadata['status'] = 'COMPLETED'
        elif 'PLANNED' in status_text:
            metadata['status'] = 'PLANNED'
        else:
            metadata['status'] = 'IN_PROGRESS'

    # Extract created date
    created_match = re.search(r'\*\*Created\*\*:\s*(.+)', content)
    if created_match:
        created_text = created_match.group(1).strip()
        # Try to parse "January 2026" format
        if 'January 2026' in created_text:
            metadata['created'] = '2026-01-10'
        elif 'created' in created_text.lower():
            metadata['created'] = '2026-01-10'

    # Extract completed date if present
    completed_match = re.search(r'\*\*Completed\*\*:\s*(.+)', content)
    if completed_match:
        completed_text = completed_match.group(1).strip()
        if 'January' in completed_text:
            metadata['completed'] = '2026-01-18'

    return metadata


def convert_enhancement(filename: str, slug: str, name: str, pulse_number: int) -> bool:
    """Convert one enhancement to a pulse."""
    source_file = ENHANCEMENTS_DIR / filename
    target_file = PULSES_DIR / f"{pulse_number:02d}+{slug}.md"

    if not source_file.exists():
        print(f"  [WARN] Source not found: {filename}")
        return False

    print(f"\n[E{pulse_number}] Converting: {filename}")
    print(f"      Target: {target_file.name}")

    # Read original content
    content = source_file.read_text(encoding='utf-8')

    # Generate UUID
    uuid = generate_v4_uuid(PROJECT_SLUG, WAVE_UUID, slug)
    print(f"      UUID: {uuid}")

    # Extract metadata
    metadata = extract_metadata(content)
    created = metadata.get('created', '2026-01-10')
    status = metadata.get('status', 'COMPLETED')

    # Create YAML frontmatter
    frontmatter = f"""---
uuid: {uuid}
slug: {slug}
name: {name}
wave_uuid: {WAVE_UUID}
created: '{created}'
status: {status}
---

"""

    # Write to target with frontmatter
    target_file.write_text(frontmatter + content, encoding='utf-8')
    print(f"      [OK] Created: {target_file}")

    return True


def update_meta_yaml():
    """Update _meta.yaml to include all pulses."""
    print("\n[META] Updating _meta.yaml...")

    # Read existing content
    meta_content = META_FILE.read_text(encoding='utf-8')

    # Build pulses list
    pulses_list = "\n".join([
        f"  - {i:02d}+{slug}"
        for i, (_, slug, _) in enumerate(ENHANCEMENTS, 1)
    ])

    # Replace pulses section
    if 'pulses:' in meta_content:
        # Update existing pulses section
        new_content = re.sub(
            r'pulses:\s*\n(?:  - .+\n)*',
            f'pulses:\n{pulses_list}\n',
            meta_content
        )
    else:
        # Add pulses section at end
        new_content = meta_content.rstrip() + f"\npulses:\n{pulses_list}\n"

    META_FILE.write_text(new_content, encoding='utf-8')
    print(f"      [OK] Updated: {META_FILE}")


def main():
    print("=" * 70)
    print("Migrate Orphaned Enhancements to Wave 01 Pulses")
    print("=" * 70)
    print(f"Project: {PROJECT_SLUG}")
    print(f"Wave: {WAVE_UUID} (Core Algorithm Foundation)")
    print(f"Source: {ENHANCEMENTS_DIR}")
    print(f"Target: {PULSES_DIR}")
    print()

    # Ensure pulses directory exists
    PULSES_DIR.mkdir(parents=True, exist_ok=True)

    # Convert each enhancement
    success_count = 0
    for i, (filename, slug, name) in enumerate(ENHANCEMENTS, 1):
        if convert_enhancement(filename, slug, name, i):
            success_count += 1

    # Update _meta.yaml
    if success_count == len(ENHANCEMENTS):
        update_meta_yaml()

    # Summary
    print("\n" + "=" * 70)
    print("CONVERSION COMPLETE")
    print("=" * 70)
    print(f"[OK] Converted: {success_count}/{len(ENHANCEMENTS)} enhancements")
    print(f"[OK] Location: {PULSES_DIR}")
    print()

    if success_count == len(ENHANCEMENTS):
        print("Next steps:")
        print("1. Review the converted pulses in context/waves/01+core-algorithm-foundation/pulses/")
        print("2. Verify _meta.yaml has been updated correctly")
        print("3. Optionally delete original files from context/enhancements/")
        return 0
    else:
        print("[WARN] Some conversions failed. Check warnings above.")
        return 1


if __name__ == '__main__':
    import sys
    sys.exit(main())
