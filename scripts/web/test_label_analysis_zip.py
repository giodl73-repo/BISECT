"""Independently verify native batch ZIP paths, UTF-8 bytes, CRC and SHA-256."""
import hashlib
import json
import zipfile
from pathlib import Path

root = Path('target/label-analysis-wasm')
result = json.loads((root / 'result.json').read_text(encoding='utf-8'))
with zipfile.ZipFile(root / 'analysis.zip') as archive:
    assert archive.testzip() is None
    assert set(archive.namelist()) == set(result['files'])
    for name, text in result['files'].items():
        data = archive.read(name)
        assert data == text.encode('utf-8'), name
        assert hashlib.sha256(data).hexdigest() == result['file_sha256'][name], name
print(f"Native batch ZIP: {len(result['files'])} exact files, paths, CRC and SHA-256 passed.")
