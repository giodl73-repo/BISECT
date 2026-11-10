"""Read generated ZIPs independently; never extract their selected paths."""
import base64
import hashlib
import json
import zipfile
from pathlib import Path

root = Path('target/fetch-data-package-wasm')
for name in ('housing', 'workplace', 'commuting'):
    result = json.loads((root / f'{name}-result.json').read_text(encoding='utf-8'))
    for mode in ('supplied', '404'):
        receipt = result if mode == 'supplied' else json.loads(
            (root / f'{name}-404-result.json').read_text(encoding='utf-8'))
        expected = {path: base64.b64decode(data, validate=True)
                    for path, data in receipt['files_base64'].items()}
        assert len(expected) == 2
        assert expected[receipt['item']['done_marker']] == b'done'
        with zipfile.ZipFile(root / f'{name}-{mode}.zip') as archive:
            assert archive.testzip() is None
            assert sorted(archive.namelist()) == sorted(expected)
            for info in archive.infolist():
                data = archive.read(info.filename)
                assert data == expected[info.filename]
                digest = receipt['marker_sha256'] if info.filename == receipt['item']['done_marker'] else receipt['target_sha256']
                assert hashlib.sha256(data).hexdigest() == digest
                assert not (info.external_attr >> 16) & 0o111
print('Fetch data ZIPs: six archives, exact CSV/marker bytes, CRC, SHA-256 and non-executable permissions passed.')
