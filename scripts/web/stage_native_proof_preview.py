"""Stage native proof orchestration after the engine, solver and checker previews."""
from pathlib import Path
import argparse
import hashlib
import json
import shutil


def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--destination', type=Path, default=Path('dist/wasm-lab-build/toolkit'))
    destination = parser.parse_args().destination.resolve()
    engine = destination / 'bisect_wasm.wasm'
    if not (destination / 'index.html').is_file() or not engine.is_file(): raise ValueError('An existing local toolkit engine preview is required')
    if sha(engine) != sha(Path('target/wasm32-unknown-unknown/release/bisect_wasm.wasm')): raise ValueError('Preview engine differs from the current tested engine artifact')
    solver = json.loads((destination / 'roundingsat/build-receipt.json').read_text(encoding='utf-8'))
    checker = json.loads((destination / 'proof-checker/proof-checker-build.json').read_text(encoding='utf-8'))
    dependencies = {'bisect_wasm.wasm': sha(engine)}
    for name in ['roundingsat.mjs', 'roundingsat.wasm']:
        file = destination / 'roundingsat' / name
        if sha(file) != solver['files'][name]['sha256']: raise ValueError('Staged solver checksum mismatch')
        dependencies['roundingsat/' + name] = sha(file)
    file = destination / 'proof-checker/bisect_proof_checker.wasm'
    if sha(file) != checker['wasm_sha256']: raise ValueError('Staged checker checksum mismatch')
    dependencies['proof-checker/bisect_proof_checker.wasm'] = sha(file)
    files = {name: Path('web/lab') / name for name in ['native-proof.html', 'native-proof-pipeline.js', 'native-proof-worker.js', 'native-proof-page.js', 'native-proof-session.js', 'native-proof-session-worker.js', 'native-external-certificate.js', 'native-external-tree.js']}
    for name, file in files.items(): shutil.copy2(file, destination / name)
    for name in ['native-tree.html', 'native-tree-bundle.js', 'native-tree-worker.js', 'native-tree-page.js', 'native-tree-plan.js', 'native-tree-generate.js', 'native-model-prepare.js']:
        file = Path('web/lab') / name; files[name] = file; shutil.copy2(file, destination / name)
    (destination / 'native-proof-build.json').write_text(json.dumps({'schema': 'bisect-native-proof-preview-v1',
        'files': {name: {'sha256': sha(file), 'bytes': file.stat().st_size} for name, file in files.items()},
        'wasm_dependencies': dependencies, 'scope': 'Native split/tree verification, native plan/audit delivery and supplied-geometry native mapping. Automatic tree generation uses shared native discovery and verified-parent derivation, stops on incomplete proofs, and has small fixture coverage; statewide proof capacity remains unestablished.'}, indent=2) + '\n', encoding='utf-8')
    print(destination / 'native-proof.html')


if __name__ == '__main__': main()
