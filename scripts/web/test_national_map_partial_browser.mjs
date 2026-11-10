import {openWorkbenchProject} from './workbench_project_open.mjs';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
const packages=process.env.BISECT_PLAYWRIGHT_PACKAGES;
if(!packages)throw new Error('Set BISECT_PLAYWRIGHT_PACKAGES.');
const {chromium}=createRequire(path.join(packages,'package.json'))('playwright');
const root=path.resolve('target/national-map-partial-browser'),staticRoot=path.resolve('web/lab'),wasm=path.resolve('target/wasm32-unknown-unknown/release/bisect_wasm.wasm');
await fs.mkdir(root,{recursive:true});
const server=createServer(async(req,res)=>{
 try{
  const url=new URL(req.url,'http://localhost'),file=url.pathname==='/bisect_wasm.wasm'?wasm:path.resolve(staticRoot,'.'+decodeURIComponent(url.pathname==='/'?'/workbench.html':url.pathname));
  if(file!==wasm&&!file.startsWith(staticRoot+path.sep))throw new Error('Outside root');
  res.setHeader('Content-Type',file.endsWith('.js')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':'text/html');res.end(await fs.readFile(file));
 }catch{res.writeHead(404);res.end();}
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const origin='http://127.0.0.1:'+server.address().port;let context;
try{
 context=await chromium.launchPersistentContext(path.join(root,'profile-'+Date.now()),{headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-background-networking','--disable-extensions']});await context.route('**/*',r=>new URL(r.request().url()).origin===origin?r.continue():r.abort());const page=await context.newPage(),errors=[];page.on('pageerror',e=>errors.push(e.message));await page.addInitScript(()=>{window.executions=[];const Real=Worker;window.Worker=class extends Real{postMessage(message,...rest){if(message?.type==='run')executions.push(message.request);return super.postMessage(message,...rest);}};});await page.goto(origin);await page.waitForFunction(()=>!document.querySelector('#execute').disabled);
 const save=async name=>{const event=page.waitForEvent('download');await page.locator('#save-project').click();await(await event).saveAs(path.join(root,name));return JSON.parse(await fs.readFile(path.join(root,name),'utf8'));};let checks=0;
 for(const name of ['partial','empty']){
  const expected=JSON.parse(await fs.readFile('target/national-map-partial/'+name+'.bisect','utf8'));const before=await page.evaluate(()=>executions.length);await openWorkbenchProject(page,'target/national-map-partial/'+name+'.bisect',{ready:({argument:__bisectOpenArgument2693,openedStatus:__bisectOpenedStatus2693})=>(filename=>__bisectOpenedStatus2693==='Opened '+filename)(__bisectOpenArgument2693),readyArg:name+'.bisect'});assert.equal(await page.evaluate(()=>executions.length),before);assert(await page.locator('#national-map-continue').isChecked());assert(await page.locator('#download-native-map-png').isDisabled());assert.equal(await page.locator('#native-map-preview img').count(),0);await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-native-map-png').disabled);await page.waitForFunction(()=>document.querySelector('#native-map-preview img')?.naturalWidth>0);assert.match(await page.locator('#result-tables').innerText(),name==='empty'?/Empty national map/:/Supplied national map coverage/);assert.match(await page.locator('#result-tables').innerText(),/omitted/);
  for(const kind of ['png','svg']){const event=page.waitForEvent('download');await page.locator('#download-native-map-'+kind).click();const downloaded=await event,target=path.join(root,name+'.'+kind);await downloaded.saveAs(target);assert.deepEqual(await fs.readFile(target),kind==='png'?Buffer.from(expected.result.png_base64,'base64'):Buffer.from(expected.result.svg_text));checks++;}
  const saved=await save(name+'-saved.bisect');assert.deepEqual(saved.files,expected.files);const normalize=result=>{const copy=structuredClone(result);for(const row of [...copy.national_coverage.omitted_states,...copy.national_coverage.state_outcomes])if(row.stage==='source-preparation'){assert.equal(typeof row.error,'string');assert(row.error.length>0);row.error='observed-source-error';}return copy;};assert.deepEqual(normalize(saved.result),normalize(expected.result));await page.screenshot({path:path.join(root,name+'.png'),fullPage:true});
  await page.locator('#national-map-continue').uncheck();assert(await page.locator('#download-native-map-png').isDisabled());assert.equal(await page.locator('#native-map-preview img').count(),0);const strict=await save(name+'-strict.bisect');const batch=JSON.parse(Buffer.from(strict.files.nationalMapBatchInputs.source.base64,'base64'));assert.equal(batch.failure_policy,'stop');const original=JSON.parse(Buffer.from(expected.files.nationalMapBatchInputs.source.base64,'base64'));assert.deepEqual(batch.states,original.states);await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#execute').disabled);assert(await page.locator('#download-native-map-png').isDisabled());assert.equal(await page.locator('#native-map-preview img').count(),0);
 }
 assert.deepEqual(errors,[]);await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({real_rust_worker:true,exact_map_downloads:checks,visible_partial_and_empty_coverage:true,inert_open:true,save_sources_results:true,policy_edit_retires_outputs:true,exact_original_state_sources_preserved:true,strict_failure_no_stale_preview:true},null,2));console.log('Partial national browser: real worker, exact partial/empty SVG/PNG, visible omissions, inert Save/Open and source-preserving policy edits with stale-output retirement passed.');
}finally{await context?.close();await new Promise(resolve=>server.close(resolve));}
