"""Hash-verified local staging and archive admission. Python 3.12+, stdlib only."""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import tempfile

REPO = Path(__file__).resolve().parents[2]


def sha(path):
    h = hashlib.sha256()
    with Path(path).open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            h.update(chunk)
    return h.hexdigest()


def write_json(path, value):
    Path(path).write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')


def relative(value):
    p = PurePosixPath(value)
    if not value or '\\' in value or ':' in value or p.is_absolute() or any(x in ('..', '.') for x in value.split('/')):
        raise ValueError(f'Unsafe relative path: {value}')
    return Path(*p.parts)


def local_directory(path):
    path = Path(os.path.abspath(path))
    for p in [path, *path.parents]:
        if p.is_symlink() or p.is_junction():
            raise ValueError(f'Workspace cannot use links: {p}')
    if not path.is_relative_to(REPO):
        raise ValueError('Workspace must be inside this checkout')
    path.mkdir(parents=True, exist_ok=True)
    return path


def checked_copy(source, target, digest, size):
    """Admit only complete matching bytes; never overwrite a different file."""
    source, target = Path(source), Path(target)
    if target.is_symlink() or target.is_junction():
        raise ValueError(f'Linked destination: {target}')
    if target.exists():
        if target.stat().st_size != size or sha(target) != digest:
            raise ValueError(f'Existing destination differs: {target}')
        return
    target.parent.mkdir(parents=True, exist_ok=True)
    fd, temp = tempfile.mkstemp(prefix='.copy-', suffix='.partial', dir=target.parent)
    try:
        with os.fdopen(fd, 'wb') as out, source.open('rb') as incoming:
            shutil.copyfileobj(incoming, out, 1024 * 1024)
            out.flush()
            os.fsync(out.fileno())
        if Path(temp).stat().st_size != size or sha(temp) != digest:
            raise ValueError(f'Checksum/size mismatch: {source.name}')
        # Same-directory hard-link admission is atomic and refuses an existing
        # destination. Only the temporary file is linked; source/cache files
        # are always copied. Finally removes the temporary name.
        os.link(temp, target)
        if sha(target) != digest:
            target.unlink()
            raise ValueError('Destination verification failed')
    finally:
        Path(temp).unlink(missing_ok=True)


def stage(selection, run_id, vault, apply=False):
    if not re.fullmatch(r'[a-z0-9][a-z0-9-]{0,79}', run_id):
        raise ValueError('Use a short lowercase run ID')
    spec = json.loads(Path(selection).read_text(encoding='utf-8'))
    if spec['schema'] != 'bisect-publication-selection-v1':
        raise ValueError('Unknown selection schema')
    work = local_directory(REPO / '.work/publication')
    cache = local_directory(work / 'cache')
    runs = local_directory(work / 'runs')
    run = runs / run_id
    if run.exists():
        raise FileExistsError(f'Use a new run ID: {run_id}')
    seen = set()
    for item in spec['files']:
        relative(item['source']); rel = relative(item['path'])
        if rel.parts[0] != 'inputs' or item['path'] in seen:
            raise ValueError('Inputs must be unique and under inputs/')
        seen.add(item['path'])
        if not re.fullmatch('[0-9a-f]{64}', item['sha256']) or item['bytes'] < 0:
            raise ValueError('Invalid digest or size')
    total = sum(x['bytes'] for x in spec['files'])
    required = total * 8 + 256 * 1024**2
    print(json.dumps({'files': len(seen), 'input_bytes': total, 'reserved_bytes': required,
                      'free_bytes': shutil.disk_usage(work).free, 'run': run_id}), flush=True)
    if shutil.disk_usage(work).free < required:
        raise OSError('Insufficient local space for staging, build and extraction')
    if not apply:
        return
    # Cache admission precedes run creation. Cached selections work without the vault.
    for item in spec['files']:
        source = Path(vault) / relative(item['source'])
        target = cache / item['sha256']
        checked_copy(source, target, item['sha256'], item['bytes'])
    run.mkdir()
    write_json(run / 'selection.json', spec)
    write_json(run / 'state.json', {'schema': 'bisect-publication-run-v1', 'status': 'staging'})
    for item in spec['files']:
        checked_copy(cache / item['sha256'], run / relative(item['path']), item['sha256'], item['bytes'])
    commit = subprocess.check_output(['git', '-c', f'safe.directory={REPO.as_posix()}', 'rev-parse', 'HEAD'], cwd=REPO, text=True).strip()
    write_json(run / 'state.json', {'schema': 'bisect-publication-run-v1', 'status': 'staged-local',
                                  'repository_commit': commit, 'selection_sha256': sha(selection)})
    print(f'Staged {run}', flush=True)


def archive(package, vault, release_id):
    package = Path(package)
    receipt_path = package.with_suffix('.verification.json')
    receipt = json.loads(receipt_path.read_text())
    if receipt.get('status') != 'verified' or receipt['zip_sha256'] != sha(package):
        raise ValueError('Package must pass extraction and rendering checks before archival')
    if not re.fullmatch(r'[a-z0-9][a-z0-9-]{0,79}', release_id):
        raise ValueError('Invalid release ID')
    vault = Path(vault)
    if not vault.is_dir():
        write_json(package.with_suffix('.archive.json'), {'status': 'verified-local-awaiting-archive'})
        print('Verified locally; archive drive unavailable')
        return
    destination = vault / 'artifacts/publication' / release_id
    for p in [destination, destination.parent, destination.parent.parent]:
        if p.is_symlink() or p.is_junction():
            raise ValueError('Archive destination must not redirect')
    for source in (package, receipt_path):
        checked_copy(source, destination / source.name, sha(source), source.stat().st_size)
    result = {'status': 'archived-verified', 'release_id': release_id,
              'package': package.name, 'zip_sha256': sha(package)}
    write_json(package.with_suffix('.archive.json'), result)
    # Existing archive evidence is never overwritten.
    report = destination / 'archive.json'
    if report.exists():
        if json.loads(report.read_text()) != result:
            raise ValueError('Existing archive receipt differs')
    else:
        write_json(report, result)
    print(json.dumps(result), flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    s = sub.add_parser('stage'); s.add_argument('--selection', type=Path, required=True)
    s.add_argument('--run-id', required=True); s.add_argument('--apply', action='store_true')
    a = sub.add_parser('archive'); a.add_argument('--package', type=Path, required=True); a.add_argument('--release-id', required=True)
    for p in (s, a): p.add_argument('--vault', type=Path)
    args = parser.parse_args()
    vault = args.vault or Path(json.loads((REPO / '.bisect-vault.json').read_text())['vault'])
    if args.command == 'stage': stage(args.selection, args.run_id, vault, args.apply)
    else: archive(args.package, vault, args.release_id)


if __name__ == '__main__':
    main()
