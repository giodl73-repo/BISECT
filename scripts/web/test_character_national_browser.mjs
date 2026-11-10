// Integration: preceding national character gate supplies both actual projects.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {parseLabProject,MAX_LAB_PROJECT_VALUES} from '../../web/lab/laboratory-project.js';
import {MAX_PROJECT_BYTES} from '../../web/lab/project.js';
assert(process.argv[2],'Supply the laboratory package.');
const lab=path.resolve(process.argv[2]),packaged=process.argv.includes('--require-package'),source=path.resolve('web/lab'),served=new Map(),sha=b=>createHash('sha256').update(b).digest('hex');
const root=await fs.mkdtemp(path.resolve('target/national-character-browser-'));
const server=createServer(async(req,res)=>{try{const url=new URL(req.url,'http://localhost'),name=decodeURIComponent(url.pathname==='/'?'/index.html':url.pathname);let file=path.resolve(lab,'.'+name);assert(file.startsWith(lab+path.sep));const overlay=path.resolve(source,'.'+name);if(!packaged&&/\.(js|css)$/.test(name)&&overlay.startsWith(source+path.sep)&&await fs.stat(overlay).then(s=>s.isFile(),()=>false))file=overlay;const bytes=await fs.readFile(file);served.set(file,{path:file,sha256:sha(bytes),byte_length:bytes.length});res.setHeader('Content-Type',file.endsWith('.js')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':file.endsWith('.json')?'application/json':'text/html');res.end(bytes);}catch{res.writeHead(404);res.end();}});
await new Promise(r=>server.listen(0,'127.0.0.1',r));const origin='http://127.0.0.1:'+server.address().port;
const {chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES,'package.json'))('playwright');let browser;const records=[];
try{
 browser=await chromium.launch({headless:true,executablePath:'C:/Program Files/Google/Chrome/Application/chrome.exe'});const context=await browser.newContext({acceptDownloads:true}),external=[],errors=[];await context.route('**/*',r=>new URL(r.request().url()).origin===origin?r.continue():(external.push(r.request().url()),r.abort()));const page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));await page.addInitScript(()=>{window.generationRequests=[];const W=Worker;window.Worker=class extends W{postMessage(m,...args){if(m?.type==='run'&&m.request)generationRequests.push(m.request.operation??'engine');return super.postMessage(m,...args);}};});await page.goto(origin);await page.locator('#open-lab-project').waitFor();
 const save=async name=>{const waiting=page.waitForEvent('download');await page.locator('#save-lab-project').click();const file=path.join(root,name);await(await waiting).saveAs(file);const text=await fs.readFile(file,'utf8');assert(Buffer.byteLength(text)<=MAX_PROJECT_BYTES);return {file,project:parseLabProject(text),bytes:Buffer.byteLength(text)};};
 for(const kind of ['economic','housing']){
  const file=path.resolve('target/character-national-'+kind+'.bisect'),expected=parseLabProject(await fs.readFile(file,'utf8'));
  await page.locator('#open-lab-project').setInputFiles(file);await page.waitForFunction(()=>document.querySelector('#lab-project-status').textContent.startsWith('Opened project.'),null,{timeout:180000});assert.equal(await page.evaluate(()=>generationRequests.length),0);await page.locator('#district-view').click();await page.locator('#map').waitFor({state:'visible'});await page.waitForFunction(()=>!document.querySelector('#download-map').disabled,null,{timeout:120000});
  const saved=await save(kind+'-download.bisect');assert.deepEqual(saved.project.assignments,expected.assignments);assert.deepEqual(saved.project.experiment.config.characters,expected.experiment.config.characters);assert.equal(saved.project.experiment.states.length,expected.experiment.states.length);
  await page.locator('#open-lab-project').setInputFiles(saved.file);await page.waitForFunction(()=>document.querySelector('#lab-project-status').textContent.startsWith('Opened project.'),null,{timeout:180000});assert.equal(await page.evaluate(()=>generationRequests.length),0);
  if(kind==='economic'){
   await page.locator('#open-lab-project').setInputFiles({name:'over-limit.json',mimeType:'application/json',buffer:Buffer.from('['+'0,'.repeat(MAX_LAB_PROJECT_VALUES-1)+'0]')});await page.waitForFunction(()=>document.querySelector('#lab-project-status').textContent.includes('2,000,000 values'),null,{timeout:60000});const retained=await save('after-refusal.bisect');assert.deepEqual(retained.project.assignments,expected.assignments);assert.deepEqual(retained.project.experiment.config.characters,expected.experiment.config.characters);
  }
  await page.locator('#district-view').click();await page.locator('#map').waitFor({state:'visible'});await page.locator('#map').screenshot({path:path.join(root,kind+'-map.png')});records.push({kind,states:expected.experiment.states.length,saved_bytes:saved.bytes,source_inputs_preserved:true,inert_open_and_reopen:true});
 }
 assert.deepEqual(errors,[]);assert.deepEqual(external,[]);if(packaged)for(const entry of served.values()){const relative=path.relative(lab,entry.path),current=path.join(source,relative);if(/\.(js|css)$/.test(relative)&&await fs.stat(current).then(s=>s.isFile(),()=>false))assert.equal(entry.sha256,sha(await fs.readFile(current)),relative+' staged source differs');}
 await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({status:'passed-national-character-browser',source_overlay:!packaged,package_sources_checked:packaged,records,import_limit_values:MAX_LAB_PROJECT_VALUES,byte_limit:MAX_PROJECT_BYTES,oversized_import_preserves_current_project:true,generation_requests:0,served_assets:[...served.values()],full_parity:false},null,2)+'\n');console.log(JSON.stringify({root,status:'passed-national-character-browser',records}));
}catch(error){throw error;}finally{await browser?.close();await new Promise(r=>server.close(r));}
