import {openWorkbenchProject} from './workbench_project_open.mjs';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
const packages=process.env.BISECT_PLAYWRIGHT_PACKAGES;assert(packages);
const {chromium}=createRequire(path.join(packages,'package.json'))('playwright');
const root=path.resolve('target/ilp-audit-scope-retention-browser');await fs.mkdir(root,{recursive:true});
const source=path.resolve('web/lab'),wasm=path.resolve('target/wasm32-unknown-unknown/release/bisect_wasm.wasm');
const server=createServer(async(req,res)=>{try{const name=new URL(req.url,'http://localhost').pathname,file=name==='/bisect_wasm.wasm'?wasm:path.resolve(source,'.'+(name==='/'?'/workbench.html':name));if(file!==wasm&&!file.startsWith(source+path.sep))throw new Error('Path refused');res.setHeader('Content-Type',file.endsWith('.js')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':'text/html');res.end(await fs.readFile(file));}catch{res.writeHead(404);res.end();}});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));const origin='http://127.0.0.1:'+server.address().port;let context;
try{
 context=await chromium.launchPersistentContext(path.join(root,'profile-'+Date.now()),{headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-background-networking','--disable-extensions']});await context.route('**/*',route=>new URL(route.request().url()).origin===origin?route.continue():route.abort());
 const page=await context.newPage(),errors=[];page.on('pageerror',e=>errors.push(e.message));await page.addInitScript(()=>{window.runs=0;const W=Worker;window.Worker=class extends W{postMessage(v,...rest){if(v.type==='run')window.runs++;return super.postMessage(v,...rest);}};});
 await page.addInitScript(()=>{const read=File.prototype.arrayBuffer;File.prototype.arrayBuffer=function(){const file=this;return window.holdChainRead?new Promise(resolve=>{window.releaseChainRead=()=>resolve(read.call(file));}):read.call(file);};});
 await page.goto(origin+'/');await page.waitForFunction(()=>!document.querySelector('#execute').disabled);


 const fixture=path.resolve('target/ilp-audit-scope-retention/browser.bisect');await openWorkbenchProject(page,fixture,{ready:({argument:__bisectOpenArgument2402,openedStatus:__bisectOpenedStatus2402})=>(()=>__bisectOpenedStatus2402==='Opened browser.bisect')(__bisectOpenArgument2402),readyArg:undefined});assert.equal(await page.evaluate(()=>runs),0);
 if(await page.locator('#operation').inputValue()!=='audit-ilp-artifacts')await page.locator('#operation').selectOption('audit-ilp-artifacts');
 await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-ilp-audit').disabled);assert.equal(await page.evaluate(()=>runs),1);
 const pending=page.waitForEvent('download');await page.locator('#download-ilp-audit').click();const file=path.join(root,'audit-summary.json');await(await pending).saveAs(file);const downloaded=JSON.parse(await fs.readFile(file,'utf8'));assert.deepEqual(downloaded.report_selection,['node.json']);
 const saving=page.waitForEvent('download');await page.locator('#save-project').click();const projectFile=path.join(root,'checked.bisect');await(await saving).saveAs(projectFile);const p=JSON.parse(await fs.readFile(projectFile,'utf8'));assert.deepEqual(p.result.summary,downloaded);assert.equal(p.result.supplied_summary_status,'match');
 const n=await page.evaluate(()=>runs);await openWorkbenchProject(page,projectFile,{ready:({argument:__bisectOpenArgument3619,openedStatus:__bisectOpenedStatus3619})=>(()=>__bisectOpenedStatus3619==='Opened checked.bisect')(__bisectOpenArgument3619),readyArg:undefined});assert.equal(await page.evaluate(()=>runs),n);assert.deepEqual(errors,[]);
 await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({actual_worker:true,download_retains_verified_report_scope:true,download_matches_saved_displayed_records:true,inert_open:true},null,2));console.log('ILP audit scope browser: explicit Worker, scoped summary download, matching saved records and inert Open passed.');
}finally{await context?.close();await new Promise(r=>server.close(r));}
