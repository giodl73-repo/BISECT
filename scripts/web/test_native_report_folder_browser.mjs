import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {createProject} from '../../web/lab/project.js';
import {nativeReportReference} from './fixtures/native-report-reference.mjs';
import {observedProjectOpen} from './fixtures/observed-project-open.mjs';
const {chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES,'package.json'))('playwright');
const directory=path.resolve(process.argv[2]),root=await fs.mkdtemp(path.resolve('target/state-report-folder/browser-')),sha=b=>createHash('sha256').update(b).digest('hex');
const stagePath=path.resolve(process.env.BISECT_REPORT_PACKAGE_RECORD??path.resolve(directory,'../../evidence.json')),stageBytes=await fs.readFile(stagePath),stage=JSON.parse(stageBytes),cases=[],faults=[],errors=[],cRun=process.env.BISECT_REPORT_C_TOOLKIT!==undefined;
async function verify(){for(const ref of stage.package_files)assert.equal(sha(await fs.readFile(ref.path)),ref.sha256,ref.path);}
await verify();
const server=createServer(async(req,res)=>{try{const name=new URL(req.url,'http://localhost').pathname,file=path.resolve(directory,'.'+decodeURIComponent(name==='/'?'/index.html':name));assert(file.startsWith(directory+path.sep));res.setHeader('Content-Type',file.endsWith('.js')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':'text/html');res.end(await fs.readFile(file));}catch{res.writeHead(404);res.end();}});
await new Promise(r=>server.listen(0,'127.0.0.1',r));let browser,page;
try{
 browser=await chromium.launch({headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe'});
 const context=await browser.newContext({acceptDownloads:true}),origin='http://127.0.0.1:'+server.address().port;
 await context.route('**/*',r=>new URL(r.request().url()).origin===origin?r.continue():r.abort());page=await context.newPage();page.setDefaultTimeout(120000);page.on('pageerror',e=>errors.push(e.message));
 await page.addInitScript(()=>{
  window.folderRequests=[];window.folderPickerCalls=0;window.folderRoot=null;window.folderMode='native';window.folderLog=[];window.folderInjected=false;window.folderReadPaused=false;
  window.showDirectoryPicker=async options=>{folderPickerCalls++;if(options.mode!=='read')throw Error('Folder test requires read-only picker intent');if(folderMode==='picker-denied')throw new DOMException('Injected picker denial','NotAllowedError');return folderMode==='native'?folderRoot:wrapFolder(folderRoot,0);};
  window.wrapFolder=(directory,depth)=>({kind:'directory',name:directory.name,async getDirectoryHandle(name,options){
   if(options?.create!==false)throw Error('Source read attempted to create a directory');folderLog.push({kind:'directory',depth,name});
   if(folderMode==='permission'&&depth===2)throw new DOMException('Injected active-read permission failure','NotAllowedError');
   return wrapFolder(await directory.getDirectoryHandle(name,options),depth+1);
  },async getFileHandle(name,options){
   if(options?.create!==false)throw Error('Source read attempted to create a file');folderLog.push({kind:'file',depth,name});const handle=await directory.getFileHandle(name,options);
   return {kind:'file',name,async getFile(){const file=await handle.getFile();folderLog.push({kind:'getFile',name,size:file.size});return {size:file.size,async arrayBuffer(){
    folderLog.push({kind:'bytes',name});
    if(folderMode==='unreadable')throw new DOMException('Injected active-read failure','NotReadableError');
    if(folderMode==='paused'&&!folderInjected){folderInjected=true;folderReadPaused=true;await new Promise(resolve=>window.releaseFolderRead=resolve);}
    const bytes=await file.arrayBuffer();
    if(!folderInjected&&['delete','replace-directory','change-bytes'].includes(folderMode)){
     folderInjected=true;
     if(folderMode==='change-bytes'){const changed=new Uint8Array(bytes.slice(0));changed[0]^=1;const writer=await handle.createWritable();await writer.write(changed);await writer.close();}
     else{await directory.removeEntry(name);if(folderMode==='replace-directory')await directory.getDirectoryHandle(name,{create:true});}
    }
    return bytes;
   }};}};
  }});
  const W=Worker;window.Worker=class extends W{postMessage(m,...a){if(m.type==='run')folderRequests.push(m.request.operation);return super.postMessage(m,...a);}};
 });
 await page.goto(origin);await page.waitForFunction(()=>!document.querySelector('#execute').disabled);
 assert(await page.evaluate(()=>typeof navigator.storage.getDirectory==='function'&&typeof FileSystemDirectoryHandle==='function'));
 const fixture=JSON.parse(await fs.readFile('scripts/web/fixtures/native-state-source.json','utf8')),{sourceRoot,reference:nativeReference,native}=await nativeReportReference(stage.profile);
 if(cRun)assert.equal(native.requested_metis_engine,'c-ffi');
 const filesFor=snapshot=>({nativeAdjacencyInputs:{...structuredClone(fixture.nativeAdjacencyInputs),adjacency:{...fixture.nativeAdjacencyInputs.adjacency,name:'fixture.adj.bin'}},nativeAdjacencySettings:fixture.nativeAdjacencySettings,nativeSelectedStateSourceSettings:{kind:'native'},nativeStateSourceSettings:{enabled:true,command_kind:'state'},stateGraphInputs:{graph:null,auxiliary:null,settings:{label:'will_be_replaced',chamber:'congressional',created_at:'2026-10-27T00:00:00Z',inspect_only:true}},tuiRunWorkflowSettings:{analysis:false,map:false,report:false,dpi:'150'},stateCommandInputs:structuredClone(snapshot.source_command)});
 const project=async(name,files)=>{const file=path.join(root,name+'.bisect');await fs.writeFile(file,JSON.stringify(createProject({name,files,operation:'inspect-native-state-command',constraints:[],result:null,lastOperation:null})));return file;};
 const open=async file=>{
  const before=await page.evaluate(()=>({requests:folderRequests.length,pickers:folderPickerCalls}));
  await observedProjectOpen(page,file);assert.deepEqual(await page.evaluate(()=>({requests:folderRequests.length,pickers:folderPickerCalls})),before);
 };
 const save=async name=>{const d=page.waitForEvent('download');await page.locator('#save-project').click();const file=path.join(root,name+'.bisect');await(await d).saveAs(file);return {file,project:JSON.parse(await fs.readFile(file,'utf8'))};};
 async function seed(snapshot,mode='native'){
  return page.evaluate(async({observations,mode})=>{
   const storage=await navigator.storage.getDirectory(),name='report-source-fixture-'+crypto.randomUUID();window.folderRoot=await storage.getDirectoryHandle(name,{create:true});window.folderMode=mode;window.folderLog=[];window.folderInjected=false;window.folderReadPaused=false;
   for(const entry of observations){if(entry.kind==='missing')continue;let directory=folderRoot;const parts=entry.relative_path.split('/');for(const component of parts.slice(0,-1))directory=await directory.getDirectoryHandle(component,{create:true});const basename=parts.at(-1);
    if(entry.kind==='directory')await directory.getDirectoryHandle(basename,{create:true});else{const file=await directory.getFileHandle(basename,{create:true}),writer=await file.createWritable();await writer.write(mode==='oversized'?new Uint8Array(9*1024*1024):Uint8Array.from(atob(entry.source_base64),c=>c.charCodeAt(0)));await writer.close();}
   }
   return {native_handle:folderRoot instanceof FileSystemDirectoryHandle,name:folderRoot.name};
  },{observations:snapshot.request.observations,mode});
 }
 const contents=()=>page.evaluate(async()=>{const rows=[];async function walk(directory,prefix=''){for await(const [name,entry] of directory.entries()){const path=prefix+name;if(entry.kind==='directory'){rows.push({path,kind:'directory'});await walk(entry,path+'/');}else{const bytes=await(await entry.getFile()).arrayBuffer(),hash=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');rows.push({path,kind:'file',size:bytes.byteLength,sha256:hash});}}}await walk(folderRoot);return rows.sort((a,b)=>a.path.localeCompare(b.path));});
 async function runAndSave(name){await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#apply-state-workflow').disabled);await page.locator('#apply-state-workflow').click();await page.waitForFunction(()=>document.querySelector('#operation').value==='prepare-native-engine-input');await page.locator('#use-single-native-state-source').click();if(cRun){assert.equal(await page.locator('#c-metis-runtime').isChecked(),false);await page.locator('#c-metis-runtime').check();}await page.locator('#flow-run').click();await page.waitForFunction(()=>!document.querySelector('#download-native-states-run').disabled);return save(name);}
 let goodSnapshot,goodProject;
 for(const row of native.outcomes){
  const snapshot=JSON.parse(await fs.readFile(path.join(sourceRoot,row.name+'.json'),'utf8'));await open(await project(row.name+'-initial',filesFor(snapshot)));const handle=await seed(snapshot);assert(handle.native_handle);
  const beforeContents=await contents(),beforeRun=await page.evaluate(()=>folderRequests.filter(n=>n==='run-native-selected-state-source').length);
  await page.locator('#native-report-folder').click();await page.waitForFunction(()=>document.querySelector('#native-report-status').textContent.startsWith('Report-source snapshot loaded'));assert.deepEqual(await contents(),beforeContents);
  const loaded=await save(row.name+'-loaded'),selected=loaded.project.files.stateReportSourceSnapshot;assert.equal(selected.receipt.source_bundle_sha256,snapshot.receipt.source_bundle_sha256);assert.equal(selected.live_folder_authority,false);assert.deepEqual(selected.request.observations,snapshot.request.observations);assert.equal(await page.evaluate(()=>folderRequests.filter(n=>n==='run-native-selected-state-source').length),beforeRun);
  const completed=await runAndSave(row.name+'-completed'),run=completed.project.result;assert.equal(run.native_report_sources.cvap_missing_warning,row.missing_cvap_warning);assert.equal(run.native_report_sources.requested_population_source,row.requested);
  if(cRun){assert.equal(run.prepared.result.backend,'c-metis-msvcrt-v1');assert.equal(run.prepared.runtime_artifacts.metis_sidecar.runtime,'msvcrt-v1');}
  if(row.expected_native_failure){assert.equal(run.status,'partial-output');assert.equal(run.prepared.export,null);assert(await page.locator('#native-states-output').isDisabled());}
  else{const expected=JSON.parse(await fs.readFile(row.files.find(f=>f.path.endsWith('/context.rctx')).path,'utf8')),context=JSON.parse(run.prepared.export.context_json);assert.deepEqual(context.demographics??null,expected.demographics??null);assert.deepEqual(context.populations,expected.populations);const nativePlan=JSON.parse(await fs.readFile(row.files.find(f=>f.path.endsWith('/plan.rplan')).path,'utf8')),actualPlan=JSON.parse(run.prepared.export.document_json);assert.deepEqual(actualPlan.plan.assignment,nativePlan.plan.assignment);assert.deepEqual(actualPlan.plan.units.unit_ids,nativePlan.plan.units.unit_ids);}
  if(row.name==='cvap-valid-file'){goodSnapshot=snapshot;goodProject=loaded;}
  cases.push({name:row.name,native_opfs_directory_handle:true,unmodified_readonly_tree:true,native_observations_and_hash_match:true,actual_state_worker_run:true,native_counts_or_failure_match:true});
 }
 assert(goodSnapshot&&goodProject);
 for(const mode of ['permission','unreadable','delete','replace-directory','change-bytes','oversized','picker-denied']){
  await open(goodProject.file);await seed(goodSnapshot,mode);const before=await page.evaluate(()=>folderRequests.filter(n=>n==='run-native-selected-state-source').length);
  await page.locator('#native-report-folder').click();await page.waitForFunction(()=>document.querySelector('#native-report-status').textContent.startsWith('No report-source snapshot applied.'));
  const message=await page.locator('#native-report-status').textContent(),failed=await save('fault-'+mode);assert.deepEqual(failed.project.files.stateReportSourceSnapshot,goodProject.project.files.stateReportSourceSnapshot);assert.equal(await page.evaluate(()=>folderRequests.filter(n=>n==='run-native-selected-state-source').length),before);
  if(mode==='oversized')assert.equal(await page.evaluate(()=>folderLog.filter(e=>e.kind==='bytes').length),0);
  if(['delete','replace-directory','change-bytes'].includes(mode))assert.match(message,/changed while loading/);
  faults.push({mode,message,prior_snapshot_retained:true,no_partition:true,log:await page.evaluate(()=>folderLog)});
 }
 // Retry is explicit and uses a fresh native directory handle/tree.
 await seed(goodSnapshot);await page.locator('#native-report-folder').click();await page.waitForFunction(()=>document.querySelector('#native-report-status').textContent.startsWith('Report-source snapshot loaded'));const recovered=await runAndSave('recovered-run');assert.equal(recovered.project.result.native_report_sources.status,'bound');
 // Pause a genuine File read. Cancellation and project Open must retire it.
 for(const action of ['cancel','open-project']){
  await open(goodProject.file);await seed(goodSnapshot,'paused');await page.locator('#native-report-folder').click();await page.waitForFunction(()=>folderReadPaused===true);
  if(action==='cancel')await page.locator('#native-report-cancel').click();else{const different=filesFor(goodSnapshot);different.stateCommandInputs={arguments_json:'["--state","RI","--year","2010"]'};await open(await project('changed-project',different));}
  const logBefore=await page.evaluate(()=>folderLog.length);await page.evaluate(()=>releaseFolderRead());await page.waitForFunction(()=>document.querySelector('#native-report-cancel').disabled);await page.waitForTimeout(150);assert.equal(await page.evaluate(()=>folderLog.length),logBefore);
  const after=await save('paused-'+action);if(action==='cancel')assert.deepEqual(after.project.files.stateReportSourceSnapshot,goodProject.project.files.stateReportSourceSnapshot);else assert.equal(after.project.files.stateReportSourceSnapshot,undefined);
  faults.push({mode:'paused-'+action,no_further_handle_calls:true,retired_selection_not_published:true});
 }
 await verify();assert.deepEqual(errors,[]);
 await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({status:'passed-native-folder-api-ui',profile:stage.profile,package_validation:{path:stagePath,sha256:sha(stageBytes)},native_reference:nativeReference,wasm:stage.wasm,cases,faults,actual_opfs_handles:true,actual_os_picker_permission:false,actual_c_dispatch:cRun,explicit_recovery:true,scope:'Actual packaged Chrome File System Access directory/file API backed by origin-private storage, guard workers and State jobs. Picker returns an explicitly seeded OPFS root, not a native OS-selected root. Positive handles and missing/type mismatch errors are native browser API behavior; permission/unreadable/pause faults use a wrapper and are not real OS denial. Real disk mutations occur only in owned OPFS fixtures. Both expected native directory/empty failures retain generation. Actual OS permission, full host/backend/method/capacity/release acceptance remain open.'},null,2));console.log(JSON.stringify({root,status:'passed-native-folder-api-ui',cases:cases.length,faults:faults.length}));
}catch(error){await fs.writeFile(path.join(root,'failure.json'),JSON.stringify({error:error.stack,project_status:await page?.locator('#project-status').textContent().catch(()=>null),errors,cases,faults},null,2));if(page)await page.screenshot({path:path.join(root,'failure.png'),fullPage:true}).catch(()=>{});throw error;}
finally{await browser?.close();await new Promise(r=>server.close(r));}
