#!/usr/bin/env python3
"""
Generate version-level dashboard aggregating data across census years.

This script creates a dashboard for a specific experimental version (e.g., v1, v2)
showing cross-year comparisons of compactness, demographics, and other metrics.

Usage:
    python scripts/web/generate_version_dashboard.py --version v1
    python scripts/web/generate_version_dashboard.py --version v2 --output-dir outputs
"""

import argparse
import json
from pathlib import Path


def generate_version_dashboard(version, output_dir='outputs', template_file='web/version_dashboard.html'):
    """
    Generate version-level dashboard for a specific version.

    Args:
        version: Version string (e.g., 'v1', 'v2')
        output_dir: Base outputs directory
        template_file: Template file path
    """
    output_path = Path(output_dir)
    version_path = output_path / version

    # Verify version directory exists
    if not version_path.exists():
        print(f"[ERROR] Version directory does not exist: {version_path}")
        print(f"        Run redistricting for {version} first.")
        return False

    # 1. Read version.json for metadata
    version_json_path = version_path / 'version.json'
    if not version_json_path.exists():
        print(f"[ERROR] version.json not found: {version_json_path}")
        return False

    with open(version_json_path, encoding='utf-8') as f:
        version_config = json.load(f)

    print(f"Loaded version config: {version}")

    # 2. Load comparison.json
    comparison_path = output_path / 'comparison.json'
    if comparison_path.exists():
        with open(comparison_path, encoding='utf-8') as f:
            comparison_data = json.load(f)
        print(f"Loaded comparison data: {len(comparison_data)} years")
    else:
        comparison_data = {}
        print("[WARN] comparison.json not found - compactness comparison will be empty")

    # 3. Load runs.json to find runs for this version
    runs_path = output_path / 'runs.json'
    if not runs_path.exists():
        print(f"[ERROR] runs.json not found: {runs_path}")
        print("        Run generate_master_dashboard.py first to create runs.json")
        return False

    with open(runs_path, encoding='utf-8') as f:
        all_runs = json.load(f)

    # Filter runs for this version
    version_runs = [r for r in all_runs if r.get('version') == version]
    print(f"Found {len(version_runs)} run(s) for {version}")

    if len(version_runs) == 0:
        print(f"[WARN] No completed runs found for {version}")

    # 4. Read template
    template_path = Path(template_file)
    if not template_path.exists():
        print(f"[ERROR] Template not found: {template_path}")
        return False

    with open(template_path, encoding='utf-8') as f:
        html = f.read()

    # 5. Replace placeholders
    description = version_config.get('description', '')
    html = html.replace('{VERSION}', version)
    html = html.replace('{DESCRIPTION}', description)
    html = html.replace('/* VERSION_CONFIG_PLACEHOLDER */', json.dumps(version_config, indent=4))
    html = html.replace('/* COMPARISON_DATA_PLACEHOLDER */', json.dumps(comparison_data, indent=4))
    html = html.replace('/* RUNS_PLACEHOLDER */', json.dumps(version_runs, indent=4))

    # 6. Write output
    output_file = version_path / 'index.html'
    with open(output_file, 'w', encoding='utf-8') as f:
        f.write(html)

    print(f"[OK] Generated version dashboard: {output_file}")
    return True


def main():
    parser = argparse.ArgumentParser(
        description='Generate version-level dashboard for cross-census analysis'
    )
    parser.add_argument(
        '--version',
        required=True,
        help='Version name (e.g., v1, v2, v3)'
    )
    parser.add_argument(
        '--output-dir',
        default='outputs',
        help='Base output directory (default: outputs)'
    )
    parser.add_argument(
        '--template',
        default='web/version_dashboard.html',
        help='Template file path (default: web/version_dashboard.html)'
    )

    args = parser.parse_args()

    print(f"\nGenerating version dashboard for {args.version}...")
    print(f"Output directory: {args.output_dir}")
    print(f"Template: {args.template}\n")

    success = generate_version_dashboard(
        version=args.version,
        output_dir=args.output_dir,
        template_file=args.template
    )

    if success:
        print(f"\n[OK] Version dashboard generation complete!")
        print(f"     Open: {args.output_dir}/{args.version}/index.html")
    else:
        print(f"\n[FAIL] Version dashboard generation failed")
        return 1

    return 0


if __name__ == '__main__':
    exit(main())
