// Own test server/profile only; never attach to the user's running browser.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
import {parseWorkbenchArchiveBytes} from '../../web/lab/workbench-archive.js';

const packageRoot=process.env.BISECT_PLAYWRIGHT_PACKAGES;
if(!packageRoot)throw new Error('Set BISECT_PLAYWRIGHT_PACKAGES to an installed Playwright package root.');
const {chromium}=createRequire(path.join(packageRoot,'package.json'))('playwright');
const root=path.resolve('target/topology-geometry-browser');await fs.mkdir(root,{recursive:true});
const oracle=JSON.parse(await fs.readFile('target/stream-state-source-batch/native-oracles/CA-52-topology.json','utf8'));
await fs.writeFile(path.join(root,'plan.rplan'),oracle.practitioner_export.document_json);
const geometry=await fs.readFile('target/original-state-acquisition/CA/topology-geometry.json');
await fs.writeFile(path.join(root,'geometry.json'),geometry);
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
 assert.equal(await page.locator('#tract-preparation-geometry-profile option[value="display-topology"]').count(),1);
 await page.locator('#operation').selectOption('render-native-district-map');
 await page.locator('#native-map-plan').setInputFiles(path.join(root,'plan.rplan'));
 await page.locator('#native-map-geometry').setInputFiles(path.join(root,'geometry.json'));
 await page.waitForFunction(()=>document.querySelector('#native-map-sources').textContent.includes('geometry: geometry.json'));
 await page.locator('#native-map-method').selectOption('batch');await page.locator('#native-map-dpi').selectOption('72');
 assert.equal(await page.evaluate(()=>executions.length),0);
 await page.locator('#execute').click();
 await page.waitForFunction(()=>!document.querySelector('#execute').disabled,null,{timeout:600000});
 assert.equal(await page.locator('#download-native-map-png').isDisabled(),false,await page.locator('#result-status').innerText());
 await page.waitForFunction(()=>{const image=document.querySelector('#native-map-preview img');return image?.complete&&image.naturalWidth>0;});
 for(const kind of ['svg','png']){
  const waiting=page.waitForEvent('download');await page.locator('#download-native-map-'+kind).click();await(await waiting).saveAs(path.join(root,'map.'+kind));
  assert.deepEqual(await fs.readFile(path.join(root,'map.'+kind)),Buffer.from(oracle.workflow.files_base64['maps/districts.'+kind],'base64'));
 }
 await page.locator('#native-map-preview').screenshot({path:path.join(root,'california-map.png')});
 const waiting=page.waitForEvent('download');await page.locator('#save-workbench-archive').click();await(await waiting).saveAs(path.join(root,'map.bisect-workbench'));
 const saved=await parseWorkbenchArchiveBytes(new Uint8Array(await fs.readFile(path.join(root,'map.bisect-workbench'))));
 assert.deepEqual(Buffer.from(saved.files.nativeMapInputs.geometry.base64,'base64'),geometry);
 assert.equal(saved.files.nativeMapInputs.dissolve_method,'batch');
 const before=await page.evaluate(()=>executions.length);
 await page.locator('#open-workbench-archive').setInputFiles(path.join(root,'map.bisect-workbench'));
 await page.waitForFunction(()=>document.querySelector('#result-status').textContent.startsWith('Imported result is unverified'),null,{timeout:120000});
 assert.equal(await page.evaluate(()=>executions.length),before);assert.equal(await page.locator('#download-native-map-png').isDisabled(),true);
 assert.deepEqual(errors,[]);
 console.log('Original CA topology browser: actual WASM worker map, exact native SVG/PNG downloads, rendered preview, source-preserving archive and inert reopen passed.');
}finally{await context?.close();await new Promise(resolve=>server.close(resolve));}
