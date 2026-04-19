#!/usr/bin/env python3
"""
Update scripts to use states/ subdirectory structure.

Changes: us_dir / state_name -> us_dir / 'states' / state_name
"""

from pathlib import Path
import re

# Files to update (excluding run_all_states.py which was already updated)
FILES_TO_UPDATE = [
    'scripts/create_us_national_map.py',
    'scripts/create_single_district_states.py',
    'scripts/create_rounds_hierarchy.py',
    'scripts/fill_missing_cities.py',
]

def update_file(file_path):
    """Update a single file to use states/ subdirectory."""
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()

    # Pattern: us_dir / state_name... (but not us_dir / 'states' already)
    # Replace with: us_dir / 'states' / state_name...
    pattern = r"(us_dir\s*/\s*)(state_name\.lower\(\)\.replace\()"
    replacement = r"\1'states' / \2"

    new_content = re.sub(pattern, replacement, content)

    if new_content != content:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(new_content)
        return True
    return False

def main():
    """Update all files."""
    print("Updating scripts to use states/ subdirectory...")

    updated = 0
    for file_path in FILES_TO_UPDATE:
        if Path(file_path).exists():
            if update_file(file_path):
                print(f"  [OK] Updated: {file_path}")
                updated += 1
            else:
                print(f"  [SKIP] No changes needed: {file_path}")
        else:
            print(f"  [NOT FOUND] {file_path}")

    print(f"\nUpdated {updated} files")
    print("\nFuture runs will use structure:")
    print("  outputs/us_2020_redistricting/")
    print("    states/")
    print("      california/")
    print("      texas/")
    print("      ...")
    print("    us_all_districts.csv")
    print("    US_National_Map_435_Districts.png")
    print("    ...")

if __name__ == '__main__':
    main()
