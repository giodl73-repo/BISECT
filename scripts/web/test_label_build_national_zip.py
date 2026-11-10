"""Independent archive and positional-assignment checks for the full-state capacity run."""
import json
import zipfile
from pathlib import Path

root = Path('target/label-build-national')
evidence = json.loads((root / 'evidence.json').read_text(encoding='utf-8'))
assert evidence['summary'] == {'total': 50, 'succeeded': 50, 'failed': 0}
with zipfile.ZipFile(root / 'completed.zip') as archive:
    assert archive.testzip() is None
    paths = archive.namelist()
    assert len(paths) == 153 and len(set(paths)) == len(paths)
    index = json.loads(archive.read('runs/national_capacity/2020/index.json'))
    assert index['summary'] == evidence['summary'] and len(index['states']) == 50
    registry = json.loads(archive.read('.bisect'))
    assert registry['national_capacity']['built'] == ['2020']
    states = set()
    for path in paths:
        assert '..' not in path.split('/') and not path.startswith('/')
        assert not (archive.getinfo(path).external_attr >> 16) & 0o111
        if not path.endswith('/plan.rplan'):
            continue
        plan = json.loads(archive.read(path))['plan']
        context = json.loads(archive.read(path.replace('plan.rplan', 'context.rctx')))
        assignments = json.loads(archive.read(path.replace('plan.rplan', 'final_assignments.json')))
        assert plan['units']['unit_ids'] == context['units']['unit_ids']
        assert plan['units']['year'] == context['units']['year'] == 2020
        assert len(assignments) == len(plan['assignment'])
        for position, district in enumerate(plan['assignment']):
            assert assignments[str(position)] == district + 1
        states.add(plan['units']['state'])
    assert states == {row['state'] for row in evidence['states']}
print('Fifty-state ZIP: 153 entries, valid CRCs, 50 matching RPLAN/RCTX/assignment sets, registry/index and safe nonexecutable paths passed.')
