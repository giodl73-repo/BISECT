#!/usr/bin/env python3
"""Update all scripts to use unified directory structure for all census years."""

import re
from pathlib import Path

def update_file(filepath):
    """Update a single file to use unified directory structure."""
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()

    original_content = content
    changes_made = []

    # Pattern 1: if year == '2020' with data/raw, else with data/tracts
    # Match multiline conditional blocks
    pattern1 = r'(\s+)if (?:args\.)?year == [\'"]2020[\'"]:?\s*\n\s+(\w+) = (?:Path\()?[\'"]?data/raw/({\w+}[^\'"]+)[\'"]?(?:\))?\s*\n\s+else:?\s*\n\s+\2 = (?:Path\()?[\'"]?data/tracts/{\w+}/({\w+}[^\'"]+)[\'"]?(?:\))?'

    def replace_pattern1(match):
        indent = match.group(1)
        var_name = match.group(2)
        filename_pattern = match.group(3)
        changes_made.append(f"Unified conditional for {var_name}")
        # Use the pattern from the else clause
        return f'{indent}{var_name} = Path(f\'data/tracts/{{year}}/{filename_pattern}\')'

    content = re.sub(pattern1, replace_pattern1, content, flags=re.MULTILINE)

    # Pattern 2: Simple conditional with data/raw vs data/tracts
    pattern2 = r'(\s+)if (?:args\.)?year == [\'"]2020[\'"]:\s*\n\s+(\w+) = Path\(f[\'"]data/raw/([^\'\"]+)[\'"]\)\s*\n\s+else:\s*\n\s+\2 = Path\(f[\'"]data/tracts/{year}/([^\'\"]+)[\'"]\)'

    def replace_pattern2(match):
        indent = match.group(1)
        var_name = match.group(2)
        new_path = match.group(4)  # Use the pattern from else clause
        changes_made.append(f"Unified conditional for {var_name}")
        return f'{indent}{var_name} = Path(f\'data/tracts/{{year}}/{new_path}\')'

    content = re.sub(pattern2, replace_pattern2, content, flags=re.MULTILINE)

    # Pattern 3: Direct data/raw references (no conditional)
    # Replace data/raw/{state}_tracts_{year} with data/tracts/{year}/{state}_tracts_{year}
    content = re.sub(
        r'([\'"])data/raw/({\w+}_tracts_{\w+}\.parquet)\1',
        r"\1data/tracts/{year}/\2\1",
        content
    )

    # Replace data/raw/{state}_places_{year} with data/tracts/{year}/{state}_places_{year}
    content = re.sub(
        r'([\'"])data/raw/({\w+}_places_{\w+}\.parquet)\1',
        r"\1data/tracts/{year}/\2\1",
        content
    )

    # Pattern 4: Adjacency graph paths
    # data/adjacency/{state}_adjacency_{year}.pkl -> data/adjacency/{year}/{state}_adjacency_{year}.pkl
    pattern4 = r'(\s+)if (?:args\.)?year == [\'"]2020[\'"]:\s*\n\s+(\w+) = Path\(f[\'"]data/adjacency/([^\'\"]+)[\'"]\)\s*\n\s+else:\s*\n\s+\2 = Path\(f[\'"]data/adjacency/{year}/([^\'\"]+)[\'"]\)'

    def replace_pattern4(match):
        indent = match.group(1)
        var_name = match.group(2)
        new_path = match.group(4)
        changes_made.append(f"Unified adjacency conditional for {var_name}")
        return f'{indent}{var_name} = Path(f\'data/adjacency/{{year}}/{new_path}\')'

    content = re.sub(pattern4, replace_pattern4, content, flags=re.MULTILINE)

    # Pattern 5: Input/output dir conditionals
    pattern5 = r'(\s+)if year == [\'"]2020[\'"]:\s*\n\s+input_dir = [\'"]data/raw[\'"]\s*\n\s+output_dir = [\'"]data/adjacency[\'"]\s*\n\s+else:\s*\n\s+input_dir = f[\'"]data/tracts/{year}[\'"]\s*\n\s+output_dir = f[\'"]data/adjacency/{year}[\'"]'

    def replace_pattern5(match):
        indent = match.group(1)
        changes_made.append("Unified input/output dirs")
        return f'{indent}input_dir = f\'data/tracts/{{year}}\'\n{indent}output_dir = f\'data/adjacency/{{year}}\''

    content = re.sub(pattern5, replace_pattern5, content, flags=re.MULTILINE)

    if content != original_content:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)
        return True, changes_made
    return False, []


def main():
    """Update all relevant scripts."""
    scripts_dir = Path('scripts')

    # Patterns to search for
    patterns = [
        'data/raw',
        "year == '2020'",
        'year == "2020"',
    ]

    files_to_update = set()

    for pattern in patterns:
        for pyfile in scripts_dir.rglob('*.py'):
            if '__pycache__' in str(pyfile):
                continue
            try:
                with open(pyfile, 'r', encoding='utf-8') as f:
                    if pattern in f.read():
                        files_to_update.add(pyfile)
            except Exception:
                pass

    print(f"Found {len(files_to_update)} files to potentially update\n")

    updated_count = 0
    for filepath in sorted(files_to_update):
        was_updated, changes = update_file(filepath)
        if was_updated:
            updated_count += 1
            rel_path = filepath.relative_to(Path.cwd())
            print(f"✓ Updated: {rel_path}")
            for change in changes:
                print(f"  - {change}")

    print(f"\n{updated_count} files updated successfully")

if __name__ == '__main__':
    main()
