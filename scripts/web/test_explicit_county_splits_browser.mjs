import {openWorkbenchProject} from './workbench_project_open.mjs';
// Own test server/profile only; never attach to the user's running browser.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
const packageRoot=process.env.BISECT_PLAYWRIGHT_PACKAGES;if(!packageRoot)throw new Error('Set BISECT_PLAYWRIGHT_PACKAGES to an installed Playwright package root.');
const {chromium}=createRequire(path.join(packageRoot,'package.json'))('playwright');
const root=path.resolve(process.env.BISECT_COUNTY_SPLITS_BROWSER_OUTPUT||'target/explicit-county-splits-browser');await fs.mkdir(root,{recursive:true});
const nativeEvidence=process.env.BISECT_COUNTY_SPLITS_NATIVE_EVIDENCE,contextTool=path.resolve(process.env.BISECT_CONTEXT_REWRITER||'target/explicit-county-context-host/release/examples/rewrite_context.exe'),hash=b=>createHash('sha256').update(b).digest('hex');
const packageIndex=process.argv.indexOf('--package'),staticRoot=path.resolve(packageIndex>=0?process.argv[packageIndex+1]:'web/lab'),wasm=packageIndex>=0?path.join(staticRoot,'bisect_wasm.wasm'):path.resolve('target/wasm32-unknown-unknown/release/bisect_wasm.wasm');
const wasmBytes=await fs.readFile(wasm);
const server=createServer(async(req,res)=>{try{const url=new URL(req.url,'http://localhost'),target=url.pathname==='/bisect_wasm.wasm'?wasm:path.resolve(staticRoot,'.'+decodeURIComponent(url.pathname==='/'?'/workbench.html':url.pathname));if(target!==wasm&&!target.startsWith(staticRoot+path.sep))throw new Error('Path rejected');res.setHeader('Content-Type',target.endsWith('.js')?'application/javascript':target.endsWith('.wasm')?'application/wasm':target.endsWith('.css')?'text/css':'text/html');res.end(target===wasm?wasmBytes:await fs.readFile(target));}catch{res.writeHead(404);res.end();}});await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const url='http://127.0.0.1:'+server.address().port,profile=path.join(root,'profile-'+Date.now());let context;
async function launch(){const c=await chromium.launchPersistentContext(profile,{headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-background-networking','--disable-extensions']});await c.route('**/*',r=>new URL(r.request().url()).origin===url?r.continue():r.abort());return c;}



import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {explicitCountySplitsCase} from './fixtures/explicit-county-splits-case.mjs';
import {createProject} from '../../web/lab/project.js';
import {execFileSync} from 'node:child_process';
const engine=await instantiateEngine(wasmBytes),fixture=await explicitCountySplitsCase(engine),source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
const geography=JSON.stringify({county_names:{'44001':'<img src=x onerror=globalThis.countyInjected=true>'}}),result=engine.execute({operation:'analyze-plan-splits',document_json:fixture.plan,context_json:fixture.context,geography_json:geography});
await fs.writeFile(root+'/project.bisect',JSON.stringify(createProject({name:'Explicit county analysis',operation:'analyze-plan-splits',files:{auditPlanText:source('custom.rplan',fixture.plan),auditContextText:source('custom.rctx',fixture.context),splitsGeography:source('geography.json',geography)},constraints:[],result,lastOperation:'analyze-plan-splits'})));
assert.match(result.report.county_identity_sha256,/^[0-9a-f]{64}$/);
try{
 context=await launch();const page=await context.newPage(),errors=[];page.on('pageerror',error=>errors.push(error.message));
 await page.addInitScript(()=>{window.requests=[];const OriginalWorker=Worker;window.Worker=class extends OriginalWorker{postMessage(value,...rest){if(value.type==='run')requests.push(structuredClone(value.request));return super.postMessage(value,...rest);}};});
 await page.goto(url);await page.waitForFunction(()=>!document.querySelector('#execute').disabled);
 let baseline=0,nativeReportHash=null;
 if(nativeEvidence){
  const nativeProject=path.resolve(nativeEvidence,'project.bisect'),proof=JSON.parse(await fs.readFile(path.join(nativeEvidence,'evidence.json'),'utf8'));
  assert.equal(proof.wasm_sha256,hash(wasmBytes));assert.equal(proof.status,'passed-explicit-county-native-and-wasm');
  await openWorkbenchProject(page,nativeProject);assert.equal(await page.evaluate(()=>requests.length),0);assert(await page.locator('#download-splits').isDisabled());
  await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-splits').disabled);
  const pending=page.waitForEvent('download');await page.locator('#download-splits').click();await(await pending).saveAs(root+'/cli-matched-splits.json');
  const expected=await fs.readFile(path.join(nativeEvidence,'native-report.json'),'utf8'),actual=await fs.readFile(root+'/cli-matched-splits.json','utf8');
  assert.equal(expected,actual+'\n');assert.equal(proof.desktop_cases.imported.stdout_sha256,hash(expected));
  const [observed]=await page.evaluate(()=>requests);assert.equal(observed.document_json,fixture.plan);assert.equal(observed.context_json,fixture.context);
  await fs.writeFile(root+'/cli-matched-request.json',JSON.stringify(observed));baseline=1;nativeReportHash=hash(expected);
 }
 await openWorkbenchProject(page,root+'/project.bisect');assert.equal(await page.locator('#operation').inputValue(),'analyze-plan-splits');assert.equal(await page.evaluate(()=>requests.length),baseline);assert(await page.locator('#auditContextText-file').isVisible());assert(await page.locator('#download-splits').isDisabled());
 await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-splits').disabled);assert.equal(await page.evaluate(()=>requests.length),baseline+1);assert(await page.getByRole('columnheader',{name:'County ID',exact:true}).isVisible());assert((await page.locator('#result-tables').textContent()).includes('<img src=x onerror=globalThis.countyInjected=true>'));assert.equal(await page.evaluate(()=>globalThis.countyInjected),undefined);
 const download=page.waitForEvent('download');await page.locator('#download-splits').click();await(await download).saveAs(root+'/splits.json');assert.equal(await fs.readFile(root+'/splits.json','utf8'),result.report_json);
 await page.locator('#retain-analysis').click();await page.waitForFunction(()=>document.querySelector('#analysis-collection-status').textContent.includes('splits.json'));
 const beforeEdit=await page.evaluate(()=>requests.length),bad=JSON.parse(fixture.context);bad.subdivisions.county_ids[0]=null;await fs.writeFile(root+'/missing-input.json',JSON.stringify(bad));const incomplete=execFileSync(contextTool,[root+'/missing-input.json'],{encoding:'utf8',windowsHide:true});await page.locator('#auditContextText-file').setInputFiles({name:'incomplete.rctx',mimeType:'application/json',buffer:Buffer.from(incomplete)});await page.waitForFunction(()=>document.querySelector('#auditContextText-name').textContent==='incomplete.rctx');assert.equal(await page.evaluate(()=>requests.length),beforeEdit);assert(await page.locator('#download-splits').isDisabled());await page.locator('#execute').click();await page.waitForFunction(()=>document.querySelector('#result-status').textContent.toLowerCase().includes('missing county'));assert(await page.locator('#download-splits').isDisabled());
 await page.locator('#auditContextText-file').setInputFiles({name:'repaired.rctx',mimeType:'application/json',buffer:Buffer.from(fixture.context)});await page.waitForFunction(()=>document.querySelector('#auditContextText-name').textContent==='repaired.rctx');assert(await page.locator('#download-splits').isDisabled());await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-splits').disabled);
 const save=page.waitForEvent('download');await page.locator('#save-project').click();await(await save).saveAs(root+'/completed.bisect');const saved=JSON.parse(await fs.readFile(root+'/completed.bisect','utf8'));assert.equal(saved.result.context_source_sha256,result.context_source_sha256);assert.equal(saved.result.report.county_identity_sha256,result.report.county_identity_sha256);
 const beforeOpen=await page.evaluate(()=>requests.length);await openWorkbenchProject(page,root+'/completed.bisect',{ready:({argument:__bisectOpenArgument6873,openedStatus:__bisectOpenedStatus6873})=>(()=>__bisectOpenedStatus6873.startsWith('Opened completed.bisect'))(__bisectOpenArgument6873),readyArg:undefined});assert.equal(await page.evaluate(()=>requests.length),beforeOpen);assert(await page.locator('#download-splits').isDisabled());
 await page.setViewportSize({width:390,height:844});assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1));await page.screenshot({path:root+'/mobile.png',fullPage:true});assert.deepEqual(errors,[]);
 const engineRuns=await page.evaluate(()=>requests.filter(request=>request.operation==='run-prepared-engine'&&!request.input.inspect_only).length);assert.equal(engineRuns,0);await fs.writeFile(root+'/evidence.json',JSON.stringify({status:'passed-explicit-county-browser',wasm_sha256:hash(wasmBytes),native_stdout_sha256:nativeReportHash,exact_desktop_cli_download:nativeEvidence?true:null,explicit_context_visible:true,inert_open_and_edits:true,opaque_id_table:true,safe_text_names:true,exact_report_download:true,analysis_retained:true,missing_ids_refused:true,repair_explicit:true,context_and_county_map_hashes_saved:true,analysis_partition_executions:0,mobile_width:390,page_errors:errors},null,2));console.log('PASS explicit county browser: matching RCTX, opaque IDs, safe names, exact download/retention, missing-ID refusal, repair and inert Open; no partitions.');
}catch(error){const page=context?.pages().at(-1);if(page)console.error('Browser status:',await page.locator('#result-status').textContent());throw error;}finally{await context?.close();await new Promise(resolve=>server.close(resolve));}
