import base64
import json
import subprocess
from pathlib import Path
from zipfile import ZipFile

root=Path('target/civic-ingest-wasm')
result=json.loads((root/'browser-project.bisect').read_bytes())['result']
output=(root/'native-readback').resolve()
with ZipFile(root/'package.zip') as archive:
    assert archive.testzip() is None
    assert set(archive.namelist())==set(result['files_base64'])
    for entry in archive.infolist():
        assert not ((entry.external_attr >> 16) & 0o111)
        payload=archive.read(entry)
        assert payload==base64.b64decode(result['files_base64'][entry.filename])
        target=(output/entry.filename).resolve()
        assert target.is_relative_to(output)
        target.parent.mkdir(parents=True,exist_ok=True)
        target.write_bytes(payload)
reference=Path('target/debug/examples/civic_reference.exe')
shown=subprocess.run([str(reference),'show',str(output),result['manifest']['label']],check=True,capture_output=True,text=True,encoding='utf-8')
assert shown.stdout.rstrip('\n')==result['manifest_json']
listed=subprocess.run([str(reference),'list',str(output)],check=True,capture_output=True,text=True,encoding='utf-8')
assert result['manifest']['label'] in listed.stdout
print('Civic package ZIP: CRC, exact four native files, safe paths, non-executable permissions and actual native Show/List readback passed.')
