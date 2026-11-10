import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {createProject} from '../../web/lab/project.js';
import {observedProjectOpen} from './fixtures/observed-project-open.mjs';
import {spawnSync} from 'node:child_process';
const option=name=>process.argv.find(arg=>arg.startsWith('--'+name+'='))?.slice(name.length+3);
const nodeRoot=option('reference');if(!nodeRoot)throw Error('Supply --reference=<passing source-start Node case root>.');
const reference=JSON.parse(await fs.readFile(path.join(nodeRoot,'evidence.json'),'utf8'));assert.equal(reference.status,'passed-ceremony-source-start');
const packageRoot=path.resolve(option('toolkit')??'dist/wasm-cffi-full-20261027/toolkit'),sourceRoot=path.resolve('web/lab');
const wasm=await fs.readFile(option('wasm')??path.join(nodeRoot,'engine.wasm')),tree=await fs.readFile(path.join(nodeRoot,'tree.json'));
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
assert.equal(hash(wasm),reference.sources.find(ref=>ref.path.endsWith('engine.wasm')).sha256);
const root=await fs.mkdtemp(path.resolve('target/ceremony-source-start/browser-')),served=new Map(),requests=[];
const server=createServer(async(req,res)=>{try{
 const url=new URL(req.url,'http://localhost').pathname;requests.push(url);let bytes,file;
 if(url==='/bisect_wasm.wasm')bytes=wasm;
 else{const relative=url==='/'?'workbench.html':decodeURIComponent(url.slice(1));file=path.resolve(sourceRoot,relative);if(!file.startsWith(sourceRoot+path.sep))throw Error('Path escaped');try{bytes=await fs.readFile(file);served.set(file,hash(bytes));}catch(error){if(error.code!=='ENOENT')throw error;file=path.resolve(packageRoot,relative);if(!file.startsWith(packageRoot+path.sep))throw error;bytes=await fs.readFile(file);}}
 const ext=path.extname(file??url);res.setHeader('Content-Type',['.js','.mjs'].includes(ext)?'application/javascript':ext==='.wasm'?'application/wasm':ext==='.css'?'text/css':ext==='.json'?'application/json':'text/html');res.setHeader('Cache-Control','no-store');res.end(bytes);
}catch{res.writeHead(404);res.end();}});await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const origin='http://127.0.0.1:'+server.address().port;
const {chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES,'package.json'))('playwright');let browser;
try{
 browser=await chromium.launch({headless:true,executablePath:process.env.BISECT_TEST_BROWSER??'C:/Program Files/Google/Chrome/Application/chrome.exe'});
 const context=await browser.newContext();await context.route('**/*',route=>new URL(route.request().url()).origin===origin?route.continue():route.abort());
 const page=await context.newPage(),errors=[];page.on('pageerror',error=>errors.push(error.message));
 await page.addInitScript(()=>{
  window.runs=[];window.replies=[];window.profileReads=[];const W=Worker;window.Worker=class extends W{constructor(...args){super(...args);this.addEventListener('message',({data})=>replies.push(data));}postMessage(data,...rest){if(data.type==='run')runs.push(structuredClone(data));return super.postMessage(data,...rest);}};
  const read=File.prototype.arrayBuffer;File.prototype.arrayBuffer=function(){profileReads.push(this.name);if(this.name==='slow-source.rules'){window.profileReadStarted=true;return new Promise(resolve=>window.releaseProfileRead=()=>resolve(new Uint8Array([7,8,9]).buffer));}return read.call(this);};
 });
 await page.goto(origin);await page.waitForFunction(()=>!document.querySelector('#execute').disabled);
 const open=async(file,expectedName)=>{if(typeof file!=='string'){const target=path.join(root,file.name);await fs.writeFile(target,file.buffer);file=target;}const before=await page.evaluate(()=>runs.length);await observedProjectOpen(page,file);await page.waitForFunction(name=>document.querySelector('#project-name').value===name&&!document.querySelector('#execute').disabled,expectedName);assert.equal(await page.evaluate(()=>runs.length),before,'Open must be inert');};
 const save=async name=>{const event=page.waitForEvent('download');await page.locator('#save-project').click();const file=path.join(root,name+'.bisect');await(await event).saveAs(file);return {file,value:JSON.parse(await fs.readFile(file,'utf8'))};};
 const run=async()=>{const before=await page.evaluate(()=>runs.length);await page.locator('#execute').click();await page.waitForFunction(n=>runs.length===n+1&&replies.some(reply=>reply.id===runs.at(-1).id&&['error','result'].includes(reply.type)),before);await page.waitForFunction(()=>!document.querySelector('#execute').disabled);return page.evaluate(()=>replies.find(reply=>reply.id===runs.at(-1).id&&['error','result'].includes(reply.type)));};
 const outcomes=[];let binarySaved;
 for(const item of reference.records){
  const title='Original profile '+item.name,project=createProject({name:title,operation:'start-ceremony-from-sources',files:{ceremonyTree:{name:'tree.json',base64:tree.toString('base64')}},constraints:[],result:null,lastOperation:null});
  await open({name:item.name+'.bisect',mimeType:'application/json',buffer:Buffer.from(JSON.stringify(project))},title);
  assert(await page.locator('#ceremony-start-source-fields').isVisible());assert(await page.locator('#ceremony-config-field').isHidden());
  const before=await page.evaluate(()=>runs.length),profile=await fs.readFile(path.join(nodeRoot,item.name,'profile'));
  await page.locator('#ceremony-start-profile').setInputFiles({name:item.name+'.rules',mimeType:'application/octet-stream',buffer:profile});await page.waitForFunction(name=>document.querySelector('#ceremony-start-profile-name').textContent.startsWith(name),item.name+'.rules');
  const arg=key=>item.native.arguments.find(value=>value.startsWith(key+'=')).slice(key.length+1),witnesses=[];for(let i=0;i<item.native.arguments.length;i++)if(item.native.arguments[i]==='--witness')witnesses.push(item.native.arguments[i+1]);
  await page.locator('#ceremony-start-genesis').fill(arg('--genesis-unix-seconds'));await page.locator('#ceremony-start-review').fill(arg('--review-seconds'));await page.locator('#ceremony-start-minimum').fill(arg('--minimum-witnesses'));await page.locator('#ceremony-start-witnesses').fill(witnesses.join('\n'));
  assert.equal(await page.evaluate(()=>runs.length),before,'File selection and edits must not start a ceremony');
  const reply=await run();
  if(item.native.exit_code===0){
   assert.equal(reply.type,'result',reply.error);const nativeText=await fs.readFile(path.join(nodeRoot,item.name,'transcript.json'),'utf8');assert.deepEqual(JSON.parse(reply.result.transcript_json),JSON.parse(nativeText));
   assert(reply.result.transcript_json.includes('"genesis_unix_seconds":'+arg('--genesis-unix-seconds')));assert(reply.result.transcript_json.includes('"minimum_review_seconds":'+arg('--review-seconds')));
   assert.equal(reply.result.source_preparation.rule_profile_sha256,hash(profile));assert.equal(reply.result.source_preparation.rule_profile_bytes,profile.length);
   assert((await page.locator('#result-tables').textContent()).includes('Recorded original ceremony sources'));assert((await page.locator('#result-tables').textContent()).includes(arg('--review-seconds')));
   const event=page.waitForEvent('download');await page.locator('#download-ceremony').click();const transcriptFile=path.join(root,item.name+'.json');await(await event).saveAs(transcriptFile);assert.equal(await fs.readFile(transcriptFile,'utf8'),reply.result.transcript_json);
  }else{assert.equal(reply.type,'error');assert.equal(reply.failure.request_retention,'retained');assert(await page.locator('#download-ceremony').isDisabled());}
  const saved=await save(item.name);assert.equal(saved.value.files.ceremonyStartInputs.rule_profile.base64,profile.toString('base64'));assert.equal(saved.value.files.ceremonyStartInputs.settings.review_seconds,arg('--review-seconds'));
  assert.deepEqual(saved.value.files.ceremonyStartInputs.settings.witness_receipts,witnesses);
  await open(saved.file,title);assert(await page.locator('#download-ceremony').isDisabled());if(item.name==='binary')binarySaved=saved;
  outcomes.push({name:item.name,native_exit:item.native.exit_code,worker_type:reply.type,profile_sha256:hash(profile),inert_save_open:true});
 }
 assert(binarySaved);
 // Native-corrected replay after a failed witness request requires a new click.
 await page.locator('#ceremony-start-witnesses').fill('A\nB');const corrected=await run();assert.equal(corrected.type,'result',corrected.error);assert.equal(corrected.result.status,'open');
 const correctedFile=path.join(root,'corrected-native.json'),correctedNative=spawnSync(path.join(nodeRoot,'native-reference.exe'),['ceremony','start','--tree',path.join(nodeRoot,'tree.json'),'--rule-profile',path.join(nodeRoot,'trimmed-duplicate-witness','profile'),'--genesis-unix-seconds=1000','--review-seconds=10','--minimum-witnesses=2','--witness','A','--witness','B','--out',correctedFile],{encoding:'utf8',windowsHide:true});assert.equal(correctedNative.status,0,correctedNative.stderr);assert.deepEqual(JSON.parse(corrected.result.transcript_json),JSON.parse(await fs.readFile(correctedFile,'utf8')));
 const count=await page.evaluate(()=>runs.length),oldName=await page.locator('#ceremony-start-profile-name').textContent();
 const slow=async()=>{await page.evaluate(()=>window.profileReadStarted=false);await page.locator('#ceremony-start-profile').setInputFiles({name:'slow-source.rules',mimeType:'application/octet-stream',buffer:Buffer.from([7,8,9])});await page.waitForFunction(()=>profileReadStarted);};
 await slow();await page.locator('#execute').click();assert.equal(await page.evaluate(()=>runs.length),count,'Pending profile read blocks Run');assert.match(await page.locator('#result-status').textContent(),/Wait for the original profile read/);
 await page.locator('#ceremony-start-cancel-read').click();await page.evaluate(()=>releaseProfileRead());await page.waitForFunction(()=>document.querySelector('#ceremony-start-cancel-read').disabled);assert.equal(await page.locator('#ceremony-start-profile-name').textContent(),oldName);assert.equal(await page.evaluate(()=>runs.length),count);
 await slow();await open(binarySaved.file,binarySaved.value.name);await page.evaluate(()=>releaseProfileRead());await page.waitForFunction(()=>document.querySelector('#ceremony-start-cancel-read').disabled);assert.match(await page.locator('#ceremony-start-profile-name').textContent(),/^binary.rules/);assert.equal(await page.evaluate(()=>runs.length),count);
 await page.locator('#ceremony-start-profile').setInputFiles({name:'oversize.rules',mimeType:'application/octet-stream',buffer:Buffer.alloc(3*1024*1024+1)});await page.waitForFunction(()=>document.querySelector('#result-status').textContent.includes('at most 3 MiB'));assert.equal(await page.evaluate(()=>profileReads.includes('oversize.rules')),false);assert.match(await page.locator('#ceremony-start-profile-name').textContent(),/^binary.rules/);
 // An intermediate minus sign remains editable and cannot execute old settings.
 await page.locator('#ceremony-start-genesis').fill('-');await page.locator('#execute').click();assert.equal(await page.evaluate(()=>runs.length),count);await page.locator('#ceremony-start-genesis').focus();await page.keyboard.press('End');assert.equal(await page.locator('#ceremony-start-genesis').evaluate(node=>node.selectionStart),1);await page.locator('#ceremony-start-genesis').pressSequentially('1000');assert.equal(await page.locator('#ceremony-start-genesis').inputValue(),'-1000');const negative=await save('negative-draft');assert.equal(negative.value.files.ceremonyStartInputs.settings.genesis_unix_seconds,'-1000');
 const opaque=structuredClone(binarySaved.value);opaque.name='Opaque receipt preservation';opaque.files.ceremonyStartInputs.settings.witness_receipts=[' A\r','B',''];await open({name:'opaque-receipts.bisect',mimeType:'application/json',buffer:Buffer.from(JSON.stringify(opaque))},opaque.name);await page.locator('#ceremony-start-genesis').fill('-2000');const opaqueSaved=await save('opaque-receipts-edited');assert.deepEqual(opaqueSaved.value.files.ceremonyStartInputs.settings.witness_receipts,[' A\r','B','']);assert.equal(await page.evaluate(()=>runs.length),count);
 const bom=structuredClone(binarySaved.value);bom.name='Native BOM tree refusal';bom.result=null;bom.lastOperation=null;bom.files.ceremonyTree.base64=Buffer.concat([Buffer.from([239,187,191]),tree]).toString('base64');await open({name:'bom-tree.bisect',mimeType:'application/json',buffer:Buffer.from(JSON.stringify(bom))},bom.name);const bomReply=await run();assert.equal(bomReply.type,'error');assert.match(bomReply.error,/UTF-8 BOM/);assert.equal(JSON.parse(bomReply.failure.request_json).input.tree_json.charCodeAt(0),0xfeff);assert(await page.locator('#download-ceremony').isDisabled());
 assert.deepEqual(errors,[]);assert(!requests.some(url=>url.startsWith('/c-metis/')));
 await page.screenshot({path:path.join(root,'source-start-form.png'),fullPage:true});
 const sources=[];for(const [file,sha256]of served){assert.equal(hash(await fs.readFile(file)),sha256);sources.push({path:path.relative(process.cwd(),file).replaceAll('\\','/'),sha256});if(process.argv.includes('--require-package'))assert.equal(hash(await fs.readFile(path.join(packageRoot,path.basename(file)==='workbench.html'?'index.html':path.basename(file)))),sha256);}
 const evidence={status:'passed-ceremony-source-start-browser',actual_workbench_worker:true,browser:browser.version(),engine_sha256:hash(wasm),reference:{path:path.join(nodeRoot,'evidence.json'),sha256:hash(await fs.readFile(path.join(nodeRoot,'evidence.json')))},sources,outcomes,explicit_runs:await page.evaluate(()=>runs.length),corrected_replay:true,bom_bytes_preserved_and_refused:true,late_read_cancel_and_open_retired:true,oversize_refused_before_read:true,editable_negative_draft:true,opaque_receipts_preserved_on_unrelated_edit:true,test_sha256:hash(await fs.readFile('scripts/web/test_ceremony_source_start_browser.mjs')),scope:'Original-profile Start through actual UI/worker: native transcript values/failures, i64/u64 exact digits, empty/binary bytes, inert Save/Open, corrected replay, native BOM refusal and source-read guards. No publication/authentication or full parity.'};
 await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(evidence,null,2)+'\n');console.log(JSON.stringify({status:evidence.status,root}));
}finally{await browser?.close();await new Promise(resolve=>server.close(resolve));}
