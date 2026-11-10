// Own server and browser profile; no external requests or user tabs.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
const {chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES,'package.json'))('playwright');
const root=path.resolve('target/state-output-maps-browser'),source=path.resolve('web/lab');await fs.mkdir(root,{recursive:true});
const wasm=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm');
const server=createServer(async(req,res)=>{try{const name=decodeURIComponent(new URL(req.url,'http://localhost').pathname);if(name.includes('..'))throw Error('Path refused');const file=name==='/'?'/workbench.html':name;res.setHeader('Content-Type',file.endsWith('.js')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':'text/html');res.end(file==='/bisect_wasm.wasm'?wasm:await fs.readFile(source+file));}catch{res.writeHead(404);res.end();}});
await new Promise(r=>server.listen(0,'127.0.0.1',r));const origin='http://127.0.0.1:'+server.address().port;let context;
try{
 context=await chromium.launchPersistentContext(path.join(root,'profile-'+Date.now()),{headless:true,executablePath:'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-background-networking','--disable-extensions']});await context.route('**/*',r=>new URL(r.request().url()).origin===origin?r.continue():r.abort());
 const page=await context.newPage(),errors=[];page.on('pageerror',e=>errors.push(e.stack));
 await page.addInitScript(()=>{window.requests=[];window.mapBlobs=new Map();const makeURL=URL.createObjectURL.bind(URL);URL.createObjectURL=blob=>{const url=makeURL(blob);mapBlobs.set(url,blob);return url;};const Original=Worker;window.Worker=class extends Original{postMessage(m,...rest){if(m?.type==='run')requests.push(m.request);super.postMessage(m,...rest);}};});await page.goto(origin);
 const original=JSON.parse(await fs.readFile('target/state-national-analysis-handoff/browser.bisect','utf8'));
 const open=async()=>{await page.locator('#open-project').setInputFiles(path.resolve('target/state-national-analysis-handoff/browser.bisect'));await page.waitForFunction(()=>document.querySelector('#result-status').textContent.startsWith('Imported result is unverified'));};
 const save=async name=>{const wait=page.waitForEvent('download');await page.locator('#save-project').click();await(await wait).saveAs(path.join(root,name+'.bisect'));return JSON.parse(await fs.readFile(path.join(root,name+'.bisect'),'utf8'));};
 const run=async()=>{await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-state-run-batch').disabled);};
 await open();assert(await page.locator('#state-run-batch-national').isDisabled());assert.equal(await page.evaluate(()=>requests.length),0);await run();
 let downloads=0;page.on('download',()=>downloads++);
 for(const year of ['2020','2010','2000'])for(const kind of ['districts','political','demographic','compactness']){
  await page.locator('#state-run-batch-view-year').selectOption(year);await page.locator('#state-run-batch-map-type').selectOption(kind);assert.equal(await page.locator('#state-run-batch-map-state').inputValue(),'IA');
  const before=await page.evaluate(()=>requests.length);await page.locator('#state-run-batch-show-map').click();await page.waitForFunction(()=>document.querySelector('#state-run-batch-map-preview').naturalWidth>0&&!document.querySelector('#state-run-batch-map-preview').hidden);
  assert((await page.locator('#state-run-batch-map-preview').getAttribute('alt')).includes(kind+' map for IA '+year));
  const png=await page.locator('#state-run-batch-map-preview').evaluate(async img=>Array.from(new Uint8Array(await mapBlobs.get(img.src).arrayBuffer())));
  const workflow=original.result.years.find(r=>r.year===year).result.states[0].result.workflow;assert.deepEqual(Buffer.from(png),Buffer.from(workflow.files_base64['maps/'+kind+'.png'],'base64'));
  for(const format of ['png','svg']){const waiting=page.waitForEvent('download');await page.locator('#state-run-batch-map-'+format).click();const download=await waiting;assert.equal(download.suggestedFilename(),year+'-IA-'+kind+'.'+format);const file=path.join(root,download.suggestedFilename());await download.saveAs(file);assert.deepEqual(await fs.readFile(file),Buffer.from(workflow.files_base64['maps/'+kind+'.'+format],'base64'));}
  assert.equal(await page.evaluate(()=>requests.length),before);
 }
 assert.equal(downloads,24);await page.locator('#state-run-batch-fields').screenshot({path:path.join(root,'map-controls.png')});
 await page.evaluate(()=>{window.releaseMapHashes=[];window.holdMapHash=true;window.originalMapDigest=crypto.subtle.digest.bind(crypto.subtle);crypto.subtle.digest=async(...args)=>{const b=new Uint8Array(args[1]);if(holdMapHash&&b[0]===137)await new Promise(r=>releaseMapHashes.push(r));return originalMapDigest(...args);};});
 await page.locator('#state-run-batch-show-map').click();await page.waitForFunction(()=>releaseMapHashes.length>0);await page.locator('#state-run-batch-map-type').selectOption('political');await page.evaluate(()=>{holdMapHash=false;releaseMapHashes.splice(0).forEach(r=>r());});await page.waitForFunction(()=>document.querySelector('#result-status').textContent.includes('map selection changed'));assert(await page.locator('#state-run-batch-map-preview').isHidden());
 await page.evaluate(()=>holdMapHash=true);await page.locator('#state-run-batch-map-png').click();await page.waitForFunction(()=>releaseMapHashes.length>0);await page.locator('#state-run-batch-view-year').selectOption('2020');await page.evaluate(()=>{holdMapHash=false;releaseMapHashes.splice(0).forEach(r=>r());});await page.waitForTimeout(200);assert.equal(downloads,24);
 await page.locator('#state-run-batch-map-type').selectOption('political');await page.evaluate(()=>holdMapHash=true);await page.locator('#state-run-batch-show-map').click();await page.waitForFunction(()=>releaseMapHashes.length>0);await open();const importedStatus=await page.locator('#result-status').innerText();await page.evaluate(()=>{holdMapHash=false;releaseMapHashes.splice(0).forEach(r=>r());});await page.waitForTimeout(200);assert.equal(await page.locator('#result-status').innerText(),importedStatus);assert(await page.locator('#state-run-batch-map-preview').isHidden());
 for(const id of ['show-map','map-png','map-svg'])assert(await page.locator('#state-run-batch-'+id).isDisabled());assert.equal(await page.locator('#state-run-batch-map-type').inputValue(),'districts');assert.deepEqual(errors,[]);
 await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({four_types:true,three_years:true,exact_png_previews:12,exact_downloaded_files:24,no_engine_call_on_view_or_download:true,changed_type_retires_preview:true,changed_year_retires_download:true,open_retires_pending_preview:true,import_inert:true,page_errors:errors},null,2)+'\n');console.log('State map browser: 12 exact PNG previews, 24 exact native downloads, no engine execution, stale type/year/Open retirement and inert imported outputs passed.');
}finally{await context?.close();await new Promise(r=>server.close(r));}

