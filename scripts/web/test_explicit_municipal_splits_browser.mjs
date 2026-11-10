import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import fs from 'node:fs/promises';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import path from 'node:path';
import {createProject} from '../../web/lab/project.js';
import {openWorkbenchProject} from './workbench_project_open.mjs';

const arg=name=>{const i=process.argv.indexOf(name);assert(i>0&&process.argv[i+1]);return path.resolve(process.argv[i+1]);};
const site=await fs.realpath(arg('--package'));
const runtime=await fs.realpath(arg('--runtime-receipt'));
const receipts=path.dirname(runtime);
const read=async name=>JSON.parse(await fs.readFile(name,'utf8'));
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const receipt=await read(runtime);
assert.equal(receipt.status,'passed-explicit-municipal-cli-wasm-and-package');
const wasmBytes=await fs.readFile(path.join(site,'bisect_wasm.wasm'));
assert.equal(wasmBytes.length,receipt.wasm.bytes);assert.equal(sha(wasmBytes),receipt.wasm.sha256);
const uiSha=sha(await fs.readFile('web/lab/count-results.js'));
assert.equal(sha(await fs.readFile(path.join(site,'count-results.js'))),uiSha);
const runtimeInput={};
for(const name of ['complete','partial','zero']){
 const row=receipt.rows.find(row=>row.name===name);
 const pinned=async (name,expected)=>{
  const file=path.join(receipts,name),raw=await fs.readFile(file);
  assert.equal(path.resolve(expected.path),file);assert.equal(raw.length,expected.bytes);assert.equal(sha(raw),expected.sha256);
  return JSON.parse(raw.toString('utf8'));
 };
 const request=await pinned(`${name}-request.json`,row.request);
 const result=await pinned(`${name}-wasm.json`,row.wasm_result);
 assert.equal(sha(request.context_json),row.context.sha256);
 assert.equal(result.schema_version,'bisect-explicit-plan-splits-v2');
 assert.equal(result.report.municipal_identity_schema,'bisect-explicit-municipal-membership-v1');
 assert.equal(result.report_sha256,sha(result.report_json));
 assert.equal(result.context_source_sha256,row.context.sha256);
 runtimeInput[name]={request,result};
}
const packageRoot=process.env.BISECT_PLAYWRIGHT_PACKAGES;
assert(packageRoot,'Set BISECT_PLAYWRIGHT_PACKAGES to the installed Playwright package root');
const {chromium}=createRequire(path.join(packageRoot,'package.json'))('playwright');
const base=path.resolve('target/explicit-municipal-splits');await fs.mkdir(base,{recursive:true});
const out=await fs.mkdtemp(path.join(base,'browser-'));
const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
const complete=runtimeInput.complete;
const initial=createProject({name:'Custom municipal split evidence',operation:'analyze-plan-splits',constraints:[],
 files:{auditPlanText:source('custom.rplan',complete.request.document_json),auditContextText:source('complete.rctx',complete.request.context_json)},
 result:complete.result,lastOperation:'analyze-plan-splits'});
await fs.writeFile(path.join(out,'prior-result.bisect'),JSON.stringify(initial),{flag:'wx'});
const mime={'.js':'application/javascript','.mjs':'application/javascript','.wasm':'application/wasm','.css':'text/css','.json':'application/json','.html':'text/html'};
const server=createServer(async(req,res)=>{
 try{
  const url=new URL(req.url,'http://localhost');
  const file=await fs.realpath(path.resolve(site,'.'+decodeURIComponent(url.pathname==='/'?'/index.html':url.pathname)));
  assert(file.startsWith(site+path.sep));
  res.setHeader('Content-Type',mime[path.extname(file)]??'application/octet-stream');res.end(await fs.readFile(file));
 }catch{res.writeHead(404);res.end();}
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const origin=`http://127.0.0.1:${server.address().port}`;
let browser;
try{
 browser=await chromium.launchPersistentContext(path.join(out,'profile'),{headless:true,
  executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe',
  args:['--no-first-run','--disable-background-networking','--disable-extensions']});
 await browser.route('**/*',r=>new URL(r.request().url()).origin===origin?r.continue():r.abort());
 const page=await browser.newPage(),errors=[],checks=[];
 page.on('pageerror',error=>errors.push(error.message));
 await page.addInitScript(()=>{window.municipalRequests=[];const OriginalWorker=Worker;
  window.Worker=class extends OriginalWorker{postMessage(value,...rest){if(value.type==='run')municipalRequests.push(structuredClone(value.request));return super.postMessage(value,...rest);}};});
 const count=()=>page.evaluate(()=>municipalRequests.length);
 const project=path.join(out,'prior-result.bisect');
 await page.goto(origin);await page.waitForFunction(()=>!document.querySelector('#execute').disabled);
 await openWorkbenchProject(page,project,{ready:({openedStatus})=>openedStatus.startsWith('Opened prior-result.bisect')});
 assert.equal(await count(),0);assert(await page.locator('#download-splits').isDisabled());
 for(const name of ['complete','partial','zero']){
  const {request,result}=runtimeInput[name], before=await count();
  if(name!=='complete'){
   await page.locator('#auditContextText-file').setInputFiles({name:name+'.rctx',mimeType:'application/json',buffer:Buffer.from(request.context_json)});
   await page.waitForFunction(n=>document.querySelector('#auditContextText-name').textContent===n,name+'.rctx');
   assert.equal(await count(),before);assert(await page.locator('#download-splits').isDisabled());
  }
  await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-splits').disabled);
  assert.equal(await count(),before+1);
  const observed=await page.evaluate(()=>municipalRequests.at(-1));
  assert.equal(observed.operation,'analyze-plan-splits');assert.equal(observed.context_json,request.context_json);
  const rows=Object.fromEntries(await page.locator('#result-tables table').first().locator('tbody tr').evaluateAll(items=>items.map(row=>Array.from(row.cells,cell=>cell.textContent))));
  assert.equal(rows['Municipal identity source'],'Matching RCTX');
  assert.equal(rows['Units with unknown municipal IDs'],String(result.report.municipalities.missing_units));
  assert.equal(rows['Split municipalities'],name==='partial'?'—':String(result.report.municipalities.split));
  assert.equal(rows['Split counties'],String(result.report.counties.split));
  if(name==='partial')assert.equal(rows['Municipality data'],'Incomplete (counts unavailable)');
  const waiting=page.waitForEvent('download');await page.locator('#download-splits').click();
  const file=path.join(out,`${name}-splits.json`);await (await waiting).saveAs(file);
  const bytes=await fs.readFile(file);assert.equal(bytes.toString('utf8'),result.report_json);
  checks.push({name,source_sha256:result.context_source_sha256,report_sha256:sha(bytes)});
 }
 const waitSave=page.waitForEvent('download');await page.locator('#save-project').click();
 const saved=path.join(out,'verified.bisect');await (await waitSave).saveAs(saved);
 const serialized=await read(saved);
 assert.equal(serialized.result.report_sha256,runtimeInput.zero.result.report_sha256);
 const before=await count();
 await openWorkbenchProject(page,saved,{ready:({openedStatus})=>openedStatus.startsWith('Opened verified.bisect')});
 assert.equal(await count(),before);assert(await page.locator('#download-splits').isDisabled());
 assert.deepEqual(errors,[]);
 await fs.writeFile(path.join(out,'evidence.json'),JSON.stringify({status:'passed-explicit-municipal-browser',checks,wasm_sha256:sha(wasmBytes),ui_sha256:uiSha,runtime_receipt_sha256:sha(await fs.readFile(runtime)),inert_open_and_edit:true,full_parity:false},null,2),{flag:'wx'});
 console.log(JSON.stringify({root:out,receipt:path.join(out,'evidence.json'),cases:checks.length}));
}catch(error){await fs.writeFile(path.join(out,'failure.txt'),error.stack,{flag:'wx'});throw error;}
finally{await browser?.close();await new Promise(resolve=>server.close(resolve));}
