"""Check real-source prepared outputs independently of the JS ZIP writer."""
import base64
import hashlib
import json
from pathlib import Path
import zipfile

root = Path('target/prepared-census-real')
evidence = json.loads((root / 'evidence.json').read_text(encoding='utf-8'))
assert len(evidence['cases']) == 3
for case in evidence['cases']:
    with zipfile.ZipFile(root / (case['unit_kind'] + '-prepared-run.zip')) as archive:
        assert archive.testzip() is None
        names = archive.namelist()
        assert len(names) == len(set(names))
        assert all(not name.startswith('/') and '..' not in name.split('/') for name in names)
        receipt = json.loads(archive.read('prepared-run-receipt.json'))
        template = archive.read('request-template.json')
        assert hashlib.sha256(template).hexdigest() == case['template_sha256']
        assert template == receipt['template_source']['source']['text'].encode('utf-8')
        request = archive.read('request.json')
        assert request == receipt['request_source']['source']['text'].encode('utf-8')
        assert b'"seed":18446744073709551615' in request
        assert json.loads(request)['request']['options']['seed'] == 2**64 - 1
        assert archive.read('accepted-replay.json') == receipt['prepared']['replay_artifact_json'].encode('utf-8')
        assert archive.read('display-geometry.geojson') == base64.b64decode(receipt['geometry']['base64'])
        for filename, field in [('plan.rplan', 'document_json'), ('context.rctx', 'context_json')]:
            assert archive.read(filename) == receipt['prepared']['export'][field].encode('utf-8')
        assert len(json.loads(archive.read('final_assignments.json'))) == case['units']
        assert hashlib.sha256(archive.read('display-geometry.geojson')).hexdigest() == case['geometry_sha256']
(root / 'zip-evidence.json').write_text(json.dumps(dict(original_state_archives=3,
    total_units=sum(c['units'] for c in evidence['cases']), independent_crc_paths_and_native_export_bytes=True,
    numeric_max_u64_token_intact=True, exact_original_template_and_accepted_replay=True), indent=2) + '\n', encoding='utf-8')
print('Original Census ZIPs: three RI resolutions, exact u64/template/replay/native exports, geometry hashes and CRC/path checks passed.')
