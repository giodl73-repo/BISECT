import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
const packages=process.env.BISECT_PLAYWRIGHT_PACKAGES;if(!packages)throw Error('Set BISECT_PLAYWRIGHT_PACKAGES');
const {chromium}=createRequire(path.join(packages,'package.json'))('playwright');
const root=path.resolve('target/aggregate-completion-browser'),source=path.resolve('web/lab');await fs.mkdir(root,{recursive:true});
const skipped=JSON.parse(await fs.readFile('target/aggregate-completion/skip-summary.bisect','utf8'));
const forced=JSON.parse(await fs.readFile('target/aggregate-completion/force-summary.bisect','utf8'));
const runRoot=path.join(root,'run-'+Date.now());
const folders={outputs:path.join(runRoot,'national'),invalid:path.join(runRoot,'states-skipped'),valid:path.join(runRoot,'states-valid')};
const normalized=result=>({...result,summaries:[...result.summaries].sort((a,b)=>a.analyzer.localeCompare(b.analyzer))});
await fs.mkdir(folders.outputs,{recursive:true});await fs.writeFile(path.join(folders.outputs,'us_summary.json'),'');
for(const key of ['invalid','valid']){
 const dir=path.join(folders[key],'alabama/analysis');await fs.mkdir(dir,{recursive:true});
 for(const [type,s]of Object.entries(skipped.files.nativeAggregateInputs.reports.alabama))await fs.writeFile(path.join(dir,type+'.json'),key==='invalid'&&type==='summary'?Buffer.alloc(3*1024*1024+1,255):Buffer.from(s.base64,'base64'));
}
const wasm=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm');
const server=createServer(async(req,res)=>{
 try{
  const name=decodeURIComponent(new URL(req.url,'http://localhost').pathname);if(name.includes('..'))throw Error('Path refused');
  const file=name==='/'?'/workbench.html':name;
  res.setHeader('Content-Type',file.endsWith('.js')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':'text/html');
  res.end(file==='/bisect_wasm.wasm'?wasm:await fs.readFile(source+file));
 }catch{res.writeHead(404);res.end();}
});
await new Promise(r=>server.listen(0,'127.0.0.1',r));const origin='http://127.0.0.1:'+server.address().port;let context;
try{
 context=await chromium.launchPersistentContext(path.join(root,'profile-'+Date.now()),{
  headless:true,executablePath:'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-background-networking','--disable-extensions']
 });await context.route('**/*',r=>new URL(r.request().url()).origin===origin?r.continue():r.abort());
 const page=await context.newPage(),errors=[];page.on('pageerror',e=>errors.push(e.stack));
 await page.addInitScript(()=>{
  window.runs=0;window.sourceReads={};const Original=Worker;
  window.Worker=class extends Original{postMessage(m,...rest){if(m?.type==='run')runs++;return super.postMessage(m,...rest);}};
  const read=File.prototype.arrayBuffer;
  File.prototype.arrayBuffer=function(){sourceReads[this.name]=(sourceReads[this.name]||0)+1;return read.call(this);};
 });await page.goto(origin);
 const save=async name=>{
  const wait=page.waitForEvent('download');await page.locator('#save-project').click();await(await wait).saveAs(path.join(root,name));
  return JSON.parse(await fs.readFile(path.join(root,name),'utf8'));
 };
 await page.locator('#open-project').setInputFiles(path.resolve('target/aggregate-completion/skip-summary.bisect'));
 await page.waitForFunction(()=>document.querySelector('#operation').value==='aggregate-native-analysis'&&!document.querySelector('#execute').disabled);
 assert.equal(await page.evaluate(()=>runs),0);
 await page.locator('#native-aggregate-outputs').setInputFiles(folders.outputs);
 assert((await page.locator('#native-aggregate-output-names').innerText()).includes('us_summary.json'));
 assert.equal(await page.evaluate(()=>sourceReads['us_summary.json']||0),0);
 await page.locator('#native-aggregate-folder').setInputFiles(folders.invalid);
 await page.waitForFunction(()=>!document.querySelector('#native-aggregate-reload').disabled&&document.querySelector('#native-aggregate-output-names').textContent.includes('Deferred sources: summary'));
 assert.equal(await page.evaluate(()=>sourceReads['summary.json']||0),0);
 await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-native-aggregate').disabled);
 let packet=await save('skip.bisect');assert.deepEqual(normalized(packet.result),normalized(skipped.result));assert.deepEqual(packet.files.nativeAggregateInputs.deferred_types,['summary']);
 assert((await page.locator('#result-tables').innerText()).includes('skipped-existing'));
 await page.locator('#native-aggregate-force').check();
 const beforeForce=await page.evaluate(()=>runs);await page.locator('#execute').click();
 await page.waitForFunction(()=>document.querySelector('#result-status').textContent.includes('Reload or reselect'));
 assert.equal(await page.evaluate(()=>runs),beforeForce);assert(await page.locator('#download-native-aggregate').isDisabled());
 packet=await save('force-pending.bisect');assert.equal(packet.files.nativeAggregateInputs.completion.force,true);
 await page.locator('#native-aggregate-folder').setInputFiles(folders.valid);
 await page.waitForFunction(()=>!document.querySelector('#native-aggregate-reload').disabled&&!document.querySelector('#native-aggregate-output-names').textContent.includes('Deferred sources'));
 await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-native-aggregate').disabled);
 packet=await save('forced.bisect');assert.deepEqual(normalized(packet.result),normalized(forced.result));assert.equal(packet.files.nativeAggregateInputs.completion.force,true);
 const beforeOpen=await page.evaluate(()=>runs);await page.locator('#open-project').setInputFiles(path.join(root,'forced.bisect'));
 await page.waitForFunction(()=>document.querySelector('#result-status').textContent.startsWith('Imported result is unverified'));
 assert.equal(await page.evaluate(()=>runs),beforeOpen);assert(await page.locator('#native-aggregate-force').isChecked());
 assert.equal(await page.locator('#native-aggregate-outputs').inputValue(),'');assert.equal(await page.locator('#native-aggregate-folder').inputValue(),'');
 await page.locator('#native-aggregate-year').selectOption('2010');const beforeYear=await page.evaluate(()=>runs);
 await page.locator('#execute').click();await page.waitForFunction(()=>document.querySelector('#result-status').textContent.includes('match the selected year'));
 assert.equal(await page.evaluate(()=>runs),beforeYear);
 await page.locator('#native-aggregate-year').selectOption('2020');await page.locator('#native-aggregate-force').uncheck();
 await fs.writeFile(path.join(folders.outputs,'us_compactness.json'),'');
 await page.locator('#native-aggregate-outputs').setInputFiles(folders.outputs);
 await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#execute').disabled&&document.querySelector('#result-tables').textContent.includes('skipped-existing'));
 packet=await save('skip-all.bisect');assert.deepEqual(packet.result.files,{});assert(await page.locator('#download-native-aggregate').isDisabled());
 await page.locator('#native-aggregate-clear').click();packet=await save('cleared-sources.bisect');
 assert.deepEqual(packet.files.nativeAggregateInputs.reports,{});assert.equal(packet.files.nativeAggregateInputs.completion.existing_outputs.length,2);
 assert.equal(await page.locator('#native-aggregate-folder').inputValue(),'');assert(await page.locator('#native-aggregate-reload').isDisabled());
 await page.locator('#native-aggregate-output-clear').click();assert((await page.locator('#native-aggregate-output-names').innerText()).includes('No existing-output'));
 assert.equal(await page.evaluate(()=>sourceReads['us_summary.json']||0),0);assert.equal(await page.evaluate(()=>sourceReads['us_compactness.json']||0),0);
 assert.deepEqual(errors,[]);
 await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({wasm_sha256:createHash('sha256').update(wasm).digest('hex'),real_worker:true,native_skip_and_force_outputs_match:true,zero_output_body_reads:true,oversized_invalid_utf8_skipped_without_reading:true,deferred_force_blocks_run_until_explicit_reload:true,source_and_observation_save_and_inert_open:true,year_mismatch_refused_before_worker:true,all_skipped_no_output_zip:true},null,2)+'\n');
 console.log('Aggregate completion browser: metadata-only observation, unread oversized source skip, explicit force/reload, native output matches, inert source/observation Save/Open and all-skipped export gate passed.');
}finally{await context?.close();await new Promise(r=>server.close(r));}
