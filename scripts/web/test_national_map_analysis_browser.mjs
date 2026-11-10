import {openWorkbenchProject} from './workbench_project_open.mjs';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
const packages=process.env.BISECT_PLAYWRIGHT_PACKAGES;
if(!packages)throw new Error('Set BISECT_PLAYWRIGHT_PACKAGES.');
const {chromium}=createRequire(path.join(packages,'package.json'))('playwright');
const root=path.resolve('target/national-map-analysis-browser'),staticRoot=path.resolve('web/lab'),wasm=path.resolve('target/wasm32-unknown-unknown/release/bisect_wasm.wasm');
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
 const save=async name=>{const event=page.waitForEvent('download');await page.locator('#save-project').click();await(await event).saveAs(path.join(root,name));return JSON.parse(await fs.readFile(path.join(root,name),'utf8'));};let downloads=0;
 for(const name of ['missing','utf8','mixed']){const expected=JSON.parse(await fs.readFile('target/national-map-analysis/'+name+'.bisect','utf8')),before=await page.evaluate(()=>executions.length);await openWorkbenchProject(page,'target/national-map-analysis/'+name+'.bisect',{ready:({argument:__bisectOpenArgument2696,openedStatus:__bisectOpenedStatus2696})=>(filename=>__bisectOpenedStatus2696==='Opened '+filename)(__bisectOpenArgument2696),readyArg:name+'.bisect'});assert.equal(await page.evaluate(()=>executions.length),before);assert.equal(await page.locator('#national-map-analysis-policy').inputValue(),'native-defaults');assert(await page.locator('#download-native-map-png').isDisabled());assert.equal(await page.locator('#native-map-preview img').count(),0);await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-native-map-png').disabled);await page.waitForFunction(()=>document.querySelector('#native-map-preview img')?.naturalWidth>0);assert.match(await page.locator('#result-tables').innerText(),/Native national analysis defaults/);assert.match(await page.locator('#result-tables').innerText(),/native-cli-unchecked/);assert.match(await page.locator('#result-tables').innerText(),/Fallback districts/);assert.match(await page.locator('#result-tables').innerText(),new RegExp(expected.result.provided_states[0].analysis_observation.status));
 for(const kind of ['png','svg']){const event=page.waitForEvent('download');await page.locator('#download-native-map-'+kind).click();const target=path.join(root,name+'.'+kind);await(await event).saveAs(target);assert.deepEqual(await fs.readFile(target),kind==='png'?Buffer.from(expected.result.png_base64,'base64'):Buffer.from(expected.result.svg_text));downloads++;}
 const saved=await save(name+'-saved.bisect');assert.deepEqual(saved.files,expected.files);assert.deepEqual(saved.result,expected.result);await page.screenshot({path:path.join(root,name+'.png'),fullPage:true});
 await page.locator('#national-map-analysis-policy').selectOption('validated');assert(await page.locator('#download-native-map-png').isDisabled());assert.equal(await page.locator('#native-map-preview img').count(),0);const strict=await save(name+'-validated.bisect'),batch=JSON.parse(Buffer.from(strict.files.nationalMapBatchInputs.source.base64,'base64')),old=JSON.parse(Buffer.from(expected.files.nationalMapBatchInputs.source.base64,'base64'));assert.equal(batch.analysis_policy,'validated');assert.deepEqual(batch.states,old.states);await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#execute').disabled);assert(await page.locator('#download-native-map-png').isDisabled());assert.equal(await page.locator('#native-map-preview img').count(),0);
 }
 assert.deepEqual(errors,[]);await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({real_rust_worker:true,exact_svg_png_downloads:downloads,missing_invalid_utf8_and_mixed_sources:true,visible_default_districts_unchecked_metadata_and_hash:true,inert_open_save_restore:true,native_and_validated_modes_distinct:true,policy_edits_retire_exports:true,original_source_bytes_preserved:true},null,2));console.log('National analysis browser: real worker, six exact SVG/PNG downloads, visible defaults/raw hashes, inert Open and source-preserving policy changes with strict-mode refusal passed.');
}finally{await context?.close();await new Promise(resolve=>server.close(resolve));}
