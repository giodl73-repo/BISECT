"""Build/run the exact vendored C METIS as an isolated Emscripten sidecar.

This is a WP-02 feasibility experiment, not a production backend substitution.
No Cargo cache source, production manifest, worker or WASM artifact is edited.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tarfile

ROOT = Path(__file__).resolve().parents[2]
FIXTURE = 'scripts/web/fixtures/wasm-cffi-probe'
# Registry checksum, intentionally literal and checked against the crate archive.
METIS_SYS_CHECKSUM = '769ee6be814b21c52afcc631f07051f578487d642f3e89a45f2a086fd94fffa9'


def sha(data):
    return hashlib.sha256(data).hexdigest()


def identity(path):
    path = Path(path)
    data = path.read_bytes()
    return {'path': str(path.relative_to(ROOT)).replace('\\', '/') if path.is_relative_to(ROOT) else str(path),
            'sha256': sha(data), 'byte_length': len(data)}


def sources():
    cargo = Path(os.environ.get('CARGO_HOME', Path.home() / '.cargo'))
    matches = list((cargo / 'registry/src').glob('*/metis-sys-0.3.2'))
    if len(matches) != 1:
        raise ValueError('Require one cached pinned metis-sys 0.3.2 source; run Cargo fetch/build first.')
    source = matches[0]
    archive = cargo / 'registry/cache' / source.parent.name / 'metis-sys-0.3.2.crate'
    if sha(archive.read_bytes()) != METIS_SYS_CHECKSUM:
        raise ValueError('Pinned metis-sys archive checksum mismatch')
    # New Cargo caches need not contain .cargo-checksum.json. Compare the actual
    # extracted files directly with the separately checksum-pinned archive.
    with tarfile.open(archive, 'r:gz') as package:
        for member in package.getmembers():
            if not member.isfile():
                continue
            parts = member.name.split('/')
            if parts[0] != source.name or any(part in ('', '..') or '\\' in part or ':' in part for part in parts):
                raise ValueError('Unsupported crate archive path')
            name = Path(*parts[1:])
            if sha((source / name).read_bytes()) != sha(package.extractfile(member).read()):
                raise ValueError('Cargo source changed: ' + str(name))
    build = (source / 'build.rs').read_text(encoding='utf-8')
    result = []
    for directory in ('metis/libmetis', 'GKlib'):
        match = re.search(r'vendor.join\("' + re.escape(directory) + r'"\),\s*&\[(.*?)\]', build, re.S)
        if not match:
            raise ValueError('Unsupported vendored build source layout')
        result += [source / 'vendor' / directory / name for name in re.findall(r'"([a-z0-9_]+\.c)"', match[1])]
    return source, archive, result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--emsdk', default='target/toolchains/emsdk')
    parser.add_argument('--output', default='target/metis-c-sidecar-matrix')
    parser.add_argument('--rng-profile', choices=('unmodified', 'msvcrt'), default='unmodified',
                        help='Explicit experiment only; msvcrt replaces rand/srand with a source-owned compatibility shim.')
    args = parser.parse_args()
    sdk = (ROOT / args.emsdk).resolve()
    output_root = (ROOT / args.output).resolve()
    output = output_root / args.rng_profile
    if not output.is_relative_to((ROOT / 'target').resolve()):
        raise SystemExit('Probe output must stay within this repository target directory')
    output.mkdir(parents=True, exist_ok=True)
    source, archive, c_sources = sources()
    environment = dict(os.environ, EM_CONFIG=str(sdk / '.emscripten'))
    evidence = {'schema': 'bisect-metis-c-sidecar-probe-v1', 'backend': 'c-ffi',
                'source_archive': identity(archive), 'vendor_build': identity(source / 'build.rs'),
                'fixture': identity(ROOT / FIXTURE / 'sidecar.c'), 'runner': identity(Path(__file__)),
                'attempts': [], 'production_integrated': False, 'parity_verified': False,
                'scope': 'Exact cached vendored C METIS, idx32/real32. Isolated browser-capable sidecar feasibility only; no engine dispatcher or UI integration.'}
    evidence['rng_profile'] = args.rng_profile
    evidence['native_platform'] = sys.platform
    evidence['comparison_source'] = identity(ROOT / FIXTURE / 'compare-sidecar.mjs')
    evidence['native_sources'] = [identity(ROOT / FIXTURE / name) for name in ('Cargo.toml', 'Cargo.lock', 'src/lib.rs', 'src/main.rs')]
    evidence['sdk_files'] = [identity(sdk / name) for name in ('emsdk.py', 'emsdk_manifest.json', '.emscripten', 'upstream/emscripten/emcc.py', 'upstream/bin/clang.exe' if sys.platform == 'win32' else 'upstream/bin/clang')]
    if args.rng_profile == 'msvcrt':
        evidence['rng_shim'] = identity(ROOT / FIXTURE / 'msvcrt_rng.c')
    protected = [ROOT / name for name in ('Cargo.toml', 'Cargo.lock', 'crates/bisect-wasm/Cargo.toml', 'target/wasm32-unknown-unknown/release/bisect_wasm.wasm')]
    evidence['production_before'] = [identity(path) for path in protected]

    def run(label, arguments, timeout=240):
        try:
            result = subprocess.run(arguments, cwd=ROOT, env=environment, capture_output=True,
                                    timeout=timeout, creationflags=0x08000000 if sys.platform == 'win32' else 0)
        except FileNotFoundError as error:
            result = subprocess.CompletedProcess(arguments, 127, b'', str(error).encode())
        except subprocess.TimeoutExpired as error:
            result = subprocess.CompletedProcess(arguments, 124, error.stdout or b'', (error.stderr or b'') + b'\nProbe timeout')
        path = output / (label + '.log')
        path.write_bytes(result.stdout + b'\n--- stderr ---\n' + result.stderr)
        evidence['attempts'].append({'label': label, 'arguments': arguments, 'exit_code': result.returncode,
                                     'status': 'passed' if result.returncode == 0 else 'failed', 'log': identity(path)})
        print(label + ': ' + evidence['attempts'][-1]['status'], flush=True)
        return result

    run('emsdk-revision', ['git', '-C', str(sdk), 'rev-parse', 'HEAD'])
    run('emscripten-version', [sys.executable, str(sdk / 'upstream/emscripten/emcc.py'), '--version'])
    run('rustc-version', ['rustc', '-vV'])
    run('node-version', ['node', '--version'])
    base = ['cargo', 'build', '--manifest-path', FIXTURE + '/Cargo.toml', '--target-dir',
            str(output_root / 'native'), '--locked', '--offline', '-j', '1', '--bin', 'bisect-wasm-cffi-probe']
    native = run('native-build', base)
    native_result = None
    if native.returncode == 0:
        executable = output_root / 'native/debug' / ('bisect-wasm-cffi-probe.exe' if sys.platform == 'win32' else 'bisect-wasm-cffi-probe')
        evidence['native_executable'] = identity(executable)
        result = run('native-run', [str(executable), '--matrix'])
        if result.returncode == 0:
            lines = result.stdout.decode('utf-8').splitlines()
            records = [line for line in lines if line.startswith('{')]
            if len(records) != 1:
                raise ValueError('Native fixture must emit exactly one JSON record; inspect raw stdout log')
            native_result = json.loads(records[0])
            evidence['native_stdout_diagnostics'] = [line for line in lines if not line.startswith('{')]
            evidence['native_values'] = native_result
            (output / 'native-reference.json').write_text(json.dumps(native_result, indent=2) + '\n', encoding='utf-8')
            evidence['native_reference'] = identity(output / 'native-reference.json')
    emcc = [sys.executable, str(sdk / 'upstream/emscripten/emcc.py')]
    command = emcc + [str(path) for path in c_sources] + [str(ROOT / FIXTURE / 'sidecar.c'),
        '-I' + str(source / 'vendor/GKlib'), '-I' + str(source / 'vendor/metis/include'),
        '-I' + str(source / 'vendor/metis/libmetis'), '-DIDXTYPEWIDTH=32', '-DREALTYPEWIDTH=32',
        '-DNDEBUG', '-DNDEBUG2', '-O2', '-sMODULARIZE=1', '-sEXPORT_ES6=1',
        '-sENVIRONMENT=web,worker,node', '-sALLOW_MEMORY_GROWTH=1', '-sFILESYSTEM=0',
        '-sDYNAMIC_EXECUTION=0', '-sABORTING_MALLOC=0',
        '-sEXPORTED_RUNTIME_METHODS=HEAPU8',
        '-sEXPORTED_FUNCTIONS=["_bisect_metis_probe","_bisect_metis_error_probe","_bisect_metis_invalid_option_probe","_bisect_metis_case","_bisect_rng_seed","_bisect_rng_next","_METIS_PartGraphKway","_METIS_SetDefaultOptions","_malloc","_free"]',
        '-o', str(output / 'metis-probe.mjs')]
    if args.rng_profile == 'msvcrt':
        command += [str(ROOT / FIXTURE / 'msvcrt_rng.c'), '-Drand=bisect_msvcrt_rand', '-Dsrand=bisect_msvcrt_srand']
    compiled = run('sidecar-build', command)
    if compiled.returncode == 0 and native_result is not None:
        evidence['sidecar_js'] = identity(output / 'metis-probe.mjs')
        evidence['sidecar_wasm'] = identity(output / 'metis-probe.wasm')
        result = run('sidecar-run', ['node', str(ROOT / FIXTURE / 'compare-sidecar.mjs'),
                                     str(output / 'metis-probe.mjs'), str(output / 'metis-probe.wasm'), str(output / 'native-reference.json')])
        if result.stdout:
            try:
                evidence['comparison'] = json.loads(result.stdout)
            except json.JSONDecodeError:
                evidence['comparison'] = {'status': 'invalid-comparison-output'}
        if result.returncode == 0:
            evidence['decision'] = 'explicit-profile-matrix-passed-in-node'
        else:
            evidence['decision'] = 'sidecar-runtime-or-outcome-mismatch'
    else:
        evidence['decision'] = 'sidecar-build-or-native-reference-failed'
        evidence['sidecar_execution'] = 'not-run'
    evidence['production_after'] = [identity(path) for path in protected]
    evidence['production_unchanged'] = evidence['production_before'] == evidence['production_after']
    evidence['next_action'] = 'Inspect exact logs; require browser and weighted/seed/error matrix before considering explicit production bridge integration. Never replace metis-core silently.'
    (output / 'evidence.json').write_text(json.dumps(evidence, indent=2) + '\n', encoding='utf-8')
    print(evidence['decision'], flush=True)
    return 0 if evidence['decision'] == 'explicit-profile-matrix-passed-in-node' and evidence['production_unchanged'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
