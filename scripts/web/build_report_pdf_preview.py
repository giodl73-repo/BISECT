"""Stage and exercise a browser-only report change using pinned existing engines.

The normal builders still compile Rust. This scoped acceptance builder reuses
the prior accepted actual build bytes after checking their identities and pinned
native sources; it does not claim a new Rust build or full release acceptance.
"""
import argparse,hashlib,json,os,shutil,subprocess,sys,time
from pathlib import Path

ROOT=Path(__file__).resolve().parents[2]
def local(name):
    value=Path(name);value=(value if value.is_absolute() else ROOT/value).resolve()
    assert value.is_relative_to(ROOT),str(value)
    return value
def digest(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def read(path):return json.loads(Path(path).read_text(encoding='utf-8-sig'))
def pin(path):
    value=local(path);return dict(path=value.relative_to(ROOT).as_posix(),sha256=digest(value),byte_length=value.stat().st_size)

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--base',default='dist/wasm-label-repository-validated')
    parser.add_argument('--output',default='dist/wasm-report-pdf-validated')
    parser.add_argument('--default-wasm',default='target/cffi-legacy-gates/default.wasm')
    parser.add_argument('--native-tool',default='target/release/examples/execute_tool_request.exe')
    parser.add_argument('--native-cli',default='target/release/bisect.exe')
    args=parser.parse_args();base=local(args.base);output=local(args.output)
    assert base.is_relative_to(ROOT/'dist') and output.is_relative_to(ROOT/'dist') and not output.exists()
    baseline=read(ROOT/'target/native-workspace-source/package-evidence.json')
    assert local(baseline['package'])==base and baseline['browser_status']=='passed'
    native=read(local(baseline['label_repository_evidence']['node']['path']))
    host=local(args.native_tool);cli=local(args.native_cli);default=local(args.default_wasm)
    rcount=local('target/release/rcount.exe');assert rcount.is_file(),'Build the actual RCOUNT CLI before transcript acceptance'
    assert digest(host)==native['native_host_sha256'] and digest(cli)==native['native_sha256'],'Reused native references differ from the accepted baseline'
    profiles={}
    for name in ['default','c-sidecar']:
        gate=read(ROOT/'target/cffi-legacy-gates'/name/'build.json');assert gate['status']=='passed'
        wasm=default if name=='default' else base/'toolkit/bisect_wasm.wasm'
        assert digest(wasm)==gate['wasm_sha256'];profiles[name]=dict(wasm=str(wasm),identity=gate['wasm_sha256'],baseline_gate=pin(ROOT/'target/cffi-legacy-gates'/name/'build.json'))
    # Read the accepted snapshot through its ledger digest, not a mutable directory guess.
    inventory=read(ROOT/'docs/specs/wasm-capability-inventory.json')
    entry=next(row for row in inventory['entries'] if row['id']=='bisect.LabelAnalyze')
    previous=next(row for row in entry['parity_record']['contracts'] if row['contract_id'].endswith('/reviewed-repository-application'))['implementation_evidence']['immutable_local_snapshot']
    snapshot=local(previous['path']);assert digest(snapshot)==previous['sha256'];manifest=read(snapshot)
    authority_row=next(row for row in manifest['files'] if row['path']=='docs/specs/wasm-practitioner-actions.json')
    authority_blob=snapshot.parent/authority_row['blob'];assert digest(authority_blob)==authority_row['sha256']
    native_pins=[row for row in read(authority_blob)['authority'] if row['path'].startswith('crates/')]
    for row in native_pins:assert digest(local(row['path']))==row['sha256'],'Pinned native source changed; use a normal fresh build: '+row['path']
    base_size=sum(file.stat().st_size for file in base.rglob('*') if file.is_file())
    overlay=[file for file in (ROOT/'web/lab').iterdir() if file.is_file() and file.name not in ['index.html','bisect_wasm.wasm']]
    required=base_size+2*sum(file.stat().st_size for file in overlay)+64*1024*1024
    assert shutil.disk_usage(ROOT).free>=required,'Insufficient capacity for fresh package, overlay and browser headroom'
    run_root=ROOT/'target/report-pdf-delivery'/('build-'+str(time.time_ns()));run_root.mkdir(parents=True)
    env=dict(os.environ,BISECT_TEST_NATIVE_TOOL=str(host),BISECT_TEST_NATIVE_CLI=str(cli),BISECT_CFFI_BROWSER_EVIDENCE=str(run_root/'cffi-browser'))
    stages=[]
    def run(name,command,*,wasm=None,json_output=False):
        print('Report PDF gate: '+name,flush=True)
        current=dict(env)
        if wasm:current['BISECT_TEST_WASM']=str(wasm)
        result=subprocess.run(command,cwd=ROOT,env=current,encoding='utf-8',errors='replace',capture_output=True)
        log=run_root/(name+'.log');log.write_text(result.stdout+'\n'+result.stderr,encoding='utf-8')
        stages.append(dict(name=name,exit_code=result.returncode,log=pin(log)))
        if result.returncode:raise RuntimeError(name+' failed; see '+str(log))
        return json.loads(result.stdout) if json_output else None
    try:
        run('source-inventory',[sys.executable,'scripts/web/wasm_practitioner_inventory.py'])
        run('source-ledger',[sys.executable,'scripts/web/wasm_parity_ledger.py'])
        run('print-observer',['node','scripts/web/test_report_print.mjs'])
        run('base-offline',['node','scripts/web/test_offline_package.mjs',str(base)])
        for name,profile in profiles.items():
            profile['current_browser']=run('pdf-'+name,['node','scripts/web/test_report_pdf_browser.mjs','--json'],wasm=profile['wasm'],json_output=True)
            profile['count_node']=run('count-node-'+name,['node','scripts/web/test_count_transcript_installation.mjs'],wasm=profile['wasm'],json_output=True)
            profile['count_browser']=run('count-browser-'+name,['node','scripts/web/test_count_transcript_installation_browser.mjs'],wasm=profile['wasm'],json_output=True)
        shutil.copytree(base,output)
        for folder in [output,output/'toolkit']:
            for file in overlay:shutil.copy2(file,folder/file.name)
        shutil.copy2(ROOT/'web/lab/workbench.html',output/'toolkit/index.html')
        run('proof-assets',[sys.executable,'scripts/web/stage_native_proof_preview.py','--destination',str(output/'toolkit')])
        if os.name=='nt':
            run('filesystem-compression',[str(Path(os.environ.get('SystemRoot','C:/Windows'))/'System32/compact.exe'),'/C','/S:'+str(output),'/I','/Q'])
        assert digest(output/'toolkit/bisect_wasm.wasm')==profiles['c-sidecar']['identity']
        run('offline-manifest',[sys.executable,'scripts/web/build_offline_manifest.py',str(output)])
        run('toolkit-assets',['node','scripts/web/test_toolkit_assets.mjs',str(output/'toolkit')])
        run('final-offline',['node','scripts/web/test_offline_package.mjs',str(output)])
        final=run('pdf-final',['node','scripts/web/test_report_pdf_browser.mjs',str(output/'toolkit'),'--json'],wasm=output/'toolkit/bisect_wasm.wasm',json_output=True)
        count_final=run('count-final',['node','scripts/web/test_count_transcript_installation_browser.mjs',str(output/'toolkit')],wasm=output/'toolkit/bisect_wasm.wasm',json_output=True)
        for name in ['test_generated_report_files.mjs','test_commission_preview_browser.mjs','test_generated_report_preview_browser.mjs']:
            command=['node','scripts/web/'+name]+([str(output/'toolkit')] if 'browser' in name else [])
            run(name.removesuffix('.mjs'),command,wasm=output/'toolkit/bisect_wasm.wasm')
        label=run('label-application',['node','scripts/web/test_label_repository_installation_browser.mjs',str(output/'toolkit'),'--json'],wasm=output/'toolkit/bisect_wasm.wasm',json_output=True)
        for name in ['test_tui_verify_browser.mjs','test_native_output_verification_browser.mjs','test_native_workspace_browser.mjs','test_explicit_c_state_batch_browser.mjs','test_cffi_engine_browser.mjs']:
            run(name.removesuffix('.mjs'),['node','scripts/web/'+name,str(output/'toolkit')],wasm=output/'toolkit/bisect_wasm.wasm')
        run('project',['node','scripts/web/test_project.mjs'])
        run('ledger-mutations',[sys.executable,'scripts/web/test_wasm_parity_ledger.py'])
        run('consumption-mutations',[sys.executable,'scripts/web/test_wasm_native_consumption.py'])
        refusal=subprocess.run([sys.executable,'scripts/web/wasm_parity_ledger.py','--full-parity'],cwd=ROOT,capture_output=True,text=True)
        assert refusal.returncode!=0 and 'bisect.Run: Parent evidence has not been verified' in refusal.stdout+refusal.stderr
        (run_root/'full-parity-refusal.log').write_text(refusal.stdout+'\n'+refusal.stderr,encoding='utf-8')
        source_names=['scripts/web/build_report_pdf_preview.py','scripts/web/test_report_pdf_browser.mjs','scripts/web/test_report_print.mjs','scripts/web/fixtures/report-pdf-case.mjs','scripts/web/validate_report_print_pdf.py','scripts/web/test_commission_preview_browser.mjs','scripts/web/test_generated_report_preview_browser.mjs','scripts/web/test_generated_report_files.mjs','scripts/web/build_wasm.ps1','scripts/web/build_wasm_lab.ps1','scripts/web/build_workspace_preview.ps1','scripts/web/check_cffi_legacy_gates.ps1','scripts/web/build_cffi_preview.ps1','scripts/web/wasm_practitioner_inventory.py','docs/specs/wasm-practitioner-actions.json']
        source_names+=['scripts/web/test_count_transcript_installation.mjs','scripts/web/test_count_transcript_installation_browser.mjs','crates/rcount-cli/src/main.rs','crates/rcount-audit/src/lib.rs','Cargo.lock']
        source_files=[pin(file) for file in (ROOT/'web/lab').iterdir() if file.is_file() and file.suffix in ['.js','.html','.css']]+[pin(name) for name in source_names]
        results=[pin(ROOT/'target/generated-report-files/evidence.json'),pin(ROOT/'target/commission-preview-browser/evidence.json'),pin(ROOT/'target/generated-report-preview-browser/evidence.json'),*[pin(ROOT/'target/tui-verify-browser'/name) for name in ['receipt.json','prechecks.bisect','replayed.bisect','threshold-not-met.bisect','blocked.bisect']],pin(ROOT/'target/native-output-verification-browser/evidence.json'),pin(ROOT/'target/native-workspace-browser/evidence.json'),pin(ROOT/'target/explicit-c-state-batch-browser/evidence.json'),pin(run_root/'cffi-browser/evidence.json')]
        evidence=dict(schema_version='bisect-report-pdf-preview-v1',package=output.relative_to(ROOT).as_posix(),browser_status='passed',engine_build_mode='Reused prior verified actual default/C-enabled artifacts; no Rust compilation or source-to-binary reproduction claim in this UI slice.',baseline_package=pin(base/'offline-manifest.json'),baseline_snapshot=previous,baseline_native_source_pins_verified=native_pins,native_cli=pin(cli),native_host=pin(host),profiles=profiles,final_browser=final,label_browser=label,source_files=source_files,results=results,stages=stages,offline_manifest=pin(output/'offline-manifest.json'),offline_version=read(output/'offline-manifest.json')['version'],full_parity=False,deployed=False)
        evidence['count_transcript_delivery']=dict(native_cli=pin(rcount),final_browser=count_final,scope='Source-preserving optional reviewed transcript addition/replacement, actual native CLI bytes and write failure. No official certification or OS atomicity claim.')
        record=run_root/'package-evidence.json';record.write_text(json.dumps(evidence,indent=2)+'\n',encoding='utf-8')
        (ROOT/'target/report-pdf-delivery/latest-package-record.json').write_text(json.dumps(dict(record=record.relative_to(ROOT).as_posix(),sha256=digest(record)),indent=2)+'\n',encoding='utf-8')
        print(json.dumps(dict(package=str(output),record=str(record),final_pdf_root=final['root'],offline_version=evidence['offline_version'])),flush=True)
    except Exception as error:
        (run_root/'failed-build.json').write_text(json.dumps(dict(status='failed',error=str(error),output=str(output),stages=stages),indent=2)+'\n',encoding='utf-8');raise

if __name__=='__main__':main()
