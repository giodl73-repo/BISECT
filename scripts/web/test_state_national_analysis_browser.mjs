// Own server and browser profile; no external requests or user tabs.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
const {chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES,'package.json'))('playwright');
const root=path.resolve('target/state-national-analysis-browser'),source=path.resolve('web/lab');await fs.mkdir(root,{recursive:true});
const wasm=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm');
const server=createServer(async(req,res)=>{try{const name=decodeURIComponent(new URL(req.url,'http://localhost').pathname);if(name.includes('..'))throw Error('Path refused');const file=name==='/'?'/workbench.html':name;res.setHeader('Content-Type',file.endsWith('.js')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':'text/html');res.end(file==='/bisect_wasm.wasm'?wasm:await fs.readFile(source+file));}catch{res.writeHead(404);res.end();}});
await new Promise(r=>server.listen(0,'127.0.0.1',r));const origin='http://127.0.0.1:'+server.address().port;let context;
try{
 context=await chromium.launchPersistentContext(path.join(root,'profile-'+Date.now()),{headless:true,executablePath:'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-background-networking','--disable-extensions']});await context.route('**/*',r=>new URL(r.request().url()).origin===origin?r.continue():r.abort());
 const page=await context.newPage(),errors=[];page.on('pageerror',e=>errors.push(e.stack));
 await page.addInitScript(()=>{window.requests=[];const Original=Worker;window.Worker=class extends Original{postMessage(m,...rest){if(m?.type==='run')requests.push(m.request);super.postMessage(m,...rest);}};});await page.goto(origin);
 const original=JSON.parse(await fs.readFile('target/state-national-analysis-handoff/browser.bisect','utf8'));
 const open=async()=>{await page.locator('#open-project').setInputFiles(path.resolve('target/state-national-analysis-handoff/browser.bisect'));await page.waitForFunction(()=>document.querySelector('#result-status').textContent.startsWith('Imported result is unverified'));};
 const save=async name=>{const wait=page.waitForEvent('download');await page.locator('#save-project').click();await(await wait).saveAs(path.join(root,name+'.bisect'));return JSON.parse(await fs.readFile(path.join(root,name+'.bisect'),'utf8'));};
 const run=async()=>{await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-state-run-batch').disabled);};
 await open();assert(await page.locator('#state-run-batch-national').isDisabled());assert.equal(await page.evaluate(()=>requests.length),0);await run();
 for(const kind of ['political','demographic','compactness']){
  await page.locator('#state-run-batch-view-year').selectOption('2000');await page.locator('#state-run-batch-national-type').selectOption(kind);assert(!await page.locator('#state-run-batch-national').isDisabled());
  const before=await page.evaluate(()=>requests.length);await page.locator('#state-run-batch-national').click();await page.waitForFunction(()=>document.querySelector('#operation').value==='render-native-national-map-batch');assert.equal(await page.evaluate(()=>requests.length),before);
  const selected=await save('selected-'+kind),expected=JSON.parse(await fs.readFile('target/state-national-analysis-handoff/2000-'+kind+'.bisect','utf8'));assert.deepEqual(selected.files.nationalMapBatchInputs,expected.files.nationalMapBatchInputs);
  await page.locator('#execute').click();await page.waitForFunction(()=>document.querySelector('#result-status').textContent.startsWith('Operation finished'));assert.deepEqual((await save('rendered-'+kind)).result,expected.result);
  await open();assert.equal(await page.locator('#state-run-batch-national-type').inputValue(),'districts');await run();
 }
 await page.evaluate(()=>{window.originalDigest=crypto.subtle.digest.bind(crypto.subtle);crypto.subtle.digest=async(...args)=>{await new Promise(r=>setTimeout(r,300));return originalDigest(...args);};});
 await page.locator('#state-run-batch-national-type').selectOption('political');await page.locator('#state-run-batch-national').click();assert(await page.locator('#execute').isDisabled());await page.locator('#state-run-batch-national-type').selectOption('compactness');
 await page.waitForFunction(()=>document.querySelector('#result-status').textContent.includes('selection changed while checking'));assert.equal(await page.locator('#operation').inputValue(),'run-state-batch');assert.equal((await save('retired-type')).files.nationalMapBatchInputs,undefined);assert(!await page.locator('#state-run-batch-national').isDisabled());
 await page.locator('#state-run-batch-national').click();await open();const importedStatus=await page.locator('#result-status').innerText();await page.waitForTimeout(2500);assert.equal(await page.locator('#result-status').innerText(),importedStatus);assert(await page.locator('#state-run-batch-national').isDisabled());
 assert.deepEqual((await save('reopened')).files.stateRunBatchInputs,original.files.stateRunBatchInputs);assert.deepEqual(errors,[]);
 await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({map_types:['political','demographic','compactness'],three_year_configured_workers:true,exact_2000_analysis_and_geometry_sources:true,national_results_exact_native_verified:true,selection_runs_no_engine:true,import_inert:true,map_type_change_retires_hash:true,open_retires_hash:true,source_roundtrip:true,page_errors:errors},null,2)+'\n');console.log('National analysis browser: three map types, exact retained analysis and native national renders; inert selection/Open and stale type/Open gates passed.');
}finally{await context?.close();await new Promise(r=>server.close(r));}
