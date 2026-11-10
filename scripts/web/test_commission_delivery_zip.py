import json,zipfile
from pathlib import Path
root=Path('target/commission-delivery-wasm')
for name in [f'delivery-{n}' for n in range(8)]+['audit']:
 expected=json.loads((root/(name+'-files.json')).read_text(encoding='utf-8'))
 with zipfile.ZipFile(root/(name+'.zip')) as z:
  assert z.testzip() is None
  assert sorted(z.namelist())==sorted(expected)
  for p,text in expected.items():assert z.read(p)==text.encode('utf-8')
(root/'zip-evidence.json').write_text(json.dumps({'independent_zipfile_archives':9,'exact_native_filenames_bytes':True,'crc':True},indent=2),encoding='utf-8')
print('Commission ZIP: nine archives independently verified for exact native filenames/bytes and CRC.')
