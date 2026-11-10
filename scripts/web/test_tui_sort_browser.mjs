// Real Rust inspection through the browser, compared with native TUI discovery.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {nextTuiLibrarySort} from '../../web/lab/tui-library-files.js';
const packages=process.env.BISECT_PLAYWRIGHT_PACKAGES;assert.ok(packages);
const {chromium}=createRequire(path.join(packages,'package.json'))('playwright');
const root=path.resolve('target/tui-sort-browser');await fs.mkdir(root,{recursive:true});
const source=path.resolve('web/lab'),wasm=path.resolve('target/wasm32-unknown-unknown/release/bisect_wasm.wasm');
const server=createServer(async(req,res)=>{try{const name=new URL(req.url,'http://localhost').pathname,file=name==='/bisect_wasm.wasm'?wasm:path.resolve(source,'.'+(name==='/'?'/workbench.html':name));if(file!==wasm&&!file.startsWith(source+path.sep))throw new Error('Path refused');res.setHeader('Content-Type',file.endsWith('.js')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':'text/html');res.end(await fs.readFile(file));}catch{res.writeHead(404);res.end();}});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));const origin='http://127.0.0.1:'+server.address().port;let context;
try{
 context=await chromium.launchPersistentContext(path.join(root,'profile-'+Date.now()),{headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-background-networking','--disable-extensions']});
 await context.route('**/*',route=>new URL(route.request().url()).origin===origin?route.continue():route.abort());const page=await context.newPage(),errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.addInitScript(()=>{window.requests=[];const Original=Worker;window.Worker=class extends Original{postMessage(data,...rest){if(data.type==='run'){window.requests.push(data.request);if(window.holdSort){window.releaseSort=()=>super.postMessage(data,...rest);return;}}return super.postMessage(data,...rest);}};});
 await page.goto(origin+'/');await page.waitForFunction(()=>!document.querySelector('#execute').disabled);
 const project=JSON.parse(await fs.readFile('target/tui-library-wasm/library.bisect','utf8'));
 await page.locator('#open-project').setInputFiles(path.resolve('target/tui-library-wasm/library.bisect'));await page.waitForFunction(()=>document.querySelector('#result-status').textContent.startsWith('Imported result is unverified'));
 assert.equal(await page.evaluate(()=>window.requests.length),0);
 const save=async name=>{const wait=page.waitForEvent('download');await page.locator('#save-project').click();const file=path.join(root,name);await(await wait).saveAs(file);return JSON.parse(await fs.readFile(file,'utf8'));};
 // Sort is an explicit inspection even on an imported project; no drawing command.
 const sequence=[['label','desc'],['splits','asc'],['splits','desc'],['pp','asc'],['pp','desc'],['deviation','asc'],['label','asc']];
 const nativeApp=await fs.readFile('crates/bisect-tui/src/app.rs','utf8');assert(nativeApp.includes('(SortColumn::Pp, SortDir::Desc)'));assert(nativeApp.includes('SortColumn::Deviation'));
 for(const [index,[column,direction]]of sequence.entries()){
  if(index>0){await page.locator('#tui-library-selection').selectOption({index:1});}
  const position=index===0?0:1;
  if(index%2===0){await page.locator('h1').click();await page.keyboard.press('s');}else await page.locator('#tui-library-cycle-sort').click();
  await page.waitForFunction(()=>!document.querySelector('#download-tui-library').disabled);
  assert.equal(await page.locator('#tui-library-sort_column').inputValue(),column);assert.equal(await page.locator('#tui-library-sort_direction').inputValue(),direction);
  const native=JSON.parse(execFileSync('target/debug/examples/plan_library_reference.exe',[path.resolve('target/tui-library-wasm/native'),'',column,direction],{encoding:'utf8'}));
  const saved=await save('cycle-'+index+'.bisect');assert.deepEqual(saved.result.plans,native);assert.deepEqual(saved.files.tuiLibraryInputs.plans,project.files.tuiLibraryInputs.plans);
  assert.equal(await page.locator('#tui-library-selection').evaluate(el=>el.selectedIndex),position);assert.equal(saved.files.tuiLibraryInputs.settings.selected_directory,native[position].plan_dir);
  assert.equal(saved.result.execution_performed,false);assert.equal(await page.evaluate(()=>window.requests.length),index+1);
 }
 // Native default branch returns deviation-desc to label-asc too.
 assert.deepEqual(nextTuiLibrarySort('deviation','desc'),{sort_column:'label',sort_direction:'asc'});
 const count=await page.evaluate(()=>window.requests.length);await page.locator('#tui-library-filter').fill('');await page.locator('#tui-library-filter').press('s');assert.equal(await page.locator('#tui-library-filter').inputValue(),'s');assert.equal(await page.evaluate(()=>window.requests.length),count);await page.locator('#tui-library-filter').fill('');
 await page.evaluate(()=>window.holdSort=true);await page.locator('#tui-library-cycle-sort').click();assert.equal(await page.locator('#tui-library-cycle-sort').isDisabled(),true);assert.equal(await page.locator('#download-tui-library').isDisabled(),true);await page.locator('h1').click();await page.keyboard.press('s');assert.equal(await page.evaluate(()=>window.requests.length),count+1);
 await page.locator('#cancel').click();assert.equal(await page.locator('#download-tui-library').isDisabled(),true);await page.evaluate(()=>window.holdSort=false);await page.waitForFunction(()=>!document.querySelector('#execute').disabled);
 // Cancelled inspection has no rows; reconstruct with a fresh explicit inspection.
 await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-tui-library').disabled);
 // A later normal inspection must not inherit the cancelled sort's positional state.
 await page.locator('#tui-library-selection').selectOption({index:2});const directory=await page.locator('#tui-library-selection').inputValue();await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-tui-library').disabled);assert.equal(await page.locator('#tui-library-selection').inputValue(),directory);
 const final=await save('restored.bisect');const beforeOpen=await page.evaluate(()=>window.requests.length);await page.locator('#open-project').setInputFiles(path.join(root,'restored.bisect'));await page.waitForFunction(()=>document.querySelector('#result-status').textContent.startsWith('Imported result is unverified'));assert.equal(await page.locator('#tui-library-selection').inputValue(),final.files.tuiLibraryInputs.settings.selected_directory);assert.equal(await page.evaluate(()=>window.requests.length),beforeOpen);assert.equal(await page.locator('#download-tui-library').isDisabled(),true);
 assert((await page.evaluate(()=>window.requests)).every(request=>request.operation==='inspect-tui-plan-library'));assert.deepEqual(errors,[]);
 await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({native_sort_states:7,native_plan_rows_equal:true,unknown_metrics_and_stable_ties:true,positional_selection_matches_native:true,exact_sources_retained:true,inert_open:true,editing_and_busy_guard:true,cancel_retires_export_and_clears_pending_selection:true,only_native_inspection_dispatched:true},null,2));
 console.log('Native Home sort: seven real browser/native row comparisons, positional selection, stable ties/unknown metrics, exact source Save/Open and edit/busy/cancel guards passed.');
}finally{await context?.close();await new Promise(resolve=>server.close(resolve));}
