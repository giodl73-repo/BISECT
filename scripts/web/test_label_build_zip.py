"""Check generated build archives independently using Python's ZIP reader."""
import json
import zipfile
from pathlib import Path

root = Path('target/label-build-wasm')
expected = json.loads((root / 'zip-files.json').read_text(encoding='utf-8'))
for name, files in expected.items():
    with zipfile.ZipFile(root / f'{name}.zip') as archive:
        assert archive.testzip() is None
        assert sorted(archive.namelist()) == sorted(files)
        for entry in archive.infolist():
            assert archive.read(entry) == files[entry.filename].encode('utf-8')
            assert not (entry.external_attr >> 16) & 0o111
            assert '..' not in entry.filename.split('/')
print('Label build ZIP: completed and cancelled archives have exact file sets, UTF-8 bytes, valid CRCs and no executable entries.')
