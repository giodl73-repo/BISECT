"""Run the independent native reader on retained, original CA archive members."""
from pathlib import Path
import shutil
import subprocess
import zipfile

root = Path('target/original-state-acquisition/CA')
parts = root / 'parts'
parts.mkdir(parents=True, exist_ok=True)
for kind, names in [('tiger', ['tl_2020_06_tract.shp', 'tl_2020_06_tract.dbf', 'tl_2020_06_tract.shx']), ('pl94171', ['cageo2020.pl', 'ca000012020.pl'])]:
    with zipfile.ZipFile(root / (kind + '.zip')) as archive:
        for name in names:
            # Fixed names only; archive paths never become filesystem targets.
            with archive.open(name) as source, (parts / name).open('wb') as output:
                shutil.copyfileobj(source, output, 1024 * 1024)
with (root / 'native-preparation.json').open('wb') as output:
    subprocess.run(['target/debug/examples/census_prepare_reference.exe', 'CA', '2020', 'tract', str(parts / 'tl_2020_06_tract.shp'), str(parts / 'cageo2020.pl'), str(parts / 'ca000012020.pl')], stdout=output, check=True)
print('Fresh original CA native Census source preparation recorded')
