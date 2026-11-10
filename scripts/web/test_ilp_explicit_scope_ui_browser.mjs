import {openWorkbenchProject} from './workbench_project_open.mjs';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
const packages=process.env.BISECT_PLAYWRIGHT_PACKAGES;assert(packages);
const {chromium}=createRequire(path.join(packages,'package.json'))('playwright');
const root=path.resolve('target/ilp-explicit-scope-ui-browser');await fs.mkdir(root,{recursive:true});
const source=path.resolve('web/lab'),wasm=path.resolve('target/wasm32-unknown-unknown/release/bisect_wasm.wasm');
const server=createServer(async(req,res)=>{try{const name=new URL(req.url,'http://localhost').pathname,file=name==='/bisect_wasm.wasm'?wasm:path.resolve(source,'.'+(name==='/'?'/workbench.html':name));if(file!==wasm&&!file.startsWith(source+path.sep))throw new Error('Path refused');res.setHeader('Content-Type',file.endsWith('.js')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':'text/html');res.end(await fs.readFile(file));}catch{res.writeHead(404);res.end();}});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));const origin='http://127.0.0.1:'+server.address().port;let context;
try{
 context=await chromium.launchPersistentContext(path.join(root,'profile-'+Date.now()),{headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-background-networking','--disable-extensions']});await context.route('**/*',route=>new URL(route.request().url()).origin===origin?route.continue():route.abort());
 const page=await context.newPage(),errors=[];page.on('pageerror',e=>errors.push(e.message));await page.addInitScript(()=>{window.runs=0;const W=Worker;window.Worker=class extends W{postMessage(v,...rest){if(v.type==='run')window.runs++;return super.postMessage(v,...rest);}};});
 await page.addInitScript(()=>{const read=File.prototype.arrayBuffer;File.prototype.arrayBuffer=function(){const file=this;return window.holdChainRead?new Promise(resolve=>{window.releaseChainRead=()=>resolve(read.call(file));}):read.call(file);};});
 await page.goto(origin+'/');await page.waitForFunction(()=>!document.querySelector('#execute').disabled);


 const fixture=path.resolve('target/ilp-explicit-scope-ui/browser.bisect');await openWorkbenchProject(page,fixture,{ready:({argument:__bisectOpenArgument2394,openedStatus:__bisectOpenedStatus2394})=>(()=>__bisectOpenedStatus2394==='Opened browser.bisect')(__bisectOpenArgument2394),readyArg:undefined});assert.equal(await page.evaluate(()=>runs),0);assert.equal(await page.locator('#ilp-audit-scope').inputValue(),'all-selected');
 const save=async name=>{const pending=page.waitForEvent('download');await page.locator('#save-project').click();const file=path.join(root,name);await(await pending).saveAs(file);return {file,project:JSON.parse(await fs.readFile(file,'utf8'))};};
 await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-ilp-audit').disabled);const all=await save('all.bisect');assert.equal(all.project.result.status,'failed');assert.equal(all.project.result.summary.checked,2);
 const n=await page.evaluate(()=>runs);await page.locator('#ilp-audit-scope').selectOption('summary-declared');assert.equal(await page.evaluate(()=>runs),n);assert((await page.locator('#ilp-audit-scope-status').textContent()).includes('node_old.json'));const prepared=await save('prepared.bisect');assert.equal(prepared.project.result,null);assert.equal(Object.keys(prepared.project.files.ilpAuditInputs.reports).length,2);
 await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-ilp-audit').disabled);const checked=await save('checked.bisect');assert.equal(checked.project.result.status,'verified');assert.equal(checked.project.result.summary.checked,1);assert.deepEqual(checked.project.result.summary.report_selection,['node.json']);assert.equal(Object.keys(checked.project.files.ilpAuditInputs.reports).length,2);
 const before=await page.evaluate(()=>runs);await openWorkbenchProject(page,checked.file,{ready:({argument:__bisectOpenArgument4120,openedStatus:__bisectOpenedStatus4120})=>(()=>__bisectOpenedStatus4120==='Opened checked.bisect')(__bisectOpenArgument4120),readyArg:undefined});assert.equal(await page.evaluate(()=>runs),before);assert.equal(await page.locator('#ilp-audit-scope').inputValue(),'summary-declared');await page.locator('#ilp-audit-remove-summary').click();assert.equal(await page.locator('#ilp-audit-scope').inputValue(),'all-selected');assert.deepEqual(errors,[]);
 await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({actual_worker_modes:true,default_keeps_all_reports:true,scope_change_runs_nothing:true,full_source_inventory_saved:true,declared_scope_restored_inertly:true,source_change_resets_scope:true},null,2));console.log('Explicit ILP scope browser: all-selected failure, explicit list preparation without execution, verified subset, full inventory Save/Open and reset on summary removal passed.');
}finally{await context?.close();await new Promise(r=>server.close(r));}
