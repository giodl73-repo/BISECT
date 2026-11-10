import {openWorkbenchProject} from './workbench_project_open.mjs';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
const packages=process.env.BISECT_PLAYWRIGHT_PACKAGES;assert(packages);
const {chromium}=createRequire(path.join(packages,'package.json'))('playwright');
const root=path.resolve('target/batch-accepted-replay-browser');await fs.mkdir(root,{recursive:true});
const source=path.resolve('web/lab'),wasm=path.resolve('target/wasm32-unknown-unknown/release/bisect_wasm.wasm');
const server=createServer(async(req,res)=>{try{const name=new URL(req.url,'http://localhost').pathname,file=name==='/bisect_wasm.wasm'?wasm:path.resolve(source,'.'+(name==='/'?'/workbench.html':name));if(file!==wasm&&!file.startsWith(source+path.sep))throw new Error('Path refused');res.setHeader('Content-Type',file.endsWith('.js')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':'text/html');res.end(await fs.readFile(file));}catch{res.writeHead(404);res.end();}});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));const origin='http://127.0.0.1:'+server.address().port;let context;
try{
 context=await chromium.launchPersistentContext(path.join(root,'profile-'+Date.now()),{headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-background-networking','--disable-extensions']});await context.route('**/*',route=>new URL(route.request().url()).origin===origin?route.continue():route.abort());
 const page=await context.newPage(),errors=[];page.on('pageerror',e=>errors.push(e.message));await page.addInitScript(()=>{window.runs=0;const W=Worker;window.Worker=class extends W{postMessage(v,...rest){if(v.type==='run')window.runs++;return super.postMessage(v,...rest);}};});
 await page.addInitScript(()=>{const read=File.prototype.arrayBuffer;File.prototype.arrayBuffer=function(){const file=this;return window.holdChainRead?new Promise(resolve=>{window.releaseChainRead=()=>resolve(read.call(file));}):read.call(file);};});
 await page.goto(origin+'/');await page.waitForFunction(()=>!document.querySelector('#execute').disabled);



 await openWorkbenchProject(page,path.resolve('target/batch-accepted-replay/browser.bisect'),{ready:({argument:__bisectOpenArgument2373,openedStatus:__bisectOpenedStatus2373})=>(()=>__bisectOpenedStatus2373==='Opened browser.bisect')(__bisectOpenArgument2373),readyArg:undefined});assert.equal(await page.evaluate(()=>runs),0);assert.equal(await page.locator('#label-build-retain_replay').isChecked(),true);
 const save=async name=>{const pending=page.waitForEvent('download');await page.locator('#save-project').click();const file=path.join(root,name);await(await pending).saveAs(file);return JSON.parse(await fs.readFile(file,'utf8'));};
 await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-label-build').disabled);const batch=await save('batch.bisect');assert(batch.result.states.every(s=>s.replay_run));
 const n=await page.evaluate(()=>runs);await page.locator('#label-build-next').selectOption('verify-tui-manifest-replay');await page.locator('#use-label-build-plan').click();assert.equal(await page.locator('#operation').inputValue(),'verify-tui-manifest');assert.equal(await page.locator('#tui-verify-mode').inputValue(),'replay');assert.equal(await page.evaluate(()=>runs),n);const staged=await save('staged.bisect');assert(staged.files.engineReplayInputs.run);assert.equal(staged.files.engineReplayInputs.run.name,'accepted_configured_run.json');const recipe=JSON.parse(Buffer.from(staged.files.engineReplayInputs.run.base64,'base64').toString('utf8'));assert.equal(recipe.kind,'configured');assert.equal(typeof recipe.input.yaml_text,'string');assert.equal(typeof recipe.attempt,'number');assert.equal(staged.files.engineReplayInputs.minSimilarity,1);assert.equal(await page.locator('#download-tui-verify').isDisabled(),true);
 await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-tui-verify').disabled);const checked=await save('checked.bisect');assert.equal(checked.result.status,'match');assert.equal(checked.result.execution_completed,true);assert.equal(checked.result.view.assignment_similarity,1);assert.equal(await page.evaluate(()=>runs),n+1);
 assert.deepEqual(errors,[]);await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({actual_worker:true,inert_open_and_replay_handoff:true,retained_accepted_request:true,explicit_manifest_bound_replay_matches:true,exact_similarity_one:true},null,2));console.log('Accepted batch replay browser: retained sources, inert Open/handoff and explicit manifest-bound native replay with exact assignment match passed.');
}finally{await context?.close();await new Promise(r=>server.close(r));}
