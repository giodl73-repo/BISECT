import {openWorkbenchProject} from './workbench_project_open.mjs';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
const {chromium}=createRequire(path.join(process.env.BISECT_PLAYWRIGHT_PACKAGES,'package.json'))('playwright');
const root=path.resolve('target/tui-library-compare-browser'),staticRoot=path.resolve('web/lab'),wasm=path.resolve('target/wasm32-unknown-unknown/release/bisect_wasm.wasm');await fs.mkdir(root,{recursive:true});
const server=createServer(async(req,res)=>{try{const url=new URL(req.url,'http://localhost'),file=url.pathname==='/bisect_wasm.wasm'?wasm:path.resolve(staticRoot,'.'+decodeURIComponent(url.pathname==='/'?'/workbench.html':url.pathname));if(file!==wasm&&!file.startsWith(staticRoot+path.sep))throw Error('Path rejected');res.setHeader('Content-Type',file.endsWith('.js')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':'text/html');res.end(await fs.readFile(file));}catch{res.writeHead(404);res.end();}});await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const origin='http://127.0.0.1:'+server.address().port;let context;
try{
 context=await chromium.launchPersistentContext(path.join(root,'profile-'+Date.now()),{headless:true,executablePath:'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-background-networking','--disable-extensions']});
 await context.route('**/*',route=>new URL(route.request().url()).origin===origin?route.continue():route.abort());
 const page=await context.newPage(),errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.addInitScript(()=>{Object.defineProperty(navigator,'hardwareConcurrency',{get:()=>2});window.requests=[];const Original=Worker;window.Worker=class extends Original{postMessage(message,...rest){if(message?.type==='run')requests.push(message.request.operation);return super.postMessage(message,...rest);}};});
 const open=async name=>{await openWorkbenchProject(page,name,{ready:({argument:__bisectOpenArgument2088,openedStatus:__bisectOpenedStatus2088})=>(name=>__bisectOpenedStatus2088==='Opened '+name)(__bisectOpenArgument2088),readyArg:path.basename(name)});};
 const save=async name=>{const pending=page.waitForEvent('download');await page.locator('#save-project').click();await(await pending).saveAs(path.join(root,name));return JSON.parse(await fs.readFile(path.join(root,name),'utf8'));};
 await page.goto(origin);await open(path.resolve('target/tui-library-compare/browser.bisect'));assert.equal(await page.evaluate(()=>requests.length),0);
 await page.locator('#tui-library-folder').setInputFiles(path.resolve('target/tui-library-compare/plans'));
 await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-tui-library').disabled);assert.equal(await page.locator('#tui-library-selection option').count(),2);
 await page.locator('#tui-library-selection').selectOption('plans/a');const before=await page.evaluate(()=>requests.length);await page.locator('#tui-library-compare').click();assert.equal(await page.locator('#operation').inputValue(),'compare-tui-plans');assert.equal(await page.evaluate(()=>requests.length),before);
 const handoff=await save('handoff.bisect'),fixture=JSON.parse(await fs.readFile('target/tui-library-compare/browser.bisect','utf8'));assert.deepEqual(handoff.files.tuiCompareInputs.a.sources['data/final_assignments.json'],fixture.files.tuiLibraryInputs.plans[0].assignments);assert.deepEqual(handoff.files.tuiCompareInputs.b,fixture.files.tuiCompareInputs.b);
 await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-tui-compare').disabled);const completed=await save('completed.bisect');assert.equal(completed.result.metrics.jaccard,1/3);assert.deepEqual(completed.result,JSON.parse(await fs.readFile('target/tui-library-compare/expected.json','utf8')));
 const count=await page.evaluate(()=>requests.length);await open(path.join(root,'completed.bisect'));assert.equal(await page.evaluate(()=>requests.length),count);assert(await page.locator('#download-tui-compare').isDisabled());const reopened=await save('reopened.bisect');assert.deepEqual(reopened.files.tuiCompareInputs,handoff.files.tuiCompareInputs);assert.deepEqual(reopened.files.tuiLibraryInputs,handoff.files.tuiLibraryInputs);
 await page.setViewportSize({width:390,height:844});assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1));assert.deepEqual(errors,[]);
 await fs.writeFile(root+'/evidence.json',JSON.stringify({inert_open:true,real_folder_assignment_read:true,explicit_handoff_no_engine_run:true,exact_u64_assignment_bytes:true,existing_plan_b_retained:true,native_metrics_equal:true,score:1/3,save_open_retains_library_and_comparison:true,imported_result_retired:true,mobile_width:390,page_errors:errors,published:false},null,2));console.log('PASS TUI library Compare browser: actual folder assignments, inert handoff, native 1/3 score, exact source Save/Open, retired imported exports and mobile layout');
}finally{await context?.close();await new Promise(resolve=>server.close(resolve));}
