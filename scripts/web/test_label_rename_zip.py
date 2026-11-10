import base64,json,zipfile
from pathlib import Path
root=Path('target/label-rename-wasm')
packages=list(root.glob('case-*/result.zip'))
assert packages, 'Run the WASM parity gate first'
for path in packages:
 expected=json.loads(path.with_name('expected.json').read_bytes())
 with zipfile.ZipFile(path) as archive:
  assert archive.testzip() is None
  assert set(archive.namelist())==set(expected['files'])|set(expected['directories'])
  for name,data in expected['files'].items():
   assert archive.read(name)==base64.b64decode(data)
   assert not ((archive.getinfo(name).external_attr>>16)&0o111)
  for name in expected['directories']: assert archive.getinfo(name).is_dir()
print(f'Label rename ZIP: {len(packages)} file-set, byte, CRC, directory and non-executable permission comparisons passed.')
