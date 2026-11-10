import {openWorkbenchProject} from './workbench_project_open.mjs';
// Isolated localhost/package and browser; no ambient tabs or external data.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {createProject} from '../../web/lab/project.js';
import {cffiEngineCase} from './fixtures/cffi-engine-case.mjs';
const packageRoot=process.env.BISECT_PLAYWRIGHT_PACKAGES;
if(!packageRoot)throw new Error('Set BISECT_PLAYWRIGHT_PACKAGES to an installed Playwright root.');
const {chromium}=createRequire(path.join(packageRoot,'package.json'))('playwright');
const root=path.resolve(process.env.BISECT_CFFI_BROWSER_EVIDENCE||'target/cffi-integration/browser'),staticRoot=path.resolve(process.argv[2]||'dist/wasm-cffi-preview/toolkit');
await fs.mkdir(root,{recursive:true});
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const packageWasm=await fs.readFile(path.join(staticRoot,'bisect_wasm.wasm'));
const packageSidecar=JSON.parse(await fs.readFile(path.join(staticRoot,'c-metis/metis-sidecar.json'),'utf8'));
const request=cffiEngineCase();Object.assign(request.options,{metis_engine:'c-ffi',metis_runtime:'msvcrt-v1'});
const inputs={source:{name:'explicit-c-request.json',text:JSON.stringify(request)},settings:{label:'C METIS browser',chamber:'congressional',created_at:'2026-10-22T00:00:00Z',inspect_only:false}};
const source=JSON.stringify(createProject({name:'Explicit C backend',operation:'run-prepared-engine',files:{preparedEngineInputs:inputs},constraints:[],result:null,lastOperation:null}));
const records=JSON.parse(await fs.readFile(process.env.BISECT_CFFI_ENGINE_RECORDS??'target/cffi-integration/engine-records.json','utf8'));
const reference=records.find(r=>r.request.graph.population.length===64&&r.request.options.structure==='standard-bisect'&&r.request.options.search==='single'&&r.request.options.weights==='geographic'&&r.request.options.seed==='7');assert(reference.result);
const requests=[];let slowResponse,notifySlow;
const slowStarted=new Promise(resolve=>notifySlow=resolve);
const raceModule=`const instantiate=WebAssembly.instantiate.bind(WebAssembly);WebAssembly.instantiate=async(...args)=>{const result=await instantiate(...args);postMessage({type:'probe-instantiated'});return result;};await import('./wasm-worker.js');postMessage({type:'probe-loaded'});`;
const server=createServer(async(req,res)=>{try{
 const url=new URL(req.url,'http://localhost'),file=path.resolve(staticRoot,'.'+decodeURIComponent(url.pathname==='/'?'/index.html':url.pathname));
 if(url.pathname==='/worker-race.mjs'){res.setHeader('Content-Type','application/javascript');res.end(raceModule);return;}
 if(url.pathname==='/slow-engine.wasm'){slowResponse=res;notifySlow();return;}
 if(!file.startsWith(staticRoot+path.sep))throw new Error('Path rejected');requests.push(url.pathname);
 res.setHeader('Content-Type',file.endsWith('.js')||file.endsWith('.mjs')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':file.endsWith('.json')?'application/json':'text/html');res.setHeader('Cache-Control','no-store');res.end(await fs.readFile(file));
}catch{res.writeHead(404);res.end();}});await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const origin='http://127.0.0.1:'+server.address().port;let browser;
try{
 browser=await chromium.launch({headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-extensions','--disable-background-networking']});
 const context=await browser.newContext();await context.route('**/*',route=>new URL(route.request().url()).origin===origin?route.continue():route.abort());
 const page=await context.newPage(),errors=[];page.on('pageerror',error=>errors.push(error.message));
 await page.addInitScript(()=>{window.engineRequests=[];const OriginalWorker=Worker;window.Worker=class extends OriginalWorker{postMessage(value,...rest){if(value.type==='run')engineRequests.push(structuredClone(value));return super.postMessage(value,...rest);}};});
 await page.goto(origin);await page.waitForFunction(()=>!document.querySelector('#execute').disabled);
 assert(await page.locator('#c-metis-runtime-control').isVisible());assert(!requests.some(p=>p.startsWith('/c-metis/')));
 await page.locator('#open-project').setInputFiles({name:'explicit-c.bisect',mimeType:'application/json',buffer:Buffer.from(source)});
 await page.waitForFunction(()=>document.querySelector('#operation').value==='run-prepared-engine');assert.equal(await page.evaluate(()=>engineRequests.length),0);assert(!requests.some(p=>p.startsWith('/c-metis/')));
 await page.locator('#execute').click();await page.waitForFunction(()=>document.querySelector('#result-status').textContent.includes('Load the verified C METIS sidecar'));assert(!requests.some(p=>p.startsWith('/c-metis/')));assert(await page.locator('#download-prepared-engine-plan').isDisabled());
 await page.locator('#c-metis-runtime').check();assert(!requests.some(p=>p.startsWith('/c-metis/')),'Selecting runtime is inert until Run');
 await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-prepared-engine-report').disabled,{},{timeout:30000});
 const download=page.waitForEvent('download');await page.locator('#download-prepared-engine-report').click();await(await download).saveAs(root+'/result.json');
 const result=JSON.parse(await fs.readFile(root+'/result.json','utf8'));
 assert.match(await page.locator('#result-status').textContent(),/plan generated/);
 assert.equal(result.execution_performed,true);assert.equal(result.runtime_artifacts.metis_sidecar.runtime,'msvcrt-v1');
 assert.deepEqual(result.runtime_artifacts.engine,{sha256:hash(packageWasm),byte_length:packageWasm.length});
 for(const key of ['javascript','wasm'])assert.deepEqual(result.runtime_artifacts.metis_sidecar[key],packageSidecar[key]);
 assert.deepEqual(result.result.assignments,reference.native.assignments);
 for(const name of ['metis-sidecar.json','metis-sidecar.mjs','metis-sidecar.wasm'])assert(requests.includes('/c-metis/'+name));
 const save=page.waitForEvent('download');await page.locator('#save-project').click();await(await save).saveAs(root+'/completed.bisect');
 const saved=JSON.parse(await fs.readFile(root+'/completed.bisect','utf8'));assert.equal(saved.result.runtime_artifacts.metis_sidecar.wasm.sha256,result.runtime_artifacts.metis_sidecar.wasm.sha256);
 const before={runs:await page.evaluate(()=>engineRequests.length),assets:requests.filter(p=>p.startsWith('/c-metis/')).length};
 await openWorkbenchProject(page,root+'/completed.bisect',{ready:({argument:__bisectOpenArgument6768,openedStatus:__bisectOpenedStatus6768})=>(()=>__bisectOpenedStatus6768.startsWith('Opened completed.bisect'))(__bisectOpenArgument6768),readyArg:undefined});
 assert.equal(await page.evaluate(()=>engineRequests.length),before.runs);assert.equal(requests.filter(p=>p.startsWith('/c-metis/')).length,before.assets);assert(!(await page.locator('#c-metis-runtime').isChecked()));assert(await page.locator('#download-prepared-engine-plan').isDisabled());
 await page.setViewportSize({width:390,height:844});assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1));await page.screenshot({path:root+'/mobile.png',fullPage:true});assert.deepEqual(errors,[]);
 // Finish two real instantiations out of order. An older initialization must
 // never replace the newer engine or emit a second ready reply.
 await page.evaluate(()=>{window.raceMessages=[];window.raceWorker=new Worker('/worker-race.mjs',{type:'module'});raceWorker.onmessage=({data})=>raceMessages.push(data);});
 await page.waitForFunction(()=>raceMessages.some(m=>m.type==='probe-loaded'));
 await page.evaluate(()=>raceWorker.postMessage({type:'initialize',url:'/slow-engine.wasm'}));
 await Promise.race([slowStarted,new Promise((_,reject)=>setTimeout(()=>reject(new Error('Slow initialization did not start')),5000))]);
 await page.evaluate(()=>raceWorker.postMessage({type:'initialize',url:'/bisect_wasm.wasm'}));
 await page.waitForFunction(()=>raceMessages.some(m=>m.type==='ready'));
 slowResponse.setHeader('Content-Type','application/wasm');slowResponse.end(await fs.readFile(path.join(staticRoot,'bisect_wasm.wasm')));
 await page.waitForFunction(()=>raceMessages.filter(m=>m.type==='probe-instantiated').length===2);
 await page.evaluate(()=>raceWorker.postMessage({type:'probe-finished'}));
 await page.waitForFunction(()=>raceMessages.some(m=>m.type==='error'&&m.error==='Unknown worker request.'));
 assert.equal(await page.evaluate(()=>raceMessages.filter(m=>m.type==='ready').length),1);await page.evaluate(()=>raceWorker.terminate());
 await fs.writeFile(root+'/evidence.json',JSON.stringify({schema:'bisect-cffi-engine-browser-v1',tested_toolkit:staticRoot,test_source_sha256:hash(await fs.readFile('scripts/web/test_cffi_engine_browser.mjs')),loaded_artifacts:result.runtime_artifacts,browser:browser.version(),actual_workbench_worker:true,native_assignments_exact:true,inert_open_and_runtime_selection:true,missing_runtime_distinguished_before_execution:true,three_sidecar_assets_loaded_on_run:true,versioned_runtime_identities_retained:true,open_retires_exports_and_runtime_selection:true,stale_initialization_suppressed_after_two_real_instantiations:true,mobile_width:390,page_errors:errors,requests},null,2)+'\n');
 console.log('PASS browser C backend: real workbench worker/native assignments, lazy verified assets, inert Open/selection, retained runtime hashes and retired exports.');
}catch(error){console.error('Browser:',error.message);if(browser){const pages=browser.contexts().flatMap(c=>c.pages());if(pages[0])console.error('Status:',await pages[0].locator('#result-status').textContent());}throw error;}
finally{await browser?.close();await new Promise(resolve=>server.close(resolve));}
