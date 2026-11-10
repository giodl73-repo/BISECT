"""Pin browser acquisition endpoints to the native built-in manifest.

Check mode is read-only. The native source-plan parity test covers the 2020
PL94 override, rather than trusting this generation rule alone.
"""
import json, pathlib, sys
root = pathlib.Path(__file__).resolve().parents[2]
manifest = json.loads((root / 'data/manifest.json').read_text())
endpoints = {}
for code, state in manifest['states'].items():
    for kind, key in [('tiger', 'tiger'), ('pl94171', 'pl94171')]:
        for year, url in state.get(key, {}).items():
            if year not in ['2000', '2010', '2020']:
                continue
            if kind == 'pl94171' and year == '2020':
                name = state['name'].replace(' ', '_')
                url = f'https://www2.census.gov/programs-surveys/decennial/2020/data/01-Redistricting_File--PL_94-171/{name}/{code.lower()}2020.pl.zip'
            if not url.startswith('https://www2.census.gov/'):
                continue
            endpoints[f'{code}:{year}:{kind}'] = url
text = '// Generated from data/manifest.json; check with scripts/web/census_endpoints.py --check.\nexport const CENSUS_ARCHIVE_ENDPOINTS=Object.freeze(' + json.dumps(endpoints, sort_keys=True, indent=2) + ');\n'
output = root / 'web/lab/census-endpoints.js'
if '--check' in sys.argv:
    if not output.exists() or output.read_text() != text:
        raise SystemExit('Census browser endpoints have drifted from the native manifest.')
else:
    output.write_text(text)
print(f'{len(endpoints)} pinned Census archive endpoints match native manifest')
