"""
Fix Phase Numbers in Enhancement Titles

Updates enhancement titles to match the phase numbers assigned in wave documents.
"""

from pathlib import Path
import re

ENHANCEMENTS_DIR = Path('../../context/enhancements')

# Fixes to apply based on validation results
FIXES = {
    # Wave 10
    23: {'old': 'Phase 2-4', 'new': 'Phase 2'},
    24: {'old': 'Phase 5A', 'new': 'Phase 3'},
    25: {'old': 'Phase 5B-5D', 'new': 'Phase 4'},

    # Wave 12
    30: {'old': 'Phase 0-1', 'new': 'Phase 1'},
    31: {'old': 'Phase 2-4', 'new': 'Phase 2'},
    32: {'old': 'Phase 5', 'new': 'Phase 3'},
    33: {'old': 'Phase 6-8', 'new': 'Phase 4'},
    34: {'old': 'Phase 9-11', 'new': 'Phase 5'},
}

def fix_phase_titles():
    """Fix phase numbers in enhancement titles"""

    print("=" * 80)
    print("FIXING PHASE TITLES IN ENHANCEMENTS")
    print("=" * 80)
    print()

    fixed_count = 0
    error_count = 0

    for enh_id, fix in FIXES.items():
        # Find the enhancement file
        enh_files = list(ENHANCEMENTS_DIR.glob(f'{enh_id}_*.md'))

        if not enh_files:
            print(f"[ERROR] Enhancement {enh_id}: File not found")
            error_count += 1
            continue

        enh_file = enh_files[0]

        # Read content
        content = enh_file.read_text(encoding='utf-8')

        # Check if the old phase is in the title
        old_phase = fix['old']
        new_phase = fix['new']

        # Try to find and replace in the title (first # line)
        lines = content.split('\n')
        title_line = None
        title_idx = None

        for idx, line in enumerate(lines):
            if line.startswith('# Enhancement'):
                title_line = line
                title_idx = idx
                break

        if not title_line:
            print(f"[ERROR] Enhancement {enh_id}: Title line not found")
            error_count += 1
            continue

        # Try to find the old phase (try both singular and plural)
        old_phase_to_replace = None
        if old_phase in title_line:
            old_phase_to_replace = old_phase
        else:
            # Try plural version (e.g., "Phases 2-4" instead of "Phase 2-4")
            old_phase_plural = old_phase.replace('Phase ', 'Phases ', 1)
            if old_phase_plural in title_line:
                old_phase_to_replace = old_phase_plural

        if not old_phase_to_replace:
            print(f"[WARN] Enhancement {enh_id}: '{old_phase}' not found in title")
            print(f"       Title: {title_line}")
            error_count += 1
            continue

        # Replace the phase number (keep plural/singular format from original)
        new_phase_to_use = new_phase
        if 'Phases' in old_phase_to_replace:
            # Keep plural format in replacement
            new_phase_to_use = new_phase.replace('Phase ', 'Phases ', 1)

        new_title = title_line.replace(old_phase_to_replace, new_phase_to_use)
        lines[title_idx] = new_title

        # Write back
        new_content = '\n'.join(lines)
        enh_file.write_text(new_content, encoding='utf-8')

        print(f"[OK] Enhancement {enh_id}: {old_phase} -> {new_phase}")
        print(f"     File: {enh_file.name}")
        fixed_count += 1

    print()
    print("=" * 80)
    print("SUMMARY")
    print("=" * 80)
    print()
    print(f"Fixed: {fixed_count} enhancements")
    print(f"Errors: {error_count} enhancements")
    print()

    if fixed_count > 0:
        print("Run validate_phases.py again to verify all fixes.")

    return error_count == 0

if __name__ == '__main__':
    success = fix_phase_titles()
    exit(0 if success else 1)
