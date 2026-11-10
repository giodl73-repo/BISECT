"""Independently inspect native State package ZIPs without executing scripts."""
import hashlib
import json
from pathlib import Path, PurePosixPath
from zipfile import ZipFile

root = Path('target/state-practitioner-handoff')
cases = []
for project in sorted(root.glob('*-reproducibility.bisect')):
    packet = json.loads(project.read_text(encoding='utf-8'))
    prefix = project.name.removesuffix('-reproducibility.bisect')
    cases.append((root / (prefix + '-package.zip'), packet['result']))
assert len(cases) == 10, len(cases)
evidence = json.loads((root / 'evidence.json').read_text(encoding='utf-8'))
if evidence['original_ri_native_package_with_exact_max_u64_seed']:
    original = json.loads((root / 'original-ri-package.json').read_text(encoding='utf-8'))
    assert json.loads(original['files']['manifest.json'])['engine_seed'] == 18446744073709551615
    cases.append((root / 'original-ri-package.zip', original))
browser = Path('target/state-practitioner-browser')
expected = json.loads((browser / 'reproducibility-result.bisect').read_text(encoding='utf-8'))['result']
cases.append((browser / 'state-package.zip', expected))
files_checked = 0
for archive, result in cases:
    with ZipFile(archive) as package:
        assert package.testzip() is None, archive
        assert len(package.namelist()) == len(set(package.namelist())), archive
        assert set(package.namelist()) == set(result['files']), archive
        for name, text in result['files'].items():
            path = PurePosixPath(name)
            assert not path.is_absolute() and '..' not in path.parts, (archive, name)
            raw = package.read(name)
            assert raw == text.encode('utf-8'), (archive, name)
            assert hashlib.sha256(raw).hexdigest() == result['file_sha256'][name], (archive, name)
            if name == 'verify.sh':
                assert (package.getinfo(name).external_attr >> 16) & 0o111
            files_checked += 1
receipt = {'archives': len(cases), 'exact_files': files_checked,
           'crc_paths_native_hashes_and_script_permissions': True,
           'verification_scripts_executed': False}
(root / 'zip-evidence.json').write_text(json.dumps(receipt, indent=2) + '\n', encoding='utf-8')
print(f'State reproducibility ZIPs: {len(cases)} archives/{files_checked} exact files; CRC, hashes and script permissions pass; no script execution.')
