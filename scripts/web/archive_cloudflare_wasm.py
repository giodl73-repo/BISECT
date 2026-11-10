"""Archive a staged Wrangler package and verify it in a fresh extraction folder."""

import argparse
import gzip
import hashlib
import json
from pathlib import Path
import re
import shutil
import stat
import zipfile

from package_cloudflare_wasm import WORKER_FILES


def require(condition, message):
    if not condition:
        raise ValueError(message)


def pin(path):
    with path.open('rb') as stream:
        return {'bytes': path.stat().st_size, 'sha256': hashlib.file_digest(stream, 'sha256').hexdigest()}


def safe_name(name):
    pieces = name.split('/')
    return (len(name) <= 260 and all(re.fullmatch(r'[A-Za-z0-9_.-]+', piece) and
            piece not in ('.', '..') and not piece.endswith('.') and
            not re.fullmatch(r'(?i:con|prn|aux|nul|com[0-9]|lpt[0-9])', piece.split('.')[0]) for piece in pieces))


def tree(root, directory):
    start = root / directory
    require(start.is_dir(), f'Missing staged directory: {directory}')
    require(all(not p.is_symlink() and not p.is_junction() for p in [start, *start.parents] if p.is_relative_to(root)),
            f'Linked staged directory: {directory}')
    result = {}
    for item in sorted(start.rglob('*')):
        name = item.relative_to(root).as_posix()
        require(safe_name(name) and not item.is_symlink() and not item.is_junction() and
                item.resolve().is_relative_to(root), f'Invalid staged path: {name}')
        if item.is_dir():
            continue
        require(item.is_file(), f'Invalid staged file: {name}')
        result[name] = pin(item)
    return result


def inspect(root):
    staged = tree(root, 'assets')
    worker = tree(root, 'cloudflare')
    require(set(worker) == {'cloudflare/' + name for name in (*WORKER_FILES, 'index.mjs')}, 'Unexpected Worker files')
    modules = tree(root, 'web/lab')
    require(modules and all(re.fullmatch(r'web/lab/[A-Za-z0-9_.-]+\.js', name) for name in modules),
            'Unexpected Worker module files')
    staged.update(worker)
    staged.update(modules)
    for name in ('hosting-manifest.json', 'package.json', 'wrangler.json'):
        item = root / name
        require(item.is_file() and not item.is_symlink() and item.resolve().is_relative_to(root), f'Invalid {name}')
        staged[name] = pin(item)
    require(len({name.lower() for name in staged}) == len(staged), 'Case-colliding staged files')
    hosting = json.loads((root / 'hosting-manifest.json').read_bytes())
    require(hosting['schema_version'] == 'bisect-cloudflare-static-package-v1' and hosting['deployed'] is False,
            'Select a locally staged Cloudflare package')
    require({name.removeprefix('assets/'): identity for name, identity in staged.items() if name.startswith('assets/')}
            == hosting['assets'], 'Staged assets differ from the hosting manifest')
    config = json.loads((root / 'wrangler.json').read_bytes())
    require(config['assets']['directory'] == './assets' and config['main'] == './cloudflare/index.mjs' and
            config['name'] == hosting['worker_name'], 'Wrangler configuration differs from the staged layout')
    decoded = {}
    for name, route in hosting['gzip_routes'].items():
        key = 'assets/' + route['asset']
        require(safe_name(name) and name.endswith('bisect_wasm.wasm') and name not in hosting['assets'] and
                staged.get(key) == route['gzip'] and '/' + name in config['assets']['run_worker_first'],
                f'Invalid hosted WASM route: {name}')
        length = 0
        sha = hashlib.sha256()
        with gzip.open(root / key, 'rb') as stream:
            while chunk := stream.read(1024**2):
                length += len(chunk)
                require(length <= route['decoded']['bytes'], f'Oversized decoded WASM: {name}')
                sha.update(chunk)
        require({'bytes': length, 'sha256': sha.hexdigest()} == route['decoded'], f'Decoded WASM differs: {name}')
        decoded[name] = route['decoded']
    manifest_name = 'assets/offline-manifest.json'
    if manifest_name in staged:
        manifest = json.loads((root / manifest_name).read_bytes())
        require(all(decoded.get(name, hosting['assets'].get(name)) == expected
                    for name, expected in manifest['assets'].items()), 'Offline manifest does not describe the packaged application')
    return staged


def archive(package, output, extracted, receipt):
    require(package.is_dir() and not package.is_symlink() and not package.is_junction(), 'Select a real staged directory')
    package = package.resolve(strict=True)
    output, extracted, receipt = (p.resolve() for p in (output, extracted, receipt))
    require(output.suffix.lower() == '.zip' and all(not p.exists() and not p.is_relative_to(package)
            for p in (output, extracted, receipt)) and len({output, extracted, receipt}) == 3 and
            not output.is_relative_to(extracted) and not receipt.is_relative_to(extracted),
            'Use fresh output paths outside the staged package and extraction folder')
    staged = inspect(package)
    with zipfile.ZipFile(output, 'x', compression=zipfile.ZIP_DEFLATED, compresslevel=6) as bundle:
        for name, identity in sorted(staged.items()):
            info = zipfile.ZipInfo(name, (2000, 1, 1, 0, 0, 0))
            info.external_attr = (stat.S_IFREG | 0o644) << 16
            info.file_size = identity['bytes']
            info.compress_type = zipfile.ZIP_STORED if name.endswith('.gz') else zipfile.ZIP_DEFLATED
            with (package / name).open('rb') as source, bundle.open(info, 'w') as destination:
                shutil.copyfileobj(source, destination, 1024**2)
    extracted.mkdir()
    with zipfile.ZipFile(output) as bundle:
        require(bundle.namelist() == sorted(staged), 'Archive member list differs from the staged project')
        for info in bundle.infolist():
            name = info.filename
            expected = staged[name]
            require(safe_name(name) and info.file_size == expected['bytes'] and not info.is_dir(), f'Invalid ZIP member: {name}')
            dest = extracted / name
            dest.parent.mkdir(parents=True, exist_ok=True)
            require(dest.resolve().is_relative_to(extracted), f'ZIP member escapes extraction folder: {name}')
            with bundle.open(info) as source, dest.open('xb') as destination:
                shutil.copyfileobj(source, destination, 1024**2)
            require(pin(dest) == expected, f'Extracted bytes differ: {name}')
    require(inspect(extracted) == staged and inspect(package) == staged, 'Package changed during ZIP readback')
    record = {'status': 'passed-cloudflare-archive-readback', 'archive': pin(output), 'files': staged,
              'verified_file_count': len(staged), 'deployed': False}
    with receipt.open('x', encoding='utf8', newline='\n') as stream:
        json.dump(record, stream, indent=2, sort_keys=True)
        stream.write('\n')
    print(f"Verified {len(staged)} extracted files: {output.name} ({record['archive']['bytes']} bytes)")


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--package', type=Path, required=True)
    parser.add_argument('--zip', type=Path, required=True)
    parser.add_argument('--extract', type=Path, required=True)
    parser.add_argument('--receipt', type=Path, required=True)
    args = parser.parse_args()
    archive(args.package, args.zip, args.extract, args.receipt)
