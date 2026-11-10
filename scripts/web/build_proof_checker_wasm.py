"""Build the in-memory VeriPB adapter and record its source/artifact identity."""
from pathlib import Path
import hashlib
import json
import os
import subprocess


def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    vendor = Path('third_party/veripb')
    provenance = json.loads((vendor / 'provenance.json').read_text(encoding='utf-8'))
    for name, hashes in provenance['files'].items():
        if sha(vendor / name) != hashes['vendored_sha256']:
            raise ValueError('Vendored file checksum mismatch: ' + name)
        if any(name.startswith(lib + '/src/') for lib in ['veripb-checker', 'veripb-formula', 'veripb-rules', 'veripb-propagator']) and hashes['upstream_sha256'] != hashes['vendored_sha256']:
            raise ValueError('Checking kernel must remain unchanged: ' + name)
    env = dict(os.environ, CARGO_INCREMENTAL='0')
    common = ['cargo', 'build', '--offline', '--locked', '--release', '-j', '1', '--manifest-path', 'web/proof-checker/Cargo.toml']
    subprocess.run([*common, '--target-dir', 'target/proof-checker-build'], env=env, check=True)
    env['CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS'] = '-C link-arg=--max-memory=536870912 -C link-arg=-zstack-size=8388608'
    subprocess.run([*common, '--lib', '--target', 'wasm32-unknown-unknown', '--target-dir', 'target/proof-checker-wasm-build'], env=env, check=True)
    wasm = Path('target/proof-checker-wasm-build/wasm32-unknown-unknown/release/bisect_proof_checker.wasm')
    receipt = {'schema': 'bisect-veripb-wasm-build-v1', 'checker': 'VeriPB 3.0.2', 'upstream_commit': provenance['commit'],
        'rustc': subprocess.check_output(['rustc', '--version'], text=True).strip(),
        'vendor_provenance_sha256': sha(vendor / 'provenance.json'),
        'adapter_source_sha256': sha(Path('web/proof-checker/src/lib.rs')),
        'model_preparation_source_sha256': sha(Path('web/proof-checker/src/model_prepare.rs')), 'cargo_lock_sha256': sha(Path('web/proof-checker/Cargo.lock')),
        'cargo_manifest_sha256': sha(Path('web/proof-checker/Cargo.toml')),
        'native_split_source_sha256': sha(Path('web/proof-checker/src/native_split.rs')),
        'native_tree_source_sha256': sha(Path('web/proof-checker/src/native_tree.rs')),
        'native_tree_export_source_sha256': sha(Path('web/proof-checker/src/tree_export.rs')),
        'native_child_derivation_source_sha256': sha(Path('crates/bisect-ilp/src/certified_tree.rs')),
        'wasm_sha256': sha(wasm), 'wasm_bytes': wasm.stat().st_size,
        'memory_limit_bytes': 536870912, 'stack_bytes': 8388608, 'plain_text_only': True,
        'refused_rules': ['external-output', 'timer', 'assertion-used', 'weakened-guarantees'],
        'scope': 'Single-formula checking, external split/tree certification and original-context native plan export with selected audit. Explicit source-preserving integer model preparation is available. Automatic generation and plan UI/maps are separate browser orchestration; global plan optimality is not claimed.'}
    (wasm.parent / 'proof-checker-build.json').write_text(json.dumps(receipt, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(receipt, indent=2))


if __name__ == '__main__': main()
