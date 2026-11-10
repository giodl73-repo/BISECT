import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject} from '../../web/lab/laboratory-project.js';

const packages=process.env.BISECT_PLAYWRIGHT_PACKAGES;assert(packages);const {chromium}=createRequire(path.join(packages,'package.json'))('playwright');
const source=path.resolve(process.argv[2]||'dist/wasm-lab-build'),root=path.resolve('target/area-init-browser');await fs.mkdir(root,{recursive:true});
const manifest=JSON.parse(await fs.readFile(path.join(source,'catalog.json'),'utf8')),engine=await instantiateEngine(await fs.readFile(path.join(source,'bisect_wasm.wasm')));
class EngineWorker{postMessage(m){setImmediate(()=>{if(this.terminated)return;try{this.onmessage({data:m.type==='initialize'?{type:'ready'}:{type:'result',id:m.id,result:engine.execute(m.request)}});}catch(e){this.onmessage({data:{type:'error',id:m.id,error:e.message}});}});}terminate(){this.terminated=true;}}
const fetcher=async ref=>{const b=await fs.readFile(path.join(source,ref));return{ok:true,arrayBuffer:async()=>b.buffer.slice(b.byteOffset,b.byteOffset+b.length)};};
const lab=new WasmCatalog(manifest,fetcher,()=>new EngineWorker());
const job=lab.submit({name:'Area initializer browser',mode:'state',states:['RI'],year:'2020',chamber:'congressional',districts:null,structure:'ratio-optimal-area',weights:'geographic',alpha_county:0,area_init:'ratio-optimal',search:'multi',seed:42,seeds:3,steps:20,percentile:0,balance_tolerance:25,area_swing:1.1,iterations:10,timeout_seconds:60});
while(lab.active)await new Promise(r=>setTimeout(r,5));assert.equal(lab.jobs.get(job.id).status,'completed');
const fixture=path.join(root,'initial.bisect');await fs.writeFile(fixture,JSON.stringify(await createLabProject(lab,job.id)));
const server=createServer(async(req,res)=>{try{const name=new URL(req.url,'http://localhost').pathname,file=path.resolve(source,'.'+(name==='/'?'/index.html':name));if(!file.startsWith(source+path.sep))throw new Error('Path refused');res.setHeader('Content-Type',file.endsWith('.js')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':file.endsWith('.json')?'application/json':'text/html');res.end(await fs.readFile(file));}catch{res.writeHead(404);res.end();}});
await new Promise(r=>server.listen(0,'127.0.0.1',r));const origin='http://127.0.0.1:'+server.address().port;let context;
try{
 context=await chromium.launchPersistentContext(path.join(root,'profile-'+Date.now()),{headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-background-networking','--disable-extensions']});await context.route('**/*',route=>new URL(route.request().url()).origin===origin?route.continue():route.abort());
 const page=await context.newPage(),errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.addInitScript(()=>{window.engineRequests=[];const W=Worker;window.Worker=class extends W{postMessage(v,...rest){if(v.type==='run')engineRequests.push(v.request);return super.postMessage(v,...rest);}};});
 await page.goto(origin+'/');await page.locator('#open-lab-project').waitFor();await page.locator('#open-lab-project').setInputFiles(fixture);await page.waitForFunction(()=>document.querySelector('#lab-project-status').textContent.startsWith('Opened project.'));assert.equal(await page.evaluate(()=>engineRequests.length),0);
 await page.locator('#area-init').evaluate(element=>{const details=element.closest('details');if(details)details.open=true;});assert(await page.locator('#area-init-field').isVisible());
 assert.match(await page.locator('#area-init-help').textContent(),/legacy CLI areasection preset, use seed 1/);
 const cases=[];
 for(const repair of ['ratio-optimal','moving-knife']){
  await page.locator('#area-init').selectOption(repair);const before=await page.evaluate(()=>engineRequests.length);await page.locator('#run').click();await page.waitForFunction(n=>engineRequests.length>n,before);await page.waitForFunction(()=>document.querySelector('#run-status').textContent==='completed');
  const request=await page.evaluate(()=>engineRequests.at(-1));assert.equal(request.options.area_init,repair);
  const file=path.join(root,repair+'-request.json');await fs.writeFile(file,JSON.stringify(request));const native=JSON.parse(execFileSync('target/release/examples/execute_request.exe',[file],{encoding:'utf8',maxBuffer:32*1024*1024}));
  const stateRequest={graph:request.graph,districts:request.options.districts,state_args:['--state','RI','--year','2020','--metis-engine','metis-core','--partition-mode','edge-weighted','--structure','ratio-optimal-area','--area-section-init',repair,'--area-swing','1.1','--weights-override','geographic','--alpha-county','0','--search','multi','--seeds','3','--niter','10','--balance-tolerance','25','--seed','42']};
  const stateFile=path.join(root,repair+'-state-request.json');await fs.writeFile(stateFile,JSON.stringify(stateRequest));const stateReference=JSON.parse(execFileSync('target/release/examples/area_state_reference.exe',[stateFile],{encoding:'utf8',maxBuffer:32*1024*1024}));
  const wait=page.waitForEvent('download');await page.locator('#save-lab-project').click();const savedFile=path.join(root,repair+'.bisect');await(await wait).saveAs(savedFile);const saved=JSON.parse(await fs.readFile(savedFile,'utf8'));
  assert.equal(saved.experiment.config.area_init,repair);assert.deepEqual(saved.assignments.RI,native.assignments);assert.deepEqual(saved.experiment.states[0].metrics.structure_evidence,native.metrics.structure_evidence);assert.deepEqual(saved.assignments.RI,stateReference.assignments);assert.equal(saved.experiment.states[0].metrics.structure_evidence.direction_radians,stateReference.direction_radians);
  const previous=await page.evaluate(()=>engineRequests.length);await page.locator('#open-lab-project').setInputFiles(savedFile);await page.waitForFunction(r=>document.querySelector('#area-init').value===r,repair);assert.equal(await page.evaluate(()=>engineRequests.length),previous);
  cases.push({repair,actual_worker_native_assignments_equal:true,actual_cli_state_assignments_equal:true,exact_native_summary:true,inert_open:true});
 }
 await page.locator('#structure').selectOption('standard-bisect');assert(await page.locator('#area-init').isDisabled());assert.equal(await page.locator('#area-init-field').isVisible(),false);
 await page.locator('#structure').selectOption('ratio-optimal-area');assert.equal(await page.locator('#area-init').inputValue(),'moving-knife');assert.equal(await page.locator('#area-init').isDisabled(),false);
 assert.deepEqual(errors,[]);await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({real_state:'RI',cases,exact_option_forwarded:true,initializer_retained_in_project:true,scope_visibility_and_disabled_input:true,open_executes_engine:false},null,2));
 console.log('Area initializer browser: Both initializers run actual RI workers with exact native State assignments/directions and summaries, inert Save/Open, retained controls and scoped exposure.');
}finally{await context?.close();await new Promise(r=>server.close(r));}
