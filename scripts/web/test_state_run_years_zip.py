"""Inspect year/state archives in memory without extracting paths to disk."""
from io import BytesIO
import json
from pathlib import Path
import zipfile

root = Path('target/state-run-years')
counts = {}
for name in ['all-years.zip', 'cancelled.zip']:
    with zipfile.ZipFile(root / name) as outer:
        assert outer.testzip() is None
        receipt = json.loads(outer.read('run-years-receipt.json'))
        assert receipt['requested_years'] == ['2020', '2010', '2000']
        seen = 0
        for row in receipt['years']:
            if 'output_archive' not in row:
                continue
            assert row['output_archive'] == row['year'] + '/state-outputs.zip'
            with zipfile.ZipFile(BytesIO(outer.read(row['output_archive']))) as child:
                assert child.testzip() is None
                batch = json.loads(child.read('batch-receipt.json'))
                assert batch['selection']['year'] == row['year']
                for state in batch['states']:
                    if state['status'] != 'ok':
                        continue
                    with zipfile.ZipFile(BytesIO(child.read(state['state_code'] + '/prepared-run.zip'))) as run:
                        assert run.testzip() is None
                        request = json.loads(run.read('request.json'))
                        plan = json.loads(run.read('plan.rplan'))
                        assert request['graph']['year'] == row['year']
                        assert str(plan['plan']['units']['year']) == row['year']
                        assert plan['plan']['units']['state'] == state['state_code']
                        assert 'context.rctx' in run.namelist()
                        seen += 1
        counts[name] = seen
assert counts == {'all-years.zip': 150, 'cancelled.zip': 1}, counts
(root / 'zip-evidence.json').write_text(json.dumps(counts, indent=2) + '\n', encoding='utf-8')
print('Multi-year ZIP:150 state archives and one retained cancelled-state archive; CRCs, year/state request/plan bindings and contexts passed.')
