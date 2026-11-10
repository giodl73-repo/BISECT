"""Freeze the scoped local target-action checks; this is not release acceptance."""
import argparse
import hashlib
import json
import shutil
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def resolve(relative):
    path = (ROOT / relative).resolve()
    if not path.is_relative_to(ROOT) or not path.is_file():
        raise ValueError('Missing or unsafe snapshot input: ' + str(relative))
    return path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    cases = {'wasm-state-target-run-validated': 'native-state-target-run',
             'wasm-bulk-workspace': 'native-bulk-workspace',
             'wasm-acquisition-recovery-validated': 'acquisition-recovery',
             'wasm-native-output-verification-validated': 'native-output-verification',
             'wasm-workspace-continuation-validated': 'native-workspace-continuation'}
    parser.add_argument('--package', choices=list(cases), default='wasm-state-target-run-validated')
    args = parser.parse_args()
    case = cases[args.package]
    destination_root = ROOT / 'target' / case / 'package-snapshots' / args.package
    evidence_path = resolve('target/native-workspace-source/package-evidence.json')
    evidence = json.loads(evidence_path.read_text(encoding='utf-8'))
    package = Path(evidence['package']).resolve()
    if package != ROOT / 'dist' / args.package:
        raise ValueError('Expected the specifically validated target-action package.')
    if evidence['schema'] != 'bisect-workspace-preview-checks-v1' or evidence['browser_status'] != 'passed':
        raise ValueError('Package browser checks have not passed.')
    refs = evidence['source_files'] + evidence['results'] + [evidence['offline_manifest']] + evidence.get('explicit_c_batch_evidence', [])
    if evidence.get('optional_c_browser_evidence'):
        refs.append(evidence['optional_c_browser_evidence'])
    for ref in refs:
        if digest(resolve(ref['path'])) != ref['sha256']:
            raise ValueError('Stale package evidence: ' + ref['path'])
    for ref in evidence['source_files']:
        if ref['path'].startswith('web/lab/'):
            staged = package / 'toolkit' / Path(ref['path']).name
            if staged.name == 'workbench.html':
                staged = staged.with_name('index.html')
            if not staged.is_file() or digest(staged) != ref['sha256']:
                raise ValueError('Staged source mismatch: ' + str(staged))
    manifest = json.loads((package / 'offline-manifest.json').read_text(encoding='utf-8'))
    if manifest['version'] != evidence['offline_version']:
        raise ValueError('Package version changed.')
    sources = {resolve(ref['path']) for ref in refs}
    sources.add(evidence_path)
    extra = [
        'scripts/web/snapshot_state_target_run.py',
        'crates/bisect-wasm/src/native_state_run.rs',
        'crates/bisect-cli/src/runner/mod.rs',
        'crates/bisect-runner/src/state_config.rs',
        'crates/bisect-runner/src/state_args.rs',
        'scripts/web/fixtures/native-output-directory.mjs',
        'scripts/web/fixtures/native-output-recovery-case.mjs',
        'scripts/web/fixtures/native-output-batch-recovery-case.mjs',
        'scripts/web/build_wasm.ps1', 'scripts/web/build_wasm_lab.ps1',
        'target/native-state-target-run/source.bisect',
        'target/native-state-target-run/run.json',
        'target/native-state-target-run-browser/completed.bisect',
        'target/native-state-target-run-browser/action-receipt.json',
        'target/native-state-target-run-browser/interrupted-action-receipt.json',
        'target/native-state-target-run-browser/mobile.png',
        'target/native-state-data-reset/cli-records.json',
    ]
    sources.update(resolve(name) for name in extra)
    if case == 'native-bulk-workspace':
        sources.update(resolve(name) for name in [
            'target/native-bulk-workspace/source.bisect',
            'target/native-bulk-workspace/result.json',
            'target/native-bulk-workspace/partial.json',
            'target/native-bulk-workspace-browser/completed.bisect',
            'target/native-bulk-workspace-browser/workspace-actions.json',
            'target/native-bulk-workspace-browser/interrupted-actions.json',
            'target/native-bulk-workspace-browser/mobile-panel.png',
            'crates/bisect-cli/src/runner/support.rs',
            'crates/bisect-core/src/state_catalog.rs',
            'crates/bisect-runner/src/run_command.rs',
            'crates/bisect-runner/src/states_config.rs',
        ])
    if case == 'acquisition-recovery':
        for folder in ['acquisition-recovery', 'acquisition-recovery-configured']:
            sources.update(resolve('target/' + folder + '/' + name) for name in [
                'partial.bisect', 'partial.json', 'verified.json', 'resumed.json',
                'native-tiger-fixture.exe', 'RI-tiger.zip', 'RI-pl94.zip',
                'VT-tiger.zip', 'VT-pl94.zip'])
            sources.update(resolve('target/' + folder + '-browser/' + name) for name in [
                'completed.bisect', 'changed.bisect', 'mobile-panel.png'])
    if case == 'native-output-verification':
        engine_hash = digest(package / 'toolkit/bisect_wasm.wasm')
        for outcome in ['target/native-output-verification/evidence.json',
                        'target/native-output-verification-browser/evidence.json']:
            if json.loads(resolve(outcome).read_text(encoding='utf-8'))['wasm']['sha256'] != engine_hash:
                raise ValueError('Verify outcome used a different engine: ' + outcome)
        for variant in ['default', 'c-sidecar']:
            gate = json.loads(resolve('target/cffi-legacy-gates/' + variant + '/build.json').read_text(encoding='utf-8'))
            if gate['status'] != 'passed' or gate['wasm_sha256'] != digest(resolve('target/cffi-legacy-gates/' + variant + '.wasm')):
                raise ValueError('Missing or changed engine-variant gate: ' + variant)
            for ref in gate['source_files']:
                if digest(resolve(ref['path'])) != ref['sha256']:
                    raise ValueError('Changed engine-variant test source: ' + ref['path'])
        sources.update(resolve('target/native-output-verification/' + name) for name in [
            'source.bisect', 'replayed.bisect', 'output.zip', 'native-cases.json', 'records.json',
            'native-inspections.json', 'native-tool-reference.exe'])
        sources.update(resolve('target/native-output-verification-browser/' + name) for name in [
            'replayed.bisect', 'verification-record.json', 'verification-390.png',
            'verification-controls-390.png', 'after-stale-read.bisect'])
        sources.update(resolve('target/cffi-legacy-gates/' + variant + '/' + name)
                       for variant in ['default', 'c-sidecar']
                       for name in ['build.json', 'native-output-verification-evidence.json',
                                    'native-output-verification-records.json',
                                    'native-output-verification-native-cases.json',
                                    'native-output-verification-native-inspections.json'])
        sources.update(resolve('target/cffi-legacy-gates/' + variant + '.wasm')
                       for variant in ['default', 'c-sidecar'])
        sources.update(resolve(name) for name in ['scripts/web/check_cffi_legacy_gates.ps1',
            'scripts/web/build_cffi_preview.ps1', 'scripts/web/test_project.mjs',
            'Cargo.lock', 'Cargo.toml', 'crates/bisect-wasm/Cargo.toml',
            'crates/bisect-wasm/src/engine.rs', 'crates/bisect-wasm/src/toolkit.rs',
            'web/lab/wasm-engine.js', 'web/lab/metis-c-sidecar.js',
            'scripts/web/test_manifest_replay_wasm.mjs', 'scripts/web/test_tui_verify_wasm.mjs',
            'target/release/examples/execute_tool_request.exe',
            'target/cffi-integration/package-evidence.json', 'target/cffi-integration/engine-evidence.json',
            'target/tui-verify-browser/prechecks.bisect', 'target/tui-verify-browser/replayed.bisect',
            'scripts/web/test_tui_verify_browser.mjs'])
    if case == 'native-workspace-continuation':
        engine_hash = digest(package / 'toolkit/bisect_wasm.wasm')
        for outcome in ['target/native-workspace-continuation/evidence.json',
                        'target/native-workspace-continuation-browser/evidence.json']:
            if json.loads(resolve(outcome).read_text(encoding='utf-8'))['wasm']['sha256'] != engine_hash:
                raise ValueError('Continuation used a different engine: ' + outcome)
        for variant in ['default', 'c-sidecar']:
            prefix = 'target/cffi-legacy-gates/' + variant + '/'
            gate = json.loads(resolve(prefix + 'build.json').read_text(encoding='utf-8'))
            if gate['status'] != 'passed' or gate['wasm_sha256'] != digest(resolve('target/cffi-legacy-gates/' + variant + '.wasm')):
                raise ValueError('Missing or changed engine-variant gate: ' + variant)
            for ref in gate['source_files']:
                if digest(resolve(ref['path'])) != ref['sha256']:
                    raise ValueError('Changed engine-variant test/source: ' + ref['path'])
            sources.add(resolve(prefix + 'build.json'))
            sources.add(resolve('target/cffi-legacy-gates/' + variant + '.wasm'))
            sources.update(resolve(prefix + 'continuation/' + name) for name in [
                'source.bisect', 'multi-source.bisect', 'multi-district-source.bisect',
                'multi-district-completed.json', 'evidence.json', 'completed.json',
                'multi-completed.json', 'verification.json', 'filesystem-events.json', 'cases.json'])
        sources.update(resolve('target/cffi-legacy-gates/default/continuation/browser/' + name) for name in [
            'one-district-evidence.json', 'evidence.json', 'completed.bisect', 'interrupted.bisect',
            'continuation-actions.json', 'continuation-390.png'])
        sources.update(resolve('target/native-workspace-continuation/' + name) for name in [
            'source.bisect', 'multi-source.bisect', 'multi-district-source.bisect', 'multi-district-completed.json'])
        sources.update(resolve('target/native-workspace-continuation-browser/' + name) for name in [
            'interrupted.bisect', 'completed.bisect', 'continuation-actions.json', 'continuation-390.png',
            'one-district-evidence.json', 'multi-district-evidence.json'])
        sources.update(resolve(name) for name in [
            'Cargo.lock', 'Cargo.toml', 'crates/bisect-wasm/Cargo.toml',
            'scripts/web/check_cffi_legacy_gates.ps1', 'web/lab/state-run-years.js',
            'scripts/web/test_state_run_batch_resume.mjs', 'scripts/web/test_state_run_years_resume.mjs',
            'scripts/web/test_state_run_batch_recovery.mjs', 'scripts/web/test_project.mjs',
            'target/state-run-batch-resume/evidence.json', 'target/state-run-years-resume/evidence.json',
            'target/state-run-batch-recovery/evidence.json'])
    sources.update(p for p in (package / 'toolkit').rglob('*.wasm'))
    sources.update(p for p in (package / 'toolkit/c-metis').glob('*') if p.is_file())
    if destination_root.exists():
        raise ValueError('Existing snapshots are retained; choose a separately versioned investigation.')
    destination_root.mkdir(parents=True)
    copies = []
    for source in sorted(sources):
        relative = source.relative_to(ROOT)
        destination = destination_root / 'files' / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, destination)
        if digest(destination) != digest(source):
            raise ValueError('Snapshot copy differs: ' + str(relative))
        copies.append({'original_path': relative.as_posix(),
                       'path': destination.relative_to(ROOT).as_posix(),
                       'sha256': digest(destination)})
    schemas = {'native-state-target-run': 'bisect-native-state-target-investigation-v1',
               'native-bulk-workspace': 'bisect-native-bulk-workspace-investigation-v1',
               'acquisition-recovery': 'bisect-state-acquisition-recovery-investigation-v1',
               'native-output-verification': 'bisect-native-output-verification-investigation-v1',
               'native-workspace-continuation': 'bisect-native-workspace-continuation-investigation-v1'}
    result = {'schema': schemas[case],
              'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
              'dirty_working_tree': True, 'package': package.relative_to(ROOT).as_posix(),
              'offline_version': manifest['version'], 'files': copies,
              'acceptance_v1': False, 'full_parity': False, 'deployed': False}
    (destination_root / 'snapshot.json').write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
    print('Frozen local investigation:', destination_root.relative_to(ROOT).as_posix(), len(copies), 'files')


if __name__ == '__main__':
    main()
