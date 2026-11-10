"""Stage reviewed ordered summary UI with verified engine identities and test the exact package."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
from evidence_blob_store import ROOT, local, digest
from validate_label_rename_engine_record import validate as validate_engine_record


def read(path):
    return json.loads(Path(path).read_text(encoding='utf-8-sig'))


def pin(path):
    path = local(path)
    return dict(path=path.relative_to(ROOT).as_posix(), sha256=digest(path), byte_length=path.stat().st_size)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--engine-record', required=True)
    parser.add_argument('--base', default='dist/wasm-label-import-validated')
    parser.add_argument('--output', default='dist/wasm-native-summary-application-validated')
    parser.add_argument('--verify-existing', help='Explicit prior failed-build record for unchanged staged bytes')
    args = parser.parse_args()
    record = local(args.engine_record)
    engines, _engine_reference = validate_engine_record(record)
    assert engines['status'] == 'passed' and all(stage['exit_code'] == 0 for stage in engines['stages'])
    for ref in engines['source_files']:
        assert digest(local(ref['path'])) == ref['sha256'], 'Engine acceptance source drift: ' + ref['path']
    base, output = local(args.base), local(args.output)
    assert base.is_relative_to(ROOT/'dist') and output.is_relative_to(ROOT/'dist')
    prior_failed = local(args.verify_existing) if args.verify_existing else None
    if prior_failed:
        failure = read(prior_failed)
        assert output.is_dir() and failure['status']=='failed'
        successful_browser = next(stage for stage in failure['stages'] if stage['name']=='native-summary-application-browser' and stage['exit_code']==0)
        assert digest(local(successful_browser['log']['path']))==successful_browser['log']['sha256']
        original_browser = json.loads(local(successful_browser['log']['path']).read_text(encoding='utf-8'))
        original_evidence = read(local(original_browser['root'])/'evidence.json')
        assert local(original_evidence['package'])==output/'toolkit' and original_evidence['source_tree'] is False
        for ref in original_evidence['served_assets']:
            assert digest(local(ref['path']))==ref['sha256'], 'Staged runtime changed since failed attempt'
        # Bind every staged runtime file before re-verifying; no staged bytes
        # are rewritten in this mode and the old failure record is preserved.
        for source in (ROOT/'web/lab').iterdir():
            if source.is_file() and source.name not in ['index.html','bisect_wasm.wasm']:
                for directory in [output,output/'toolkit']:
                    assert digest(directory/source.name)==digest(source)
        assert digest(output/'toolkit/index.html')==digest(ROOT/'web/lab/workbench.html')
        original_catalog, candidate_catalog = read(base/'catalog.json'), read(output/'catalog.json')
        candidate_catalog['wasm_sha256']=original_catalog['wasm_sha256']
        assert candidate_catalog==original_catalog, 'Historical catalog/data records changed'
    else:
        assert not output.exists()
    legacy = {}
    for name in ['default','c-sidecar']:
        gate = ROOT/'target/cffi-legacy-gates'/name/'build.json'
        value = read(gate)
        assert value['status'] == 'passed' and value['native_summary_evidence']['status'] == 'passed'
        assert value['native_summary_evidence']['browser_status'] == 'passed'
        assert value['label_rename_evidence']['status']=='passed' and value['label_rename_evidence']['browser_status']=='passed'
        assert value['native_summary_application_evidence']['status']=='passed' and value['native_summary_application_evidence']['browser_status']=='passed'
        assert value['label_import_evidence']['status']=='passed' and value['label_import_evidence']['browser_status']=='passed'
        wasm = gate.parent.parent/(name+'.wasm')
        assert digest(wasm) == value['wasm_sha256']
        assert digest(wasm)==engines['profiles'][name]['sha256'], 'Gate runtime differs from supplied owned build profile'
        for ref in value['source_files']:
            assert digest(local(ref['path'])) == ref['sha256'], 'Legacy gate source drift: ' + ref['path']
        legacy[name] = dict(record=pin(gate), wasm=pin(wasm), summary=value['native_summary_evidence'],rename=value['label_rename_evidence'],imported=value['label_import_evidence'],application=value['native_summary_application_evidence'])
    for kind in ['native_cli','native_host','native_summary_reference']:
        assert digest(local(engines[kind]['path'])) == engines[kind]['sha256']
    assert digest(ROOT/'target/release/examples/label_summary_application_reference.exe')==engines['native_summary_reference']['sha256'], 'Current native summary helper differs from tested reference'
    overlay = [file for file in (ROOT/'web/lab').iterdir() if file.is_file() and file.name not in ['index.html','bisect_wasm.wasm']]
    required = sum(file.stat().st_size for file in base.rglob('*') if file.is_file()) + 2*sum(file.stat().st_size for file in overlay) + 96*1024*1024
    assert shutil.disk_usage(ROOT).free >= (64*1024*1024 if prior_failed else required), 'Package capacity refusal before writing output'
    run_root = ROOT/'target/native-summary-application'/('package-'+str(time.time_ns()))
    run_root.mkdir()
    stages = []
    env = dict(os.environ, BISECT_TEST_NATIVE_TOOL=str(local(engines['native_host']['path'])), BISECT_TEST_NATIVE_CLI=str(local(engines['native_cli']['path'])), BISECT_CFFI_BROWSER_EVIDENCE=str(run_root/'cffi-browser'))

    def run(name, arguments, *, wasm=None, json_output=False):
        print('Summary application package gate: ' + name, flush=True)
        current = dict(env)
        if wasm:
            current['BISECT_TEST_WASM'] = str(wasm)
        result = subprocess.run(arguments, cwd=ROOT, env=current, capture_output=True, encoding='utf-8', errors='replace')
        log = run_root/(name+'.log')
        log.write_text(result.stdout+'\n'+result.stderr, encoding='utf-8')
        stages.append(dict(name=name, exit_code=result.returncode, log=pin(log)))
        if result.returncode:
            raise RuntimeError(name+' failed: '+str(log))
        return json.loads(result.stdout) if json_output else None

    try:
        run('source-ledger',[sys.executable,'scripts/web/wasm_parity_ledger.py'])
        run('source-inventory',[sys.executable,'scripts/web/wasm_practitioner_inventory.py'])
        run('base-offline',['node','scripts/web/test_offline_package.mjs',str(base)])
        if not prior_failed:
            shutil.copytree(base,output)
            for directory in [output,output/'toolkit']:
                for source in overlay:
                    shutil.copy2(source,directory/source.name)
                shutil.copy2(local(legacy['c-sidecar']['wasm']['path']),directory/'bisect_wasm.wasm')
            # Current served identity only; historical producer rows stay intact.
            for directory in [output,output/'toolkit']:
                catalog_path = directory/'catalog.json'
                if catalog_path.is_file():
                    catalog = read(catalog_path)
                    catalog['wasm_sha256'] = legacy['c-sidecar']['wasm']['sha256']
                    catalog_path.write_text(json.dumps(catalog,indent=2)+'\n',encoding='utf-8')
            shutil.copy2(ROOT/'web/lab/workbench.html',output/'toolkit/index.html')
            run('proof-assets',[sys.executable,'scripts/web/stage_native_proof_preview.py','--destination',str(output/'toolkit')])
            if os.name == 'nt':
                run('filesystem-compression',[str(Path(os.environ['SystemRoot'])/'System32/compact.exe'),'/C','/S:'+str(output),'/I','/Q'])
            run('offline-manifest',[sys.executable,'scripts/web/build_offline_manifest.py',str(output)])
        assert digest(output/'bisect_wasm.wasm')==digest(output/'toolkit/bisect_wasm.wasm')==legacy['c-sidecar']['wasm']['sha256']
        run('assets',['node','scripts/web/test_toolkit_assets.mjs',str(output/'toolkit')])
        run('offline',['node','scripts/web/test_offline_package.mjs',str(output)])
        wasm = output/'toolkit/bisect_wasm.wasm'
        final = run('native-summary-application-browser',['node','scripts/web/test_native_summary_application_browser.mjs',str(output/'toolkit')],wasm=wasm,json_output=True)
        imported = run('label-import-browser',['node','scripts/web/test_label_import_application_browser.mjs',str(output/'toolkit')],wasm=wasm,json_output=True)
        rename = run('label-rename-browser',['node','scripts/web/test_label_rename_application_browser.mjs',str(output/'toolkit')],wasm=wasm,json_output=True)
        summary = run('native-summary-browser',['node','scripts/web/test_native_label_summary_browser.mjs',str(output/'toolkit')],wasm=wasm,json_output=True)
        pdf = run('report-pdf-browser',['node','scripts/web/test_report_pdf_browser.mjs',str(output/'toolkit'),'--json'],wasm=wasm,json_output=True)
        count = run('count-transcript-browser',['node','scripts/web/test_count_transcript_installation_browser.mjs',str(output/'toolkit')],wasm=wasm,json_output=True)
        label = run('label-installation-browser',['node','scripts/web/test_label_repository_installation_browser.mjs',str(output/'toolkit'),'--json'],wasm=wasm,json_output=True)
        run('tui-verify-current-runtime-fixture',['node','scripts/web/test_tui_verify_wasm.mjs'],wasm=wasm)
        run('explicit-c-current-runtime-fixture',['node','scripts/web/test_explicit_c_state_batch.mjs',str(output/'toolkit')],wasm=wasm)
        for script in ['test_generated_report_preview_browser.mjs','test_commission_preview_browser.mjs','test_tui_verify_browser.mjs','test_native_output_verification_browser.mjs','test_native_workspace_browser.mjs','test_explicit_c_state_batch_browser.mjs','test_cffi_engine_browser.mjs']:
            run(script.removesuffix('.mjs'),['node','scripts/web/'+script,str(output/'toolkit')],wasm=wasm)
        run('project',['node','scripts/web/test_project.mjs'])
        run('ledger-mutations',[sys.executable,'scripts/web/test_wasm_parity_ledger.py'])
        run('consumption-mutations',[sys.executable,'scripts/web/test_wasm_native_consumption.py'])
        refused = subprocess.run([sys.executable,'scripts/web/wasm_parity_ledger.py','--full-parity'],cwd=ROOT,capture_output=True,text=True)
        assert refused.returncode != 0 and 'bisect.Run: Parent evidence has not been verified' in refused.stdout+refused.stderr
        (run_root/'full-parity-refusal.log').write_text(refused.stdout+refused.stderr,encoding='utf-8')
        # Bind captured current source pins to the exact staged bytes tested above.
        for directory in [output,output/'toolkit']:
            for source in overlay:
                assert digest(directory/source.name)==digest(source), 'Source changed after staging: '+source.name
        assert digest(output/'toolkit/index.html')==digest(ROOT/'web/lab/workbench.html')
        source_names = ['scripts/web/build_native_summary_application_preview.py','scripts/web/build_native_summary_application_engines.py','scripts/web/test_native_summary_application.mjs','scripts/web/test_native_summary_application_browser.mjs','crates/bisect-cli/examples/label_summary_application_reference.rs','scripts/web/build_label_import_preview.py','scripts/web/test_label_import_application.mjs','scripts/web/test_label_import_application_browser.mjs','crates/bisect-cli/src/import_label.rs','crates/bisect-wasm/src/label_import.rs','crates/bisect-report/src/label_import.rs','scripts/web/test_label_rename_engine_record.py','scripts/web/validate_label_rename_engine_record.py','scripts/web/build_label_rename_engines.py','scripts/web/test_native_case_alias_cli.py','crates/bisect-report/src/label_rename.rs','crates/bisect-cli/src/label_cmd.rs','scripts/web/build_label_rename_preview.py','scripts/web/test_label_rename_application.mjs','scripts/web/test_label_rename_application_browser.mjs','scripts/web/fixtures/native-output-directory.mjs','scripts/web/build_native_label_summary_preview.py','scripts/web/build_native_label_summary_engines.py','scripts/web/test_native_label_summary_cli.py','scripts/web/test_native_label_summary_wasm.mjs','scripts/web/test_native_label_summary_batch.mjs','scripts/web/test_native_label_summary_browser.mjs','scripts/web/check_cffi_legacy_gates.ps1','scripts/web/build_workspace_preview.ps1','scripts/web/build_wasm.ps1','scripts/web/build_wasm_lab.ps1','scripts/web/wasm_practitioner_inventory.py','docs/specs/wasm-practitioner-actions.json','crates/bisect-report/src/label_summary.rs','crates/bisect-report/tests/label_summary_security.rs','crates/bisect-wasm/src/native_label_summary.rs','crates/bisect-cli/src/analyze_label.rs']
        sources = [pin(file) for file in (ROOT/'web/lab').iterdir() if file.is_file() and file.suffix in ['.js','.html','.css']] + [pin(name) for name in source_names]
        evidence = dict(schema_version='bisect-native-summary-application-preview-v1',package=output.relative_to(ROOT).as_posix(),browser_status='passed',engine_record=pin(record),legacy_gates=legacy,native_cli=engines['native_cli'],native_host=engines['native_host'],native_summary_reference=engines['native_summary_reference'],final_browser=final,import_browser=imported,rename_browser=rename,summary_browser=summary,pdf_browser=pdf,count_browser=count,label_browser=label,source_files=sources,stages=stages,offline_manifest=pin(output/'offline-manifest.json'),offline_version=read(output/'offline-manifest.json')['version'],full_parity=False,deployed=False)
        if prior_failed:
            evidence['unchanged_package_reverification']=dict(original_failed_build=pin(prior_failed),scope='Fresh complete verification after current-runtime fixture regeneration; staged package bytes were not changed or re-versioned and original failures remain retained.')
        destination = run_root/'package-evidence.json'
        destination.write_text(json.dumps(evidence,indent=2)+'\n',encoding='utf-8')
        print(json.dumps(dict(record=destination.relative_to(ROOT).as_posix(),sha256=digest(destination),package=str(output))),flush=True)
    except Exception as error:
        (run_root/'failed-build.json').write_text(json.dumps(dict(status='failed',package=output.relative_to(ROOT).as_posix(),error=str(error),stages=stages),indent=2)+'\n',encoding='utf-8')
        raise


if __name__ == '__main__':
    main()
