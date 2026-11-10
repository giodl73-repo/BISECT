"""Export original CA shared-boundary geometry through the independent native reader."""
from pathlib import Path
import subprocess

root = Path('target/original-state-acquisition/CA')
parts = root / 'parts'
for extension in ['shp', 'dbf', 'shx']:
    if not (parts / f'tl_2020_06_tract.{extension}').is_file():
        raise SystemExit('Run prepare_original_ca_oracle.py with retained original CA ZIPs first.')
with (root / 'topology-geometry.json').open('wb') as output:
    subprocess.run([
        'target/release/examples/census_topology_geometry.exe',
        str(parts / 'tl_2020_06_tract.shp'), '11', '06', 'CA', '2020',
    ], stdout=output, check=True)
print('Fresh original CA native shared-boundary geometry recorded')
