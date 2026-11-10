import base64
import hashlib
import io
import json
from pathlib import Path
import zipfile

root = Path('target/native-source-batch')
jobs = {job['scope']['state']: job for job in json.loads((root / 'jobs.json').read_text(encoding='utf-8'))}
checked = 0
for path in [root / 'outputs.zip', root / 'partial.zip', Path('target/native-source-batch-browser/outputs.zip')]:
    with zipfile.ZipFile(path) as outer:
        assert outer.testzip() is None
        names = outer.namelist()
        assert len(names) == len(set(names))
        assert all(not name.startswith('/') and '..' not in name.split('/') for name in names)
        receipt = json.loads(outer.read('batch-receipt.json'))
        for row in receipt['states']:
            state = row['state_code']
            assert state + '/prepared-geometry.json' not in names
            prep = row.get('source_preparation')
            if prep:
                assert outer.read(state + '/prepared-graph.json') == prep['graph_json'].encode('utf-8')
            if row['status'] != 'ok':
                assert state + '/prepared-run.zip' not in names
                continue
            with zipfile.ZipFile(io.BytesIO(outer.read(state + '/prepared-run.zip'))) as inner:
                assert inner.testzip() is None
                assert len(inner.namelist()) == len(set(inner.namelist()))
                assert all(not name.startswith('/') and '..' not in name.split('/') for name in inner.namelist())
                result = row['result']
                native = result['native_input']
                assert native == jobs[state]['native_preparation']
                assert json.loads(inner.read('native-inputs/recipe.json')) == native
                for key, filename in [('adjacency', 'adjacency.bin'), ('geoids', 'geoids.json'), ('centroids', 'centroids.json'), ('geometry', 'physical-geometry.json')]:
                    source = native['sources'].get(key)
                    if source:
                        data = base64.b64decode(source['base64'])
                        assert inner.read('native-inputs/' + filename) == data
                        assert hashlib.sha256(data).hexdigest() == prep['sources'][key]['sha256']
                assert inner.read('request-template.json') == jobs[state]['prepared_engine_template']['source']['text'].encode('utf-8')
                assert b'18446744073709551615' in inner.read('request-template.json')
                for name, field in [('plan.rplan', 'document_json'), ('context.rctx', 'context_json')]:
                    assert inner.read(name) == result['prepared']['export'][field].encode('utf-8')
                assert inner.read('accepted-replay.json') == result['prepared']['replay_artifact_json'].encode('utf-8')
                checked += 1
assert checked == 6
(root / 'zip-evidence.json').write_text(json.dumps(dict(nested_native_source_archives=checked,
    independent_crc_paths_and_bytes=True, original_binary_sources_match_native_hashes=True,
    exact_template_native_exports_and_replay=True, no_polygon_geometry_fabricated=True), indent=2) + '\n', encoding='utf-8')
print('Native source ZIP: six nested archives; exact original native bytes/hashes, recipe/template/exports/replay, CRC/paths and no fabricated polygon geometry passed.')
