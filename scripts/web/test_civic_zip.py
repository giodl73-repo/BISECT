import json
from pathlib import Path
from zipfile import ZipFile

result = json.loads(Path('target/civic-wasm/browser-project.bisect').read_bytes())['result']
with ZipFile('target/civic-wasm/assessment.zip') as archive:
    assert archive.testzip() is None
    expected = {'assessment.json', 'conflicts.json', 'submission-1/normalized.csv', 'submission-2/normalized.csv'}
    assert set(archive.namelist()) == expected
    assert archive.read('conflicts.json').decode('utf-8') == result['conflicts_json']
    assert json.loads(archive.read('assessment.json')) == result
    for i, source in enumerate(result['inputs'], 1):
        assert archive.read(f'submission-{i}/normalized.csv') == source['normalized_csv'].encode('utf-8')
    assert all((entry.external_attr >> 16) & 0o111 == 0 for entry in archive.infolist())
print('Civic ZIP: exact entries, CRC, normalized/conflict/report bytes and non-executable permissions passed.')
