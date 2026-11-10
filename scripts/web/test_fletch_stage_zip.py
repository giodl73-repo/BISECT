from pathlib import Path
import base64, hashlib, json, zipfile, sys
root=Path('target/fletch-stage-wasm')
count=0
for p in root.glob('*.zip'):
    project=json.loads(p.with_suffix('.bisect').read_text(encoding='utf-8'))
    result=project['result'];selected=project['files']['fletchStageInputs'];raw=selected['source'] if not selected.get('source_ref') else project['files']['fetchArchiveSource']['source'];source=base64.b64decode(raw['base64'])
    with zipfile.ZipFile(p) as archive:
        assert archive.namelist()==['cache-manifest.json',result['relative_path']]
        assert archive.testzip() is None
        assert archive.read(result['relative_path'])==source
        manifest=archive.read('cache-manifest.json')
        assert manifest==result['manifest_json'].encode('utf-8')
        entry=json.loads(manifest)['entries'][0]
        assert entry['sha256']=='sha256:'+hashlib.sha256(source).hexdigest()
        assert entry['bytes']==len(source)
        assert entry['fetched_at_ms']==int(result['fetched_at_ms_text'])
        assert entry['relative_path']==result['relative_path']
        assert entry['verified'] is True
    count+=1
assert count==11,count
print(f'Portable FLETCH cache ZIP: {count} independent original-object, exact-manifest/u64, native-path and CRC checks passed.')
if '--local-ri' in sys.argv:
    census=Path('target/census-zip-acquisition')
    project=json.loads((census/'ri2020-cache.bisect-workbench').read_text(encoding='utf-8'))
    raw=(census/'ri-pl94.zip').read_bytes();result=project['result']
    assert base64.b64decode(project['files']['fetchArchiveSource']['source']['base64'])==raw
    assert project['files']['fletchStageInputs']['source'] is None
    assert project['files']['fletchStageInputs']['source_ref']=='fetchArchiveSource'
    with zipfile.ZipFile(census/'ri2020-cache.zip') as archive:
        assert archive.testzip() is None
        assert archive.read(result['relative_path'])==raw
        assert archive.read('cache-manifest.json')==result['manifest_json'].encode('utf-8')
        assert json.loads(archive.read('cache-manifest.json'))['entries'][0]['sha256']=='sha256:'+hashlib.sha256(raw).hexdigest()
    print('Actual RI PL94 cache ZIP: original downloaded-fixture archive bytes, exact manifest, retained project reference and CRC passed.')
