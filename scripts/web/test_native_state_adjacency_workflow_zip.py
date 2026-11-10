"""Independently inspect selected adjacency recipes and exact retained bytes."""
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
root = Path('target/native-state-adjacency-workflow')
fixture = json.loads(Path('scripts/web/fixtures/native-state-source.json').read_text(encoding='utf-8'))
digest = lambda value: hashlib.sha256(value).hexdigest()
canonical = lambda value: json.dumps(value, ensure_ascii=False, separators=(',', ':')).encode('utf-8')

def check(data):
    with zipfile.ZipFile(io.BytesIO(data)) as archive:
        assert archive.testzip() is None
        recipe = json.loads(archive.read('state-flags/recipe.json'))
        identities = json.loads(archive.read('state-flags/identities.json'))
        binding = json.loads(archive.read('state-flags/workflow-binding.json'))
        assert recipe['workflow_binding'] == binding
        assert digest(canonical(recipe)) == identities['source_sha256']
        for kind in ['source', 'engine']:
            command = binding[kind + '_command']['arguments_json'].encode('utf-8')
            assert digest(command) == binding[kind + '_sha256']
        assert archive.read('state-flags/original-workflow-arguments.json') == binding['source_command']['arguments_json'].encode('utf-8')
        assert '--adjacency' in json.loads(binding['source_command']['arguments_json'])
        assert not any(arg.startswith('--adjacency') for arg in json.loads(binding['engine_command']['arguments_json']))
        preparation = recipe['source_inputs']['preparation']
        assert digest(canonical(preparation)) == binding['source_override']['preparation_sha256']
        assert binding['source_override']['adjacency_path_intent'] == 'unopened/source/v3.adj.bin'
        assert preparation['input']['sources'] == fixture['nativeAdjacencyInputs']
        for key, source in fixture['nativeAdjacencyInputs'].items():
            if source is not None:
                assert base64.b64decode(preparation['input']['sources'][key]['base64'], validate=True) == base64.b64decode(source['base64'], validate=True)
        receipt = json.loads(archive.read('prepared-run-receipt.json'))
        request_text = receipt['request_source']['source']['text']
        assert digest(request_text.encode('utf-8')) == identities['request_sha256']
        request = json.loads(request_text)
        assert json.loads(archive.read('request.json')) == request
        assert request['request']['options']['seed'] == '18446744073709551615'
        expected = json.loads((root / 'accepted.json').read_text(encoding='utf-8'))['result']['prepared']['export']
        assert archive.read('plan.rplan') == expected['document_json'].encode('utf-8')
        assert archive.read('context.rctx') == expected['context_json'].encode('utf-8')

check((root / 'accepted.zip').read_bytes())
if not args.skip_browser:
    with zipfile.ZipFile('target/native-state-adjacency-workflow-browser/accepted.zip') as archive:
        check(archive.read('RI/prepared-run.zip'))
print('PASS adjacency workflow ZIPs: exact selected binary/companions, source/preparation/command/request hashes, original override path, max-u64 seed and plan/context bytes')
