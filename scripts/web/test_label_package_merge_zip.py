import hashlib,json,zipfile
from pathlib import Path
root=Path('target/label-package-merge-wasm')
result=json.loads((root/'report.json').read_text(encoding='utf-8'))
with zipfile.ZipFile(root/'repository.zip') as archive:
 assert archive.testzip() is None
 assert sorted(archive.namelist())==sorted(result['files'])
 for info in archive.infolist():
  data=archive.read(info.filename)
  assert data==result['files'][info.filename].encode('utf-8')
  assert hashlib.sha256(data).hexdigest()==result['file_sha256'][info.filename]
  assert not (info.external_attr>>16)&0o111
print(f"Repository ZIP: {len(result['files'])} exact UTF-8 files, CRC, SHA-256 and non-executable permissions passed.")
