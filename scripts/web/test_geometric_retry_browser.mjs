import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {createProject} from '../../web/lab/project.js';
const packages=process.env.BISECT_PLAYWRIGHT_PACKAGES;
if(!packages)throw new Error('Set BISECT_PLAYWRIGHT_PACKAGES');
const {chromium}=createRequire(path.join(packages,'package.json'))('playwright');
const root='target/geometric-retry-browser',source=path.resolve('web/lab');
await fs.mkdir(root,{recursive:true});
const server=createServer(async(req,res)=>{
 try{
  const name=decodeURIComponent(new URL(req.url,'http://localhost').pathname);
  if(name.includes('..'))throw new Error('Path rejected');
  const file=name==='/'?'/workbench.html':name;
  res.setHeader('Content-Type',file.endsWith('.js')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':'text/html');
  res.end(await fs.readFile(file==='/bisect_wasm.wasm'?'target/wasm32-unknown-unknown/release/bisect_wasm.wasm':source+file));
 }catch{res.writeHead(404);res.end();}
});
await new Promise(r=>server.listen(0,'127.0.0.1',r));
const origin='http://127.0.0.1:'+server.address().port;let context;
try{
 context=await chromium.launchPersistentContext(path.resolve(root,'profile-'+Date.now()),{
  headless:true,executablePath:'C:/Program Files/Google/Chrome/Application/chrome.exe',
  args:['--no-first-run','--disable-background-networking','--disable-extensions']
 });
 await context.route('**/*',r=>new URL(r.request().url()).origin===origin?r.continue():r.abort());
 const page=await context.newPage(),errors=[];
 page.on('pageerror',e=>errors.push(e.message));
 await page.addInitScript(()=>{
  window.requests=[];const Original=Worker;
  window.Worker=class extends Original{
   postMessage(m,...rest){if(m?.type==='run')requests.push(m.request);return super.postMessage(m,...rest);}
  };
 });
 await page.goto(origin);
 const download=async(id,file)=>{
  const pending=page.waitForEvent('download');await page.locator(id).click();
  await(await pending).saveAs(root+'/'+file);return fs.readFile(root+'/'+file,'utf8');
 };
 const evidence=JSON.parse(await fs.readFile('target/geometric-state-retries/evidence.json','utf8'));
 const records=[];
 for(const fixture of ['later-passing-grid','prepared-real-state','dominant-unit']){
  const index=evidence.records.findIndex(r=>r.fixture===fixture&&(fixture!=='prepared-real-state'||r.state==='IA'&&r.structure==='moving-knife'));
  assert(index>=0);const caseRoot=`target/geometric-state-retries/case-${index}`;
  const text=await fs.readFile(caseRoot+'/request.json','utf8');
  const settings={label:'Geometric native retry',chamber:'congressional',created_at:'2026-10-18T00:00:00Z',inspect_only:false};
  const inputs={source:{name:'request.json',text},settings};
  const project=createProject({name:'Retry '+fixture,operation:'run-prepared-engine',lastOperation:'run-prepared-engine',constraints:[],files:{preparedEngineInputs:inputs}});
  const file=root+'/'+fixture+'.bisect';await fs.writeFile(file,JSON.stringify(project));
  const before=await page.evaluate(()=>requests.length);
  await page.locator('#open-project').setInputFiles(path.resolve(file));
  await page.waitForFunction(name=>document.querySelector('#project-name').value===name&&!document.querySelector('#execute').disabled,project.name);
  assert.equal(await page.evaluate(()=>requests.length),before);
  assert(await page.locator('#download-prepared-engine-plan').isDisabled());
  await page.locator('#execute').click();
  if(fixture==='dominant-unit'){
   await page.waitForFunction(()=>document.querySelector('#result-status').textContent.includes('exhausted after 51 attempts'));
   assert(await page.locator('#download-prepared-engine-plan').isDisabled());
  }else{
   await page.waitForFunction(()=>!document.querySelector('#download-prepared-engine-plan').disabled);
   const expected=JSON.parse(await fs.readFile(caseRoot+'/prepared.json','utf8'));
   const actual=JSON.parse(await download('#download-prepared-engine-report',fixture+'-report.json'));
   assert.deepEqual(actual,expected);
   assert.equal(await download('#download-prepared-engine-plan',fixture+'.rplan'),expected.export.document_json);
   const table=page.locator('table').filter({has:page.locator('th', {hasText:'Population balanced'})});
   const rows=await table.locator('tbody tr').allTextContents();
   assert.equal(rows.length,expected.result.build_report.attempts.length);
   for(const [i,attempt]of expected.result.build_report.attempts.entries())assert(rows[i].includes(attempt.seed??'Omitted'));
   if(fixture==='later-passing-grid'){
    assert.equal(rows.length,7);assert(rows[0].includes('false'));assert(rows[6].includes('true'));
    assert((await page.locator('#result-tables').innerText()).includes('Accepted attempt'));
   }
  }
  const saved=JSON.parse(await download('#save-project',fixture+'-saved.bisect'));
  assert.deepEqual(saved.files.preparedEngineInputs,inputs);
  const runs=await page.evaluate(()=>requests.length);
  await page.locator('#open-project').setInputFiles(path.resolve(root+'/'+fixture+'-saved.bisect'));
  await page.waitForFunction(()=>document.querySelector('#download-prepared-engine-plan').disabled);
  assert.equal(await page.evaluate(()=>requests.length),runs);
  records.push({fixture,accepted_attempt:evidence.records[index].accepted_attempt,source_preserved:true,inert_reopen:true});
 }
 assert.deepEqual(errors,[]);
 const wasm=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm');
 await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm_sha256:createHash('sha256').update(wasm).digest('hex'),real_worker:true,exact_native_verified_outputs:true,records},null,2)+'\n');
 console.log('Geometric retry browser: later accepted CVD ledger, native Iowa moving-knife export, exhaustion refusal and source-preserving inert Save/Open passed.');
}finally{await context?.close();await new Promise(r=>server.close(r));}
