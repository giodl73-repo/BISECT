import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const packages=process.env.BISECT_PLAYWRIGHT_PACKAGES;
if(!packages)throw new Error('Set BISECT_PLAYWRIGHT_PACKAGES');
const {chromium}=createRequire(path.join(packages,'package.json'))('playwright');
const root='target/annealing-defaults-browser',source=path.resolve('web/lab');await fs.mkdir(root,{recursive:true});
const wasm=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),engine=await instantiateEngine(wasm);
const server=createServer(async(req,res)=>{
 try{
  const name=decodeURIComponent(new URL(req.url,'http://localhost').pathname);
  if(name.includes('..'))throw new Error('Path rejected');const file=name==='/'?'/workbench.html':name;
  res.setHeader('Content-Type',file.endsWith('.js')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':'text/html');
  res.end(file==='/bisect_wasm.wasm'?wasm:await fs.readFile(source+file));
 }catch{res.writeHead(404);res.end();}
});
await new Promise(r=>server.listen(0,'127.0.0.1',r));const origin='http://127.0.0.1:'+server.address().port;let context;
try{
 context=await chromium.launchPersistentContext(path.resolve(root,'profile-'+Date.now()),{
  headless:true,executablePath:'C:/Program Files/Google/Chrome/Application/chrome.exe',
  args:['--no-first-run','--disable-background-networking','--disable-extensions']
 });
 await context.route('**/*',r=>new URL(r.request().url()).origin===origin?r.continue():r.abort());
 const page=await context.newPage(),errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.addInitScript(()=>{
  window.requests=[];const Original=Worker;
  window.Worker=class extends Original{
   postMessage(m,...rest){if(m?.type==='run')requests.push(m.request);return super.postMessage(m,...rest);}
  };
 });await page.goto(origin);
 const download=async(id,name)=>{
  const pending=page.waitForEvent('download');await page.locator(id).click();
  await(await pending).saveAs(root+'/'+name);return fs.readFile(root+'/'+name,'utf8');
 };
 const original=JSON.parse(await fs.readFile('target/annealing-state-oracle/default.bisect','utf8')),records=[];
 for(const variant of ['omitted','partial','default-build']){
  const project=structuredClone(original);project.name='Annealing '+variant;
  if(variant==='partial'){
   const request=JSON.parse(project.files.preparedEngineInputs.source.text);request.options.sa_steps_per_tract=0;
   project.files.preparedEngineInputs.source.text=JSON.stringify(request);project.result=null;
  }
  if(variant==='default-build'){
   const request=JSON.parse(project.files.preparedEngineInputs.source.text);
   project.files.preparedEngineInputs.source.text=JSON.stringify({operation:'run-engine-build',request});project.result=null;
  }
  const file=root+'/'+variant+'.bisect';await fs.writeFile(file,JSON.stringify(project));
  const inputs=project.files.preparedEngineInputs;
  const expected=engine.execute({operation:'run-prepared-engine',input:{request_json:inputs.source.text,...inputs.settings}});
  assert(expected.export,expected.export_error);
  const before=await page.evaluate(()=>requests.length);await page.locator('#open-project').setInputFiles(path.resolve(file));
  await page.waitForFunction(name=>document.querySelector('#project-name').value===name&&!document.querySelector('#execute').disabled,project.name);
  assert.equal(await page.evaluate(()=>requests.length),before);assert(await page.locator('#download-prepared-engine-plan').isDisabled());
  await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-prepared-engine-plan').disabled);
  const result=JSON.parse(await download('#download-prepared-engine-report',variant+'-report.json'));assert.deepEqual(result,expected);
  const rows=await page.locator('table').filter({has:page.locator('th',{hasText:'Setting',exact:true})}).locator('tbody tr').allTextContents();
  assert(rows.some(r=>r.includes('Steps per tract')&&r.includes(variant==='partial'?'0':'10')));
  assert(rows.some(r=>r.includes('Initial temperature factor')&&r.includes('0.01')));
  assert(rows.some(r=>r.includes('Final temperature')&&r.includes('0.0001')));
  assert.equal(await download('#download-prepared-engine-plan',variant+'.rplan'),expected.export.document_json);
  const saved=JSON.parse(await download('#save-project',variant+'-saved.bisect'));assert.deepEqual(saved.files.preparedEngineInputs,inputs);
  const stored=JSON.parse(saved.files.preparedEngineInputs.source.text),request=stored.request??stored;
  assert(!Object.hasOwn(request.options,'sa_t0_factor'));assert(!Object.hasOwn(request.options,'sa_t_final'));
  const count=await page.evaluate(()=>requests.length);await page.locator('#open-project').setInputFiles(path.resolve(root+'/'+variant+'-saved.bisect'));
  await page.waitForFunction(()=>document.querySelector('#download-prepared-engine-plan').disabled);assert.equal(await page.evaluate(()=>requests.length),count);
  records.push({variant,effective_settings:expected.result.metrics.structure_evidence,source_preserved:true,inert_reopen:true});
 }
 assert.deepEqual(errors,[]);
 await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm_sha256:engine.artifactIdentity.sha256,real_worker:true,records},null,2)+'\n');
 console.log('Annealing defaults browser: omitted/partial settings, visible effective defaults, exact report/plan bytes, unchanged source and inert Save/Open passed.');
}finally{await context?.close();await new Promise(r=>server.close(r));}
