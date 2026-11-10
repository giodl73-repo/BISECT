import hashlib
import json
from pathlib import Path, PurePosixPath
from zipfile import ZipFile

archives = 0
files = 0
for archive in sorted(Path('target/export-folder').glob('*/export.zip')) + sorted(Path('target/export-folder-browser').glob('*.zip')):
    source = archive.parent / 'files.json' if archive.name == 'export.zip' else archive.with_name(archive.stem + '-files.json')
    expected = json.loads(source.read_text(encoding='utf-8'))
    with ZipFile(archive) as package:
        assert package.testzip() is None, archive
        assert len(package.namelist()) == len(set(package.namelist())), archive
        assert set(package.namelist()) == set(expected), archive
        for name, text in expected.items():
            assert not PurePosixPath(name).is_absolute() and '..' not in PurePosixPath(name).parts
            raw = package.read(name)
            assert raw == text.encode('utf-8'), (archive, name)
            assert hashlib.sha256(raw).hexdigest() == hashlib.sha256(text.encode('utf-8')).hexdigest()
            files += 1
    archives += 1
assert archives >= 14, archives
Path('target/export-folder/zip-evidence.json').write_text(json.dumps({'archives': archives, 'files': files, 'crc_safe_paths_and_exact_bytes': True}, indent=2) + '\n', encoding='utf-8')
for receipt in [Path('target/export-folder/evidence.json'), Path('target/export-folder-browser/evidence.json')]:
    evidence = json.loads(receipt.read_text(encoding='utf-8'))
    evidence['zip_checks_pending'] = False
    evidence['independent_zip_evidence'] = 'target/export-folder/zip-evidence.json'
    receipt.write_text(json.dumps(evidence, indent=2) + '\n', encoding='utf-8')
print(f'Native folder export ZIPs: {archives} archives/{files} exact files, independent CRC and safe-path checks passed.')
