"""Extract fixed graph/geometry oracles from the verified original CA delivery."""
import json
from pathlib import Path
import re
import zipfile

root = Path('target/stream-state-source-batch')
evidence = json.loads((root / 'original-ca-topology-evidence.json').read_text(encoding='utf-8'))
folder = evidence['output_folder']
assert re.fullmatch(r'bisect-run-[0-9a-f-]{36}', folder)
output = Path('target/full-ca-browser')
output.mkdir(exist_ok=True)
with zipfile.ZipFile(root / folder / 'CA/state-output.zip') as archive:
    for name in ['prepared-graph.json', 'prepared-geometry.json']:
        (output / name).write_bytes(archive.read('CA/' + name))
print('Original CA browser graph/geometry oracles prepared')
