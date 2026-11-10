import {openWorkbenchProject} from './workbench_project_open.mjs';
// Actual local workers; no external fetches, no user tabs or source-directory writes.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
const {chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES,'package.json'))('playwright');
const root=path.resolve('target/configured-map-handoff-browser'),source=path.resolve('web/lab'),fixtures=path.resolve('target/configured-map-handoff'),wasm=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm');await fs.mkdir(root,{recursive:true});
const server=createServer(async(req,res)=>{try{const name=decodeURIComponent(new URL(req.url,'http://localhost').pathname),file=path.resolve(source,'.'+(name==='/'?'/workbench.html':name));if(!file.startsWith(source+path.sep))throw Error('Path refused');res.setHeader('Content-Type',file.endsWith('.js')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':'text/html');res.end(name==='/bisect_wasm.wasm'?wasm:await fs.readFile(file));}catch{res.writeHead(404);res.end();}});await new Promise(r=>server.listen(0,'127.0.0.1',r));const origin='http://127.0.0.1:'+server.address().port;let context,page;
try{
 context=await chromium.launchPersistentContext(path.join(root,'profile-'+Date.now()),{headless:true,executablePath:'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-background-networking','--disable-extensions']});await context.route('**/*',r=>new URL(r.request().url()).origin===origin?r.continue():r.abort());page=await context.newPage();const errors=[];page.on('pageerror',e=>errors.push(e.message));await page.addInitScript(()=>{window.requests=[];const W=Worker;window.Worker=class extends W{postMessage(m,...rest){if(m.type==='run'){requests.push(m.request);if(window.holdNativePrepare){window.releaseNativePrepare=()=>super.postMessage(m,...rest);return;}}return super.postMessage(m,...rest);}};});await page.goto(origin);
 const open=async file=>{const packet=JSON.parse(await fs.readFile(file,'utf8'));await openWorkbenchProject(page,file,{ready:({argument:__bisectOpenArgument2319,openedStatus:__bisectOpenedStatus2319})=>(({name,file})=>document.querySelector('#project-name').value===name&&__bisectOpenedStatus2319==='Opened '+file&&!document.querySelector('#execute').disabled)(__bisectOpenArgument2319),readyArg:{name:packet.name,file:path.basename(file)}});};
 const save=async name=>{const wait=page.waitForEvent('download');await page.locator('#save-project').click();const file=root+'/'+name+'.bisect';await(await wait).saveAs(file);return JSON.parse(await fs.readFile(file,'utf8'));};
 const run=async()=>{await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#execute').disabled,null,{timeout:120000});};

 const exportFresh=async()=>{
 await run();assert(await page.locator('#map-configured-plan').isDisabled());
 await page.evaluate(()=>window.holdNativePrepare=true);await page.locator('#prepare-configured-export').click();await page.waitForFunction(()=>!!window.releaseNativePrepare);assert(await page.locator('#map-configured-plan').isDisabled());
 await page.evaluate(()=>{window.holdNativePrepare=false;window.releaseNativePrepare();delete window.releaseNativePrepare;});await page.waitForFunction(()=>!document.querySelector('#map-configured-plan').disabled);
 };
 for(const state of ['RI','GB','ZZ']){
 const initial=await page.evaluate(()=>requests.length);await open(fixtures+'/'+state+'.bisect');assert.equal(await page.evaluate(()=>requests.length),initial);assert(await page.locator('#map-configured-plan').isDisabled());
 await exportFresh();const exported=await save(state+'-exported'),before=await page.evaluate(()=>requests.length);await page.locator('#map-configured-plan').click();
 await page.waitForFunction(()=>document.querySelector('#operation').value==='render-native-district-map');assert.equal(await page.evaluate(()=>requests.length),before);
 const selected=await save(state+'-selected');assert.equal(Buffer.from(selected.files.nativeMapInputs.plan.base64,'base64').toString(),exported.result.practitioner_export.document_json);assert.deepEqual(selected.files.nativeMapInputs.geometry,exported.files.planGeometry);assert.equal(selected.files.nativeMapInputs.analysis,null);assert.equal(selected.files.nativeMapInputs.round_assignments,null);assert.equal(selected.files.nativeMapInputs.map_type,'districts');
 await page.locator('#native-map-dpi').selectOption('72');await run();await page.waitForFunction(()=>!document.querySelector('#download-native-map-png').disabled);
 const mapped=await save(state+'-mapped'),request=await page.evaluate(()=>requests.filter(r=>r.operation==='render-native-district-map').at(-1)),file=root+'/'+state+'-map.request.json';await fs.writeFile(file,JSON.stringify(request));
 const proc=spawnSync(path.resolve('target/release/examples/execute_tool_request.exe'),[file],{encoding:'utf8',windowsHide:true,timeout:180000,maxBuffer:64*1024*1024});assert.equal(proc.status,0,proc.stderr);assert.deepEqual(mapped.result,JSON.parse(proc.stdout));
 await page.waitForFunction(()=>{const img=document.querySelector('#native-map-preview img');return img?.complete&&img.naturalWidth>0;});
 for(const kind of ['png','svg']){const wait=page.waitForEvent('download');await page.locator('#download-native-map-'+kind).click();await(await wait).saveAs(root+'/'+state+'.'+kind);const bytes=await fs.readFile(root+'/'+state+'.'+kind);assert.equal(createHash('sha256').update(bytes).digest('hex'),mapped.result[kind+'_sha256']);}
 if(state==='GB')await page.locator('#native-map-preview').screenshot({path:root+'/map-preview.png'});
 const beforeOpen=await page.evaluate(()=>requests.length);await open(root+'/'+state+'-mapped.bisect');assert.equal(await page.evaluate(()=>requests.length),beforeOpen);assert(await page.locator('#download-native-map-png').isDisabled());assert.equal(await page.locator('#native-map-preview img').count(),0);
 }
 const missing=JSON.parse(await fs.readFile(fixtures+'/GB.bisect','utf8')),geometry=missing.files.planGeometry;delete missing.files.planGeometry;missing.name='SYNTHETIC map without geometry';await fs.writeFile(root+'/missing.bisect',JSON.stringify(missing));await open(root+'/missing.bisect');await exportFresh();
 const beforeMissing=await page.evaluate(()=>requests.length);await page.locator('#map-configured-plan').click();assert.equal(await page.evaluate(()=>requests.length),beforeMissing);assert.match(await page.locator('#result-status').innerText(),/Select matching geographic unit geometry/);const noGeometry=await save('missing-selected');assert.equal(noGeometry.files.nativeMapInputs.geometry,null);
 await page.locator('#native-map-geometry').setInputFiles({name:geometry.name,mimeType:'application/json',buffer:Buffer.from(geometry.base64,'base64')});await page.waitForFunction(()=>!document.querySelector('#execute').disabled);await run();await page.waitForFunction(()=>!document.querySelector('#download-native-map-png').disabled);
 assert.deepEqual(errors,[]);await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm_sha256:createHash('sha256').update(wasm).digest('hex'),real_owned_workers:true,native_whole_maps:3,synthetic_RI_GB_ZZ:true,exact_handoff_without_execution:true,export_busy_guards:true,missing_geometry_selection:true,hash_verified_png_svg_downloads:true,inert_open_retires_preview_and_exports:true,page_errors:errors},null,2)+'\n');
 console.log('PASS configured map browser: 3 whole native maps, explicit handoff/Run, preview and PNG/SVG, export busy guard, missing geometry and inert Open');
}catch(error){if(page)console.log('Browser state',await page.evaluate(()=>({status:document.querySelector('#result-status')?.textContent,operation:document.querySelector('#operation')?.value})).catch(()=>null));throw error;}finally{await context?.close();await new Promise(r=>server.close(r));}
