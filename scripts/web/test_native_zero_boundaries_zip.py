"""Read downloaded ZIP without executing any payload; independently check bytes/hashes."""
from pathlib import Path
from zipfile import ZipFile
import json,hashlib,base64
root=Path('target/native-zero-boundaries-browser')
project=json.loads((root/'exported.bisect').read_text(encoding='utf-8'))
with ZipFile(root/'sources.zip') as package:
 assert package.testzip() is None
 manifest=json.loads(package.read('bundle.json'))
 assert set(package.namelist())==set(manifest['files'])|{'bundle.json'}
 for name,identity in manifest['files'].items():
  assert not name.startswith('/') and '..' not in name.split('/')
  data=package.read(name)
  assert identity=={'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()}
 graph_bytes=package.read('sources/graph.json')
 assert graph_bytes==base64.b64decode(project['files']['configRunInputs']['graph']['base64'])
 graph=json.loads(graph_bytes)
 assert len(graph['geoids'])==250 and sum(graph['population'])==1097379
 assert len(graph['edges'])>0 and all(edge[2]==0 for edge in graph['edges'])
 assert len(graph['areas'])==len(graph['exterior_perimeters'])==len(graph['centroids'])==250
 # The original RADJ selection remains exact inside the saved project.
 selected=project['files']['nativeAdjacencyInputs']['adjacency']
 probe=json.loads(Path('target/native-zero-boundaries/probe.json').read_text(encoding='utf-8'))
 assert base64.b64decode(selected['base64'])==(Path(probe['workspace'])/'all-zero.adj.bin').read_bytes()
 record={'archive_files':len(manifest['files']),'crc_paths_hashes_verified':True,'graph_source_exact':True,'all_zero_weights_preserved':True,'original_native_binary_in_project_exact':True,'script_or_engine_execution':False}
Path('target/native-zero-boundaries/zip-evidence.json').write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8')
print('Zero boundaries ZIP: every member hash/CRC verified, exact graph and saved native binary, zero weights retained.')
