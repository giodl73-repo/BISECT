"""Fresh native/default/C build-application source checks; no package acceptance."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import time
from evidence_blob_store import ROOT, digest


def main():
    assert shutil.disk_usage(ROOT).free >= 2*1024**3, 'Need 2 GiB for owned fresh builds'
    root=ROOT/'target/build-application'/('engines-'+str(time.time_ns()))
    root.mkdir()
    print('Build evidence: '+str(root),flush=True)
    stages=[]
    def pin(p):
        return dict(path=str(p.relative_to(ROOT)),sha256=digest(p),byte_length=p.stat().st_size)
    def run(name,args,extra=None):
        print('Build gate: '+name,flush=True)
        env=dict(os.environ)
        env.update(extra or {})
        result=subprocess.run(args,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace')
        log=root/(name+'.log');log.write_text(result.stdout+'\n'+result.stderr,encoding='utf-8')
        stages.append(dict(name=name,args=args,exit_code=result.returncode,log=pin(log)))
        (root/'stages.json').write_text(json.dumps(stages,indent=2)+'\n',encoding='utf-8')
        if result.returncode:raise RuntimeError(name+' failed: '+str(log))
        return result.stdout
    try:
        run('native-cli',['cargo','build','-p','bisect-cli','--bin','bisect','--release','--no-default-features','--locked','--offline','-j','1'])
        shutil.copy2(ROOT/'target/release/bisect.exe',root/'bisect.exe')
        run('native-host',['cargo','build','-p','bisect-wasm','--release','--example','execute_tool_request','--locked','--offline','-j','1'])
        shutil.copy2(ROOT/'target/release/examples/execute_tool_request.exe',root/'execute_tool_request.exe')
        profiles={}
        for profile in ['default','c-sidecar']:
            args=['cargo','build','-p','bisect-wasm','--release','--target','wasm32-unknown-unknown','--locked','--offline','-j','1']
            if profile=='c-sidecar':args+=['--features','c-metis-sidecar']
            run(profile+'-build',args)
            wasm=root/(profile+'.wasm');shutil.copy2(ROOT/'target/wasm32-unknown-unknown/release/bisect_wasm.wasm',wasm)
            env={'BISECT_TEST_WASM':str(wasm),'BISECT_TEST_NATIVE_CLI':str(root/'bisect.exe'),'BISECT_TEST_NATIVE_TOOL':str(root/'execute_tool_request.exe')}
            direct=run(profile+'-application',['node','scripts/web/test_label_build_application.mjs'],env)
            profiles[profile]=dict(artifact=pin(wasm),application=json.loads(direct))
        sources=['web/lab/label-build-application.js','web/lab/build-repository.js','web/lab/label-build-application-ui.js','web/lab/label-build-files.js','web/lab/label-build-archive.js','web/lab/project.js','web/lab/workbench.js','web/lab/workbench.html','crates/bisect-wasm/src/label_build.rs','crates/bisect-wasm/src/engine_build.rs','crates/bisect-wasm/src/config.rs','crates/bisect-wasm/src/toolkit.rs','crates/bisect-cli/src/runner/mod.rs','crates/bisect-runner/src/balance_retry.rs','scripts/web/test_label_build_application.mjs','scripts/web/test_label_build_application_ui.mjs','scripts/web/build_label_build_application_engines.py','Cargo.lock']
        evidence=dict(status='passed',stages=stages,profiles=profiles,native_cli=pin(root/'bisect.exe'),native_host=pin(root/'execute_tool_request.exe'),source_files=[pin(ROOT/n) for n in sources],rustc=subprocess.check_output(['rustc','-Vv'],text=True),scope='Fresh CLI/native-host and actual default/C direct application checks. Actual browser UI/worker cancellation, final package and host acceptance remain separate.',full_parity=False)
        record=root/'evidence.json';record.write_text(json.dumps(evidence,indent=2)+'\n',encoding='utf-8');print(json.dumps(pin(record)),flush=True)
    except Exception as error:
        (root/'failed-build.json').write_text(json.dumps(dict(error=str(error),stages=stages),indent=2)+'\n',encoding='utf-8')
        raise


if __name__=='__main__':main()
