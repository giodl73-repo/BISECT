import {openWorkbenchProject} from './workbench_project_open.mjs';
// Manual OS picker grant, with automated source/job/artifact checks afterward.
// No permission dialog is accepted or security preference changed by this script.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {createProject} from '../../web/lab/project.js';
const {chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES,'package.json'))('playwright');
const directory=path.resolve(process.argv[2]),root=await fs.mkdtemp(path.resolve('target/state-report-folder/permission-')),working=path.join(root,'selected-working-root'),stagePath=path.resolve(directory,'../../evidence.json'),sha=b=>createHash('sha256').update(b).digest('hex');
const stageBytes=await fs.readFile(stagePath),stage=JSON.parse(stageBytes),snapshot=JSON.parse(await fs.readFile('target/state-report-sources/node-B9yqVS/cvap-valid-file.json','utf8'));
async function verify(){for(const ref of stage.package_files)assert.equal(sha(await fs.readFile(ref.path)),ref.sha256,ref.path);}
await verify();await fs.mkdir(working);
for(const entry of snapshot.request.observations){if(entry.kind==='missing')continue;const file=path.resolve(working,entry.relative_path);assert(file.startsWith(working+path.sep));if(entry.kind==='directory')await fs.mkdir(file,{recursive:true});else{await fs.mkdir(path.dirname(file),{recursive:true});await fs.writeFile(file,Buffer.from(entry.source_base64,'base64'));}}
const fixture=JSON.parse(await fs.readFile('scripts/web/fixtures/native-state-source.json','utf8')),files={nativeAdjacencyInputs:{...structuredClone(fixture.nativeAdjacencyInputs),adjacency:{...fixture.nativeAdjacencyInputs.adjacency,name:'fixture.adj.bin'}},nativeAdjacencySettings:fixture.nativeAdjacencySettings,nativeSelectedStateSourceSettings:{kind:'native'},nativeStateSourceSettings:{enabled:true,command_kind:'state'},stateGraphInputs:{graph:null,auxiliary:null,settings:{label:'will_be_replaced',chamber:'congressional',created_at:'2026-10-27T00:00:00Z',inspect_only:true}},stateCommandInputs:snapshot.source_command,tuiRunWorkflowSettings:{analysis:false,map:false,report:false,dpi:'150'}};
const initial=path.join(root,'initial.bisect');await fs.writeFile(initial,JSON.stringify(createProject({name:'Native folder permission check',files,operation:'inspect-native-state-command',constraints:[],result:null,lastOperation:null})));
const server=createServer(async(req,res)=>{try{const name=new URL(req.url,'http://localhost').pathname,file=path.resolve(directory,'.'+decodeURIComponent(name==='/'?'/index.html':name));assert(file.startsWith(directory+path.sep));res.setHeader('Content-Type',file.endsWith('.js')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':'text/html');res.end(await fs.readFile(file));}catch{res.writeHead(404);res.end();}});
await new Promise(r=>server.listen(0,'127.0.0.1',r));let browser,page;const errors=[];
try{
 browser=await chromium.launch({headless:false,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-background-networking','--window-size=1280,1000']});
 const context=await browser.newContext({acceptDownloads:true}),origin='http://127.0.0.1:'+server.address().port;await context.route('**/*',r=>new URL(r.request().url()).origin===origin?r.continue():r.abort());page=await context.newPage();page.setDefaultTimeout(120000);page.on('pageerror',e=>errors.push(e.message));
 await page.addInitScript(({expectedName})=>{
  window.permissionObservations=[];window.permissionRequests=[];const picker=window.showDirectoryPicker?.bind(window);
  window.showDirectoryPicker=async options=>{
   try{
    if(!picker)throw Error('Native directory picker is unavailable');
    const root=await picker(options);const actualNative=root instanceof FileSystemDirectoryHandle,read=await root.queryPermission({mode:'read'}),write=await root.queryPermission({mode:'readwrite'});
    permissionObservations.push({kind:'selected',name:root.name,actualNative,read,write,requestedMode:options.mode});
    if(!actualNative||root.name!==expectedName)throw Error('Select only the prepared selected-working-root fixture. No source read was started.');return root;
   }catch(error){permissionObservations.push({kind:'error',name:error.name,message:error.message});throw error;}
  };
  const W=Worker;window.Worker=class extends W{postMessage(m,...a){if(m.type==='run')permissionRequests.push(m.request.operation);return super.postMessage(m,...a);}};
 },{expectedName:path.basename(working)});
 await page.goto(origin);await page.waitForFunction(()=>!document.querySelector('#execute').disabled);await openWorkbenchProject(page,initial,{ready:({argument:__bisectOpenArgument4881,openedStatus:__bisectOpenedStatus4881})=>(()=>__bisectOpenedStatus4881.startsWith('Opened'))(__bisectOpenArgument4881),readyArg:undefined});
 await page.locator('#native-report-folder').scrollIntoViewIfNeeded();await page.evaluate(()=>document.title='BISECT — native folder permission check');
 const pending={status:'waiting-for-user-native-picker',working_root:working,origin,stage_validation:{path:stagePath,sha256:sha(stageBytes)},instructions:'In the dedicated Chrome window, click Read selected working folder. You may first cancel to exercise cancellation. Then select the prepared selected-working-root folder and approve read access yourself. The script will never accept the native picker or any permission prompt.'};await fs.writeFile(path.join(root,'pending.json'),JSON.stringify(pending,null,2));console.log(JSON.stringify({root,pending:path.join(root,'pending.json'),working_root:working,origin}),{flush:true});
 // User owns every native picker/permission action. A live browser/page is required.
 await page.waitForFunction(()=>document.querySelector('#native-report-status').textContent.startsWith('Report-source snapshot loaded'),null,{timeout:0});
 const observations=await page.evaluate(()=>permissionObservations);assert(observations.some(r=>r.kind==='selected'&&r.actualNative&&r.read==='granted'&&r.requestedMode==='read'));
 const save=async name=>{const d=page.waitForEvent('download');await page.locator('#save-project').click();const file=path.join(root,name+'.bisect');await(await d).saveAs(file);return {file,project:JSON.parse(await fs.readFile(file,'utf8'))};};
 const loaded=await save('loaded');assert.equal(loaded.project.files.stateReportSourceSnapshot.live_folder_authority,false);assert.equal(loaded.project.files.stateReportSourceSnapshot.receipt.source_bundle_sha256,snapshot.receipt.source_bundle_sha256);
 await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#apply-state-workflow').disabled);await page.locator('#apply-state-workflow').click();await page.waitForFunction(()=>document.querySelector('#operation').value==='prepare-native-engine-input');await page.locator('#use-single-native-state-source').click();await page.locator('#flow-run').click();await page.waitForFunction(()=>!document.querySelector('#download-native-states-run').disabled);
 const completed=await save('completed');assert.equal(completed.project.result.native_report_sources.status,'bound');
 const before=await page.evaluate(()=>({requests:permissionRequests.length,observations:permissionObservations.length}));await openWorkbenchProject(page,completed.file,{ready:({argument:__bisectOpenArgument7487,openedStatus:__bisectOpenedStatus7487})=>(()=>__bisectOpenedStatus7487.startsWith('Opened'))(__bisectOpenArgument7487),readyArg:undefined});assert.deepEqual(await page.evaluate(()=>({requests:permissionRequests.length,observations:permissionObservations.length})),before);assert(await page.locator('#native-states-output').isDisabled());
 await verify();assert.deepEqual(errors,[]);await page.locator('#native-report-sources').screenshot({path:path.join(root,'selected-folder.png')});
 const result={status:'passed-manual-native-folder-permission',profile:stage.profile,package_validation:{path:stagePath,sha256:sha(stageBytes)},wasm:stage.wasm,native_picker_observations:observations,loaded_snapshot_hash:snapshot.receipt.source_bundle_sha256,manual_os_picker_read_grant:true,inert_project_reopen:true,source_tree_paths:snapshot.request.observations.map(o=>o.relative_path),scope:'User manually selected the owned public fixture folder and approved native Chrome read access. The test observed actual FileSystemDirectoryHandle/queryPermission and source/job output. No permission prompt was automated. One valid selected-root case; other permission/host/backend/capacity/release scopes remain separate.'};await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(result,null,2));console.log(JSON.stringify({root,status:result.status}));
}catch(error){await fs.writeFile(path.join(root,'failure.json'),JSON.stringify({error:error.stack,errors},null,2));throw error;}
finally{await browser?.close();await new Promise(r=>server.close(r));}
