"""Generate tiny synthetic state-scoped ZIPs for acquisition tests.

Geometry and population remain the two-tract fixture. Only state identifiers
change; these files are not authentic state datasets or capacity evidence.
"""
import json
from pathlib import Path
import zipfile

root = Path('target/state-source-acquisition/synthetic-states')
root.mkdir(parents=True, exist_ok=True)
states = json.loads(Path('data/manifest.json').read_text(encoding='utf-8'))['states']
for state, entry in states.items():
    if state == 'PR':
        continue
    for kind, source in [('tiger', 'deflated.zip'), ('pl94', 'pl2020.zip')]:
        with zipfile.ZipFile(Path('target/census-archive-tests') / source) as original:
            with zipfile.ZipFile(root / f'{state}-{kind}.zip', 'w', zipfile.ZIP_STORED) as output:
                for member in original.infolist():
                    data = original.read(member)
                    if member.filename.lower().endswith('.dbf') or 'geo2020.pl' in member.filename.lower():
                        data = data.replace(b'44', entry['fips'].encode('ascii'))
                    output.writestr(member.filename, data)
print(f'Synthetic state acquisition fixtures generated in {root}')
