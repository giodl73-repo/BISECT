// Actual C dispatch/report-source integration on an existing exact local package.
// Native build features are supplied by the explicit C builder; no tool downloads.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
const args=process.argv.slice(2),option=name=>args.find(a=>a.startsWith('--'+name+'='))?.slice(name.length+3),browser=args.includes('--browser');
assert(args.every(a=>a==='--browser'||['toolkit','native','native-reference'].some(n=>a.startsWith('--'+n+'='))));
const toolkit=path.resolve(option('toolkit')??''),native=path.resolve(option('native')??''),nativeReference=path.resolve(option('native-reference')??'');
for(const name of ['toolkit','native','native-reference'])assert(option(name),'Supply --'+name);
for(const file of [toolkit,native,nativeReference])assert(file.startsWith(process.cwd()+path.sep),'C workflow inputs must be owned workspace paths');
const base=path.resolve('target/cffi-report-workflow');fs.mkdirSync(base,{recursive:true});const root=fs.mkdtempSync(path.join(base,'run-')),packageRoot=path.dirname(toolkit);
const sha=bytes=>createHash('sha256').update(bytes).digest('hex'),pin=file=>{const absolute=path.resolve(file),b=fs.readFileSync(absolute);return {path:path.relative(process.cwd(),absolute).replaceAll('\\','/'),sha256:sha(b),byte_length:b.length};};
function files(directory){return fs.readdirSync(directory,{withFileTypes:true}).flatMap(e=>{const p=path.join(directory,e.name);assert(!e.isSymbolicLink(),'No package links');return e.isDirectory()?files(p):[p];});}
const wasm=pin(path.join(toolkit,'bisect_wasm.wasm')),packageFiles=files(packageRoot).sort().map(pin),environment={...process.env,BISECT_TEST_WASM:path.join(toolkit,'bisect_wasm.wasm'),BISECT_REPORT_PROFILE:'c-sidecar',BISECT_REPORT_C_TOOLKIT:toolkit,BISECT_CFFI_EVIDENCE_ROOT:path.join(root,'kernel'),BISECT_CFFI_NATIVE_REFERENCE:nativeReference,BISECT_CFFI_BROWSER_EVIDENCE:path.join(root,'browser-kernel')},stages=[];
delete environment.BISECT_POPULATION_NATIVE_REFERENCE;delete environment.BISECT_REPORT_SOURCE_EVIDENCE;delete environment.BISECT_REPORT_PACKAGE_RECORD;
function progress(status,error){fs.writeFileSync(path.join(root,'progress.json'),JSON.stringify({status,wasm,stages,...(error?{error}:{} )},null,2)+'\n');}
function run(name,command,arguments_){
 const stage={name,command,arguments:arguments_,status:'running'};stages.push(stage);progress('running');
 const result=spawnSync(command,arguments_,{env:environment,encoding:'utf8',windowsHide:true,maxBuffer:32*1024*1024}),log=path.join(root,name+'.log');fs.writeFileSync(log,(result.stdout??'')+(result.stderr??''));stage.log=pin(log);stage.exit_code=result.status;stage.status=result.status===0?'passed':'failed';progress(stage.status==='failed'?'failed':'running');if(result.error)throw result.error;assert.equal(result.status,0,name+' failed: '+log);return (result.stdout??'').trim();
}
try{
 run('assets','node',['scripts/web/test_toolkit_assets.mjs',toolkit]);run('offline','node',['scripts/web/test_offline_package.mjs',packageRoot]);
 run('C-kernel','node',['scripts/web/test_cffi_engine.mjs','--toolkit',toolkit]);
 const kernel=pin(path.join(environment.BISECT_CFFI_EVIDENCE_ROOT,'engine-evidence.json')),kernelData=JSON.parse(fs.readFileSync(kernel.path));assert.equal(kernelData.loaded_artifacts.engine.sha256,wasm.sha256);
 const nativeRoot=run('C-native-report-reference','python',['scripts/web/prepare_population_source_native_reference.py','--native',native,'--engine','c-ffi']);environment.BISECT_POPULATION_NATIVE_REFERENCE=path.join(nativeRoot,'evidence.json');
 const preparation=JSON.parse(run('C-source-preparation','node',['scripts/web/test_native_report_source_snapshot.mjs']));environment.BISECT_REPORT_SOURCE_EVIDENCE=path.join(preparation.root,'evidence.json');
 const workflow=JSON.parse(run('C-State-report-workflow','node',['scripts/web/test_native_report_state_workflow.mjs']));const workflowRef=pin(path.join(workflow.root,'evidence.json'));assert(JSON.parse(fs.readFileSync(workflowRef.path)).actual_c_dispatch);
 const packageRecord=path.join(root,'package.json');fs.writeFileSync(packageRecord,JSON.stringify({status:'passed-stage-build',profile:'c-sidecar',wasm,package:packageRoot,package_files:packageFiles,deployed:false,acceptance_v1:false,scope:'Exact existing owned C package; actual kernel/report workflow checks are recorded separately.'},null,2)+'\n');environment.BISECT_REPORT_PACKAGE_RECORD=packageRecord;environment.BISECT_CFFI_ENGINE_RECORDS=path.join(environment.BISECT_CFFI_EVIDENCE_ROOT,'engine-records.json');
 let portable=null,folder=null,browserKernel=null,importOracle=null;
 if(browser){
  const observed=JSON.parse(run('import-oracle','node',['scripts/web/test_observed_project_open.mjs']));importOracle=pin(path.join(observed.root,'evidence.json'));
  run('C-kernel-browser','node',['scripts/web/test_cffi_engine_browser.mjs',toolkit]);browserKernel=pin(path.join(environment.BISECT_CFFI_BROWSER_EVIDENCE,'evidence.json'));
  const p=JSON.parse(run('C-portable-report-browser','node',['scripts/web/test_native_report_source_ui_browser.mjs',toolkit]));portable=pin(path.join(p.root,'evidence.json'));assert(JSON.parse(fs.readFileSync(portable.path)).actual_c_dispatch);
  const f=JSON.parse(run('C-folder-report-browser','node',['scripts/web/test_native_report_folder_browser.mjs',toolkit]));folder=pin(path.join(f.root,'evidence.json'));assert(JSON.parse(fs.readFileSync(folder.path)).actual_c_dispatch);
 }
 for(const ref of packageFiles)assert.deepEqual(pin(ref.path),ref,'Package changed during C checks');
 const evidence={status:'passed-C-report-workflow',wasm,native_cli:pin(native),native_oracle:pin(nativeReference),kernel,native_reference:pin(environment.BISECT_POPULATION_NATIVE_REFERENCE),source_preparation:pin(environment.BISECT_REPORT_SOURCE_EVIDENCE),workflow:workflowRef,package_record:pin(packageRecord),portable_browser:portable,folder_browser:folder,kernel_browser:browserKernel,browser_checks:browser,import_oracle:importOracle,actual_c_dispatch:true,runtime:'msvcrt-v1',stages,source_files:['scripts/web/check_cffi_report_workflow.mjs','scripts/web/prepare_population_source_native_reference.py','scripts/web/test_cffi_engine.mjs','scripts/web/test_cffi_engine_browser.mjs','scripts/web/test_native_report_source_snapshot.mjs','scripts/web/test_native_report_state_workflow.mjs','scripts/web/test_native_report_source_ui_browser.mjs','scripts/web/test_native_report_folder_browser.mjs','scripts/web/fixtures/native-report-reference.mjs','scripts/web/fixtures/observed-project-open.mjs','scripts/web/test_observed_project_open.mjs'].map(pin),actual_os_picker_permission:false,full_parity:false,deployed:false,scope:'Actual Windows-CRT C METIS dispatch: independent native C matrix and full original State report-source CLI jobs, with optional exact package workers/portable/folder browser journeys and recovery. Four-unit RI 2020 report cases, 64/300-node kernel grids; not national C, other CRTs/hosts, all capacity/methods, OS permissions or full parent/release acceptance.'};fs.writeFileSync(path.join(root,'evidence.json'),JSON.stringify(evidence,null,2)+'\n');progress('passed');console.log(JSON.stringify({root,status:evidence.status,evidence:pin(path.join(root,'evidence.json'))}));
}catch(error){progress('failed',error.stack);throw error;}
