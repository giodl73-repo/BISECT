// Own test server/profile only; never attach to the user's running browser.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {manifestReplayRequest} from '../../web/lab/engine-replay-files.js';
const packageRoot=process.env.BISECT_PLAYWRIGHT_PACKAGES;if(!packageRoot)throw new Error('Set BISECT_PLAYWRIGHT_PACKAGES to an installed Playwright package root.');
const {chromium}=createRequire(path.join(packageRoot,'package.json'))('playwright');
const root=path.resolve('target/coi-multiscale-browser');await fs.mkdir(root,{recursive:true});
const staticRoot=path.resolve('web/lab'),wasm=path.resolve('target/wasm32-unknown-unknown/release/bisect_wasm.wasm');
const wasmBytes=await fs.readFile(wasm),identity={sha256:createHash('sha256').update(wasmBytes).digest('hex'),byte_length:wasmBytes.length};
let nativeMatches=0;
async function assertNativeReceipt(name,saved,fixture){
 const request={...manifestReplayRequest(saved.files.packageVerificationInputs,saved.files.engineReplayInputs),runtime_artifact:identity},requestPath=path.join(root,name+'-request.json');
 await fs.writeFile(requestPath,JSON.stringify(request));
 const nativeText=execFileSync('target/release/examples/execute_tool_request.exe',[requestPath],{encoding:'utf8',maxBuffer:64*1024*1024,windowsHide:true});
 await fs.writeFile(path.join(root,name+'-native.json'),nativeText);
 const native=JSON.parse(nativeText);
 assert.deepEqual(native,{...fixture.result,binary_check:{...fixture.result.binary_check,loaded_wasm:identity}});
 assert.deepEqual(saved.result,native);nativeMatches++;
}
const server=createServer(async(req,res)=>{try{const url=new URL(req.url,'http://localhost'),target=url.pathname==='/bisect_wasm.wasm'?wasm:path.resolve(staticRoot,'.'+decodeURIComponent(url.pathname==='/'?'/workbench.html':url.pathname));if(target!==wasm&&!target.startsWith(staticRoot+path.sep))throw new Error('Path rejected');res.setHeader('Content-Type',target.endsWith('.js')?'application/javascript':target.endsWith('.wasm')?'application/wasm':target.endsWith('.css')?'text/css':'text/html');res.end(await fs.readFile(target));}catch{res.writeHead(404);res.end();}});await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const url='http://127.0.0.1:'+server.address().port,profile=path.join(root,'profile-'+Date.now());let context;
async function launch(){const c=await chromium.launchPersistentContext(profile,{headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-background-networking','--disable-extensions']});await c.route('**/*',r=>new URL(r.request().url()).origin===url?r.continue():r.abort());return c;}




try {
 context=await launch();const page=await context.newPage(),errors=[];page.on('pageerror',e=>errors.push(e.message));await page.goto(url);
 const save=async name=>{const wait=page.waitForEvent('download');await page.locator('#save-project').click();const d=await wait,file=path.join(root,name);await d.saveAs(file);return JSON.parse(await fs.readFile(file,'utf8'));};
 for(const name of ['engine-geographic','bg-tract-true-0']) {
  const fixture=path.resolve('target/coi-multiscale-wasm/'+name+'.bisect'),expected=JSON.parse(await fs.readFile(fixture,'utf8'));
  await page.locator('#open-project').setInputFiles(fixture);await page.waitForFunction(()=>document.querySelector('#result-status').textContent.startsWith('Imported result is unverified'));
  assert.equal(await page.locator('#download-engine-replay').isDisabled(),true);
  await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-engine-replay').disabled);
  const saved=await save(name+'-fresh.bisect');await assertNativeReceipt(name+'-fresh',saved,expected);assert.deepEqual(saved.files.engineReplayInputs,expected.files.engineReplayInputs);assert.equal(saved.result.binding.auxiliary_bindings.coi,'verified');
  const bad=JSON.parse(await fs.readFile('target/coi-multiscale-wasm/'+name+'-changed-weight.bisect','utf8')),runfile=path.join(root,name+'-changed-run.json');await fs.writeFile(runfile,Buffer.from(bad.files.engineReplayInputs.run.base64,'base64'));
  await page.locator('#engine-replay-run').setInputFiles(runfile);await page.waitForFunction(()=>document.querySelector('#engine-replay-run-name').textContent.endsWith('-changed-run.json'));assert.equal(await page.locator('#download-engine-replay').isDisabled(),true);
  await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-engine-replay').disabled);const failed=await save(name+'-failed.bisect');await assertNativeReceipt(name+'-failed',failed,bad);assert.equal(failed.result.execution_performed,false);assert.equal(failed.result.replay,null);assert.equal(failed.files.engineReplayInputs.run.base64,bad.files.engineReplayInputs.run.base64);
 }
 assert.equal(nativeMatches,4);await fs.writeFile(path.join(root,'native-comparison-summary.json'),JSON.stringify({nativeMatches,identity,browser:context.browser().version()},null,2));assert.deepEqual(errors,[]);await page.screenshot({path:path.join(root,'coi-correspondence-preview.png'),fullPage:true});console.log('COI correspondence browser: actual worker full receipt equality for engine and adaptive BG/tract artifacts; inert Open, explicit fresh replay, selected-source change invalidating downloads, changed COI rejection before execution and exact saved source retention passed.');
}finally{await context?.close();await new Promise(resolve=>server.close(resolve));}
