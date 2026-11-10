"""Verify registry stages retain exact native bytes in both delivered archives."""
import hashlib
import json
import zipfile
from pathlib import Path
root = Path('target/label-stage-wasm')
count = 0
for result_name, archive_name in [('analysis.json', 'analysis.zip'), ('report.json', 'reports.zip')]:
    result = json.loads((root / result_name).read_text(encoding='utf-8'))
    with zipfile.ZipFile(root / archive_name) as archive:
        assert archive.testzip() is None
        assert set(archive.namelist()) == set(result['files'])
        assert archive.read('.bisect') == result['registry_json'].encode('utf-8')
        for name, text in result['files'].items():
            data = archive.read(name)
            assert data == text.encode('utf-8'), name
            assert hashlib.sha256(data).hexdigest() == result['file_sha256'][name], name
            count += 1
print(f'Native stage ZIPs: {count} files including exact .bisect registries, CRC and SHA-256 passed.')
