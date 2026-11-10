"""Independent custom-source ZIP, identity and exact-export verification."""
import argparse
import base64
import hashlib
import io
import json
from pathlib import Path
import zipfile

parser = argparse.ArgumentParser()
parser.add_argument('--skip-browser', action='store_true')
args = parser.parse_args()
root = Path('target/native-custom-state-sources')
outputs = json.loads((root/'outputs.json').read_text(encoding='utf-8'))
baseline = json.loads(Path('target/native-adjacency-state/outputs.json').read_text(encoding='utf-8'))[:3]
digest = lambda data: hashlib.sha256(data).hexdigest()
canonical = lambda value: json.dumps(value, ensure_ascii=False, separators=(',', ':')).encode('utf-8')
assert len(outputs) == 3
for output, native in zip(outputs, baseline):
    result = output['result']
    assert result['prepared'] == native['accepted']
    recipe = result['native_state']['recipe']
    assert recipe['source_scope_policy'] == 'match-native-adjacency-flags'
    assert digest(canonical(recipe)) == result['native_state']['source_sha256']
    assert json.loads(recipe['command']['arguments_json']) == native['request']['input']['arguments']
    assert recipe['source_inputs']['preparation']['input'] == native['request']['input']['source']
    assert result['single_source_selection']['scope_changed'] is False
    assert result['single_source_selection']['acquisition_performed'] is False
    assert result['native_state']['compilation']['adjacency_override']['path_accessed'] is False
    request_text = result['request_source']['source']['text']
    assert digest(request_text.encode('utf-8')) == result['native_state']['request_sha256']
    request = json.loads(request_text)['request']
    assert request['options']['seed'] == '18446744073709551615'
    assert request['graph']['geoids'] == ['ward-A', 'ward-B', 'ward-C', 'ward-D']
    assert request['graph']['county_ids'] == ['North', 'North', 'South', 'South']
    for key, source in recipe['source_inputs']['preparation']['input']['sources'].items():
        if source:
            receipt = result['source_preparation']['sources'][key]
            raw = base64.b64decode(source['base64'], validate=True)
            assert receipt['name'] == source['name'] and receipt['sha256'] == digest(raw)
    paths = [root/(output['name']+'.zip')]
    if not args.skip_browser:
        paths.append(Path('target/native-custom-state-sources-browser')/(output['name']+'.zip'))
    for path in paths:
        with zipfile.ZipFile(io.BytesIO(path.read_bytes())) as archive:
            assert len(archive.namelist()) == len(set(archive.namelist()))
            assert archive.read('request.json') == request_text.encode('utf-8')
            assert archive.read('plan.rplan') == native['accepted']['export']['document_json'].encode('utf-8')
            assert archive.read('context.rctx') == native['accepted']['export']['context_json'].encode('utf-8')
            assert json.loads(archive.read('state-flags/recipe.json')) == recipe
            assert json.loads(archive.read('prepared-run-receipt.json')) == result
            geometry = json.loads(archive.read('display-geometry.geojson'))
            assert geometry['state'] == result['scope']['state']
            assert [f['properties']['GEOID'] for f in geometry['features']] == request['graph']['geoids']
    archive = json.loads((root/(output['name']+'.source.bisect-workbench')).read_text(encoding='utf-8'))
    assert archive['result'] is None
print('PASS custom State ZIPs: three exact native exports, recipe/request/source hashes, custom IDs/counties, exact seed, matching display geometry and inert source archives')
