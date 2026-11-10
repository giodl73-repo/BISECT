"""Independent readback of native output produced by the streamed source pipeline."""
from pathlib import Path
import hashlib
import io
import json
import zipfile
import sys

root = Path('target/stream-native-output')
sha = lambda data: hashlib.sha256(data).hexdigest()
count = 0
for filename, expected in [('good.json', 2), ('national.json', 50)]:
    run = json.loads((root / filename).read_text(encoding='utf-8'))
    base = Path(run['base']).resolve()
    assert base.is_relative_to(root.resolve())
    rows = run['receipt']['states']
    assert len(rows) == expected
    for row in rows:
        assert row['native_output']['status'] == 'completed'
        target = (base / row['native_output']['target_namespace']).resolve()
        assert target.is_relative_to(base)
        assert not (target / 'manifest.tmp').exists()
        receipt = json.loads((target / 'browser/install-receipt.json').read_text(encoding='utf-8'))
        for entry in receipt['files']:
            file = (target / entry['path']).resolve()
            assert file.is_relative_to(target)
            data = file.read_bytes()
            assert len(data) == entry['byte_length'] and sha(data) == entry['sha256']
        plan = json.loads((target / 'plan.rplan').read_text(encoding='utf-8'))
        assignments = json.loads((target / 'data/final_assignments.json').read_text(encoding='utf-8'))
        assert plan['plan']['units']['state'] == row['state']
        assert plan['plan']['k'] == 1
        assert [assignments[str(i)] for i in range(len(plan['plan']['assignment']))] == [d + 1 for d in plan['plan']['assignment']]
        marker = json.loads((base / row['state'] / 'completed.json').read_text(encoding='utf-8'))
        assert marker['native_output'] == row['native_output']
        with zipfile.ZipFile(base / row['state'] / 'state-output.zip') as archive:
            assert archive.testzip() is None
            with zipfile.ZipFile(io.BytesIO(archive.read(row['state'] + '/prepared-run.zip'))) as inner:
                assert inner.testzip() is None
        count += 1

if '--skip-browser' not in sys.argv:
    browser = Path('target/stream-native-output-browser')
    project = json.loads((browser / 'completed.bisect').read_text(encoding='utf-8'))
    files = json.loads((browser / 'installed.json').read_text(encoding='utf-8'))
    for row in project['result']['states']:
        namespace = row['native_output']['target_namespace']
        assert namespace + '/manifest.tmp' not in files
        receipt = json.loads(bytes(files[namespace + '/browser/install-receipt.json']))
        for entry in receipt['files']:
            data = bytes(files[namespace + '/' + entry['path']])
            assert len(data) == entry['byte_length'] and sha(data) == entry['sha256']
print(f'PASS streamed native output independent readback: {count} local synthetic state installs, original output ZIPs; browser OPFS installs checked when enabled')
