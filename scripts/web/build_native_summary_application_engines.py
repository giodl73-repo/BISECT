"""Build fresh summary-application/native-clock and preserved default/C engines."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
from evidence_blob_store import ROOT, digest


def main():
    root = ROOT / 'target/native-summary-application' / ('engines-' + str(time.time_ns()))
    assert shutil.disk_usage(ROOT).free>=2*1024**3, 'Engine build capacity refusal: require 2 GiB before writes'
    root.mkdir()
    print('Engine evidence: ' + str(root), flush=True)
    stages = []

    def run(name, arguments, *, wasm=None):
        print('Summary application engine gate: ' + name, flush=True)
        env = dict(os.environ)
        if wasm:
            env['BISECT_TEST_WASM'] = str(wasm)
        result = subprocess.run(arguments, cwd=ROOT, env=env, capture_output=True, text=True, encoding='utf-8', errors='replace')
        log = root / (name + '.log')
        log.write_text(result.stdout + '\n' + result.stderr, encoding='utf-8')
        stages.append(dict(name=name, arguments=arguments, wasm=str(wasm) if wasm else None, exit_code=result.returncode, log=str(log.relative_to(ROOT)), sha256=digest(log)))
        if result.returncode:
            raise RuntimeError(name + ' failed; inspect ' + str(log))
        return result.stdout

    try:
        run('shared-rename-kernel',['cargo','test','--release','-p','bisect-report','--no-default-features','--lib','label_rename','--locked','--offline','-j','1'])
        run('native-rename-tests',['cargo','test','--release','-p','bisect-cli','--no-default-features','--lib','label_cmd::tests','--locked','--offline','-j','1','--','--test-threads=1'])
        run('native-report-clock-tests',['cargo','test','--release','-p','bisect-cli','--no-default-features','--lib','analyze_label::tests','--locked','--offline','-j','1','--','--test-threads=1'])
        run('shared-kernel', ['cargo', 'test', '--release', '-p', 'bisect-report', '--no-default-features', '--lib', 'label_summary', '--locked', '--offline', '-j', '1'])
        run('native-cli', ['cargo', 'build', '-p', 'bisect-cli', '--bin','bisect','--example','label_summary_application_reference','--release', '--no-default-features', '--locked', '--offline', '-j', '1'])
        run('native-case-alias-cli',[sys.executable,'scripts/web/test_native_case_alias_cli.py'])
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
            rename = json.loads(run(profile+'-rename',['node','scripts/web/test_label_rename_application.mjs'],wasm=destination))
            application=json.loads(run(profile+'-summary-application',['node','scripts/web/test_native_summary_application.mjs'],wasm=destination))
            application_browser=json.loads(run(profile+'-summary-application-browser',['node','scripts/web/test_native_summary_application_browser.mjs'],wasm=destination))
            profiles[profile] = dict(application=application,application_browser=application_browser,rename=rename,path=str(destination.relative_to(ROOT)), sha256=digest(destination), byte_length=destination.stat().st_size, direct=direct, batch=batch)
        names = ['scripts/web/build_native_summary_application_engines.py','scripts/web/test_native_summary_application.mjs','scripts/web/test_native_summary_application_browser.mjs','web/lab/native-summary-application.js','web/lab/native-summary-application-ui.js','web/lab/project.js','web/lab/workbench.js','web/lab/workbench.html','crates/bisect-cli/examples/label_summary_application_reference.rs','crates/bisect-report/src/label_rename.rs','crates/bisect-cli/src/label_cmd.rs','crates/bisect-cli/src/run_registry.rs','crates/bisect-wasm/src/label_rename.rs','scripts/web/test_native_case_alias_cli.py','scripts/web/test_label_rename_application.mjs','scripts/web/test_label_rename_application_browser.mjs','scripts/web/build_label_rename_engines.py','web/lab/label-rename-application.js','web/lab/label-rename-application-ui.js','crates/bisect-report/src/label_summary.rs', 'crates/bisect-report/src/lib.rs', 'crates/bisect-cli/src/analyze_label.rs', 'crates/bisect-wasm/src/native_label_summary.rs', 'crates/bisect-wasm/src/lib.rs', 'crates/bisect-wasm/src/toolkit.rs', 'Cargo.lock', 'scripts/web/test_native_label_summary_cli.py', 'scripts/web/test_native_label_summary_wasm.mjs', 'scripts/web/test_native_label_summary_batch.mjs', 'scripts/web/build_native_label_summary_engines.py', 'web/lab/native-label-summary.js', 'web/lab/label-report-files.js']
        pin = lambda name: dict(path=name, sha256=digest(ROOT/name), byte_length=(ROOT/name).stat().st_size)
        for source,name in [(ROOT/'target/release/examples/label_summary_application_reference.exe','label_summary_application_reference.exe'),(ROOT/'target/release/bisect.exe','bisect.exe'),(ROOT/'target/release/examples/execute_tool_request.exe','execute_tool_request.exe')]:
            shutil.copy2(source,root/name)
        evidence = dict(source_revision=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),status='passed', profiles=profiles, stages=stages, native_summary_reference=pin(str((root/'label_summary_application_reference.exe').relative_to(ROOT))),native_cli=pin(str((root/'bisect.exe').relative_to(ROOT))), native_host=pin(str((root/'execute_tool_request.exe').relative_to(ROOT))), source_files=[pin(name) for name in names], rustc=subprocess.check_output(['rustc','-Vv'], text=True), cargo=subprocess.check_output(['cargo','-V'], text=True), full_parity=False, scope='Fresh native report clock regressions, exact recorded-clock filesystem reference, ordered summary writer/source browsers; shared native rename case-alias safety and summary kernels, serial native label tests, four actual CLI case-only refusals preserving all stages, actual default/C rename files/interruption cases and summary ordering. Browser UI, final package, matrix and release acceptance remain separate.')
        record = root/'evidence.json'
        record.write_text(json.dumps(evidence, indent=2)+'\n', encoding='utf-8')
        print(json.dumps(dict(record=str(record.relative_to(ROOT)), sha256=digest(record))), flush=True)
    except Exception as error:
        (root/'failed-build.json').write_text(json.dumps(dict(status='failed', error=str(error), stages=stages), indent=2)+'\n', encoding='utf-8')
        raise


if __name__ == '__main__':
    main()
