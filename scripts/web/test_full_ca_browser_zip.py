"""Check actual browser downloads independently against the native state oracle."""
import base64
import hashlib
import json
from pathlib import Path
import re
import sys
import zipfile

state = sys.argv[1] if len(sys.argv) > 1 else 'CA'
assert re.fullmatch('[A-Z]{2}', state)
root = Path('target/full-state-browser') / state if len(sys.argv) > 1 else Path('target/full-ca-browser')
districts = json.loads((root / 'browser-inputs.json').read_text(encoding='utf-8'))['districts'] if len(sys.argv) > 1 else 52
oracle = json.loads(Path(f'target/stream-state-source-batch/native-oracles/{state}-{districts}-topology.json').read_text(encoding='utf-8'))
with zipfile.ZipFile(root / 'native-run.zip') as archive:
    assert archive.testzip() is None
    assert len(archive.namelist()) == len(set(archive.namelist()))
    assert set(archive.namelist()) == {'run-receipt.json', 'plan.rplan', 'context.rctx', 'manifest.json', 'effective-config.yml', 'final_assignments.json'}
    assert json.loads(archive.read('run-receipt.json')) == oracle
    assert archive.read('plan.rplan') == oracle['practitioner_export']['document_json'].encode('utf-8')
    assert archive.read('context.rctx') == oracle['practitioner_export']['context_json'].encode('utf-8')
    assert archive.read('manifest.json') == oracle['report_manifest']['manifest_json'].encode('utf-8')
    assert archive.read('effective-config.yml') == oracle['effective_yaml'].encode('utf-8')
    assert json.loads(archive.read('final_assignments.json')) == oracle['configured_run']['result']['assignments']
with zipfile.ZipFile(root / 'native-workflow.zip') as archive:
    assert archive.testzip() is None
    assert len(archive.namelist()) == len(set(archive.namelist()))
    assert set(archive.namelist()) == set(oracle['workflow']['files_base64']) | {'workflow-receipt.json'}
    assert json.loads(archive.read('workflow-receipt.json')) == {key: value for key, value in oracle['workflow'].items() if key != 'files_base64'}
    for name, encoded in oracle['workflow']['files_base64'].items():
        assert archive.read(name) == base64.b64decode(encoded, validate=True)
evidence = {'state': state, 'native_run_workflow_exact': True, 'browser_source_preparation': True, 'geometry_profile': 'display-topology', 'files': {}}
for name in ['handoff.bisect-workbench', 'run.bisect-workbench', 'native-run.zip', 'native-workflow.zip']:
    data = (root / name).read_bytes()
    evidence['files'][name] = {'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()}
(root / 'evidence.json').write_text(json.dumps(evidence, indent=2) + '\n', encoding='utf-8')
print(f'Full {state} browser ZIPs: independent CRCs, complete native Run receipt, exact plan/context/manifest and every workflow byte passed.')
