"""Read streamed multi-year source/output bundles independently of JS writers."""
import hashlib
import io
import json
from pathlib import Path
import sys
import zipfile


def check(base, result, original2000=False):
    assert json.loads((base / 'run-years-receipt.json').read_text(encoding='utf-8')) == result
    assert [r['year'] for r in result['years']] == ['2020', '2010', '2000']
    assert result['exit_code'] == 0
    assert all(row['status'] == 'completed' for row in result['years'])
    for year in result['years']:
        child = year['result']
        folder = base / year['year'] / child['output_folder']
        assert json.loads((folder / 'batch-receipt.json').read_text(encoding='utf-8')) == child
        state = child['states'][0]
        marker = json.loads((folder / 'VT/completed.json').read_text(encoding='utf-8'))
        assert marker['native_output']['status'] == 'completed'
        for role, kind in [('tiger', 'tiger'), ('pl94', 'pl94171')]:
            actual = (folder / f'VT/{kind}.zip').read_bytes()
            expected = Path(f'target/census-2000-sources/VT/{kind}.zip') if original2000 and year["year"] == "2000" else Path(f'target/stream-native-run-years/fixtures/{year["year"]}-{role}.zip')
            assert actual == expected.read_bytes()
            with zipfile.ZipFile(io.BytesIO(actual)) as source:
                assert source.testzip() is None
        with zipfile.ZipFile(folder / 'VT/state-output.zip') as batch:
            assert batch.testzip() is None
            with zipfile.ZipFile(io.BytesIO(batch.read('VT/prepared-run.zip'))) as output:
                assert output.testzip() is None
                recipe = json.loads(output.read('run-flags/recipe.json'))
                identity = json.loads(output.read('run-flags/identities.json'))
                assert recipe['command_kind'] == 'run'
                assert json.loads(recipe['command']['arguments_json']) == result['native_run_command']['arguments']
                assert recipe['source_inputs']['preparation']['input']['pl94']['year'] == int(year['year'])
                compact = json.dumps(recipe, ensure_ascii=False, separators=(',', ':')).encode()
                assert hashlib.sha256(compact).hexdigest() == identity['source_sha256']
                assert hashlib.sha256(output.read('request.json')).hexdigest() == identity['request_sha256']
                plan = json.loads(output.read('plan.rplan'))['plan']
                assert plan['units']['state'] == 'VT' and plan['k'] == 1
                namespace = folder / state['native_output']['target_namespace']
                accepted = json.loads((namespace / 'browser/accepted-run.json').read_text(encoding='utf-8'))
                assert output.read('plan.rplan').decode() == accepted['prepared']['export']['document_json']
                assert output.read('context.rctx').decode() == accepted['prepared']['export']['context_json']
                assert not (namespace / 'manifest.tmp').exists()
                assert (namespace / 'manifest.json').is_file()
                install = json.loads((namespace / 'browser/install-receipt.json').read_text(encoding='utf-8'))
                for item in install['files']:
                    raw = (namespace / item['path']).read_bytes()
                    assert len(raw) == item['byte_length']
                    assert hashlib.sha256(raw).hexdigest() == item['sha256']


good = json.loads(Path('target/stream-native-run-years/accepted.json').read_text(encoding='utf-8'))
check(Path(good['base']), good['result'])
if '--skip-browser' not in sys.argv:
    original2000 = '--original-browser-2000' in sys.argv
    browser = Path('target/stream-native-run-years-browser' + ('-original-2000' if original2000 else ''))
    project = json.loads((browser / 'completed.bisect').read_text(encoding='utf-8'))
    check(browser / 'captured', project['result'], original2000)
print('PASS streamed Run years files: exact original ZIPs, native flags/source/request hashes, accepted plans/contexts, installation readback and three-vintage success receipt')
