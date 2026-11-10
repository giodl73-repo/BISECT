"""Capture current regular repository inputs without extension-based omissions.

Keep prior explicitly captured paths, including ignored proof sources. Add all
existing tracked files and nonignored rg-visible files. Output under target/.
This inventories source bytes; it does not establish runtime acceptance.
"""
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path, PurePosixPath
import argparse
import hashlib
import json
import os
import subprocess

EXCLUDED = {'.git', 'target', 'dist', 'node_modules'}

def collect(root, previous):
    root = root.resolve()
    prior = {row['path'] for row in previous}
    def listed(command):
        result = subprocess.run(command, cwd=root, capture_output=True,
                                env={**os.environ, 'GIT_OPTIONAL_LOCKS': '0'})
        if result.returncode not in (0, 1) or result.returncode == 1 and result.stderr:
            raise ValueError(result.stderr.decode(errors='replace'))
        return {name.decode('utf-8').replace('\\', '/') for name in result.stdout.split(b'\0') if name}
    names = prior | listed(['git', 'ls-files', '-z']) | listed([
        'rg', '--files', '--hidden', '--null', '-g', '!.git', '-g', '!target',
        '-g', '!dist', '-g', '!node_modules'])
    jobs = []
    for name in sorted(names):
        logical = PurePosixPath(name)
        if not name or logical.is_absolute() or '..' in logical.parts or ':' in name or '\\' in name:
            raise ValueError('Invalid source path: ' + name)
        # Prior target/ proof inputs are explicit dependencies, never dropped.
        if logical.parts[0] in EXCLUDED and name not in prior:
            continue
        file = root / name
        if file.is_symlink() or file.is_junction():
            raise ValueError('Source links require explicit review: ' + name)
        if not file.is_file():
            if name in prior:
                raise ValueError('Prior captured source is missing: ' + name)
            continue  # Deleted tracked files and gitlink directories are not inputs.
        resolved = file.resolve()
        if not resolved.is_relative_to(root):
            raise ValueError('Source escapes repository: ' + name)
        jobs.append((name, resolved))
    def pin(job):
        name, file = job
        before = file.stat()
        with file.open('rb') as stream:
            digest = hashlib.file_digest(stream, 'sha256').hexdigest()
        after = file.stat()
        if (before.st_size, before.st_mtime_ns) != (after.st_size, after.st_mtime_ns):
            raise ValueError('Source changed while capturing: ' + name)
        return {'path': name, 'sha256': digest, 'bytes': after.st_size}
    with ThreadPoolExecutor(max_workers=4) as pool:
        rows = list(pool.map(pin, jobs))
    assert prior <= {row['path'] for row in rows}
    return rows

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, required=True)
    parser.add_argument('--previous', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists():
        raise SystemExit('Preserve previous captures; choose a new output.')
    previous = json.loads(args.previous.read_text(encoding='utf-8-sig'))
    rows = collect(args.root, previous)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open('xb') as stream:
        stream.write(json.dumps(rows, indent=2).encode('utf-8'))
    print(json.dumps({'status': 'captured-current-repository-source-inputs',
                      'files': len(rows), 'bytes': sum(row['bytes'] for row in rows),
                      'prior_paths_preserved': len(previous)}))

if __name__ == '__main__':
    main()
