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
const source=path.resolve(process.argv[2]||'dist/wasm-lab-build'),root=path.resolve('target/character-controls-browser');await fs.mkdir(root,{recursive:true});
const manifest=JSON.parse(await fs.readFile(path.join(source,'catalog.json'),'utf8')),engine=await instantiateEngine(await fs.readFile(path.join(source,'bisect_wasm.wasm')));
class EngineWorker{postMessage(m){setImmediate(()=>{if(this.terminated)return;try{this.onmessage({data:m.type==='initialize'?{type:'ready'}:{type:'result',id:m.id,result:engine.execute(m.request)}});}catch(e){this.onmessage({data:{type:'error',id:m.id,error:e.message}});}});}terminate(){this.terminated=true;}}
const fetcher=async ref=>{const b=await fs.readFile(path.join(source,ref));return{ok:true,arrayBuffer:async()=>b.buffer.slice(b.byteOffset,b.byteOffset+b.length)};};
const lab=new WasmCatalog(manifest,fetcher,()=>new EngineWorker()),config={name:'Character controls',mode:'state',states:['RI'],year:'2020',chamber:'congressional',districts:null,structure:'standard-bisect',weights:'economic-character',alpha_county:0,search:'multi',character_alpha:.5,characters:{},seed:'18446744073709551615',seeds:3,steps:8,percentile:.5,balance_tolerance:10,area_swing:1.1,iterations:10,timeout_seconds:60};
const graph=JSON.parse(await fs.readFile(path.join(source,manifest.graphs['RI:2020'].graph_ref),'utf8')),fixtures={};
for(const kind of ['economic','housing']){
 const header=kind==='economic'?'geoid,c000,cns01,cns02,cns05,cns07,cns08,cns09,cns10,cns11':'geoid,pct_single_family,pct_multifamily,pct_owner,housing_vintage';
 const rows=graph.geoids.map((id,i)=>id+','+(kind==='economic'?(i%3===0?'0,0,0,0,0,0,0,0,0':'1000,10,20,30,400,40,100,50,50'):(i%3===0?'0,0,0,0':'.8,.1,.6,.2')));
 const input=engine.execute({operation:'import-character-csv',state:'RI',year:'2020',data_year:'2022',kind,source_label:'SYNTHETIC browser observations; not LODES/ACS records',source_base64:Buffer.from(header+'\n'+rows.join('\n')+'\n').toString('base64')});
 const job=lab.submit({...config,weights:kind+'-character',characters:{RI:input}});while(lab.active)await new Promise(r=>setTimeout(r,5));assert.equal(lab.jobs.get(job.id).status,'completed');const file=path.join(root,kind+'-initial.bisect');await fs.writeFile(file,JSON.stringify(await createLabProject(lab,job.id)));fixtures[kind]={file,input};
}
const server=createServer(async(req,res)=>{try{const name=new URL(req.url,'http://localhost').pathname,file=path.resolve(source,'.'+(name==='/'?'/index.html':name));if(!file.startsWith(source+path.sep))throw new Error('Path refused');res.setHeader('Content-Type',file.endsWith('.js')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':file.endsWith('.json')?'application/json':'text/html');res.end(await fs.readFile(file));}catch{res.writeHead(404);res.end();}});await new Promise(r=>server.listen(0,'127.0.0.1',r));const origin='http://127.0.0.1:'+server.address().port;let context;
try{
 context=await chromium.launchPersistentContext(path.join(root,'profile-'+Date.now()),{headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-background-networking','--disable-extensions']});await context.route('**/*',route=>new URL(route.request().url()).origin===origin?route.continue():route.abort());
 const page=await context.newPage(),errors=[];page.on('pageerror',e=>errors.push(e.message));await page.addInitScript(()=>{window.engineRequests=[];const W=Worker;window.Worker=class extends W{postMessage(v,...rest){if(v.type==='run')engineRequests.push(v.request);return super.postMessage(v,...rest);}};});await page.goto(origin+'/');await page.locator('#open-lab-project').waitFor();
const cases=[];
for(const kind of ['economic','housing']){
 await page.locator('#open-lab-project').setInputFiles(fixtures[kind].file);await page.waitForFunction(k=>document.querySelector('#lab-project-status').textContent.startsWith('Opened project.')&&document.querySelector('#weights').value===k+'-character',kind);
 assert((await page.locator('#character-help').innerText()).includes('Native State uses alpha 0.5'));
 await page.locator('#character-alpha').evaluate(e=>{const d=e.closest('details');if(d)d.open=true;});
 for(const alpha of [0,.5,1]){
  await page.locator('#character-alpha').fill(String(alpha));const before=await page.evaluate(()=>engineRequests.length);await page.locator('#run').click();await page.waitForFunction(n=>engineRequests.length>n,before);await page.waitForFunction(()=>document.querySelector('#run-status').textContent==='completed');const request=await page.evaluate(()=>engineRequests.at(-1));assert.equal(request.options.character_alpha,alpha);
  const state_args=['--state','RI','--year','2020','--partition-mode','edge-weighted','--structure','standard-bisect','--weights-override',kind+'-character','--metis-engine','metis-core','--search','multi','--seeds','3','--niter','10','--balance-tolerance','10','--seed','18446744073709551615'];const file=path.join(root,kind+'-'+alpha+'-request.json');await fs.writeFile(file,JSON.stringify({graph:request.graph,state_args,districts:request.options.districts,...(alpha===.5?{}:{alpha_override:alpha}),...(kind==='economic'?{raw:request.graph.geoids.map(id=>request.character.data.raw_counts[id])}:{characters:request.graph.geoids.map(id=>request.character.data.characters[id])})}));const native=JSON.parse(execFileSync('target/release/examples/character_state_reference.exe',[file],{encoding:'utf8',maxBuffer:32*1024*1024}));
  const pending=page.waitForEvent('download');await page.locator('#save-lab-project').click();const savedFile=path.join(root,kind+'-'+alpha+'.bisect');await(await pending).saveAs(savedFile);const saved=JSON.parse(await fs.readFile(savedFile,'utf8'));assert.deepEqual(saved.assignments.RI,native.assignments);assert.equal(saved.experiment.config.character_alpha,alpha);assert.deepEqual(saved.experiment.config.characters.RI,fixtures[kind].input);assert.equal(saved.experiment.config.seed,config.seed);
  const previous=await page.evaluate(()=>engineRequests.length);await page.locator('#open-lab-project').setInputFiles(savedFile);await page.waitForFunction(v=>document.querySelector('#lab-project-status').textContent.startsWith('Opened project.')&&document.querySelector('#character-alpha').value===String(v),alpha);assert.equal(await page.evaluate(()=>engineRequests.length),previous);cases.push({kind,alpha,native_assignments_equal:true,cli_alpha:native.cli_alpha,inert_open:true});
 }
}
assert.deepEqual(errors,[]);await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({real_state:'RI',cases,declared_seed_preserved:true,open_executes_engine:false,observations:'Explicit synthetic character values'},null,2));console.log('Character browser: both inputs, native CLI default blend/native weighter endpoints, exact seed and inert Save/Open passed.');
}finally{await context?.close();await new Promise(r=>server.close(r));}
