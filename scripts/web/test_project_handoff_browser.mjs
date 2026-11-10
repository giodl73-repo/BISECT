import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject} from '../../web/lab/laboratory-project.js';

const packages=process.env.BISECT_PLAYWRIGHT_PACKAGES;assert(packages);
const {chromium}=createRequire(path.join(packages,'package.json'))('playwright');
const root=path.resolve('target/project-handoff-browser');await fs.mkdir(root,{recursive:true});
const source=path.resolve('web/lab'),wasm=path.resolve('target/wasm32-unknown-unknown/release/bisect_wasm.wasm');
const laboratory=path.resolve(process.argv[2]||'dist/wasm-lab-build');
const manifest=JSON.parse(await fs.readFile(path.join(laboratory,'catalog.json'),'utf8'));
assert(manifest.graphs['RI:2020'],'The browser test requires the prepared RI 2020 catalog graph.');
const engine=await instantiateEngine(await fs.readFile(wasm));
class EngineWorker {postMessage(message){setImmediate(()=>{if(this.terminated)return;try{this.onmessage({data:message.type==='initialize'?{type:'ready'}:{type:'result',id:message.id,result:engine.execute(message.request)}});}catch(error){this.onmessage({data:{type:'error',id:message.id,error:error.message}});}});}terminate(){this.terminated=true;}}
const fetcher=async ref=>{const bytes=await fs.readFile(path.join(laboratory,ref));return{ok:true,arrayBuffer:async()=>bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.length)};};
const lab=new WasmCatalog(manifest,fetcher,()=>new EngineWorker());
const run=lab.submit({name:'Direct workbench transfer',mode:'state',states:['RI'],year:'2020',chamber:'congressional',districts:null,structure:'standard-bisect',weights:'geographic',search:'single',seed:42,seeds:2,steps:20,percentile:0,alpha_county:0,balance_tolerance:5,area_swing:1.1,iterations:10,timeout_seconds:60});
while(lab.active)await new Promise(resolve=>setTimeout(resolve,5));assert.equal(lab.jobs.get(run.id).status,'completed');
const fixture=(await lab.exportPractitionerProject(run.id,'RI')).project;
const labFile=path.join(root,'generated-laboratory.bisect');await fs.writeFile(labFile,JSON.stringify(await createLabProject(lab,run.id)));
const server=createServer(async(req,res)=>{try{
  const name=new URL(req.url,'http://localhost').pathname;
  const file=name==='/bisect_wasm.wasm'?wasm:name.startsWith('/laboratory/')?path.resolve(laboratory,name.slice('/laboratory/'.length)+(name.endsWith('/')?'index.html':'')):path.resolve(source,'.'+(name==='/'?'/workbench.html':name));
  if(file!==wasm&&!file.startsWith(source+path.sep)&&!file.startsWith(laboratory+path.sep))throw new Error('Path refused');
  res.setHeader('Content-Type',file.endsWith('.js')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':file.endsWith('.json')?'application/json':'text/html');res.end(await fs.readFile(file));
}catch{res.writeHead(404);res.end();}});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));const origin='http://127.0.0.1:'+server.address().port;let context;
try{
  context=await chromium.launchPersistentContext(path.join(root,'profile-'+Date.now()),{headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-background-networking','--disable-extensions']});
  await context.route('**/*',route=>new URL(route.request().url()).origin===origin?route.continue():route.abort());
  const page=await context.newPage(),errors=[];page.on('pageerror',e=>errors.push(e.message));
  await page.addInitScript(()=>{window.runs=0;const W=Worker;window.Worker=class extends W{constructor(url,...rest){super(url,...rest);this.importWorker=String(url).includes('/project-worker.js');}postMessage(v,...rest){if(v.type==='run')window.runs++;if(this.importWorker&&location.search.includes('import=held')){window.releaseImport=()=>super.postMessage(v,...rest);return;}return super.postMessage(v,...rest);}};});
  await page.goto(origin+'/');await page.waitForFunction(()=>!document.querySelector('#execute').disabled);
  // Real IndexedDB transaction tests, including atomic single consumption.
  const checks=await page.evaluate(async project=>{
    const {queueProjectHandoff,takeProjectHandoff,handoffToken}=await import('/project-handoff.js');
    const {writeAutosave}=await import('/project.js');
    await writeAutosave({...project,name:'Previous saved project'});
    const first=await queueProjectHandoff(project),second=await queueProjectHandoff(project);
    if(await takeProjectHandoff(first)!==null)throw new Error('Older transfer retained');
    if(await takeProjectHandoff('0'.repeat(32))!==null)throw new Error('Wrong token consumed transfer');
    const concurrent=await Promise.all([takeProjectHandoff(second),takeProjectHandoff(second)]);
    if(concurrent.filter(Boolean).length!==1)throw new Error('Transfer replayed');
    const opened=JSON.parse(await concurrent.find(Boolean).text());
    if(opened.name!==project.name)throw new Error('Transfer changed');
    for(const hash of ['#project=bad','#project='+first+'&url=https://invalid.example','#project='+first.toUpperCase()])if(handoffToken(hash)!==null)throw new Error('Invalid URL accepted');
    if(handoffToken('#project='+first)!==first)throw new Error('Valid token rejected');
    async function put(record){await new Promise((resolve,reject)=>{const open=indexedDB.open('bisect-project-handoff',1);open.onsuccess=()=>{const db=open.result,tx=db.transaction('transfers','readwrite');tx.objectStore('transfers').put(record,'pending');tx.oncomplete=()=>{db.close();resolve();};tx.onerror=()=>{db.close();reject(tx.error);};};open.onerror=()=>reject(open.error);});}
    await put({token:first,expires:Date.now()-1,blob:new Blob(['{}'])});if(await takeProjectHandoff(first)!==null)throw new Error('Expired transfer accepted');
    await put({token:first,expires:Date.now()+60000,blob:new Blob(['x'.repeat(25*1024*1024+1)])});if(await takeProjectHandoff(first)!==null)throw new Error('Oversize transfer accepted');
    await put({token:first,expires:Date.now()+60000,blob:{size:1}});if(await takeProjectHandoff(first)!==null)throw new Error('Invalid blob accepted');
    return {token:await queueProjectHandoff(project),storage_cases:8};
  },fixture);
  await page.goto(origin+'/?import=first#project='+checks.token);
  try { await page.waitForFunction(()=>document.querySelector('#project-status').textContent.startsWith('Opened laboratory plan')); }
  catch(error) { throw new Error(error.message+'; status: '+await page.locator('#project-status').textContent()+'; page errors: '+errors.join('; ')); }
  assert.equal(await page.evaluate(()=>runs),0);assert.equal(new URL(page.url()).hash,'');
  assert.equal(await page.locator('#project-name').inputValue(),fixture.name);
  const retained=await page.evaluate(async()=> (await (await import('/project.js')).readAutosave()).name);assert.equal(retained,'Previous saved project');
  const download=page.waitForEvent('download');await page.locator('#save-project').click();
  const savedFile=path.join(root,'transferred.bisect');await(await download).saveAs(savedFile);const saved=JSON.parse(await fs.readFile(savedFile,'utf8'));
  for(const [key,value]of Object.entries(fixture.files))assert.deepEqual(saved.files[key],value);
  // Reopening a consumed URL leaves a fresh workbench and the autosave intact.
  await page.goto(origin+'/?import=replay#project='+checks.token);await page.waitForFunction(()=>document.querySelector('#project-status').textContent.includes('already opened'));
  assert.equal(await page.evaluate(()=>runs),0);assert.notEqual(await page.locator('#project-name').inputValue(),fixture.name);
  await page.locator('#restore-project').click();await page.waitForFunction(()=>document.querySelector('#project-name').value==='Previous saved project');
  // A hostile stored payload follows the usual import worker rejection path.
  const bad=await page.evaluate(async()=>{const {queueProjectHandoff}=await import('/project-handoff.js');const project=await(await import('/project.js')).readAutosave();const token=await queueProjectHandoff(project);await new Promise((resolve,reject)=>{const open=indexedDB.open('bisect-project-handoff',1);open.onsuccess=()=>{const db=open.result,tx=db.transaction('transfers','readwrite');tx.objectStore('transfers').put({token,expires:Date.now()+60000,blob:new Blob(['{"__proto__":{"polluted":true}}'])},'pending');tx.oncomplete=()=>{db.close();resolve();};tx.onerror=()=>reject(tx.error);};});return token;});
  await page.goto(origin+'/?import=hostile#project='+bad);await page.waitForFunction(()=>document.querySelector('#project-status').textContent.includes('Unsafe'));
  assert.equal(await page.evaluate(()=>runs),0);assert.equal(await page.evaluate(async()=> (await(await import('/project.js')).readAutosave()).name),'Previous saved project');
  const held=await page.evaluate(async project=>(await import('/project-handoff.js')).queueProjectHandoff(project),fixture);
  await page.goto(origin+'/?import=held#project='+held);await page.waitForFunction(()=>typeof releaseImport==='function');
  await page.locator('#project-name').fill('Edits made during import');await page.evaluate(()=>releaseImport());
  await page.waitForFunction(()=>document.querySelector('#project-status').textContent.startsWith('Current project changed during transfer'));
  assert.equal(await page.locator('#project-name').inputValue(),'Edits made during import');assert.equal(await page.evaluate(()=>runs),0);
  // Exercise the actual lab controls and cross-directory navigation too.
  await page.goto(origin+'/laboratory/');await page.locator('#open-lab-project').waitFor();
  await page.locator('#open-lab-project').setInputFiles(labFile);
  await page.waitForFunction(()=>document.querySelector('#lab-project-status').textContent.startsWith('Opened project.'));
  await page.locator('#practitioner-tools summary').click();await page.locator('#export-audit-project').click();
  await page.waitForFunction(()=>!document.querySelector('#open-audit-project').disabled);
  await page.locator('#open-audit-project').click();
  await page.waitForURL('**/toolkit/**');
  await page.waitForFunction(()=>document.querySelector('#project-status').textContent.startsWith('Opened laboratory plan'));
  assert.equal(await page.evaluate(()=>runs),0);assert.equal(new URL(page.url()).hash,'');
  assert.match(await page.locator('#project-name').inputValue(),/RI/);
  assert.deepEqual(errors,[]);
  await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({storage_cases:checks.storage_cases,actual_lab_button_to_workbench:true,actual_workbench_import:true,source_bytes_retained:true,engine_runs_on_open:0,previous_autosave_preserved:true,replay_rejected:true,hostile_payload_rejected:true,edits_during_import_preserved:true,temporary_token_removed_from_url:true},null,2));
  console.log('Project transfer: real IndexedDB bounds, expiry, atomic single use, actual inert workbench import, exact sources, autosave preservation and hostile payload rejection passed.');
}finally{await context?.close();await new Promise(resolve=>server.close(resolve));}
