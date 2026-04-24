#!/usr/bin/env python3
"""
Generate a self-contained dashboard HTML with all state maps embedded as base64.
Reads outputs/V1/2020/index.html, embeds ALL map PNGs, writes docs/dashboard.html.

Handles:
  - State district maps:  states/{state}/maps/all_districts.png
  - State round maps:     states/{state}/maps/rounds/round_NN.png  (dynamic JS var)
  - National map:         maps/us_all_districts.png
  - Individual districts: states/{state}/maps/districts/district_NN.png (dynamic)
"""

import base64
import io
import re
import sys
from pathlib import Path

from PIL import Image

OUTPUTS_DIR = Path('outputs/V3/2020')   # default; overridden by --version/--year
DASHBOARD_SRC = OUTPUTS_DIR / 'index.html'
DASHBOARD_OUT = Path('docs/dashboard.html')

MAP_WIDTH = 600        # all_districts maps
ROUND_WIDTH = 500      # round progression maps
NATIONAL_WIDTH = 900   # national map
DISTRICT_WIDTH = 400   # individual district maps
JPEG_QUALITY = 75


def encode_image(img_path: Path, max_width: int) -> str | None:
    if not img_path.exists():
        return None
    try:
        img = Image.open(img_path).convert('RGB')
        w, h = img.size
        if w > max_width:
            new_h = int(h * max_width / w)
            img = img.resize((max_width, new_h), Image.LANCZOS)
        buf = io.BytesIO()
        img.save(buf, format='JPEG', quality=JPEG_QUALITY, optimize=True)
        data = base64.b64encode(buf.getvalue()).decode('ascii')
        return f'data:image/jpeg;base64,{data}'
    except Exception as e:
        print(f'  [WARN] Could not encode {img_path}: {e}', file=sys.stderr)
        return None


def build_map_table() -> dict[str, str]:
    """
    Build mapping: relative path key (no extension) -> data URI.
    Keys mirror what the JS constructs, e.g.:
      'alabama/maps/all_districts'
      'alabama/maps/rounds/round_01'
      'national/us_all_districts'
    """
    table: dict[str, str] = {}
    states_dir = OUTPUTS_DIR / 'states'

    state_dirs = sorted(p for p in states_dir.iterdir() if p.is_dir())
    print(f'Embedding maps for {len(state_dirs)} states...')

    for state_dir in state_dirs:
        state = state_dir.name
        maps_dir = state_dir / 'maps'

        # all_districts
        src = encode_image(maps_dir / 'all_districts.png', MAP_WIDTH)
        if src:
            table[f'{state}/maps/all_districts'] = src
            print(f'  {state}: all_districts ({len(src)//1024}KB)', flush=True)

        # all_districts_with_cities
        src = encode_image(maps_dir / 'all_districts_with_cities.png', MAP_WIDTH)
        if src:
            table[f'{state}/maps/all_districts_with_cities'] = src

        # round maps — key uses zero-padded number matching the filename
        rounds_dir = maps_dir / 'rounds'
        if rounds_dir.exists():
            for rnd in sorted(rounds_dir.glob('round_*.png')):
                # stem is e.g. "round_01"
                key = f'{state}/maps/rounds/{rnd.stem}'
                src = encode_image(rnd, ROUND_WIDTH)
                if src:
                    table[key] = src

        # individual district maps
        districts_dir = maps_dir / 'districts'
        if districts_dir.exists():
            for d in sorted(districts_dir.glob('district_*.png')):
                key = f'{state}/maps/districts/{d.stem}'
                src = encode_image(d, DISTRICT_WIDTH)
                if src:
                    table[key] = src

    # National map
    national_candidates = [
        OUTPUTS_DIR / 'maps' / 'us_all_districts.png',
        OUTPUTS_DIR / 'us_all_districts.png',
        Path('outputs') / 'figures' / '2020' / 'us_all_districts.png',
    ]
    for candidate in national_candidates:
        src = encode_image(candidate, NATIONAL_WIDTH)
        if src:
            table['national/us_all_districts'] = src
            print(f'  [national] us_all_districts ({len(src)//1024}KB)')
            break
    else:
        print('  [national] us_all_districts.png not found — skipping', file=sys.stderr)

    return table


def strip_navigation(html: str, version_label: str = '') -> str:
    """Remove header controls that only work when served from outputs/.

    The ← Cross-Census Overview link and run selector both point to relative
    paths that don't exist in the standalone docs/ deployment. Strip them and
    replace with a simple static label showing which version this is.
    """
    # Remove the entire header-controls div (contains broken nav links)
    html = re.sub(
        r'<div class="header-controls">.*?</div>\s*(?=</div>)',
        (f'<div class="header-controls" style="color:#ccc;font-size:0.9rem;padding:8px 0;">'
         f'{version_label}</div>\n            ') if version_label else '',
        html,
        count=1,
        flags=re.DOTALL,
    )
    return html


def patch_html(html: str, table: dict[str, str]) -> str:
    """Patch the dashboard HTML to use embedded images."""

    # Build the JS lookup table
    js_entries = ',\n'.join(f'    {repr(k)}: {repr(v)}' for k, v in table.items())

    injection = f"""
<script>
// Embedded map images (base64) — generated by embed_maps_dashboard.py
const EMBEDDED_MAPS = {{
{js_entries}
}};

/**
 * Look up an embedded map by state + relative path.
 * relPath may contain JS expressions evaluated at call time, e.g.:
 *   getEmbeddedMap(state, 'maps/rounds/round_' + roundNumPadded)
 */
function getEmbeddedMap(state, relPath) {{
    const key = state + '/' + relPath;
    return EMBEDDED_MAPS[key] || null;
}}

/**
 * Look up a national map by key (no state prefix).
 */
function getEmbeddedNational(relPath) {{
    const key = 'national/' + relPath;
    return EMBEDDED_MAPS[key] || null;
}}
</script>
"""

    # -------------------------------------------------------------------------
    # Patch 1: Static state map srcs  (all_districts, all_districts_with_cities,
    #          demographic, political, compactness — all static path suffixes)
    #
    # Pattern in HTML:
    #   src="${basePath}/states/${state}/maps/all_districts.png"
    # Becomes:
    #   src="${getEmbeddedMap(state, 'maps/all_districts') || `${basePath}/states/${state}/maps/all_districts.png`}"
    # -------------------------------------------------------------------------
    def replace_static_state_src(m: re.Match) -> str:
        rel = m.group(1)  # e.g. "maps/all_districts"
        fallback = f'`${{basePath}}/states/${{state}}/{rel}.png`'
        return f'src="${{getEmbeddedMap(state, \'{rel}\') || {fallback}}}"'

    # Match static paths (no ${...} JS vars inside the path itself)
    html = re.sub(
        r'src="\$\{basePath\}/states/\$\{state\}/([^"$]+?)\.png"',
        replace_static_state_src,
        html
    )

    # -------------------------------------------------------------------------
    # Patch 2: Dynamic round maps — state level
    #
    # Pattern:
    #   src="${basePath}/states/${state}/maps/rounds/round_${roundNumPadded}.png"
    # Becomes:
    #   src="${getEmbeddedMap(state, 'maps/rounds/round_' + roundNumPadded) || `${basePath}/states/${state}/maps/rounds/round_${roundNumPadded}.png`}"
    # -------------------------------------------------------------------------
    html = html.replace(
        'src="${basePath}/states/${state}/maps/rounds/round_${roundNumPadded}.png"',
        "src=\"${getEmbeddedMap(state, 'maps/rounds/round_' + roundNumPadded) || `${basePath}/states/${state}/maps/rounds/round_${roundNumPadded}.png`}\""
    )

    # -------------------------------------------------------------------------
    # Patch 3: Dynamic individual district maps (concatenation style)
    #
    # Pattern:
    #   basePath + '/states/' + state + '/maps/districts/district_' + districtNumPadded + '.png'
    # Becomes:
    #   getEmbeddedMap(state, 'maps/districts/district_' + districtNumPadded) || (basePath + '/states/' + state + '/maps/districts/district_' + districtNumPadded + '.png')
    # -------------------------------------------------------------------------
    html = html.replace(
        "basePath + '/states/' + state + '/maps/districts/district_' + districtNumPadded + '.png'",
        "(getEmbeddedMap(state, 'maps/districts/district_' + districtNumPadded) || (basePath + '/states/' + state + '/maps/districts/district_' + districtNumPadded + '.png'))"
    )

    # -------------------------------------------------------------------------
    # Patch 4: National maps
    #
    # Pattern:
    #   src="${basePath}/maps/us_all_districts.png"
    # Becomes:
    #   src="${getEmbeddedNational('us_all_districts') || `${basePath}/maps/us_all_districts.png`}"
    # -------------------------------------------------------------------------
    def replace_national_src(m: re.Match) -> str:
        name = m.group(1)  # e.g. "us_all_districts"
        fallback = f'`${{basePath}}/maps/{name}.png`'
        return f'src="${{getEmbeddedNational(\'{name}\') || {fallback}}}"'

    html = re.sub(
        r'src="\$\{basePath\}/maps/([^"$]+?)\.png"',
        replace_national_src,
        html
    )

    # -------------------------------------------------------------------------
    # Patch 5: Dynamic national round maps
    #
    # Pattern:
    #   src="${basePath}/maps/rounds/round_${roundNumPadded}.png"
    # -------------------------------------------------------------------------
    html = html.replace(
        'src="${basePath}/maps/rounds/round_${roundNumPadded}.png"',
        "src=\"${getEmbeddedNational('rounds/round_' + roundNumPadded) || `${basePath}/maps/rounds/round_${roundNumPadded}.png`}\""
    )

    # -------------------------------------------------------------------------
    # Patch 6: Metro maps (dynamic ${metro.file})
    #
    # Pattern:
    #   src="${basePath}/states/${state}/maps/metros/${metro.file}.png"
    # Leave as-is (no embedded metro maps); just ensure fallback works.
    # -------------------------------------------------------------------------

    # Inject the lookup table before </body>
    html = html.replace('</body>', injection + '\n</body>')

    return html


def main():
    import argparse
    parser = argparse.ArgumentParser(
        description='Embed maps into dashboard HTML for GitHub Pages deployment'
    )
    parser.add_argument('--version', '-v', default=None,
                        help='Pipeline version (e.g. V3, V4). Default: V3')
    parser.add_argument('--year', '-y', default='2020',
                        choices=['2020', '2010', '2000'],
                        help='Census year (default: 2020)')
    parser.add_argument('--out', default='docs/dashboard.html',
                        help='Output path (default: docs/dashboard.html)')
    args = parser.parse_args()

    global OUTPUTS_DIR, DASHBOARD_SRC, DASHBOARD_OUT

    if args.version:
        # Try exact case, then uppercase, then lowercase
        for v in [args.version, args.version.upper(), args.version.lower()]:
            candidate = Path(f'outputs/{v}/{args.year}')
            if candidate.exists():
                OUTPUTS_DIR = candidate
                break
        else:
            print(f'[ERROR] outputs/{args.version}/{args.year}/ not found', file=sys.stderr)
            sys.exit(1)
    else:
        OUTPUTS_DIR = Path(f'outputs/V3/{args.year}')

    DASHBOARD_SRC = OUTPUTS_DIR / 'index.html'
    DASHBOARD_OUT = Path(args.out)

    if not DASHBOARD_SRC.exists():
        print(f'[ERROR] Source not found: {DASHBOARD_SRC}', file=sys.stderr)
        print(f'        Run: python scripts/web/generate_master_dashboard.py', file=sys.stderr)
        sys.exit(1)

    print(f'Source:  {DASHBOARD_SRC}')
    print(f'Output:  {DASHBOARD_OUT}')
    print(f'Reading HTML...')
    html = DASHBOARD_SRC.read_text(encoding='utf-8')

    table = build_map_table()
    total_kb = sum(len(v) for v in table.values()) // 1024
    print(f'\nEmbedded {len(table)} images (~{total_kb}KB base64 total)')

    print('Stripping navigation...')
    v = args.version.upper() if args.version else 'V3'
    version_label = f'{v} / {args.year}'
    html = strip_navigation(html, version_label)

    print('Patching HTML...')
    html = patch_html(html, table)

    DASHBOARD_OUT.parent.mkdir(parents=True, exist_ok=True)
    DASHBOARD_OUT.write_text(html, encoding='utf-8')
    size_mb = DASHBOARD_OUT.stat().st_size / 1_048_576
    print(f'[OK] {DASHBOARD_OUT} ({size_mb:.1f} MB)')
    print()
    print('Commit and push:')
    print('  git add docs/dashboard.html && git commit -m "Deploy dashboard" && git push origin master:main')


if __name__ == '__main__':
    main()
