// Original-state sources in an owned browser/profile/server; no user's tabs or folders.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {parseWorkbenchArchiveBytes} from '../../web/lab/workbench-archive.js';
const packageRoot=process.env.BISECT_PLAYWRIGHT_PACKAGES;
if(!packageRoot)throw new Error('Set BISECT_PLAYWRIGHT_PACKAGES to an installed Playwright package root.');
const {chromium}=createRequire(path.join(packageRoot,'package.json'))('playwright');
const stateArgument=process.argv.find(value=>value.startsWith('--state='));
if(stateArgument&&!/^--state=[A-Z]{2}$/.test(stateArgument))throw new Error('Use an uppercase two-letter original state code.');
const state=stateArgument?stateArgument.slice(8):'CA',root=path.resolve(stateArgument?'target/full-state-browser/'+state:'target/full-ca-browser');await fs.mkdir(root,{recursive:true});
const inputs=stateArgument?JSON.parse(await fs.readFile(path.join(root,'browser-inputs.json'),'utf8')):{state:'CA',units:9129,districts:52};assert.equal(inputs.state,state);
const sourcePath=path.resolve(stateArgument?path.join(root,'source.bisect-workbench'):'target/stream-source-projects/CA-topology.bisect-workbench');
const staticRoot=path.resolve('web/lab'),wasm=path.resolve('target/wasm32-unknown-unknown/release/bisect_wasm.wasm');
const server=createServer(async(req,res)=>{try{
 const url=new URL(req.url,'http://localhost'),target=url.pathname==='/bisect_wasm.wasm'?wasm:path.resolve(staticRoot,'.'+decodeURIComponent(url.pathname==='/'?'/workbench.html':url.pathname));
 if(target!==wasm&&!target.startsWith(staticRoot+path.sep))throw new Error('Path rejected');
 res.setHeader('Content-Type',target.endsWith('.js')?'application/javascript':target.endsWith('.wasm')?'application/wasm':target.endsWith('.css')?'text/css':'text/html');res.end(await fs.readFile(target));
}catch{res.writeHead(404);res.end();}});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const url='http://127.0.0.1:'+server.address().port;let context;
try{
 context=await chromium.launchPersistentContext(path.join(root,'profile-'+Date.now()),{headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-background-networking','--disable-extensions']});
 await context.route('**/*',route=>new URL(route.request().url()).origin===url?route.continue():route.abort());
 const page=await context.newPage(),errors=[];page.on('pageerror',error=>errors.push(error.message));
 await page.addInitScript(()=>{window.executions=[];const Original=Worker;window.Worker=class extends Original{postMessage(message,...rest){if(message?.type==='run')executions.push(message.request.operation);return super.postMessage(message,...rest);}};});
 await page.goto(url);
 await page.locator('#open-workbench-archive').setInputFiles(sourcePath);
 await page.waitForFunction(name=>document.querySelector('#project-status').textContent.startsWith('Opened workbench archive '+name+'.'),path.basename(sourcePath),{timeout:120000});
 assert.equal(await page.evaluate(()=>executions.length),0);assert.equal(await page.locator('#tract-preparation-geometry-profile').inputValue(),'display-topology');assert.equal(await page.locator('#tui-run-seed').inputValue(),'18446744073709551615');
 console.log('Original '+state+' source archive opened inertly; browser preparing Census sources.');
 await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#execute').disabled,null,{timeout:900000});
 assert.equal(await page.locator('#use-tract-graph').isDisabled(),false,await page.locator('#result-status').innerText());
 const download=async(button,name)=>{const waiting=page.waitForEvent('download',{timeout:120000});await page.locator(button).click();const filename=path.join(root,name);await(await waiting).saveAs(filename);return filename;};
 for(const [kind,filename] of [['graph','prepared-graph.json'],['geometry','prepared-geometry.json']]){
  const actual=await download('#download-tract-'+kind,'browser-'+filename);assert.deepEqual(await fs.readFile(actual),await fs.readFile(path.join(root,filename)));
 }
 await page.locator('#use-tract-graph').click();assert.equal(await page.locator('#operation').inputValue(),'run-tui-configured');
 const save=async name=>{
  const waiting=page.waitForEvent('download',{timeout:120000});await page.locator('#save-workbench-archive').click();const filename=path.join(root,name);await(await waiting).saveAs(filename);return parseWorkbenchArchiveBytes(new Uint8Array(await fs.readFile(filename)));
 };
 const handoff=await save('handoff.bisect-workbench');assert.equal(handoff.result.geometry_json,undefined);assert.equal(handoff.result.geometry_project_field,'planGeometry');
 assert.deepEqual(Buffer.from(handoff.files.planGeometry.base64,'base64'),await fs.readFile(path.join(root,'prepared-geometry.json')));
 const source=await parseWorkbenchArchiveBytes(new Uint8Array(await fs.readFile(sourcePath)));
 assert.deepEqual(handoff.files.tigerInputs,source.files.tigerInputs);assert.deepEqual(handoff.files.pl94Inputs,source.files.pl94Inputs);
 for(const kind of ['coi','demographics'])assert.deepEqual(handoff.files.configRunInputs[kind],source.files.configRunInputs[kind]);
 for(const key of ['seats_per_district','total_seats','native_build','capture_rounds'])assert.deepEqual(handoff.files.configRunSettings[key],source.files.configRunSettings[key]);
 console.log('Original '+state+' preparation/reference source bytes and source-preserving handoff passed. Browser running the configured engine/workflow.');
 await page.locator('#tui-run-workflow-fields').evaluate(element=>element.open=true);
 for(const kind of ['analysis','map','report'])await page.locator('#tui-workflow-'+kind).check();
 await page.locator('#tui-workflow-dpi').selectOption('72');await page.locator('#tui-workflow-method').selectOption('batch');
 await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#execute').disabled,null,{timeout:900000});
 assert.equal(await page.locator('#download-tui-workflow').isDisabled(),false,await page.locator('#result-status').innerText());
 const saved=await save('run.bisect-workbench'),native=JSON.parse(await fs.readFile('target/stream-state-source-batch/native-oracles/'+state+'-'+inputs.districts+'-topology.json','utf8'));
 assert.deepEqual(saved.result,native);assert.deepEqual(saved.files.tigerInputs,source.files.tigerInputs);assert.deepEqual(saved.files.pl94Inputs,source.files.pl94Inputs);
 assert.equal(await page.locator('#plan-map path').count(),inputs.units,await page.locator('#plan-map').innerText());await page.locator('#plan-map svg').screenshot({path:path.join(root,stateArgument?state.toLowerCase()+'-plan.png':'california-plan.png')});
 await download('#download-tui-run','native-run.zip');await download('#download-tui-workflow','native-workflow.zip');
 const before=await page.evaluate(()=>executions.length);await page.locator('#open-workbench-archive').setInputFiles(path.join(root,'run.bisect-workbench'));
 await page.waitForFunction(()=>document.querySelector('#result-status').textContent.startsWith('Imported result is unverified'),null,{timeout:120000});
 assert.equal(await page.evaluate(()=>executions.length),before);assert.equal(await page.locator('#download-tui-workflow').isDisabled(),true);assert.equal(await page.locator('#tui-run-seed').inputValue(),'18446744073709551615');
 assert.deepEqual(errors,[]);
 console.log('Original '+state+' full browser: fresh WASM Census preparation, exact reference graph/geometry, source-preserving backups, full native Run/workflow equality, '+inputs.units+'-unit map, ZIP exports and inert reopen passed.');
}finally{await context?.close();await new Promise(resolve=>server.close(resolve));}
