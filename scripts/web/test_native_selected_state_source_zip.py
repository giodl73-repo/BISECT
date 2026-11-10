"""Verify single-source scope/count provenance and original bytes independently."""
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
root = Path('target/native-selected-state-source')
outputs = json.loads((root / 'outputs.json').read_text(encoding='utf-8'))
evidence = json.loads((root / 'evidence.json').read_text(encoding='utf-8'))
assert len(outputs) == (3 if evidence['source_kinds'] == ['native'] else 5)
native_fixture = json.loads(Path('scripts/web/fixtures/native-state-source.json').read_text(encoding='utf-8'))
canonical = lambda v: json.dumps(v, ensure_ascii=False, separators=(',', ':')).encode('utf-8')
digest = lambda b: hashlib.sha256(b).hexdigest()
files = ['count-4.zip', 'count-1.zip', 'count-2-inferred.zip', 'census.zip', 'tract.zip']

def check(data, output):
    with zipfile.ZipFile(io.BytesIO(data)) as archive:
        assert archive.testzip() is None
        receipt = json.loads(archive.read('prepared-run-receipt.json'))
        assert receipt == output['result']
        recipe = json.loads(archive.read('state-flags/recipe.json'))
        identities = json.loads(archive.read('state-flags/identities.json'))
        assert recipe == output['request']['input']['native_state']
        if recipe['source_inputs']['preparation']['operation'] == 'prepare-native-engine-input':
            assert recipe['source_inputs']['preparation']['input']['sources'] == native_fixture['nativeAdjacencyInputs']
        assert digest(canonical(recipe)) == identities['source_sha256']
        scope = receipt['single_source_selection']
        assert scope['scope_changed'] is False and scope['acquisition_performed'] is False
        assert scope['scope'] == receipt['scope']
        count = scope['district_count']['districts']
        assert count == receipt['scope']['districts']
        exact = receipt['request_source']['source']['text']
        assert digest(exact.encode('utf-8')) == identities['request_sha256']
        request = json.loads(exact)
        assert json.loads(archive.read('request.json')) == request
        assert request['request']['options']['districts'] == count
        assert request['request']['options']['seed'] == '18446744073709551615'
        assert archive.read('plan.rplan') == receipt['prepared']['export']['document_json'].encode('utf-8')
        assert archive.read('context.rctx') == receipt['prepared']['export']['context_json'].encode('utf-8')
        assert 'graph_json' not in receipt['source_preparation']
        binding = recipe.get('workflow_binding')
        if binding:
            assert json.loads(archive.read('state-flags/workflow-binding.json')) == binding
            for kind in ['source', 'engine']:
                assert digest(binding[kind + '_command']['arguments_json'].encode('utf-8')) == binding[kind + '_sha256']
            assert archive.read('state-flags/original-workflow-arguments.json') == binding['source_command']['arguments_json'].encode('utf-8')
            assert digest(canonical(recipe['source_inputs']['preparation'])) == binding['source_override']['preparation_sha256']
        # Every selected source in the recipe remains byte-exact, not a numeric
        # JSON roundtrip of the native binary, geographic file or DBF.
        def sources(value):
            if isinstance(value, dict):
                if set(value) == {'name', 'base64'}:
                    yield value
                else:
                    for child in value.values():
                        yield from sources(child)
            elif isinstance(value, list):
                for child in value:
                    yield from sources(child)
        actual = list(sources(recipe['source_inputs']))
        expected = list(sources(output['request']['input']['native_state']['source_inputs']))
        assert len(actual) == len(expected) and actual
        for a, e in zip(actual, expected):
            assert a['name'] == e['name']
            assert base64.b64decode(a['base64'], validate=True) == base64.b64decode(e['base64'], validate=True)

for filename, output in zip(files, outputs):
    check((root / filename).read_bytes(), output)
if not args.skip_browser:
    check(Path('target/native-selected-state-source-browser/accepted.zip').read_bytes(), outputs[0])
    audit = json.loads(Path('target/native-selected-state-source-browser/audit-inputs.bisect').read_text(encoding='utf-8'))
    assert 'auditCertificateText' not in audit['files'] and 'auditLineageText' not in audit['files']
    assert base64.b64decode(audit['files']['auditPlanText']['base64']).decode('utf-8') == outputs[0]['result']['prepared']['export']['document_json']
print(f'PASS selected State source ZIPs: {len(outputs)} source/count cases, exact source/command/recipe/request identities, accepted plan/context, max-u64 seed and browser export/audit source retention')
