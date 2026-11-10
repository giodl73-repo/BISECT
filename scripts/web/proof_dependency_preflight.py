"""Read-only proof prerequisite validation; does not establish reproducible builds."""
from pathlib import Path
import argparse
import hashlib
import json
import subprocess

CHECKER_SOURCES = {
    'adapter_source_sha256': 'web/proof-checker/src/lib.rs',
    'model_preparation_source_sha256': 'web/proof-checker/src/model_prepare.rs',
    'cargo_lock_sha256': 'web/proof-checker/Cargo.lock',
    'cargo_manifest_sha256': 'web/proof-checker/Cargo.toml',
    'native_split_source_sha256': 'web/proof-checker/src/native_split.rs',
    'native_tree_source_sha256': 'web/proof-checker/src/native_tree.rs',
    'native_tree_export_source_sha256': 'web/proof-checker/src/tree_export.rs',
    'native_child_derivation_source_sha256': 'crates/bisect-ilp/src/certified_tree.rs',
}


def sha(file):
    return hashlib.sha256(file.read_bytes()).hexdigest()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def validate_checker(repo, build=None):
    repo = Path(repo)
    build = Path(build) if build is not None else repo / 'target/proof-checker-wasm-build/wasm32-unknown-unknown/release'
    receipt_file = build / 'proof-checker-build.json'
    receipt = json.loads(receipt_file.read_text(encoding='utf-8'))
    require(receipt.get('schema') == 'bisect-veripb-wasm-build-v1', 'Unsupported checker receipt')
    require(receipt.get('upstream_commit') == 'a50f0483d65aa019918d03ccc590b71dc23c666c', 'Checker revision mismatch')
    wasm = build / 'bisect_proof_checker.wasm'
    require(sha(wasm) == receipt.get('wasm_sha256') and wasm.stat().st_size == receipt.get('wasm_bytes'), 'Checker artifact mismatch')
    for field, name in CHECKER_SOURCES.items():
        require(sha(repo / name) == receipt.get(field), 'Stale checker source: ' + name)
    vendor = repo / 'third_party/veripb'
    require(sha(vendor / 'provenance.json') == receipt.get('vendor_provenance_sha256'), 'Checker provenance mismatch')
    for name in ['LICENSE-MIT', 'LICENSE-APACHE']:
        require((vendor / name).is_file(), 'Missing checker license: ' + name)
    return {'receipt_sha256': sha(receipt_file), 'wasm_sha256': sha(wasm), 'source_fields_checked': len(CHECKER_SOURCES)}


def validate_solver(repo, build=None):
    repo = Path(repo)
    build = Path(build) if build is not None else repo / 'target/roundingsat-wasm'
    receipt_file = build / 'build-receipt.json'
    receipt = json.loads(receipt_file.read_text(encoding='utf-8'))
    require(receipt.get('schema') == 'bisect-roundingsat-wasm-build-v1', 'Unsupported solver receipt')
    require(set(receipt.get('files', {})) == {'roundingsat.mjs', 'roundingsat.wasm'}, 'Unsupported solver file inventory')
    source = repo / 'target/proof-toolchain/roundingsat'
    commit = subprocess.check_output(['git', '--no-optional-locks', '-C', str(source), 'rev-parse', 'HEAD'], text=True).strip()
    require(commit == receipt.get('commit') == 'd4edbf7908a9bb951fd181940919e0f3ac7ab1ee', 'Solver revision mismatch')
    require(not subprocess.check_output(['git', '--no-optional-locks', '-C', str(source), 'status', '--porcelain'], text=True).strip(), 'Solver source checkout is dirty')
    for name, expected in receipt['files'].items():
        file = build / name
        require(sha(file) == expected.get('sha256') and file.stat().st_size == expected.get('bytes'), 'Solver artifact mismatch: ' + name)
    for file in [source / 'LICENSE', repo / 'target/proof-toolchain/boost/LICENSE_1_0.txt']:
        require(file.is_file(), 'Missing solver license: ' + str(file))
    return {'receipt_sha256': sha(receipt_file), 'files': {name: sha(build / name) for name in sorted(receipt['files'])}}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', type=Path, default=Path.cwd())
    args = parser.parse_args()
    print(json.dumps({'checker': validate_checker(args.repo), 'solver': validate_solver(args.repo), 'scope': 'Current recorded sources and local artifact identities only; no compilation, staging or runtime acceptance'}, indent=2))


if __name__ == '__main__':
    main()
