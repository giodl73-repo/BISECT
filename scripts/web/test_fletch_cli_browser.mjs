import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {observedProjectOpen} from './fixtures/observed-project-open.mjs';
const option=n=>process.argv.find(a=>a.startsWith('--'+n+'='))?.slice(n.length+3);
const reference=option('reference');if(!reference)throw Error('Pass --reference=<passing actual CLI case root>.');
const evidence=JSON.parse(await fs.readFile(path.join(reference,'evidence.json'),'utf8'));assert.equal(evidence.status,'passed-actual-CLI-fletch-contract');
const root=await fs.mkdtemp(path.resolve('target/fletch-cli-contract/browser-')),sourceRoot=path.resolve('web/lab'),packageRoot=path.resolve(option('toolkit')||'dist/wasm-cffi-source-start-final/toolkit'),served=new Map();
const hash=b=>createHash('sha256').update(b).digest('hex'),wasm=await fs.readFile(path.join(reference,'engine.wasm'));assert.equal(hash(wasm),evidence.wasm.sha256);
const server=createServer(async(req,res)=>{try{
 const name=decodeURIComponent(new URL(req.url,'http://localhost').pathname),relative=name==='/'?'workbench.html':name.slice(1);let file=path.resolve(sourceRoot,relative),bytes;
 if(!file.startsWith(sourceRoot+path.sep))throw Error('Unsafe path');
 if(relative==='bisect_wasm.wasm')bytes=wasm;else{try{bytes=await fs.readFile(file);served.set(file,hash(bytes));}catch(error){if(error.code!=='ENOENT')throw error;file=path.resolve(packageRoot,relative);if(!file.startsWith(packageRoot+path.sep))throw error;bytes=await fs.readFile(file);}}
 res.setHeader('Content-Type',file.endsWith('.js')||file.endsWith('.mjs')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':file.endsWith('.json')?'application/json':'text/html');res.setHeader('Cache-Control','no-store');res.end(bytes);
 }catch{res.writeHead(404);res.end();}});await new Promise(r=>server.listen(0,'127.0.0.1',r));
const origin='http://127.0.0.1:'+server.address().port,{chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES,'package.json'))('playwright');let browser;
try{
 browser=await chromium.launch({headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe'});const context=await browser.newContext(),network=[];
 await context.route('**/*',route=>{const url=route.request().url();if(new URL(url).origin===origin)return route.continue();network.push(url);return route.abort();});
 const page=await context.newPage(),errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.addInitScript(()=>{window.runs=[];window.replies=[];const W=Worker;window.Worker=class extends W{constructor(...args){super(...args);this.addEventListener('message',({data})=>replies.push(data));}postMessage(data,...args){if(data.type==='run')runs.push(structuredClone(data));return super.postMessage(data,...args);}};});
 await page.goto(origin);await page.waitForFunction(()=>!document.querySelector('#execute').disabled);const outcomes=[];
 for(const row of evidence.records){
  const file=path.join(reference,row.name,'project.bisect'),before=await page.evaluate(()=>runs.length);await observedProjectOpen(page,file);
  await page.waitForFunction(name=>document.querySelector('#project-name').value===name&&!document.querySelector('#execute').disabled,row.name);assert.equal(await page.evaluate(()=>runs.length),before,'Open is inert');
  const kind=row.request.operation==='inspect-fletch-sources'?'source':'cache';assert(await page.locator('#download-fletch-'+kind).isDisabled(),'Imported result is unverified');
  await page.locator('#execute').click();await page.waitForFunction(n=>runs.length===n+1&&replies.some(r=>r.id===runs.at(-1).id&&['error','result'].includes(r.type)),before);await page.waitForFunction(()=>!document.querySelector('#execute').disabled);
  const reply=await page.evaluate(()=>replies.find(r=>r.id===runs.at(-1).id&&['result','error'].includes(r.type)));
  if(row.error){assert.equal(reply.type,'error');assert(await page.locator('#download-fletch-cache').isDisabled());if(row.name==='cache-bom')assert.equal(JSON.parse(reply.failure.request_json).input.cache_manifest_json.charCodeAt(0),0xfeff);}
  else{assert.equal(reply.type,'result',reply.error);assert.deepEqual(reply.result,row.result);const pending=page.waitForEvent('download');await page.locator('#download-fletch-'+kind).click();const out=path.join(root,row.name+(kind==='source'?'.csv':'.json'));await(await pending).saveAs(out);const actual=await fs.readFile(out),native=await fs.readFile(path.join(reference,row.name,kind==='source'?'report.csv':'report.json'));assert.deepEqual(actual,kind==='source'?Buffer.from(native.toString().replaceAll('\\','/')):native);}
  const pending=page.waitForEvent('download');await page.locator('#save-project').click();const saved=path.join(root,row.name+'.bisect');await(await pending).saveAs(saved);const project=JSON.parse(await fs.readFile(saved,'utf8'));if(kind==='cache')assert.equal(project.files.fletchCacheManifest.base64,JSON.parse(await fs.readFile(file,'utf8')).files.fletchCacheManifest.base64);assert.equal(await page.evaluate(()=>runs.length),before+1,'Save is inert');outcomes.push({name:row.name,native_exit:row.native_exit,browser:reply.type});
 }
 assert.deepEqual(errors,[]);assert.deepEqual(network,[]);const sources=[];for(const [file,sha256]of served){assert.equal(hash(await fs.readFile(file)),sha256);if(process.argv.includes('--require-package'))assert.equal(hash(await fs.readFile(path.join(packageRoot,path.basename(file)==='workbench.html'?'index.html':path.basename(file)))),sha256,'Final package served source identity');sources.push({path:path.relative(process.cwd(),file).replaceAll('\\','/'),sha256});}
 const result={status:'passed-actual-CLI-fletch-browser',sources,reference:{path:path.join(reference,'evidence.json'),sha256:hash(await fs.readFile(path.join(reference,'evidence.json')))},engine_sha256:hash(wasm),browser:browser.version(),outcomes,explicit_runs:await page.evaluate(()=>runs.length),test_sha256:hash(await fs.readFile('scripts/web/test_fletch_cli_browser.mjs')),scope:'Actual workbench/worker, original manifest bytes, native reports/gate outcomes and inert Open/Save. No provider acquisition, automatic disk discovery, selected-folder writes or full parent acceptance.'};await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(result,null,2)+'\n');console.log(JSON.stringify({status:result.status,root}));
}finally{await browser?.close();await new Promise(r=>server.close(r));}
