"""Independent bound custom workflow/output artifact checks."""
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
root = Path('target/native-custom-state-workflow')
outputs = json.loads((root/'outputs.json').read_text(encoding='utf-8'))
canonical = lambda v: json.dumps(v, ensure_ascii=False, separators=(',', ':')).encode('utf-8')
sha = lambda b: hashlib.sha256(b).hexdigest()
assert len(outputs) == 3
for row in outputs:
    result = row['result']
    recipe = result['native_state']['recipe']
    binding = recipe['workflow_binding']
    assert sha(canonical(recipe)) == result['native_state']['source_sha256']
    assert sha(binding['source_command']['arguments_json'].encode('utf-8')) == binding['source_sha256']
    assert sha(binding['engine_command']['arguments_json'].encode('utf-8')) == binding['engine_sha256']
    assert sha(canonical(recipe['source_inputs']['preparation'])) == binding['source_override']['preparation_sha256']
    original = json.loads(binding['source_command']['arguments_json'])
    assert original[original.index('--version')+1] == 'vCustom'
    assert original[original.index('--output-dir')+1] == 'Never opened/custom/output'
    assert binding['settings']['layout'] == 'labeled' and binding['settings']['dpi'] == '72'
    compiled = result['native_state']['compilation']
    assert compiled['adjacency_override']['path_accessed'] is False
    assert compiled['native_command']['arguments'] == json.loads(recipe['command']['arguments_json']) + ['--adjacency='+binding['source_override']['adjacency_path_intent']]
    text = result['request_source']['source']['text']
    assert sha(text.encode('utf-8')) == result['prepared']['request_sha256']
    request = json.loads(text)['request']
    assert request['options']['seed'] == '18446744073709551615'
    assert request['graph']['geoids'] == ['ward-A', 'ward-B', 'ward-C', 'ward-D']
    assert request['graph']['county_ids'] == ['North', 'North', 'South', 'South']
    workflow = result['workflow']
    assert workflow['status'] == 'partial'
    analysis = next(s for s in workflow['stages'] if s['name'] == 'analysis')
    assert any(s['name'] == 'splits' and s['status'] == 'computed' for s in analysis['steps'])
    assert any(s['name'] == 'election' and s['status'] == 'unavailable' for s in analysis['steps'])
    splits = json.loads(base64.b64decode(workflow['files_base64']['analysis/splits.json']))
    assert splits['subdivision_identity'] == 'explicit-unit-county-map-v1'
    assert splits['counties']['legal_standard'] is None
    assert splits['counties']['total'] == 2
    effective = dict(zip(request['graph']['geoids'], request['graph']['county_ids']))
    identity = json.dumps(effective, ensure_ascii=False, separators=(',', ':'), sort_keys=True).encode('utf-8')
    assert splits['county_identity_sha256'] == sha(b'bisect-explicit-county-identity-v1\0' + identity)
    assert next(s for s in workflow['stages'] if s['name'] == 'map:districts')['status'] == 'completed'
    assert next(s for s in workflow['stages'] if s['name'] == 'report')['status'] == 'completed'
    plan = result['prepared']['export']['document_json'].encode('utf-8')
    context = result['prepared']['export']['context_json'].encode('utf-8')
    assert workflow['accepted_plan_source_sha256'] == sha(plan)
    assert workflow['accepted_context_source_sha256'] == sha(context)
    paths = [root]
    if not args.skip_browser:
        paths.append(Path('target/native-custom-state-workflow-browser'))
    for path in paths:
        with zipfile.ZipFile(io.BytesIO((path/(row['name']+'.workflow.zip')).read_bytes())) as z:
            assert len(z.namelist()) == len(set(z.namelist())) and z.testzip() is None
            for name, digest in workflow['file_sha256'].items():
                actual = z.read(name)
                assert actual == base64.b64decode(workflow['files_base64'][name], validate=True)
                assert sha(actual) == digest
                if name.endswith('.png'):
                    assert struct.unpack('>II', actual[16:24])[0] == 576
            assert z.read('plan.rplan') == plan and z.read('context.rctx') == context
        output = row['output']
        assert output['partition_execution_performed'] is False
        assert output['target_namespace'].startswith('vCustom/'+result['scope']['year']+'/plans/Custom_')
        with zipfile.ZipFile(io.BytesIO((path/(row['name']+'.output.zip')).read_bytes())) as z:
            prefix = output['target_namespace']+'/'
            assert len(z.namelist()) == len(set(z.namelist())) and z.testzip() is None
            for name, digest in output['file_sha256'].items():
                actual = z.read(prefix+name)
                assert actual == base64.b64decode(output['files_base64'][name], validate=True)
                assert sha(actual) == digest
            for name in workflow['files_base64']:
                assert z.read(prefix+name) == base64.b64decode(workflow['files_base64'][name], validate=True)
    archive = json.loads((root/(row['name']+'.source.bisect-workbench')).read_text(encoding='utf-8'))
    assert archive['result'] is None
    assert archive['files']['stateWorkflowBinding'] == binding
    assert archive['files']['tuiRunWorkflowSettings']['dpi'] == '72'
partial = json.loads((root/'partial.json').read_text(encoding='utf-8'))
assert partial['prepared'] == outputs[0]['result']['prepared'] and 'geometry' in partial['workflow_error']
assert json.loads((root/'map-only.json').read_text(encoding='utf-8'))['workflow']['status'] == 'completed'
print('PASS custom workflow artifacts: three native-bound workflow/output ZIPs, exact source/recipe/request/plan/context identities, explicit county identity hash and missing-election disclosure, maps/reports and accepted-plan retention')
