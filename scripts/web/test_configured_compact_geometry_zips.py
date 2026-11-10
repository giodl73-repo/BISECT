"""Independent CRC/hash/settings inspection; imports or scripts are not executed."""
import hashlib
import json
from pathlib import Path, PurePosixPath
from zipfile import ZipFile

root = Path('target/configured-compact-geometry')
files_checked = 0
for index in [0, 10, 13]:
    with ZipFile(root / f'source-{index}.zip') as package:
        assert package.testzip() is None
        assert len(package.namelist()) == len(set(package.namelist()))
        manifest = json.loads(package.read('bundle.json'))
        assert set(manifest['files']) == set(package.namelist()) - {'bundle.json'}
        for name, identity in manifest['files'].items():
            path = PurePosixPath(name)
            assert not path.is_absolute() and '..' not in path.parts
            data = package.read(name)
            assert identity == {'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()}
            files_checked += 1
        plan = json.loads(package.read('plan.rplan'))
        recipe = json.loads(package.read('sources/run-settings.json'))
        assert recipe['settings']['compact_geometry'] == 'native-state-circular'
        assert plan['provenance']['producer']['engine_options']['compact_geometry'] == 'native-state-circular'
        assert package.read('sources/config.yml').decode('utf-8') == plan['provenance']['producer']['algorithm_configuration']['yaml_text']
        original = json.loads((root / f'configured-{index}.json').read_text(encoding='utf-8'))
        assert json.loads(package.read('sources/graph.json')) == original['input']['graph']
        assert recipe['execution_attempt'] == plan['provenance']['producer']['algorithm_configuration']['execution_attempt']
receipt = {'archives': 3, 'hashed_files': files_checked,
           'crc_hashes_original_yaml_graph_and_geometry_recipe_exact': True,
           'execution_performed': False}
(root / 'zip-evidence.json').write_text(json.dumps(receipt, indent=2) + '\n', encoding='utf-8')
print(f'Configured compact ZIPs: 3 archives/{files_checked} hashed files, CRC and exact YAML/graph/geometry settings verified.')
