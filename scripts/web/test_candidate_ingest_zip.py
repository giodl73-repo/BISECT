import base64,json,subprocess
from pathlib import Path
from zipfile import ZipFile
root=Path('target/candidate-ingest-wasm');project=json.loads((root/'browser-project.bisect').read_bytes());r=project['result'];inputs=project['files']['candidateInputs'];m=r['manifest'];prefix=f"v1/civic_inputs/{m['label']}/";out=(root/'zip-readback').resolve()
expected={prefix+'candidate_race.csv':base64.b64decode(inputs['csv']['base64']),prefix+m['attestation_doc']:base64.b64decode(inputs['documents'][inputs['staged_path']]),**{prefix+n:t.encode() for n,t in r['files'].items()}}
with ZipFile(root/'package.zip') as z:
    assert z.testzip() is None
    assert set(z.namelist())==set(expected)
    for e in z.infolist():
        assert not ((e.external_attr>>16)&0o111)
        data=z.read(e);assert data==expected[e.filename]
        target=(out/e.filename).resolve();assert target.is_relative_to(out)
        target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(data)
folder=out/prefix;native_out=(root/'native-readback-output').resolve();binary=Path('target/debug/examples/candidate_reference.exe').resolve()
subprocess.run([str(binary),'candidate_race.csv',m['attestation_doc'],m['label'],m['state'],m['year'],m['submitter'],str(native_out)],cwd=folder,check=True,capture_output=True)
rebuilt=native_out/prefix
assert (rebuilt/'annotations.json').read_bytes()==r['files']['annotations.json'].encode()
manifest=json.loads((rebuilt/'manifest.json').read_bytes())
for field in ['n_annotations','n_curators','n_disputes','annotations_independently_verified','original_csv_sha256','attestation_doc_sha256','annotations_sha256']:assert manifest[field]==m[field]
print('Candidate ZIP: five exact native files, CRC, safe paths, non-executable permissions and actual native re-ingestion passed.')
