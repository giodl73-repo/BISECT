import hashlib,json,zipfile
from pathlib import Path
result=json.loads(Path('target/analysis-package-wasm/result.json').read_text(encoding='utf-8'))
with zipfile.ZipFile('target/analysis-package-wasm/analysis.zip') as archive:
    assert set(archive.namelist())==set(result['files'])
    assert archive.testzip() is None
    for name,text in result['files'].items():
        data=archive.read(name)
        assert data==text.encode('utf-8'),name
        assert hashlib.sha256(data).hexdigest()==result['file_sha256'][name],name
print(f"Independent analysis ZIP: exact paths, UTF-8 bytes, CRC and SHA-256 for {len(result['files'])} files passed.")
