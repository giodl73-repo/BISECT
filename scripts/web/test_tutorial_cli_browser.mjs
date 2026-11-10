import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {observedProjectOpen} from './fixtures/observed-project-open.mjs';
const option=n=>process.argv.find(a=>a.startsWith('--'+n+'='))?.slice(n.length+3),reference=option('reference');if(!reference)throw Error('Pass --reference=<actual tutorial CLI cases>.');
const data=JSON.parse(await fs.readFile(path.join(reference,'evidence.json'),'utf8'));assert.equal(data.status,'passed-actual-tutorial-cli-contract');
const root=await fs.mkdtemp(path.resolve('target/tutorial-cli-contract/browser-')),sourceRoot=path.resolve('web/lab'),packageRoot=path.resolve(option('toolkit')||'dist/wasm-manifest-cli-complete/toolkit'),served=new Map(),hash=b=>createHash('sha256').update(b).digest('hex');const wasm=await fs.readFile(data.wasm.path);assert.equal(hash(wasm),data.wasm.sha256);
const server=createServer(async(req,res)=>{try{const u=new URL(req.url,'http://localhost');if(u.pathname==='/__tutorial_fault_worker.js'){const original=u.searchParams.get('original');if(!original||new URL(original).origin!==origin)throw Error('Invalid controlled worker URL');res.setHeader('Content-Type','application/javascript');res.end('import '+JSON.stringify(original)+';const slice=File.prototype.slice;File.prototype.slice=function(...args){if(this.name==="input.bin"&&this.size===13)return {arrayBuffer:async()=>{throw Error("Controlled selected-file read refusal")}};return slice.apply(this,args)};');return;}const relative=decodeURIComponent(new URL(req.url,'http://localhost').pathname).slice(1)||'workbench.html';let file=path.resolve(sourceRoot,relative),bytes;if(!file.startsWith(sourceRoot+path.sep))throw Error('Unsafe path');if(relative==='bisect_wasm.wasm')bytes=wasm;else{try{bytes=await fs.readFile(file);served.set(file,hash(bytes));}catch(e){if(e.code!=='ENOENT')throw e;file=path.resolve(packageRoot,relative);if(!file.startsWith(packageRoot+path.sep))throw e;bytes=await fs.readFile(file);}}res.setHeader('Content-Type',file.endsWith('.js')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':file.endsWith('.json')?'application/json':'text/html');res.setHeader('Cache-Control','no-store');res.end(bytes);}catch{res.writeHead(404);res.end();}});await new Promise(r=>server.listen(0,'127.0.0.1',r));
const origin='http://127.0.0.1:'+server.address().port,{chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES,'package.json'))('playwright');let browser;
let packageIdentity=null;
if(process.argv.includes('--require-package')){
 const html=await fs.readFile(path.join(sourceRoot,'workbench.html'));
 for(const name of ['index.html','workbench.html'])assert.equal(hash(await fs.readFile(path.join(packageRoot,name))),hash(html),'Workbench entry alias: '+name);
 assert.equal(hash(await fs.readFile(path.join(packageRoot,'../workbench.html'))),hash(html));
 assert.match(await fs.readFile(path.join(packageRoot,'../index.html'),'utf8'),/<title>BISECT Laboratory<\/title>/,'Preserve the laboratory landing page');
 const manifest=await fs.readFile(path.join(packageRoot,'../offline-manifest.json'));packageIdentity={offline_manifest_sha256:hash(manifest),version:JSON.parse(manifest).version,workbench_aliases_match:true,laboratory_landing_preserved:true};
}
try{
 browser=await chromium.launch({headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe'});const context=await browser.newContext(),external=[];await context.route('**/*',r=>{if(new URL(r.request().url()).origin===origin)return r.continue();external.push(r.request().url());return r.abort();});const page=await context.newPage(),errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.addInitScript(()=>{
  window.runs=[];window.replies=[];window.delayedReads={};
  const W=Worker;window.Worker=class extends W{
   constructor(url,options){
    const original=new URL(url,location.href).href;
    super('/__tutorial_fault_worker.js?original='+encodeURIComponent(original),{...options,type:'module'});this.addEventListener('message',({data})=>replies.push(data));
   }
   postMessage(data,...args){if(data.type==='run')runs.push(structuredClone(data));return super.postMessage(data,...args);}
  };
  const read=File.prototype.arrayBuffer;File.prototype.arrayBuffer=function(){if(this.name.startsWith('delayed-'))return new Promise((resolve,reject)=>{delayedReads[this.name]={release:()=>read.call(this).then(resolve,reject)};});return read.call(this);};
 });
 await page.goto(origin);await page.waitForFunction(()=>!document.querySelector('#execute').disabled);const outcomes=[];
 const download=async(id,name)=>{const pending=page.waitForEvent('download');await page.locator('#'+id).click();const file=path.join(root,name);await(await pending).saveAs(file);return fs.readFile(file,'utf8');};const save=async name=>JSON.parse(await download('save-project',name+'.bisect'));
 for(const row of data.records){
  const before=await page.evaluate(()=>runs.length);await observedProjectOpen(page,path.join(reference,row.name+'.bisect'));assert.equal(await page.evaluate(()=>runs.length),before);assert(await page.locator('#download-tutorial-console').isDisabled());assert(await page.locator('#download-tutorial').isDisabled());
  await page.locator('#tutorial-checksums').setInputFiles(path.join(reference,row.name,'examples/synthetic-walkthrough/checksums.json'));await page.waitForFunction(()=>!document.querySelector('#tutorial-checksums').disabled);
  const project=await save(row.name+'-source');assert.deepEqual(project.files.tutorialInputs,row.inputs);assert.equal(await page.evaluate(()=>runs.length),before);
  if(row.name==='pass')await page.locator('#tutorial-folder').setInputFiles(path.join(reference,'pass'));
  else if(row.selected_files.length)await page.locator('#tutorial-files').setInputFiles(row.selected_files.map(s=>({name:s.path,mimeType:'application/octet-stream',buffer:s.kind==='controlled-read-error'?Buffer.alloc(13):Buffer.from(s.base64,'base64')})));
  await page.locator('#execute').click();await page.waitForFunction(n=>runs.length===n+1&&replies.some(r=>r.id===runs.at(-1).id&&['result','error'].includes(r.type)),before);await page.waitForFunction(()=>!document.querySelector('#execute').disabled);const run=await page.evaluate(()=>({request:runs.at(-1).request,reply:replies.find(r=>r.id===runs.at(-1).id&&['result','error'].includes(r.type))}));
  assert.equal(run.request.operation,'verify-tutorial-files');assert.equal(run.request.checksums_json,Buffer.from(row.source.base64,'base64').toString('utf8'));assert.equal(run.request.tutorial,row.inputs.tutorial);assert.deepEqual(run.request.files.map(f=>f.path).sort(),row.selected_files.map(f=>f.path).sort());assert.equal(run.reply.type,'result',run.reply.error);assert.deepEqual(run.reply.result,row.result);
  assert.equal(await download('download-tutorial-console',row.name+'.txt'),row.console_text);assert.deepEqual(JSON.parse(await download('download-tutorial',row.name+'-result.json')),row.result);outcomes.push({name:row.name,native_exit_code:row.result.native_exit_code});
 }
 const before=await page.evaluate(()=>runs.length);for(const name of ['delayed-A.json','delayed-B.json'])await fs.writeFile(path.join(root,name),'{}');
 await page.locator('#tutorial-checksums').setInputFiles(path.join(root,'delayed-A.json'));await page.waitForFunction(()=>delayedReads['delayed-A.json']);assert(await page.locator('#execute').isDisabled());await page.locator('#tutorial-cancel').click();
 await page.locator('#tutorial-checksums').setInputFiles(path.join(root,'delayed-B.json'));await page.waitForFunction(()=>delayedReads['delayed-B.json']);await page.evaluate(async()=>{await document.querySelector('#tutorial-checksums').onchange({target:{files:[new File(['{}'],'reentrant.json')]}});document.querySelector('#execute').click();});assert.equal(await page.evaluate(()=>runs.length),before);
 await page.evaluate(()=>delayedReads['delayed-A.json'].release());assert(await page.locator('#tutorial-checksums').isDisabled());assert(await page.locator('#execute').isDisabled());
 await observedProjectOpen(page,path.join(reference,'bom.bisect'));await page.evaluate(()=>delayedReads['delayed-B.json'].release());await page.waitForFunction(()=>!document.querySelector('#tutorial-checksums').disabled);assert.deepEqual((await save('after-late')).files.tutorialInputs,data.records.find(r=>r.name==='bom').inputs);assert.equal(await page.evaluate(()=>runs.length),before);assert(await page.locator('#download-tutorial-console').isDisabled());
 assert.deepEqual(external,[]);assert.deepEqual(errors,[]);const sources=[];for(const [file,sha256]of served){assert.equal(hash(await fs.readFile(file)),sha256);if(process.argv.includes('--require-package'))assert.equal(hash(await fs.readFile(path.join(packageRoot,path.basename(file)==='workbench.html'?'index.html':path.basename(file)))),sha256);sources.push({path:path.relative(process.cwd(),file).replaceAll('\\','/'),sha256});}
 const evidence={status:'passed-tutorial-cli-workbench',sources,reference:{path:path.join(reference,'evidence.json'),sha256:hash(await fs.readFile(path.join(reference,'evidence.json')))},engine_sha256:hash(wasm),package_identity:packageIdentity,outcomes,explicit_runs:await page.evaluate(()=>runs.length),actual_file_and_folder_inputs:true,controlled_file_read_error:true,controlled_cancel_reentrant_late_open:true,actual_os_picker_grant:false,test_sha256:hash(await fs.readFile('scripts/web/test_tutorial_cli_browser.mjs')),scope:'Real file/folder selection and worker-streamed hashing, complete report downloads and inert projects. Worker shim injects one read-error class; main-thread delayed reads test callback ownership. Not actual OS read-error/permission acceptance, physical IO abortion, authenticity or full parent.'};await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(evidence,null,2)+'\n');console.log(JSON.stringify({root,status:evidence.status,cases:outcomes.length}));
}finally{await browser?.close();await new Promise(r=>server.close(r));}
