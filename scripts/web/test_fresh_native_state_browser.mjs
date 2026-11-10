import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {loadNativeUnitIdentityCorpus} from './fixtures/native-unit-identity.mjs';
import {freshNativeStateInput} from './fixtures/fresh-native-state.mjs';
const argv=process.argv.slice(2),arg=name=>{const i=argv.indexOf(name);assert(i>=0,'Supply '+name);return path.resolve(argv[i+1]);};
const site=arg('--site'),parent=arg('--output');await fs.mkdir(parent,{recursive:true});
const root=await fs.mkdtemp(path.join(parent,'fresh-panel-'));
const require=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES??process.cwd(),'package.json'));
const {chromium}=require('playwright');
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const html='<!doctype html><html><head><meta charset="utf-8"><meta http-equiv="Content-Security-Policy" content="default-src \'self\'; script-src \'self\' \'wasm-unsafe-eval\'; worker-src \'self\'; connect-src \'self\'; object-src \'none\'; base-uri \'none\'"><link rel="stylesheet" href="./workbench.css"></head><body><main><section id="practitioner-flow">Workbench fresh-source panel test</section></main><script type="module" src="./fresh-native-state-panel.js"></script></body></html>';
const server=createServer(async(req,res)=>{try{
 const url=new URL(req.url,'http://localhost'),name=decodeURIComponent(url.pathname);
 if(name==='/panel.html'){res.setHeader('Content-Type','text/html');res.end(html);return;}
 const file=path.resolve(site,'.'+name);if(!file.startsWith(site+path.sep)){res.writeHead(403);res.end();return;}
 const bytes=await fs.readFile(file);res.setHeader('Content-Type',file.endsWith('.wasm')?'application/wasm':file.endsWith('.js')?'text/javascript':file.endsWith('.css')?'text/css':'text/html');res.end(bytes);
 }catch{res.writeHead(404);res.end();}});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));const origin='http://127.0.0.1:'+server.address().port;
const rows=await loadNativeUnitIdentityCorpus(),errors=[],external=[];
const evidence={status:'running',scope:'Real Chrome panel, actual package WASM worker, twelve complete raw-source cases. Synthetic exact-ID display polygons test rendering only. No C sidecar, full-state capacity, fine/report context or release claim.',wasm_sha256:sha(await fs.readFile(path.join(site,'bisect_wasm.wasm'))),cases:[]};
await fs.writeFile(path.join(root,'declaration.json'),JSON.stringify({...evidence,cases:rows.map(row=>row.name)},null,2));
const save=()=>fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(evidence,null,2));await save();
let browser,context;
try{
 browser=await chromium.launch({headless:true,executablePath:process.env.BISECT_CHROME_PATH??'C:/Program Files/Google/Chrome/Application/chrome.exe'});
 context=await browser.newContext({acceptDownloads:true});
 await context.route('**/*',route=>{if(new URL(route.request().url()).origin!==origin){external.push(route.request().url());return route.abort();}return route.continue();});
 await context.addInitScript(()=>{const Native=window.Worker;window.freshOperations=[];window.Worker=class extends Native{postMessage(value,...options){if(value?.request?.operation)window.freshOperations.push(value.request.operation);return super.postMessage(value,...options);}};});
 const page=await context.newPage();page.on('pageerror',error=>errors.push(error.message));
 const get=name=>page.locator('#fresh-native-state [data-fresh="'+name+'"]');
 const download=async(name,file)=>{const pending=page.waitForEvent('download');await get(name).click();const value=await pending;await value.saveAs(file);return fs.readFile(file,'utf8');};
 for(const row of rows){
  const folder=path.join(root,row.name);await fs.mkdir(folder);const input=freshNativeStateInput(row),saved=JSON.stringify({schema_version:'bisect-fresh-native-state-inputs-v1',input});await fs.writeFile(path.join(folder,'selected-inputs.json'),saved);
  await page.goto(origin+'/panel.html');await page.locator('#fresh-native-state summary').click();
  await get('open').setInputFiles({name:'selected-inputs.json',mimeType:'application/json',buffer:Buffer.from(saved)});
  await page.waitForFunction(()=>document.querySelector('[data-fresh="status"]').textContent.startsWith('Inputs opened.'));
  assert.deepEqual(await page.evaluate(()=>window.freshOperations),[]);
  const ids=JSON.parse(row.document_json).plan.units.unit_ids;
  const geometry={type:'FeatureCollection',features:ids.map((id,i)=>{const x=-71.5+i/16,y=41.5,w=1/32;return {type:'Feature',properties:{GEOID:id},geometry:{type:'Polygon',coordinates:[[[x,y],[x+w,y],[x+w,y+w],[x,y+w],[x,y]]]}};})};
  await get('geometry').setInputFiles({name:'synthetic-display.geojson',mimeType:'application/geo+json',buffer:Buffer.from(JSON.stringify(geometry))});
  await page.waitForFunction(()=>!document.querySelector('[data-fresh="review"]').disabled);
  await get('review').click();await page.waitForFunction(()=>!document.querySelector('[data-fresh="run"]').disabled);
  assert.deepEqual(await page.evaluate(()=>window.freshOperations),['prepare-fresh-native-state-run']);
  assert.match(await get('seed').textContent(),/18446744073709551615/);
  const reopened=await download('save',path.join(folder,'saved-inputs.json'));
  await get('run').click();await page.waitForFunction(()=>!document.querySelector('[data-fresh="plan"]').disabled);
  const document=JSON.parse(await download('plan',path.join(folder,'plan.rplan'))),nativeContext=JSON.parse(await download('context',path.join(folder,'context.rctx')));
  assert.deepEqual(document.plan,JSON.parse(row.document_json).plan);
  const original=JSON.parse(row.context_json);for(const key of ['units','graph','populations','subdivisions','demographics','geometry'])assert.deepEqual(nativeContext[key],original[key],row.name+':'+key);
  assert.equal(await get('map').locator('svg path').count(),4);
  if(evidence.cases.length===0){
   await page.screenshot({path:path.join(root,'mapped-panel.png'),fullPage:true});
   await page.setViewportSize({width:390,height:844});
   assert(await get('arguments').evaluate(node=>node.getBoundingClientRect().width>300));
   assert(await page.locator('#fresh-native-state').evaluate(node=>node.scrollWidth<=node.clientWidth+2));
   await page.screenshot({path:path.join(root,'mobile-panel.png'),fullPage:true});await page.setViewportSize({width:1280,height:720});
  }
  assert.match(await get('seed').textContent(),/18446744073709551615/);
  const record=JSON.parse(await download('report',path.join(folder,'run-record.json')));
  assert.equal(record.result.input_sha256,record.result.export.fresh_native_state_binding.input_sha256);
  assert.equal(record.result.export.fresh_native_state_binding.original_context_loaded,false);
  assert.equal(record.result.recorded_paths_opened,false);
  assert.equal(record.input_json_sha256,sha(Buffer.from(JSON.stringify(record.input))));
  assert.equal(record.display_geometry_json,JSON.stringify(geometry));assert.equal(record.display_geometry_sha256,sha(Buffer.from(record.display_geometry_json)));
  await get('label').fill('Changed inputs');assert(await get('run').isDisabled());assert(await get('plan').isDisabled());
  const before=await page.evaluate(()=>window.freshOperations.length);
  await get('open').setInputFiles({name:'saved-inputs.json',mimeType:'application/json',buffer:Buffer.from(reopened)});
  await page.waitForFunction(()=>document.querySelector('[data-fresh="status"]').textContent.startsWith('Inputs opened.'));
  assert.equal(await page.evaluate(()=>window.freshOperations.length),before);assert(await get('run').isDisabled());
  await get('review').click();await page.waitForFunction(()=>!document.querySelector('[data-fresh="run"]').disabled);
  await get('run').click();await page.waitForFunction(()=>!document.querySelector('[data-fresh="plan"]').disabled);
  const second=JSON.parse(await download('plan',path.join(folder,'reopened-plan.rplan')));assert.deepEqual(second.plan,document.plan);
  assert.equal(await get('map').locator('svg path').count(),4);
  evidence.cases.push({name:row.name,status:'passed',explicit_runs:2,review_inert:true,open_inert:true,edit_retires_result:true,plan_and_physical_context_exact:true,map_units:4,saved_display_geometry_restored:true});await save();
 }
 // Selected raw files also use the same path; imported inputs are unnecessary.
 await page.goto(origin+'/panel.html');await page.locator('#fresh-native-state summary').click();
 const selected=freshNativeStateInput(rows.find(row=>row.name==='tract'));
 for(const kind of ['adjacency','geoids','centroids'])if(selected.source.sources[kind]){
  const source=selected.source.sources[kind];await get(kind).setInputFiles({name:source.name,mimeType:'application/octet-stream',buffer:Buffer.from(source.base64,'base64')});await page.waitForFunction(()=>!document.querySelector('[data-fresh="review"]').disabled);
 }
 await get('arguments').fill(JSON.stringify(selected.arguments.slice(0,-2)));await get('review').click();await page.waitForFunction(()=>!document.querySelector('[data-fresh="run"]').disabled);assert.deepEqual(await page.evaluate(()=>window.freshOperations),['prepare-fresh-native-state-run']);
 // Cancel retires the review worker before any partition request.
 await get('review').click();await get('cancel').click();assert(await get('run').isDisabled());assert(!await get('review').isDisabled());
 assert.equal((await page.evaluate(()=>window.freshOperations)).filter(v=>v==='run-fresh-native-state').length,0);
 await get('review').click();await page.waitForFunction(()=>!document.querySelector('[data-fresh="run"]').disabled);
 await get('run').click();await page.waitForFunction(()=>!document.querySelector('[data-fresh="plan"]').disabled);
 await page.screenshot({path:path.join(root,'fresh-native-state-panel.png'),fullPage:true});
 const malicious={schema_version:'bisect-fresh-native-state-inputs-v1',input:{...selected,context_json:'<script>window.freshInjected=true</script>'}};
 const count=await page.evaluate(()=>window.freshOperations.length);
 await get('open').setInputFiles({name:'untrusted.json',mimeType:'application/json',buffer:Buffer.from(JSON.stringify(malicious))});
 await page.waitForFunction(()=>document.querySelector('[data-fresh="status"]').textContent.includes('unsupported fields'));
 assert.equal(await page.evaluate(()=>window.freshInjected),undefined);assert.equal(await page.evaluate(()=>window.freshOperations.length),count);assert(await get('run').isDisabled());
 await page.goto(origin+'/index.html');await page.waitForSelector('#fresh-native-state');assert.equal(await page.locator('#fresh-native-state').count(),1);
 assert.equal((await page.evaluate(()=>window.freshOperations)).filter(v=>v==='run-fresh-native-state').length,0);
 assert.deepEqual(errors,[]);assert.deepEqual(external,[]);
 evidence.workbench_panel_registered_without_fresh_execution=true;evidence.mobile_controls_fit=true;evidence.raw_file_selection_run=true;evidence.cancel_retires_review_without_partition=true;evidence.imported_context_refused_without_execution=true;evidence.browser=browser.version();evidence.status='passed-twelve-fresh-state-panel-save-open-map-workflows';
}catch(error){evidence.status='failed';evidence.error={message:error.message,stack:error.stack};process.exitCode=1;}
finally{await save();await context?.close();await browser?.close();await new Promise(resolve=>server.close(resolve));}
console.log(JSON.stringify({root,status:evidence.status,cases:evidence.cases.length,error:evidence.error}));
