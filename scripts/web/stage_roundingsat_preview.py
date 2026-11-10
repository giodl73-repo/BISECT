"""Install the experimental external solver into an existing local toolkit preview."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--destination', type=Path, default=Path('dist/wasm-lab-build/toolkit'))
    args = parser.parse_args()
    destination = args.destination.resolve()
    if not (destination / 'index.html').is_file():
        raise ValueError('Destination must be an existing local toolkit preview')
    build = Path('target/roundingsat-wasm')
    receipt = json.loads((build / 'build-receipt.json').read_text(encoding='utf-8'))
    expected = {'roundingsat.wasm', 'roundingsat.mjs'}
    if receipt.get('schema') != 'bisect-roundingsat-wasm-build-v1' or set(receipt.get('files', {})) != expected:
        raise ValueError('Unsupported build receipt')
    source = Path('target/proof-toolchain/roundingsat')
    commit = subprocess.check_output(['git', '-C', str(source), 'rev-parse', 'HEAD'], text=True).strip()
    if commit != 'd4edbf7908a9bb951fd181940919e0f3ac7ab1ee' or commit != receipt['commit']:
        raise ValueError('License source revision mismatch')
    if subprocess.check_output(['git', '-C', str(source), 'status', '--porcelain'], text=True).strip():
        raise ValueError('License source checkout must be clean')
    for name in expected:
        if sha(build / name) != receipt['files'][name]['sha256'] or (build / name).stat().st_size != receipt['files'][name]['bytes']:
            raise ValueError('Solver artifact checksum mismatch: ' + name)
    files = {f'roundingsat/{name}': build / name for name in [*sorted(expected), 'build-receipt.json']}
    files.update({'roundingsat/LICENSE-RoundingSat.txt': source / 'LICENSE',
                  'roundingsat/LICENSE-Boost.txt': Path('target/proof-toolchain/boost/LICENSE_1_0.txt')})
    for name in ['proof-solver.html', 'proof-solver-core.js', 'proof-solver-page.js', 'proof-solver-worker.js']:
        files[name] = Path('web/lab') / name
    # Hash every input before writing; no path names come from imported projects.
    hashes = {name: {'sha256': sha(path), 'bytes': path.stat().st_size} for name, path in files.items()}
    for name, path in files.items():
        target = destination / name
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(path, target)
    (destination / 'roundingsat/package-receipt.json').write_text(json.dumps({
        'schema': 'bisect-roundingsat-preview-package-v1', 'files': hashes,
        'scope': 'One-model proof generation with a write-time byte budget. Results and retained prefixes remain unverified; concrete proof checking and native split/tree adoption are separate operations.'
    }, indent=2) + '\n', encoding='utf-8')
    print(destination / 'proof-solver.html')


if __name__ == '__main__':
    main()
