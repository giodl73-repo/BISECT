import json
from pathlib import Path, PurePosixPath
from zipfile import ZipFile

archives = files = 0
for archive in sorted(Path('target/native-plan-import').glob('*/import.zip')) + sorted(Path('target/native-plan-import-browser').glob('*.zip')):
    source = archive.parent / 'files.json' if archive.name == 'import.zip' else archive.with_name(archive.stem + '-files.json')
    expected = json.loads(source.read_text(encoding='utf-8'))
    with ZipFile(archive) as package:
        assert package.testzip() is None
        assert len(package.namelist()) == len(set(package.namelist()))
        assert set(package.namelist()) == set(expected)
        assert len(expected) in (2, 3)
        for name, text in expected.items():
            assert not PurePosixPath(name).is_absolute() and '..' not in PurePosixPath(name).parts
            assert package.read(name) == text.encode('utf-8'), (archive, name)
            files += 1
    archives += 1
assert archives >= 21
Path('target/native-plan-import/zip-evidence.json').write_text(json.dumps({'archives': archives, 'files': files, 'crc_safe_paths_and_exact_bytes': True}, indent=2) + '\n', encoding='utf-8')
for receipt in [Path('target/native-plan-import/evidence.json'), Path('target/native-plan-import-browser/evidence.json')]:
    evidence = json.loads(receipt.read_text(encoding='utf-8'))
    evidence['zip_checks_pending'] = False
    evidence['independent_zip_evidence'] = 'target/native-plan-import/zip-evidence.json'
    receipt.write_text(json.dumps(evidence, indent=2) + '\n', encoding='utf-8')
print(f'Native plan import ZIPs: {archives} archives/{files} exact files, independent CRC and safe-path checks passed.')
