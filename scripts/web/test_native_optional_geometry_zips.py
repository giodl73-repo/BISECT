"""Independently verify original sources, missing metadata and native exports."""
import base64
import hashlib
import io
import json
from pathlib import Path
from zipfile import ZipFile

root = Path('target/native-optional-geometry-browser')
checked = 0
with ZipFile(root / 'sources.zip') as package:
    assert package.testzip() is None
    manifest = json.loads(package.read('bundle.json'))
    assert set(package.namelist()) == set(manifest['files']) | {'bundle.json'}
    for name, identity in manifest['files'].items():
        data = package.read(name)
        assert identity == {'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()}
        checked += 1
    graph_bytes = package.read('sources/graph.json')
    graph = json.loads(graph_bytes)
    for key in ['areas', 'exterior_perimeters', 'centroids']:
        assert graph[key] == []
    assert len(graph['geoids']) == 250 and sum(graph['population']) == 1097379
with ZipFile(root / 'batch.zip') as outer:
    assert outer.testzip() is None
    receipt = json.loads(outer.read('batch-receipt.json'))
    row = receipt['states'][0]
    assert row['status'] == 'ok' and row['state_code'] == 'RI'
    assert 'RI/prepared-geometry.json' not in outer.namelist()
    assert outer.read('RI/prepared-graph.json') == graph_bytes
    with ZipFile(io.BytesIO(outer.read('RI/prepared-run.zip'))) as package:
        assert package.testzip() is None
        names = package.namelist()
        assert len(names) == len(set(names))
        assert all(not name.startswith('/') and '..' not in name.split('/') for name in names)
        recipe = json.loads(package.read('native-inputs/recipe.json'))
        assert recipe['require_geometry'] is False
        assert recipe['sources']['centroids'] is None and recipe['sources']['geometry'] is None
        assert 'native-inputs/centroids.json' not in names
        assert 'native-inputs/physical-geometry.json' not in names
        assert 'display-geometry.geojson' not in names
        for key, filename in [('adjacency', 'adjacency.bin'), ('geoids', 'geoids.json')]:
            data = package.read('native-inputs/' + filename)
            assert data == base64.b64decode(recipe['sources'][key]['base64'])
            assert hashlib.sha256(data).hexdigest() == row['source_preparation']['sources'][key]['sha256']
            checked += 1
        assert package.read('native-inputs/adjacency.bin')[:8] == b'RADJ\x02\x00\x00\x00'
        output = row['result']['prepared']['export']
        assert package.read('plan.rplan').decode('utf-8') == output['document_json']
        assert package.read('context.rctx').decode('utf-8') == output['context_json']
        assert package.read('accepted-replay.json').decode('utf-8') == row['result']['prepared']['replay_artifact_json']
        checked += 3
Path('target/native-optional-geometry/zip-evidence.json').write_text(json.dumps({
    'archives_including_nested': 3, 'original_or_native_exact_files': checked,
    'crc_paths_hashes_and_original_radj_v2_verified': True,
    'missing_metadata_and_display_geometry_not_fabricated': True,
    'script_or_engine_execution': False,
}, indent=2) + '\n', encoding='utf-8')
print(f'Optional geometry ZIPs: three archives, {checked} exact files; original v2 bytes, metadata absence and native exports verified.')
