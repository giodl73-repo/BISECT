"""Validate already staged national correction packages without replacing tested bytes."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
from evidence_blob_store import ROOT, local, digest
from validate_label_build_application_engines import validate as validate_engines


def pin(name):
    p = local(name)
    return dict(path=p.relative_to(ROOT).as_posix(), sha256=digest(p), byte_length=p.stat().st_size)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--engine-record', required=True)
    parser.add_argument('--package', default='dist/wasm-national-queue-preview')
    parser.add_argument('--default-package', default='dist/wasm-national-queue-default-preview')
    args = parser.parse_args()
    assert shutil.disk_usage(ROOT).free >= 2*1024**3, 'Need 2 GiB for owned package checks'
    record = local(args.engine_record)
    engines, _ = validate_engines(record)
    assert engines['status'] == 'passed'
    packages = {'c-sidecar': local(args.package), 'default': local(args.default_package)}
    for name, package in packages.items():
        assert package.is_relative_to(ROOT/'dist') and package.is_dir()
        expected = engines['profiles'][name]['artifact']['sha256']
        assert digest(package/'bisect_wasm.wasm') == digest(package/'toolkit/bisect_wasm.wasm') == expected
        for source in (ROOT/'web/lab').iterdir():
            if source.is_file() and source.name not in ['index.html', 'bisect_wasm.wasm']:
                for directory in [package, package/'toolkit']:
                    assert digest(directory/source.name) == digest(source), 'Staged source drift: '+str(directory/source.name)
        assert digest(package/'toolkit/index.html') == digest(ROOT/'web/lab/workbench.html')
        gate = json.loads((ROOT/'target/cffi-legacy-gates'/name/'build.json').read_text(encoding='utf-8-sig'))
        assert gate['status'] == 'passed' and gate['wasm_sha256'] == expected
        # Source bindings prevent an old passing marker from standing in for new gates.
        refs = {r['path']: r for r in gate['source_files']}
        for path in ['web/lab/project.js','web/lab/state-run-batch.js','web/lab/state-batch-zip.js','web/lab/workbench-archive.js','scripts/web/test_state_batch_zip.mjs','scripts/web/check_cffi_legacy_gates.ps1']:
            assert refs[path]['sha256'] == digest(ROOT/path), 'Registered gate is stale: '+path
    run_root = ROOT/'target/national-matrix'/('package-'+str(time.time_ns()))
    run_root.mkdir()
    if os.name == 'nt':
        subprocess.run(['compact.exe','/C','/I','/Q',str(run_root)],check=True,capture_output=True)
    stages = []
    original = {p.relative_to(ROOT).as_posix(): digest(p) for package in packages.values() for p in package.rglob('*') if p.is_file()}
    env = dict(os.environ, BISECT_TEST_WASM=str(packages['c-sidecar']/'toolkit/bisect_wasm.wasm'), BISECT_TEST_NATIVE_CLI=str(local(engines['native_cli']['path'])), BISECT_TEST_NATIVE_TOOL=str(local(engines['native_host']['path'])))

    def run(name, command, expected_exit=0):
        print('National package gate: '+name, flush=True)
        result = subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace')
        log = run_root/(name+'.log')
        log.write_text(result.stdout+'\n'+result.stderr,encoding='utf-8')
        stages.append(dict(name=name,args=command,exit_code=result.returncode,expected_exit_code=expected_exit,log=pin(log)))
        (run_root/'stages.json').write_text(json.dumps(stages,indent=2),encoding='utf-8')
        assert result.returncode == expected_exit, name+' failed: '+str(log)
        return result.stdout+result.stderr

    try:
        run('source-ledger',[sys.executable,'scripts/web/wasm_parity_ledger.py'])
        run('source-inventory',[sys.executable,'scripts/web/wasm_practitioner_inventory.py'])
        for name, package in packages.items():
            run(name+'-assets',['node','scripts/web/test_toolkit_assets.mjs',str(package/'toolkit')])
            run(name+'-offline',['node','scripts/web/test_offline_package.mjs',str(package)])
        run('zip-format',['node','scripts/web/test_state_batch_zip.mjs'])
        toolkit = str(packages['c-sidecar']/'toolkit')
        cases = [
            ('label-build-application','test_label_build_application_browser.mjs',[]),
            ('native-summary-application','test_native_summary_application_browser.mjs',[]),
            ('label-import','test_label_import_application_browser.mjs',[]),
            ('label-rename','test_label_rename_application_browser.mjs',[]),
            ('native-summary','test_native_label_summary_browser.mjs',[]),
            ('report-pdf','test_report_pdf_browser.mjs',['--json']),
            ('count-transcript','test_count_transcript_installation_browser.mjs',[]),
            ('label-installation','test_label_repository_installation_browser.mjs',['--json']),
        ]
        for name, script, extra in cases:
            run(name,['node','scripts/web/'+script,toolkit,*extra])
        run('tui-verify-fixture',['node','scripts/web/test_tui_verify_wasm.mjs'])
        run('explicit-c-fixture',['node','scripts/web/test_explicit_c_state_batch.mjs',toolkit])
        for script in ['test_generated_report_preview_browser.mjs','test_commission_preview_browser.mjs','test_tui_verify_browser.mjs','test_native_output_verification_browser.mjs','test_native_workspace_browser.mjs','test_explicit_c_state_batch_browser.mjs','test_cffi_engine_browser.mjs']:
            run(script.removesuffix('.mjs'),['node','scripts/web/'+script,toolkit])
        run('full-chamber-followups',['node','scripts/web/test_full_chamber_map_report_browser.mjs','target/full-chamber-matrix/map-report-1792824673295',toolkit])
        run('default-full-chamber-followups',['node','scripts/web/test_full_chamber_map_report_browser.mjs','target/full-chamber-matrix/map-report-1792824673295',str(packages['default']/'toolkit')])
        run('project',['node','scripts/web/test_project.mjs'])
        run('ledger-mutations',[sys.executable,'scripts/web/test_wasm_parity_ledger.py'])
        refusal = run('full-parity-refusal',[sys.executable,'scripts/web/wasm_parity_ledger.py','--full-parity'],1)
        assert 'bisect.Run: Parent evidence has not been verified' in refusal
        assert all(digest(ROOT/name) == identity for name,identity in original.items()), 'Package bytes changed during validation'
        matrix = json.loads((ROOT/'docs/specs/wasm-real-data-matrix.json').read_text(encoding='utf-8'))
        campaign = next(c for c in matrix['national_campaigns'] if c['id']=='national-50-2020-tract-standard-core-42-v1')
        witnesses = []
        for group, keys in [('browser_queue_followup',['c_enabled_core_browser','default_core_browser','saved_refusals','default_saved_refusals']),('map_followup',['c_enabled_staged_browser','default_staged_browser'])]:
            for key in keys:
                ref = campaign[group][key]
                assert digest(local(ref['path'])) == ref['sha256']
                value = json.loads(local(ref['path']).read_text(encoding='utf-8'))
                assert value['status'].startswith('passed')
                assert local(value['package']) in [p/'toolkit' for p in packages.values()]
                for served in value.get('served_assets', []):
                    assert digest(local(served['path'])) == served['sha256'], 'Witness served asset drift'
                witnesses.append(ref)
        bundle = dict(schema_version='bisect-national-browser-package-v1',status='passed',browser_status='passed',package=packages['c-sidecar'].relative_to(ROOT).as_posix(),default_package=packages['default'].relative_to(ROOT).as_posix(),engine_record=pin(record),stages=stages,national_witnesses=witnesses,package_files=[dict(path=name,sha256=identity,byte_length=(ROOT/name).stat().st_size) for name,identity in sorted(original.items())],source_files=[pin(p) for p in (ROOT/'web/lab').iterdir() if p.is_file()],scope='Scoped retained national core queue/map and context/archive/export corrections. Broader source/host/lifecycle and full parity remain open.',full_parity=False,deployed=False)
        output = run_root/'package-evidence.json'
        output.write_text(json.dumps(bundle,indent=2),encoding='utf-8')
        print(json.dumps(pin(output)),flush=True)
    except Exception as error:
        (run_root/'failed-package.json').write_text(json.dumps(dict(status='failed',error=str(error),stages=stages),indent=2),encoding='utf-8')
        raise


if __name__ == '__main__':
    main()
