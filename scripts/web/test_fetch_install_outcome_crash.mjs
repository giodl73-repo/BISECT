// Abrupt owned Chrome process-tree termination; persistent IndexedDB and OPFS.
// This local acceptance tier requires retained RI assets, Windows and Chrome.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {spawn,execFileSync} from 'node:child_process';
import {once} from 'node:events';
const {chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES,'package.json'))('playwright');
const index=process.argv.indexOf('--package'),directory=path.resolve(index<0?'web/lab':process.argv[index+1]),output=path.resolve(process.env.BISECT_FETCH_HISTORY_OUTPUT||'target/fetch-outcome-history/crash-'+Date.now());await fs.mkdir(output,{recursive:true});
const relative='V3/data/2020/adjacency',scope={repository:'giodl73-repo/REDIST',tag:'data-inputs-v1',state:'RI',year:'2020'},names=['ri_adjacency_2020.pkl','ri_adjacency_2020.adj.bin','ri_adjacency_2020_geoids.json'],sha=b=>createHash('sha256').update(b).digest('hex');
const assets=await Promise.all(names.map(async name=>({name,bytes:await fs.readFile('target/adjacency-release/private-browser-1793032583676/native-outputs/'+relative+'/'+name)})));
const acquired={opaque_files:assets.map(a=>({name:a.name,bytes:Array.from(a.bytes)})),release_receipt:{schema_version:'bisect-adjacency-release-acquisition-v1',purpose:'native-fetch-bundle',...scope,release_id:352236194,transport:'retained-private-provider-bytes',files:assets.map(a=>({name:a.name,byte_length:a.bytes.length,sha256:sha(a.bytes),published_digest:'sha256:'+sha(a.bytes),published_digest_verified:true}))}};
const html=`<!doctype html><meta charset="utf-8"><main id="host"></main><script type="module">
import {installNativeFetchFolderControls} from './native-fetch-install-ui.js';
window.pickerCalls=0;window.sourceCalls=0;window.paused=false;window.pauseFirst=false;window.effects=0;window.testErrors=[];
const originRoot=await navigator.storage.getDirectory(),root=await originRoot.getDirectoryHandle('owned-history-test',{create:true});
function wrapped(d){return {name:d.name,kind:d.kind,queryPermission:d.queryPermission.bind(d),entries:d.entries.bind(d),removeEntry:async n=>{effects++;return d.removeEntry(n);},getDirectoryHandle:async(n,o)=>wrapped(await d.getDirectoryHandle(n,o)),getFileHandle:async(n,o)=>{const f=await d.getFileHandle(n,o);return {name:f.name,kind:f.kind,getFile:f.getFile.bind(f),createWritable:async()=>{const w=await f.createWritable();return {write:w.write.bind(w),abort:w.abort.bind(w),close:async()=>{await w.close();effects++;if(pauseFirst){pauseFirst=false;paused=true;await new Promise(()=>{});}}};}};}};}
window.showDirectoryPicker=async()=>{pickerCalls++;return wrapped(root);};
window.controls=installNativeFetchFolderControls({document,host:document.querySelector('#host'),getContext:()=>({bundle:true,busy:false,revision:0,scope:${JSON.stringify(scope)}}),onBegin(){},onEnd(){},onWorking(v){window.working=v;},onError(e){testErrors.push(e);}});
window.authorizeSource=async()=>{sourceCalls++;const a=await(await fetch('/history-source.json')).json();for(const f of a.opaque_files)f.bytes=Uint8Array.from(f.bytes);controls.setBundle(a);};window.ready=true;
</script>`;
const server=createServer(async(req,res)=>{try{const u=new URL(req.url,'http://localhost');res.setHeader('Cache-Control','no-store');if(u.pathname==='/history-fixture.html'){res.setHeader('Content-Type','text/html');res.end(html);return;}if(u.pathname==='/history-source.json'){res.setHeader('Content-Type','application/json');res.end(JSON.stringify(acquired));return;}const p=path.resolve(directory,'.'+decodeURIComponent(u.pathname));assert(p.startsWith(directory+path.sep));res.setHeader('Content-Type',p.endsWith('.js')?'application/javascript':'application/octet-stream');res.end(await fs.readFile(p));}catch{res.writeHead(404);res.end();}});await new Promise(r=>server.listen(0,'127.0.0.1',r));const origin='http://127.0.0.1:'+server.address().port,url=origin+'/history-fixture.html';let browser;
const wait=page=>page.waitForFunction(()=>window.ready&&document.querySelector('#fetch-history-status').textContent);
const read=page=>page.evaluate(async()=>(await import('/fetch-install-outcome-store.js')).readFetchOutcome());
async function review(page){await page.locator('#fetch-folder-choose').click();await page.evaluate(()=>authorizeSource());await page.locator('#fetch-folder-review').click();await page.waitForFunction(()=>!document.querySelector('#fetch-folder-apply').disabled);}
assert.equal(process.platform,'win32','This gate specifically tests Windows Chrome process-tree termination.');
assert(index>=0,'Select a sealed package');
const option=name=>{const i=process.argv.indexOf(name);assert(i>=0,'Missing '+name);return process.argv[i+1];},validationPath=option('--validation'),profile=option('--profile'),validation=JSON.parse(await fs.readFile(validationPath,'utf8'));
assert.equal(validation.status,'passed');assert.equal(validation.source_unchanged,true);const packagePins=validation.packages[profile];assert(Array.isArray(packagePins)&&packagePins.some(p=>path.resolve(p.path)===path.join(directory,'bisect_wasm.wasm')));
const sourcePin={path:'scripts/web/test_fetch_install_outcome_crash.mjs',sha256:sha(await fs.readFile(import.meta.filename))};
async function verifyPackage(){for(const p of packagePins){const bytes=await fs.readFile(p.path);assert.equal(bytes.length,p.byte_length,p.path);assert.equal(sha(bytes),p.sha256,p.path);}}
await verifyPackage();
const parent=path.resolve('target/fetch-outcome-history'),profileRoot=path.join(parent,'profile-crash-'+Date.now()+'-'+process.pid);assert(profileRoot.startsWith(parent+path.sep));await fs.mkdir(profileRoot);const realProfile=await fs.realpath(profileRoot);assert.equal(realProfile,path.resolve(profileRoot));assert(!(await fs.lstat(profileRoot)).isSymbolicLink());
let child=null,exited=null,crashes=0,sourceRequests=0;const errors=[],records=[];
server.on('request',req=>{if(req.url==='/history-source.json')sourceRequests++;});
const delay=ms=>new Promise(resolve=>setTimeout(resolve,ms));
async function launch(){
 const portFile=path.join(profileRoot,'DevToolsActivePort');assert(portFile.startsWith(realProfile+path.sep));await fs.rm(portFile,{force:true});
 child=spawn('C:/Program Files/Google/Chrome/Application/chrome.exe',['--headless=new','--remote-debugging-address=127.0.0.1','--remote-debugging-port=0','--user-data-dir='+profileRoot,'--no-first-run','--no-default-browser-check','--disable-extensions','--disable-sync','--disable-background-networking','--disable-component-update','--disable-session-crashed-bubble','about:blank'],{windowsHide:true,stdio:'ignore'});exited=once(child,'exit');
 let port;const deadline=Date.now()+30000;
 while(Date.now()<deadline){assert.equal(child.exitCode,null,'Owned Chrome exited during startup');try{port=Number((await fs.readFile(portFile,'utf8')).split('\n')[0]);if(Number.isSafeInteger(port)&&port>0&&port<=65535)break;}catch{}await delay(100);}
 assert(Number.isSafeInteger(port)&&port>0,'Owned Chrome CDP startup deadline');browser=await chromium.connectOverCDP('http://127.0.0.1:'+port);const contexts=browser.contexts();assert.equal(contexts.length,1);const context=contexts[0];
 await context.route('**/*',route=>new URL(route.request().url()).origin===origin?route.continue():route.abort());context.on('page',p=>p.on('pageerror',e=>errors.push(e.message)));
 const page=await context.newPage();await page.goto(url);await wait(page);return page;
}
async function crash(checkpoint=true){
 assert(child&&child.exitCode===null&&child.signalCode===null&&Number.isSafeInteger(child.pid),'Only the owned live Chrome may be terminated');
 execFileSync('taskkill',['/PID',String(child.pid),'/T','/F'],{windowsHide:true,stdio:['ignore','pipe','pipe']});await exited;assert(child.exitCode!==null||child.signalCode!==null,'Owned browser exit must be authoritative');if(checkpoint)crashes++;child=null;browser=null;
}
async function inert(page,expected){
 await page.waitForFunction(()=>document.querySelector('#fetch-history-status').textContent.startsWith('Stored observation:'));
 assert.deepEqual(await read(page),expected);assert.deepEqual(await page.evaluate(()=>({pickerCalls,sourceCalls,effects})),{pickerCalls:0,sourceCalls:0,effects:0});assert(await page.locator('#fetch-folder-review').isDisabled());assert(await page.locator('#fetch-folder-apply').isDisabled());assert.equal(await page.locator('#fetch-folder-force').isChecked(),false);
}
async function files(page){return page.evaluate(async()=>{let d=await navigator.storage.getDirectory();for(const n of ['owned-history-test','V3','data','2020','adjacency'])d=await d.getDirectoryHandle(n);const out={};for await(const [name,h]of d.entries()){if(h.kind==='file')out[name]=Array.from(new Uint8Array(await(await h.getFile()).arrayBuffer()));}return out;});}
async function complete(page){await review(page);await page.locator('#fetch-folder-apply').click();await page.waitForFunction(()=>document.querySelector('#fetch-folder-status').textContent.startsWith('Reviewed files installed'));const value=await read(page);assert.equal(value.outcome.status,'completed');assert.equal(value.outcome.receipt.marker_committed,true);const actual=await files(page);assert.equal(Buffer.from(actual['ri_adjacency_2020.done']).toString(),'done');for(const a of assets)assert.equal(sha(Buffer.from(actual[a.name])),sha(a.bytes));return value;}
let result='failed';
try{
 let page=await launch();await review(page);await page.evaluate(()=>pauseFirst=true);await page.locator('#fetch-folder-apply').click();await page.waitForFunction(()=>paused);const initial=await read(page);assert.equal(initial.outcome.status,'in-progress');assert.equal(initial.outcome.receipt,null);assert.equal(await page.evaluate(()=>effects),1);const partial=await files(page);assert.equal(Object.keys(partial).length,1);assert.equal(partial['ri_adjacency_2020.done'],undefined);console.log('Reached crash checkpoint '+(crashes+1));records.push({checkpoint:'first-source-close',record:initial,closed_paths:Object.keys(partial)});await crash();
 const before=sourceRequests;page=await launch();await inert(page,initial);assert.equal(sourceRequests,before);assert.deepEqual(await files(page),partial);assert.match(await page.locator('#fetch-history-status').textContent(),/completed writes are unknown/);const completed=await complete(page);assert.notEqual(completed.id,initial.id);
 // Force invalidates the old completion marker before the next source close.
 await page.locator('#fetch-folder-force').check();await page.evaluate(()=>authorizeSource());await page.locator('#fetch-folder-review').click();await page.waitForFunction(()=>!document.querySelector('#fetch-folder-apply').disabled);await page.evaluate(()=>pauseFirst=true);await page.locator('#fetch-folder-apply').click();await page.waitForFunction(()=>paused);const forced=await read(page);assert.equal(forced.outcome.status,'in-progress');assert.equal(forced.outcome.force,true);assert.equal(forced.outcome.receipt,null);assert.equal((await files(page))['ri_adjacency_2020.done'],undefined);console.log('Reached crash checkpoint '+(crashes+1));records.push({checkpoint:'forced-first-source-close-after-marker-removal',record:forced});await crash();
 page=await launch();await inert(page,forced);assert.equal((await files(page))['ri_adjacency_2020.done'],undefined);const recovered=await complete(page);assert.equal(recovered.outcome.force,false);assert.notEqual(recovered.id,forced.id);console.log('Reached crash checkpoint '+(crashes+1));records.push({checkpoint:'terminal-receipt-and-marker-verified',record:recovered});await crash();
 page=await launch();await inert(page,recovered);const finalFiles=await files(page);for(const a of assets)assert.equal(sha(Buffer.from(finalFiles[a.name])),sha(a.bytes));assert.equal(Buffer.from(finalFiles['ri_adjacency_2020.done']).toString(),'done');assert.equal(sourceRequests,4);assert.deepEqual(errors,[]);await verifyPackage();assert.equal(sha(await fs.readFile(import.meta.filename)),sourcePin.sha256,'Test source changed during the gate');result='passed';
}finally{
 try{await browser?.close();}finally{if(child&&child.exitCode===null)await crash(false);await new Promise(r=>server.close(r));}
 // The test profile is owned and contains only the local fixture. Do not retain
 // raw browser profiles as evidence. Verify its resolved boundary before removal.
 assert.equal(await fs.realpath(profileRoot),realProfile);assert(realProfile.startsWith(parent+path.sep));assert(!(await fs.lstat(profileRoot)).isSymbolicLink());await fs.rm(realProfile,{recursive:true,force:true});
 await fs.writeFile(output+'/evidence.json',JSON.stringify({status:result,profile,package_validation:validationPath,test_source:sourcePin,actual_browser:true,actual_persistent_indexeddb_and_opfs:true,abrupt_owned_windows_process_tree_terminations:crashes,first_file_and_forced_marker_invalidation_checkpoints:true,terminal_receipt_survives_process_restart:result==='passed',no_restored_picker_source_review_force_or_auto_resume:result==='passed',explicit_re_review_recovers:result==='passed',native_external_directory_permissions_claim:false,power_loss_durability_claim:false,profile_removed:true,harness_cleanup_owned_process_termination:true,accepted_final_gate:result==='passed',records},null,2)+'\n');console.log(JSON.stringify({status:result,crashes,evidence:path.relative(process.cwd(),output+'/evidence.json')}));
}
