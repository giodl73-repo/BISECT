import {parseWorkbenchArchiveBytes} from '../../web/lab/workbench-archive.js';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createServer} from 'node:http';
import {createRequire} from 'node:module';
const packages=process.env.BISECT_PLAYWRIGHT_PACKAGES;
if(!packages)throw new Error('Set BISECT_PLAYWRIGHT_PACKAGES.');
const {chromium}=createRequire(path.join(packages,'package.json'))('playwright');
const national=process.argv.includes('--original-national');const root=path.resolve(national?'target/original-national-map-browser':'target/national-map-package-browser'),staticRoot=path.resolve('web/lab'),wasm=path.resolve('target/wasm32-unknown-unknown/release/bisect_wasm.wasm');
await fs.mkdir(root,{recursive:true});
const server=createServer(async(req,res)=>{
 try{
  const url=new URL(req.url,'http://localhost'),file=url.pathname==='/bisect_wasm.wasm'?wasm:path.resolve(staticRoot,'.'+decodeURIComponent(url.pathname==='/'?'/workbench.html':url.pathname));
  if(file!==wasm&&!file.startsWith(staticRoot+path.sep))throw new Error('Outside root');
  res.setHeader('Content-Type',file.endsWith('.js')?'application/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':'text/html');res.end(await fs.readFile(file));
 }catch{res.writeHead(404);res.end();}
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const origin='http://127.0.0.1:'+server.address().port;let context;
try{
 context=await chromium.launchPersistentContext(path.join(root,'profile-'+Date.now()),{headless:true,executablePath:process.env.BISECT_TEST_BROWSER||'C:/Program Files/Google/Chrome/Application/chrome.exe',args:['--no-first-run','--disable-background-networking','--disable-extensions']});
 await context.route('**/*',r=>new URL(r.request().url()).origin===origin?r.continue():r.abort());
 const page=await context.newPage(),errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.addInitScript(()=>{window.executions=[];const Original=Worker;window.Worker=class extends Original{postMessage(message,...rest){if(message?.type==='run')executions.push(message.request);return super.postMessage(message,...rest);}};});
 page.setDefaultTimeout(180000);await page.goto(origin);
 const inputs=national?'target/original-national-map-stream':'target/national-map-package-capacity',expected=JSON.parse(await fs.readFile(inputs+'/result.json','utf8')),before=await page.evaluate(()=>executions.length);
 await page.locator('#open-workbench-archive').setInputFiles(path.resolve(inputs+(national?'/national.bisect-workbench':'/backup.bisect-workbench')));
 await page.waitForFunction(()=>document.querySelector('#open-workbench-archive').value===''&&document.querySelector('#result-status').textContent.startsWith('Imported result is unverified'));
 assert.equal(await page.evaluate(()=>executions.length),before);assert.equal(await page.locator('#download-native-map-png').isDisabled(),true);
 await page.locator('#national-map-batch-source').setInputFiles(path.resolve(inputs+'/package.json'));
 await page.waitForFunction(()=>document.querySelector('#national-map-batch-status').textContent.includes('package.json'));
 assert.equal(await page.evaluate(()=>executions.length),before);
 await page.locator('#execute').click();await page.waitForFunction(()=>!document.querySelector('#download-native-map-png').disabled,null,{timeout:1800000});
 const image=page.locator('#native-map-preview img');await image.evaluate(img=>img.decode());assert.deepEqual(await image.evaluate(img=>[img.naturalWidth,img.naturalHeight]),[expected.width,expected.height]);
 let waiting=page.waitForEvent('download');waiting.catch(()=>{});await page.locator('#download-native-map-png').click();await(await waiting).saveAs(path.join(root,'national.png'));assert.deepEqual(await fs.readFile(path.join(root,'national.png')),Buffer.from(expected.png_base64,'base64'));
 waiting=page.waitForEvent('download');waiting.catch(()=>{});await page.locator('#save-workbench-archive').click();await(await waiting).saveAs(path.join(root,'saved.bisect-workbench'));
 const saved=await parseWorkbenchArchiveBytes(new Uint8Array(await fs.readFile(path.join(root,'saved.bisect-workbench'))));assert.deepEqual(saved.result,expected);
 const hash=b=>createHash('sha256').update(b).digest('hex');assert.equal(saved.files.nationalMapBatchInputs.source.name,'package.json');assert.equal(hash(Buffer.from(saved.files.nationalMapBatchInputs.source.base64,'base64')),hash(await fs.readFile(inputs+'/package.json')));
 await page.screenshot({path:path.join(root,'national.png-preview.png'),fullPage:true});
 const after=await page.evaluate(()=>executions.length);await page.locator('#open-workbench-archive').setInputFiles(path.join(root,'saved.bisect-workbench'));await page.waitForFunction(()=>document.querySelector('#open-workbench-archive').value===''&&document.querySelector('#result-status').textContent.startsWith('Imported result is unverified'));assert.equal(await page.evaluate(()=>executions.length),after);assert.equal(await page.locator('#download-native-map-png').isDisabled(),true);assert.deepEqual(errors,[]);
 await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({exact_worker_result:true,exact_source_sha256:true,exact_native_png:true,decoded_dimensions:true,inert_archive_open:true,package_bytes:(await fs.stat(inputs+'/package.json')).size,scope:national?'All 50 original prepared state tract geometries, 435 native-verified districts, explicit per-state algorithms; browser rendering, exact exports and inert source-preserving backup. Not all engine options/resolutions or legal certification.':'Original AL/AZ plans/coordinates with trailing JSON whitespace test package byte capacity and actual browser source/archive paths. Not a 50-state capacity proof.'},null,2));
 console.log((national?'Original 50-state':'National >64 MiB package')+' browser: exact native result/PNG, selected source digest, compressed backup and inert reopening passed.');
}finally{await context?.close();await new Promise(resolve=>server.close(resolve));}
