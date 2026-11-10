"""Stage explicit election source adaptation on both retained engine profiles."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
from evidence_blob_store import ROOT, digest

OVERLAYS = ['workbench.js', 'project.js', 'analysis-package.js', 'state-workflow-files.js',
            'state-source-job.js', 'native-state-followup.js', 'election-source-format.js',
            'election-source-format-ui.js']

def pin(path):
    path = Path(path)
    return dict(path=path.relative_to(ROOT).as_posix(), sha256=digest(path), byte_length=path.stat().st_size)

def main():
    names = ['web/lab/' + n for n in OVERLAYS] + [
        'scripts/web/test_election_source_format.mjs', 'scripts/web/test_election_source_format_browser.mjs',
        'scripts/web/test_project.mjs', 'scripts/web/build_election_source_format_preview.py',
        'scripts/data/elections/download_election_data.py', 'scripts/data/elections/process_election_data.py',
        'scripts/data/elections/fetch_elections.py', 'scripts/data/elections/sources.json',
        'crates/bisect-wasm/src/election_analysis.rs', 'crates/bisect-analysis/src/political.rs']
    before = [pin(ROOT / n) for n in names]
    output = ROOT / 'target/election-source-format' / ('package-' + str(time.time_ns()))
    output.mkdir(parents=True)
    checks, packages = [], {}
    record = dict(status='running', checks=checks, packages=packages, deployed=False, acceptance_v1=False)
    def run(name, command, env=None):
        print(name, flush=True)
        result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True,
                                encoding='utf-8', errors='replace')
        log = output / (name + '.log'); log.write_text(result.stdout + result.stderr, encoding='utf-8')
        checks.append(dict(name=name, command=command, exit_code=result.returncode, log=pin(log)))
        assert result.returncode == 0, str(log)
    try:
        run('project-security-and-handler-regression', ['node', 'scripts/web/test_project.mjs'])
        for profile in ['default', 'c-sidecar']:
            base = ROOT / 'dist' / ('wasm-adjacency-release-' + profile + '-preview-v6')
            package = ROOT / 'dist' / ('wasm-election-source-format-' + profile + '-preview')
            assert base.is_dir() and not package.exists()
            original = [pin(p) for p in sorted(base.rglob('*')) if p.is_file()]
            assert not any(p.is_symlink() for p in base.rglob('*'))
            shutil.copytree(base, package, copy_function=os.link)
            for relative in OVERLAYS + ['toolkit/' + n for n in OVERLAYS] + ['offline-manifest.json']:
                target = package / relative
                temporary = target.with_name(target.name + '.format-tmp')
                source = target if relative == 'offline-manifest.json' else ROOT / 'web/lab' / target.name
                shutil.copyfile(source, temporary); os.replace(temporary, target)
            run(profile + '-offline-build', [sys.executable, 'scripts/web/build_offline_manifest.py', str(package)])
            assert all(pin(ROOT / p['path']) == p for p in original), 'Accepted base changed'
            for source in (ROOT / 'web/lab').iterdir():
                if source.is_file() and source.suffix in ['.js', '.css']:
                    for directory in [package, package / 'toolkit']:
                        assert digest(directory / source.name) == digest(source), source
            packages[profile] = [pin(p) for p in sorted(package.rglob('*')) if p.is_file()]
            env = dict(os.environ, BISECT_TEST_WASM=str(package / 'toolkit/bisect_wasm.wasm'))
            run(profile + '-assets', ['node', 'scripts/web/test_toolkit_assets.mjs', str(package / 'toolkit')])
            run(profile + '-offline', ['node', 'scripts/web/test_offline_package.mjs', str(package)])
            for kind, script in [('unit', 'test_election_source_format.mjs'), ('browser', 'test_election_source_format_browser.mjs')]:
                run(profile + '-' + kind, ['node', 'scripts/web/' + script] + (['--package', str(package / 'toolkit')] if kind == 'browser' else []),
                    dict(env, BISECT_ELECTION_FORMAT_OUTPUT=str(output / (profile + '-' + kind))))
            run(profile + '-release-browser-regression', ['node', 'scripts/web/test_adjacency_release_browser.mjs', '--package', str(package / 'toolkit')],
                dict(env, BISECT_RELEASE_TEST_OUTPUT=str(output / (profile + '-release-browser'))))
            assert all(pin(ROOT / p['path']) == p for p in packages[profile]), 'Tested package changed'
        record['status'] = 'passed'
    except Exception as error:
        record.update(status='failed', error=str(error)); raise
    finally:
        record['source_files'] = before
        record['source_unchanged'] = [pin(ROOT / n) for n in names] == before
        record['native_host'] = pin(ROOT / 'target/release/examples/execute_tool_request.exe')
        assert record['source_unchanged'], 'Source changed during validation'
        destination = output / 'evidence.json'; destination.write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
        print(destination, flush=True)

if __name__ == '__main__':
    main()
