#!/usr/bin/env python3
"""Update all scripts to use unified directory structure."""

import re
from pathlib import Path

def update_script(filepath):
    """Update a single script file."""
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()

    original = content

    # Pattern 1: Tracts file conditional (most common pattern)
    pattern1 = re.compile(
        r"(\s+)if (?:args\.)?year == ['\"]2020['\"]:\s*\n"
        r"\s+tracts_file = (?:Path\()?f?['\"]data/raw/\{[^}]+\}_tracts_\{[^}]+\}\.parquet['\"](?:\))?\s*\n"
        r"\s+else:\s*\n"
        r"\s+tracts_file = ((?:Path\()?f?['\"]data/tracts/\{[^}]+\}/\{[^}]+\}_tracts_\{[^}]+\}\.parquet['\"](?:\))?)",
        re.MULTILINE
    )
    content = pattern1.sub(r'\1tracts_file = \2', content)

    # Pattern 2: Places file conditional
    pattern2 = re.compile(
        r"(\s+)if (?:args\.)?year == ['\"]2020['\"]:\s*\n"
        r"\s+places_file = (?:Path\()?f?['\"]data/raw/\{[^}]+\}_places_\{[^}]+\}\.parquet['\"](?:\))?\s*\n"
        r"\s+else:\s*\n"
        r"\s+places_file = ((?:Path\()?f?['\"]data/tracts/\{[^}]+\}/\{[^}]+\}_places_\{[^}]+\}\.parquet['\"](?:\))?)",
        re.MULTILINE
    )
    content = pattern2.sub(r'\1places_file = \2', content)

    # Pattern 3: Input/output dir for adjacency
    pattern3 = re.compile(
        r"(\s+)if year == ['\"]2020['\"]:\s*\n"
        r"\s+input_dir = ['\"]data/raw['\"]\s*\n"
        r"\s+output_dir = ['\"]data/adjacency['\"]\s*\n"
        r"\s+else:\s*\n"
        r"\s+input_dir = f['\"]data/tracts/\{year\}['\"]\s*\n"
        r"\s+output_dir = f['\"]data/adjacency/\{year\}['\"]",
        re.MULTILINE
    )
    content = pattern3.sub(
        r"\1input_dir = f'data/tracts/{year}'\n"
        r"\1output_dir = f'data/adjacency/{year}'",
        content
    )

    # Pattern 4: Graph file conditional
    pattern4 = re.compile(
        r"(\s+)if year == ['\"]2020['\"]:\s*\n"
        r"\s+graph_file = (?:Path\()?f?['\"]data/adjacency/\{[^}]+\}_adjacency_\{[^}]+\}\.pkl['\"](?:\))?\s*\n"
        r"\s+else:\s*\n"
        r"\s+graph_file = ((?:Path\()?f?['\"]data/adjacency/\{[^}]+\}/\{[^}]+\}_adjacency_\{[^}]+\}\.pkl['\"](?:\))?)",
        re.MULTILINE
    )
    content = pattern4.sub(r'\1graph_file = \2', content)

    # Pattern 5: Tracts file check conditional
    pattern5 = re.compile(
        r"(\s+)if year == ['\"]2020['\"]:\s*\n"
        r"\s+tracts_file = Path\(f['\"]data/raw/\{[^}]+\}_tracts_\{[^}]+\}\.parquet['\"]\)\s*\n"
        r"\s+else:\s*\n"
        r"\s+tracts_file = (Path\(f['\"]data/tracts/\{[^}]+\}/\{[^}]+\}_tracts_\{[^}]+\}\.parquet['\"]\))",
        re.MULTILINE
    )
    content = pattern5.sub(r'\1tracts_file = \2', content)

    if content != original:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)
        return True
    return False

# List of scripts to update
scripts = [
    "scripts/pipeline/add_cities_to_districts.py",
    "scripts/pipeline/create_individual_district_maps.py",
    "scripts/pipeline/create_us_national_map.py",
    "scripts/pipeline/create_us_national_rounds_progression.py",
    "scripts/pipeline/run_complete_redistricting.py",
    "scripts/pipeline/run_state_redistricting.py",
    "scripts/pipeline/visualize_all_rounds.py",
    "scripts/political/analyze_districts.py",
    "scripts/political/visualize_partisan_lean.py",
    "scripts/political/create_us_national_political_map.py",
    "scripts/demographic/analyze_district_demographics.py",
    "scripts/demographic/visualize_district_demographics.py",
    "scripts/demographic/create_us_national_demographic_map.py",
    "scripts/compactness/visualize_compactness.py",
    "scripts/compactness/create_us_national_compactness_map.py",
    "scripts/data/geography/build_all_adjacency_graphs.py",
    "scripts/data/geography/check_graph_connectivity.py",
    "scripts/data/geography/check_isolated_tracts.py",
    "scripts/data/geography/diagnose_components.py",
    "scripts/visualization/create_metro_area_maps.py",
]

updated = 0
for script_path in scripts:
    path = Path(script_path)
    if path.exists():
        if update_script(path):
            print(f"✓ Updated: {script_path}")
            updated += 1
        else:
            print(f"  Skipped: {script_path} (no changes needed)")
    else:
        print(f"  Missing: {script_path}")

print(f"\n{updated} files updated")
