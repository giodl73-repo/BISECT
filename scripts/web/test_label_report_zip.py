"""Independent byte, filename, CRC and hash verification of native report batches."""
import hashlib
import json
import zipfile
from pathlib import Path
root = Path('target/label-report-wasm')
count = 0
for result_name, archive_name in [('result.json', 'reports.zip'), ('html-result.json', 'html-reports.zip')]:
    result = json.loads((root / result_name).read_text(encoding='utf-8'))
    with zipfile.ZipFile(root / archive_name) as archive:
        assert archive.testzip() is None
        assert set(archive.namelist()) == set(result['files'])
        for name, text in result['files'].items():
            data = archive.read(name)
            assert data == text.encode('utf-8'), name
            assert hashlib.sha256(data).hexdigest() == result['file_sha256'][name], name
            count += 1
print(f'Native report ZIPs: {count} exact JSON/HTML/audit/index/record files, paths, CRCs and SHA-256 values passed.')
