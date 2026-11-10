"""Build a byte-bound opt-in offline app shell and independent state/map packs."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import tempfile


def write_generated(path, data):
    # Preview builders can hardlink a retained package. Replace the generated
    # file rather than writing through a link into the retained base.
    with tempfile.NamedTemporaryFile(dir=path.parent, delete=False) as stream:
        temporary = Path(stream.name)
        stream.write(data)
    try:
        os.replace(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)


def build(root):
    catalog = json.loads((root / 'catalog.json').read_text(encoding='utf-8'))
    geometries = catalog['geometries']
    stale_flags = {}
    for state in catalog['catalog']['states']:
        for year in state['years']:
            scope = f"{state['code']}:{year['year']}"
            if year.get('geometry') is not (scope in geometries):
                if year.get('geometry') is not False or scope not in catalog['graphs'] or scope not in geometries:
                    raise ValueError('Catalog geometry availability disagrees with geometry index: ' + scope)
                stale_flags[scope] = year
    refreshed_catalog = None
    if stale_flags:
        # Older exported packages already shipped complete offline state packs
        # but inherited false display flags from the smaller saved catalog.
        # Permit just that upgrade when the copied offline index binds each
        # affected graph and map. A fresh inconsistent export still fails.
        prior_path = root / 'offline-manifest.json'
        prior = json.loads(prior_path.read_text(encoding='utf-8')) if prior_path.is_file() else {}
        for scope, year in stale_flags.items():
            refs = [catalog['graphs'][scope]['graph_ref'], geometries[scope]]
            if (prior.get('schema_version') != 'bisect-offline-package-v1'
                or prior.get('states', {}).get(scope) != refs
                or any(prior.get('assets', {}).get(ref, {}).get('sha256') != catalog['assets'][ref] for ref in refs)):
                raise ValueError('Catalog geometry availability disagrees with geometry index: ' + scope)
            year['geometry'] = True
        refreshed_catalog = (json.dumps(catalog, indent=2, ensure_ascii=False) + '\n').encode('utf-8')
    assets, states = {}, {}

    def record(name):
        path = root / name
        if path.is_symlink() or not path.resolve().is_relative_to(root.resolve()):
            raise ValueError('Offline asset escapes output: ' + name)
        data = refreshed_catalog if name == 'catalog.json' and refreshed_catalog is not None else path.read_bytes()
        assets[name] = {'sha256': hashlib.sha256(data).hexdigest(), 'bytes': len(data)}

    # Source files copied by the exporter; generated proof pages and user/provider
    # outputs are excluded. Include the complete toolkit module/worker closure.
    source_names = {p.name for p in Path('web/lab').iterdir() if p.is_file()}
    shell = []
    for directory in ['', 'toolkit/']:
        for p in sorted((root / directory).iterdir()):
            if p.is_file() and (p.name in source_names or p.name in ['index.html', 'bisect_wasm.wasm', 'catalog.json']):
                name = directory + p.name
                record(name)
                shell.append(name)
    for directory in ['toolkit/fixtures', 'toolkit/roundingsat', 'toolkit/proof-checker', 'c-metis', 'toolkit/c-metis']:
        if not (root / directory).is_dir():
            continue
        for p in sorted((root / directory).rglob('*')):
            if p.is_file():
                name = p.relative_to(root).as_posix()
                record(name)
                shell.append(name)
    for engine in ['bisect_wasm.wasm', 'toolkit/bisect_wasm.wasm']:
        if assets.get(engine, {}).get('sha256') != catalog.get('wasm_sha256'):
            raise ValueError('Offline engine differs from catalog identity: ' + engine)
    for scope, graph in catalog['graphs'].items():
        paths = [graph['graph_ref'], geometries[scope]]
        for name in paths:
            record(name)
            if assets[name]['sha256'] != catalog['assets'][name]:
                raise ValueError('Catalog hash disagrees: ' + name)
        states[scope] = paths
    version = hashlib.sha256(json.dumps({'assets': assets, 'shell': shell, 'states': states}, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
    result = {'schema_version': 'bisect-offline-package-v1', 'version': version, 'shell': shell, 'states': states, 'assets': assets}
    if refreshed_catalog is not None:
        write_generated(root / 'catalog.json', refreshed_catalog)
        print(f'Refreshed {len(stale_flags)} legacy catalog geometry flags')
    write_generated(root / 'offline-manifest.json', (json.dumps(result, indent=2) + '\n').encode('utf-8'))
    print(f'Offline manifest: {len(shell)} shell files, {len(states)} state/year packs, version {version[:12]}')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('root', type=Path)
    build(parser.parse_args().root)
