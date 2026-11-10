import base64
import json
from pathlib import Path, PurePosixPath
from zipfile import ZipFile

archives = files = directories = 0
for archive in sorted(Path('target/migration-base').glob('*/migration.zip')) + sorted(Path('target/migration-base-browser').glob('*.zip')):
    file_source = archive.parent / 'files.json' if archive.name == 'migration.zip' else archive.with_name(archive.stem + '-files.json')
    dir_source = archive.parent / 'dirs.json' if archive.name == 'migration.zip' else archive.with_name(archive.stem + '-dirs.json')
    expected = json.loads(file_source.read_text(encoding='utf-8'))
    expected_dirs = json.loads(dir_source.read_text(encoding='utf-8'))
    with ZipFile(archive) as package:
        assert package.testzip() is None
        assert len(package.namelist()) == len(set(package.namelist()))
        assert set(package.namelist()) == set(expected) | {d + '/' for d in expected_dirs}
        for name, encoded in expected.items():
            assert not PurePosixPath(name).is_absolute() and '..' not in PurePosixPath(name).parts
            assert package.read(name) == base64.b64decode(encoded, validate=True), (archive, name)
            files += 1
        for name in expected_dirs:
            info = package.getinfo(name + '/')
            assert info.is_dir() and info.file_size == 0 and info.CRC == 0
            directories += 1
    archives += 1
assert archives >= 15
Path('target/migration-base/zip-evidence.json').write_text(json.dumps({'archives': archives, 'files': files, 'directories': directories, 'exact_bytes_empty_directories_crc_safe_paths': True}, indent=2) + '\n', encoding='utf-8')
for receipt in [Path('target/migration-base/evidence.json'), Path('target/migration-base-browser/evidence.json')]:
    evidence = json.loads(receipt.read_text(encoding='utf-8'))
    evidence['zip_checks_pending'] = False
    evidence['independent_zip_evidence'] = 'target/migration-base/zip-evidence.json'
    receipt.write_text(json.dumps(evidence, indent=2) + '\n', encoding='utf-8')
print(f'Migration base ZIPs: {archives} archives/{files} exact files/{directories} explicit directories, CRCs and safe paths passed.')
