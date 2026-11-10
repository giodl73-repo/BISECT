"""Independent ZIP CRC, member bytes and index-to-unit checks with Python zipfile."""
import io,json,zipfile
from pathlib import Path
base=Path('target/native-state-files');expect=json.loads((base/'expectations.json').read_text(encoding='utf-8'));checked=0
def inspect(data,assignments=None,summary=None):
 global checked
 with zipfile.ZipFile(io.BytesIO(data)) as archive:
  assert archive.testzip() is None
  names=archive.namelist();assert len(names)==len(set(names));assert all(not n.startswith('/') and '..' not in n.split('/') for n in names)
  prefix='native-state/'
  unit=json.loads(archive.read(prefix+'unit-index.json'));mapping=json.loads(archive.read(prefix+'data/final_assignments.json'));by_unit=json.loads(archive.read(prefix+'assignments-by-unit.json'));receipt=json.loads(archive.read(prefix+'receipt.json'))
  assert set(mapping)=={str(i) for i in range(len(unit['unit_ids']))}
  assert {unit['unit_ids'][int(i)]:district for i,district in mapping.items()}==by_unit
  assert receipt['units']==len(unit['unit_ids']) and receipt['assignment_labels']=='one-based'
  assert all(prefix+name in names for name in receipt['files'])
  if assignments is not None:assert assignments==by_unit
  if summary is not None:assert archive.read(prefix+'data/vra_analysis.json').decode()==summary
  else:assert prefix+'data/vra_analysis.json' not in names
  checked+=1
for item in expect['artifacts']:
 data=(base/item['name']).read_bytes();inspect(data,item['assignments'],item['native_vra'])
 with zipfile.ZipFile(io.BytesIO(data)) as archive:assert archive.read('native-state/data/final_assignments.json').decode()==item['native_assignments']
for name,key in [('prepared.zip','prepared_summary'),('tui.zip','tui_summary'),('workflow.zip','workflow_summary')]:inspect((base/name).read_bytes(),summary=expect[key])
for name,key in [('prepared-data.zip','prepared_summary'),('tui-data.zip','tui_summary')]:inspect((base/name).read_bytes(),summary=expect[key])
ms=json.loads((base/'multiscale-expected.json').read_text(encoding='utf-8'));inspect((base/'multiscale-data.zip').read_bytes(),assignments=ms['assignments'])
with zipfile.ZipFile(base/'multiscale-data.zip') as archive:
 assert json.loads(archive.read('native-state/unit-index.json'))['unit_ids']==ms['unit_ids'];assert json.loads(archive.read('native-state/multiscale-settings.json'))['fine_level']=='bg'
with zipfile.ZipFile(base/'batch.zip') as archive:
 assert archive.testzip() is None;inspect(archive.read('RI/prepared-run.zip'),summary=expect['prepared_summary'])
with zipfile.ZipFile(base/'legacy-tui.zip') as archive:
 assert archive.testzip() is None;assert all(not n.startswith('native-state/') for n in archive.namelist());assert {'final_assignments.json','data/final_assignments.json','plan.rplan','context.rctx'}<=set(archive.namelist())
lab=Path('target/native-state-files-lab')
for item in json.loads((lab/'expectations.json').read_text(encoding='utf-8')):
 data=(lab/item['name']).read_bytes();inspect(data,item['assignments'],item['native_vra'])
 with zipfile.ZipFile(io.BytesIO(data)) as archive:
  assert json.loads(archive.read('native-state/engine-options.json'))['seed']==item['seed']
  if item['fine_level']=='bg':
   assert json.loads(archive.read('native-state/unit-index.json'))['unit_kind']=='block-group';assert json.loads(archive.read('native-state/multiscale-settings.json'))['fine_level']=='bg'
browser=Path('target/native-state-files-browser')
if (browser/'expectations.json').exists():
 for item in json.loads((browser/'expectations.json').read_text(encoding='utf-8')):inspect((browser/item['name']).read_bytes(),item['assignments'],item['summary'])
evidence={'independent_zip_reader':'Python zipfile','archives_checked':checked,'crc_and_duplicate_path_checks':True,'native_assignment_and_available_summary_bytes':True,'unit_index_roundtrips':True,'legacy_members_retained':True,'published':False};(base/'zip-evidence.json').write_text(json.dumps(evidence,indent=2)+'\n',encoding='utf-8');print(f'PASS independent native State archives: {checked} ZIPs, exact native data bytes, CRC/path checks and unit-index roundtrips')
