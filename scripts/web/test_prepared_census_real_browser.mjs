// Original-source browser parity; owned server/profile, no live-provider requests.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {createWorkbenchArchive,parseWorkbenchArchiveBytes} from '../../web/lab/workbench-archive.js';
const packageRoot=process.env.BISECT_PLAYWRIGHT_PACKAGES;if(!packageRoot)throw new Error('Set BISECT_PLAYWRIGHT_PACKAGES.');
const {chromium}=createRequire(path.join(packageRoot,'package.json'))('playwright');
const kind=process.argv.includes('--block')?'block':'block-group',units=kind==='block'?25649:792;
const root=path.resolve('target/prepared-census-real-browser/'+kind);await fs.mkdir(root,{recursive:true});
const source=JSON.parse(await fs.readFile('target/real-census-preparation/RI/'+kind+'/run.bisect-workbench','utf8'));
// This checkpoint is written only after full native binding/review/run equality.
const generated=JSON.parse(await fs.readFile('target/prepared-census-real/'+kind+'-output.json','utf8'));
const expected=generated.result;
source.files.preparedCensusTemplateInputs={enabled:true,use_display_geometry:false,...expected.template_source};
delete source.files.configSource;source.result=null;source.lastOperation=null;source.operation='prepare-census-engine-input';source.name='Original RI '+kind+' browser';
source.files.stateBatchInputs={year:'2020',states:['RI'],manifest:null};
const fixture=path.join(root,'sources.bisect-workbench');await fs.writeFile(fixture,createWorkbenchArchive(source));
const staticRoot=path.resolve('web/lab'),wasm=path.resolve('target/wasm32-unknown-unknown/release/bisect_wasm.wasm');
const server=createServer(async(req,res)=>{try{const pathname=new URL(req.url,'http://localhost').pathname,target=pathname==='/bisect_wasm.wasm'?wasm:path.resolve(staticRoot,'.'+decodeURIComponent(pathname==='/'?'/workbench.html':pathname));if(target!==wasm&&!target.startsWith(staticRoot+path.sep))throw new Error('Invalid path');res.setHeader('Content-Type',target.endsWith('.js')?'application/javascript':target.endsWith('.wasm')?'application/wasm':target.endsWith('.css')?'text/css':'text/html');res.end(await fs.readFile(target));}catch{res.writeHead(404);res.end();}});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));const url='http://127.0.0.1:'+server.address().port;
let context,page;
try{
 context=await chromium.launchPersistentContext(path.join(root,'profile-'+Date.now()),{headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-background-networking','--disable-extensions']});
 await context.route('**/*',route=>new URL(route.request().url()).origin===url?route.continue():route.abort());
 page=await context.newPage();const errors=[];page.on('pageerror',error=>errors.push(error.stack));
 await page.addInitScript(()=>{window.executions=[];const Original=Worker;window.Worker=class extends Original{postMessage(message,...rest){if(message?.type==='run')executions.push(message.request.operation);return super.postMessage(message,...rest);}};});
 await page.goto(url);await page.locator('#open-workbench-archive').setInputFiles(fixture);
 await page.waitForFunction(()=>document.querySelector('#project-name').value.startsWith('Original RI'),null,{timeout:120000});
 console.log('Source archive opened',await page.locator('#result-status').innerText());
 assert.equal(await page.evaluate(()=>executions.length),0);assert(await page.locator('#prepared-census-enabled').isChecked());
 await page.locator('#state-source-add-batch').click();await page.waitForFunction(()=>document.querySelector('#result-status').textContent.includes('sources queued'),null,{timeout:120000});assert.equal(await page.evaluate(()=>executions.length),0);
 await page.locator('#operation').selectOption('run-state-batch');await page.locator('#execute').click();
 await page.waitForFunction(()=>!document.querySelector('#execute').disabled,null,{timeout:1200000});
 assert.equal(await page.locator('#download-state-run-batch').isDisabled(),false,await page.locator('#result-status').innerText()+'\n'+await page.locator('#result-tables').innerText());
 assert((await page.locator('#result-tables').innerText()).includes('18446744073709551615'));
 const download=page.waitForEvent('download',{timeout:180000});await page.locator('#save-workbench-archive').click();const saved=path.join(root,'completed.bisect-workbench');await(await download).saveAs(saved);
 const completed=await parseWorkbenchArchiveBytes(new Uint8Array(await fs.readFile(saved)));assert.equal(completed.result.summary.succeeded,1);assert.deepEqual(completed.result.states[0].result,expected);assert.equal(completed.files.configSource,undefined);
 await page.locator('#state-run-batch-map-state').selectOption('RI');await page.locator('#state-run-batch-show-map').click();assert.equal(await page.locator('#state-run-batch-prepared-map svg path').count(),units);
 // A real Census block can be sub-pixel at whole-state extent. Verify its
 // supported keyboard inspection without relying on rounded mouse coordinates.
 const first=page.locator('#state-run-batch-prepared-map svg path').first();await first.focus();await first.press('Enter');assert.equal(await page.locator('#state-run-batch-prepared-map [role="status"]').innerText(),await first.getAttribute('aria-label'));
 await page.locator('#state-run-batch-prepared-map').screenshot({path:path.join(root,'state-map.png')});
 const before=await page.evaluate(()=>executions.length);await page.locator('#open-workbench-archive').setInputFiles(saved);await page.waitForFunction(()=>document.querySelector('#result-status').textContent.startsWith('Imported result is unverified'),null,{timeout:180000});assert.equal(await page.evaluate(()=>executions.length),before);assert(await page.locator('#download-state-run-batch').isDisabled());assert.equal(await page.locator('#state-run-batch-prepared-map svg path').count(),0);assert.deepEqual(errors,[]);
 await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({state:'RI',year:'2020',unit_kind:kind,units,original_source_worker_run_matches_native_compared_result:true,opaque_source_open_and_queue_no_execution:true,no_yaml_dependency:true,exact_requested_seed_visible:true,all_units_mapped:true,inert_completed_archive_reopen:true,wasm_sha256:JSON.parse(await fs.readFile('target/prepared-census-real/evidence.json','utf8')).wasm_sha256},null,2));
 console.log(`Original RI ${kind} browser: source-only queue, actual worker preparation/binding/build equals Native-compared result, ${units}-unit map, exact seed, source-preserving Save/Open and inert reopen passed.`);
}catch(error){if(page)console.log('Browser state',await page.evaluate(()=>({name:document.querySelector('#project-name')?.value,status:document.querySelector('#project-status')?.textContent,result:document.querySelector('#result-status')?.textContent,executeDisabled:document.querySelector('#execute')?.disabled})).catch(()=>null));throw error;}finally{await context?.close();await new Promise(resolve=>server.close(resolve));}
