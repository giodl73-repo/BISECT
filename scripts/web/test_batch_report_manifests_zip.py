import json
import zipfile
from pathlib import Path
root=Path('target/batch-report-manifests')
manifests=0
for i in range(2):
 expected=json.loads((root/f'batch-{i}-files.json').read_text(encoding='utf-8'))
 with zipfile.ZipFile(root/f'batch-{i}.zip') as archive:
  assert archive.testzip() is None
  assert set(archive.namelist())==set(expected)
  for name,text in expected.items():
   assert archive.read(name)==text.encode('utf-8')
   if name.endswith('/manifest.json'):manifests+=1
assert manifests==6
print('Batch manifest ZIP: six native manifests and all companion files retain exact bytes and valid ZIP CRCs.')
