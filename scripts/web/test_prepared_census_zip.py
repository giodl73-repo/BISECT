"""Independent byte/CRC checks for the prepared Census source fixtures."""
import hashlib
import io
import json
from pathlib import Path
import zipfile


def checked_zip(data):
    archive = zipfile.ZipFile(io.BytesIO(data))
    assert archive.testzip() is None
    names = archive.namelist()
    assert len(names) == len(set(names))
    assert all(not name.startswith('/') and '..' not in name.split('/') for name in names)
    return archive


root = Path('target/prepared-census-jobs')
checked = 0
for name in ['outputs.zip', 'large-profile-outputs.zip']:
    with checked_zip((root / name).read_bytes()) as outer:
        row = json.loads(outer.read('batch-receipt.json'))['states'][0]
        with checked_zip(outer.read('RI/prepared-run.zip')) as inner:
            result = json.loads(inner.read('prepared-run-receipt.json'))
            assert inner.read('request-template.json') == (root / 'template.json').read_bytes()
            assert inner.read('request.json') == result['request_source']['source']['text'].encode('utf-8')
            assert b'18446744073709551615' in inner.read('request.json')
            assert inner.read('accepted-replay.json') == result['prepared']['replay_artifact_json'].encode('utf-8')
            for filename, field in [('plan.rplan', 'document_json'), ('context.rctx', 'context_json')]:
                assert inner.read(filename) == result['prepared']['export'][field].encode('utf-8')
            preparation = json.loads(outer.read('RI/source-preparation.json'))
            for filename, field in [('prepared-graph.json', 'graph'), ('prepared-geometry.json', 'geometry')]:
                data = outer.read('RI/' + filename)
                assert hashlib.sha256(data).hexdigest() == preparation[field + '_sha256']
                if field + '_json' in preparation:
                    assert data == preparation[field + '_json'].encode('utf-8')
            if 'result' in row:
                assert row['result'] == result
            else:
                assert row['prepared_run_archive'] == 'RI/prepared-run.zip'
                assert row['prepared_run_receipt'] == 'prepared-run-receipt.json'
                assert 'native_run_archive' not in row
            checked += 1

disk = Path(Path('target/prepared-census-acquisition/disk-path.txt').read_text(encoding='utf-8').strip())
receipt = json.loads((disk / 'completed.json').read_text(encoding='utf-8'))
for artifact in receipt['artifacts']:
    assert Path(artifact['name']).name == artifact['name']
    data = (disk / artifact['name']).read_bytes()
    assert len(data) == artifact['byte_length']
    assert hashlib.sha256(data).hexdigest() == artifact['sha256']
assert (disk / 'tiger.zip').read_bytes() == Path('target/census-archive-tests/deflated.zip').read_bytes()
assert (disk / 'pl94171.zip').read_bytes() == Path('target/census-archive-tests/pl2020.zip').read_bytes()
with checked_zip((disk / 'state-output.zip').read_bytes()) as outer:
    with checked_zip(outer.read('RI/prepared-run.zip')) as inner:
        result = json.loads(inner.read('prepared-run-receipt.json'))
        assert inner.read('request-template.json') == result['template_source']['source']['text'].encode('utf-8')
        assert inner.read('request.json') == result['request_source']['source']['text'].encode('utf-8')
        checked += 1
evidence = dict(nested_prepared_archives=checked, independent_crc_path_and_byte_checks=True,
                disk_completion_hashes_match=True, exact_original_source_archives=True,
                exact_template_bound_request_native_exports_and_replay=True)
(root / 'zip-evidence.json').write_text(json.dumps(evidence, indent=2) + '\n', encoding='utf-8')
print(f'Prepared Census ZIP: {checked} nested archives; CRC/paths, exact source/template/native bytes and completed disk hashes passed.')
