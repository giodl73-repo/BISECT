import json
import zipfile
from pathlib import Path
root=Path('target/batch-accepted-replay');expected=json.loads((root/'package-files.json').read_text(encoding='utf-8'))
with zipfile.ZipFile(root/'package.zip') as archive:
 assert archive.testzip() is None
 assert set(archive.namelist())==set(expected)
 for path,text in expected.items():assert archive.read(path)==text.encode('utf-8')
 assert len([p for p in archive.namelist() if p.endswith('/accepted_engine_run.json')])==3
print('Accepted batch replay ZIP: three exact run artifacts and every companion file retain byte identity and valid CRCs.')
