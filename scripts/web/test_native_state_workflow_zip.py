"""Independently read workflow command provenance, maps and native output ZIPs."""
import base64
import hashlib
import io
import json
from pathlib import Path
import struct
import sys
import zipfile


def check(archive, result):
    with zipfile.ZipFile(archive) as z:
        assert z.testzip() is None
        recipe = json.loads(z.read('state-flags/recipe.json'))
        binding = json.loads(z.read('state-flags/workflow-binding.json'))
        assert recipe['workflow_binding'] == binding
        identities = json.loads(z.read('state-flags/identities.json'))
        assert hashlib.sha256(json.dumps(recipe, ensure_ascii=False, separators=(',', ':')).encode()).hexdigest() == identities['source_sha256']
        assert hashlib.sha256(z.read('request.json')).hexdigest() == identities['request_sha256']
        assert z.read('state-flags/original-workflow-arguments.json').decode() == binding['source_command']['arguments_json']
        for name in ['source', 'engine']:
            assert hashlib.sha256(binding[name + '_command']['arguments_json'].encode()).hexdigest() == binding[name + '_sha256']
        original = json.loads(binding['source_command']['arguments_json'])
        assert '--label' in original and '--manifest' in original and '--output-dir' in original
        assert json.loads(z.read('request.json'))['request']['options']['seed'] == '18446744073709551615'
        assert z.read('plan.rplan').decode() == result['prepared']['export']['document_json']
        assert z.read('context.rctx').decode() == result['prepared']['export']['context_json']
        assert json.loads(z.read('plan.rplan'))['metadata']['label'] == 'Workflow_Lab'


root = Path('target/native-state-workflow')
accepted = json.loads((root / 'accepted.json').read_text(encoding='utf-8'))['result']
check(root / 'accepted.zip', accepted)
output = json.loads((root / 'output.json').read_text(encoding='utf-8'))
prefix = 'vWorkflow/2020/plans/Workflow_Lab/'
with zipfile.ZipFile(root / 'native-output.zip') as z:
    assert z.testzip() is None
    assert all(name.startswith(prefix) for name in z.namelist())
    assert json.loads(z.read(prefix + 'manifest.json'))['label'] == 'Workflow_Lab'
    assert z.read(prefix + 'Workflow_Lab.rplan').decode() == accepted['prepared']['export']['document_json']
    png = z.read(prefix + 'maps/districts.png')
    assert png[:8] == b'\x89PNG\r\n\x1a\n'
    assert struct.unpack('>II', png[16:24]) == (576, 133)
    assert not any('manifest.tmp' in name for name in z.namelist())
if '--skip-browser' not in sys.argv:
    browser = Path('target/native-state-workflow-browser')
    run = json.loads((browser / 'completed.bisect').read_text(encoding='utf-8'))['result']['states'][0]['result']
    with zipfile.ZipFile(browser / 'accepted.zip') as z:
        check(io.BytesIO(z.read('RI/prepared-run.zip')), run)
    saved = json.loads((browser / 'output.bisect').read_text(encoding='utf-8'))['result']
    assert saved['target_namespace'] == prefix[:-1]
    assert json.loads(base64.b64decode(saved['files_base64']['manifest.json']))['label'] == 'Workflow_Lab'
print('PASS State workflow ZIPs: exact original/engine flags and hashes, max-u64 seed, plan/context, labeled native manifest and selected 72 DPI map')
