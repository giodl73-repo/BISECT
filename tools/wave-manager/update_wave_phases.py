"""
Update Wave Files with Explicit Phase Mappings

Converts plain text "Enhancements: X, Y, Z" to proper schema:
**Enhancements**: X, Y, Z
**Phases**:
- Phase 1: Enhancement X
- Phase 2: Enhancement Y
- Phase 3: Enhancement Z
"""

from pathlib import Path
import re

WAVES_DIR = Path('../../context/waves')

def update_wave_file(wave_path):
    """Update a wave file with proper phase mappings"""

    print(f"\nProcessing: {wave_path.name}")
    content = wave_path.read_text(encoding='utf-8')

    # Find the Enhancements line (plain text format)
    match = re.search(r'^Enhancements:\s*([\d,\s]+)$', content, re.MULTILINE)
    if not match:
        print(f"  [SKIP] No Enhancements line found")
        return False

    # Extract enhancement IDs
    enh_line = match.group(0)
    enh_nums_str = match.group(1)
    enh_ids = [int(n.strip()) for n in enh_nums_str.split(',') if n.strip().isdigit()]

    if not enh_ids:
        print(f"  [SKIP] No enhancement IDs found")
        return False

    print(f"  Found {len(enh_ids)} enhancements: {enh_ids}")

    # Build replacement text
    # Format:
    # **Enhancements**: X, Y, Z
    # **Phases**:
    # - Phase 1: Enhancement X
    # - Phase 2: Enhancement Y

    replacement_lines = [
        f"**Enhancements**: {', '.join(map(str, enh_ids))}",
        "**Phases**:"
    ]

    for idx, enh_id in enumerate(enh_ids, 1):
        replacement_lines.append(f"- Phase {idx}: Enhancement {enh_id}")

    replacement = '\n'.join(replacement_lines)

    # Replace the old line with the new format
    updated_content = content.replace(enh_line, replacement)

    # Write back
    wave_path.write_text(updated_content, encoding='utf-8')

    print(f"  [OK] Updated with {len(enh_ids)} phase mappings")
    return True

def main():
    print("=" * 80)
    print("WAVE PHASE MAPPING UPDATER")
    print("=" * 80)

    wave_files = sorted(WAVES_DIR.glob('WAVE*.md'))
    print(f"\nFound {len(wave_files)} wave files")

    updated_count = 0
    for wave_path in wave_files:
        if update_wave_file(wave_path):
            updated_count += 1

    print()
    print("=" * 80)
    print(f"COMPLETE: Updated {updated_count}/{len(wave_files)} wave files")
    print("=" * 80)

if __name__ == '__main__':
    main()
