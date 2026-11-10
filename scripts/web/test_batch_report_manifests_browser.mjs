import {openWorkbenchProject} from './workbench_project_open.mjs';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
const packages=process.env.BISECT_PLAYWRIGHT_PACKAGES;assert(packages);
const {chromium}=createRequire(path.join(packages,'package.json'))('playwright');
const root=path.resolve('target/batch-report-manifest-browser');await fs.mkdir(root,{recursive:true});
const source=path.resolve('web/lab'),wasm=path.resolve('target/wasm32-unknown-unknown/release/bisect_wasm.wasm');
const server=createServer(async(req,res)=>{try{const name=new URL(req.url,'http://localhost').pathname,file=name==='/bisect_wasm.wasm'?wasm:path.resolve(source,'.'+(name==='/'?'/workbench.html':name));if(file!==wasm&&!file.startsWith(source+path.sep))throw new Error('Path refused');res.setHeader('Content-Type',file.endsWith('.js')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':'text/html');res.end(await fs.readFile(file));}catch{res.writeHead(404);res.end();}});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));const origin='http://127.0.0.1:'+server.address().port;let context;
try{
 context=await chromium.launchPersistentContext(path.join(root,'profile-'+Date.now()),{headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-background-networking','--disable-extensions']});await context.route('**/*',route=>new URL(route.request().url()).origin===origin?route.continue():route.abort());
 const page=await context.newPage(),errors=[];page.on('pageerror',e=>errors.push(e.message));await page.addInitScript(()=>{window.runs=0;const W=Worker;window.Worker=class extends W{postMessage(v,...rest){if(v.type==='run')window.runs++;return super.postMessage(v,...rest);}};});
 await page.addInitScript(()=>{const read=File.prototype.arrayBuffer;File.prototype.arrayBuffer=function(){const file=this;return window.holdChainRead?new Promise(resolve=>{window.releaseChainRead=()=>resolve(read.call(file));}):read.call(file);};});
 await page.goto(origin+'/');await page.waitForFunction(()=>!document.querySelector('#execute').disabled);

 await openWorkbenchProject(page,path.resolve('target/batch-report-manifests/browser.bisect'),{ready:({argument:__bisectOpenArgument2372,openedStatus:__bisectOpenedStatus2372})=>(()=>__bisectOpenedStatus2372==='Opened browser.bisect')(__bisectOpenArgument2372),readyArg:undefined});assert.equal(await page.evaluate(()=>runs),0);assert.equal(await page.locator('#label-build-write_manifest').isChecked(),true);
 const save=async name=>{const pending=page.waitForEvent('download');await page.locator('#save-project').click();const file=path.join(root,name);await(await pending).saveAs(file);return JSON.parse(await fs.readFile(file,'utf8'));};
 assert.equal(await page.locator('#download-label-build').isDisabled(),true);await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-label-build').disabled);const current=await save('manifest-build.bisect');assert.equal(current.files.labelBuildInputs.settings.write_manifest,true);for(const row of current.result.states){assert(row.report_manifest);const manifest=JSON.parse(current.result.files[row.report_manifest.path]);assert.equal(manifest.submission_type,'research');assert.equal(manifest.state_code,row.state);}
 await page.locator('#label-build-write_manifest').uncheck();assert.equal(await page.locator('#download-label-build').isDisabled(),true);await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-label-build').disabled);const plain=await save('plain-build.bisect');assert.equal(plain.files.labelBuildInputs.settings.write_manifest,undefined);for(const row of plain.result.states)assert.equal(row.report_manifest,undefined);assert(!Object.keys(plain.result.files).some(p=>p.endsWith('/manifest.json')));
 assert.deepEqual(errors,[]);await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({actual_worker:true,optional_manifest_retained_on_inert_open:true,native_research_manifests_in_completed_states:true,setting_edit_retires_download:true,disabled_option_keeps_legacy_package:true},null,2));console.log('Batch manifest browser: actual worker, inert source/option retention, native manifest files, stale export guard and legacy option-off package passed.');
}finally{await context?.close();await new Promise(r=>server.close(r));}
