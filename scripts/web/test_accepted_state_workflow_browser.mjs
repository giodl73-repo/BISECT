// Own test server/profile only; never attach to the user's running browser.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
const packageRoot=process.env.BISECT_PLAYWRIGHT_PACKAGES;if(!packageRoot)throw new Error('Set BISECT_PLAYWRIGHT_PACKAGES to an installed Playwright package root.');
const {chromium}=createRequire(path.join(packageRoot,'package.json'))('playwright');
const root=path.resolve('target/accepted-state-workflow-browser');await fs.mkdir(root,{recursive:true});
const staticRoot=path.resolve('web/lab'),wasm=path.resolve('target/wasm32-unknown-unknown/release/bisect_wasm.wasm');
const server=createServer(async(req,res)=>{try{const url=new URL(req.url,'http://localhost'),target=url.pathname==='/bisect_wasm.wasm'?wasm:path.resolve(staticRoot,'.'+decodeURIComponent(url.pathname==='/'?'/workbench.html':url.pathname));if(target!==wasm&&!target.startsWith(staticRoot+path.sep))throw new Error('Path rejected');res.setHeader('Content-Type',target.endsWith('.js')?'application/javascript':target.endsWith('.wasm')?'application/wasm':target.endsWith('.css')?'text/css':'text/html');res.end(await fs.readFile(target));}catch{res.writeHead(404);res.end();}});await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const url='http://127.0.0.1:'+server.address().port,profile=path.join(root,'profile-'+Date.now());let context;
async function launch(){const c=await chromium.launchPersistentContext(profile,{headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-background-networking','--disable-extensions']});await c.route('**/*',r=>new URL(r.request().url()).origin===url?r.continue():r.abort());return c;}



try{
 context=await launch();const page=await context.newPage(),errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.addInitScript(()=>{window.requests=[];const NativeWorker=Worker;window.Worker=class extends NativeWorker{postMessage(data,...args){if(data.type==='run')window.requests.push(structuredClone(data.request));return super.postMessage(data,...args);}};});
 await page.goto(url);await page.waitForFunction(()=>!document.querySelector('#execute').disabled);
 const save=async(name)=>{const wait=page.waitForEvent('download');await page.locator('#save-project').click();const d=await wait,file=path.join(root,name);await d.saveAs(file);return JSON.parse(await fs.readFile(file,'utf8'));};
 const open=async(file)=>{const before=await page.evaluate(()=>window.requests.length);await page.locator('#open-project').setInputFiles(file);await page.waitForFunction(()=>document.querySelector('#result-status').textContent.startsWith('Imported result'));assert.equal(await page.evaluate(()=>window.requests.length),before);};
 await open(path.resolve('target/accepted-state-workflow/prepared.bisect'));
 assert(await page.locator('#prepared-engine-workflow').isDisabled());
 // Imported results stay inactive until explicit execution reproduces the Run.
 await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#prepared-engine-workflow').disabled,{timeout:120000});
 assert.equal(await page.evaluate(()=>window.requests.length),1);
 await page.locator('#prepared-engine-workflow').click();assert.equal(await page.locator('#operation').inputValue(),'run-accepted-state-workflow');
 assert.equal(await page.evaluate(()=>window.requests.length),1);assert(await page.locator('#state-workflow-fields').isVisible());assert(await page.locator('#auditContextText-file').isVisible());assert(await page.locator('#plan-geometry-file').isVisible());
 await page.locator('#tui-run-workflow-fields').evaluate(el=>el.open=true);
 await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-tui-workflow').disabled,{timeout:120000});
 const saved=await save('complete.bisect');assert.equal(saved.result.schema_version,'bisect-accepted-state-workflow-v1');assert.equal(saved.result.partition_execution_performed,false);assert.equal(saved.result.workflow.status,'completed',JSON.stringify(saved.result.workflow.stages));
 const requests=await page.evaluate(()=>window.requests);assert.equal(requests.length,2);assert.equal(requests[1].operation,'run-accepted-state-workflow');const original=JSON.parse(await fs.readFile('target/accepted-state-workflow/prepared.bisect','utf8'));assert.equal(requests[1].input.document_json,original.result.export.document_json);assert.equal(requests[1].input.context_json,original.result.export.context_json);
 await page.locator('#tui-workflow-preview-type').selectOption('political');await page.waitForFunction(()=>document.querySelector('#tui-workflow-map-preview img')?.naturalWidth>0);
 await page.locator('#preview-commission').click();await page.waitForFunction(()=>!!document.querySelector('#commission-preview-host iframe'));const sandbox=await page.locator('#commission-preview-host iframe').getAttribute('sandbox');assert(!sandbox.includes('allow-scripts'));await page.locator('#close-commission-preview').click();
 const wait=page.waitForEvent('download');await page.locator('#download-tui-workflow').click();await(await wait).saveAs(path.join(root,'complete.zip'));
 await page.locator('#tui-workflow-dpi').selectOption('100');assert(await page.locator('#download-tui-workflow').isDisabled());assert.equal(await page.locator('#tui-workflow-map-preview img').count(),0);
 await open(path.join(root,'complete.bisect'));assert(await page.locator('#download-tui-workflow').isDisabled());assert.equal(await page.locator('#tui-workflow-map-preview img').count(),0);assert.equal(await page.evaluate(()=>window.requests.length),2);
 await page.setViewportSize({width:390,height:844});assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth));assert.deepEqual(errors,[]);
 await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({worker_requests:2,inert_open_and_handoff:true,no_partition_in_followup:true,native_map_preview:true,sandboxed_report_preview:true,zip_download:true,mobile_width:390,page_errors:errors},null,2));console.log('PASS accepted State workflow browser: inert Open/handoff, exact exports, one follow-up request, native map preview/ZIP and mobile bounds');
}finally{await context?.close();await new Promise(resolve=>server.close(resolve));}
