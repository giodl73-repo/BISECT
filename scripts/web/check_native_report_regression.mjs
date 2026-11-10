// Integration gate: create fresh CLI/reference snapshots before consuming them.
// Prior report-native case/snapshot outputs are not reused. Browser staging
// separately requires the existing hash-verified accepted package base.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
const args=process.argv.slice(2),profile=args.find(a=>a.startsWith('--profile='))?.slice(10),browser=args.includes('--browser');
assert(['default','c-sidecar'].includes(profile)&&args.every(a=>a==='--browser'||a==='--profile='+profile));
const wasm=process.env.BISECT_TEST_WASM;assert(wasm,'Select the exact gate WASM through BISECT_TEST_WASM');
const base=path.resolve('target/state-report-regression');fs.mkdirSync(base,{recursive:true});const root=fs.mkdtempSync(path.join(base,profile+'-'));
const sha=bytes=>createHash('sha256').update(bytes).digest('hex'),pin=file=>{const absolute=path.resolve(file),bytes=fs.readFileSync(absolute);return {path:path.relative(process.cwd(),absolute).replaceAll('\\','/'),sha256:sha(bytes),byte_length:bytes.length};};
const runtime=pin(wasm),stages=[],environment={...process.env,BISECT_REPORT_PROFILE:profile};
// Explicit generated inputs replace any inherited local replay settings.
delete environment.BISECT_REPORT_SOURCE_EVIDENCE;delete environment.BISECT_POPULATION_NATIVE_REFERENCE;
delete environment.BISECT_REPORT_C_TOOLKIT;delete environment.BISECT_REPORT_PACKAGE_RECORD;
function write(status,error){fs.writeFileSync(path.join(root,'progress.json'),JSON.stringify({status,profile,wasm:runtime,stages,...(error?{error}:{} )},null,2)+'\n');}
function run(name,command,arguments_){
 const stage={name,command,arguments:arguments_,status:'running'};stages.push(stage);write('running');
 const result=spawnSync(command,arguments_,{env:environment,encoding:'utf8',windowsHide:true,maxBuffer:32*1024*1024});
 const log=path.join(root,name+'.log');fs.writeFileSync(log,(result.stdout??'')+(result.stderr??''));stage.log=pin(log);stage.exit_code=result.status;stage.status=result.status===0?'passed':'failed';write(stage.status==='failed'?'failed':'running');
 if(result.error)throw result.error;assert.equal(result.status,0,name+' failed; see '+log);
 return (result.stdout??'').trim();
}
try{
 const nativeArguments=['scripts/web/prepare_population_source_native_reference.py'];if(environment.BISECT_REPORT_NATIVE_CLI)nativeArguments.push('--native',environment.BISECT_REPORT_NATIVE_CLI);
 const nativeRoot=run('native-report-reference','python',nativeArguments);environment.BISECT_POPULATION_NATIVE_REFERENCE=path.join(nativeRoot,'evidence.json');
 const preparation=JSON.parse(run('source-preparation','node',['scripts/web/test_native_report_source_snapshot.mjs']));environment.BISECT_REPORT_SOURCE_EVIDENCE=path.join(preparation.root,'evidence.json');
 const binding=JSON.parse(run('export-binding','node',['scripts/web/test_native_report_export_binding.mjs']));
 const workflow=JSON.parse(run('state-workflow','node',['scripts/web/test_native_report_state_workflow.mjs']));
 let stage=null,portable=null,folder=null,importOracle=null;
 if(browser){
  const observed=JSON.parse(run('import-oracle','node',['scripts/web/test_observed_project_open.mjs']));importOracle=pin(path.join(observed.root,'evidence.json'));
  stage=run('stage-package','python',['scripts/web/stage_state_identity_preview.py','--wasm',wasm,'--profile',profile,'--workflow']);
  const toolkit=path.join(stage,'package/toolkit');portable=JSON.parse(run('portable-ui','node',['scripts/web/test_native_report_source_ui_browser.mjs',toolkit]));folder=JSON.parse(run('folder-api-ui','node',['scripts/web/test_native_report_folder_browser.mjs',toolkit]));
 }
 assert.deepEqual(pin(wasm),runtime,'Gate engine changed while checking report sources');
 const evidence={status:'passed',full_parity:false,profile,wasm:runtime,browser_checks:browser,native_reference:pin(environment.BISECT_POPULATION_NATIVE_REFERENCE),source_preparation:pin(environment.BISECT_REPORT_SOURCE_EVIDENCE),export_binding:pin(path.join(binding.root,'evidence.json')),state_workflow:pin(path.join(workflow.root,'evidence.json')),stage:stage?pin(path.join(stage,'evidence.json')):null,portable_ui:portable?pin(path.join(portable.root,'evidence.json')):null,folder_api_ui:folder?pin(path.join(folder.root,'evidence.json')):null,actual_os_picker_permission:false,import_oracle:importOracle,stages,source_files:['scripts/web/check_native_report_regression.mjs','scripts/web/prepare_population_source_native_reference.py','scripts/web/fixtures/native-report-reference.mjs','scripts/web/fixtures/observed-project-open.mjs','scripts/web/test_observed_project_open.mjs','scripts/web/test_native_report_source_snapshot.mjs','scripts/web/test_native_report_export_binding.mjs','scripts/web/test_native_report_state_workflow.mjs','scripts/web/test_native_report_source_ui_browser.mjs','scripts/web/test_native_report_folder_browser.mjs','scripts/web/stage_state_identity_preview.py'].map(pin),scope:'Fresh actual CLI outcomes and supplied-source preparation/export/original State jobs, with optional exact staged portable/folder API browser checks. OPFS/injected errors do not establish OS permission grants; C-enabled uses metis-core fixtures. No full parent/release/deployment claim.'};
 fs.writeFileSync(path.join(root,'evidence.json'),JSON.stringify(evidence,null,2)+'\n');write('passed');console.log(JSON.stringify({root,status:'passed',evidence:pin(path.join(root,'evidence.json')),browser_checks:browser}));
}catch(error){write('failed',error.stack);throw error;}
