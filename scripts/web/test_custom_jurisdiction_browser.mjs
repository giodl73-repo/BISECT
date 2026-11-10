import {openWorkbenchProject} from './workbench_project_open.mjs';
// Actual local workers; no external fetches, no user tabs or source-directory writes.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
const {chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES,'package.json'))('playwright');
const root=path.resolve('target/custom-jurisdiction-browser'),source=path.resolve('web/lab'),fixtures=path.resolve('target/custom-jurisdiction-engine'),wasm=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm');await fs.mkdir(root,{recursive:true});
const server=createServer(async(req,res)=>{try{const name=decodeURIComponent(new URL(req.url,'http://localhost').pathname),file=path.resolve(source,'.'+(name==='/'?'/workbench.html':name));if(!file.startsWith(source+path.sep))throw Error('Path refused');res.setHeader('Content-Type',file.endsWith('.js')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':'text/html');res.end(name==='/bisect_wasm.wasm'?wasm:await fs.readFile(file));}catch{res.writeHead(404);res.end();}});await new Promise(r=>server.listen(0,'127.0.0.1',r));const origin='http://127.0.0.1:'+server.address().port;let context,page;
try{
 context=await chromium.launchPersistentContext(path.join(root,'profile-'+Date.now()),{headless:true,executablePath:'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-background-networking','--disable-extensions']});await context.route('**/*',r=>new URL(r.request().url()).origin===origin?r.continue():r.abort());page=await context.newPage();const errors=[];page.on('pageerror',e=>errors.push(e.message));await page.addInitScript(()=>{window.requests=[];const W=Worker;window.Worker=class extends W{postMessage(m,...rest){if(m.type==='run'){requests.push(m.request);if(window.holdNativePrepare){window.releaseNativePrepare=()=>super.postMessage(m,...rest);return;}}return super.postMessage(m,...rest);}};});await page.goto(origin);
 const open=async file=>{const packet=JSON.parse(await fs.readFile(file,'utf8'));await openWorkbenchProject(page,file,{ready:({argument:__bisectOpenArgument2273,openedStatus:__bisectOpenedStatus2273})=>(({name,file})=>document.querySelector('#project-name').value===name&&__bisectOpenedStatus2273==='Opened '+file&&!document.querySelector('#execute').disabled)(__bisectOpenArgument2273),readyArg:{name:packet.name,file:path.basename(file)}});};
 const save=async name=>{const wait=page.waitForEvent('download');await page.locator('#save-project').click();const file=root+'/'+name+'.bisect';await(await wait).saveAs(file);return JSON.parse(await fs.readFile(file,'utf8'));};
 const run=async()=>{await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#execute').disabled,null,{timeout:120000});};

 for(const [state,kind]of [['GB','precinct'],['ZZ','imported']]){
 const before=await page.evaluate(()=>requests.length);await open(fixtures+'/'+state+'-'+kind+'.bisect');assert.equal(await page.evaluate(()=>requests.length),before);
 assert.equal(await page.locator('#native-adjacency-state').inputValue(),state);assert.equal(await page.locator('#native-adjacency-unit_kind').inputValue(),kind);
 assert(await page.locator('#queue-native-source-job').isDisabled());await run();await page.waitForFunction(()=>!document.querySelector('#use-native-engine-graph').disabled);
 const prepared=await save(state+'-prepared'),graph=JSON.parse(prepared.result.graph_json);assert.equal(graph.state,state);assert.equal(prepared.result.suggested_districts,null);
 await page.locator('#use-native-engine-graph').click();await page.waitForFunction(()=>document.querySelector('#operation').value==='run-algorithm-config');
 await page.locator('#config-run-districts').fill('2');await page.locator('#config-run-districts').dispatchEvent('change');await run();
 const completed=await save(state+'-configured'),expected=JSON.parse(await fs.readFile(fixtures+'/'+state+'-'+kind+'-configured-result.json','utf8'));delete expected.practitioner_export;assert.deepEqual(completed.result,expected);
 await page.locator('#prepare-configured-export').click();await page.waitForFunction(()=>!document.querySelector('#download-configured-context').disabled);
 const exported=await save(state+'-exported');assert.equal(exported.result.practitioner_export.context.units.state,state);assert.deepEqual(exported.result.practitioner_export.context.subdivisions.county_ids,graph.county_ids);
 const download=page.waitForEvent('download');await page.locator('#download-configured-source-bundle').click();await(await download).saveAs(root+'/'+state+'-sources.zip');
 const beforeOpen=await page.evaluate(()=>requests.length);await open(root+'/'+state+'-exported.bisect');assert.equal(await page.evaluate(()=>requests.length),beforeOpen);assert(await page.locator('#download-configured-context').isDisabled());
 }
 await page.locator('#operation').selectOption('prepare-native-engine-input');await page.locator('#native-adjacency-fields').screenshot({path:root+'/custom-jurisdiction.png'});assert.deepEqual(errors,[]);
 await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm_sha256:createHash('sha256').update(wasm).digest('hex'),real_owned_workers:true,synthetic_GB_precinct_and_ZZ_imported:true,native_whole_configured_results:true,exports_and_source_zips:true,congressional_queue_unavailable:true,open_inert:true,page_errors:errors},null,2)+'\n');
 console.log('PASS custom jurisdiction browser: original binary selection, configured Run, native results, export, source ZIP, explicit district counts and inert Open');
}catch(error){if(page)console.log('Browser state',await page.evaluate(()=>({status:document.querySelector('#result-status')?.textContent,operation:document.querySelector('#operation')?.value})).catch(()=>null));throw error;}finally{await context?.close();await new Promise(r=>server.close(r));}
