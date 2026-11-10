"""Compare original Census block records to exported native/WASM tract totals."""
from collections import defaultdict
import hashlib
import json
from pathlib import Path
import zipfile

root = Path('target/census-2000-sources')
evidence = json.loads((root / 'verified.json').read_text(encoding='utf-8'))
with zipfile.ZipFile(root / 'VT/pl94171.zip') as archive:
    assert archive.testzip() is None
    names = [name for name in archive.namelist() if name.lower().endswith('vtgeo.upl')]
    assert len(names) == 1
    source = archive.read(names[0])
blocks = {}
for line in source.splitlines():
    if line[8:11] != b'750':
        continue
    geoid = (line[29:34] + line[55:61] + line[62:66]).decode('ascii')
    assert len(geoid) == 15 and geoid.isdigit() and geoid not in blocks
    blocks[geoid] = int(line[292:301])
assert sum(blocks.values()) == 608827
tracts = defaultdict(int)
for geoid, population in blocks.items():
    tracts[geoid[:11]] += population
result = json.loads(Path('target/census-2000-acquisition/original-result.json').read_text(encoding='utf-8'))
graph = json.loads(result['source_preparation']['graph_json'])
actual = dict(zip(graph['geoids'], graph['population']))
assert all(actual[geoid] == population for geoid, population in tracts.items())
assert all(population == tracts.get(geoid, 0) for geoid, population in actual.items())
assert sum(actual.values()) == 608827
for receipt in evidence['original_archives']:
    raw = (root / 'VT' / (receipt['kind'] + '.zip')).read_bytes()
    assert len(raw) == receipt['byte_length']
    assert hashlib.sha256(raw).hexdigest() == receipt['sha256']
print(f'PASS original 2000 VT: {len(blocks)} unique blocks, {len(tracts)} populated/header tracts and {len(actual)} graph tracts; every tract total matches original PL94, state population 608827')
