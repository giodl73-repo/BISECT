import json, pathlib, zipfile, subprocess
root=pathlib.Path('target/suite-assembly-wasm')
project=json.loads((root/'browser-project.bisect').read_bytes())
files=project['result']['files']
destination=(root/'zip-readback').resolve()
with zipfile.ZipFile(root/'portable.zip') as archive:
    assert archive.testzip() is None
    assert set(archive.namelist())==set(files)
    for info in archive.infolist():
        assert archive.read(info).decode('utf-8')==files[info.filename]
        assert not ((info.external_attr>>16)&0o111)
        path=(destination/info.filename).resolve()
        assert path.is_relative_to(destination)
        path.parent.mkdir(parents=True,exist_ok=True)
        path.write_bytes(archive.read(info))
reference=subprocess.run(['target/debug/examples/suite_assembly_reference.exe','--validate',str(destination/'portable')],capture_output=True,text=True,check=True)
assert json.loads(reference.stdout)['valid'] is True
print('Suite ZIP: exact paths, CRCs, original bytes, non-executable permissions and actual extracted-package CLI nesting readback passed.')
