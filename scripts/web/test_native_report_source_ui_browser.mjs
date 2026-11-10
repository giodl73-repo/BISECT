import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {createProject} from '../../web/lab/project.js';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {nativeReportReference} from './fixtures/native-report-reference.mjs';
import {observedProjectOpen} from './fixtures/observed-project-open.mjs';
const {chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES,'package.json'))('playwright');
const directory=path.resolve(process.argv[2]),root=await fs.mkdtemp(path.resolve('target/state-report-ui/browser-')),sha=b=>createHash('sha256').update(b).digest('hex');
const stagePath=path.resolve(process.env.BISECT_REPORT_PACKAGE_RECORD??path.resolve(directory,'../../evidence.json')),stageBytes=await fs.readFile(stagePath),stage=JSON.parse(stageBytes),cRun=process.env.BISECT_REPORT_C_TOOLKIT!==undefined;
const engine=await instantiateEngine(await fs.readFile(stage.wasm.path));
async function verify(){for(const ref of stage.package_files)assert.equal(sha(await fs.readFile(ref.path)),ref.sha256,ref.path);}
await verify();
const server=createServer(async(req,res)=>{try{const name=new URL(req.url,'http://localhost').pathname,file=path.resolve(directory,'.'+decodeURIComponent(name==='/'?'/index.html':name));assert(file.startsWith(directory+path.sep));res.setHeader('Content-Type',file.endsWith('.js')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':'text/html');res.end(await fs.readFile(file));}catch{res.writeHead(404);res.end();}});
await new Promise(r=>server.listen(0,'127.0.0.1',r));let browser,page;const errors=[],cases=[];
try{
 browser=await chromium.launch({headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe'});
 const context=await browser.newContext({acceptDownloads:true}),origin='http://127.0.0.1:'+server.address().port;
 await context.route('**/*',r=>new URL(r.request().url()).origin===origin?r.continue():r.abort());page=await context.newPage();page.setDefaultTimeout(120000);page.on('pageerror',e=>errors.push(e.message));
 await page.addInitScript(()=>{window.reportRequests=[];window.reportPickerCalls=0;window.showDirectoryPicker=()=>{reportPickerCalls++;return new Promise(resolve=>{window.reportResolvePicker=resolve;});};const W=Worker;window.Worker=class extends W{postMessage(m,...a){if(m.type==='run')reportRequests.push(m.request.operation);return super.postMessage(m,...a);}};});
 await page.goto(origin);await page.waitForFunction(()=>!document.querySelector('#execute').disabled);
 const fixture=JSON.parse(await fs.readFile('scripts/web/fixtures/native-state-source.json','utf8')),{sourceRoot,native}=await nativeReportReference(stage.profile);if(cRun)assert.equal(native.requested_metis_engine,'c-ffi');
 const open=async file=>{const count=await page.evaluate(()=>reportRequests.length),pickers=await page.evaluate(()=>reportPickerCalls);await observedProjectOpen(page,file);assert.equal(await page.evaluate(()=>reportRequests.length),count);assert.equal(await page.evaluate(()=>reportPickerCalls),pickers);};
 const save=async name=>{const d=page.waitForEvent('download');await page.locator('#save-project').click();const file=path.join(root,name);await(await d).saveAs(file);return {file,project:JSON.parse(await fs.readFile(file,'utf8'))};};
 let completedProject;
 for(const row of native.outcomes){
  const snapshot=JSON.parse(await fs.readFile(path.join(sourceRoot,row.name+'.json'),'utf8'));
  // A portable file bundle cannot represent a directory. Its API case stays in the job suite.
  if(snapshot.request.observations.some(o=>o.kind==='directory'))continue;
  const expectedReceipt=engine.execute({operation:'inspect-native-state-report-sources',arguments:snapshot.request.arguments,observations:snapshot.request.observations,observation_origin:'explicit-file-bundle'});
  const files={nativeAdjacencyInputs:{...structuredClone(fixture.nativeAdjacencyInputs),adjacency:{...fixture.nativeAdjacencyInputs.adjacency,name:'fixture.adj.bin'}},nativeAdjacencySettings:fixture.nativeAdjacencySettings,nativeSelectedStateSourceSettings:{kind:'native'},nativeStateSourceSettings:{enabled:true,command_kind:'state'},stateGraphInputs:{graph:null,auxiliary:null,settings:{label:'will_be_replaced',chamber:'congressional',created_at:'2026-10-27T00:00:00Z',inspect_only:true}},tuiRunWorkflowSettings:{analysis:false,map:false,report:false,dpi:'150'},stateCommandInputs:snapshot.source_command};
  const initial=path.join(root,row.name+'-initial.bisect');await fs.writeFile(initial,JSON.stringify(createProject({name:row.name,files,operation:'inspect-native-state-command',constraints:[],result:null,lastOperation:null})));await open(initial);
  await page.locator('#native-report-review').click();await page.waitForFunction(()=>document.querySelector('#native-report-status').textContent.startsWith('Candidate paths reviewed'));
  assert((await page.locator('#native-report-paths').textContent()).includes('Report search order'));
  assert.equal(await page.locator('#native-report-accept-missing').isChecked(),false);
  if(!cases.length){
   await page.locator('#native-report-empty').click();await page.waitForFunction(()=>document.querySelector('#native-report-status').textContent.includes('Choose how this portable bundle'));
   const rejected=await save('absence-choice-required.bisect');assert.equal(rejected.project.files.stateReportSourceSnapshot,undefined);
   await page.locator('#native-report-accept-missing').check();await page.locator('#native-report-files').setInputFiles({name:'not-a-native-candidate.csv',mimeType:'text/csv',buffer:Buffer.from('x')});await page.waitForFunction(()=>document.querySelector('#native-report-status').textContent.includes('exact candidate filename'));
   const unexpected=await save('unexpected-file-refused.bisect');assert.equal(unexpected.project.files.stateReportSourceSnapshot,undefined);
  }
  const selected=snapshot.request.observations.filter(o=>o.kind==='file').map(o=>({name:o.relative_path.split('/').at(-1),mimeType:'text/csv',buffer:Buffer.from(o.source_base64,'base64')}));
  await page.locator('#native-report-accept-missing').check();if(selected.length)await page.locator('#native-report-files').setInputFiles(selected);else await page.locator('#native-report-empty').click();
  await page.waitForFunction(()=>document.querySelector('#native-report-status').textContent.startsWith('Report-source snapshot loaded'));
  const loaded=await save(row.name+'-loaded.bisect');assert.equal(loaded.project.files.stateReportSourceSnapshot.live_folder_authority,false);assert.equal(loaded.project.files.stateReportSourceSnapshot.source_command.arguments_json,snapshot.source_command.arguments_json);assert.equal(loaded.project.files.stateReportSourceSnapshot.receipt.source_bundle_sha256,expectedReceipt.source_bundle_sha256);
  assert(!(await page.evaluate(()=>reportRequests)).includes('run-prepared-engine'));
  await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#apply-state-workflow').disabled);await page.locator('#apply-state-workflow').click();await page.waitForFunction(()=>document.querySelector('#operation').value==='prepare-native-engine-input');
  const applied=await save(row.name+'-applied.bisect');assert.equal(applied.project.files.stateReportSourceSnapshot,null);assert.equal(applied.project.files.stateWorkflowBinding.report_sources.receipt.source_bundle_sha256,expectedReceipt.source_bundle_sha256);
  const before=await page.evaluate(()=>reportRequests.length);await page.locator('#use-single-native-state-source').click();if(cRun){assert.equal(await page.locator('#c-metis-runtime').isChecked(),false);await page.locator('#c-metis-runtime').check();}await page.locator('#flow-run').click();await page.waitForFunction(()=>!document.querySelector('#download-native-states-run').disabled);
  const completed=await save(row.name+'-completed.bisect'),run=completed.project.result;assert.equal(run.native_report_sources.requested_population_source,row.requested);assert.equal(run.native_report_sources.cvap_missing_warning,row.missing_cvap_warning);
  if(cRun){assert.equal(run.prepared.result.backend,'c-metis-msvcrt-v1');assert.equal(run.prepared.runtime_artifacts.metis_sidecar.runtime,'msvcrt-v1');}
  if(row.expected_native_failure){assert.equal(run.status,'partial-output');assert.equal(run.prepared.export,null);assert(run.prepared.result.assignments);assert(await page.locator('#native-states-output').isDisabled());assert(await page.locator('#native-states-audit').isDisabled());assert((await page.locator('#native-report-outcome').textContent()).includes('Plan generation is retained'));const download=page.waitForEvent('download');await page.locator('#download-native-states-run').click();await(await download).saveAs(path.join(root,row.name+'-retained.zip'));}
  else{
   const expected=JSON.parse(await fs.readFile(row.files.find(f=>f.path.endsWith('/context.rctx')).path,'utf8'));assert.deepEqual(JSON.parse(run.prepared.export.context_json).demographics??null,expected.demographics??null);assert.deepEqual(JSON.parse(run.prepared.export.context_json).populations,expected.populations);
   const nativePlan=JSON.parse(await fs.readFile(row.files.find(f=>f.path.endsWith('/plan.rplan')).path,'utf8')),actualPlan=JSON.parse(run.prepared.export.document_json);assert.deepEqual(actualPlan.plan.assignment,nativePlan.plan.assignment);assert.deepEqual(actualPlan.plan.units.unit_ids,nativePlan.plan.units.unit_ids);
   if(row.name==='cvap-valid-file')completedProject=completed;
  }
  assert((await page.evaluate(()=>reportRequests.slice(-5))).length);assert((await page.evaluate(n=>reportRequests.slice(n),before)).includes('run-native-selected-state-source'));
  cases.push({name:row.name,explicit_file_or_empty_bundle:true,original_flags_applied:true,actual_worker_run:true,native_counts_or_failure_match:true,inert_initial_open:true});
  // Retire request history between fixtures so the pre-run assertion is scoped.
  await page.evaluate(()=>{reportRequests=[];});
 }
 assert(completedProject);await open(completedProject.file);assert(await page.locator('#native-states-output').isDisabled());
 const beforeReplay=await page.evaluate(()=>reportRequests.length);if(cRun){assert.equal(await page.locator('#c-metis-runtime').isChecked(),false);await page.locator('#c-metis-runtime').check();}await page.locator('#flow-run').click();await page.waitForFunction(()=>!document.querySelector('#download-native-states-run').disabled);const replay=await save('explicit-replay.bisect');assert.deepEqual(replay.project.result.prepared.result.assignments,completedProject.project.result.prepared.result.assignments);assert((await page.evaluate(n=>reportRequests.slice(n),beforeReplay)).includes('run-native-selected-state-source'));
 await page.locator('#review-state-workflow-original').click();await page.locator('#native-report-folder').click();await page.waitForFunction(()=>reportPickerCalls===1);await page.locator('#native-report-cancel').click();
 await page.evaluate(()=>{window.reportHandleReads=0;reportResolvePicker({kind:'directory',getDirectoryHandle(){reportHandleReads++;throw Error('Retired picker must not be read');}});});await page.waitForTimeout(150);assert.equal(await page.evaluate(()=>reportHandleReads),0);
 await page.locator('#native-report-folder').click();await page.waitForFunction(()=>reportPickerCalls===2);await page.locator('#state-command-arguments').fill('["--state","RI","--year","2010"]');
 await page.evaluate(()=>reportResolvePicker({kind:'directory',getDirectoryHandle(){reportHandleReads++;throw Error('Stale picker must not be read');}}));await page.waitForTimeout(150);assert.equal(await page.evaluate(()=>reportHandleReads),0);
 await verify();assert.deepEqual(errors,[]);await page.screenshot({path:path.join(root,'controls.png'),fullPage:true});
 await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({status:'passed-report-source-ui',profile:stage.profile,package_validation:{path:stagePath,sha256:sha(stageBytes)},wasm:stage.wasm,cases,inert_open:true,explicit_replay:true,cancel_and_changed_flags_retire_pending_picker:true,actual_native_picker_permission:false,actual_c_dispatch:cRun,scope:'Actual packaged Chrome UI/file inputs/guard workers/original Apply/State jobs against retained native cases, saved files and explicit replay. Pending picker cancellation uses an injected delayed handle, not a real OS permission grant. Directory kind and read-only filesystem behavior remain separately tested in API suites. No full matrix/parent acceptance.'},null,2));console.log(JSON.stringify({root,status:'passed-report-source-ui',cases:cases.length}));
}catch(error){await fs.writeFile(path.join(root,'failure.json'),JSON.stringify({error:error.stack,import_observation:error.import_observation??null,project_status:await page?.locator('#project-status').textContent().catch(()=>null),errors,cases},null,2));if(page)await page.screenshot({path:path.join(root,'failure.png'),fullPage:true}).catch(()=>{});throw error;}
finally{await browser?.close();await new Promise(r=>server.close(r));}
