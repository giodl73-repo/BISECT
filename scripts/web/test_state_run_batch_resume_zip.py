"""Independently compare resumed ZIP contents with native-verified full runs."""
from pathlib import Path
from zipfile import ZipFile
import sys

def compare(actual_path, reference_path, prefixes):
    with ZipFile(actual_path) as actual, ZipFile(reference_path) as reference:
        assert actual.testzip() is None, actual_path
        assert len(actual.namelist()) == len(set(actual.namelist()))
        names = [name for name in reference.namelist() if any(name.startswith(prefix) for prefix in prefixes)]
        assert names
        actual_names = [name for name in actual.namelist() if any(name.startswith(prefix) for prefix in prefixes)]
        assert set(actual_names) == set(names)
        for name in names:
            assert actual.read(name) == reference.read(name), (actual_path, name)
    return len(names)

root = Path('target/state-run-batch-resume')
count = compare(root / 'resumed.zip', Path('target/state-run-batch/complete.zip'), ['IA/', 'RI/', 'VT/'])
count += compare(root / 'source-resumed.zip', Path('target/state-source-batch/source.zip'), ['RI/'])
if '--browser' in sys.argv:
    count += compare(Path('target/state-run-batch-browser/resumed-outputs.zip'), Path('target/state-run-batch/complete.zip'), ['IA/', 'RI/', 'VT/'])
if '--recovery' in sys.argv:
    recovery = Path('target/state-run-batch-recovery')
    count += compare(recovery / 'resumed.zip', Path('target/state-run-batch/complete.zip'), ['IA/', 'RI/', 'VT/'])
    count += compare(recovery / 'source-verified.zip', Path('target/state-source-batch/source.zip'), ['RI/'])
    if '--real' in sys.argv:
        count += compare(recovery / 'original-ri-verified.zip', Path('target/state-source-batch/real-ri.zip'), ['RI/'])
    if '--browser' in sys.argv:
        count += compare(Path('target/state-run-batch-browser/recovered-outputs.zip'), Path('target/state-run-batch/complete.zip'), ['IA/', 'RI/', 'VT/'])
print(f'Resumed ZIP: CRCs, unique members and {count} exact state files/nested native archives match native-verified complete runs.')
