// Actual workbench/worker against current source over an owned existing package.
// Only the first C asset bundle is deliberately instrumented; no project can
// select the fixture. Later explicit Run receives the real pinned C bundle.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {createProject} from '../../web/lab/project.js';
import {cffiEngineCase} from './fixtures/cffi-engine-case.mjs';
import {createFaultSidecarAssets} from './fixtures/cffi-fault-cases.mjs';
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const root=await fs.mkdtemp(path.resolve('target/cffi-worker-recovery/run-'));
const packageRoot=path.resolve(process.argv[2]??'target/report-cffi/package/toolkit');
const sourceRoot=path.resolve('web/lab'),sidecar=path.join(packageRoot,'c-metis');
const previous=process.env.BISECT_CFFI_RECOVERY_PREVIOUS?path.resolve(process.env.BISECT_CFFI_RECOVERY_PREVIOUS):null;
const javascript=await fs.readFile(path.join(sidecar,'metis-sidecar.mjs')),binary=await fs.readFile(path.join(sidecar,'metis-sidecar.wasm'));
const manifest=JSON.parse(await fs.readFile(path.join(sidecar,'metis-sidecar.json'),'utf8'));
assert.equal(hash(javascript),manifest.javascript.sha256);assert.equal(hash(binary),manifest.wasm.sha256);
const sourcePaths=new Set(),requests=[];let active;
const servedHashes=new Map();
const server=createServer(async(req,res)=>{try{
  const url=new URL(req.url,'http://localhost').pathname;requests.push({mode:active?.mode,url});let bytes,file;
  if(url.startsWith('/c-metis/')){
    if(url.endsWith('/metis-sidecar.json')){active.loads++;active.fixtureSelected=active.loads===1;}
    const bundle=active.fixtureSelected?active.fixture:{javascript,wasm:binary,manifest};
    bytes=url.endsWith('.json')?Buffer.from(JSON.stringify(bundle.manifest)):url.endsWith('.mjs')?bundle.javascript:bundle.wasm;
  }else{
    const relative=url==='/'?'workbench.html':decodeURIComponent(url.slice(1));
    file=path.resolve(sourceRoot,relative);if(!file.startsWith(sourceRoot+path.sep))throw new Error('Path escaped source');
    if(previous&&['metis-c-sidecar.js','wasm-engine.js','wasm-worker.js','workbench.js'].includes(relative))file=path.join(previous,relative);
    try{bytes=await fs.readFile(file);sourcePaths.add(file);servedHashes.set(file,hash(bytes));}catch(error){if(error.code!=='ENOENT')throw error;file=path.resolve(packageRoot,relative);if(!file.startsWith(packageRoot+path.sep))throw error;bytes=await fs.readFile(file);}
  }
  const extension=path.extname(file??url);
  res.setHeader('Content-Type',['.js','.mjs'].includes(extension)?'application/javascript':extension==='.wasm'?'application/wasm':extension==='.css'?'text/css':extension==='.json'?'application/json':'text/html');
  res.setHeader('Cache-Control','no-store');res.end(bytes);
}catch{res.writeHead(404);res.end();}});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const origin='http://127.0.0.1:'+server.address().port;
const {chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES,'package.json'))('playwright');
const request=cffiEngineCase();Object.assign(request.options,{metis_engine:'c-ffi',metis_runtime:'msvcrt-v1'});
const inputs={source:{name:'c-request.json',text:JSON.stringify(request)},settings:{label:'Runtime recovery',chamber:'congressional',created_at:'2026-10-27T00:00:00Z',inspect_only:false}};
const project=Buffer.from(JSON.stringify(createProject({name:'Runtime recovery',operation:'run-prepared-engine',files:{preparedEngineInputs:inputs},constraints:[],result:null,lastOperation:null})));
const reference=JSON.parse(await fs.readFile('target/report-cffi/kernel/engine-records.json','utf8')).find(row=>row.request.graph.population.length===64&&row.request.options.structure==='standard-bisect'&&row.request.options.search==='single'&&row.request.options.weights==='geographic'&&row.request.options.seed==='7');
assert(reference?.native?.assignments);
const results=[];let browser;
try{
  browser=await chromium.launch({headless:true,executablePath:process.env.BISECT_TEST_BROWSER??'C:/Program Files/Google/Chrome/Application/chrome.exe'});
  for(const mode of ['partition-trap','free-trap','allocation-refusal']){
    active={mode,loads:0,fixture:await createFaultSidecarAssets({mode,javascript,binary,sha:hash})};
    const context=await browser.newContext();await context.route('**/*',route=>new URL(route.request().url()).origin===origin?route.continue():route.abort());
    const page=await context.newPage(),errors=[];page.on('pageerror',error=>errors.push(error.message));
    await page.addInitScript(()=>{window.workerRequests=[];window.workerReplies=[];const Original=Worker;window.Worker=class extends Original{constructor(...args){super(...args);this.addEventListener('message',({data})=>workerReplies.push(data));}postMessage(data,...args){if(data.type==='run')workerRequests.push(structuredClone(data));return super.postMessage(data,...args);}};});
    await page.goto(origin);await page.waitForFunction(()=>!document.querySelector('#execute').disabled);
    await page.locator('#open-project').setInputFiles({name:'recovery.bisect',mimeType:'application/json',buffer:project});
    await page.waitForFunction(()=>document.querySelector('#operation').value==='run-prepared-engine');
    assert.equal(active.loads,0);assert.equal(await page.evaluate(()=>workerRequests.length),0);
    await page.locator('#c-metis-runtime').check();assert.equal(active.loads,0);
    await page.locator('#execute').click();await page.waitForFunction(()=>workerReplies.some(data=>data.type==='error'));
    await page.waitForFunction(()=>!document.querySelector('#execute').disabled);
    const failure=await page.evaluate(()=>workerReplies.find(data=>data.type==='error'));
    assert(failure.failure?.request_json,'Failure retains the submitted request');
    assert.equal(failure.failure.submitted_operation,'run-prepared-engine');
    assert.equal(failure.runtimeRecovery,mode==='allocation-refusal'?undefined:'c-metis-reload-on-next-run');
    assert.equal(await page.evaluate(()=>workerRequests.length),1,'Failure must not replay automatically');
    assert.equal(active.loads,1);assert(await page.locator('#download-prepared-engine-plan').isDisabled());
    const status=await page.locator('#result-status').textContent();
    if(mode!=='allocation-refusal')assert.match(status,/C runtime stopped\. The next Run with C enabled will load a fresh runtime/);
    // Saving is an inert way to inspect the failed project's retained receipt.
    const download=page.waitForEvent('download');await page.locator('#save-project').click();
    const savedPath=path.join(root,mode+'-failed.bisect');await(await download).saveAs(savedPath);
    const saved=JSON.parse(await fs.readFile(savedPath,'utf8'));assert(saved.files.workerFailureReceipt);assert.equal(saved.result,null);
    assert.equal(active.loads,1);assert.equal(await page.evaluate(()=>workerRequests.length),1);
    await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-prepared-engine-report').disabled,{},{timeout:30000});
    const reportDownload=page.waitForEvent('download');await page.locator('#download-prepared-engine-report').click();
    const reportPath=path.join(root,mode+'-recovered.json');await(await reportDownload).saveAs(reportPath);
    const result=JSON.parse(await fs.readFile(reportPath,'utf8'));assert.equal(result.execution_performed,true);
    assert.deepEqual(result.result.assignments,reference.native.assignments);
    assert.equal(active.loads,mode==='allocation-refusal'?1:2,'Trap reloads; ordinary refusal reuses the live module');
    assert.equal(await page.evaluate(()=>workerRequests.length),2,'Exactly the two explicit Run clicks execute');
    if(mode!=='allocation-refusal')assert.deepEqual(result.runtime_artifacts.metis_sidecar.javascript,manifest.javascript);
    assert.doesNotMatch(await page.locator('#result-status').textContent(),/C runtime stopped/);
    assert.deepEqual(errors,[]);
    await page.screenshot({path:path.join(root,mode+'-recovered.png'),fullPage:true});
    results.push({mode,initial_failure:failure,initial_status:status,explicit_runs:2,sidecar_loads:active.loads,recovered_assignments_match_retained_native:true});
    await context.close();
  }
  const sources=await Promise.all([...sourcePaths].sort().map(async file=>{const sha256=hash(await fs.readFile(file));assert.equal(sha256,servedHashes.get(file),'Source changed after serving: '+file);return {path:path.relative(process.cwd(),file).replaceAll('\\','/'),sha256};}));
  if(process.argv.includes('--require-package'))for(const ref of sources){
    const name=path.basename(ref.path),packageFile=path.join(packageRoot,name==='workbench.html'?'index.html':name);
    assert.equal(hash(await fs.readFile(packageFile)),ref.sha256,'Final package must contain the tested source: '+name);
  }
  const testSources=await Promise.all(['scripts/web/test_cffi_worker_recovery_browser.mjs','scripts/web/fixtures/cffi-fault-cases.mjs','scripts/web/fixtures/cffi-engine-case.mjs','target/report-cffi/kernel/engine-records.json'].map(async file=>({path:file,sha256:hash(await fs.readFile(file))})));
  const evidence={status:'passed-workbench-C-recovery',browser:browser.version(),actual_workbench_worker:true,current_source_overlay:true,base_package:packageRoot,engine_sha256:hash(await fs.readFile(path.join(packageRoot,'bisect_wasm.wasm'))),real_sidecar:manifest,test_sources:testSources,sources,results,requests,scope:'Controlled real WASM trap/allocation refusal through current workbench and worker; explicit retry, inert save and retained native fixture assignments. Not full package, real exhaustion, termination or wider host acceptance.'};
  await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(evidence,null,2)+'\n');
  const receipt=process.argv.find(arg=>arg.startsWith('--receipt='));
  if(receipt)await fs.writeFile(path.resolve(receipt.slice(10)),JSON.stringify({status:evidence.status,evidence:{path:path.relative(process.cwd(),path.join(root,'evidence.json')).replaceAll('\\','/'),sha256:hash(await fs.readFile(path.join(root,'evidence.json')))}},null,2)+'\n');
  console.log('PASS actual workbench: C trap recovery on explicit Run; no automatic replay; allocation refusal reuses live module. '+root);
}finally{await browser?.close();await new Promise(resolve=>server.close(resolve));}
