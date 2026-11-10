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
const source=path.resolve(process.argv[2]||'dist/wasm-lab-build'),root=path.resolve('target/burst-controls-browser');await fs.mkdir(root,{recursive:true});
const manifest=JSON.parse(await fs.readFile(path.join(source,'catalog.json'),'utf8')),engine=await instantiateEngine(await fs.readFile(path.join(source,'bisect_wasm.wasm')));
class EngineWorker{postMessage(m){setImmediate(()=>{if(this.terminated)return;try{this.onmessage({data:m.type==='initialize'?{type:'ready'}:{type:'result',id:m.id,result:engine.execute(m.request)}});}catch(e){this.onmessage({data:{type:'error',id:m.id,error:e.message}});}});}terminate(){this.terminated=true;}}
const fetcher=async ref=>{const b=await fs.readFile(path.join(source,ref));return{ok:true,arrayBuffer:async()=>b.buffer.slice(b.byteOffset,b.byteOffset+b.length)};};
const lab=new WasmCatalog(manifest,fetcher,()=>new EngineWorker());
const config={name:'Burst control browser',mode:'state',states:['RI'],year:'2020',chamber:'congressional',districts:null,structure:'standard-bisect',weights:'geographic',alpha_county:0,search:'short-burst',burst_length:2,n_bursts:3,seed:'18446744073709551615',seeds:8,steps:20,percentile:.5,balance_tolerance:25,area_swing:1.1,iterations:10,timeout_seconds:60};
const job=lab.submit(config);while(lab.active)await new Promise(r=>setTimeout(r,5));assert.equal(lab.jobs.get(job.id).status,'completed');const fixture=path.join(root,'initial.bisect');await fs.writeFile(fixture,JSON.stringify(await createLabProject(lab,job.id)));
const server=createServer(async(req,res)=>{try{const name=new URL(req.url,'http://localhost').pathname,file=path.resolve(source,'.'+(name==='/'?'/index.html':name));if(!file.startsWith(source+path.sep))throw new Error('Path refused');res.setHeader('Content-Type',file.endsWith('.js')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':file.endsWith('.json')?'application/json':'text/html');res.end(await fs.readFile(file));}catch{res.writeHead(404);res.end();}});
await new Promise(r=>server.listen(0,'127.0.0.1',r));const origin='http://127.0.0.1:'+server.address().port;let context;
try{
 context=await chromium.launchPersistentContext(path.join(root,'profile-'+Date.now()),{headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-background-networking','--disable-extensions']});await context.route('**/*',route=>new URL(route.request().url()).origin===origin?route.continue():route.abort());
 const page=await context.newPage(),errors=[];page.on('pageerror',e=>errors.push(e.message));await page.addInitScript(()=>{window.engineRequests=[];const W=Worker;window.Worker=class extends W{postMessage(v,...rest){if(v.type==='run')engineRequests.push(v.request);return super.postMessage(v,...rest);}};});
 await page.goto(origin+'/');await page.locator('#open-lab-project').waitFor();await page.locator('#open-lab-project').setInputFiles(fixture);await page.waitForFunction(()=>document.querySelector('#lab-project-status').textContent.startsWith('Opened project.'));assert.equal(await page.evaluate(()=>engineRequests.length),0);
 await page.locator('#burst-length').evaluate(element=>{const details=element.closest('details');if(details)details.open=true;});assert(await page.locator('#burst-fields').isVisible());
 const cases=[];
 for(const search of ['short-burst','short-burst-forest','short-burst-merge-split'])for(const length of [0,2]){
  await page.locator('#search').selectOption(search);await page.locator('#burst-length').fill(String(length));await page.locator('#n-bursts').fill('3');const before=await page.evaluate(()=>engineRequests.length);await page.locator('#run').click();await page.waitForFunction(n=>engineRequests.length>n,before);await page.waitForFunction(()=>document.querySelector('#run-status').textContent==='completed');
  const request=await page.evaluate(()=>engineRequests.at(-1));assert.equal(request.options.burst_length,length);assert.equal(request.options.n_bursts,3);assert.equal(request.options.seed,'18446744073709551615');
  const yaml_text=`name: browser_burst\nalgorithm:\n  structure: standard-bisect\n  engine: metis-core\n  weights: geographic\n  search: ${search}\n  burst_length: ${length}\n  n_bursts: 3\n  percentile: 0.5\n  balance_tolerance: 25\n  seed: 18446744073709551615\n`;
  const file=path.join(root,search+'-'+length+'.json');await fs.writeFile(file,JSON.stringify({graph:request.graph,yaml_text,options:{districts:request.options.districts,iterations:10}}));const native=JSON.parse(execFileSync('target/release/examples/config_seed_reference.exe',[file],{encoding:'utf8',maxBuffer:32*1024*1024}));
  const wait=page.waitForEvent('download');await page.locator('#save-lab-project').click();const savedFile=path.join(root,search+'-'+length+'.bisect');await(await wait).saveAs(savedFile);const saved=JSON.parse(await fs.readFile(savedFile,'utf8'));
  assert.equal(saved.experiment.config.burst_length,length);assert.equal(saved.experiment.config.n_bursts,3);assert.deepEqual(saved.assignments.RI,native.assignments);assert.equal(saved.experiment.states[0].metrics.structure_evidence.total_proposals,length*3);
  const previous=await page.evaluate(()=>engineRequests.length);await page.locator('#open-lab-project').setInputFiles(savedFile);await page.waitForFunction(v=>document.querySelector('#burst-length').value===String(v),length);assert.equal(await page.evaluate(()=>engineRequests.length),previous);cases.push({search,length,bursts:3,exact_native_cli_assignments:true,inert_open:true});
 }
 await page.locator('#search').selectOption('single');assert(await page.locator('#burst-length').isDisabled());assert.equal(await page.locator('#burst-fields').isVisible(),false);await page.locator('#search').selectOption('short-burst');
 const before=await page.evaluate(()=>engineRequests.length);await page.locator('#burst-length').fill('100000');await page.locator('#n-bursts').fill('2');await page.locator('#run').click();assert.equal(await page.evaluate(()=>engineRequests.length),before);
 assert.deepEqual(errors,[]);await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({real_state:'RI',cases,exact_controls_forwarded:true,full_u64_seed_retained:true,scope_visibility_and_disabled_inputs:true,oversize_proposal_product_never_executes:true,open_executes_engine:false},null,2));
 console.log('Burst browser: all three variants/zero work, actual RI native CLI matches, exact controls/u64 seed, inert Save/Open and proposal budget guard passed.');
}finally{await context?.close();await new Promise(r=>server.close(r));}
