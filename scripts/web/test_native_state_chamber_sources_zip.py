"""Check chamber metadata, unchanged flags, counts and source bytes in ZIPs."""
import argparse
import base64
import hashlib
import io
import json
from pathlib import Path
import struct
import zipfile

parser = argparse.ArgumentParser()
parser.add_argument('--skip-browser', action='store_true')
args = parser.parse_args()
root = Path('target/native-state-chamber-sources')
outputs = json.loads((root / 'outputs.json').read_text(encoding='utf-8'))
evidence = json.loads((root / 'evidence.json').read_text(encoding='utf-8'))
assert [o['name'] for o in outputs] == evidence['cases']
assert len(outputs) in [4, 8]
canonical = lambda v: json.dumps(v, ensure_ascii=False, separators=(',', ':')).encode('utf-8')
digest = lambda b: hashlib.sha256(b).hexdigest()

def check(data, output):
    with zipfile.ZipFile(io.BytesIO(data)) as archive:
        assert archive.testzip() is None
        receipt = json.loads(archive.read('prepared-run-receipt.json'))
        assert receipt == output['result']
        recipe = json.loads(archive.read('state-flags/recipe.json'))
        assert recipe == output['request']['input']['native_state']
        identities = json.loads(archive.read('state-flags/identities.json'))
        assert digest(canonical(recipe)) == identities['source_sha256']
        assert recipe['source_scope_policy'] == 'match-native-flags'
        assert recipe['source_inputs']['settings']['chamber'] == receipt['scope']['chamber']
        assert receipt['scope']['chamber'] in ['house', 'senate']
        binding = recipe.get('workflow_binding')
        compiled = receipt['native_state']['compilation']
        expected_arguments = json.loads(recipe['command']['arguments_json'])
        if binding and binding.get('source_override'):
            # The reviewed engine command omits disk discovery intent. The
            # verified-source entry point restores that intent for native parsing
            # while reading only the selected in-memory binary and companions.
            assert binding['engine_command']['arguments_json'] == recipe['command']['arguments_json']
            path_intent = binding['source_override']['adjacency_path_intent']
            expected_arguments += ['--adjacency=' + path_intent]
            assert compiled['adjacency_override']['path_accessed'] is False
            for kind in ['source', 'engine']:
                assert digest(binding[kind + '_command']['arguments_json'].encode('utf-8')) == binding[kind + '_sha256']
        assert compiled['native_command']['arguments'] == expected_arguments
        assert 'state-flags/scope-binding.json' not in archive.namelist()
        exact = receipt['request_source']['source']['text']
        assert digest(exact.encode('utf-8')) == identities['request_sha256']
        request = json.loads(exact)
        assert json.loads(archive.read('request.json')) == request
        assert request['request']['options']['seed'] == '18446744073709551615'
        assert request['request']['options']['districts'] == receipt['scope']['districts']
        plan_bytes = archive.read('plan.rplan')
        assert plan_bytes == receipt['prepared']['export']['document_json'].encode('utf-8')
        assert archive.read('context.rctx') == receipt['prepared']['export']['context_json'].encode('utf-8')
        plan = json.loads(plan_bytes)
        assert plan['metadata']['chamber'] == receipt['scope']['chamber']
        assert plan['plan']['k'] == receipt['scope']['districts']
        if binding:
            assert json.loads(archive.read('state-flags/workflow-binding.json')) == binding
            assert archive.read('state-flags/original-workflow-arguments.json') == binding['source_command']['arguments_json'].encode('utf-8')
            assert digest(canonical(recipe['source_inputs']['preparation'])) == binding['source_override']['preparation_sha256']
        preparation = recipe['source_inputs']['preparation']
        if preparation['operation'] == 'prepare-native-engine-input':
            sources = preparation['input']['sources']
            assert sources == output['files']['nativeAdjacencyInputs']
            binary = base64.b64decode(sources['adjacency']['base64'], validate=True)
            assert binary[:4] == b'RADJ'
            version, vertices, edges = struct.unpack('<III', binary[4:16])
            assert version == 3 and vertices == len(request['request']['graph']['population'])
            assert sum(struct.unpack('<' + 'q' * vertices, binary[16:16+8*vertices])) == sum(request['request']['graph']['population'])
        if output['name'].endswith('inferred'):
            assert receipt['single_source_selection']['district_count']['allocation_source'] == 'policy'
            field = next(f for f in receipt['native_state']['compilation']['native_command']['fields'] if f['id'] == 'districts')
            assert field['source'] != 'command-line' and field['values'] == []

for output in outputs:
    check((root / (output['name'] + '.zip')).read_bytes(), output)
    source = json.loads((root / (output['name'] + '.source.bisect-workbench')).read_text(encoding='utf-8'))
    assert source['operation'] == 'run-native-selected-state-source'
    assert source['result'] is None
    assert source['files']['stateGraphInputs']['settings']['chamber'] == output['result']['scope']['chamber']
if not args.skip_browser:
    browser = Path('target/native-state-chamber-sources-browser')
    for output in outputs:
        check((browser / (output['name'] + '.zip')).read_bytes(), output)
    output = json.loads((browser / 'house-output.bisect').read_text(encoding='utf-8'))['result']
    assert output['partition_execution_performed'] is False
    assert base64.b64decode(output['files_base64']['plan.rplan']) == outputs[0]['result']['prepared']['export']['document_json'].encode('utf-8')
print(f'PASS House/Senate ZIPs: {len(outputs)} module/browser cases, exact chamber/unchanged flags/count provenance/source/recipe/request/plan/context identities and inert source projects')
