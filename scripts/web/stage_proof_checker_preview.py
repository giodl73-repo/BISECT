"""Stage the verified standalone VeriPB assets into an existing local preview."""
from pathlib import Path
import argparse
import hashlib
import json
import shutil
from proof_dependency_preflight import validate_checker


def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--destination', type=Path, default=Path('dist/wasm-lab-build/toolkit'))
    destination = parser.parse_args().destination.resolve()
    if not (destination / 'index.html').is_file(): raise ValueError('An existing local toolkit preview is required')
    validate_checker(Path.cwd())
    build = Path('target/proof-checker-wasm-build/wasm32-unknown-unknown/release')
    receipt = json.loads((build / 'proof-checker-build.json').read_text(encoding='utf-8'))
    wasm = build / 'bisect_proof_checker.wasm'
    if receipt.get('schema') != 'bisect-veripb-wasm-build-v1' or receipt.get('upstream_commit') != 'a50f0483d65aa019918d03ccc590b71dc23c666c' or sha(wasm) != receipt['wasm_sha256'] or wasm.stat().st_size != receipt['wasm_bytes']:
        raise ValueError('Proof checker artifact identity mismatch')
    vendor = Path('third_party/veripb')
    if sha(vendor / 'provenance.json') != receipt['vendor_provenance_sha256']: raise ValueError('Vendor provenance mismatch')
    files = {'proof-checker/bisect_proof_checker.wasm': wasm, 'proof-checker/proof-checker-build.json': build / 'proof-checker-build.json',
             'proof-checker/provenance.json': vendor / 'provenance.json',
             'proof-checker/LICENSE-MIT': vendor / 'LICENSE-MIT', 'proof-checker/LICENSE-APACHE': vendor / 'LICENSE-APACHE'}
    for name in ['proof-checker.html', 'proof-checker-core.js', 'proof-checker-worker.js', 'proof-checker-page.js', 'proof-solver-core.js']:
        files[name] = Path('web/lab') / name
    hashes = {name: {'sha256': sha(path), 'bytes': path.stat().st_size} for name, path in files.items()}
    for name, path in files.items():
        target = destination / name; target.parent.mkdir(parents=True, exist_ok=True); shutil.copy2(path, target)
    (destination / 'proof-checker/package-receipt.json').write_text(json.dumps({'schema': 'bisect-proof-checker-preview-package-v1', 'files': hashes}, indent=2) + '\n', encoding='utf-8')
    print(destination / 'proof-checker.html')


if __name__ == '__main__': main()
