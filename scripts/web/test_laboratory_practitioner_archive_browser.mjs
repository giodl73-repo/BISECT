import assert from 'node:assert/strict';

import fs from 'node:fs/promises';

import path from 'node:path';

import {createServer} from 'node:http';

import {createRequire} from 'node:module';

import {createHash} from 'node:crypto';

import {parseWorkbenchArchiveBytes} from '../../web/lab/workbench-archive.js';



const option=n=>process.argv.find(a=>a.startsWith('--'+n+'='))?.slice(n.length+3);

const reference=option('reference'),laboratory=path.resolve(option('laboratory')||'dist/wasm-parity-standard-20261028'),wasmPath=option('wasm');

if(!reference||!wasmPath)throw Error('Pass --reference and --wasm.');

const hash=b=>createHash('sha256').update(b).digest('hex');

const data=JSON.parse(await fs.readFile(path.join(reference,'evidence.json'),'utf8'));

assert.equal(data.status,'passed-large-laboratory-practitioner-archive');

const referenceBytes=await fs.readFile(data.archive.path);assert.equal(hash(referenceBytes),data.archive.sha256);

const expected=await parseWorkbenchArchiveBytes(referenceBytes);

await fs.mkdir('target/laboratory-practitioner-archive',{recursive:true});
const root=await fs.mkdtemp(path.resolve('target/laboratory-practitioner-archive/browser-'));

const source=path.resolve('web/lab'),fallback=path.resolve(option('toolkit')||'web/lab');

const toolkitWasm=await fs.readFile(wasmPath),served=new Map();
const packagePin=process.argv.includes('--require-package')?hash(await fs.readFile(path.join(laboratory,'offline-manifest.json'))):null;
if(packagePin)assert.equal(hash(await fs.readFile(path.join(laboratory,'bisect_wasm.wasm'))),hash(toolkitWasm));

const server=createServer(async(req,res)=>{try{

 const url=new URL(req.url,'http://localhost'),lab=url.pathname.startsWith('/laboratory/');

 let relative=url.pathname.slice(lab?'/laboratory/'.length:'/toolkit/'.length);

 if(!lab&&!url.pathname.startsWith('/toolkit/'))throw Error('Unknown route');

 if(!relative)relative=lab?'index.html':'workbench.html';

 let bytes,file;

 if(!lab&&relative==='bisect_wasm.wasm'){bytes=toolkitWasm;file=path.resolve(wasmPath);}

 else{

  const candidates=lab?(relative==='index.html'||relative==='catalog.json'||relative==='bisect_wasm.wasm'||relative.startsWith('assets/')?[laboratory]:[source,laboratory,fallback]):[source,fallback];

  for(const directory of candidates){const candidate=path.resolve(directory,relative);if(!candidate.startsWith(directory+path.sep))throw Error('Unsafe path');try{bytes=await fs.readFile(candidate);file=candidate;break;}catch(e){if(e.code!=='ENOENT')throw e;}}

  if(!bytes)throw Error('Missing asset');

 }

 served.set(file,hash(bytes));res.setHeader('Cache-Control','no-store');res.setHeader('Content-Type',file.endsWith('.js')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':file.endsWith('.json')?'application/json':'text/html');res.end(bytes);

 }catch{res.writeHead(404);res.end();}});

await new Promise(r=>server.listen(0,'127.0.0.1',r));const origin='http://127.0.0.1:'+server.address().port;

const {chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES,'package.json'))('playwright');let browser;

try{

 browser=await chromium.launch({headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe'});

 const context=await browser.newContext(),external=[];await context.route('**/*',r=>new URL(r.request().url()).origin===origin?r.continue():(external.push(r.request().url()),r.abort()));

 const page=await context.newPage(),errors=[];page.on('pageerror',e=>errors.push(e.message));

 await page.addInitScript(()=>{window.runs=[];window.replies=[];const W=Worker;window.Worker=class extends W{constructor(...args){super(...args);this.addEventListener('message',({data})=>replies.push(data));}postMessage(v,...args){if(v.type==='run')runs.push(structuredClone(v));return super.postMessage(v,...args);}};});

 await page.goto(origin+'/laboratory/');await page.locator('#open-lab-project').setInputFiles(path.join(reference,'CA-laboratory.bisect-lab'));

 await page.waitForFunction(()=>document.querySelector('#lab-project-status').textContent.startsWith('Opened project.'),null,{timeout:120000});

 assert.equal(await page.evaluate(()=>runs.length),0,'Opening retained laboratory results must not partition');

 await page.locator('#practitioner-tools summary').click();await page.locator('#export-audit-project').click();

 await page.waitForFunction(()=>!document.querySelector('#download-audit-project').disabled,null,{timeout:120000});

 assert(await page.locator('#open-audit-project').isDisabled(),'Large archive must not enter ordinary project transfer');

 assert.match(await page.locator('#download-audit-project').textContent(),/archive/i);

 const labRuns=await page.evaluate(()=>runs.map(r=>r.request.operation));assert.deepEqual(labRuns,['export-engine-plan']);

 const pending=page.waitForEvent('download');await page.locator('#download-audit-project').click();const archive=path.join(root,'downloaded.bisect-workbench');await(await pending).saveAs(archive);

 const downloaded=await parseWorkbenchArchiveBytes(await fs.readFile(archive));assert.deepEqual(downloaded.files,expected.files);

 await page.goto(origin+'/toolkit/');await page.waitForFunction(()=>!document.querySelector('#execute').disabled);

 await page.locator('#open-workbench-archive').setInputFiles(archive);

 await page.waitForFunction(()=>document.querySelector('#project-status').textContent.startsWith('Opened workbench archive'),null,{timeout:120000});

 assert.equal(await page.evaluate(()=>runs.length),0,'Archive Open must remain inert');

 assert.equal(await page.locator('#operation').inputValue(),'validate-rplan-text');

 await page.locator('#execute').click();await page.waitForFunction(()=>runs.length===1&&replies.some(r=>r.id===runs[0].id&&['result','error'].includes(r.type)),null,{timeout:120000});

 await page.waitForFunction(()=>!document.querySelector('#execute').disabled,null,{timeout:120000});

 const run=await page.evaluate(()=>({request:runs[0].request,reply:replies.find(r=>r.id===runs[0].id&&['result','error'].includes(r.type))}));

 await fs.writeFile(path.join(root,'run.json'),JSON.stringify(run));assert.equal(run.reply.type,'result',run.reply.error);

 assert.equal(run.request.document_json,Buffer.from(expected.files.auditPlanText.base64,'base64').toString('utf8'));

 assert.equal(run.reply.result.units,9129);assert.equal(run.reply.result.districts,52);

 await page.waitForFunction(()=>document.querySelectorAll('#plan-map svg path').length===9129,null,{timeout:120000});

 assert.match(await page.locator('#plan-map').textContent(),/9129 matched units.*52 districts/);

 await page.locator('#plan-map').screenshot({path:path.join(root,'CA-map.png')});

 const save=page.waitForEvent('download');await page.locator('#save-workbench-archive').click();const saved=path.join(root,'saved.bisect-workbench');await(await save).saveAs(saved);

 const reopened=await parseWorkbenchArchiveBytes(await fs.readFile(saved));assert.deepEqual(reopened.files,expected.files);assert.deepEqual(reopened.result,run.reply.result);

 await page.locator('#open-workbench-archive').setInputFiles(saved);await page.waitForFunction(()=>document.querySelector('#project-status').textContent.startsWith('Opened workbench archive'),null,{timeout:120000});

 assert.equal(await page.evaluate(()=>runs.length),1,'Reopening the validated result must not execute again');

 assert.match(await page.locator('#result-status').textContent(),/unverified/i);assert.deepEqual(errors,[]);assert.deepEqual(external,[]);

 const sources=[];for(const [file,sha256]of served){assert.equal(hash(await fs.readFile(file)),sha256);sources.push({path:path.relative(process.cwd(),file).replaceAll('\\','/'),sha256});}

 if(packagePin){
  assert.equal(hash(await fs.readFile(path.join(laboratory,'offline-manifest.json'))),packagePin,'Package changed during browser check');
  for(const [file,sha256]of served)if(file.startsWith(source+path.sep)){
   const name=path.basename(file),packaged=path.join(laboratory,'toolkit',name==='workbench.html'?'index.html':name);
   assert.equal(hash(await fs.readFile(packaged)),sha256,'Packaged toolkit source differs: '+name);
   if(name!=='workbench.html')assert.equal(hash(await fs.readFile(path.join(laboratory,name))),sha256,'Packaged laboratory source differs: '+name);
  }
 }
 const evidence={status:'passed-large-laboratory-practitioner-archive-browser',sources,reference:{path:path.join(reference,'evidence.json'),sha256:hash(await fs.readFile(path.join(reference,'evidence.json')))},toolkit_wasm:{path:wasmPath,sha256:hash(toolkitWasm)},package_manifest_sha256:packagePin,archive_bytes:(await fs.stat(archive)).size,lab_runs:labRuns,workbench_runs:1,source_bytes_retained:true,map_units:9129,map_districts:52,ordinary_transfer_refused:true,inert_open_and_reopen:true,units:9129,districts:52,test_sha256:hash(await fs.readFile('scripts/web/test_laboratory_practitioner_archive_browser.mjs')),scope:'Actual laboratory retained-result Open, explicit archive preparation/download, workbench archive Open/explicit typed validation/Save/reopen. Laboratory engine matches the supplied package; toolkit engine is explicitly pinned separately. No full-builder/package/deployment/OS grant acceptance.'};

 await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(evidence,null,2)+'\n');console.log(JSON.stringify({root,status:evidence.status,archive_bytes:evidence.archive_bytes}));

}finally{await browser?.close();await new Promise(r=>server.close(r));}

