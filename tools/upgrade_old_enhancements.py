"""
Upgrade old enhancement files to new format with frontmatter metadata
"""
from pathlib import Path
import re

def upgrade_enhancement(file_path):
    """Upgrade an enhancement file to new format"""
    content = file_path.read_text(encoding='utf-8')

    # Check if already has new format (starts with # Enhancement)
    if content.startswith('# Enhancement'):
        print(f"[SKIP] {file_path.name} - already in new format")
        return False

    # Extract enhancement ID from filename
    match = re.match(r'(\d+)_', file_path.name)
    if not match:
        print(f"[SKIP] {file_path.name} - no ID in filename")
        return False

    enhancement_id = int(match.group(1))

    # Extract title from first ## heading
    title_match = re.search(r'^##\s+Enhancement \d+:\s*(.+?)(?:\s*[✅🔄📋]\s*(?:COMPLETED|IN PROGRESS|PLANNED))?\s*$', content, re.MULTILINE)
    if not title_match:
        title_match = re.search(r'^##\s+(.+?)(?:\s*[✅🔄📋]\s*(?:COMPLETED|IN PROGRESS|PLANNED))?\s*$', content, re.MULTILINE)

    if not title_match:
        print(f"[WARN] {file_path.name} - could not extract title")
        return False

    title = title_match.group(1).strip()

    # Determine status from directory and title
    if 'completed' in str(file_path):
        status = '✅ COMPLETED'
        completed_date = 'January 2026'  # Placeholder
    else:
        status = '📋 PLANNED'
        completed_date = None

    # Build new header
    new_header = f"# Enhancement {enhancement_id}: {title}\n\n"
    new_header += f"**Status**: {status}\n"
    new_header += f"**Priority**: Medium\n"
    new_header += f"**Estimated Complexity**: Medium\n"
    new_header += f"**Created**: January 2026\n"
    if completed_date:
        new_header += f"**Completed**: {completed_date}\n"
    new_header += "\n"

    # Remove old ## title line and replace with new header
    content = re.sub(
        r'^##\s+Enhancement \d+:.*$\n\n',
        '',
        content,
        count=1,
        flags=re.MULTILINE
    )

    # Prepend new header
    new_content = new_header + content

    # Write back
    file_path.write_text(new_content, encoding='utf-8')
    print(f"[OK] {file_path.name} - upgraded to new format")
    return True

def main():
    base_path = Path(__file__).parent.parent / 'docs' / 'enhancements'

    upgraded = 0
    skipped = 0

    # Process completed directory
    completed_path = base_path / 'completed'
    print(f"Processing {completed_path}...")
    for file_path in sorted(completed_path.glob('*.md')):
        if upgrade_enhancement(file_path):
            upgraded += 1
        else:
            skipped += 1

    # Process active directory
    active_path = base_path / 'active'
    print(f"\nProcessing {active_path}...")
    for file_path in sorted(active_path.glob('*.md')):
        if upgrade_enhancement(file_path):
            upgraded += 1
        else:
            skipped += 1

    print(f"\n[SUMMARY] Upgraded: {upgraded}, Skipped: {skipped}")

if __name__ == '__main__':
    main()
