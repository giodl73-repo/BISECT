#!/usr/bin/env python3
"""
Deploy the redistricting dashboard to docs/ for GitHub Pages.

Generates docs/dashboard.html — a fully self-contained HTML file with all
state maps embedded as base64 JPEG. Works with no server and no outputs/.

Usage:
    # Deploy V3 2020 (default — canonical edge-weighted result)
    python scripts/web/deploy_docs.py

    # Deploy V4 (VRA) results
    python scripts/web/deploy_docs.py --version V4 --year 2020

    # Preview without writing
    python scripts/web/deploy_docs.py --dry-run

After running, commit docs/dashboard.html and push — GitHub Pages will
serve it at: https://giodl73-repo.github.io/REDIST/dashboard.html

Full workflow after completing a pipeline run:
    python scripts/web/generate_master_dashboard.py
    python scripts/web/deploy_docs.py --version V3 --year 2020
    git add docs/dashboard.html
    git commit -m "Deploy dashboard: V3/2020"
    git push origin master:main
"""

import argparse
import json
import subprocess
import sys
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parents[2]


def find_best_version(outputs_dir: Path, year: str) -> str | None:
    """Return the highest numbered version that has completed state runs."""
    candidates = []
    for v in outputs_dir.iterdir():
        if not v.is_dir():
            continue
        name = v.name.upper()
        if not name.startswith('V') or not name[1:].isdigit():
            continue
        states_dir = v / year / 'states'
        if states_dir.exists() and any(states_dir.iterdir()):
            candidates.append(v.name)
    if not candidates:
        return None
    candidates.sort(key=lambda x: int(x.upper().lstrip('V')), reverse=True)
    return candidates[0]


def main():
    parser = argparse.ArgumentParser(
        description='Deploy self-contained dashboard to docs/ for GitHub Pages',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )
    parser.add_argument('--version', '-v', default=None,
                        help='Version to deploy (e.g. V3, V4). '
                             'Default: highest completed version for the year.')
    parser.add_argument('--year', '-y', default='2020',
                        choices=['2020', '2010', '2000'],
                        help='Census year (default: 2020)')
    parser.add_argument('--out', '-o', default=None,
                        help='Output filename in docs/ (default: dashboard.html for V3, '
                             'dashboard_v4.html for V4, etc.)')
    parser.add_argument('--dry-run', action='store_true',
                        help='Show what would be done without writing')
    args = parser.parse_args()

    outputs_dir = PROJECT_ROOT / 'outputs'

    # Resolve version
    version = args.version
    if version is None:
        version = find_best_version(outputs_dir, args.year)
        if version is None:
            print(f'ERROR: No completed runs found for {args.year} in outputs/')
            print('       Run the pipeline first: run -y 2020 -v V3')
            sys.exit(1)
        print(f'[auto] Selected version {version} (highest completed for {args.year})')

    # Find the version directory
    for v in [version, version.upper(), version.lower()]:
        version_dir = outputs_dir / v
        if version_dir.exists():
            version = v
            break
    else:
        print(f'ERROR: outputs/{version}/ not found')
        sys.exit(1)

    year_dir = version_dir / args.year
    if not (year_dir / 'index.html').exists():
        print(f'ERROR: {year_dir}/index.html not found')
        print('       Run: python scripts/web/generate_master_dashboard.py')
        sys.exit(1)

    # Read metadata for display
    version_json = version_dir / 'version.json'
    description = f'{version.upper()} / {args.year}'
    mode = 'edge-weighted'
    if version_json.exists():
        with open(version_json) as f:
            meta = json.load(f)
        description = meta.get('description', description)
        mode = meta.get('algorithm', {}).get('partition_mode', 'edge_weighted').replace('_', '-')

    # Determine output filename
    out_filename = args.out
    if out_filename is None:
        vnum = version.upper().lstrip('V')
        if vnum == '3':
            out_filename = 'dashboard.html'
        elif mode in ('metis-vra', 'metis_vra'):
            out_filename = 'dashboard_vra.html'
        else:
            out_filename = f'dashboard_v{vnum.lower()}.html'
    out_path = f'docs/{out_filename}'

    print()
    print('=== Deploy Dashboard to GitHub Pages ===')
    print(f'  Version : {version.upper()} — {description}')
    print(f'  Mode    : {mode}')
    print(f'  Year    : {args.year}')
    print(f'  Source  : {year_dir.relative_to(PROJECT_ROOT)}/index.html')
    print(f'  Output  : {out_path}')
    print()

    if args.dry_run:
        print(f'[dry-run] Would write {out_path}.')
        return

    # Delegate to embed_maps_dashboard.py
    embed_script = PROJECT_ROOT / 'scripts' / 'web' / 'embed_maps_dashboard.py'
    cmd = [
        sys.executable, str(embed_script),
        '--version', version,
        '--year', args.year,
        '--out', out_path,
    ]

    print('Embedding maps (this takes a few minutes)...')
    result = subprocess.run(cmd, cwd=str(PROJECT_ROOT))

    if result.returncode != 0:
        print('\n[ERROR] Embed failed.')
        sys.exit(1)

    out = PROJECT_ROOT / out_path
    size_mb = out.stat().st_size / 1_048_576
    print()
    print(f'[OK] {out_path} ready ({size_mb:.1f} MB)')
    print()
    print('Next steps:')
    print(f'  git add {out_path}')
    print(f'  git commit -m "Deploy dashboard: {version.upper()}/{args.year}"')
    print(f'  git push origin master:main')
    print()
    print('GitHub Pages URL:')
    print('  https://giodl73-repo.github.io/REDIST/dashboard.html')


if __name__ == '__main__':
    main()
