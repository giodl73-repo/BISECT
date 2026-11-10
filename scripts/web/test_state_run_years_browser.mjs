import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
const packages=process.env.BISECT_PLAYWRIGHT_PACKAGES;if(!packages)throw Error('Set BISECT_PLAYWRIGHT_PACKAGES');
const {chromium}=createRequire(path.join(packages,'package.json'))('playwright');
const root=path.resolve('target/state-run-years-browser'),source=path.resolve('web/lab');await fs.mkdir(root,{recursive:true});
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
  window.requests=[];window.held=0;window.stateWorkers=[];const Original=Worker;
  window.Worker=class extends Original{
   constructor(...args){super(...args);this.operations=[];this.closedByOwner=false;stateWorkers.push(this);}
   postMessage(m,...rest){
    if(m?.type==='run'){requests.push(m.request);this.operations.push(m.request.operation);}
    if(m?.request?.operation==='run-prepared-state-job'&&m.request.input.scope.year===window.holdYear){held++;return;}
    return super.postMessage(m,...rest);
   }
   terminate(){this.closedByOwner=true;return super.terminate();}
  };
 });await page.goto(origin);
 const save=async name=>{
  const wait=page.waitForEvent('download');await page.locator('#save-project').click();await(await wait).saveAs(path.join(root,name));
  return JSON.parse(await fs.readFile(path.join(root,name),'utf8'));
 };
 const original=JSON.parse(await fs.readFile('target/state-run-years/browser.bisect','utf8'));
 await page.locator('#open-project').setInputFiles(path.resolve('target/state-run-years/browser.bisect'));
 await page.waitForFunction(name=>document.querySelector('#project-name').value===name&&!document.querySelector('#execute').disabled,original.name);
 assert(await page.locator('#state-run-batch-all-years').isChecked());assert.equal(await page.evaluate(()=>requests.length),0);
 assert(await page.locator('#download-state-run-batch').isDisabled());assert(await page.locator('#state-run-batch-show-map').isDisabled());
 const run=async()=>{await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-state-run-batch').disabled);};
 await run();const fresh=await save('fresh.bisect');
 assert.equal(fresh.result.schema_version,'bisect-state-run-years-v1');assert.equal(fresh.result.state_summary.succeeded,3);
 assert.deepEqual(fresh.result.years.map(r=>r.year),['2020','2010','2000']);
 for(const row of fresh.result.years)assert.deepEqual(row.result.states,original.result.years.find(r=>r.year===row.year).result.states);
 assert.deepEqual(fresh.files.stateRunBatchInputs,original.files.stateRunBatchInputs);
 const beforeMaps=await page.evaluate(()=>requests.length);
 for(const year of ['2020','2010','2000']){
  await page.locator('#state-run-batch-view-year').selectOption(year);await page.locator('#state-run-batch-show-map').click();
  assert.equal(await page.locator('#state-run-batch-prepared-map svg path').count(),16);
  assert((await page.locator('#state-run-batch-prepared-map').innerText()).includes(year));
 }
 assert.equal(await page.evaluate(()=>requests.length),beforeMaps);
 const zipWait=page.waitForEvent('download');await page.locator('#download-state-run-batch').click();await(await zipWait).saveAs(path.join(root,'all-years.zip'));
 const beforeOpen=await page.evaluate(()=>requests.length);await page.locator('#open-project').setInputFiles(path.join(root,'fresh.bisect'));
 await page.waitForFunction(()=>document.querySelector('#result-status').textContent.startsWith('Imported result is unverified'));
 assert.equal(await page.evaluate(()=>requests.length),beforeOpen);assert.equal(await page.locator('#state-run-batch-prepared-map svg path').count(),0);
 assert(await page.locator('#state-run-batch-resume').isDisabled());assert(!await page.locator('#state-run-batch-verify-saved').isDisabled());
 const beforeVerify=await page.evaluate(()=>requests.length);
 await page.locator('#state-run-batch-verify-saved').click();await page.waitForFunction(()=>!document.querySelector('#execute').disabled&&!document.querySelector('#download-state-run-batch').disabled);
 const verifiedComplete=await save('verified-complete.bisect');assert.equal(verifiedComplete.result.state_summary.succeeded,3);
 assert.deepEqual(await page.evaluate(start=>requests.slice(start).filter(r=>r.operation==='run-prepared-state-job').map(r=>r.input.scope.year),beforeVerify),['2020','2010','2000']);
 await page.evaluate(()=>{window.holdYear='2010';window.held=0;});await page.locator('#execute').click();await page.waitForFunction(()=>held>0);
 assert(await page.locator('#state-run-batch-all-years').isDisabled());await page.locator('#cancel').click();
 await page.waitForFunction(()=>!document.querySelector('#execute').disabled);const cancelled=await save('cancelled.bisect');
 assert.equal(cancelled.result.status,'cancelled');assert.deepEqual(cancelled.result.years.map(r=>r.year),['2020','2010']);assert.equal(cancelled.result.state_summary.succeeded,1);
 assert(!await page.locator('#state-run-batch-resume').isDisabled());assert(!await page.locator('#download-state-run-batch').isDisabled());
 await page.evaluate(()=>window.holdYear=null);
 const beforeResume=await page.evaluate(()=>requests.length);
 await page.locator('#state-run-batch-resume').click();
 await page.waitForFunction(()=>!document.querySelector('#execute').disabled&&!document.querySelector('#download-state-run-batch').disabled);
 const resumed=await save('resumed.bisect');assert.equal(resumed.result.state_summary.succeeded,3);
 assert.deepEqual(resumed.result.years[0].result.states,cancelled.result.years[0].result.states);
 assert.deepEqual(await page.evaluate(start=>requests.slice(start).filter(r=>r.operation==='run-prepared-state-job').map(r=>r.input.scope.year),beforeResume),['2010','2000']);
 assert(await page.locator('#state-run-batch-resume').isDisabled());
 const beforeReopen=await page.evaluate(()=>requests.length);
 await page.locator('#open-project').setInputFiles(path.join(root,'cancelled.bisect'));
 await page.waitForFunction(()=>document.querySelector('#result-status').textContent.startsWith('Imported result is unverified'));
 assert.equal(await page.evaluate(()=>requests.length),beforeReopen);assert(await page.locator('#state-run-batch-resume').isDisabled());
 await page.locator('#state-run-batch-verify-saved').click();await page.waitForFunction(()=>!document.querySelector('#execute').disabled&&!document.querySelector('#state-run-batch-resume').disabled);
 const recovered=await save('verified-cancelled.bisect');assert.equal(recovered.result.state_summary.succeeded,1);
 assert.deepEqual(await page.evaluate(start=>requests.slice(start).filter(r=>r.operation==='run-prepared-state-job').map(r=>r.input.scope.year),beforeReopen),['2020']);
 const beforeRecoveredResume=await page.evaluate(()=>requests.length);await page.locator('#state-run-batch-resume').click();await page.waitForFunction(()=>!document.querySelector('#execute').disabled&&!document.querySelector('#download-state-run-batch').disabled);
 assert.deepEqual(await page.evaluate(start=>requests.slice(start).filter(r=>r.operation==='run-prepared-state-job').map(r=>r.input.scope.year),beforeRecoveredResume),['2010','2000']);
 await page.locator('#state-run-batch-all-years').uncheck();assert(await page.locator('#download-state-run-batch').isDisabled());
 await run();const single=await save('single.bisect');assert.equal(single.result.schema_version,'bisect-state-run-batch-v1');assert.equal(single.result.selection.year,'2020');
 assert.equal(Object.hasOwn(single.files.stateRunBatchInputs,'all_years'),false);
 const missing=structuredClone(original);missing.name='SYNTHETIC missing 2010 Run';
 const jobs=JSON.parse(Buffer.from(missing.files.stateRunBatchInputs.jobs.base64,'base64'));missing.files.stateRunBatchInputs.jobs.base64=Buffer.from(JSON.stringify(jobs.filter(j=>j.scope.year!=='2010'))).toString('base64');
 await fs.writeFile(path.join(root,'missing.bisect'),JSON.stringify(missing));await page.locator('#open-project').setInputFiles(path.join(root,'missing.bisect'));
 await page.waitForFunction(name=>document.querySelector('#project-name').value===name,missing.name);await run();
 const failed=await save('missing-result.bisect');assert.deepEqual(failed.result.years.map(r=>r.status),['completed','failed','completed']);assert.equal(failed.result.exit_code,1);
 await page.locator('#state-run-batch-view-year').selectOption('2010');assert(await page.locator('#state-run-batch-show-map').isDisabled());
 await page.locator('#state-run-batch-view-year').selectOption('2000');await page.locator('#state-run-batch-show-map').click();assert.equal(await page.locator('#state-run-batch-prepared-map svg path').count(),16);
 const workflow=JSON.parse(await fs.readFile('target/state-run-years-workflow/browser.bisect','utf8'));
 const beforeWorkflow=await page.evaluate(()=>requests.length);await page.locator('#open-project').setInputFiles(path.resolve('target/state-run-years-workflow/browser.bisect'));
 await page.waitForFunction(name=>document.querySelector('#project-name').value===name,workflow.name);assert.equal(await page.evaluate(()=>requests.length),beforeWorkflow);
 assert(await page.locator('#preview-commission').isDisabled());await run();
 const followed=await save('workflows.bisect');
 for(const row of followed.result.years)assert.deepEqual(row.result.states,workflow.result.years.find(r=>r.year===row.year).result.states);
 assert.equal(await page.locator('#generated-report-select option').count(),3);
 const reportKey=await page.locator('#generated-report-select option').evaluateAll(options=>options.find(o=>o.value.startsWith('2000/')).value);
 await page.locator('#generated-report-select').selectOption(reportKey);await page.locator('#preview-commission').click();
 await page.waitForFunction(()=>!document.querySelector('#print-commission').disabled);
 assert((await page.frameLocator('#commission-preview-host iframe').locator('body').innerText()).includes('2000'));
 await page.locator('#close-commission-preview').click();await page.locator('#state-run-batch-view-year').selectOption('2000');
 await page.locator('#state-run-batch-show-map').click();await page.waitForFunction(()=>document.querySelector('#state-run-batch-map-preview').naturalWidth>0);
 assert(!await page.locator('#state-run-batch-map-preview').isHidden());
 await page.locator('#state-run-batch-fields').screenshot({path:path.join(root,'year-controls.png')});
 await page.evaluate(()=>{
  const read=File.prototype.arrayBuffer;
  File.prototype.arrayBuffer=function(){if(this.name==='slow-years.json'){window.slowYearsDone=false;return new Promise(resolve=>{window.releaseYears=async()=>{const bytes=await read.call(this);resolve(bytes);window.slowYearsDone=true;};});}return read.call(this);};
 });
 const slowBytes=Buffer.from(original.files.stateRunBatchInputs.jobs.base64,'base64');
 await page.locator('#state-run-batch-jobs').setInputFiles({name:'slow-years.json',mimeType:'application/json',buffer:slowBytes});
 await page.waitForFunction(()=>!!window.releaseYears);assert(await page.locator('#execute').isDisabled());assert(await page.locator('#state-run-batch-all-years').isDisabled());
 const beforeSlow=await page.evaluate(()=>requests.length);await page.evaluate(()=>document.querySelector('#execute').onclick());assert.equal(await page.evaluate(()=>requests.length),beforeSlow);
 await page.locator('#state-run-batch-clear').click();await page.evaluate(()=>releaseYears());await page.waitForFunction(()=>slowYearsDone);
 const cleared=await save('cleared-pending.bisect');assert.equal(cleared.files.stateRunBatchInputs.jobs,null);
 await page.evaluate(()=>{window.releaseYears=null;});await page.locator('#state-run-batch-jobs').setInputFiles({name:'slow-years.json',mimeType:'application/json',buffer:slowBytes});await page.waitForFunction(()=>!!window.releaseYears);
 const beforeRetire=await page.evaluate(()=>requests.length);await page.locator('#open-project').setInputFiles(path.resolve('target/state-run-years-workflow/browser.bisect'));
 await page.waitForFunction(()=>document.querySelector('#result-status').textContent.startsWith('Imported result is unverified'));
 assert(!await page.locator('#execute').isDisabled());await page.evaluate(()=>releaseYears());await page.waitForFunction(()=>slowYearsDone);
 const restored=await save('restored-pending.bisect');assert.deepEqual(restored.files.stateRunBatchInputs,workflow.files.stateRunBatchInputs);assert.equal(await page.evaluate(()=>requests.length),beforeRetire);
 assert(await page.evaluate(()=>stateWorkers.filter(w=>w.operations.includes('run-prepared-state-job')).every(w=>w.closedByOwner)));
 assert.deepEqual(errors,[]);await page.screenshot({path:path.join(root,'years-preview.png'),fullPage:false});
 await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({wasm_sha256:createHash('sha256').update(wasm).digest('hex'),real_workers:true,synthetic_scope:true,three_year_native_verified_states:true,three_native_analysis_map_report_workflows:true,safe_year_scoped_report_preview:true,per_year_maps_without_execution:true,cancellation_retains_first_year:true,session_resume_only_unfinished_years:true,saved_full_replay:true,saved_cancelled_replay_then_resume:true,missing_year_continues:true,all_year_toggle_retires_exports:true,single_year_path_unchanged:true,pending_job_read_blocks_execution:true,clear_and_open_retire_pending_reads:true,source_project_and_inert_open:true,workers_closed:true},null,2)+'\n');
 console.log('Multi-year browser: three native-verified RI years, per-year maps/ZIP, failure continuation, cancellation, inert Open, scope freshness and single-year switching passed.');
}finally{await context?.close();await new Promise(r=>server.close(r));}
