"""Independently verify the exact native capture files delivered in the ZIP."""
import hashlib
import json
from pathlib import Path
import zipfile

root = Path('target/batch-round-capture-wasm')
result = json.loads((root / 'result.json').read_text(encoding='utf-8'))
with zipfile.ZipFile(root / 'captured.zip') as archive:
    assert archive.testzip() is None
    assert set(archive.namelist()) == set(result['files'])
    hashes = {item['path']: item for item in result['output_hashes']}
    for path, text in result['files'].items():
        expected = text.encode('utf-8')
        assert archive.read(path) == expected
        assert hashes[path]['bytes'] == len(expected)
        assert hashes[path]['sha256'] == hashlib.sha256(expected).hexdigest()
    for row in result['states']:
        capture = json.loads(archive.read(row['round_capture']['path']))
        index_text = capture['unit_index_json']
        assert hashlib.sha256(index_text.encode('utf-8')).hexdigest() == capture['unit_index_sha256']
        index = json.loads(index_text)
        plan = json.loads(archive.read(row['plan_path']))['plan']
        assert {str(i): unit for i, unit in enumerate(plan['units']['unit_ids'])} == index
        for event in capture['events']:
            text = event['assignments_json']
            assert hashlib.sha256(text.encode('utf-8')).hexdigest() == event['assignments_sha256']
            assignments = json.loads(text)
            assert len(assignments) == event['units']
            assert len(set(assignments.values())) == event['regions']
            if event['matches_returned_assignment']:
                assert {str(i): region + 1 for i, region in enumerate(plan['assignment'])} == assignments
print('Captured batch ZIP: independent CRC/file set/exact UTF-8/output hashes, native index order, event hashes/counts/final-label binding passed.')
