"""Check custom-source provenance and exact compiled/exported artifacts."""
import base64
import hashlib
import json
from pathlib import Path

root = Path('target/native-adjacency-state')
evidence = json.loads((root / 'evidence.json').read_text(encoding='utf-8'))
assert evidence['native_comparison_performed']
assert evidence['independent_native_nway_matches'] == 4
assert evidence['explicit_prepared_runs'] == 4
assert evidence['paths_accessed'] is False and evidence['ui_connected'] is False
outputs = json.loads((root / 'outputs.json').read_text(encoding='utf-8'))
assert len(outputs) == 4
digest = lambda b: hashlib.sha256(b).hexdigest()
for name, output in zip(evidence['cases'], outputs):
    request, compiled, accepted = [output[k] for k in ['request', 'compiled', 'accepted']]
    assert request['input']['arguments'] == compiled['native_command']['arguments']
    assert compiled['execution_performed'] is False
    exact = compiled['prepared_request_json']
    assert digest(exact.encode('utf-8')) == compiled['prepared_request_sha256']
    assert compiled['prepared_request_sha256'] == accepted['request_sha256']
    graph = json.loads(exact)['request']['graph']
    for key, source in request['input']['source']['sources'].items():
        if source is not None:
            identity = compiled['native_adjacency_source']['sources'][key]
            raw = base64.b64decode(source['base64'], validate=True)
            assert identity['sha256'] == digest(raw)
            assert identity['bytes'] == len(raw) and identity['name'] == source['name']
    assert (root / (name + '.rplan')).read_bytes() == accepted['export']['document_json'].encode('utf-8')
    assert (root / (name + '.rctx')).read_bytes() == accepted['export']['context_json'].encode('utf-8')
    plan = json.loads(accepted['export']['document_json'])
    context = json.loads(accepted['export']['context_json'])
    assert plan['metadata']['jurisdiction'] == graph['state']
    assert plan['metadata']['chamber'] == request['input']['chamber']
    assert plan['plan']['units']['unit_kind'] == graph['unit_kind']
    assert str(plan['plan']['units']['year']) == graph['year']
    assert plan['plan']['units']['unit_ids'] == graph['geoids']
    assert json.loads(exact)['request']['options']['seed'] == '18446744073709551615'
    if graph['unit_kind'] in ['precinct', 'imported']:
        assert context['subdivisions']['county_ids'] == graph['county_ids']
        assert graph['geoids'] == ['ward-A', 'ward-B', 'ward-C', 'ward-D']
print('PASS adjacency State artifacts: four source-byte identities, compiled request hashes, custom unit/county/year/chamber identities, exact seed and byte-exact plan/context exports; UI remains pending')
