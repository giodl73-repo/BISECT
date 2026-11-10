"""Stage explicit partition timing only after fresh registered dual-build gates."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[2]


def pin(path):
    return dict(path=path.relative_to(ROOT).as_posix(), sha256=hashlib.sha256(path.read_bytes()).hexdigest(), byte_length=path.stat().st_size)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--suffix', choices=['preview', 'preview-v2'], default='preview')
    args = parser.parse_args()
    overlays = ['partition-timing.js', 'native-state-workflow.js', 'native-state-diagnostics.js', 'native-state-job.js', 'native-adjacency-state-job.js', 'prepared-state-job.js', 'state-run-batch.js', 'workbench.js', 'count-results.js', 'wasm-engine.js']
    scripts = ['test_partition_timing.mjs', 'test_partition_timing_browser.mjs', 'test_state_diagnostic_flags.mjs', 'test_native_state_workflow.mjs', 'test_native_fetch_install_browser.mjs', 'build_partition_timing_preview.py']
    sources = [ROOT / 'web/lab' / n for n in overlays] + [ROOT / 'scripts/web' / n for n in scripts] + [ROOT / n for n in ['crates/bisect-wasm/src/partition_timing.rs', 'crates/bisect-wasm/src/engine.rs', 'crates/bisect-wasm/src/multiscale_input.rs', 'crates/bisect-wasm/src/prepared_engine.rs', 'crates/bisect-wasm/src/toolkit.rs', 'crates/bisect-wasm/src/lib.rs', 'crates/bisect-cli/src/runner/mod.rs', 'scripts/web/fixtures/native-state-source.json', 'target/prepared-engine/later-build.bisect', 'target/multiscale-fixtures/request-0-0.json', 'target/release/examples/execute_tool_request.exe', 'target/release/bisect.exe']]
    before = [pin(p) for p in sources]
    legacy = {}
    engines = {}
    for profile in ['default', 'c-sidecar']:
        record = ROOT / 'target/cffi-legacy-gates' / profile / 'build.json'
        data = json.loads(record.read_text(encoding='utf-8-sig'))
        assert data['status'] == 'passed' and data['profile'] == profile
        assert data['engine_build']['mode'] == 'fresh-cargo-build' and data['engine_build']['recompiled_in_this_runner'] is True
        engine = ROOT / 'target/cffi-legacy-gates' / (profile + '.wasm')
        assert pin(engine)['sha256'] == data['wasm_sha256']
        for ref in data['source_files']:
            assert pin(ROOT / ref['path'])['sha256'] == ref['sha256'], 'Legacy source changed: ' + ref['path']
        legacy[profile] = dict(record=pin(record), engine=pin(engine), run_id=data['run_id'])
        engines[profile] = engine
    assert legacy['default']['run_id'] == legacy['c-sidecar']['run_id']
    output = ROOT / 'target/partition-timing' / ('package-' + str(time.time_ns()))
    output.mkdir(parents=True)
    record = dict(status='running', checks=[], packages={}, source_files=before, fresh_registered_legacy=legacy, deployed=False, acceptance_v1=False)

    def run(name, command, env):
        print(name, flush=True)
        result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True, encoding='utf-8', errors='replace')
        log = output / (name + '.log')
        log.write_text(result.stdout + result.stderr, encoding='utf-8')
        record['checks'].append(dict(name=name, exit_code=result.returncode, log=pin(log)))
        assert result.returncode == 0, str(log)

    def replace(source, destination):
        temporary = destination.with_name(destination.name + '.timing-tmp')
        shutil.copyfile(source, temporary)
        os.replace(temporary, destination)

    def write_json(destination, value):
        temporary = destination.with_name(destination.name + '.timing-tmp')
        temporary.write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')
        os.replace(temporary, destination)

    try:
        for profile in ['default', 'c-sidecar']:
            base = ROOT / 'dist' / ('wasm-state-diagnostic-' + profile + '-preview')
            package = ROOT / 'dist' / ('wasm-partition-timing-' + profile + '-' + args.suffix)
            assert base.is_dir() and not package.exists(), package
            assert not any(p.is_symlink() for p in base.rglob('*'))
            original = [pin(p) for p in sorted(base.rglob('*')) if p.is_file()]
            shutil.copytree(base, package, copy_function=os.link)
            for directory in [package, package / 'toolkit']:
                for name in overlays:
                    replace(ROOT / 'web/lab' / name, directory / name)
                replace(engines[profile], directory / 'bisect_wasm.wasm')
                catalog = directory / 'catalog.json'
                if catalog.is_file():
                    value = json.loads(catalog.read_text(encoding='utf-8'))
                    value['wasm_sha256'] = pin(engines[profile])['sha256']
                    write_json(catalog, value)
                proof = directory / 'native-proof-build.json'
                if proof.is_file():
                    value = json.loads(proof.read_text(encoding='utf-8'))
                    for name, identity in value['wasm_dependencies'].items():
                        assert name in ['bisect_wasm.wasm', 'roundingsat/roundingsat.mjs', 'roundingsat/roundingsat.wasm', 'proof-checker/bisect_proof_checker.wasm']
                        if name == 'bisect_wasm.wasm':
                            value['wasm_dependencies'][name] = pin(engines[profile])['sha256']
                        else:
                            assert pin(directory / name)['sha256'] == identity, 'Proof dependency changed'
                    write_json(proof, value)
            # The manifest writer must not write through the accepted base link.
            replace(package / 'offline-manifest.json', package / 'offline-manifest.json')
            unit = output / (profile + '-unit')
            env = dict(os.environ, BISECT_TEST_WASM=str(package / 'toolkit/bisect_wasm.wasm'), BISECT_PARTITION_TIMING_OUTPUT=str(unit), BISECT_STATE_DIAGNOSTIC_OUTPUT=str(output / (profile + '-diagnostics')), BISECT_STATE_WORKFLOW_OUTPUT=str(output / (profile + '-legacy-workflow')), BISECT_FETCH_INSTALL_OUTPUT=str(output / (profile + '-fetch-browser')))
            run(profile + '-offline-build', [sys.executable, 'scripts/web/build_offline_manifest.py', str(package)], env)
            assert all(pin(ROOT / p['path']) == p for p in original), 'Accepted base changed'
            for source in (ROOT / 'web/lab').glob('*.js'):
                for directory in [package, package / 'toolkit']:
                    assert pin(directory / source.name)['sha256'] == pin(source)['sha256'], source
            sealed = [pin(p) for p in sorted(package.rglob('*')) if p.is_file()]
            record['packages'][profile] = sealed
            unit_command = ['node', 'scripts/web/test_partition_timing.mjs']
            if profile == 'c-sidecar':
                unit_command += ['--c-sidecar', str(package / 'toolkit/c-metis')]
            for label, command in [
                ('assets', ['node', 'scripts/web/test_toolkit_assets.mjs', str(package / 'toolkit')]),
                ('offline', ['node', 'scripts/web/test_offline_package.mjs', str(package)]),
                ('timing-unit', unit_command),
                ('diagnostics-regression', ['node', 'scripts/web/test_state_diagnostic_flags.mjs']),
                ('legacy-workflow', ['node', 'scripts/web/test_native_state_workflow.mjs', '--core-only']),
                ('browser', ['node', 'scripts/web/test_partition_timing_browser.mjs', '--package', str(package / 'toolkit'), '--fixtures', str(unit), '--output', str(output / (profile + '-browser'))]),
                ('fetch-browser', ['node', 'scripts/web/test_native_fetch_install_browser.mjs', '--package', str(package / 'toolkit')]),
            ]:
                run(profile + '-' + label, command, env)
            assert all(pin(ROOT / p['path']) == p for p in sealed), 'Tested package changed'
        record['status'] = 'passed'
    except Exception as error:
        record.update(status='failed', error=str(error))
        raise
    finally:
        record['source_unchanged'] = [pin(p) for p in sources] == before
        destination = output / 'evidence.json'
        destination.write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
        print(destination, flush=True)
        assert record['source_unchanged'], 'Source changed during validation'


if __name__ == '__main__':
    main()
