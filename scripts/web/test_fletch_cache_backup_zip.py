"""Independent ZIP/SHA/native-entry checks for actual browser backup downloads."""
import hashlib
import json
from pathlib import Path
from zipfile import ZipFile
root = Path('target/fletch-cache-browser')
fixtures = [json.loads(Path(f'target/fletch-stage-wasm/{name}.bisect').read_text(encoding='utf-8')) for name in ('text', 'binary')]
import base64
expected = {p['result']['cache_key']: base64.b64decode(p['files']['fletchStageInputs']['source']['base64']) for p in fixtures}
for index_name, override in [('backup-index.json', None), ('ui-backup-index.json', 'ui-backup.zip')]:
    index = json.loads((root / index_name).read_text(encoding='utf-8'))
    assert index['schema_version'] == 'bisect-cache-backup-index-v1'
    assert hashlib.sha256(index['manifest_json'].encode()).hexdigest() == index['snapshot_sha256']
    manifest = json.loads(index['manifest_json'])
    entries = manifest['entries']
    assert index['object_count'] == len(entries) == 2
    assert any(e['fetched_at_ms'] == 9007199254740993 for e in entries)
    restored = []
    for ordinal, descriptor in enumerate(index['volumes']):
        filename = override or descriptor['name']
        raw = (root / filename).read_bytes()
        assert len(raw) == descriptor['byte_length']
        assert hashlib.sha256(raw).hexdigest() == descriptor['sha256']
        with ZipFile(root / filename) as z:
            assert z.testzip() is None
            names = z.namelist()
            assert len(names) == len(set(names))
            library = json.loads(z.read('cache-library.json'))
            part = json.loads(z.read('cache-manifest.json'))
            assert library['snapshot_sha256'] == index['snapshot_sha256']
            assert library['volume_index'] == ordinal
            assert library['volume_count'] == len(index['volumes'])
            assert descriptor['object_keys'] == [e['cache_key'] for e in part['entries']]
            assert set(names) == {'cache-library.json', 'cache-manifest.json'} | {e['relative_path'] for e in part['entries']}
            for e, row in zip(part['entries'], library['rows'], strict=True):
                assert json.loads(row['entry_json']) == [e]
                assert row['fetched_at_ms'] == str(e['fetched_at_ms'])
                body = z.read(e['relative_path'])
                assert body == expected[e['cache_key']]
                assert len(body) == e['bytes']
                assert 'sha256:' + hashlib.sha256(body).hexdigest() == e['sha256']
                restored.append(e)
    assert restored == entries, 'Volumes must reconstruct original native entry order and exact integers'
print('Independent Python: split-volume and UI ZIP/index hashes, CRCs, exact source bytes, exact u64, native entries and full snapshot ordering passed.')
