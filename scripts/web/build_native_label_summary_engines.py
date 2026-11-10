"""Build and compare fresh native/default/C summary kernels; not package acceptance."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
from evidence_blob_store import ROOT, digest


def main():
    root = ROOT / 'target/native-label-summary-delivery' / ('engines-' + str(time.time_ns()))
    root.mkdir()
    print('Engine evidence: ' + str(root), flush=True)
    stages = []

    def run(name, arguments, *, wasm=None):
        print('Native summary gate: ' + name, flush=True)
        env = dict(os.environ)
        if wasm:
            env['BISECT_TEST_WASM'] = str(wasm)
        result = subprocess.run(arguments, cwd=ROOT, env=env, capture_output=True, text=True, encoding='utf-8', errors='replace')
        log = root / (name + '.log')
        log.write_text(result.stdout + '\n' + result.stderr, encoding='utf-8')
        stages.append(dict(name=name, exit_code=result.returncode, log=str(log.relative_to(ROOT)), sha256=digest(log)))
        if result.returncode:
            raise RuntimeError(name + ' failed; inspect ' + str(log))
        return result.stdout

    try:
        run('shared-kernel', ['cargo', 'test', '--release', '-p', 'bisect-report', '--no-default-features', '--lib', 'label_summary', '--locked', '--offline', '-j', '1'])
        run('native-cli', ['cargo', 'build', '-p', 'bisect-cli', '--release', '--no-default-features', '--locked', '--offline', '-j', '1'])
        run('original-cli-comparison', [sys.executable, 'scripts/web/test_native_label_summary_cli.py'])
        run('native-host', ['cargo', 'build', '-p', 'bisect-wasm', '--release', '--example', 'execute_tool_request', '--locked', '--offline', '-j', '1'])
        profiles = {}
        for profile in ['default', 'c-sidecar']:
            arguments = ['cargo', 'build', '-p', 'bisect-wasm', '--release', '--target', 'wasm32-unknown-unknown', '--locked', '--offline', '-j', '1']
            if profile == 'c-sidecar':
                arguments += ['--features', 'c-metis-sidecar']
            run(profile + '-build', arguments)
            destination = root / (profile + '.wasm')
            shutil.copy2(ROOT / 'target/wasm32-unknown-unknown/release/bisect_wasm.wasm', destination)
            direct = json.loads(run(profile + '-direct', ['node', 'scripts/web/test_native_label_summary_wasm.mjs'], wasm=destination))
            batch = json.loads(run(profile + '-batch', ['node', 'scripts/web/test_native_label_summary_batch.mjs'], wasm=destination))
            profiles[profile] = dict(path=str(destination.relative_to(ROOT)), sha256=digest(destination), byte_length=destination.stat().st_size, direct=direct, batch=batch)
        names = ['crates/bisect-report/src/label_summary.rs', 'crates/bisect-report/src/lib.rs', 'crates/bisect-cli/src/analyze_label.rs', 'crates/bisect-wasm/src/native_label_summary.rs', 'crates/bisect-wasm/src/lib.rs', 'crates/bisect-wasm/src/toolkit.rs', 'Cargo.lock', 'scripts/web/test_native_label_summary_cli.py', 'scripts/web/test_native_label_summary_wasm.mjs', 'scripts/web/test_native_label_summary_batch.mjs', 'scripts/web/build_native_label_summary_engines.py', 'web/lab/native-label-summary.js', 'web/lab/label-report-files.js']
        pin = lambda name: dict(path=name, sha256=digest(ROOT/name), byte_length=(ROOT/name).stat().st_size)
        evidence = dict(status='passed', profiles=profiles, stages=stages, native_cli=pin('target/release/bisect.exe'), native_host=pin('target/release/examples/execute_tool_request.exe'), source_files=[pin(name) for name in names], rustc=subprocess.check_output(['rustc','-Vv'], text=True), cargo=subprocess.check_output(['cargo','-V'], text=True), full_parity=False, scope='Fresh native/default/C shared summary kernel, native files/failures and portable all-year/output sequencing. Browser UI, package, broader matrix and release acceptance remain separate.')
        record = root/'evidence.json'
        record.write_text(json.dumps(evidence, indent=2)+'\n', encoding='utf-8')
        print(json.dumps(dict(record=str(record.relative_to(ROOT)), sha256=digest(record))), flush=True)
    except Exception as error:
        (root/'failed-build.json').write_text(json.dumps(dict(status='failed', error=str(error), stages=stages), indent=2)+'\n', encoding='utf-8')
        raise


if __name__ == '__main__':
    main()
