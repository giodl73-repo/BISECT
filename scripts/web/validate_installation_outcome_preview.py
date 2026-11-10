"""Check both staged shells and real OPFS receipt-download/recovery journeys."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import time
from stage_installation_outcome_preview import ROOT, pin


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--suffix', default='preview')
    args = parser.parse_args()
    assert args.suffix in ['preview', 'durable-preview']
    output = ROOT / 'target/install-outcome' / ('validation-' + str(time.time_ns()))
    output.mkdir(parents=True)
    checks = []
    before = {}
    env = dict(os.environ)
    assert env.get('BISECT_PLAYWRIGHT_PACKAGES'), 'Set the Playwright package root'
    record = dict(schema_version='bisect-installation-outcome-validation-v1',
                  status='running', checks=checks, acceptance_v1=False, deployed=False)
    try:
        result = subprocess.run(['node', 'scripts/web/test_installation_outcome_store.mjs'], cwd=ROOT, capture_output=True, text=True, encoding='utf-8')
        log = output / 'storage-schema.log'
        log.write_text(result.stdout + result.stderr, encoding='utf-8')
        checks.append(dict(name='storage-schema', exit_code=result.returncode, log=pin(log)))
        assert result.returncode == 0, str(log)
        for profile in ['default', 'c-sidecar']:
            package = ROOT / 'dist' / ('wasm-install-outcome-' + profile + '-' + args.suffix)
            assert package.is_dir()
            before[profile] = [pin(p) for p in sorted(package.rglob('*')) if p.is_file()]
            for name, command in [
                ('assets', ['node', 'scripts/web/test_toolkit_assets.mjs', str(package / 'toolkit')]),
                ('offline', ['node', 'scripts/web/test_offline_package.mjs', str(package)]),
                ('single', ['node', 'scripts/web/test_native_state_recovery_browser.mjs', '--package', str(package / 'toolkit')]),
                ('batch', ['node', 'scripts/web/test_native_state_batch_recovery_browser.mjs', '--package', str(package / 'toolkit')]),
            ]:
                print(profile + ': ' + name, flush=True)
                run_dir = output / (profile + '-' + name)
                result = subprocess.run(command, cwd=ROOT,
                    env=dict(env, BISECT_INSTALL_OUTCOME_OUTPUT_DIR=str(run_dir)),
                    capture_output=True, text=True, encoding='utf-8', errors='replace')
                log = output / (profile + '-' + name + '.log')
                log.write_text(result.stdout + result.stderr, encoding='utf-8')
                checks.append(dict(profile=profile, name=name, command=command,
                                   exit_code=result.returncode, log=pin(log)))
                assert result.returncode == 0, str(log)
                if name in ['single', 'batch']:
                    evidence = json.loads((run_dir / 'evidence.json').read_text(encoding='utf-8'))
                    for key in ['installation_outcome_download', 'download_does_not_dispatch_or_write', 'open_clears_outcome', 'stored_outcome_reload', 'stored_outcome_no_authority', 'stored_outcome_forget', 'stale_tab_guard', 'storage_bounds_and_failures']:
                        assert evidence[key] is True
                    if name == 'single':
                        assert evidence['storage_unavailable_session_fallback'] is True
                    if name == 'batch':
                        assert evidence['hard_navigation_in_progress_marker'] is True
                    checks[-1]['browser_evidence'] = pin(run_dir / 'evidence.json')
                    checks[-1]['downloaded_outcomes'] = [pin(p) for p in sorted(run_dir.glob('*-outcome.json'))]
        record['status'] = 'passed'
    except Exception as error:
        record['status'] = 'failed'
        record['error'] = str(error)
        raise
    finally:
        record['packages'] = before
        for profile, refs in before.items():
            assert all(pin(ROOT / ref['path']) == ref for ref in refs), profile + ' package changed'
        record['source_files'] = [pin(ROOT / name) for name in [
            'web/lab/native-state-output-ui.js',
            'web/lab/installation-outcome-store.js',
            'scripts/web/test_installation_outcome_store.mjs',
            'scripts/web/test_native_state_recovery_browser.mjs',
            'scripts/web/test_native_state_batch_recovery_browser.mjs',
            'scripts/web/stage_installation_outcome_preview.py',
            'scripts/web/validate_installation_outcome_preview.py']]
        destination = output / 'evidence.json'
        destination.write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
        print(destination, flush=True)


if __name__ == '__main__':
    main()
