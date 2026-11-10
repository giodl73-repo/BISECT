import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {createProject} from '../../web/lab/project.js';
import {packageVerificationRequest} from '../../web/lab/package-verification-files.js';
import {assignmentDiscoveryRequest} from '../../web/lab/assignment-discovery-files.js';
import {multiSummaryRequest} from '../../web/lab/multi-summary-files.js';
import {observedProjectOpen} from './fixtures/observed-project-open.mjs';

const option=n=>process.argv.find(a=>a.startsWith('--'+n+'='))?.slice(n.length+3);
const reference=option('reference');if(!reference)throw Error('Pass --reference=<native comparison root>.');
const hash=b=>createHash('sha256').update(b).digest('hex');
const data=JSON.parse(await fs.readFile(path.join(reference,'evidence.json'),'utf8'));
assert.equal(data.status,'passed-manifest-reader-width');assert.equal(data.records.length,72);
const root=await fs.mkdtemp(path.resolve('target/manifest-reader-width/browser-'));
const staged=process.argv.includes('--staged-site');
if(staged&&!option('toolkit'))throw Error('Pass --toolkit=<staged site> with --staged-site.');
const packageRoot=path.resolve(option('toolkit')||'dist/wasm-manifest-reader-width/toolkit'),sourceRoot=staged?await fs.realpath(packageRoot):path.resolve('web/lab');
const manifestPath=path.join(packageRoot,'../offline-manifest.json');
const packageBefore=process.argv.includes('--require-package')?hash(await fs.readFile(manifestPath)):null;
const wasm=await fs.readFile(staged?path.join(sourceRoot,'bisect_wasm.wasm'):data.wasm.path);assert.equal(hash(wasm),data.wasm.sha256);
if(packageBefore)assert.equal(hash(await fs.readFile(path.join(packageRoot,'bisect_wasm.wasm'))),hash(wasm));
const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
function selected(row){
 const input=row.input, files=Object.fromEntries(Object.entries(input.files_base64||{}).map(([p,base64])=>[p,{name:p.split('/').at(-1),base64}]));
 if(row.operation==='summarize-plan-labels'){
  const v={labels:input.labels,plans:Object.fromEntries(Object.entries(input.plans).map(([label,files])=>[label,Object.fromEntries(Object.entries(files).map(([p,text])=>[p,source(p,text)]))]))};
  assert.deepEqual(multiSummaryRequest(v),{operation:row.operation,input});return {multiSummaryInputs:v};
 }
 const v={manifest:source(input.manifest_path||'manifest.json',input.manifest_json),files};
 if(row.operation==='discover-original-assignments'){
  v.explicitLabel=input.explicit_label||'';assert.deepEqual(assignmentDiscoveryRequest(v),{operation:row.operation,input});return {assignmentDiscoveryInputs:v};
 }
 assert.deepEqual(packageVerificationRequest(v,row.operation),{operation:row.operation,input});return {packageVerificationInputs:v};
}
const served=new Map();
const server=createServer(async(req,res)=>{try{
 const relative=decodeURIComponent(new URL(req.url,'http://localhost').pathname).slice(1)||(staged?'index.html':'workbench.html');
 let file=path.resolve(sourceRoot,relative);if(!file.startsWith(sourceRoot+path.sep))throw Error('Unsafe path');
 let bytes;if(relative==='bisect_wasm.wasm')bytes=wasm;else{try{bytes=await fs.readFile(file);served.set(file,hash(bytes));}catch(e){if(staged||e.code!=='ENOENT')throw e;file=path.resolve(packageRoot,relative);if(!file.startsWith(packageRoot+path.sep))throw e;bytes=await fs.readFile(file);}}
 res.setHeader('Content-Type',file.endsWith('.js')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':file.endsWith('.json')?'application/json':'text/html');res.setHeader('Cache-Control','no-store');res.end(bytes);
 }catch{res.writeHead(404);res.end();}});
await new Promise(r=>server.listen(0,'127.0.0.1',r));
const origin='http://127.0.0.1:'+server.address().port;
const {chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES,'package.json'))('playwright');let browser;
try{
 browser=await chromium.launch({headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe'});
 const context=await browser.newContext(),external=[];await context.route('**/*',r=>new URL(r.request().url()).origin===origin?r.continue():(external.push(r.request().url()),r.abort()));
 const page=await context.newPage(),errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.addInitScript(()=>{window.runs=[];window.replies=[];const W=Worker;window.Worker=class extends W{constructor(...a){super(...a);this.addEventListener('message',({data})=>replies.push(data));}postMessage(data,...a){if(data.type==='run')runs.push(structuredClone(data));return super.postMessage(data,...a);}};});
 await page.goto(origin);await page.waitForFunction(()=>!document.querySelector('#execute').disabled);
 const outcomes=[];
 async function download(id,name){const pending=page.waitForEvent('download');await page.locator(id).click();const file=path.join(root,name);await(await pending).saveAs(file);return fs.readFile(file,'utf8');}
 for(let index=0;index<data.records.length;index++){
  const row=data.records[index],files=selected(row),name=`reader-${index}`,file=path.join(root,name+'.bisect');
  await fs.writeFile(file,JSON.stringify(createProject({name,files,operation:row.operation,constraints:[],result:row.actual,lastOperation:row.operation})));
  const before=await page.evaluate(()=>runs.length);await observedProjectOpen(page,file);
  await page.waitForFunction(name=>document.querySelector('#project-name').value===name&&!document.querySelector('#execute').disabled,name);
  assert.equal(await page.evaluate(()=>runs.length),before,'Open must remain inert');
  const receipt=row.operation.startsWith('verify-manifest-')?'#download-package-verification':row.operation==='summarize-plan-labels'?'#download-multi-summary-table':null;
  if(receipt)assert(await page.locator(receipt).isDisabled(),'Imported results must remain inactive');
  await page.locator('#execute').click();
  await page.waitForFunction(n=>runs.length===n+1&&replies.some(r=>r.id===runs.at(-1).id&&['result','error'].includes(r.type)),before);
  await page.waitForFunction(()=>!document.querySelector('#execute').disabled);
  const run=await page.evaluate(()=>({request:runs.at(-1).request,reply:replies.find(r=>r.id===runs.at(-1).id&&['result','error'].includes(r.type))}));
  await fs.writeFile(path.join(root,'progress.json'),JSON.stringify({name:row.name,...run},null,2));
  assert.deepEqual(run.request,{operation:row.operation,input:row.input});assert.equal(run.reply.type,'result',run.reply.error);assert.deepEqual(run.reply.result,row.actual);
  if(row.operation.startsWith('verify-manifest-'))assert.deepEqual(JSON.parse(await download(receipt,name+'-receipt.json')),row.actual);
  if(row.operation==='summarize-plan-labels'){
   assert.equal(await download('#download-multi-summary-table',name+'-table.txt'),row.actual.table);
   assert.equal(await download('#download-multi-summary-csv',name+'-table.csv'),row.actual.csv);
  }
  const saved=JSON.parse(await download('#save-project',name+'-saved.bisect'));assert.deepEqual(saved.files,files);assert.deepEqual(saved.result,row.actual);
  assert.equal(await page.evaluate(()=>runs.length),before+1);
  outcomes.push({name:row.name,operation:row.operation,explicit_run:true,exact_request:true,exact_result:true,exact_saved_sources:true});
 }
 assert.deepEqual(errors,[]);assert.deepEqual(external,[]);
 const sources=[];for(const [file,sha256]of served){assert.equal(hash(await fs.readFile(file)),sha256);if(packageBefore)assert.equal(hash(await fs.readFile(path.join(packageRoot,path.basename(file)==='workbench.html'?'index.html':path.basename(file)))),sha256);sources.push({path:path.relative(process.cwd(),file).replaceAll('\\','/'),sha256});}
 if(packageBefore)assert.equal(hash(await fs.readFile(manifestPath)),packageBefore,'Package changed during browser acceptance');
 const evidence={status:'passed-manifest-reader-width-workbench',sources,reference:{path:path.join(reference,'evidence.json'),sha256:hash(await fs.readFile(path.join(reference,'evidence.json')))},engine_sha256:hash(wasm),outcomes,explicit_runs:await page.evaluate(()=>runs.length),package_manifest_sha256:packageBefore,test_sha256:hash(await fs.readFile('scripts/web/test_manifest_reader_width_browser.mjs')),browser:browser.version(),staged_site:staged?sourceRoot:null,scope:'Actual workbench Open/explicit Run/worker/download/Save over every native reference case. Exact raw source bytes retained. No OS picker grant, native full workflow, source authenticity or full parent acceptance claim.'};
 await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(evidence,null,2)+'\n');console.log(JSON.stringify({root,status:evidence.status,cases:outcomes.length}));
}finally{await browser?.close();await new Promise(r=>server.close(r));}
