"""Run the isolated, locked WP-02 C-FFI feasibility witness.

Writes raw command logs and byte identities under target/wasm-cffi-feasibility.
A failed build is a recorded unsuccessful attempt, never an accepted port.
The production WASM binary, features and Cargo.lock are not modified.
"""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
FIXTURE = 'scripts/web/fixtures/wasm-cffi-probe'
OUTPUT = 'target/wasm-cffi-feasibility'


def identity(path):
    data = (ROOT / path).read_bytes()
    return {'path': path, 'sha256': hashlib.sha256(data).hexdigest(), 'byte_length': len(data)}


def main():
    output = ROOT / OUTPUT
    output.mkdir(parents=True, exist_ok=True)
    evidence = {
        'schema': 'bisect-cffi-feasibility-attempt-v1', 'backend': 'c-ffi',
        'fixture': {'graph': 'four-cycle', 'parts': 2, 'seed': '7', 'idx_bits': 32, 'real_bits': 32},
        'sources': [identity(FIXTURE + '/' + name) for name in ('Cargo.toml', 'Cargo.lock', 'src/lib.rs', 'src/main.rs')],
        'attempts': [], 'production_engine_changed': False, 'parity_verified': False,
        'scope': 'Isolated metis 0.2.2 vendored dependency build and fixed-seed call. Even a successful probe does not port the BISECT dispatcher, prove general method parity or establish capacity.',
    }
    evidence['runner'] = identity('scripts/web/probe_wasm_cffi.py')
    protected = ['Cargo.toml', 'Cargo.lock', 'crates/bisect-wasm/Cargo.toml',
                 'target/wasm32-unknown-unknown/release/bisect_wasm.wasm']
    evidence['production_before'] = [identity(path) for path in protected if (ROOT / path).is_file()]

    def run(label, arguments):
        try:
            result = subprocess.run(arguments, cwd=ROOT, capture_output=True)
        except FileNotFoundError as error:
            result = subprocess.CompletedProcess(arguments, 127, b'', str(error).encode('utf-8'))
        log = OUTPUT + '/' + label + '.log'
        (ROOT / log).write_bytes(result.stdout + b'\n--- stderr ---\n' + result.stderr)
        record = {'label': label, 'arguments': arguments, 'exit_code': result.returncode,
                  'status': 'passed' if result.returncode == 0 else 'failed', 'log': identity(log)}
        evidence['attempts'].append(record)
        print(label + ': ' + record['status'], flush=True)
        return result

    run('rustc-version', ['rustc', '-vV'])
    run('cargo-version', ['cargo', '-V'])
    run('revision', ['git', 'rev-parse', 'HEAD'])
    base = ['cargo', 'build', '--manifest-path', FIXTURE + '/Cargo.toml', '--target-dir', OUTPUT + '/build',
            '--locked', '--offline', '-j', '1']
    native = run('native-build', base + ['--bin', 'bisect-wasm-cffi-probe'])
    if native.returncode == 0:
        executable = OUTPUT + '/build/debug/bisect-wasm-cffi-probe' + ('.exe' if sys.platform == 'win32' else '')
        evidence['native_executable'] = identity(executable)
        run('native-run', [str(ROOT / executable)])
    else:
        evidence['attempts'].append({'label': 'native-run', 'status': 'not-run', 'exit_code': None,
                                     'reason': 'Native build failed'})
    wasm = run('wasm-build', base + ['--lib', '--target', 'wasm32-unknown-unknown'])
    if wasm.returncode == 0:
        artifact = OUTPUT + '/build/wasm32-unknown-unknown/debug/bisect_wasm_cffi_probe.wasm'
        evidence['wasm'] = identity(artifact)
        run('wasm-run', ['node', '--input-type=module', '-e',
            "import fs from 'node:fs'; const bytes=fs.readFileSync(process.argv[1]); "
            "const module=await WebAssembly.compile(bytes); console.log(JSON.stringify(WebAssembly.Module.imports(module))); "
            "const instance=await WebAssembly.instantiate(module,{}); const result=instance.exports.partition_probe(); "
            "console.log(result); if(result<0)process.exitCode=1;", artifact])
    else:
        evidence['attempts'].append({'label': 'wasm-run', 'status': 'not-run', 'exit_code': None,
                                     'reason': 'WASM build failed; no module was instantiated'})
    runs = {attempt['label']: attempt for attempt in evidence['attempts']}
    both_ran = all(label in runs and runs[label]['exit_code'] == 0 for label in ('native-run', 'wasm-run'))
    evidence['decision'] = 'probe-runs-comparison-still-required' if both_ran else 'not-ported-build-or-runtime-obstacle'
    evidence['next_action'] = ('Compare exact native/WASM assignments, then integrate an explicit opt-in backend and full BISECT fixtures.'
                               if both_ran else 'Inspect retained build/runtime logs; supply the indicated cross C compiler/sysroot or address the observed C dependency incompatibility, then repeat this exact locked probe. Keep C-FFI support unresolved.')
    evidence['production_after'] = [identity(ref['path']) for ref in evidence['production_before']]
    evidence['production_engine_changed'] = evidence['production_before'] != evidence['production_after']
    (output / 'evidence.json').write_text(json.dumps(evidence, indent=2) + '\n', encoding='utf-8')
    print(evidence['decision'], flush=True)
    return 0 if both_ran and not evidence['production_engine_changed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
